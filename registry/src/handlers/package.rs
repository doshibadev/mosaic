use askalono::Store;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::package::{DeprecatePackageRequest, Package, PackageVersion, PublishVersionRequest};
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

/// Helper to get the latest version for a package.
///
/// We need this for list/search endpoints because the DB schema separates packages
/// from their versions. This just grabs the most recent one by timestamp.
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

/// Lists all packages in the registry.
///
/// No filtering, no search—just returns everything. Useful for browsing.
/// Each result includes the latest version so clients can see what's current.
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
            "download_count": pkg.download_count,
            "deprecated": pkg.deprecated,
            "deprecation_reason": pkg.deprecation_reason
        }));
    }

    (StatusCode::OK, Json(json!(results)))
}

/// Searches for packages by name/description.
///
/// Supports query parameters:
/// - q: search term (uses Postgres full-text search)
/// - sort: "downloads" | "newest" | "updated" (default: "updated")
/// - limit: how many results (capped at 100 for sanity)
///
/// If no query, just returns packages sorted by your preference.
/// If query is provided, uses Postgres's websearch_to_tsquery for better results.
pub async fn search_packages(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let q = params.get("q").map(|s| s.as_str()).unwrap_or("");
    let sort = params.get("sort").map(|s| s.as_str()).unwrap_or("updated");
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(20)
        .min(100);

    let order_clause = match sort {
        "downloads" => "download_count DESC",
        "newest" => "created_at DESC",
        "updated" => "updated_at DESC",
        _ => "updated_at DESC", // Default
    };

    let packages = if q.is_empty() {
        // No search query—just return sorted results
        let query_str = format!("SELECT * FROM packages ORDER BY {} LIMIT $1", order_clause);
        match sqlx::query_as::<_, Package>(&query_str)
            .bind(limit)
            .fetch_all(&state.db)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
                );
            }
        }
    } else {
        // User provided a search query. Two cases:
        // 1. If they explicitly asked for a sort, use that (e.g., "show me downloads matching 'logger'")
        // 2. If no explicit sort, use relevance ranking (ts_rank) to show best matches first
        // This is a bit of a UX thing—relevance usually matters more than recency when searching.

        let order_sql = if params.contains_key("sort") {
            order_clause
        } else {
            // Default to relevance ranking when searching
            "ts_rank(to_tsvector('english', name || ' ' || description), websearch_to_tsquery('english', $1)) DESC"
        };

        let query_str = format!(
            r#"
            SELECT * FROM packages 
            WHERE to_tsvector('english', name || ' ' || description) @@ websearch_to_tsquery('english', $1)
            ORDER BY {}
            LIMIT $2
            "#,
            order_sql
        );

        match sqlx::query_as::<_, Package>(&query_str)
            .bind(q)
            .bind(limit)
            .fetch_all(&state.db)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
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
            "download_count": pkg.download_count,
            "deprecated": pkg.deprecated,
            "deprecation_reason": pkg.deprecation_reason
        }));
    }

    (StatusCode::OK, Json(json!(results)))
}

/// Gets a single package by name.
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
            // Fetch the latest version AND its readme
            let latest_version = match sqlx::query_as::<_, PackageVersion>(
                "SELECT * FROM package_versions WHERE package_id = $1 ORDER BY created_at DESC LIMIT 1"
            )
            .bind(p.id)
            .fetch_optional(&state.db)
            .await {
                Ok(v) => v,
                Err(_) => None,
            };

            let (version, readme, license) = match latest_version {
                Some(v) => (v.version, v.readme, v.license),
                None => ("0.0.0".to_string(), None, None),
            };

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
                    "download_count": p.download_count,
                    "version": version,
                    "readme": readme,
                    "license": license,
                    "deprecated": p.deprecated,
                    "deprecation_reason": p.deprecation_reason
                })),
            )
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Package not found"})),
        ),
    }
}

/// Creates a new package in the registry.
///
/// Only authenticated users can create packages. The author is automatically set to
/// the logged-in user, so you can't create packages under someone else's name.
/// Package names must be globally unique.
pub async fn create_package(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<Package>,
) -> (StatusCode, Json<serde_json::Value>) {
    // 0. Validate package name strictly
    if let Err(e) = crate::utils::validation::validate_package_name(&payload.name) {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": e})));
    }

    let now = chrono::Utc::now().timestamp();

    // Create the package. Author is always the authenticated user—can't lie about ownership.
    // We rely on the UNIQUE(name) constraint to prevent duplicates.
    let created = sqlx::query_as::<_, Package>(
        r#"
        INSERT INTO packages (name, description, author, repository, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(&payload.name)
    .bind(payload.description)
    .bind(user.username) // Force the author to be the logged-in user
    .bind(payload.repository)
    .bind(now)
    .bind(now)
    .fetch_one(&state.db)
    .await;

    match created {
        Ok(p) => (StatusCode::CREATED, Json(json!(p))),
        Err(e) => {
            // Check for unique constraint violation (Postgres code 23505)
            if let Some(db_err) = e.as_database_error() {
                if db_err.code() == Some("23505".into()) {
                    return (
                        StatusCode::CONFLICT,
                        Json(json!({"error": "Package name already taken"})),
                    );
                }
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Could not create package: {}", e)})),
            )
        }
    }
}

/// Registers a new version for a package.
///
/// The actual Lua source blob is uploaded separately via upload_blob().
/// This just creates the version record in the database.
/// Version must be valid semver (e.g., "1.0.0", "2.1.3-beta.1").
pub async fn create_version(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(name): Path<String>,
    Json(payload): Json<PublishVersionRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    // Validate semver early to fail fast
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
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
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

    // Only the owner can publish versions of their package
    if package.author != user.username {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "You do not own this package"})),
        );
    }

    let pkg_id = package.id.expect("package should have an id");
    let now = chrono::Utc::now().timestamp();

    // Create the version record. lua_source_url will be updated later when the blob is uploaded.
    // We rely on the UNIQUE(package_id, version) constraint to prevent duplicates.
    let created_version = sqlx::query_as::<_, PackageVersion>(
        r#"
        INSERT INTO package_versions (package_id, version, lua_source_url, created_at, dependencies)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(pkg_id)
    .bind(&payload.version)
    .bind(payload.lua_source_url)
    .bind(now)
    .bind(serde_json::to_value(&payload.dependencies).unwrap_or(json!({})))
    .fetch_one(&state.db)
    .await;

    // Update the package's updated_at timestamp so it shows as recently modified
    // We do this optimistically; if the version insert failed, this update is harmless/redundant
    // but typically we'd only reach here if insert succeeded or we handle error below.
    // Actually, let's only update if successful.
    if created_version.is_ok() {
        let _ = sqlx::query("UPDATE packages SET updated_at = $1 WHERE id = $2")
            .bind(now)
            .bind(pkg_id)
            .execute(&state.db)
            .await;
    }

    match created_version {
        Ok(v) => (StatusCode::CREATED, Json(json!(v))),
        Err(e) => {
            // Check for unique constraint violation (Postgres code 23505)
            if let Some(db_err) = e.as_database_error() {
                if db_err.code() == Some("23505".into()) {
                    return (
                        StatusCode::CONFLICT,
                        Json(json!({"error": "Version already exists"})),
                    );
                }
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to create version: {}", e)})),
            )
        }
    }
}

/// Lists all versions of a package.
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
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
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
    let versions = match sqlx::query_as::<_, PackageVersion>(
        "SELECT * FROM package_versions WHERE package_id = $1 ORDER BY created_at DESC",
    )
    .bind(pkg_id)
    .fetch_all(&state.db)
    .await
    {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            );
        }
    };

    (StatusCode::OK, Json(json!(versions)))
}

/// Uploads the package blob to R2 storage and updates the version record.
///
/// Multi-step process:
/// 1. Verify the authenticated user owns the package (authorization check)
/// 2. Hash the blob (SHA256) and extract any README.md for display
/// 3. Upload the zip to R2 using the hash as the key
/// 4. Update the version record with the R2 URL and README content
pub async fn upload_blob(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path((name, version)): Path<(String, String)>,
    body: Bytes,
) -> (StatusCode, Json<serde_json::Value>) {
    // 1. Ownership check: make sure the user owns this package
    let package = match sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE name = $1")
        .bind(name)
        .fetch_optional(&state.db)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
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

    // 2. Hash the blob so we can use it as the storage key.
    // SHA256 is overkill but makes it hard to guess URLs, so why not.
    let mut hasher = Sha256::new();
    hasher.update(&body);
    let hash = format!("{:x}", hasher.finalize());

    // 2.5 Extract README and License from the zip if they exist
    // Users can include documentation and we'll display it on the registry.
    let mut readme_content: Option<String> = None;
    let mut license_detected: Option<String> = None;

    if let Ok(mut archive) = zip::ZipArchive::new(Cursor::new(&body)) {
        for i in 0..archive.len() {
            if let Ok(mut file) = archive.by_index(i) {
                let name = file.name().to_string();
                
                // Check for README
                if name.eq_ignore_ascii_case("README.md") {
                    let mut s = String::new();
                    if file.read_to_string(&mut s).is_ok() {
                        readme_content = Some(s);
                    }
                }
                
                // Check for LICENSE
                // We look for common names like LICENSE, LICENSE.md, LICENSE.txt
                if name.eq_ignore_ascii_case("LICENSE") 
                    || name.eq_ignore_ascii_case("LICENSE.md") 
                    || name.eq_ignore_ascii_case("LICENSE.txt") 
                {
                    let mut s = String::new();
                    if file.read_to_string(&mut s).is_ok() {
                        // Detect license using askalono
                        // We load the embedded cache. It's small (~300KB compressed).
                        let cache_data = include_bytes!("../utils/license_cache.bin.zstd");
                        if let Ok(store) = Store::from_cache(&cache_data[..]) {
                            let analysis = store.analyze(&text_content(&s));
                            if analysis.score > 0.8 {
                                license_detected = Some(analysis.name.to_string());
                            } else {
                                license_detected = Some("Custom".to_string());
                            }
                        } else {
                            // Fallback if cache fails (shouldn't happen)
                            license_detected = Some("Custom".to_string());
                        }
                    }
                }
            }
        }
    }

    // 3. Upload the blob to R2
    // If this fails, we bail before updating the version record, so the upload is "atomic" in spirit.
    if let Err(e) = state.storage.upload_blob(&hash, body.to_vec()).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Storage error: {}", e)})),
        );
    }

    // 4. Update the version record with the R2 URL and any README/License we found
    let pkg_id = package.id.expect("id exists");
    let source_url = format!("/packages/blobs/{}", hash);

    let result = sqlx::query("UPDATE package_versions SET lua_source_url = $1, readme = $2, license = $3 WHERE package_id = $4 AND version = $5")
        .bind(source_url)
        .bind(readme_content)
        .bind(license_detected)
        .bind(pkg_id)
        .bind(version)
        .execute(&state.db)
        .await;

    if let Err(e) = result {
        tracing::error!(
            "DB Update failed: {}. Initiating rollback for blob {}",
            e,
            hash
        );

        // Rollback: delete the uploaded blob to prevent orphaned files
        if let Err(cleanup_err) = state.storage.delete_blob(&hash).await {
            tracing::error!(
                "CRITICAL: Rollback failed for blob {}: {}",
                hash,
                cleanup_err
            );
        } else {
            tracing::info!("Rollback successful: blob {} deleted.", hash);
        }

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

/// Downloads a package blob from R2 and increments the download counter.
pub async fn download_blob(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> impl IntoResponse {
    // 1. Increment the download count for this package.
    // We have to do a subquery to find which package owns this blob hash,
    // then bump its counter. A bit awkward but necessary since the hash lives in package_versions.
    let url_pattern = format!("/packages/blobs/{}", hash);

    let _ = sqlx::query(
        r#"
        UPDATE packages 
        SET download_count = download_count + 1 
        WHERE id = (
            SELECT package_id FROM package_versions WHERE lua_source_url = $1 LIMIT 1
        )
    "#,
    )
    .bind(&url_pattern)
    .execute(&state.db)
    .await;

    // 2. Fetch and return the blob from R2
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

/// Sets the deprecation status of a package.
///
/// Only the package author can do this.
pub async fn deprecate_package(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(name): Path<String>,
    Json(payload): Json<DeprecatePackageRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let package = match sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE name = $1")
        .bind(&name)
        .fetch_optional(&state.db)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
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

    let pkg_id = package.id.expect("Package ID should be present");

    let result = sqlx::query("UPDATE packages SET deprecated = $1, deprecation_reason = $2 WHERE id = $3")
        .bind(payload.deprecated)
        .bind(payload.reason)
        .bind(pkg_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": "Deprecation status updated"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

/// Unpublishes a version of a package.
///
/// Policy:
/// 1. Must be author.
/// 2. Must be within 24 hours of publish.
/// 3. No other packages must depend on this package (conservative check).
pub async fn unpublish_version(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path((name, version)): Path<(String, String)>,
) -> (StatusCode, Json<serde_json::Value>) {
    let package = match sqlx::query_as::<_, Package>("SELECT * FROM packages WHERE name = $1")
        .bind(&name)
        .fetch_optional(&state.db)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
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

    let pkg_id = package.id.expect("id exists");

    // Fetch the specific version to check timestamp
    let target_version = match sqlx::query_as::<_, PackageVersion>(
        "SELECT * FROM package_versions WHERE package_id = $1 AND version = $2"
    )
    .bind(pkg_id)
    .bind(&version)
    .fetch_optional(&state.db)
    .await {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            );
        }
    };

    let target_version = match target_version {
        Some(v) => v,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Version not found"})),
            );
        }
    };

    // Check 1: Time limit (24 hours)
    let now = chrono::Utc::now().timestamp();
    if now - target_version.created_at > 24 * 60 * 60 {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Cannot unpublish versions older than 24 hours. Deprecate it instead."})),
        );
    }

    // Check 2: Dependents (Left-pad protection)
    // Checks if ANY package depends on this package name.
    let dependents: Option<i32> = match sqlx::query_scalar(
        "SELECT 1 FROM package_versions WHERE dependencies ? $1 LIMIT 1"
    )
    .bind(&name)
    .fetch_optional(&state.db)
    .await {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            );
        }
    };

    if dependents.is_some() {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "Cannot unpublish: other packages depend on this package."})),
        );
    }

    // Proceed to delete
    // 1. Delete blob from R2
    let hash = target_version.lua_source_url.replace("/packages/blobs/", "");
    if let Err(e) = state.storage.delete_blob(&hash).await {
        tracing::error!("Failed to delete blob {} during unpublish: {}", hash, e);
        // Continue anyway to remove from DB, otherwise we leave a broken record.
    }

    // 2. Delete from DB
    let delete_res = sqlx::query("DELETE FROM package_versions WHERE id = $1")
        .bind(target_version.id)
        .execute(&state.db)
        .await;

    match delete_res {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({"message": format!("Successfully unpublished {}@{}", name, version)})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

fn text_content(s: &str) -> askalono::TextData {
    askalono::TextData::from(s)
}
