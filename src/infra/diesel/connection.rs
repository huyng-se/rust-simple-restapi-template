use std::sync::Arc;

use diesel_async::{
    AsyncPgConnection,
    pooled_connection::{AsyncDieselConnectionManager, bb8::Pool},
};

use crate::core::{config::DatabaseConfig, error::AppResult};

pub type DbPool = Pool<AsyncPgConnection>;

pub async fn init_db_pool(config: &DatabaseConfig) -> AppResult<Arc<DbPool>> {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(config.url.to_owned());
    let pool = Pool::builder()
        .max_size(config.max_connections)
        .build(manager)
        .await?;

    Ok(Arc::new(pool))
}
