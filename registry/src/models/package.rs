use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Package {
    pub id: Option<Uuid>,
    pub name: String,
    pub description: String,
    pub author: String,
    pub repository: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)] 
    pub download_count: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PackageVersion {
    pub id: Option<Uuid>,
    pub package_id: Uuid,
    pub version: String,
    pub lua_source_url: String,
    pub readme: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishVersionRequest {
    pub version: String,
    pub lua_source_url: String,
}

