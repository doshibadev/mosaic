use crate::middleware::auth::AuthenticatedUser;
use crate::models::package::{Package, PackageVersion, PublishVersionRequest};
use crate::state::AppState;
use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use semver::Version;
use serde_json::json;
use sha2::{Digest, Sha256};

/// Fetches the latest version string for a given package from the package_version table.
async fn get_latest_version(state: &AppState, pkg: &Package) -> String {
    let Some(ref pkg_id) = pkg.id else {
        return "0.0.0".to_string();
    };

    let versions: Vec<PackageVersion> = match state
        .db
        .query("SELECT * FROM package_version WHERE package_id = $pkg_id ORDER BY created_at DESC LIMIT 1")
        .bind(("pkg_id", pkg_id.clone()))
        .await
    {
        Ok(mut res) => res.take(0).unwrap_or(vec![]),
        Err(_) => vec![],
    };

    versions
        .first()
        .map(|v| v.version.clone())
        .unwrap_or_else(|| "0.0.0".to_string())
}

pub async fn list_packages(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let packages: Vec<Package> = match state.db.select("package").await {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("DB error: {}", e)})),
            );
        }
    };

    let mut results = Vec::new();
    for pkg in &packages {
        let version = get_latest_version(&state, pkg).await;
        results.push(json!({
            "name": pkg.name,
            "description": pkg.description,
            "author": pkg.author,
            "version": version,
            "repository": pkg.repository,
        }));
    }

    (StatusCode::OK, Json(json!(results)))
}

pub async fn search_packages(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let q = params.get("q").map(|s| s.as_str()).unwrap_or("");

    // If query is empty, return all packages
    let packages: Vec<Package> = if q.is_empty() {
        match state.db.select("package").await {
            Ok(p) => p,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("DB error: {}", e)})),
                );
            }
        }
    } else {
        match state
            .db
            .query("SELECT * FROM package WHERE name @1@ $q OR description @1@ $q")
            .bind(("q", q.to_string()))
            .await
        {
            Ok(mut res) => res.take(0).unwrap_or(vec![]),
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("DB error: {}", e)})),
                );
            }
        }
    };

    let mut results = Vec::new();
    for pkg in &packages {
        let version = get_latest_version(&state, pkg).await;
        results.push(json!({
            "name": pkg.name,
            "description": pkg.description,
            "author": pkg.author,
            "version": version,
            "repository": pkg.repository,
        }));
    }

    (StatusCode::OK, Json(json!(results)))
}

pub async fn get_package(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let package: Option<Package> = match state
        .db
        .query("SELECT * FROM package WHERE name = $name")
        .bind(("name", name))
        .await
    {
        Ok(mut res) => res.take(0).unwrap_or(None),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("DB error: {}", e)})),
            );
        }
    };

    match package {
        Some(p) => {
            let version = get_latest_version(&state, &p).await;
            (
                StatusCode::OK,
                Json(json!({
                    "id": p.id,
                    "name": p.name,
                    "description": p.description,
                    "author": p.author,
                    "repository": p.repository,
                    "created_at": p.created_at,
                    "updated_at": p.updated_at,
                    "version": version
                })),
            )
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Package not found"})),
        ),
    }
}

pub async fn create_package(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(mut payload): Json<Package>,
) -> (StatusCode, Json<serde_json::Value>) {
    let existing: Option<Package> = match state
        .db
        .query("SELECT * FROM package WHERE name = $name")
        .bind(("name", payload.name.clone()))
        .await
    {
        Ok(mut res) => res.take(0).unwrap_or(None),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("DB error: {}", e)})),
            );
        }
    };

    if existing.is_some() {
        return (
            StatusCode::CONFLICT,
            Json(json!({"error": "Package name already taken"})),
        );
    }

    payload.author = user.username;
    payload.created_at = chrono::Utc::now().timestamp();
    payload.updated_at = payload.created_at;

    let created: Result<Option<Package>, _> = state.db.create("package").content(payload).await;

    match created {
        Ok(Some(p)) => (StatusCode::CREATED, Json(json!(p))),
        Ok(None) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Empty response from DB"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Could not create package: {}", e)})),
        ),
    }
}

pub async fn create_version(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(name): Path<String>,
    Json(payload): Json<PublishVersionRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    if Version::parse(&payload.version).is_err() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid semantic version"})),
        );
    }

    let package: Option<Package> = match state
        .db
        .query("SELECT * FROM package WHERE name = $name")
        .bind(("name", name.clone()))
        .await
    {
        Ok(mut res) => res.take(0).unwrap_or(None),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("DB error: {}", e)})),
            );
        }
    };

    let package = match package {
        Some(p) => p,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Package not found"})),
            );
        }
    };

    if package.author != user.username {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "You do not own this package"})),
        );
    }

    let pkg_id = package.id.expect("package should have an id");
    let existing_version: Option<PackageVersion> = match state
        .db
        .query("SELECT * FROM package_version WHERE package_id = $pkg_id AND version = $version")
        .bind(("pkg_id", pkg_id.clone()))
        .bind(("version", payload.version.clone()))
        .await
    {
        Ok(mut res) => res.take(0).unwrap_or(None),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("DB error: {}", e)})),
            );
        }
    };

    if existing_version.is_some() {
        return (
            StatusCode::CONFLICT,
            Json(json!({"error": "Version already exists"})),
        );
    }

    let version_record = PackageVersion {
        id: None,
        package_id: pkg_id.clone(),
        version: payload.version,
        lua_source_url: payload.lua_source_url,
        created_at: chrono::Utc::now().timestamp(),
    };

    let created: Result<Option<PackageVersion>, _> = state
        .db
        .create("package_version")
        .content(version_record)
        .await;

    let _: Result<Option<Package>, _> = state
        .db
        .update(("package", pkg_id.id.to_string()))
        .merge(json!({"updated_at": chrono::Utc::now().timestamp()}))
        .await;

    match created {
        Ok(Some(v)) => (StatusCode::CREATED, Json(json!(v))),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to create version"})),
        ),
    }
}

pub async fn list_versions(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let package: Option<Package> = match state
        .db
        .query("SELECT * FROM package WHERE name = $name")
        .bind(("name", name.clone()))
        .await
    {
        Ok(mut res) => res.take(0).unwrap_or(None),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("DB error: {}", e)})),
            );
        }
    };

    let package = match package {
        Some(p) => p,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Package not found"})),
            );
        }
    };

    let pkg_id = package.id.expect("package should have an id");
    let versions: Vec<PackageVersion> = match state
        .db
        .query("SELECT * FROM package_version WHERE package_id = $pkg_id ORDER BY created_at DESC")
        .bind(("pkg_id", pkg_id.clone()))
        .await
    {
        Ok(mut res) => res.take(0).unwrap_or(vec![]),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("DB error: {}", e)})),
            );
        }
    };

    (StatusCode::OK, Json(json!(versions)))
}

pub async fn upload_blob(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path((name, version)): Path<(String, String)>,
    body: Bytes,
) -> (StatusCode, Json<serde_json::Value>) {
    // 1. Ownership check
    let package: Option<Package> = match state
        .db
        .query("SELECT * FROM package WHERE name = $name")
        .bind(("name", name.clone()))
        .await
    {
        Ok(mut res) => res.take(0).unwrap_or(None),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("DB error: {}", e)})),
            );
        }
    };

    let package = match package {
        Some(p) => p,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Package not found"})),
            );
        }
    };

    if package.author != user.username {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Not the owner"})),
        );
    }

    // 2. Hash computation (SHA256)
    let mut hasher = Sha256::new();
    hasher.update(&body);
    let hash = format!("{:x}", hasher.finalize());

    // 3. Upload to R2
    if let Err(e) = state.storage.upload_blob(&hash, body.to_vec()).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Storage error: {}", e)})),
        );
    }

    // 4. Update Version record with source URL (pointing to our blob endpoint)
    let pkg_id = package.id.expect("id exists");
    let source_url = format!("/packages/blobs/{}", hash);

    let _: Option<PackageVersion> = match state.db
        .query("UPDATE package_version SET lua_source_url = $url WHERE package_id = $pkg_id AND version = $version")
        .bind(("url", source_url))
        .bind(("pkg_id", pkg_id))
        .bind(("version", version))
        .await
    {
        Ok(mut res) => res.take(0).unwrap_or(None),
        Err(_) => None, // We don't strictly care if the update fails for the response, but ideally we'd log it
    };

    (
        StatusCode::OK,
        Json(json!({"message": "Uploaded successfully", "hash": hash})),
    )
}

pub async fn download_blob(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    match state.storage.get_blob(&hash).await {
        Ok(data) => (
            StatusCode::OK,
            [("content-type", "application/octet-stream")],
            data,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Blob not found").into_response(),
    }
}
