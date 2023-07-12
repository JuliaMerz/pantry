-- Your SQL goes here

-- migrations/202307091235_up.sql
-- Replace all Timestamp with TimestamptzSqlite

CREATE TABLE llm (
    uuid TEXT PRIMARY KEY NOT NULL,
    id TEXT NOT NULL,
    family_id TEXT NOT NULL,
    organization TEXT NOT NULL,
    name TEXT NOT NULL,
    homepage TEXT NOT NULL,
    description TEXT NOT NULL,
    license TEXT NOT NULL,
    downloaded_reason TEXT NOT NULL,
    downloaded_date DATETIME NOT NULL,
    last_called DATETIME,
    capabilities TEXT NOT NULL,
    tags TEXT NOT NULL,
    requirements TEXT NOT NULL,
    url TEXT NOT NULL,
    create_thread BOOLEAN NOT NULL,
    connector_type TEXT NOT NULL,
    config TEXT NOT NULL,
    model_path TEXT,
    parameters TEXT NOT NULL,
    user_parameters TEXT NOT NULL,
    session_parameters TEXT NOT NULL,
    user_session_parameters TEXT NOT NULL
);

CREATE TABLE requests (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    originator TEXT NOT NULL,
    reason TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    request TEXT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES user(id)
);

CREATE TABLE llm_history (
    id TEXT PRIMARY KEY NOT NULL,
    llm_session_id TEXT NOT NULL,
    updated_timestamp DATETIME NOT NULL,
    call_timestamp DATETIME NOT NULL,
    complete BOOLEAN NOT NULL,
    parameters TEXT NOT NULL,
    input TEXT NOT NULL,
    output TEXT NOT NULL,
    FOREIGN KEY(llm_session_id) REFERENCES llm_session(id)

);

CREATE TABLE llm_session (
    id TEXT PRIMARY KEY NOT NULL,
    llm_uuid TEXT NOT NULL,
    user_id TEXT NOT NULL,
    started DATETIME NOT NULL,
    last_called DATETIME NOT NULL,
    session_parameters TEXT NOT NULL,
    FOREIGN KEY(llm_uuid) REFERENCES llm(uuid),
    FOREIGN KEY(user_id) REFERENCES user(id)
);

CREATE TABLE user (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    api_key TEXT NOT NULL,
    perm_superuser BOOLEAN NOT NULL,
    perm_load_llm BOOLEAN NOT NULL,
    perm_unload_llm BOOLEAN NOT NULL,
    perm_download_llm BOOLEAN NOT NULL,
    perm_session BOOLEAN NOT NULL,
    perm_request_download BOOLEAN NOT NULL,
    perm_request_load BOOLEAN NOT NULL,
    perm_request_unload BOOLEAN NOT NULL
);
