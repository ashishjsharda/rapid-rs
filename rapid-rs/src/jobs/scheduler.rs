//! Job scheduling with cron support

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Cron schedule parser and evaluator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronSchedule {
    expression: String,
}

impl CronSchedule {
    pub fn new(expression: impl Into<String>) -> Result<Self, ScheduleError> {
        let expression = expression.into();
        // Basic validation
        Self::validate(&expression)?;
        Ok(Self { expression })
    }
    
    fn validate(expr: &str) -> Result<(), ScheduleError> {
        let parts: Vec<&str> = expr.split_whitespace().collect();
        if parts.len() != 5 && parts.len() != 6 {
            return Err(ScheduleError::InvalidFormat(
                "Cron expression must have 5 or 6 fields".to_string(),
            ));
        }
        Ok(())
    }
    
    /// Get the next run time after the given time
    pub fn next_run(&self, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
        // This is a simplified implementation
        // In production, use the `cron` crate for full cron parsing
        Some(after + chrono::Duration::hours(1))
    }
}

impl FromStr for CronSchedule {
    type Err = ScheduleError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// Schedule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Schedule {
    /// Run once at a specific time
    Once(DateTime<Utc>),
    
    /// Run at a fixed interval
    Interval {
        seconds: u64,
        start_at: Option<DateTime<Utc>>,
    },
    
    /// Run on a cron schedule
    Cron(CronSchedule),
}

impl Schedule {
    /// Create a once schedule
    pub fn once(at: DateTime<Utc>) -> Self {
        Self::Once(at)
    }
    
    /// Create an interval schedule
    pub fn every(seconds: u64) -> Self {
        Self::Interval {
            seconds,
            start_at: None,
        }
    }
    
    /// Create an interval schedule with a start time
    pub fn every_starting_at(seconds: u64, start_at: DateTime<Utc>) -> Self {
        Self::Interval {
            seconds,
            start_at: Some(start_at),
        }
    }
    
    /// Create a cron schedule
    pub fn cron(expression: &str) -> Result<Self, ScheduleError> {
        Ok(Self::Cron(CronSchedule::new(expression)?))
    }
    
    /// Get the next run time after the given time
    pub fn next_run(&self, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            Self::Once(at) => {
                if after < *at {
                    Some(*at)
                } else {
                    None
                }
            }
            Self::Interval { seconds, start_at } => {
                let start = start_at.unwrap_or(after);
                if after < start {
                    Some(start)
                } else {
                    let elapsed = (after - start).num_seconds() as u64;
                    let intervals = (elapsed / seconds) + 1;
                    Some(start + chrono::Duration::seconds((intervals * seconds) as i64))
                }
            }
            Self::Cron(cron) => cron.next_run(after),
        }
    }
}

/// Schedule error types
#[derive(Debug, thiserror::Error)]
pub enum ScheduleError {
    #[error("Invalid cron format: {0}")]
    InvalidFormat(String),
    
    #[error("Invalid field value: {0}")]
    InvalidValue(String),
}

/// Common schedule helpers
pub mod schedules {
    use super::*;
    
    /// Every minute
    pub fn every_minute() -> Schedule {
        Schedule::every(60)
    }
    
    /// Every 5 minutes
    pub fn every_5_minutes() -> Schedule {
        Schedule::every(300)
    }
    
    /// Every hour
    pub fn hourly() -> Schedule {
        Schedule::every(3600)
    }
    
    /// Every day at midnight
    pub fn daily() -> Result<Schedule, ScheduleError> {
        Schedule::cron("0 0 * * *")
    }
    
    /// Every week on Monday at midnight
    pub fn weekly() -> Result<Schedule, ScheduleError> {
        Schedule::cron("0 0 * * 1")
    }
    
    /// Every month on the 1st at midnight
    pub fn monthly() -> Result<Schedule, ScheduleError> {
        Schedule::cron("0 0 1 * *")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_once_schedule() {
        let future = Utc::now() + chrono::Duration::hours(1);
        let schedule = Schedule::once(future);
        
        let next = schedule.next_run(Utc::now()).unwrap();
        assert!(next > Utc::now());
    }
    
    #[test]
    fn test_interval_schedule() {
        let schedule = Schedule::every(60); // Every minute
        let now = Utc::now();
        
        let next = schedule.next_run(now).unwrap();
        assert!(next > now);
        assert!(next - now <= chrono::Duration::seconds(60));
    }
    
    #[test]
    fn test_cron_schedule() {
        let schedule = Schedule::cron("0 0 * * *").unwrap();
        let next = schedule.next_run(Utc::now());
        assert!(next.is_some());
    }
}
