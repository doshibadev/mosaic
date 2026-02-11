use registry::{db, routes};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 0. Install rustls crypto provider BEFORE anything else
    // This needs to happen before any TLS connections are made (database, etc).
    // If you move this down, you'll get cryptic errors about "no crypto provider".
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // 1. Initialize logging
    // Uses tracing for structured logs. Respects RUST_LOG env var.
    // Defaults to debug level for the registry and tower_http so you can see what's happening.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "registry=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Mosaic Registry API...");

    // 2. Load .env file
    // Uses dotenvy which is just dotenv but maintained. Silently ignores if no .env exists.
    dotenvy::dotenv().ok();

    // 3. Connect to database
    // Runs migrations automatically and panics if DATABASE_URL isn't set.
    // By this point we should be connected to Neon PostgreSQL.
    let db = db::connect().await?;
    tracing::info!("Connected to Neon PostgreSQL successfully!");

    // 4. Initialize R2 storage
    // Reads R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_ENDPOINT from env.
    // If any of these are missing, it panics. Intentional—storage is non-negotiable.
    let storage = registry::utils::storage::StorageService::new().await;
    tracing::info!("Storage service initialized!");

    // 5. Build the app state
    // This is what gets passed to all route handlers. Contains the DB pool and storage service.
    let state = registry::state::AppState { db, storage };
    let app = routes::create_routes(state);

    // 6. Start the server
    // Listens on PORT env var (defaults to 3000).
    // 0.0.0.0 so it binds to all interfaces (necessary in Docker).
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = SocketAddr::from(([0, 0, 0, 0], port.parse()?));

    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
