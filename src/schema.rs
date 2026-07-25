// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "user_role"))]
    pub struct UserRole;

    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "user_status"))]
    pub struct UserStatus;
}

diesel::table! {
    flyway_schema_history (installed_rank) {
        installed_rank -> Int4,
        #[max_length = 50]
        version -> Nullable<Varchar>,
        #[max_length = 200]
        description -> Varchar,
        #[sql_name = "type"]
        #[max_length = 20]
        type_ -> Varchar,
        #[max_length = 1000]
        script -> Varchar,
        checksum -> Nullable<Int4>,
        #[max_length = 100]
        installed_by -> Varchar,
        installed_on -> Timestamp,
        execution_time -> Int4,
        success -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Int8,
        email -> Text,
        password -> Text,
        first_name -> Text,
        last_name -> Nullable<Text>,
        role -> crate::schema::sql_types::UserRole,
        status -> crate::schema::sql_types::UserStatus,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(flyway_schema_history, users,);
