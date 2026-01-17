//! Background job processing for rapid-rs
//!
//! Provides async task queue with retry logic, scheduling, and monitoring.

pub mod queue;
pub mod worker;
pub mod scheduler;
pub mod storage;

pub use queue::{JobQueue, JobConfig, JobPriority};
pub use worker::{Job, JobContext, JobResult};
pub use scheduler::{CronSchedule, Schedule};
pub use storage::{JobStorage, InMemoryJobStorage};

#[cfg(feature = "database")]
pub use storage::PostgresJobStorage;

use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Dead,
    Cancelled,
}

/// Job metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    pub id: Uuid,
    pub job_type: String,
    pub priority: JobPriority,
    pub status: JobStatus,
    pub retry_count: u32,
    pub max_retries: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
}

impl Default for JobMetadata {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            job_type: String::new(),
            priority: JobPriority::Normal,
            status: JobStatus::Pending,
            retry_count: 0,
            max_retries: 3,
            created_at: chrono::Utc::now(),
            scheduled_at: None,
            started_at: None,
            completed_at: None,
            error: None,
        }
    }
}