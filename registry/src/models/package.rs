use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    pub id: Option<surrealdb::sql::Thing>,
    pub name: String,
    pub description: String,
    pub author: String,
    pub repository: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageVersion {
    pub id: Option<surrealdb::sql::Thing>,
    pub package_id: surrealdb::sql::Thing,
    pub version: String,
    pub lua_source_url: String,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishVersionRequest {
    pub version: String,
    pub lua_source_url: String,
}
