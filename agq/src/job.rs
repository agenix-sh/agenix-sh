use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Status of a Job (Task execution)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    /// Waiting for dependencies to complete
    Pending,
    /// Dependencies met, ready for execution
    Ready,
    /// Currently being executed by a worker
    Running,
    /// Successfully completed
    Completed,
    /// Execution failed
    Failed,
    /// Cancelled by user or system
    Cancelled,
}

impl JobStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
        )
    }
}

/// A Job represents a single Task execution unit within the AGQ system.
///
/// Unlike the previous architecture where a Job was a full Plan execution,
/// a Job now corresponds to a single Task from a Plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique identifier for this job (task execution)
    pub id: String,

    /// ID of the Action that triggered this job
    pub action_id: String,

    /// ID of the Plan this task belongs to
    pub plan_id: String,

    /// Task number within the plan (1-based)
    pub task_number: u32,

    /// The command to execute
    pub command: String,

    /// Arguments for the command
    pub args: Vec<String>,

    /// Environment variables / Input substitutions
    pub env: serde_json::Value,

    /// Current status of the job
    pub status: JobStatus,

    /// IDs of jobs that must complete successfully before this job can start
    pub dependencies: HashSet<String>,

    /// IDs of jobs that depend on this job (reverse dependency graph)
    /// Used for efficient DAG traversal upon completion
    pub dependents: HashSet<String>,

    /// ID of the worker currently executing this job (if Running)
    pub worker_id: Option<String>,

    /// Timestamp when created
    pub created_at: u64,

    /// Timestamp when started
    pub started_at: Option<u64>,

    /// Timestamp when completed/failed
    pub completed_at: Option<u64>,

    /// Exit code (if completed)
    pub exit_code: Option<i32>,

    /// Required worker tags (e.g., "gpu", "linux")
    pub tags: Vec<String>,
}

impl Job {
    pub fn new(
        id: String,
        action_id: String,
        plan_id: String,
        task_number: u32,
        command: String,
        args: Vec<String>,
        env: serde_json::Value,
        tags: Vec<String>,
    ) -> Self {
        Self {
            id,
            action_id,
            plan_id,
            task_number,
            command,
            args,
            env,
            status: JobStatus::Pending,
            dependencies: HashSet::new(),
            dependents: HashSet::new(),
            worker_id: None,
            created_at: crate::server::get_current_timestamp_secs().unwrap_or(0),
            started_at: None,
            completed_at: None,
            exit_code: None,
            tags,
        }
    }
}

/// Represents a Plan template (Execution Layer 2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub plan_id: String,
    pub plan_description: Option<String>,
    pub tasks: Vec<TaskTemplate>,
}

/// Represents a Task definition within a Plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTemplate {
    pub task_number: u32,
    pub command: String,
    pub args: Vec<String>,
    pub input_from_task: Option<u32>,
    pub timeout_secs: Option<u32>,
}
