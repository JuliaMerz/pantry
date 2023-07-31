use crate::state;
use crate::{llm::LLMSession, user};
use chrono::prelude::*;
use diesel::deserialize::FromSql;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::serialize::{self, Output, ToSql};
use diesel::sqlite::{Sqlite, SqliteValue};
use diesel::*;
use serde_json::Value;
use std::fmt;
use std::io::Write;

use std::{collections::HashMap, path::PathBuf};
use tiny_tokio_actor::*;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub mod factory;
pub mod llm_actor;
pub mod llm_manager;

pub mod generic;
pub mod llmrs;
pub mod openai;

//src/connectors/mod.rs

// We use the system event for debug monitoring
#[derive(Clone, Debug)]
pub struct SysEvent(String);

impl SystemEvent for SysEvent {}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, FromSqlRow, AsExpression)]
#[diesel(sql_type = diesel::sql_types::Text)]
#[serde(rename_all = "lowercase")]
pub enum LLMConnectorType {
    GenericAPI,
    LLMrs,
    OpenAI,
}

impl fmt::Display for LLMConnectorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LLMConnectorType::GenericAPI => write!(f, "GenericAPI"),
            LLMConnectorType::LLMrs => write!(f, "LLMrs"),
            LLMConnectorType::OpenAI => write!(f, "OpenAI"),
        }
    }
}

impl FromSql<diesel::sql_types::Text, Sqlite> for LLMConnectorType {
    fn from_sql(bytes: SqliteValue<'_, '_, '_>) -> diesel::deserialize::Result<Self> {
        let str = <String as FromSql<diesel::sql_types::Text, Sqlite>>::from_sql(bytes)?;
        match str.as_str() {
            "GenericAPI" => Ok(LLMConnectorType::GenericAPI),
            "LLMrs" => Ok(LLMConnectorType::LLMrs),
            "OpenAI" => Ok(LLMConnectorType::OpenAI),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl ToSql<diesel::sql_types::Text, Sqlite> for LLMConnectorType {
    fn to_sql<'W>(&'W self, out: &mut Output<'W, '_, Sqlite>) -> serialize::Result {
        match *self {
            LLMConnectorType::GenericAPI => out.set_value("GenericAPI"),
            LLMConnectorType::LLMrs => out.set_value("LLMrs"),
            LLMConnectorType::OpenAI => out.set_value("OpenAI"),
        }
        Ok(serialize::IsNull::No)
    }
}

pub fn get_new_llm_connector(
    connector_type: LLMConnectorType,
    uuid: Uuid,
    data_path: PathBuf,
    config: HashMap<String, Value>,
    model_path: Option<PathBuf>,
    user_settings: state::UserSettings,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Box<dyn LLMInternalWrapper> {
    match connector_type {
        LLMConnectorType::GenericAPI => Box::new(generic::GenericAPIConnector::new(
            uuid,
            data_path,
            config,
            user_settings,
            pool,
        )),
        LLMConnectorType::OpenAI => Box::new(openai::OpenAIConnector::new(
            uuid,
            data_path,
            config,
            user_settings,
            pool,
        )),
        LLMConnectorType::LLMrs => Box::new(llmrs::LLMrsConnector::new(
            uuid,
            data_path,
            config,
            model_path.unwrap(),
            user_settings,
            pool,
        )),
    }
}

/* Actually connect to the LLMs */
#[async_trait]
pub trait LLMInternalWrapper: Send + Sync {
    // async fn call_llm(self: &mut Self, msg: String, session_params: HashMap<String, Value>, params: HashMap<String, Value>, user: user::User, sender: mpsc::Sender<LLMEvent>) -> Result<Uuid, String>;
    // kill get_sessions, they should be in the db now.
    // async fn get_sessions(self: &Self, user: user::User) -> Result<Vec<LLMSession>, String>;
    //mut because we're going to modify our internal session storage
    async fn create_session(
        self: &mut Self,
        params: HashMap<String, Value>,
        user: user::User,
    ) -> Result<Uuid, String>; //uuid
                               //mut because we're going to modify our internal session storage
    async fn prompt_session(
        self: &mut Self,
        session_id: Uuid,
        msg: String,
        params: HashMap<String, Value>,
        user: user::User,
        sender: mpsc::Sender<LLMEvent>,
        cancellation: CancellationToken,
    ) -> Result<(), String>;

    async fn load_llm(self: &mut Self) -> Result<(), String>;
    async fn pre_unload(self: &Self) -> Result<(), String>; //called by manager before shutdown
    async fn unload_llm(self: &Self) -> Result<(), String>; //called by shutdown
    async fn maintenance(self: &mut Self) -> Result<(), String>;
}

#[derive(Clone, serde::Serialize, Debug)]
pub struct LLMEvent {
    stream_id: Uuid,
    timestamp: DateTime<Utc>,
    call_timestamp: DateTime<Utc>,
    parameters: HashMap<String, Value>,
    input: String,
    llm_uuid: Uuid,
    session: LLMSessionStatus,
    event: LLMEventInternal,
}

// We don't want to expose DbUuid to outside parties.
// llmevent gets used by listeners (aka the API) so we want
// a different type for it.
#[derive(Clone, serde::Serialize, Debug)]
pub struct LLMSessionStatus {
    pub id: Uuid, //this is a uuid
    pub llm_uuid: Uuid,
    pub user_id: Uuid,
    pub started: DateTime<Utc>,
    pub last_called: DateTime<Utc>,
    pub session_parameters: HashMap<String, Value>,
}
impl From<&LLMSession> for LLMSessionStatus {
    fn from(sess: &LLMSession) -> Self {
        LLMSessionStatus {
            id: sess.id.0.clone(), //this is a uuid
            llm_uuid: sess.llm_uuid.0.clone(),
            user_id: sess.user_id.0.clone(),
            started: sess.started.clone(),
            last_called: sess.last_called.clone(),
            session_parameters: sess.session_parameters.0.clone(),
        }
    }
}

#[derive(Clone, serde::Serialize, Debug)]
#[serde(tag = "type")]
pub enum LLMEventInternal {
    PromptProgress { previous: String, next: String }, // Next words of an LLM.
    PromptCompletion { previous: String },             // Finished the prompt
    PromptError { message: String },
    Other,
}
