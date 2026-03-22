//! Email and SMS notification support
//!
//! Provides a unified interface for sending email and SMS notifications
//! with support for multiple backends.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use rapid_rs::notifications::{NotificationService, EmailConfig, EmailMessage};
//!
//! let config = EmailConfig::new()
//!     .with_smtp_host("smtp.gmail.com")
//!     .with_port(587)
//!     .with_credentials("user@gmail.com", "app-password")
//!     .with_from("My App <noreply@myapp.com>");
//!
//! let service = NotificationService::new(config);
//!
//! service.send_email(EmailMessage {
//!     to: vec!["user@example.com".to_string()],
//!     subject: "Welcome!".to_string(),
//!     body: "Welcome to our platform!".to_string(),
//!     html_body: None,
//! }).await?;
//! ```

pub mod email;

#[cfg(feature = "notifications-sms")]
pub mod sms;

pub use email::{EmailConfig, EmailMessage, EmailProvider, SmtpEmailProvider};

#[cfg(feature = "notifications-sms")]
pub use sms::{SmsConfig, SmsMessage, SmsProvider, TwilioSmsProvider};

use crate::error::ApiError;

/// Notification result type
pub type NotificationResult = Result<(), ApiError>;

/// Notification service - unified interface for all notification channels
pub struct NotificationService {
    email_provider: Option<SmtpEmailProvider>,
    #[cfg(feature = "notifications-sms")]
    sms_provider: Option<TwilioSmsProvider>,
}

impl NotificationService {
    /// Create a new notification service without any providers configured
    pub fn new() -> Self {
        Self {
            email_provider: None,
            #[cfg(feature = "notifications-sms")]
            sms_provider: None,
        }
    }

    /// Add email provider
    pub fn with_email(mut self, config: EmailConfig) -> Self {
        self.email_provider = Some(SmtpEmailProvider::new(config));
        self
    }

    /// Add SMS provider (requires notifications-sms feature)
    #[cfg(feature = "notifications-sms")]
    pub fn with_sms(mut self, config: SmsConfig) -> Self {
        self.sms_provider = Some(TwilioSmsProvider::new(config));
        self
    }

    /// Send an email notification
    pub async fn send_email(&self, message: EmailMessage) -> NotificationResult {
        let provider = self.email_provider.as_ref().ok_or_else(|| {
            ApiError::InternalServerError("Email provider not configured".to_string())
        })?;
        provider.send(message).await
    }

    /// Send an SMS notification (requires notifications-sms feature)
    #[cfg(feature = "notifications-sms")]
    pub async fn send_sms(&self, message: SmsMessage) -> NotificationResult {
        let provider = self.sms_provider.as_ref().ok_or_else(|| {
            ApiError::InternalServerError("SMS provider not configured".to_string())
        })?;
        provider.send(message).await
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_service_creation() {
        let service = NotificationService::new();
        assert!(service.email_provider.is_none());
    }

    #[test]
    fn test_email_config_builder() {
        let config = EmailConfig::new()
            .with_smtp_host("smtp.example.com")
            .with_port(587)
            .with_credentials("user@example.com", "password")
            .with_from("Test <test@example.com>");

        assert_eq!(config.smtp_host, "smtp.example.com");
        assert_eq!(config.port, 587);
        assert_eq!(config.from, "Test <test@example.com>");
    }
}
