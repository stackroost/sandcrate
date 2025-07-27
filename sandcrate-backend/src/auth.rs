use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use pam::Authenticator;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub struct AuthConfig {
    pub jwt_secret: String,
}

impl AuthConfig {
    pub fn new() -> Self {
        Self {
            jwt_secret: "your-secret-key-change-in-production".to_string(),
        }
    }
}

pub async fn login(
    State(config): State<Arc<AuthConfig>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Authenticate using PAM
    let mut authenticator = Authenticator::with_password("login")
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to initialize PAM".to_string(),
                }),
            )
        })?;

    authenticator
        .get_handler()
        .set_credentials(&payload.username, &payload.password);

    match authenticator.authenticate() {
        Ok(_) => {
            // Generate JWT token
            let now = Utc::now();
            let expires_at = now + Duration::hours(24);
            
            let claims = Claims {
                sub: payload.username.clone(),
                exp: expires_at.timestamp() as usize,
                iat: now.timestamp() as usize,
            };

            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(config.jwt_secret.as_ref()),
            )
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "Failed to generate token".to_string(),
                    }),
                )
            })?;

            Ok(Json(LoginResponse {
                token,
                username: payload.username,
                expires_at: expires_at.to_rfc3339(),
            }))
        }
        Err(_) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid credentials".to_string(),
            }),
        )),
    }
}

pub fn auth_routes() -> Router<Arc<AuthConfig>> {
    Router::new().route("/login", post(login))
}