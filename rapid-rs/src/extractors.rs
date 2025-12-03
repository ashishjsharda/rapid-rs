use axum::{
    async_trait,
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{de::DeserializeOwned, Serialize};
use validator::Validate;

/// Extractor that deserializes and validates JSON payloads
///
/// # Example
///
/// ```rust,ignore
/// use rapid_rs::prelude::*;
/// use validator::Validate;
///
/// #[derive(Deserialize, Validate)]
/// struct CreateUser {
///     #[validate(email)]
///     email: String,
///     #[validate(length(min = 8))]
///     password: String,
/// }
///
/// async fn create_user(
///     ValidatedJson(payload): ValidatedJson<CreateUser>
/// ) -> ApiResult<User> {
///     // payload is guaranteed to be valid
///     Ok(Json(user))
/// }
/// ```
pub struct ValidatedJson<T>(pub T);

#[derive(Serialize)]
struct ValidationErrorResponse {
    code: String,
    message: String,
    errors: Vec<ValidationFieldError>,
}

#[derive(Serialize)]
struct ValidationFieldError {
    field: String,
    message: String,
}

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // First, extract JSON
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| {
                tracing::error!("JSON deserialization failed: {:?}", rejection);

                let error_response = ValidationErrorResponse {
                    code: "INVALID_JSON".to_string(),
                    message: "Invalid JSON payload".to_string(),
                    errors: vec![],
                };

                (StatusCode::BAD_REQUEST, Json(error_response)).into_response()
            })?;

        // Then validate
        value.validate().map_err(|validation_errors| {
            tracing::error!("Validation failed: {:?}", validation_errors);

            let errors: Vec<ValidationFieldError> = validation_errors
                .field_errors()
                .into_iter()
                .flat_map(|(field, errors)| {
                    errors.iter().map(move |error| ValidationFieldError {
                        field: field.to_string(),
                        message: error
                            .message
                            .as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| "Validation failed".to_string()),
                    })
                })
                .collect();

            let error_response = ValidationErrorResponse {
                code: "VALIDATION_ERROR".to_string(),
                message: "Request validation failed".to_string(),
                errors,
            };

            (StatusCode::UNPROCESSABLE_ENTITY, Json(error_response)).into_response()
        })?;

        Ok(ValidatedJson(value))
    }
}
