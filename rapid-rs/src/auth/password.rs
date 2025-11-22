//! Password hashing utilities using Argon2

use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2, Algorithm, Params, Version,
};

use super::config::AuthConfig;
use crate::error::ApiError;

/// Hash a password using Argon2id
/// 
/// # Example
/// 
/// ```rust,ignore
/// use rapid_rs::auth::{hash_password, AuthConfig};
/// 
/// let config = AuthConfig::default();
/// let hashed = hash_password("my-secure-password", &config)?;
/// ```
pub fn hash_password(password: &str, config: &AuthConfig) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    
    let params = Params::new(
        config.argon2_memory_cost,
        config.argon2_time_cost,
        config.argon2_parallelism,
        None,
    ).map_err(|e| ApiError::InternalServerError(format!("Invalid Argon2 params: {}", e)))?;
    
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| ApiError::InternalServerError(format!("Failed to hash password: {}", e)))?
        .to_string();
    
    Ok(password_hash)
}

/// Hash a password with default configuration
/// 
/// Uses sensible defaults for Argon2 parameters.
pub fn hash_password_default(password: &str) -> Result<String, ApiError> {
    hash_password(password, &AuthConfig::default())
}

/// Verify a password against a hash
/// 
/// # Example
/// 
/// ```rust,ignore
/// use rapid_rs::auth::{hash_password, verify_password, AuthConfig};
/// 
/// let config = AuthConfig::default();
/// let hashed = hash_password("my-secure-password", &config)?;
/// 
/// assert!(verify_password("my-secure-password", &hashed)?);
/// assert!(!verify_password("wrong-password", &hashed)?);
/// ```
pub fn verify_password(password: &str, hash: &str) -> Result<bool, ApiError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| ApiError::InternalServerError(format!("Invalid password hash: {}", e)))?;
    
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Validate password strength
/// 
/// Returns an error if the password doesn't meet minimum requirements:
/// - At least 8 characters
/// - Contains at least one uppercase letter
/// - Contains at least one lowercase letter
/// - Contains at least one digit
pub fn validate_password_strength(password: &str) -> Result<(), ApiError> {
    if password.len() < 8 {
        return Err(ApiError::ValidationError(
            "Password must be at least 8 characters long".to_string(),
        ));
    }
    
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ApiError::ValidationError(
            "Password must contain at least one uppercase letter".to_string(),
        ));
    }
    
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(ApiError::ValidationError(
            "Password must contain at least one lowercase letter".to_string(),
        ));
    }
    
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(ApiError::ValidationError(
            "Password must contain at least one digit".to_string(),
        ));
    }
    
    Ok(())
}

/// Validate password strength with custom rules
pub struct PasswordValidator {
    min_length: usize,
    require_uppercase: bool,
    require_lowercase: bool,
    require_digit: bool,
    require_special: bool,
}

impl PasswordValidator {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn min_length(mut self, length: usize) -> Self {
        self.min_length = length;
        self
    }
    
    pub fn require_uppercase(mut self, required: bool) -> Self {
        self.require_uppercase = required;
        self
    }
    
    pub fn require_lowercase(mut self, required: bool) -> Self {
        self.require_lowercase = required;
        self
    }
    
    pub fn require_digit(mut self, required: bool) -> Self {
        self.require_digit = required;
        self
    }
    
    pub fn require_special(mut self, required: bool) -> Self {
        self.require_special = required;
        self
    }
    
    pub fn validate(&self, password: &str) -> Result<(), ApiError> {
        let mut errors = Vec::new();
        
        if password.len() < self.min_length {
            errors.push(format!(
                "Password must be at least {} characters long",
                self.min_length
            ));
        }
        
        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain at least one uppercase letter".to_string());
        }
        
        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain at least one lowercase letter".to_string());
        }
        
        if self.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
            errors.push("Password must contain at least one digit".to_string());
        }
        
        if self.require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
            errors.push("Password must contain at least one special character".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ApiError::ValidationError(errors.join("; ")))
        }
    }
}

impl Default for PasswordValidator {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_and_verify() {
        let config = AuthConfig::default();
        let password = "SecurePass123";
        
        let hash = hash_password(password, &config).unwrap();
        
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong-password", &hash).unwrap());
    }
    
    #[test]
    fn test_password_validation() {
        assert!(validate_password_strength("SecurePass1").is_ok());
        assert!(validate_password_strength("short").is_err());
        assert!(validate_password_strength("nouppercase1").is_err());
        assert!(validate_password_strength("NOLOWERCASE1").is_err());
        assert!(validate_password_strength("NoDigitsHere").is_err());
    }
    
    #[test]
    fn test_custom_validator() {
        let validator = PasswordValidator::new()
            .min_length(12)
            .require_special(true);
        
        assert!(validator.validate("SecurePass1!").is_ok());
        assert!(validator.validate("SecurePass1").is_err()); // No special char
    }
}
