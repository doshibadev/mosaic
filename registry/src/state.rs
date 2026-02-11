use crate::db::DB;
use crate::utils::storage::StorageService;

#[derive(Clone)]
pub struct AppState {
    pub db: DB,
    pub storage: StorageService,
}

