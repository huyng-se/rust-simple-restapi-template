use anyhow::{Context, anyhow};
use std::{env, fmt, str::FromStr};

use crate::core::error::AppResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEnv {
    Local,
    Development,
    Staging,
    Production,
}

impl AppEnv {
    pub fn is_production(&self) -> bool {
        matches!(self, AppEnv::Production)
    }

    pub fn is_local(&self) -> bool {
        matches!(self, AppEnv::Local)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AppEnv::Local => "local",
            AppEnv::Development => "development",
            AppEnv::Staging => "staging",
            AppEnv::Production => "production",
        }
    }
}

impl FromStr for AppEnv {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_lowercase().as_str() {
            "local" => Ok(AppEnv::Local),
            "dev" | "development" => Ok(AppEnv::Development),
            "staging" | "stage" => Ok(AppEnv::Staging),
            "prod" | "production" => Ok(AppEnv::Production),
            other => Err(anyhow!("invalid APP_ENV: {other}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub db: DatabaseConfig,
    pub valkey: ValkeyConfig,
    pub jwt: JwtConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub env: AppEnv,
    pub host: String,
    pub port: u16,
    pub request_timeout_s: u64,
    pub body_limit_mb: usize,
    pub cors_allowed_origins: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct ValkeyConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub access_secret: String,
    pub refresh_secret: String,
    pub access_ttl_seconds: i64,
    pub refresh_ttl_seconds: i64,
}

pub fn load_config() -> AppResult<AppConfig> {
    dotenvy::dotenv().ok();
    let app_env: AppEnv = get_env_parse_or("APP_ENV", "local".to_owned())?.parse()?;

    let config = AppConfig {
        server: ServerConfig {
            env: app_env,
            host: get_env_parse_or("APP_HOST", "0.0.0.0".to_owned())?,
            port: get_env_parse_or("APP_PORT", 8088)?,
            request_timeout_s: get_env_parse_or("APP_REQUEST_TIMEOUT_SECONDS", 30)?,
            body_limit_mb: get_env_parse_or("APP_BODY_LIMIT_MB", 2)?,
            cors_allowed_origins: get_env_list_or(
                "CORS_ALLOWED_ORIGINS",
                vec![
                    "http://localhost:3000".to_string(),
                    "http://localhost:5173".to_string(),
                ],
            ),
        },
        db: DatabaseConfig {
            url: get_env("DATABASE_URL")?,
            max_connections: get_env_parse_or("DATABASE_MAX_CONNECTIONS", 10)?,
        },
        valkey: ValkeyConfig {
            url: get_env("VALKEY_URL")?,
        },
        jwt: JwtConfig {
            access_secret: get_env("JWT_ACCESS_SECRET")?,
            refresh_secret: get_env("JWT_REFRESH_SECRET")?,
            access_ttl_seconds: get_env_parse_or("JWT_ACCESS_TTL_SECONDS", 900)?,
            refresh_ttl_seconds: get_env_parse_or("JWT_REFRESH_TTL_SECONDS", 2_592_000)?,
        },
    };

    validate_config(&config)?;
    Ok(config)
}

fn validate_config(config: &AppConfig) -> AppResult<()> {
    if config.server.port == 0 {
        return Err(anyhow!("APP_PORT must be greater than 0").into());
    }

    if config.server.request_timeout_s == 0 {
        return Err(anyhow!("APP_REQUEST_TIMEOUT_SECONDS must be greater than 0").into());
    }

    if config.server.body_limit_mb == 0 {
        return Err(anyhow!("APP_BODY_LIMIT_MB must be greater than 0").into());
    }

    if config.db.max_connections == 0 {
        return Err(anyhow!("DATABASE_MAX_CONNECTIONS must be greater than 0").into());
    }

    if config.jwt.access_ttl_seconds <= 0 {
        return Err(anyhow!("JWT_ACCESS_TTL_SECONDS must be greater than 0").into());
    }

    if config.jwt.refresh_ttl_seconds <= 0 {
        return Err(anyhow!("JWT_REFRESH_TTL_SECONDS must be greater than 0").into());
    }

    if config.jwt.refresh_ttl_seconds <= config.jwt.access_ttl_seconds {
        return Err(anyhow!(
            "JWT_REFRESH_TTL_SECONDS should be greater than JWT_ACCESS_TTL_SECONDS"
        )
        .into());
    }

    validate_jwt_secret(
        "JWT_ACCESS_SECRET",
        &config.jwt.access_secret,
        config.server.env.is_production(),
    )?;

    validate_jwt_secret(
        "JWT_REFRESH_SECRET",
        &config.jwt.refresh_secret,
        config.server.env.is_production(),
    )?;

    if config.jwt.access_secret == config.jwt.refresh_secret {
        return Err(anyhow!("JWT_ACCESS_SECRET and JWT_REFRESH_SECRET must be different").into());
    }

    if config.server.env.is_production() {
        for origin in &config.server.cors_allowed_origins {
            if origin == "*" {
                return Err(
                    anyhow!("CORS_ALLOWED_ORIGINS must not contain '*' in production").into(),
                );
            }
        }
    }

    Ok(())
}

fn validate_jwt_secret(name: &str, value: &str, is_production: bool) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("{name} must not be empty").into());
    }

    if is_production {
        if value.len() < 32 {
            return Err(anyhow!("{name} must be at least 32 characters in production").into());
        }

        if value.contains("change-me") {
            return Err(anyhow!("{name} must not use default placeholder value").into());
        }
    }

    Ok(())
}

fn get_env(key: &str) -> AppResult<String> {
    Ok(env::var(key).with_context(|| format!("missing required env var: {key}"))?)
}

fn get_env_parse_or<T>(key: &str, default: T) -> AppResult<T>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    match env::var(key) {
        Ok(value) => Ok(value
            .parse::<T>()
            .map_err(|err| anyhow!("invalid env var {key}: {err}"))?),
        Err(_) => Ok(default),
    }
}

fn get_env_list_or(key: &str, default: Vec<String>) -> Vec<String> {
    match env::var(key) {
        Ok(value) => value
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToString::to_string)
            .collect(),
        Err(_) => default,
    }
}
