// @generated automatically by Diesel CLI.

diesel::table! {
    llm (uuid) {
        uuid -> Text,
        id -> Text,
        family_id -> Text,
        organization -> Text,
        name -> Text,
        homepage -> Text,
        description -> Text,
        license -> Text,
        downloaded_reason -> Text,
        downloaded_date -> TimestamptzSqlite,
        last_called -> Nullable<TimestamptzSqlite>,
        capabilities -> Text,
        tags -> Text,
        requirements -> Text,
        url -> Text,
        local -> Bool,
        connector_type -> Text,
        config -> Text,
        model_path -> Nullable<Text>,
        parameters -> Text,
        user_parameters -> Text,
        session_parameters -> Text,
        user_session_parameters -> Text,
    }
}

diesel::table! {
    llm_history (id) {
        id -> Text,
        llm_session_id -> Text,
        updated_timestamp -> TimestamptzSqlite,
        call_timestamp -> TimestamptzSqlite,
        complete -> Bool,
        parameters -> Text,
        input -> Text,
        output -> Text,
    }
}

diesel::table! {
    llm_session (id) {
        id -> Text,
        llm_uuid -> Text,
        user_id -> Text,
        started -> TimestamptzSqlite,
        last_called -> TimestamptzSqlite,
        session_parameters -> Text,
    }
}

diesel::table! {
    requests (id) {
        id -> Text,
        user_id -> Text,
        originator -> Text,
        reason -> Text,
        timestamp -> TimestamptzSqlite,
        request -> Text,
        complete -> Bool,
        accepted -> Bool,
    }
}

diesel::table! {
    user (id) {
        id -> Text,
        name -> Text,
        api_key -> Text,
        perm_superuser -> Bool,
        perm_load_llm -> Bool,
        perm_unload_llm -> Bool,
        perm_download_llm -> Bool,
        perm_session -> Bool,
        perm_request_download -> Bool,
        perm_request_load -> Bool,
        perm_request_unload -> Bool,
        perm_view_llms -> Bool,
        perm_bare_model -> Bool,
    }
}

diesel::joinable!(llm_history -> llm_session (llm_session_id));
diesel::joinable!(llm_session -> llm (llm_uuid));
diesel::joinable!(llm_session -> user (user_id));
diesel::joinable!(requests -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    llm,
    llm_history,
    llm_session,
    requests,
    user,
);
