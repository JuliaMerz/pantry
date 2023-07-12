




use diesel::prelude::*;

use diesel::r2d2::{ConnectionManager, Pool};











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
    results.map_err(|err| format!("Failed because: {:?}", err.to_string()))
}

pub fn get_available_llms(
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<LLM>, String> {
    let _conn = &mut pool.get().unwrap();
    todo!()
}
pub fn get_llm_session(
    _llm_session_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMSession, String> {
    let _conn = &mut pool.get().unwrap();
    todo!()
}

pub fn get_llm_history(
    _llm_history_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMHistoryItem, String> {
    let _conn = &mut pool.get().unwrap();
    todo!()
}

pub fn get_request(
    _request_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<UserRequest, String> {
    let _conn = &mut pool.get().unwrap();
    todo!()
}

pub fn get_user(
    _user_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<User, String> {
    let _conn = &mut pool.get().unwrap();
    todo!()
}

pub fn get_sessions_for_llm(
    _llm_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<LLMSession>, String> {
    let _conn = &mut pool.get().unwrap();
    todo!()
}

pub fn get_history_for_session(
    _llm_session_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<LLMHistoryItem>, String> {
    let _conn = &mut pool.get().unwrap();
    todo!()
}

pub fn update_llm_last_called(
    _llm: LLM,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLM, String> {
    let _conn = &mut pool.get().unwrap();
    todo!()
}

pub fn update_session_last_called(
    _llm_session: LLMSession,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMSession, String> {
    let _conn = &mut pool.get().unwrap();
    todo!()
}

// set complete bool at the same time
pub fn append_token(
    _llm_history: LLMHistoryItem,
    _t: String,
    _complete: bool,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMHistoryItem, String> {
    let _conn = &mut pool.get().unwrap();
    //remember to update timeastamps!~
    todo!()
}

pub fn save_new_llm(
    _llm: LLM,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLM, String> {
    let _conn = &mut pool.get().unwrap();
    todo!()
}

pub fn save_new_llm_session(
    _llm_session: LLMSession,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMSession, String> {
    let _conn = &mut pool.get().unwrap();
    todo!()
}

pub fn save_new_llm_history(
    _llm_history: LLMHistoryItem,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<LLMHistoryItem, String> {
    let _conn = &mut pool.get().unwrap();
    // Remember to write a smoooooooth update statement to update session last called
    todo!()
}

pub fn save_new_request(
    _request: UserRequest,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<UserRequest, String> {
    let _conn = &mut pool.get().unwrap();
    todo!()
}

pub fn save_new_user(
    _user: User,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<User, String> {
    let _conn = &mut pool.get().unwrap();
    todo!()
}

pub fn get_llm_sessions_user(
    _user: User,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<Vec<LLMSession>, String> {
    let _conn = &mut pool.get().unwrap();
    todo!()
}

pub fn delete_llm(
    _llm_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<User, String> {
    let _conn = &mut pool.get().unwrap();
    todo!()
}
