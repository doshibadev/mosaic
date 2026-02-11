use anyhow::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

pub type DB = PgPool;

pub async fn connect() -> Result<DB> {
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;

    // Run migrations automatically on startup
    // We must execute these one by one because sqlx prepared statements don't support multiple commands
    
    // 1. Extensions
    sqlx::query(r#"CREATE EXTENSION IF NOT EXISTS "pg_search";"#)
        .execute(&pool)
        .await?;

    // 2. Users Table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at BIGINT NOT NULL
        )
    "#)
    .execute(&pool)
    .await?;

    // 3. Packages Table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS packages (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name TEXT UNIQUE NOT NULL,
            description TEXT NOT NULL,
            author TEXT NOT NULL,
            repository TEXT,
            created_at BIGINT NOT NULL,
            updated_at BIGINT NOT NULL
        )
    "#)
    .execute(&pool)
    .await?;

    // 4. Versions Table
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS package_versions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            package_id UUID REFERENCES packages(id) ON DELETE CASCADE,
            version TEXT NOT NULL,
            lua_source_url TEXT NOT NULL,
            readme TEXT,
            created_at BIGINT NOT NULL,
            UNIQUE(package_id, version)
        )
    "#)
    .execute(&pool)
    .await?;

    // 5. Search Index
    // Note: If using pure Neon/Postgres, we use standard FTS.
    sqlx::query(r#"
        CREATE INDEX IF NOT EXISTS packages_search_idx ON packages 
        USING GIN (to_tsvector('english', name || ' ' || description));
    "#)
    .execute(&pool)
    .await?;

    Ok(pool)
}
