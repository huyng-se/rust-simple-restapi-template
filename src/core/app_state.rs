use crate::{
    infra::diesel::connection::DbPool,
    modules::{auth::auth_service::AuthService, user::user_service::UserService},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<DbPool>,
    pub auth_service: Arc<dyn AuthService + Send + Sync>,
    pub user_service: Arc<dyn UserService + Send + Sync>,
}
