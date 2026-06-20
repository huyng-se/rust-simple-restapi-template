use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    core::error::{AppError, AppResult},
    modules::user::{
        domain::{CreateUserRequest, NewUserPayload, UserResponse},
        repository::UserRepository,
    },
};

#[async_trait]
pub trait UserService {
    async fn create(&self, req: CreateUserRequest) -> AppResult<UserResponse>;
    async fn get_by_id(&self, id: i64) -> AppResult<Option<UserResponse>>;
    async fn get_by_email(&self, email: &str) -> AppResult<Option<UserResponse>>;
    async fn get_list(&self) -> AppResult<Vec<UserResponse>>;
}

pub struct UserServiceImpl {
    repo: Arc<dyn UserRepository + Send + Sync>,
}

impl UserServiceImpl {
    pub fn new(repo: Arc<dyn UserRepository + Send + Sync>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn create(&self, req: CreateUserRequest) -> AppResult<UserResponse> {
        let new_user = NewUserPayload {
            email: req.email,
            password: "".to_string(),
            first_name: req.name,
            last_name: "user".to_string(),
            status: "ACTIVE".to_string(),
        };
        let user = self.repo.create(new_user).await?;

        Ok(UserResponse::from(user))
    }

    async fn get_by_id(&self, id: i64) -> AppResult<Option<UserResponse>> {
        let user = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(Some(UserResponse::from(user)))
    }

    async fn get_by_email(&self, email: &str) -> AppResult<Option<UserResponse>> {
        let user = self
            .repo
            .find_by_email(email)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(Some(UserResponse::from(user)))
    }

    async fn get_list(&self) -> AppResult<Vec<UserResponse>> {
        let users = self.repo.list().await?;

        Ok(users.into_iter().map(UserResponse::from).collect())
    }
}
