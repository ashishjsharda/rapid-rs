//! Email notification support via SMTP (lettre)

use lettre::{
    message::{header::ContentType, Message, Mailbox},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use crate::error::ApiError;

/// SMTP email configuration
#[derive(Debug, Clone)]
pub struct EmailConfig {
    /// SMTP server hostname
    pub smtp_host: String,
    /// SMTP port (typically 587 for TLS, 465 for SSL)
    pub port: u16,
    /// Username for authentication
    pub username: String,
    /// Password or app password for authentication
    pub password: String,
    /// From address (e.g., "My App <noreply@myapp.com>")
    pub from: String,
    /// Whether to use TLS
    pub use_tls: bool,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_host: "localhost".to_string(),
            port: 25,
            username: String::new(),
            password: String::new(),
            from: "noreply@localhost".to_string(),
            use_tls: false,
        }
    }
}

impl EmailConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_smtp_host(mut self, host: impl Into<String>) -> Self {
        self.smtp_host = host.into();
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn with_credentials(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.username = username.into();
        self.password = password.into();
        self
    }

    pub fn with_from(mut self, from: impl Into<String>) -> Self {
        self.from = from.into();
        self
    }

    pub fn with_tls(mut self) -> Self {
        self.use_tls = true;
        self
    }
}

/// Email message
#[derive(Debug, Clone)]
pub struct EmailMessage {
    /// Recipient email addresses
    pub to: Vec<String>,
    /// CC recipients
    pub cc: Vec<String>,
    /// Email subject
    pub subject: String,
    /// Plain text body
    pub body: String,
    /// Optional HTML body (if provided, sends multipart email)
    pub html_body: Option<String>,
}

impl EmailMessage {
    pub fn new(to: impl Into<String>, subject: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            to: vec![to.into()],
            cc: Vec::new(),
            subject: subject.into(),
            body: body.into(),
            html_body: None,
        }
    }

    pub fn with_cc(mut self, cc: impl Into<String>) -> Self {
        self.cc.push(cc.into());
        self
    }

    pub fn with_html(mut self, html: impl Into<String>) -> Self {
        self.html_body = Some(html.into());
        self
    }

    pub fn to_multiple(mut self, recipients: Vec<String>) -> Self {
        self.to = recipients;
        self
    }
}

/// Email provider trait
#[async_trait::async_trait]
pub trait EmailProvider: Send + Sync {
    async fn send(&self, message: EmailMessage) -> Result<(), ApiError>;
}

/// SMTP email provider using lettre
pub struct SmtpEmailProvider {
    config: EmailConfig,
}

impl SmtpEmailProvider {
    pub fn new(config: EmailConfig) -> Self {
        Self { config }
    }

    fn build_transport(&self) -> Result<AsyncSmtpTransport<Tokio1Executor>, ApiError> {
        let creds = Credentials::new(
            self.config.username.clone(),
            self.config.password.clone(),
        );

        let transport = if self.config.use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.smtp_host)
                .map_err(|e| ApiError::InternalServerError(format!("SMTP relay error: {}", e)))?
                .credentials(creds)
                .port(self.config.port)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.config.smtp_host)
                .credentials(creds)
                .port(self.config.port)
                .build()
        };

        Ok(transport)
    }
}

#[async_trait::async_trait]
impl EmailProvider for SmtpEmailProvider {
    async fn send(&self, message: EmailMessage) -> Result<(), ApiError> {
        if message.to.is_empty() {
            return Err(ApiError::BadRequest("No recipients specified".to_string()));
        }

        let from: Mailbox = self.config.from.parse()
            .map_err(|e| ApiError::BadRequest(format!("Invalid from address: {}", e)))?;

        let mut builder = Message::builder().from(from);

        for recipient in &message.to {
            let mailbox: Mailbox = recipient.parse()
                .map_err(|e| ApiError::BadRequest(format!("Invalid recipient: {}", e)))?;
            builder = builder.to(mailbox);
        }

        for cc_addr in &message.cc {
            let mailbox: Mailbox = cc_addr.parse()
                .map_err(|e| ApiError::BadRequest(format!("Invalid CC address: {}", e)))?;
            builder = builder.cc(mailbox);
        }

        let email = if let Some(html) = &message.html_body {
            use lettre::message::{MultiPart, SinglePart};
            builder
                .subject(&message.subject)
                .multipart(
                    MultiPart::alternative()
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_PLAIN)
                                .body(message.body.clone()),
                        )
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(html.clone()),
                        ),
                )
                .map_err(|e| ApiError::InternalServerError(format!("Email build error: {}", e)))?
        } else {
            builder
                .subject(&message.subject)
                .header(ContentType::TEXT_PLAIN)
                .body(message.body.clone())
                .map_err(|e| ApiError::InternalServerError(format!("Email build error: {}", e)))?
        };

        let transport = self.build_transport()?;
        transport.send(email).await
            .map_err(|e| ApiError::InternalServerError(format!("Email send error: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_message_builder() {
        let msg = EmailMessage::new("to@example.com", "Test Subject", "Hello!")
            .with_cc("cc@example.com")
            .with_html("<p>Hello!</p>");

        assert_eq!(msg.to, vec!["to@example.com"]);
        assert_eq!(msg.cc, vec!["cc@example.com"]);
        assert_eq!(msg.subject, "Test Subject");
        assert_eq!(msg.body, "Hello!");
        assert_eq!(msg.html_body, Some("<p>Hello!</p>".to_string()));
    }

    #[test]
    fn test_email_config() {
        let config = EmailConfig::new()
            .with_smtp_host("smtp.gmail.com")
            .with_port(587)
            .with_tls()
            .with_credentials("user@gmail.com", "pass")
            .with_from("App <app@gmail.com>");

        assert_eq!(config.smtp_host, "smtp.gmail.com");
        assert_eq!(config.port, 587);
        assert!(config.use_tls);
    }
}
