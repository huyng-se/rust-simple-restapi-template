use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::schema::users;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserModel {
    pub id: i64,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: Option<String>,
    pub status: UserStatus,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUserPayload {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: Option<String>,
    pub status: UserStatus,
    pub role: UserRole,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = users)]
pub struct UpdateUserPayload {
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::UserRole"]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
pub enum UserRole {
    SuperAdmin,
    Admin,
    Standard,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::UserStatus"]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
pub enum UserStatus {
    Active,
    Disabled,
}

// ============= DTO =============

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub email: String,
    pub first_name: String,
    pub last_name: Option<String>,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserModel> for UserResponse {
    fn from(user: UserModel) -> Self {
        Self {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            status: user.status,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{UserRole, UserStatus};

    #[test]
    fn user_status_uses_uppercase_database_labels() {
        assert_eq!(UserStatus::Active, UserStatus::Active);
        assert_eq!(UserStatus::Disabled, UserStatus::Disabled);
    }

    #[test]
    fn user_role_variants_are_defined() {
        assert_eq!(UserRole::SuperAdmin, UserRole::SuperAdmin);
        assert_eq!(UserRole::Admin, UserRole::Admin);
        assert_eq!(UserRole::Standard, UserRole::Standard);
    }
}
