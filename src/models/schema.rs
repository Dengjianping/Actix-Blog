table! {
    posts (id) {
        id -> Int4,
        title -> Varchar,
        slug -> Varchar,
        body -> Text,
        publish -> Timestamp,
        created -> Timestamp,
        updated -> Timestamp,
        status -> Varchar,
        user_id -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Varchar,
        first_name -> Varchar,
        last_name -> Varchar,
        email -> Varchar,
        is_superuser -> Bool,
        is_staff -> Bool,
        is_active -> Bool,
        last_login -> Nullable<Timestamp>,
        date_joined -> Nullable<Timestamp>,
    }
}

joinable!(posts -> users (user_id));

allow_tables_to_appear_in_same_query!(
    posts,
    users,
);
