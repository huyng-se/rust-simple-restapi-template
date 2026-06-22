use std::sync::Arc;

use crate::{
    core::error::{self, AppResult},
    infra::diesel::connection::DbPool,
    modules::user::domain::{NewUserPayload, UserModel},
    schema::users,
};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[async_trait]
pub trait UserRepository {
    async fn create(&self, payload: NewUserPayload) -> AppResult<UserModel>;
    async fn find_by_id(&self, id: i64) -> AppResult<Option<UserModel>>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<UserModel>>;
    async fn list(&self) -> AppResult<Vec<UserModel>>;
}

pub struct DbUserRepository {
    db_pool: Arc<DbPool>,
}

impl DbUserRepository {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl UserRepository for DbUserRepository {
    async fn create(&self, payload: NewUserPayload) -> AppResult<UserModel> {
        let mut conn = self.db_pool.get().await.map_err(|err| {
            tracing::error!(error = %err, "failed to get database connection for user create");
            err
        })?;

        let created = diesel::insert_into(users::table)
            .values(payload)
            .returning(UserModel::as_returning())
            .get_result::<UserModel>(&mut conn)
            .await
            .map_err(|err| {
                tracing::error!(error = %err, "failed to insert user record");
                error::map_diesel_error(err)
            })?;

        Ok(created)
    }

    async fn find_by_id(&self, user_id: i64) -> AppResult<Option<UserModel>> {
        let mut conn = self.db_pool.get().await.map_err(|err| {
            tracing::error!(error = %err, "failed to get database connection for find user by id");
            err
        })?;

        let result = users::table
            .filter(users::id.eq(user_id))
            .select(UserModel::as_select())
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(|err| {
                tracing::error!(user_id, error = %err, "failed to find user by id");
                error::map_diesel_error(err)
            })?;

        Ok(result)
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<UserModel>> {
        let mut conn = self.db_pool.get().await.map_err(|err| {
            tracing::error!(error = %err, "failed to get database connection for find user by email");
            err
        })?;

        let result = users::table
            .filter(users::email.eq(email))
            .select(UserModel::as_select())
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(|err| {
                tracing::error!(email, error = %err, "failed to find user by email");
                error::map_diesel_error(err)
            })?;

        Ok(result)
    }

    async fn list(&self) -> AppResult<Vec<UserModel>> {
        let mut conn = self.db_pool.get().await.map_err(|err| {
            tracing::error!(error = %err, "failed to get database connection for list users");
            err
        })?;

        let results = users::table
            .select(UserModel::as_select())
            .load::<UserModel>(&mut conn)
            .await
            .map_err(|err| {
                tracing::error!(error = %err, "failed to list users");
                error::map_diesel_error(err)
            })?;

        Ok(results)
    }
}
