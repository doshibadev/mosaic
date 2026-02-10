use crate::models::user::{AuthResponse, Claims, LoginRequest, SignupRequest, User};
use crate::state::AppState;
use crate::utils::auth::{hash_password, verify_password};
use axum::{Json, extract::State, http::StatusCode};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde_json::json;
use std::env;

pub async fn signup(
    State(state): State<AppState>,
    Json(payload): Json<SignupRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    // 1. Check if user already exists
    let existing: Option<User> = match state
        .db
        .query("SELECT * FROM user WHERE username = $username")
        .bind(("username", payload.username.clone()))
        .await
    {
        Ok(mut res) => res.take(0).unwrap_or(None),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            );
        }
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
    let user = User {
        id: None,
        username: payload.username,
        password_hash,
        created_at: chrono::Utc::now().timestamp(),
    };

    let created_user: User = match state.db.create("user").content(user).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create user"})),
            );
        }
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
        sub: created_user.id.map(|id| id.to_string()).unwrap_or_default(),
        username: created_user.username.clone(),
        exp: expiration,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(t) => t,
        Err(_) => {
            // User was created but token failed - they can still log in manually
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
            username: created_user.username,
        })),
    )
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    // 1. Fetch user
    let user: Option<User> = match state
        .db
        .query("SELECT * FROM user WHERE username = $username")
        .bind(("username", payload.username.clone()))
        .await
    {
        Ok(mut res) => res.take(0).unwrap_or(None),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            );
        }
    };

    let user = match user {
        Some(u) => u,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid credentials"})),
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
