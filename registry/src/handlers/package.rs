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
use std::io::{Cursor, Read};

/// Fetches the latest version string for a given package from the package_versions table.
async fn get_latest_version(state: &AppState, pkg: &Package) -> String {
    let Some(pkg_id) = pkg.id else {
        return "0.0.0".to_string();
    };

    let version: Option<String> = match sqlx::query_scalar("SELECT version FROM package_versions WHERE package_id = $1 ORDER BY created_at DESC LIMIT 1")
        .bind(pkg_id)
        .fetch_optional(&state.db)
        .await {
            Ok(v) => v,
            Err(_) => None,
        };

    version.unwrap_or_else(|| "0.0.0".to_string())
}

pub async fn list_packages(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let packages = match sqlx::query_as::<_, Package>("SELECT * FROM packages")
        .fetch_all(&state.db)
        .await
    {
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

    let packages = if q.is_empty() {
        match sqlx::query_as::<_, Package>("SELECT * FROM packages")
            .fetch_all(&state.db)
            .await
        {
            Ok(p) => p,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
        }
    } else {
        match sqlx::query_as::<_, Package>(
            r#"
            SELECT * FROM packages 
            WHERE to_tsvector('english', name || ' ' || description) @@ websearch_to_tsquery('english', $1)
            "#
        )
        .bind(q)
        .fetch_all(&state.db)
        .await
        {
            Ok(p) => p,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
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
    let package = match sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE name = $1")
        .bind(name)
        .fetch_optional(&state.db)
        .await
    {
        Ok(p) => p,
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
    Json(payload): Json<Package>,
) -> (StatusCode, Json<serde_json::Value>) {
    // Check if exists
    let existing: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM packages WHERE name = $1")
        .bind(&payload.name)
        .fetch_one(&state.db)
        .await 
    {
        Ok(count) => count,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    };

    if existing > 0 {
        return (
            StatusCode::CONFLICT,
            Json(json!({"error": "Package name already taken"})),
        );
    }

    let now = chrono::Utc::now().timestamp();
    
    // Insert
    let created = sqlx::query_as::<_, Package>(
        r#"
        INSERT INTO packages (name, description, author, repository, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#
    )
    .bind(payload.name)
    .bind(payload.description)
    .bind(user.username)
    .bind(payload.repository)
    .bind(now)
    .bind(now)
    .fetch_one(&state.db)
    .await;

    match created {
        Ok(p) => (StatusCode::CREATED, Json(json!(p))),
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

    let package = match sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE name = $1")
        .bind(name)
        .fetch_optional(&state.db)
        .await
    {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
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
    
    // Check if version exists
    let existing_count: i64 = match sqlx::query_scalar("SELECT COUNT(*) FROM package_versions WHERE package_id = $1 AND version = $2")
        .bind(pkg_id)
        .bind(&payload.version)
        .fetch_one(&state.db)
        .await {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
    };

    if existing_count > 0 {
        return (
            StatusCode::CONFLICT,
            Json(json!({"error": "Version already exists"})),
        );
    }

    let now = chrono::Utc::now().timestamp();

    // Create version
    let created_version = sqlx::query_as::<_, PackageVersion>(
        r#"
        INSERT INTO package_versions (package_id, version, lua_source_url, created_at)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#
    )
    .bind(pkg_id)
    .bind(payload.version)
    .bind(payload.lua_source_url)
    .bind(now)
    .fetch_one(&state.db)
    .await;

    // Update package timestamp
    let _ = sqlx::query("UPDATE packages SET updated_at = $1 WHERE id = $2")
        .bind(now)
        .bind(pkg_id)
        .execute(&state.db)
        .await;

    match created_version {
        Ok(v) => (StatusCode::CREATED, Json(json!(v))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to create version: {}", e)})),
        ),
    }
}

pub async fn list_versions(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let package = match sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE name = $1")
        .bind(name)
        .fetch_optional(&state.db)
        .await
    {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
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
    let versions = match sqlx::query_as::<_, PackageVersion>("SELECT * FROM package_versions WHERE package_id = $1 ORDER BY created_at DESC")
        .bind(pkg_id)
        .fetch_all(&state.db)
        .await
    {
        Ok(v) => v,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
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
    let package = match sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE name = $1")
        .bind(name)
        .fetch_optional(&state.db)
        .await
    {
        Ok(p) => p,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
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

    // 2.5 Extract README
    let mut readme_content: Option<String> = None;
    if let Ok(mut archive) = zip::ZipArchive::new(Cursor::new(&body)) {
        for i in 0..archive.len() {
            if let Ok(mut file) = archive.by_index(i) {
                if file.name().eq_ignore_ascii_case("README.md") {
                    let mut s = String::new();
                    if file.read_to_string(&mut s).is_ok() {
                        readme_content = Some(s);
                    }
                    break;
                }
            }
        }
    }

    // 3. Upload to R2
    if let Err(e) = state.storage.upload_blob(&hash, body.to_vec()).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Storage error: {}", e)})),
        );
    }

    // 4. Update Version record
    let pkg_id = package.id.expect("id exists");
    let source_url = format!("/packages/blobs/{}", hash);

    let result = sqlx::query("UPDATE package_versions SET lua_source_url = $1, readme = $2 WHERE package_id = $3 AND version = $4")
        .bind(source_url)
        .bind(readme_content)
        .bind(pkg_id)
        .bind(version)
        .execute(&state.db)
        .await;

    if let Err(e) = result {
         return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("DB Update failed: {}", e)})),
        );
    }

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
