use axum::http::{Request, StatusCode};
use governor::{clock::QuantaInstant, middleware::NoOpMiddleware};
use jsonwebtoken::{DecodingKey, Validation, decode};
use std::{env, hash::Hash, net::IpAddr, sync::Arc, time::Duration};
use tower_governor::{
    governor::{GovernorConfig, GovernorConfigBuilder},
    key_extractor::KeyExtractor,
    errors::GovernorError,
};

use crate::models::user::Claims;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IpKeyExtractor;

impl KeyExtractor for IpKeyExtractor {
    type Key = IpAddr;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        let headers = req.headers();

        // 1. Check Cloudflare header first
        // If we're behind Cloudflare, the real IP is in 'cf-connecting-ip'.
        // We trust this because we assume the server is configured to only accept traffic from CF.
        if let Some(ip) = headers
            .get("cf-connecting-ip")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<IpAddr>().ok())
        {
            return Ok(ip);
        }

        // 2. Check X-Forwarded-For as backup
        // Standard proxy header. We take the first IP in the list as it's the client.
        // Useful if we're behind a generic load balancer or Nginx.
        if let Some(ip) = headers
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse::<IpAddr>().ok())
        {
            return Ok(ip);
        }
        
        // 3. Fallback to localhost
        // If we can't find an IP, we default to 127.0.0.1.
        // This is mostly for local dev where headers might be missing.
        // In prod, this puts unknown IPs in the same bucket, which is better than panicking.
        Ok("127.0.0.1".parse().unwrap()) 
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UserKeyExtractor;

impl KeyExtractor for UserKeyExtractor {
    type Key = String;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        let headers = req.headers();
        
        // 1. Grab Authorization header
        // If it's missing, we return a GovernorError::Other with 401.
        // This stops the request early at the rate limit layer.
        let auth_header = headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or(GovernorError::Other {
                code: StatusCode::UNAUTHORIZED,
                msg: Some("Missing Authorization header".to_string()),
                headers: None,
            })?;

        // 2. Verify Bearer prefix
        // Standard JWT format. If it's not Bearer, it's not for us.
        if !auth_header.starts_with("Bearer ") {
            return Err(GovernorError::Other {
                code: StatusCode::UNAUTHORIZED,
                msg: Some("Invalid Authorization header".to_string()),
                headers: None,
            });
        }

        // 3. Decode JWT to get User ID
        // We need the secret from env. If it fails, something is very wrong with the server.
        // If decoding fails, token is invalid/expired -> 401.
        let token = &auth_header[7..];
        let secret = env::var("JWT_SECRET").map_err(|_| GovernorError::Other {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: Some("Server configuration error".to_string()),
            headers: None,
        })?;
        
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| GovernorError::Other {
            code: StatusCode::UNAUTHORIZED,
            msg: Some("Invalid or expired token".to_string()),
            headers: None,
        })?;

        Ok(token_data.claims.sub)
    }
}

// Type aliases for sanity
// Using NoOpMiddleware<QuantaInstant> because that's what the default builder gives us.
pub type PublishConfig = GovernorConfig<UserKeyExtractor, NoOpMiddleware<QuantaInstant>>;
pub type LoginConfig = GovernorConfig<IpKeyExtractor, NoOpMiddleware<QuantaInstant>>;
pub type SearchConfig = GovernorConfig<IpKeyExtractor, NoOpMiddleware<QuantaInstant>>;

pub fn create_publish_config() -> Arc<PublishConfig> {
    // 1. Publish Rate Limit
    // 10 requests per hour per user.
    // Prevents spamming the registry with garbage packages.
    Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(UserKeyExtractor)
            .period(Duration::from_secs(360)) // 360s * 10 = 1 hour
            .burst_size(10)
            .finish()
            .unwrap(),
    )
}

pub fn create_login_config() -> Arc<LoginConfig> {
    // 2. Login Rate Limit
    // 5 attempts per 15 minutes per IP.
    // Standard brute-force protection. Tight enough to annoy attackers, loose enough for typos.
    Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(IpKeyExtractor)
            .period(Duration::from_secs(180)) // 180s * 5 = 15 mins
            .burst_size(5)
            .finish()
            .unwrap(),
    )
}

pub fn create_search_config() -> Arc<SearchConfig> {
    // 3. Search Rate Limit
    // 60 requests per minute per IP.
    // Search is expensive-ish (DB queries). 1 req/sec is plenty for humans.
    Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(IpKeyExtractor)
            .period(Duration::from_secs(1))
            .burst_size(60)
            .finish()
            .unwrap(),
    )
}