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
    pub status: String,
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
    pub status: String,
}

// ============= DTO =============

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub email: String,
    pub first_name: String,
    pub last_name: Option<String>,
    pub status: String,
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
