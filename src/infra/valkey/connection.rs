use redis::Client;

use crate::core::{config::ValkeyConfig, error::AppResult};

pub async fn init_valkey_connection(config: &ValkeyConfig) -> AppResult<Client> {
    let client = Client::open(config.url.to_owned())?;

    let mut conn = client.get_multiplexed_async_connection().await?;
    redis::cmd("PING").query_async::<String>(&mut conn).await?;

    Ok(client)
}
