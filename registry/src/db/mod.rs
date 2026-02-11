use anyhow::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

pub type DB = PgPool;

/// Connects to the database and runs all migrations.
///
/// Uses `sqlx` to execute raw SQL because sqlx migrations are overkill for this.
/// We execute everything one-by-one because Postgres doesn't let you batch multiple
/// DDL statements in prepared statements (thanks, Postgres). This means startup is a
/// bit chatty with the database, but it's idempotent so it's fine.
pub async fn connect() -> Result<DB> {
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;

    // --- Migrations (run on every startup) ---
    // We can't use a proper migration tool because sqlx migrations are... a lot.
    // Instead, every CREATE/ALTER is `IF NOT EXISTS` or `IF NOT ALREADY EXISTS`,
    // so this is safe to re-run. Startup takes a few extra queries, but ¯\_(ツ)_/¯

    // 1. Extensions
    // pg_search used to be here but honestly we're not using it anymore.
    // Keeping it in case someone wants to add it back later.
    sqlx::query(r#"CREATE EXTENSION IF NOT EXISTS "pg_search";"#)
        .execute(&pool)
        .await?;

    // 2. Users Table
    // Simple auth. username is UNIQUE because we assume usernames are the auth identifier.
    // password_hash is bcrypt'd somewhere else (in the API layer).
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at BIGINT NOT NULL
        )
    "#,
    )
    .execute(&pool)
    .await?;

    // 3. Packages Table
    // The registry's main table. Name is UNIQUE because package names can't collide.
    // author is a string because we weren't fancy enough to FK to users (TODO someday?)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS packages (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name TEXT UNIQUE NOT NULL,
            description TEXT NOT NULL,
            author TEXT NOT NULL,
            repository TEXT,
            created_at BIGINT NOT NULL,
            updated_at BIGINT NOT NULL
        )
    "#,
    )
    .execute(&pool)
    .await?;

    // 4. Versions Table
    // Each package can have multiple versions. Semver goes in the version field.
    // UNIQUE(package_id, version) prevents duplicate versions of the same package.
    // lua_source_url points to the R2 blob location (e.g. /packages/logger/v1.0.0/source.zip)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS package_versions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            package_id UUID REFERENCES packages(id) ON DELETE CASCADE,
            version TEXT NOT NULL,
            lua_source_url TEXT NOT NULL,
            readme TEXT,
            created_at BIGINT NOT NULL,
            UNIQUE(package_id, version)
        )
    "#,
    )
    .execute(&pool)
    .await?;

    // 5. Full Text Search Index
    // Combines package name and description for searching.
    // Using standard Postgres FTS (not pg_search anymore, but leaving the extension in case).
    // This gets a bit slow if there are thousands of packages, but works fine for now.
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS packages_search_idx ON packages 
        USING GIN (to_tsvector('english', name || ' ' || description));
    "#,
    )
    .execute(&pool)
    .await?;

    // 6. Download Count Column
    // Added this later, hence the separate ALTER TABLE.
    // We ignore errors here because if it already exists, that's fine—
    // the important thing is that it's there by the end.
    let _ = sqlx::query(
        r#"
        ALTER TABLE packages ADD COLUMN IF NOT EXISTS download_count BIGINT NOT NULL DEFAULT 0;
    "#,
    )
    .execute(&pool)
    .await;

    // 7. Revoked Tokens Table
    // Used for server-side logout. We store the JTI (JWT ID) of revoked tokens.
    // They only need to stay here until they naturally expire.
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS revoked_tokens (
            jti UUID PRIMARY KEY,
            expires_at BIGINT NOT NULL
        )
    "#,
    )
    .execute(&pool)
    .await?;

    // 8. Dependencies Column
    // We store dependencies as JSONB because it's flexible and Postgres handles it well.
    // Each entry is a map of "package-name": "version-requirement".
    let _ = sqlx::query(
        r#"
        ALTER TABLE package_versions ADD COLUMN IF NOT EXISTS dependencies JSONB NOT NULL DEFAULT '{}'::jsonb;
    "#,
    )
    .execute(&pool)
    .await;

    Ok(pool)
}
