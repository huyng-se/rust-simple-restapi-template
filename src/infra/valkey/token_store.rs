use async_trait::async_trait;
use redis::AsyncCommands;

use crate::core::error::{AppError, AppResult};

#[async_trait]
pub trait TokenStore {
    async fn save_refresh_token(&self, user_id: i64, token_id: &str, ttl_sec: i64) -> AppResult<()>;
    async fn is_refresh_token_valid(&self, user_id: i64, token_id: &str) -> AppResult<bool>;
    async fn revoke_refresh_token(&self, user_id: i64, token_id: &str) -> AppResult<()>;
    async fn blacklist_access_token(&self, token_id: &str, ttl_sec: i64) -> AppResult<()>;
    async fn is_access_token_blacklisted(&self, token_id: &str) -> AppResult<bool>;
}

#[derive(Clone)]
pub struct RedisTokenStore {
    client: redis::Client,
}

impl RedisTokenStore {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }

    fn refresh_key(user_id: i64, token_id: &str) -> String {
        format!("auth:refresh:{user_id}:{token_id}")
    }

    fn blacklist_key(token_id: &str) -> String {
        format!("auth:blacklist:access:{token_id}")
    }
}

#[async_trait]
impl TokenStore for RedisTokenStore {
    async fn save_refresh_token(
        &self,
        user_id: i64,
        token_id: &str,
        ttl_sec: i64,
    ) -> AppResult<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = Self::refresh_key(user_id, token_id);

        conn.set_ex::<_, _, ()>(key, "1", ttl_sec as u64)
            .await
            .map_err(|err| {
                tracing::error!(
                    error = ?err,
                    "failed to save refresh token"
                );

                AppError::Redis(err)
            })?;

        Ok(())
    }

    async fn is_refresh_token_valid(&self, user_id: i64, token_id: &str) -> AppResult<bool> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = Self::refresh_key(user_id, token_id);

        let exists: bool = conn.exists(key).await.map_err(|err| {
            tracing::error!(
                error = ?err,
                "failed to check is refresh token valid"
            );

            AppError::Redis(err)
        })?;

        Ok(exists)
    }

    async fn revoke_refresh_token(&self, user_id: i64, token_id: &str) -> AppResult<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = Self::refresh_key(user_id, token_id);

        conn.del::<_, ()>(key).await.map_err(|err| {
            tracing::error!(
                error = ?err,
                "failed to revoke refresh token"
            );

            AppError::Redis(err)
        })?;

        Ok(())
    }

    async fn blacklist_access_token(&self, token_id: &str, ttl_sec: i64) -> AppResult<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = Self::blacklist_key(token_id);

        conn.set_ex::<_, _, ()>(key, "1", ttl_sec as u64)
            .await
            .map_err(|err| {
                tracing::error!(
                    error = ?err,
                    "failed to blacklist refresh token"
                );

                AppError::Redis(err)
            })?;

        Ok(())
    }

    async fn is_access_token_blacklisted(&self, token_id: &str) -> AppResult<bool> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = Self::blacklist_key(token_id);

        let exists: bool = conn.exists(key).await.map_err(|err| {
            tracing::error!(
                error = ?err,
                "failed to check is access token blacklisted"
            );

            AppError::Redis(err)
        })?;

        Ok(exists)
    }
}
