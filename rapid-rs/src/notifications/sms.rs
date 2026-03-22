//! SMS notification support via Twilio API

use reqwest::Client;
use serde::Deserialize;
use crate::error::ApiError;

/// SMS provider configuration
#[derive(Debug, Clone)]
pub struct SmsConfig {
    /// Twilio Account SID
    pub account_sid: String,
    /// Twilio Auth Token
    pub auth_token: String,
    /// Default "from" phone number (Twilio number)
    pub from_number: String,
    /// API base URL (override for testing)
    pub api_url: String,
}

impl SmsConfig {
    pub fn new(account_sid: impl Into<String>, auth_token: impl Into<String>, from_number: impl Into<String>) -> Self {
        let account_sid = account_sid.into();
        let api_url = format!("https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json", account_sid);
        Self {
            account_sid,
            auth_token: auth_token.into(),
            from_number: from_number.into(),
            api_url,
        }
    }

    pub fn with_api_url(mut self, url: impl Into<String>) -> Self {
        self.api_url = url.into();
        self
    }
}

/// SMS message
#[derive(Debug, Clone)]
pub struct SmsMessage {
    /// Recipient phone number (E.164 format, e.g., +1234567890)
    pub to: String,
    /// Optional override for from number
    pub from: Option<String>,
    /// Message body (max 1600 chars for SMS)
    pub body: String,
}

impl SmsMessage {
    pub fn new(to: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            to: to.into(),
            from: None,
            body: body.into(),
        }
    }

    pub fn with_from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }
}

/// Twilio API response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TwilioResponse {
    sid: Option<String>,
    status: Option<String>,
    error_code: Option<u32>,
    error_message: Option<String>,
}

/// SMS provider trait
#[async_trait::async_trait]
pub trait SmsProvider: Send + Sync {
    async fn send(&self, message: SmsMessage) -> Result<(), ApiError>;
}

/// Twilio SMS provider
pub struct TwilioSmsProvider {
    config: SmsConfig,
    client: Client,
}

impl TwilioSmsProvider {
    pub fn new(config: SmsConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl SmsProvider for TwilioSmsProvider {
    async fn send(&self, message: SmsMessage) -> Result<(), ApiError> {
        if message.to.is_empty() {
            return Err(ApiError::BadRequest("No recipient phone number specified".to_string()));
        }

        let from = message.from.as_deref().unwrap_or(&self.config.from_number);

        let params = [
            ("To", message.to.as_str()),
            ("From", from),
            ("Body", message.body.as_str()),
        ];

        let response = self.client
            .post(&self.config.api_url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .form(&params)
            .send()
            .await
            .map_err(|e| ApiError::InternalServerError(format!("SMS request error: {}", e)))?;

        let status = response.status();

        if !status.is_success() {
            let error_body: TwilioResponse = response.json().await
                .unwrap_or(TwilioResponse {
                    sid: None,
                    status: Some(status.to_string()),
                    error_code: None,
                    error_message: Some("Unknown error".to_string()),
                });

            return Err(ApiError::InternalServerError(format!(
                "Twilio error {}: {}",
                error_body.error_code.unwrap_or(0),
                error_body.error_message.unwrap_or_else(|| "Unknown".to_string())
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sms_message() {
        let msg = SmsMessage::new("+1234567890", "Hello!")
            .with_from("+0987654321");

        assert_eq!(msg.to, "+1234567890");
        assert_eq!(msg.from, Some("+0987654321".to_string()));
        assert_eq!(msg.body, "Hello!");
    }

    #[test]
    fn test_sms_config() {
        let config = SmsConfig::new("ACxxxxx", "auth_token", "+15005550006");
        assert_eq!(config.from_number, "+15005550006");
        assert!(config.api_url.contains("ACxxxxx"));
    }
}
