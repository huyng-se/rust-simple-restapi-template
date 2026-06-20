// @generated automatically by Diesel CLI.

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
        status -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(flyway_schema_history, users,);
