use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use axum_extra::{
    extract::TypedHeader,
    headers::{authorization::Bearer, Authorization},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
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
    pub user: UserInfo,
    pub expires_at: String,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub name: String,
    pub role: String,
    pub is_admin: bool,
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

fn check_user_privileges(username: &str) -> (bool, String) {
    if username == "root" {
        return (true, "root".to_string());
    }
    
    let output = std::process::Command::new("sudo")
        .args(["-l", "-U", username])
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains("(ALL : ALL)") || output_str.contains("(root)") {
                    return (true, "sudo".to_string());
                }
            }
        }
        Err(_) => {}
    }
    
    (false, "user".to_string())
}

pub async fn login(
    State(config): State<Arc<AuthConfig>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    println!("Login attempt for user: {}", payload.username);
    // Authenticate using PAM
    let mut authenticator = Authenticator::with_password("login")
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to initialize PAM authentication".to_string(),
                }),
            )
        })?;

    authenticator
        .get_handler()
        .set_credentials(&payload.username, &payload.password);

    match authenticator.authenticate() {
        Ok(_) => {
            // Check user privileges
            let (is_admin, role) = check_user_privileges(&payload.username);
            
            // Get user's real name from system
            let real_name = std::process::Command::new("getent")
                .args(["passwd", &payload.username])
                .output()
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        let line = String::from_utf8_lossy(&output.stdout);
                        line.split(':').nth(4).map(|s| s.trim().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| payload.username.clone());

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
                        error: "Failed to generate authentication token".to_string(),
                    }),
                )
            })?;

            let user_info = UserInfo {
                id: payload.username.clone(),
                username: payload.username.clone(),
                name: real_name,
                role,
                is_admin,
            };

            println!("Login successful for user: {}", payload.username);
            Ok(Json(LoginResponse {
                token,
                user: user_info,
                expires_at: expires_at.to_rfc3339(),
            }))
        }
        Err(_) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid username or password".to_string(),
            }),
        )),
    }
}

pub async fn validate_token(
    State(config): State<Arc<AuthConfig>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<UserInfo>, (StatusCode, Json<ErrorResponse>)> {
    let token = bearer.token();
    
    println!("Validating token: {}", token);
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| {
        println!("Token validation error: {:?}", e);
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid or expired token".to_string(),
            }),
        )
    })?;

    let username = token_data.claims.sub;
    println!("Token validated for user: {}", username);
    
    let (is_admin, role) = check_user_privileges(&username);
    
    let real_name = std::process::Command::new("getent")
        .args(["passwd", &username])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                let line = String::from_utf8_lossy(&output.stdout);
                line.split(':').nth(4).map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| username.clone());

    let user_info = UserInfo {
        id: username.clone(),
        username,
        name: real_name,
        role,
        is_admin,
    };

    println!("Returning user info: {:?}", user_info);
    Ok(Json(user_info))
}

pub fn auth_routes() -> Router<Arc<AuthConfig>> {
    Router::new()
        .route("/login", post(login))
        .route("/validate", get(validate_token))
}