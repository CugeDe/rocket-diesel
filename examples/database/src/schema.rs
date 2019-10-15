table! {
    access_token (id) {
        id -> Nullable<Integer>,
        client_id -> Integer,
        user_id -> Nullable<Integer>,
        token -> Text,
        expires_at -> Nullable<Integer>,
        scope -> Nullable<Text>,
    }
}

table! {
    auth_code (id) {
        id -> Nullable<Integer>,
        client_id -> Integer,
        user_id -> Nullable<Integer>,
        token -> Text,
        redirect_uri -> Text,
        expires_at -> Nullable<Integer>,
        scope -> Nullable<Text>,
    }
}

table! {
    client (id) {
        id -> Nullable<Integer>,
        random_id -> Text,
        redirect_uris -> Text,
        secret -> Text,
        allowed_grant_types -> Text,
    }
}

table! {
    refresh_token (id) {
        id -> Nullable<Integer>,
        client_id -> Integer,
        user_id -> Nullable<Integer>,
        token -> Text,
        expires_at -> Nullable<Integer>,
        scope -> Nullable<Text>,
    }
}

table! {
    users (id) {
        id -> Integer,
        name -> Text,
        firstname -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    access_token,
    auth_code,
    client,
    refresh_token,
    users,
);
