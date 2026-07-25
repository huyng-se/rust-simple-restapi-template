use crate::core::error::{AppError, AppResult};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);

    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AppError::Internal)?
        .to_string();

    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    let parsed_hash = PasswordHash::new(hash).map_err(|_| AppError::Internal)?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
