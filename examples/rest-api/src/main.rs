use rapid_rs::prelude::*;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
struct User {
    id: Uuid,
    email: String,
    name: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize, Validate, ToSchema)]
struct CreateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    email: String,
    
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    name: String,
}

#[derive(Deserialize, Validate, ToSchema)]
struct UpdateUserRequest {
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    name: Option<String>,
}

// In-memory "database" for demo purposes
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

type Database = Arc<Mutex<HashMap<Uuid, User>>>;

/// Create a new user
async fn create_user(
    State(db): State<Database>,
    ValidatedJson(payload): ValidatedJson<CreateUserRequest>,
) -> ApiResult<User> {
    let user = User {
        id: Uuid::new_v4(),
        email: payload.email,
        name: payload.name,
        created_at: Utc::now(),
    };

    db.lock().unwrap().insert(user.id, user.clone());
    
    Ok(Json(user))
}

/// Get all users
async fn list_users(State(db): State<Database>) -> ApiResult<Vec<User>> {
    let users: Vec<User> = db.lock().unwrap().values().cloned().collect();
    Ok(Json(users))
}

/// Get a user by ID
async fn get_user(
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, ApiError> {
    let db = db.lock().unwrap();
    let user = db
        .get(&id)
        .ok_or_else(|| ApiError::NotFound(format!("User with id {} not found", id)))?;
    
    Ok(Json(user.clone()))
}

/// Update a user
async fn update_user(
    State(db): State<Database>,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateUserRequest>,
) -> Result<Json<User>, ApiError> {
    let mut db = db.lock().unwrap();
    let user = db
        .get_mut(&id)
        .ok_or_else(|| ApiError::NotFound(format!("User with id {} not found", id)))?;
    
    if let Some(name) = payload.name {
        user.name = name;
    }
    
    Ok(Json(user.clone()))
}

/// Delete a user
async fn delete_user(
    State(db): State<Database>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, ApiError> {
    let mut db = db.lock().unwrap();
    let user = db
        .remove(&id)
        .ok_or_else(|| ApiError::NotFound(format!("User with id {} not found", id)))?;
    
    Ok(Json(user))
}

fn user_routes() -> Router<Database> {
    Router::new()
        .route("/users", post(create_user))
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user))
        .route("/users/:id", patch(update_user))
        .route("/users/:id", delete(delete_user))
}

#[tokio::main]
async fn main() {
    // Create shared database
    let db: Database = Arc::new(Mutex::new(HashMap::new()));

    // Build and run the app
    App::new()
        .auto_configure()
        .mount(user_routes().with_state(db))
        .run()
        .await
        .unwrap();
}
