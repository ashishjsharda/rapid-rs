//! Job worker trait and execution context

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Job execution context
#[derive(Debug, Clone)]
pub struct JobContext {
    pub job_id: Uuid,
    pub job_type: String,
    pub retry_count: u32,
    pub metadata: HashMap<String, String>,
}

impl JobContext {
    pub fn new(job_id: Uuid, job_type: String) -> Self {
        Self {
            job_id,
            job_type,
            retry_count: 0,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_retry_count(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }
    
    pub fn add_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Job execution result
pub type JobResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// Trait for defining background jobs
#[async_trait]
pub trait Job: Send + Sync + Serialize + for<'de> Deserialize<'de> {
    /// Execute the job
    async fn execute(&self, ctx: JobContext) -> JobResult;
    
    /// Job type identifier
    fn job_type(&self) -> &str;
    
    /// Maximum number of retries (override default)
    fn max_retries(&self) -> Option<u32> {
        None
    }
    
    /// Timeout in seconds (override default)
    fn timeout_seconds(&self) -> Option<u64> {
        None
    }
    
    /// Called before job execution
    async fn before_execute(&self, _ctx: &JobContext) -> JobResult {
        Ok(())
    }
    
    /// Called after successful execution
    async fn after_execute(&self, _ctx: &JobContext) -> JobResult {
        Ok(())
    }
    
    /// Called when job fails (for cleanup)
    async fn on_failure(&self, _ctx: &JobContext, _error: &dyn std::error::Error) {}
}

/// Job registry for managing job handlers
pub struct JobRegistry {
    handlers: Arc<tokio::sync::RwLock<HashMap<String, Box<dyn JobHandler>>>>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a job handler
    pub async fn register<J: Job + 'static>(&self, job_type: &str) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(
            job_type.to_string(),
            Box::new(TypedJobHandler::<J>::new()),
        );
        tracing::info!(job_type = %job_type, "Registered job handler");
    }
    
    /// Execute a job by type
    pub async fn execute(
        &self,
        job_type: &str,
        payload: serde_json::Value,
        ctx: JobContext,
    ) -> JobResult {
        let handlers = self.handlers.read().await;
        
        if let Some(handler) = handlers.get(job_type) {
            handler.handle(payload, ctx).await
        } else {
            Err(format!("No handler registered for job type: {}", job_type).into())
        }
    }
}

impl Default for JobRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal trait for type-erased job handling
#[async_trait]
trait JobHandler: Send + Sync {
    async fn handle(&self, payload: serde_json::Value, ctx: JobContext) -> JobResult;
}

/// Typed job handler wrapper
struct TypedJobHandler<J: Job> {
    _phantom: std::marker::PhantomData<J>,
}

impl<J: Job> TypedJobHandler<J> {
    fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<J: Job + 'static> JobHandler for TypedJobHandler<J> {
    async fn handle(&self, payload: serde_json::Value, ctx: JobContext) -> JobResult {
        let job: J = serde_json::from_value(payload)
            .map_err(|e| format!("Failed to deserialize job: {}", e))?;
        
        job.before_execute(&ctx).await?;
        
        let result = job.execute(ctx.clone()).await;
        
        match &result {
            Ok(_) => {
                job.after_execute(&ctx).await?;
            }
            Err(e) => {
                job.on_failure(&ctx, e.as_ref()).await;
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Serialize, Deserialize)]
    struct TestJob {
        message: String,
    }
    
    #[async_trait]
    impl Job for TestJob {
        async fn execute(&self, _ctx: JobContext) -> JobResult {
            println!("Executing: {}", self.message);
            Ok(())
        }
        
        fn job_type(&self) -> &str {
            "test_job"
        }
    }
    
    #[tokio::test]
    async fn test_job_registry() {
        let registry = JobRegistry::new();
        registry.register::<TestJob>("test_job").await;
        
        let payload = serde_json::json!({
            "message": "Hello, World!"
        });
        
        let ctx = JobContext::new(Uuid::new_v4(), "test_job".to_string());
        
        let result = registry.execute("test_job", payload, ctx).await;
        assert!(result.is_ok());
    }
}
