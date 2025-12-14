use std::fs;
use std::path::Path;

pub fn generate_grpc_template(base: &Path, name: &str) -> anyhow::Result<()> {
    // Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
rapid-rs = "0.2"
tokio = {{ version = "1", features = ["full"] }}
tonic = "0.11"
prost = "0.12"
serde = {{ version = "1.0", features = ["derive"] }}
uuid = {{ version = "1.0", features = ["v4"] }}

[build-dependencies]
tonic-build = "0.11"
"#,
        name
    );
    fs::write(base.join("Cargo.toml"), cargo_toml)?;

    // build.rs
    let build_rs = r#"fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/user.proto")?;
    Ok(())
}
"#;
    fs::write(base.join("build.rs"), build_rs)?;

    // Create proto directory
    fs::create_dir_all(base.join("proto"))?;

    // user.proto
    let user_proto = r#"syntax = "proto3";

package user;

service UserService {
  rpc GetUser (GetUserRequest) returns (User);
  rpc ListUsers (ListUsersRequest) returns (ListUsersResponse);
  rpc CreateUser (CreateUserRequest) returns (User);
  rpc UpdateUser (UpdateUserRequest) returns (User);
  rpc DeleteUser (DeleteUserRequest) returns (DeleteUserResponse);
}

message User {
  string id = 1;
  string name = 2;
  string email = 3;
  string created_at = 4;
}

message GetUserRequest {
  string id = 1;
}

message ListUsersRequest {
  int32 page = 1;
  int32 page_size = 2;
}

message ListUsersResponse {
  repeated User users = 1;
  int32 total = 2;
}

message CreateUserRequest {
  string name = 1;
  string email = 2;
}

message UpdateUserRequest {
  string id = 1;
  optional string name = 2;
  optional string email = 3;
}

message DeleteUserRequest {
  string id = 1;
}

message DeleteUserResponse {
  bool success = 1;
}
"#;
    fs::write(base.join("proto/user.proto"), user_proto)?;

    // main.rs
    let main_rs = r#"use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

// Import the generated code
pub mod user {
    tonic::include_proto!("user");
}

use user::{
    user_service_server::{UserService, UserServiceServer},
    CreateUserRequest, DeleteUserRequest, DeleteUserResponse, GetUserRequest,
    ListUsersRequest, ListUsersResponse, UpdateUserRequest, User,
};

#[derive(Debug, Default)]
pub struct UserServiceImpl {}

#[tonic::async_trait]
impl UserService for UserServiceImpl {
    async fn get_user(&self, request: Request<GetUserRequest>) -> Result<Response<User>, Status> {
        let req = request.into_inner();
        
        // Mock implementation - replace with database query
        let user = User {
            id: req.id,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        Ok(Response::new(user))
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let _req = request.into_inner();
        
        // Mock implementation - replace with database query
        let users = vec![
            User {
                id: Uuid::new_v4().to_string(),
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            },
            User {
                id: Uuid::new_v4().to_string(),
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        ];
        
        Ok(Response::new(ListUsersResponse {
            users,
            total: 2,
        }))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<User>, Status> {
        let req = request.into_inner();
        
        // Validate request
        if req.name.is_empty() {
            return Err(Status::invalid_argument("Name is required"));
        }
        if req.email.is_empty() {
            return Err(Status::invalid_argument("Email is required"));
        }
        
        // Mock implementation - replace with database insert
        let user = User {
            id: Uuid::new_v4().to_string(),
            name: req.name,
            email: req.email,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        Ok(Response::new(user))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<User>, Status> {
        let req = request.into_inner();
        
        // Mock implementation - replace with database update
        let user = User {
            id: req.id,
            name: req.name.unwrap_or_else(|| "Updated Name".to_string()),
            email: req.email.unwrap_or_else(|| "updated@example.com".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        Ok(Response::new(user))
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let _req = request.into_inner();
        
        // Mock implementation - replace with database delete
        Ok(Response::new(DeleteUserResponse { success: true }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let user_service = UserServiceImpl::default();

    println!("🚀 gRPC server starting on {}", addr);
    println!("💡 Use a gRPC client like grpcurl or BloomRPC to test");

    Server::builder()
        .add_service(UserServiceServer::new(user_service))
        .serve(addr)
        .await?;

    Ok(())
}
"#;
    fs::write(base.join("src/main.rs"), main_rs)?;

    // README
    let readme = format!(
        r#"# {} - gRPC API

A rapid-rs gRPC API project.

## Quick Start

```bash
cargo build
cargo run
```

Server will start on `0.0.0.0:50051`

## Testing with grpcurl

Install grpcurl: https://github.com/fullstorydev/grpcurl

### List services
```bash
grpcurl -plaintext localhost:50051 list
```

### Get a user
```bash
grpcurl -plaintext -d '{{"id": "123"}}' localhost:50051 user.UserService/GetUser
```

### List all users
```bash
grpcurl -plaintext -d '{{}}' localhost:50051 user.UserService/ListUsers
```

### Create a user
```bash
grpcurl -plaintext -d '{{"name": "Jane Doe", "email": "jane@example.com"}}' \
  localhost:50051 user.UserService/CreateUser
```

## Project Structure

- `proto/` - Protocol buffer definitions
- `src/main.rs` - gRPC service implementation
- `build.rs` - Build script that generates Rust code from proto files

## Next Steps

1. Connect to a real database
2. Add authentication (JWT in metadata)
3. Implement proper error handling
4. Add streaming RPCs for real-time data
5. Add health checks and reflection
"#,
        name
    );
    fs::write(base.join("README.md"), readme)?;

    // .gitignore
    let gitignore = r#"/target
/Cargo.lock
.env
"#;
    fs::write(base.join(".gitignore"), gitignore)?;

    Ok(())
}
