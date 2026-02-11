use crate::models::user::{AuthResponse, Claims, LoginRequest, SignupRequest, User};
use crate::state::AppState;
use crate::utils::auth::{hash_password, verify_password};
use axum::{Json, extract::State, http::StatusCode};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde_json::json;
use std::env;
use uuid::Uuid;

/// Creates a new user account.
///
/// Does the standard signup flow:
/// 1. Check if username is taken (can't have collisions)
/// 2. Hash the password (never store plaintext, obviously)
/// 3. Insert user into database
/// 4. Generate JWT for immediate auth
///
/// Returns the token so the client can start using the API right away.
pub async fn signup(
    State(state): State<AppState>,
    Json(payload): Json<SignupRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    // 1. Check if user already exists
    // We do this upfront so we can fail fast instead of waiting for the database INSERT to complain.
    let existing: Option<Uuid> =
        match sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
            .bind(&payload.username)
            .fetch_optional(&state.db)
            .await
        {
            Ok(id) => id,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
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
    // argon2 because bcrypt is fine but argon2 is slightly more modern.
    // Either way, don't let passwords live in plaintext for 5 minutes.
    let password_hash = match hash_password(&payload.password) {
        Ok(h) => h,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Hashing error"})),
            );
        }
    };

    // 3. Create user in database
    // RETURNING * so we get back the full user object (mostly for the UUID).
    let now = chrono::Utc::now().timestamp();
    let user = match sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, password_hash, created_at)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
    )
    .bind(payload.username)
    .bind(password_hash)
    .bind(now)
    .fetch_one(&state.db)
    .await
    {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Could not create user: {}", e)})),
            );
        }
    };

    // 4. Generate JWT
    // 7-day expiration because that's a reasonable default.
    // Users will have to log back in after a week, which is fine for a package manager.
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
            // Weird edge case: user was created but token generation failed.
            // Still return 201 because the user *does* exist, but warn about the token.
            // Client should probably retry login if they get this.
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

/// Authenticates a user and returns a JWT.
///
/// Simple flow:
/// 1. Look up user by username
/// 2. Verify password matches
/// 3. Generate JWT
///
/// Returns 401 for both "user not found" and "bad password" because we don't want
/// to leak whether a username exists. (Timing attacks are a thing, but we're not
/// worried about that level of paranoia for a package manager.)
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    // 1. Fetch user by username
    // fetch_optional returns Ok(None) if not found, which is handled below.
    let user = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(payload.username)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            // User not found. Return generic "invalid credentials" so we don't leak usernames.
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid credentials"})),
            );
        }
        Err(e) => {
            // Actual database error (connection lost, etc). Surface it.
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", e)})),
            );
        }
    };

    // 2. Verify password
    // Both "user not found" and "bad password" return the same error message
    // to avoid leaking whether a username exists.
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
    // Same logic as signup—7-day expiration.
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
