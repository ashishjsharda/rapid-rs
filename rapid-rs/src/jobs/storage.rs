//! Job storage backends

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{JobMetadata, JobStatus};
use crate::error::ApiError;
use crate::jobs::queue::QueueStats;

/// Trait for job storage backends
#[async_trait]
pub trait JobStorage: Send + Sync + 'static {
    /// Save a job with its metadata
    async fn save_job(&self, metadata: &JobMetadata, payload: Value) -> Result<(), ApiError>;
    
    /// Get job metadata by ID
    async fn get_job(&self, job_id: Uuid) -> Result<JobMetadata, ApiError>;
    
    /// Fetch the next pending job
    async fn fetch_next_job(&self) -> Result<Option<(JobMetadata, Value)>, ApiError>;
    
    /// Get queue statistics
    async fn get_stats(&self) -> Result<QueueStats, ApiError>;
    
    /// Clean up old completed jobs
    async fn cleanup_old_jobs(&self, older_than_days: u32) -> Result<usize, ApiError>;
}

/// In-memory job storage (for development/testing)
#[derive(Clone)]
pub struct InMemoryJobStorage {
    jobs: Arc<RwLock<HashMap<Uuid, (JobMetadata, Value)>>>,
}

impl InMemoryJobStorage {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryJobStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl JobStorage for InMemoryJobStorage {
    async fn save_job(&self, metadata: &JobMetadata, payload: Value) -> Result<(), ApiError> {
        let mut jobs = self.jobs.write().await;
        jobs.insert(metadata.id, (metadata.clone(), payload));
        Ok(())
    }
    
    async fn get_job(&self, job_id: Uuid) -> Result<JobMetadata, ApiError> {
        let jobs = self.jobs.read().await;
        jobs.get(&job_id)
            .map(|(metadata, _)| metadata.clone())
            .ok_or_else(|| ApiError::NotFound(format!("Job {} not found", job_id)))
    }
    
    async fn fetch_next_job(&self) -> Result<Option<(JobMetadata, Value)>, ApiError> {
        let mut jobs = self.jobs.write().await;
        
        // Find highest priority pending job
        let mut pending_jobs: Vec<_> = jobs
            .iter()
            .filter(|(_, (metadata, _))| {
                metadata.status == JobStatus::Pending
                    && metadata.scheduled_at.map_or(true, |t| t <= chrono::Utc::now())
            })
            .collect();
        
        pending_jobs.sort_by(|a, b| b.1 .0.priority.cmp(&a.1 .0.priority));
        
        if let Some((job_id, (metadata, payload))) = pending_jobs.first() {
            let result = Some(((*metadata).clone(), payload.clone()));
            
            // Update status to running
            if let Some((meta, pay)) = jobs.get_mut(*job_id) {
                meta.status = JobStatus::Running;
                meta.started_at = Some(chrono::Utc::now());
            }
            
            Ok(result)
        } else {
            Ok(None)
        }
    }
    
    async fn get_stats(&self) -> Result<QueueStats, ApiError> {
        let jobs = self.jobs.read().await;
        
        let mut stats = QueueStats {
            pending: 0,
            running: 0,
            completed: 0,
            failed: 0,
            dead: 0,
        };
        
        for (metadata, _) in jobs.values() {
            match metadata.status {
                JobStatus::Pending => stats.pending += 1,
                JobStatus::Running => stats.running += 1,
                JobStatus::Completed => stats.completed += 1,
                JobStatus::Failed => stats.failed += 1,
                JobStatus::Dead => stats.dead += 1,
                _ => {}
            }
        }
        
        Ok(stats)
    }
    
    async fn cleanup_old_jobs(&self, older_than_days: u32) -> Result<usize, ApiError> {
        let mut jobs = self.jobs.write().await;
        let cutoff = chrono::Utc::now() - chrono::Duration::days(older_than_days as i64);
        
        let to_remove: Vec<Uuid> = jobs
            .iter()
            .filter(|(_, (metadata, _))| {
                metadata.status == JobStatus::Completed
                    && metadata.completed_at.map_or(false, |t| t < cutoff)
            })
            .map(|(id, _)| *id)
            .collect();
        
        let count = to_remove.len();
        
        for id in to_remove {
            jobs.remove(&id);
        }
        
        Ok(count)
    }
}

/// PostgreSQL job storage
#[cfg(feature = "database")]
pub struct PostgresJobStorage {
    pool: sqlx::PgPool,
}

#[cfg(feature = "database")]
impl PostgresJobStorage {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
    
    /// Initialize the jobs table
    pub async fn init(&self) -> Result<(), ApiError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS jobs (
                id UUID PRIMARY KEY,
                job_type VARCHAR(255) NOT NULL,
                payload JSONB NOT NULL,
                priority INTEGER NOT NULL,
                status VARCHAR(50) NOT NULL,
                retry_count INTEGER NOT NULL DEFAULT 0,
                max_retries INTEGER NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                scheduled_at TIMESTAMPTZ,
                started_at TIMESTAMPTZ,
                completed_at TIMESTAMPTZ,
                error TEXT
            );
            
            CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
            CREATE INDEX IF NOT EXISTS idx_jobs_priority ON jobs(priority DESC);
            CREATE INDEX IF NOT EXISTS idx_jobs_scheduled ON jobs(scheduled_at);
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}

#[cfg(feature = "database")]
#[async_trait]
impl JobStorage for PostgresJobStorage {
    async fn save_job(&self, metadata: &JobMetadata, payload: Value) -> Result<(), ApiError> {
        sqlx::query(
            r#"
            INSERT INTO jobs (
                id, job_type, payload, priority, status, retry_count, max_retries,
                created_at, scheduled_at, started_at, completed_at, error
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (id) DO UPDATE SET
                status = $5,
                retry_count = $6,
                started_at = $10,
                completed_at = $11,
                error = $12
            "#,
        )
        .bind(metadata.id)
        .bind(&metadata.job_type)
        .bind(&payload)
        .bind(metadata.priority as i32)
        .bind(format!("{:?}", metadata.status))
        .bind(metadata.retry_count as i32)
        .bind(metadata.max_retries as i32)
        .bind(metadata.created_at)
        .bind(metadata.scheduled_at)
        .bind(metadata.started_at)
        .bind(metadata.completed_at)
        .bind(&metadata.error)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn get_job(&self, job_id: Uuid) -> Result<JobMetadata, ApiError> {
        let row = sqlx::query_as::<_, (Uuid, String, i32, String, i32, i32, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>, Option<String>)>(
            "SELECT id, job_type, priority, status, retry_count, max_retries, created_at, scheduled_at, started_at, completed_at, error FROM jobs WHERE id = $1"
        )
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Job {} not found", job_id)))?;
        
        let status = match row.3.as_str() {
            "Pending" => JobStatus::Pending,
            "Running" => JobStatus::Running,
            "Completed" => JobStatus::Completed,
            "Failed" => JobStatus::Failed,
            "Dead" => JobStatus::Dead,
            "Cancelled" => JobStatus::Cancelled,
            _ => JobStatus::Pending,
        };
        
        let priority = match row.2 {
            0 => crate::jobs::JobPriority::Low,
            1 => crate::jobs::JobPriority::Normal,
            2 => crate::jobs::JobPriority::High,
            3 => crate::jobs::JobPriority::Critical,
            _ => crate::jobs::JobPriority::Normal,
        };
        
        Ok(JobMetadata {
            id: row.0,
            job_type: row.1,
            priority,
            status,
            retry_count: row.4 as u32,
            max_retries: row.5 as u32,
            created_at: row.6,
            scheduled_at: row.7,
            started_at: row.8,
            completed_at: row.9,
            error: row.10,
        })
    }
    
    async fn fetch_next_job(&self) -> Result<Option<(JobMetadata, Value)>, ApiError> {
        let row = sqlx::query_as::<_, (Uuid, String, Value, i32, String, i32, i32, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>, Option<String>)>(
            r#"
            UPDATE jobs
            SET status = 'Running', started_at = NOW()
            WHERE id = (
                SELECT id FROM jobs
                WHERE status = 'Pending'
                AND (scheduled_at IS NULL OR scheduled_at <= NOW())
                ORDER BY priority DESC, created_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING id, job_type, payload, priority, status, retry_count, max_retries, created_at, scheduled_at, started_at, completed_at, error
            "#
        )
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let status = match row.4.as_str() {
                "Pending" => JobStatus::Pending,
                "Running" => JobStatus::Running,
                "Completed" => JobStatus::Completed,
                "Failed" => JobStatus::Failed,
                "Dead" => JobStatus::Dead,
                "Cancelled" => JobStatus::Cancelled,
                _ => JobStatus::Pending,
            };
            
            let priority = match row.3 {
                0 => crate::jobs::JobPriority::Low,
                1 => crate::jobs::JobPriority::Normal,
                2 => crate::jobs::JobPriority::High,
                3 => crate::jobs::JobPriority::Critical,
                _ => crate::jobs::JobPriority::Normal,
            };
            
            let metadata = JobMetadata {
                id: row.0,
                job_type: row.1.clone(),
                priority,
                status,
                retry_count: row.5 as u32,
                max_retries: row.6 as u32,
                created_at: row.7,
                scheduled_at: row.8,
                started_at: row.9,
                completed_at: row.10,
                error: row.11,
            };
            
            Ok(Some((metadata, row.2)))
        } else {
            Ok(None)
        }
    }
    
    async fn get_stats(&self) -> Result<QueueStats, ApiError> {
        let row = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status = 'Pending') as pending,
                COUNT(*) FILTER (WHERE status = 'Running') as running,
                COUNT(*) FILTER (WHERE status = 'Completed') as completed,
                COUNT(*) FILTER (WHERE status = 'Failed') as failed,
                COUNT(*) FILTER (WHERE status = 'Dead') as dead
            FROM jobs
            "#
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(QueueStats {
            pending: row.0 as usize,
            running: row.1 as usize,
            completed: row.2 as usize,
            failed: row.3 as usize,
            dead: row.4 as usize,
        })
    }
    
    async fn cleanup_old_jobs(&self, older_than_days: u32) -> Result<usize, ApiError> {
        let result = sqlx::query(
            r#"
            DELETE FROM jobs
            WHERE status = 'Completed'
            AND completed_at < NOW() - $1::INTERVAL
            "#
        )
        .bind(format!("{} days", older_than_days))
        .execute(&self.pool)
        .await?;
        
        Ok(result.rows_affected() as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::JobPriority;
    
    #[tokio::test]
    async fn test_in_memory_storage() {
        let storage = InMemoryJobStorage::new();
        
        let mut metadata = JobMetadata::default();
        metadata.job_type = "test_job".to_string();
        metadata.priority = JobPriority::High;
        
        let payload = serde_json::json!({"test": "data"});
        
        storage.save_job(&metadata, payload.clone()).await.unwrap();
        
        let retrieved = storage.get_job(metadata.id).await.unwrap();
        assert_eq!(retrieved.job_type, "test_job");
        assert_eq!(retrieved.priority, JobPriority::High);
    }
}
