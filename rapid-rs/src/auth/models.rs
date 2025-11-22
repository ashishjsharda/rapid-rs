//! Authentication request and response models

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Login request payload
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    /// User email address
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    /// User password
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

/// Registration request payload
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    /// User email address
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    /// User password (min 8 chars, must include uppercase, lowercase, and digit)
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    
    /// User's display name
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    pub name: String,
}

/// Token refresh request
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct TokenRefreshRequest {
    /// The refresh token
    #[validate(length(min = 1, message = "Refresh token is required"))]
    pub refresh_token: String,
}

/// Authentication response containing tokens
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuthResponse {
    /// JWT access token (short-lived)
    pub access_token: String,
    
    /// JWT refresh token (long-lived)
    pub refresh_token: String,
    
    /// Token type (always "Bearer")
    pub token_type: String,
    
    /// Access token expiration time in seconds
    pub expires_in: u64,
    
    /// Authenticated user information
    pub user: AuthUserInfo,
}

/// User information returned in auth responses
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuthUserInfo {
    /// User ID
    pub id: String,
    
    /// User email
    pub email: String,
    
    /// User's display name
    pub name: String,
    
    /// User roles
    pub roles: Vec<String>,
}

/// Logout request (optional - for refresh token invalidation)
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct LogoutRequest {
    /// The refresh token to invalidate
    pub refresh_token: Option<String>,
}

/// Password change request
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct ChangePasswordRequest {
    /// Current password
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,
    
    /// New password
    #[validate(length(min = 8, message = "New password must be at least 8 characters"))]
    pub new_password: String,
}

/// Password reset request
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct PasswordResetRequest {
    /// Email address to send reset link to
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

/// Password reset confirmation
#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct PasswordResetConfirm {
    /// Reset token from email
    #[validate(length(min = 1, message = "Reset token is required"))]
    pub token: String,
    
    /// New password
    #[validate(length(min = 8, message = "New password must be at least 8 characters"))]
    pub new_password: String,
}

/// Generic message response
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

impl MessageResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}
