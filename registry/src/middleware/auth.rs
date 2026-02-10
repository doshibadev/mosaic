use crate::models::user::Claims;
use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use std::env;

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
        // 1. Get token from Authorization header
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

        let token = &auth_header[7..];

        // 2. Verify JWT
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
