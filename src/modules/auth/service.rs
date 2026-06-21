use crate::{
    core::error::{AppError, AppResult},
    infra::valkey::token_store::TokenStore,
    modules::{
        auth::{
            domain::{AuthResponse, LoginRequest, RegisterRequest},
            password,
            token::{Claims, JwtService},
        },
        user::{
            domain::{NewUserPayload, UserResponse},
            repository::UserRepository,
        },
    },
};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;

#[async_trait]
pub trait AuthService {
    async fn register(&self, req: RegisterRequest) -> AppResult<AuthResponse>;
    async fn login(&self, req: LoginRequest) -> AppResult<AuthResponse>;
    async fn refresh(&self, refresh_token: String) -> AppResult<AuthResponse>;
    async fn logout(&self, access_token: String, refresh_token: String) -> AppResult<()>;
    async fn me(&self, user_id: i64) -> AppResult<UserResponse>;
    async fn verify_access_token(&self, token: String) -> AppResult<Claims>;
}

pub struct AuthServiceImpl {
    user_repo: Arc<dyn UserRepository + Send + Sync>,
    token_store: Arc<dyn TokenStore + Send + Sync>,
    jwt_service: JwtService,
}

impl AuthServiceImpl {
    pub fn new(
        user_repo: Arc<dyn UserRepository + Send + Sync>,
        token_store: Arc<dyn TokenStore + Send + Sync>,
        jwt_service: JwtService,
    ) -> Self {
        Self {
            user_repo,
            token_store,
            jwt_service,
        }
    }

    fn parse_user_id_from_claims(claims: &Claims) -> AppResult<i64> {
        claims
            .sub
            .parse::<i64>()
            .map_err(|_| AppError::Unauthorized)
    }

    fn remaining_token_ttl_seconds(claims: &Claims) -> i64 {
        let now = Utc::now().timestamp();

        if claims.exp as i64 <= now {
            return 0;
        }

        claims.exp as i64 - now
    }

    fn ensure_user_active(status: &str) -> Result<(), AppError> {
        if status != "active" {
            return Err(AppError::Forbidden);
        }

        Ok(())
    }
}

#[async_trait]
impl AuthService for AuthServiceImpl {
    async fn register(&self, req: RegisterRequest) -> AppResult<AuthResponse> {
        let existed = self.user_repo.find_by_email(&req.email).await?;

        if existed.is_some() {
            return Err(AppError::Conflict("email already exists".to_string()));
        }

        let password_hash = password::hash_password(&req.password)?;

        let new_user = NewUserPayload {
            email: req.email,
            password: password_hash,
            first_name: req.name,
            last_name: None,
            status: "ACTIVE".to_string(),
        };

        let user = self.user_repo.create(new_user).await?;

        let (access_token, _access_jti) = self
            .jwt_service
            .generate_token(&user.id.to_string(), "STANDARD")?;

        let (refresh_token, refresh_jti) = self
            .jwt_service
            .generate_refresh_token(&user.id.to_string(), "STANDARD")?;

        self.token_store
            .save_refresh_token(
                user.id,
                &refresh_jti,
                self.jwt_service.refresh_ttl_seconds(),
            )
            .await?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
        })
    }

    async fn login(&self, req: LoginRequest) -> AppResult<AuthResponse> {
        let user = self
            .user_repo
            .find_by_email(&req.email)
            .await?
            .ok_or(AppError::Unauthorized)?;

        Self::ensure_user_active(&user.status)?;

        let password_valid = password::verify_password(&req.password, &user.password)?;

        if !password_valid {
            return Err(AppError::Unauthorized);
        }

        let (access_token, _access_jti) = self
            .jwt_service
            .generate_token(&user.id.to_string(), "STANDARD")?;

        let (refresh_token, refresh_jti) = self
            .jwt_service
            .generate_refresh_token(&user.id.to_string(), "STANDARD")?;

        self.token_store
            .save_refresh_token(
                user.id,
                &refresh_jti,
                self.jwt_service.refresh_ttl_seconds(),
            )
            .await?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
        })
    }

    async fn refresh(&self, refresh_token: String) -> AppResult<AuthResponse> {
        let refresh_claims = self.jwt_service.verify_refresh_token(&refresh_token)?;

        let user_id = Self::parse_user_id_from_claims(&refresh_claims)?;

        let refresh_token_valid = self
            .token_store
            .is_refresh_token_valid(user_id, &refresh_claims.jti)
            .await?;

        if !refresh_token_valid {
            return Err(AppError::Unauthorized);
        }

        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(AppError::Unauthorized)?;

        Self::ensure_user_active(&user.status)?;

        let (new_access_token, _new_access_jti) = self
            .jwt_service
            .generate_token(&user.id.to_string(), "STANDARD")?;

        // Simple strategy:
        // - Keep the existing refresh token.
        // - Only issue a new access token.
        //
        // More secure strategy:
        // - Rotate refresh token on every refresh.
        // - Revoke old refresh token jti.
        // - Save new refresh token jti.
        Ok(AuthResponse {
            access_token: new_access_token,
            refresh_token,
        })
    }

    async fn logout(&self, access_token: String, refresh_token: String) -> AppResult<()> {
        let access_claims = self.jwt_service.verify_access_token(&access_token)?;
        let access_user_id = Self::parse_user_id_from_claims(&access_claims)?;

        let refresh_claims = self.jwt_service.verify_refresh_token(&refresh_token)?;
        let refresh_user_id = Self::parse_user_id_from_claims(&refresh_claims)?;

        if access_user_id != refresh_user_id {
            return Err(AppError::Unauthorized);
        }

        let access_ttl = Self::remaining_token_ttl_seconds(&access_claims);

        if access_ttl > 0 {
            self.token_store
                .blacklist_access_token(&access_claims.jti, access_ttl)
                .await?;
        }

        self.token_store
            .revoke_refresh_token(refresh_user_id, &refresh_claims.jti)
            .await?;

        Ok(())
    }

    async fn me(&self, user_id: i64) -> AppResult<UserResponse> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

        Self::ensure_user_active(&user.status)?;

        Ok(UserResponse::from(user))
    }

    async fn verify_access_token(&self, token: String) -> AppResult<Claims> {
        let claims = self.jwt_service.verify_access_token(&token)?;

        let token_blacklisted = self
            .token_store
            .is_access_token_blacklisted(&claims.jti)
            .await?;

        if token_blacklisted {
            return Err(AppError::Unauthorized);
        }

        let user_id = Self::parse_user_id_from_claims(&claims)?;

        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(AppError::Unauthorized)?;

        Self::ensure_user_active(&user.status)?;

        Ok(claims)
    }
}
