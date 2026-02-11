use crate::handlers::{
    auth::{login, signup},
    health::health_check,
    package::{
        create_package, create_version, download_blob, get_package, list_packages, list_versions,
        search_packages, upload_blob,
    },
};
use crate::middleware::rate_limit;
use crate::state::AppState;
use axum::{
    Router,
    handler::Handler,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use tower_governor::GovernorLayer;
use tower_http::cors::{Any, CorsLayer};

pub fn create_routes(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Rate limit configurations
    let publish_conf = rate_limit::create_publish_config();
    let login_conf = rate_limit::create_login_config();
    let search_conf = rate_limit::create_search_config();

    let auth_routes = Router::new()
        .route("/signup", post(signup))
        .route(
            "/login", 
            post(login.layer(GovernorLayer::new(login_conf)))
        );

    let package_routes = Router::new()
        .route("/", get(list_packages))
        .route(
            "/", 
            post(create_package.layer(GovernorLayer::new(publish_conf.clone())))
        )
        .route(
            "/search", 
            get(search_packages.layer(GovernorLayer::new(search_conf)))
        )
        .route("/blobs/{hash}", get(download_blob))
        .route("/{name}", get(get_package))
        .route("/{name}/versions", get(list_versions))
        .route(
            "/{name}/versions", 
            post(create_version.layer(GovernorLayer::new(publish_conf.clone())))
        )
        .route(
            "/{name}/versions/{version}/upload", 
            // 5MB limit. Lua scripts are tiny text files. 
            // If you're uploading 5MB of text, you're doing something wrong.
            // This stops someone from nuking our R2 bandwidth.
            post(upload_blob
                .layer(DefaultBodyLimit::max(5 * 1024 * 1024))
                .layer(GovernorLayer::new(publish_conf.clone()))
            )
        );

    Router::new()
        .route("/health", get(health_check))
        .nest("/auth", auth_routes)
        .nest("/packages", package_routes)
        .layer(cors)
        .with_state(state)
}
