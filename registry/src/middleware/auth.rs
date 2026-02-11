use crate::models::user::Claims;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use std::env;

/// Represents an authenticated user extracted from the JWT.
///
/// Use this as a handler parameter and Axum will automatically:
/// 1. Extract the Authorization header
/// 2. Verify the JWT signature
/// 3. Return AuthenticatedUser if valid, or 401 if not
///
/// Makes authorization super convenient—just add `user: AuthenticatedUser` to your handler.
pub struct AuthenticatedUser {
    pub user_id: String,
    pub username: String,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Extract token from Authorization header
        // Expected format: "Bearer <token>"
        // We use and_then to chain the operations and fail gracefully if any step doesn't work.
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header"))?;

        if !auth_header.starts_with("Bearer ") {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Invalid Authorization header format",
            ));
        }

        // Skip the "Bearer " prefix (7 chars) to get the actual token
        let token = &auth_header[7..];

        // 2. Decode and verify the JWT
        // This checks:
        // - Signature is valid (using JWT_SECRET)
        // - Token hasn't expired (claims.exp)
        // - Basic structure is sound
        // If any of these fail, we return 401.
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired token"))?;

        Ok(AuthenticatedUser {
            user_id: token_data.claims.sub,
            username: token_data.claims.username,
        })
    }
}
