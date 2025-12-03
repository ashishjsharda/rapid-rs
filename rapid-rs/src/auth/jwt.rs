//! JWT token generation and verification

use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::config::AuthConfig;
use crate::error::ApiError;

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,

    /// User email
    pub email: String,

    /// User role(s)
    #[serde(default)]
    pub roles: Vec<String>,

    /// Token type: "access" or "refresh"
    pub token_type: String,

    /// Issued at (Unix timestamp)
    pub iat: i64,

    /// Expiration time (Unix timestamp)
    pub exp: i64,

    /// Not before (Unix timestamp)
    pub nbf: i64,

    /// Issuer
    pub iss: String,

    /// Audience
    pub aud: String,

    /// JWT ID (unique identifier for this token)
    pub jti: String,
}

impl Claims {
    /// Create new claims for an access token
    pub fn new_access(
        user_id: impl Into<String>,
        email: impl Into<String>,
        roles: Vec<String>,
        config: &AuthConfig,
    ) -> Self {
        let now = Utc::now();
        let exp = now + Duration::seconds(config.access_token_expiry_secs as i64);

        Self {
            sub: user_id.into(),
            email: email.into(),
            roles,
            token_type: "access".to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            nbf: now.timestamp(),
            iss: config.issuer.clone(),
            aud: config.audience.clone(),
            jti: Uuid::new_v4().to_string(),
        }
    }

    /// Create new claims for a refresh token
    pub fn new_refresh(
        user_id: impl Into<String>,
        email: impl Into<String>,
        config: &AuthConfig,
    ) -> Self {
        let now = Utc::now();
        let exp = now + Duration::seconds(config.refresh_token_expiry_secs as i64);

        Self {
            sub: user_id.into(),
            email: email.into(),
            roles: vec![],
            token_type: "refresh".to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            nbf: now.timestamp(),
            iss: config.issuer.clone(),
            aud: config.audience.clone(),
            jti: Uuid::new_v4().to_string(),
        }
    }

    /// Check if this is an access token
    pub fn is_access_token(&self) -> bool {
        self.token_type == "access"
    }

    /// Check if this is a refresh token
    pub fn is_refresh_token(&self) -> bool {
        self.token_type == "refresh"
    }

    /// Check if the user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if the user has any of the specified roles
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }

    /// Check if the user has all of the specified roles
    pub fn has_all_roles(&self, roles: &[&str]) -> bool {
        roles.iter().all(|role| self.has_role(role))
    }
}

/// A pair of access and refresh tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// The access token (short-lived)
    pub access_token: String,

    /// The refresh token (long-lived)
    pub refresh_token: String,

    /// Token type (always "Bearer")
    pub token_type: String,

    /// Access token expiration time in seconds
    pub expires_in: u64,
}

/// Create a new token pair for a user
pub fn create_token_pair(
    user_id: impl Into<String>,
    email: impl Into<String>,
    roles: Vec<String>,
    config: &AuthConfig,
) -> Result<TokenPair, ApiError> {
    let user_id = user_id.into();
    let email = email.into();

    // Create access token
    let access_claims = Claims::new_access(&user_id, &email, roles, config);
    let access_token = encode(
        &Header::new(Algorithm::HS256),
        &access_claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| ApiError::InternalServerError(format!("Failed to create access token: {}", e)))?;

    // Create refresh token
    let refresh_claims = Claims::new_refresh(&user_id, &email, config);
    let refresh_token = encode(
        &Header::new(Algorithm::HS256),
        &refresh_claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| ApiError::InternalServerError(format!("Failed to create refresh token: {}", e)))?;

    Ok(TokenPair {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: config.access_token_expiry_secs,
    })
}

/// Verify a JWT token and return the claims
pub fn verify_token(token: &str, config: &AuthConfig) -> Result<Claims, ApiError> {
    // Sanitize the token string to remove quotes or whitespace
    let token = token.trim().trim_matches('"');
    tracing::debug!("Verifying JWT: {}", token);

    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[&config.issuer]);
    validation.set_audience(&[&config.audience]);
    validation.validate_exp = true;
    validation.validate_nbf = true;

    let token_data: TokenData<Claims> = decode(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &validation,
    )
    .map_err(|e| {
        tracing::debug!("Token verification failed: {}", e);
        match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => ApiError::Unauthorized,
            jsonwebtoken::errors::ErrorKind::InvalidToken => ApiError::Unauthorized,
            _ => ApiError::Unauthorized,
        }
    })?;

    Ok(token_data.claims)
}

/// Verify that a token is an access token
pub fn verify_access_token(token: &str, config: &AuthConfig) -> Result<Claims, ApiError> {
    let claims = verify_token(token, config)?;

    if !claims.is_access_token() {
        return Err(ApiError::Unauthorized);
    }

    Ok(claims)
}

/// Verify that a token is a refresh token
pub fn verify_refresh_token(token: &str, config: &AuthConfig) -> Result<Claims, ApiError> {
    let claims = verify_token(token, config)?;

    if !claims.is_refresh_token() {
        return Err(ApiError::Unauthorized);
    }

    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_verify_token() {
        let config = AuthConfig::default();
        let token_pair = create_token_pair(
            "user-123",
            "test@example.com",
            vec!["user".to_string()],
            &config,
        )
        .unwrap();

        let claims = verify_access_token(&token_pair.access_token, &config).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.email, "test@example.com");
        assert!(claims.has_role("user"));
    }

    #[test]
    fn test_refresh_token() {
        let config = AuthConfig::default();
        let token_pair =
            create_token_pair("user-123", "test@example.com", vec![], &config).unwrap();

        let claims = verify_refresh_token(&token_pair.refresh_token, &config).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert!(claims.is_refresh_token());
    }
}
