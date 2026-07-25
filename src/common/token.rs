use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::{
    config::JwtConfig,
    error::{AppError, AppResult},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub jti: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Clone)]
pub struct JwtService {
    config: JwtConfig,
}

impl JwtService {
    pub fn new(config: JwtConfig) -> Self {
        Self { config }
    }

    pub fn access_ttl_seconds(&self) -> i64 {
        self.config.access_ttl_seconds
    }

    pub fn refresh_ttl_seconds(&self) -> i64 {
        self.config.refresh_ttl_seconds
    }

    pub fn generate_token(&self, user_id: &str, role: &str) -> AppResult<(String, String)> {
        let jti = Uuid::new_v4().to_string();
        let now = Utc::now();
        let exp = now + Duration::seconds(self.config.access_ttl_seconds);

        let claims = Claims {
            sub: user_id.to_owned(),
            jti,
            role: role.to_owned(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.access_secret.as_bytes()),
        )
        .map_err(|_| AppError::Internal)?;

        Ok((token, claims.jti))
    }

    pub fn generate_refresh_token(&self, user_id: &str, role: &str) -> AppResult<(String, String)> {
        let jti = Uuid::new_v4().to_string();
        let now = Utc::now();
        let exp = now + Duration::seconds(self.config.refresh_ttl_seconds);

        let claims = Claims {
            sub: user_id.to_owned(),
            jti,
            role: role.to_owned(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.refresh_secret.as_bytes()),
        )
        .map_err(|_| AppError::Internal)?;

        Ok((token, claims.jti))
    }

    pub fn verify_access_token(&self, token: &str) -> AppResult<Claims> {
        let result = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.access_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized)?;

        Ok(result.claims)
    }

    pub fn verify_refresh_token(&self, token: &str) -> AppResult<Claims> {
        let result = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.refresh_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized)?;

        Ok(result.claims)
    }
}
