// Allow module inception - this is a common Rust pattern for protocol clients
#![allow(clippy::module_name_repetitions)]

use crate::error::{AgwError, AgwResult};
use crate::plan::Plan;
use tracing::{debug, error, info, warn};

/// Result of a single task execution
#[derive(Debug, Clone, PartialEq)]
pub struct TaskResult {
    /// Task number that was executed
    pub task_number: u32,
    /// Standard output from the command
    pub stdout: String,
    /// Standard error from the command
    pub stderr: String,
    /// Exit code (0 = success)
    pub exit_code: i32,
    /// Whether execution was successful (exit code 0)
    pub success: bool,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Result of entire plan execution
#[derive(Debug, Clone, PartialEq)]
pub struct PlanResult {
    /// Job ID that was executed
    pub job_id: String,
    /// Plan ID
    pub plan_id: String,
    /// Results from each task that was executed
    pub task_results: Vec<TaskResult>,
    /// Whether all tasks succeeded
    pub success: bool,
}

impl TaskResult {
    /// Create a new task result
    #[must_use]
    pub fn new(task_number: u32, stdout: String, stderr: String, exit_code: i32) -> Self {
        Self {
            task_number,
            stdout,
            stderr,
            exit_code,
            success: exit_code == 0,
            execution_time_ms: 0,
        }
    }
}

impl PlanResult {
    /// Create a new plan result
    #[must_use]
    pub fn new(job_id: String, plan_id: String, task_results: Vec<TaskResult>) -> Self {
        let success = task_results.iter().all(|r| r.success);
        Self {
            job_id,
            plan_id,
            task_results,
            success,
        }
    }

    /// Combine stdout from all tasks
    ///
    /// Task outputs already contain trailing newlines from command execution,
    /// so this simply concatenates them without additional separators to avoid
    /// creating double newlines between tasks.
    #[must_use]
    pub fn combined_stdout(&self) -> String {
        self.task_results
            .iter()
            .map(|r| r.stdout.as_str())
            .collect::<String>()
    }

    /// Combine stderr from all tasks
    ///
    /// Task outputs already contain trailing newlines from command execution,
    /// so this simply concatenates them without additional separators to avoid
    /// creating double newlines between tasks.
    #[must_use]
    pub fn combined_stderr(&self) -> String {
        self.task_results
            .iter()
            .map(|r| r.stderr.as_str())
            .collect::<String>()
    }
}

/// Execute an entire plan sequentially
///
/// # Errors
///
/// Returns an error if:
/// - Command spawning fails
/// - IO operations fail while reading/writing stdout/stderr
/// - Timeout is exceeded
/// - Process cannot be killed after timeout
///
/// # Panics
///
/// This function will not panic under normal conditions. The unwrap at line 111
/// is safe because `task_results` is guaranteed to be non-empty when we check success.
///
/// Note: This function will halt on first failure and return partial results
pub async fn execute_plan(job_id: &str, plan: &Plan) -> AgwResult<PlanResult> {
    info!(
        "Executing plan {} (job {}) with {} tasks",
        plan.plan_id,
        job_id,
        plan.tasks.len()
    );

    let mut task_results = Vec::new();
    let mut previous_outputs: std::collections::HashMap<u32, String> =
        std::collections::HashMap::new();

    for task in &plan.tasks {
        info!("Executing task {}: {}", task.task_number, task.command);

        // Get input from previous task if specified
        let input = task
            .input_from_task
            .and_then(|task_num| previous_outputs.get(&task_num).cloned());

        match execute_task(
            &task.command,
            &task.args,
            input.as_deref(),
            task.timeout_secs,
            task.task_number,
        )
        .await
        {
            Ok(result) => {
                // Store stdout for potential use by later tasks
                previous_outputs.insert(task.task_number, result.stdout.clone());

                let success = result.success;
                task_results.push(result);

                // Halt on first failure
                if !success {
                    warn!(
                        "Task {} failed with exit code {}, halting plan execution",
                        task.task_number,
                        task_results.last().unwrap().exit_code
                    );
                    break;
                }
            }
            Err(e) => {
                error!("Task {} execution failed: {e}", task.task_number);
                return Err(e);
            }
        }
    }

    let plan_result = PlanResult::new(job_id.to_string(), plan.plan_id.clone(), task_results);

    info!(
        "Plan {} completed: {} tasks executed, success={}",
        plan.plan_id,
        plan_result.task_results.len(),
        plan_result.success
    );

    Ok(plan_result)
}

/// Execute a single task as a subprocess
///
/// # Errors
///
/// Returns an error if:
/// - Command spawning fails
/// - IO operations fail while reading stdout/stderr
/// - Timeout is exceeded
/// - Process cannot be killed after timeout
/// Execute a single task as a subprocess
///
/// # Errors
///
/// Returns an error if:
/// - Command spawning fails
/// - IO operations fail while reading stdout/stderr
/// - Timeout is exceeded
/// - Process cannot be killed after timeout
pub async fn execute_task(
    command: &str,
    args: &[String],
    stdin_input: Option<&str>,
    timeout_secs: Option<u32>,
    task_number: u32,
) -> AgwResult<TaskResult> {
    debug!("Command: {} with args: {:?}", command, args);

    // Validate command is not empty
    if command.is_empty() {
        return Err(AgwError::Executor("Command cannot be empty".to_string()));
    }

    // Create sandbox
    let sandbox = crate::sandbox::create_sandbox();

    let start_time = std::time::Instant::now();

    // Prepare environment (if needed)
    let env = vec![];

    // Execute command in sandbox
    // TODO: Pass stdin_input and timeout_secs to sandbox.run if supported
    // For now, we ignore stdin/timeout in the sandbox trait signature, 
    // but we should update the trait to support them.
    // Or we can wrap the sandbox call in a timeout here.
    
    let run_future = sandbox.run(command, args, &env);
    
    let output_result = if let Some(timeout) = timeout_secs {
        let duration = std::time::Duration::from_secs(u64::from(timeout));
        match tokio::time::timeout(duration, run_future).await {
            Ok(res) => res,
            Err(_) => {
                return Ok(TaskResult {
                    task_number,
                    success: false,
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: format!("Task timed out after {}s", timeout),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                });
            }
        }
    } else {
        run_future.await
    };

    let output = match output_result {
        Ok(out) => out,
        Err(e) => {
            return Ok(TaskResult {
                task_number,
                success: false,
                exit_code: -1,
                stdout: String::new(),
                stderr: format!("Sandbox execution failed: {}", e),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
            });
        }
    };

    let duration = start_time.elapsed();
    let execution_time_ms = duration.as_millis() as u64;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);
    let success = output.status.success();

    info!(
        "Task {} execution completed in {}ms (exit code: {})",
        task_number, execution_time_ms, exit_code
    );

    Ok(TaskResult {
        task_number,
        success,
        exit_code,
        stdout,
        stderr,
        execution_time_ms,
    })
}



#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_task_plan() {
        let plan = Plan {
            plan_id: "plan-456".to_string(),
            plan_description: None,
            tasks: vec![Task {
                task_number: 1,
                command: "echo".to_string(),
                args: vec!["hello".to_string()],
                input_from_task: None,
                timeout_secs: Some(30),
            }],
        };

        let result = execute_plan("job-123", &plan).await.unwrap();
        assert_eq!(result.job_id, "job-123");
        assert_eq!(result.plan_id, "plan-456");
        assert_eq!(result.task_results.len(), 1);
        assert_eq!(result.task_results[0].stdout.trim(), "hello");
        assert_eq!(result.task_results[0].exit_code, 0);
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_execute_multi_step_plan() {
        let plan = Plan {
            plan_id: "plan-456".to_string(),
            plan_description: Some("Multi-step test".to_string()),
            tasks: vec![
                Task {
                    task_number: 1,
                    command: "echo".to_string(),
                    args: vec!["line1\nline2\nline3".to_string()],
                    input_from_task: None,
                    timeout_secs: Some(30),
                },
                Task {
                    task_number: 2,
                    command: "wc".to_string(),
                    args: vec!["-l".to_string()],
                    input_from_task: Some(1),
                    timeout_secs: Some(30),
                },
            ],
        };

        let result = execute_plan("job-123", &plan).await.unwrap();
        assert_eq!(result.task_results.len(), 2);
        assert!(result.task_results[0].success);
        assert!(result.task_results[1].success);
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_execute_plan_with_failure() {
        let plan = Plan {
            plan_id: "plan-456".to_string(),
            plan_description: None,
            tasks: vec![
                Task {
                    task_number: 1,
                    command: "sh".to_string(),
                    args: vec!["-c".to_string(), "exit 42".to_string()],
                    input_from_task: None,
                    timeout_secs: Some(30),
                },
                Task {
                    task_number: 2,
                    command: "echo".to_string(),
                    args: vec!["should not run".to_string()],
                    input_from_task: None,
                    timeout_secs: Some(30),
                },
            ],
        };

        let result = execute_plan("job-123", &plan).await.unwrap();
        // Should only execute first task
        assert_eq!(result.task_results.len(), 1);
        assert_eq!(result.task_results[0].exit_code, 42);
        assert!(!result.task_results[0].success);
        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_execute_plan_with_timeout() {
        let plan = Plan {
            plan_id: "plan-456".to_string(),
            plan_description: None,
            tasks: vec![Task {
                task_number: 1,
                command: "sleep".to_string(),
                args: vec!["10".to_string()],
                input_from_task: None,
                timeout_secs: Some(1),
            }],
        };

        let result = execute_plan("job-123", &plan).await.unwrap();
        assert_eq!(result.task_results.len(), 1);
        assert!(!result.task_results[0].success);
        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_execute_plan_with_stdin_piping() {
        let plan = Plan {
            plan_id: "plan-456".to_string(),
            plan_description: None,
            tasks: vec![
                Task {
                    task_number: 1,
                    command: "echo".to_string(),
                    args: vec!["foo\nbar\nfoo".to_string()],
                    input_from_task: None,
                    timeout_secs: Some(30),
                },
                Task {
                    task_number: 2,
                    command: "sort".to_string(),
                    args: vec![],
                    input_from_task: Some(1),
                    timeout_secs: Some(30),
                },
                Task {
                    task_number: 3,
                    command: "uniq".to_string(),
                    args: vec![],
                    input_from_task: Some(2),
                    timeout_secs: Some(30),
                },
            ],
        };

        let result = execute_plan("job-123", &plan).await.unwrap();
        assert_eq!(result.task_results.len(), 3);
        assert!(result.success);

        // Final output should be sorted and unique
        let final_output = result.task_results[2].stdout.trim();
        assert!(final_output.contains("bar"));
        assert!(final_output.contains("foo"));
    }

    #[tokio::test]
    async fn test_execute_invalid_command() {
        let plan = Plan {
            plan_id: "plan-456".to_string(),
            plan_description: None,
            tasks: vec![Task {
                task_number: 1,
                command: "this_command_does_not_exist_12345".to_string(),
                args: vec![],
                input_from_task: None,
                timeout_secs: None,
            }],
        };

        let result = execute_plan("job-123", &plan).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_combined_output_methods() {
        let task_results = vec![
            TaskResult::new(1, "output1\n".to_string(), "error1\n".to_string(), 0),
            TaskResult::new(2, "output2\n".to_string(), "error2\n".to_string(), 0),
            TaskResult::new(3, "output3\n".to_string(), "error3\n".to_string(), 0),
        ];

        let plan_result =
            PlanResult::new("job-123".to_string(), "plan-456".to_string(), task_results);

        // Outputs already have newlines, so concatenation doesn't add extra separators
        assert_eq!(plan_result.combined_stdout(), "output1\noutput2\noutput3\n");
        assert_eq!(plan_result.combined_stderr(), "error1\nerror2\nerror3\n");
    }

    #[test]
    fn test_combined_output_empty() {
        let plan_result = PlanResult::new("job-123".to_string(), "plan-456".to_string(), vec![]);

        assert_eq!(plan_result.combined_stdout(), "");
        assert_eq!(plan_result.combined_stderr(), "");
    }
}
