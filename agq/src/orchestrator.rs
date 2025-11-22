use crate::error::Result;
use crate::job::{Job, JobStatus};
use crate::storage::Database;
use std::collections::HashSet;
use tracing::{debug, info, warn};

/// Orchestrator manages the lifecycle of Jobs and their dependencies.
pub struct Orchestrator<'a> {
    db: &'a Database,
}

impl<'a> Orchestrator<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Submit a set of jobs (usually from a single Action)
    ///
    /// This function:
    /// 1. Stores all jobs in the database
    /// 2. Identifies jobs with no pending dependencies
    /// 3. Moves those ready jobs to the appropriate queues
    pub fn submit_jobs(&self, jobs: Vec<Job>) -> Result<()> {
        let mut ready_jobs = Vec::new();

        for job in jobs {
            // Store the job
            self.save_job(&job)?;

            // Check if ready (no dependencies)
            if job.dependencies.is_empty() {
                ready_jobs.push(job);
            }
        }

        // Queue ready jobs
        for job in ready_jobs {
            self.enqueue_job(&job)?;
        }

        Ok(())
    }

    /// Mark a job as completed and trigger dependents
    pub fn complete_job(&self, job_id: &str, exit_code: i32) -> Result<()> {
        let mut job = self.get_job(job_id)?;

        // Update status
        job.status = JobStatus::Completed;
        job.completed_at = Some(crate::server::get_current_timestamp_secs().unwrap_or(0));
        job.exit_code = Some(exit_code);
        self.save_job(&job)?;

        info!("Job {} completed", job_id);

        // Trigger dependents
        self.trigger_dependents(&job)?;

        Ok(())
    }

    /// Mark a job as failed
    pub fn fail_job(&self, job_id: &str, exit_code: i32) -> Result<()> {
        let mut job = self.get_job(job_id)?;

        // Update status
        job.status = JobStatus::Failed;
        job.completed_at = Some(crate::server::get_current_timestamp_secs().unwrap_or(0));
        job.exit_code = Some(exit_code);
        self.save_job(&job)?;

        warn!("Job {} failed", job_id);

        // TODO: Handle failure propagation (cancel dependents?)
        // For now, dependents will just stay pending forever (or until timeout)

        Ok(())
    }

    /// Check dependents and enqueue them if all their dependencies are met
    fn trigger_dependents(&self, completed_job: &Job) -> Result<()> {
        for dependent_id in &completed_job.dependents {
            let dependent = self.get_job(dependent_id)?;

            if dependent.status != JobStatus::Pending {
                continue;
            }

            // Check if ALL dependencies are completed
            let all_met = self.check_dependencies_met(&dependent)?;

            if all_met {
                debug!("All dependencies met for job {}, queuing", dependent.id);
                self.enqueue_job(&dependent)?;
            }
        }

        Ok(())
    }

    /// Check if all dependencies for a job are in Completed state
    fn check_dependencies_met(&self, job: &Job) -> Result<bool> {
        for dep_id in &job.dependencies {
            let dep = self.get_job(dep_id)?;
            if dep.status != JobStatus::Completed {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Move a job to the Ready state and push to the appropriate queue
    fn enqueue_job(&self, job: &Job) -> Result<()> {
        let mut job = job.clone();
        job.status = JobStatus::Ready;
        self.save_job(&job)?;

        // Determine queue based on tags
        // Default: queue:default
        // If tags contains "gpu": queue:gpu
        let queue_name = if job.tags.contains(&"gpu".to_string()) {
            "queue:gpu"
        } else {
            "queue:default"
        };

        // Push job ID to Redis list
        // We push the ID, workers will fetch metadata via JOB.GET
        // Note: We use the raw storage interface here
        // In a real implementation, we might want a cleaner abstraction for queues
        use crate::storage::ListOps;
        self.db.lpush(queue_name, job.id.as_bytes())?;

        info!("Enqueued job {} to {}", job.id, queue_name);

        Ok(())
    }

    // --- Storage Helpers ---

    fn save_job(&self, job: &Job) -> Result<()> {
        let key = format!("job:{}", job.id);
        let json = serde_json::to_string(job)
            .map_err(|e| crate::error::Error::Protocol(format!("Failed to serialize job: {}", e)))?;

        use crate::storage::StringOps;
        self.db.set(&key, json.as_bytes())?;
        Ok(())
    }

    fn get_job(&self, job_id: &str) -> Result<Job> {
        let key = format!("job:{}", job_id);
        use crate::storage::StringOps;
        
        let json = self.db.get(&key)?
            .ok_or_else(|| crate::error::Error::Protocol(format!("Job not found: {}", job_id)))?;

        let job: Job = serde_json::from_slice(&json)
            .map_err(|e| crate::error::Error::Protocol(format!("Failed to deserialize job: {}", e)))?;

        Ok(job)
    }
}
