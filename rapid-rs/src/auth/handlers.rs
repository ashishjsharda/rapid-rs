//! Authentication route handlers

use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};

use super::{
    config::AuthConfig,
    jwt::{create_token_pair, verify_refresh_token},
    models::*,
    extractors::AuthUser,
};
use crate::error::ApiError;
use crate::extractors::ValidatedJson;

/// User storage trait - implement this for your database
/// 
/// This trait defines the interface for user storage operations.
/// Implement this for your specific database (PostgreSQL, MySQL, etc.)
/// 
/// # Example
/// 
/// ```rust,ignore
/// use rapid_rs::auth::{UserStore, StoredUser};
/// use sqlx::PgPool;
/// 
/// struct PostgresUserStore {
///     pool: PgPool,
/// }
/// 
/// #[async_trait]
/// impl UserStore for PostgresUserStore {
///     async fn find_by_email(&self, email: &str) -> Result<Option<StoredUser>, ApiError> {
///         let user = sqlx::query_as!(
///             StoredUser,
///             "SELECT id, email, name, password_hash, roles FROM users WHERE email = $1",
///             email
///         )
///         .fetch_optional(&self.pool)
///         .await?;
///         Ok(user)
///     }
///     
///     // ... implement other methods
/// }
/// ```
#[async_trait::async_trait]
pub trait UserStore: Send + Sync + 'static {
    /// Find a user by email
    async fn find_by_email(&self, email: &str) -> Result<Option<StoredUser>, ApiError>;
    
    /// Find a user by ID
    async fn find_by_id(&self, id: &str) -> Result<Option<StoredUser>, ApiError>;
    
    /// Create a new user
    async fn create(&self, user: CreateUserData) -> Result<StoredUser, ApiError>;
    
    /// Update user's password hash
    async fn update_password(&self, id: &str, password_hash: &str) -> Result<(), ApiError>;
    
    /// Check if email is already taken
    async fn email_exists(&self, email: &str) -> Result<bool, ApiError>;
}

/// Stored user data from database
#[derive(Debug, Clone)]
pub struct StoredUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub roles: Vec<String>,
}

/// Data for creating a new user
#[derive(Debug, Clone)]
pub struct CreateUserData {
    pub email: String,
    pub name: String,
    pub password_hash: String,
}

/// In-memory user store for development/testing
/// 
/// **WARNING: Do not use in production!**
/// This is only for development and testing purposes.
#[derive(Clone, Default)]
pub struct InMemoryUserStore {
    users: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, StoredUser>>>,
}

impl InMemoryUserStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl UserStore for InMemoryUserStore {
    async fn find_by_email(&self, email: &str) -> Result<Option<StoredUser>, ApiError> {
        let users = self.users.lock().unwrap();
        Ok(users.values().find(|u| u.email == email).cloned())
    }
    
    async fn find_by_id(&self, id: &str) -> Result<Option<StoredUser>, ApiError> {
        let users = self.users.lock().unwrap();
        Ok(users.get(id).cloned())
    }
    
    async fn create(&self, user: CreateUserData) -> Result<StoredUser, ApiError> {
        let mut users = self.users.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let stored = StoredUser {
            id: id.clone(),
            email: user.email,
            name: user.name,
            password_hash: user.password_hash,
            roles: vec!["user".to_string()],
        };
        users.insert(id, stored.clone());
        Ok(stored)
    }
    
    async fn update_password(&self, id: &str, password_hash: &str) -> Result<(), ApiError> {
        let mut users = self.users.lock().unwrap();
        if let Some(user) = users.get_mut(id) {
            user.password_hash = password_hash.to_string();
            Ok(())
        } else {
            Err(ApiError::NotFound("User not found".to_string()))
        }
    }
    
    async fn email_exists(&self, email: &str) -> Result<bool, ApiError> {
        let users = self.users.lock().unwrap();
        Ok(users.values().any(|u| u.email == email))
    }
}

/// Application state for auth routes
#[derive(Clone)]
pub struct AuthAppState<S: UserStore> {
    pub config: AuthConfig,
    pub user_store: S,
}

/// Login handler
/// 
/// Authenticates a user with email and password, returns JWT tokens.
pub async fn login<S: UserStore>(
    State(state): State<AuthAppState<S>>,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Find user by email
    let user = state
        .user_store
        .find_by_email(&payload.email)
        .await?
        .ok_or_else(|| ApiError::Unauthorized)?;
    
    // Verify password
    let password_valid = super::password::verify_password(&payload.password, &user.password_hash)?;
    if !password_valid {
        return Err(ApiError::Unauthorized);
    }
    
    // Generate tokens
    let token_pair = create_token_pair(&user.id, &user.email, user.roles.clone(), &state.config)?;
    
    Ok(Json(AuthResponse {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        token_type: token_pair.token_type,
        expires_in: token_pair.expires_in,
        user: AuthUserInfo {
            id: user.id,
            email: user.email,
            name: user.name,
            roles: user.roles,
        },
    }))
}

/// Registration handler
/// 
/// Creates a new user account and returns JWT tokens.
pub async fn register<S: UserStore>(
    State(state): State<AuthAppState<S>>,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Validate password strength
    super::password::validate_password_strength(&payload.password)?;
    
    // Check if email is already taken
    if state.user_store.email_exists(&payload.email).await? {
        return Err(ApiError::BadRequest("Email already registered".to_string()));
    }
    
    // Hash password
    let password_hash = super::password::hash_password(&payload.password, &state.config)?;
    
    // Create user
    let user = state
        .user_store
        .create(CreateUserData {
            email: payload.email,
            name: payload.name,
            password_hash,
        })
        .await?;
    
    // Generate tokens
    let token_pair = create_token_pair(&user.id, &user.email, user.roles.clone(), &state.config)?;
    
    tracing::info!(user_id = %user.id, "New user registered");
    
    Ok(Json(AuthResponse {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        token_type: token_pair.token_type,
        expires_in: token_pair.expires_in,
        user: AuthUserInfo {
            id: user.id,
            email: user.email,
            name: user.name,
            roles: user.roles,
        },
    }))
}

/// Refresh token handler
/// 
/// Exchanges a refresh token for a new access/refresh token pair.
pub async fn refresh_token<S: UserStore>(
    State(state): State<AuthAppState<S>>,
    ValidatedJson(payload): ValidatedJson<TokenRefreshRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Verify refresh token
    let claims = verify_refresh_token(&payload.refresh_token, &state.config)?;
    
    // Get user (to ensure they still exist and get current roles)
    let user = state
        .user_store
        .find_by_id(&claims.sub)
        .await?
        .ok_or_else(|| ApiError::Unauthorized)?;
    
    // Generate new tokens
    let token_pair = create_token_pair(&user.id, &user.email, user.roles.clone(), &state.config)?;
    
    Ok(Json(AuthResponse {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        token_type: token_pair.token_type,
        expires_in: token_pair.expires_in,
        user: AuthUserInfo {
            id: user.id,
            email: user.email,
            name: user.name,
            roles: user.roles,
        },
    }))
}

/// Logout handler
/// 
/// For stateless JWT, this is a no-op on the server side.
/// In a production app, you might want to:
/// - Add the token to a blacklist
/// - Invalidate the refresh token in the database
pub async fn logout() -> Json<MessageResponse> {
    // For stateless JWT, logout is handled client-side by discarding tokens
    // In production, you might want to blacklist the token or invalidate refresh tokens
    Json(MessageResponse::new("Successfully logged out"))
}

/// Get current user info
pub async fn me<S: UserStore>(
    user: AuthUser,
    State(state): State<AuthAppState<S>>,
) -> Result<Json<AuthUserInfo>, ApiError> {
    let stored_user = state
        .user_store
        .find_by_id(&user.id)
        .await?
        .ok_or_else(|| ApiError::NotFound("User not found".to_string()))?;
    
    Ok(Json(AuthUserInfo {
        id: stored_user.id,
        email: stored_user.email,
        name: stored_user.name,
        roles: stored_user.roles,
    }))
}

/// Create auth routes with a custom user store
/// 
/// # Example
/// 
/// ```rust,ignore
/// use rapid_rs::auth::{auth_routes_with_store, AuthConfig, InMemoryUserStore};
/// 
/// let config = AuthConfig::default();
/// let store = InMemoryUserStore::new();
/// 
/// let routes = auth_routes_with_store(config, store);
/// ```
pub fn auth_routes_with_store<S: UserStore + Clone>(
    config: AuthConfig,
    user_store: S,
) -> Router {
    let state = AuthAppState {
        config: config.clone(),
        user_store,
    };
    
    Router::new()
        .route("/auth/login", post(login::<S>))
        .route("/auth/register", post(register::<S>))
        .route("/auth/refresh", post(refresh_token::<S>))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(me::<S>))
        .with_state(state)
}

/// Create auth routes with in-memory store (for development)
/// 
/// **WARNING: Do not use in production!**
pub fn auth_routes(config: AuthConfig) -> Router {
    auth_routes_with_store(config, InMemoryUserStore::new())
}
