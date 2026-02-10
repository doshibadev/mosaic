use crate::handlers::{
    auth::{login, signup},
    health::health_check,
    package::{
        create_package, create_version, download_blob, get_package, list_packages, list_versions,
        search_packages, upload_blob,
    },
};
use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};

pub fn create_routes(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let auth_routes = Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login));

    let package_routes = Router::new()
        .route("/", get(list_packages))
        .route("/", post(create_package))
        .route("/search", get(search_packages))
        .route("/blobs/:hash", get(download_blob))
        .route("/:name", get(get_package))
        .route("/:name/versions", get(list_versions))
        .route("/:name/versions", post(create_version))
        .route("/:name/versions/:version/upload", post(upload_blob));

    Router::new()
        .route("/health", get(health_check))
        .nest("/auth", auth_routes)
        .nest("/packages", package_routes)
        .layer(cors)
        .with_state(state)
}
