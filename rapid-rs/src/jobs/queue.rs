//! Job queue implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{JobMetadata, JobStatus, JobStorage};
use crate::error::ApiError;

/// Job priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Job queue configuration
#[derive(Debug, Clone)]
pub struct JobConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Delay between retries (exponential backoff multiplier)
    pub retry_delay_seconds: u64,
    /// Number of worker threads
    pub worker_count: usize,
    /// Job timeout duration
    pub job_timeout_seconds: u64,
}

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_seconds: 60,
            worker_count: 4,
            job_timeout_seconds: 300, // 5 minutes
        }
    }
}

/// Job queue for managing background tasks
pub struct JobQueue<S: JobStorage> {
    storage: Arc<S>,
    config: JobConfig,
    workers: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

impl<S: JobStorage> JobQueue<S> {
    /// Create a new job queue with custom storage
    pub fn new(storage: S, config: JobConfig) -> Self {
        Self {
            storage: Arc::new(storage),
            config,
            workers: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Enqueue a job with default priority
    pub async fn enqueue<J: Serialize>(
        &self,
        job: J,
        job_type: &str,
    ) -> Result<Uuid, ApiError> {
        self.enqueue_with_priority(job, job_type, JobPriority::Normal)
            .await
    }
    
    /// Enqueue a job with specific priority
    pub async fn enqueue_with_priority<J: Serialize>(
        &self,
        job: J,
        job_type: &str,
        priority: JobPriority,
    ) -> Result<Uuid, ApiError> {
        let payload = serde_json::to_value(job)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to serialize job: {}", e)))?;
        
        let mut metadata = JobMetadata::default();
        metadata.job_type = job_type.to_string();
        metadata.priority = priority;
        metadata.max_retries = self.config.max_retries;
        
        self.storage.save_job(&metadata, payload).await?;
        
        tracing::info!(
            job_id = %metadata.id,
            job_type = %job_type,
            priority = ?priority,
            "Job enqueued"
        );
        
        Ok(metadata.id)
    }
    
    /// Schedule a job to run at a specific time
    pub async fn schedule<J: Serialize>(
        &self,
        job: J,
        job_type: &str,
        scheduled_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<Uuid, ApiError> {
        let payload = serde_json::to_value(job)
            .map_err(|e| ApiError::InternalServerError(format!("Failed to serialize job: {}", e)))?;
        
        let mut metadata = JobMetadata::default();
        metadata.job_type = job_type.to_string();
        metadata.scheduled_at = Some(scheduled_at);
        metadata.max_retries = self.config.max_retries;
        
        self.storage.save_job(&metadata, payload).await?;
        
        tracing::info!(
            job_id = %metadata.id,
            job_type = %job_type,
            scheduled_at = %scheduled_at,
            "Job scheduled"
        );
        
        Ok(metadata.id)
    }
    
    /// Get job status
    pub async fn get_status(&self, job_id: Uuid) -> Result<JobStatus, ApiError> {
        let metadata = self.storage.get_job(job_id).await?;
        Ok(metadata.status)
    }
    
    /// Cancel a pending job
    pub async fn cancel(&self, job_id: Uuid) -> Result<(), ApiError> {
        let mut metadata = self.storage.get_job(job_id).await?;
        
        if metadata.status == JobStatus::Pending {
            metadata.status = JobStatus::Cancelled;
            let payload = serde_json::Value::Null;
            self.storage.save_job(&metadata, payload).await?;
            
            tracing::info!(job_id = %job_id, "Job cancelled");
            Ok(())
        } else {
            Err(ApiError::BadRequest(format!(
                "Cannot cancel job with status {:?}",
                metadata.status
            )))
        }
    }
    
    /// Get queue statistics
    pub async fn stats(&self) -> Result<QueueStats, ApiError> {
        self.storage.get_stats().await
    }
    
    /// Start background workers
    pub async fn start_workers(&self) {
        let mut workers = self.workers.write().await;
        
        for i in 0..self.config.worker_count {
            let storage = Arc::clone(&self.storage);
            let config = self.config.clone();
            
            let handle = tokio::spawn(async move {
                tracing::info!("Worker {} started", i);
                
                loop {
                    match storage.fetch_next_job().await {
                        Ok(Some((mut metadata, payload))) => {
                            metadata.status = JobStatus::Running;
                            metadata.started_at = Some(chrono::Utc::now());
                            
                            if let Err(e) = storage.save_job(&metadata, payload.clone()).await {
                                tracing::error!(job_id = %metadata.id, error = %e, "Failed to update job status");
                                continue;
                            }
                            
                            tracing::info!(
                                job_id = %metadata.id,
                                job_type = %metadata.job_type,
                                "Processing job"
                            );
                            
                            // Job execution would happen here via registered handlers
                            // For now, mark as completed
                            metadata.status = JobStatus::Completed;
                            metadata.completed_at = Some(chrono::Utc::now());
                            
                            if let Err(e) = storage.save_job(&metadata, payload).await {
                                tracing::error!(job_id = %metadata.id, error = %e, "Failed to complete job");
                            }
                        }
                        Ok(None) => {
                            // No jobs available, sleep briefly
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "Error fetching job");
                            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        }
                    }
                }
            });
            
            workers.push(handle);
        }
        
        tracing::info!("Started {} workers", self.config.worker_count);
    }
    
    /// Stop all workers
    pub async fn stop_workers(&self) {
        let mut workers = self.workers.write().await;
        
        for handle in workers.drain(..) {
            handle.abort();
        }
        
        tracing::info!("All workers stopped");
    }
}

/// Queue statistics
#[derive(Debug, Clone, Serialize)]
pub struct QueueStats {
    pub pending: usize,
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
    pub dead: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::InMemoryJobStorage;
    
    #[tokio::test]
    async fn test_enqueue_job() {
        let storage = InMemoryJobStorage::new();
        let queue = JobQueue::new(storage, JobConfig::default());
        
        let job_id = queue
            .enqueue(serde_json::json!({"test": "data"}), "test_job")
            .await
            .unwrap();
        
        let status = queue.get_status(job_id).await.unwrap();
        assert_eq!(status, JobStatus::Pending);
    }
}
