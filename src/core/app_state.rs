use crate::{infra::diesel::connection::DbPool, modules::user::service::UserService};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<DbPool>,
    pub user_service: Arc<dyn UserService + Send + Sync>,
}
