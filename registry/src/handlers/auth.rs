use crate::models::user::{AuthResponse, Claims, LoginRequest, SignupRequest, User};
use crate::state::AppState;
use crate::utils::auth::{hash_password, verify_password};
use axum::{Json, extract::State, http::StatusCode};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde_json::json;
use std::env;
use uuid::Uuid;

pub async fn signup(
    State(state): State<AppState>,
    Json(payload): Json<SignupRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    // 1. Check if user already exists
    let existing: Option<Uuid> = match sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&state.db)
        .await {
            Ok(id) => id,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
        };

    if existing.is_some() {
        return (
            StatusCode::CONFLICT,
            Json(json!({"error": "Username already taken"})),
        );
    }

    // 2. Hash password
    let password_hash = match hash_password(&payload.password) {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Hashing error"})),
            );
        }
    };

    // 3. Create user
    let now = chrono::Utc::now().timestamp();
    let user = match sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, password_hash, created_at)
        VALUES ($1, $2, $3)
        RETURNING *
        "#
    )
    .bind(payload.username)
    .bind(password_hash)
    .bind(now)
    .fetch_one(&state.db)
    .await {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Could not create user: {}", e)})),
            );
        }
    };

    // 4. Generate JWT
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user.id.map(|id| id.to_string()).unwrap_or_default(),
        username: user.username.clone(),
        exp: expiration,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::CREATED,
                Json(json!({"message": "User created successfully but token generation failed"})),
            );
        }
    };

    (
        StatusCode::CREATED,
        Json(json!(AuthResponse {
            token,
            username: user.username,
        })),
    )
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    // 1. Fetch user
    let user = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(payload.username)
        .fetch_optional(&state.db)
        .await {
            Ok(Some(u)) => u,
            Ok(None) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Invalid credentials"})),
                );
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("Database error: {}", e)})),
                );
            }
        };

    // 2. Verify password
    match verify_password(&payload.password, &user.password_hash) {
        Ok(true) => (),
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid credentials"})),
            );
        }
    }

    // 3. Generate JWT
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user.id.map(|id| id.to_string()).unwrap_or_default(),
        username: user.username.clone(),
        exp: expiration,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Token generation error"})),
            );
        }
    };

    (
        StatusCode::OK,
        Json(json!(AuthResponse {
            token,
            username: user.username,
        })),
    )
}
