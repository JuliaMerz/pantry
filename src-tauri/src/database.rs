use chrono::{DateTime, Utc};
use diesel::backend::Backend;
use diesel::connection::SimpleConnection;
use diesel::deserialize::{self, FromSql};
use diesel::expression::{Expression, NonAggregate, SelectableExpression};
use diesel::prelude::*;
use diesel::query_builder::QueryId;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::Text;
use diesel::sqlite::{Sqlite, SqliteValue};
use diesel::*;
use serde_json;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::RwLock;
use uuid::Uuid;

use crate::database_types::*;
use crate::llm::{LLMHistoryItem, LLMSession, LLM};
use crate::request::UserRequest;
use crate::schema;
use crate::user::User;

pub fn get_llm(
    llm_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLM, String> {
    let conn = &mut pool.get().unwrap();
    use schema::llm::dsl::*;
    let results = llm
        .filter(uuid.eq(DbUuid(llm_id)))
        .select(LLM::as_select())
        .first(conn);
    Ok(results)
}

pub fn get_available_llms(
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<LLM>, String> {
    let conn = &mut pool.get().unwrap();
    todo!()
}
pub fn get_llm_session(
    llm_session_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMSession, String> {
    let conn = &mut pool.get().unwrap();
    todo!()
}

pub fn get_llm_history(
    llm_history_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMHistoryItem, String> {
    let conn = &mut pool.get().unwrap();
    todo!()
}

pub fn get_request(
    request_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<UserRequest, String> {
    let conn = &mut pool.get().unwrap();
    todo!()
}

pub fn get_user(
    user_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<User, String> {
    let conn = &mut pool.get().unwrap();
    todo!()
}

pub fn get_sessions_for_llm(
    llm_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<LLMSession>, String> {
    let conn = &mut pool.get().unwrap();
    todo!()
}

pub fn get_history_for_session(
    llm_session_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<LLMHistoryItem>, String> {
    let conn = &mut pool.get().unwrap();
    todo!()
}

pub fn update_llm_last_called(
    llm: LLM,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLM, String> {
    let conn = &mut pool.get().unwrap();
    todo!()
}

pub fn update_session_last_called(
    llm_session: LLMSession,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMSession, String> {
    let conn = &mut pool.get().unwrap();
    todo!()
}

// set complete bool at the same time
pub fn append_token(
    llm_history: LLMHistoryItem,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMHistoryItem, String> {
    let conn = &mut pool.get().unwrap();
    todo!()
}

pub fn save_new_llm(
    llm: LLM,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLM, String> {
    let conn = &mut pool.get().unwrap();
    todo!()
}

pub fn save_new_llm_session(
    llm_session: LLMSession,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMSession, String> {
    let conn = &mut pool.get().unwrap();
    todo!()
}

pub fn save_new_llm_history(
    llm_history: LLMHistoryItem,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMHistoryItem, String> {
    let conn = &mut pool.get().unwrap();
    todo!()
}

pub fn save_new_request(
    request: UserRequest,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<UserRequest, String> {
    let conn = &mut pool.get().unwrap();
    todo!()
}

pub fn save_new_user(
    user: User,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<User, String> {
    let conn = &mut pool.get().unwrap();
    todo!()
}

pub fn delete_llm(
    llm_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<User, String> {
    let conn = &mut pool.get().unwrap();
    todo!()
}
