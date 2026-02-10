use anyhow::Result;
use std::env;
use surrealdb::Surreal;
use surrealdb::engine::any;

pub type DB = Surreal<surrealdb::engine::any::Any>;

pub async fn connect() -> Result<DB> {
    let url = env::var("SURREAL_URL").expect("SURREAL_URL must be set");
    let user = env::var("SURREAL_USER").unwrap_or_else(|_| "root".to_string());
    let pass = env::var("SURREAL_PASS").unwrap_or_else(|_| "root".to_string());
    let ns = env::var("SURREAL_NS").unwrap_or_else(|_| "mosaic".to_string());
    let db_name = env::var("SURREAL_DB").unwrap_or_else(|_| "registry".to_string());

    // 1. Connect
    let db = any::connect(url).await?;

    // 2. Authenticate
    db.signin(surrealdb::opt::auth::Root {
        username: &user,
        password: &pass,
    })
    .await?;

    // 3. Select namespace and database
    db.use_ns(ns).use_db(db_name).await?;

    // 4. Define Search Indices (Full-Text)
    // We define an index on name and description for fast discovery.
    // SurrealDB will handle the typo-tolerance and relevance.
    let _ = db.query("
        DEFINE INDEX IF NOT EXISTS package_search ON TABLE package COLUMNS name, description SEARCH ANALYZER ascii BM25 HIGHLIGHTS;
    ").await;

    Ok(db)
}
