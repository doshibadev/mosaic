use registry::{db, routes};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize Logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "registry=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Mosaic Registry API...");

    // 2. Load Environment
    dotenvy::dotenv().ok();

    // 3. Connect to Database
    let db = db::connect().await?;
    tracing::info!("Connected to SurrealDB Cloud successfully!");

    // 4. Initialize Storage
    let storage = registry::utils::storage::StorageService::new().await;
    tracing::info!("Storage service initialized!");

    // 5. Setup Router
    let state = registry::state::AppState { db, storage };
    let app = routes::create_routes(state);

    // 5. Run Server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = SocketAddr::from(([0, 0, 0, 0], port.parse()?));

    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
