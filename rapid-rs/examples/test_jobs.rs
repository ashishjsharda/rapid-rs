use rapid_rs::jobs::{JobQueue, JobMetadata, JobPriority, JobStatus};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create job queue
    let queue = JobQueue::new();
    
    // Submit a job
    let job_id = queue.submit(
        "send_email",
        json!({
            "to": "user@example.com",
            "subject": "Welcome!"
        }),
        JobPriority::High,
    ).await?;
    
    println!("✅ Job submitted: {}", job_id);
    
    // Fetch job
    if let Some((metadata, payload)) = queue.fetch_next().await? {
        println!("✅ Job fetched: {}", metadata.job_id);
        println!("   Type: {}", metadata.job_type);
        println!("   Priority: {:?}", metadata.priority);
        println!("   Payload: {}", payload);
        
        // Mark as completed
        queue.complete(metadata.job_id, json!({"status": "sent"})).await?;
        println!("✅ Job completed");
    }
    
    // List jobs
    let jobs = queue.list_jobs(Some(JobStatus::Completed), None, 10, 0).await?;
    println!("✅ Found {} completed jobs", jobs.len());
    
    Ok(())
}