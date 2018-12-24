table! {
    comments (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        comment -> Text,
        committed_time -> Nullable<Timestamp>,
        post_id -> Int4,
    }
}

table! {
    contacts (id) {
        id -> Int4,
        tourist_name -> Varchar,
        email -> Varchar,
        message -> Varchar,
        committed_time -> Nullable<Timestamp>,
    }
}

table! {
    posts (id) {
        id -> Int4,
        title -> Varchar,
        slug -> Varchar,
        body -> Text,
        publish -> Nullable<Timestamp>,
        created -> Nullable<Timestamp>,
        updated -> Nullable<Timestamp>,
        status -> Varchar,
        user_id -> Int4,
        likes -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
        password -> Varchar,
        username -> Varchar,
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

joinable!(comments -> posts (post_id));
joinable!(posts -> users (user_id));

allow_tables_to_appear_in_same_query!(
    comments,
    contacts,
    posts,
    users,
);
