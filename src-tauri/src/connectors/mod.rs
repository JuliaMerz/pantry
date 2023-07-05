use crate::state;
use crate::{llm::LLMSession, registry::LLMRegistryEntryConnector, user};
use chrono::prelude::*;
use serde_json::Value;
use std::fmt;
use std::sync::{Arc, RwLock};
use std::{collections::HashMap, path::PathBuf};
use tiny_tokio_actor::*;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::error::PantryError;

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

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
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

pub fn get_new_llm_connector(
    connector_type: LLMConnectorType,
    uuid: Uuid,
    data_path: PathBuf,
    config: HashMap<String, Value>,
    model_path: Option<PathBuf>,
    user_settings: state::UserSettings,
) -> Box<dyn LLMInternalWrapper> {
    match connector_type {
        LLMConnectorType::GenericAPI => Box::new(generic::GenericAPIConnector::new(
            uuid,
            data_path,
            config,
            user_settings,
        )),
        LLMConnectorType::OpenAI => Box::new(openai::OpenAIConnector::new(
            uuid,
            data_path,
            config,
            user_settings,
        )),
        LLMConnectorType::LLMrs => Box::new(llmrs::LLMrsConnector::new(
            uuid,
            data_path,
            config,
            model_path.unwrap(),
            user_settings,
        )),
    }
}

// Conversion from the format used by the index into our internal typing
impl From<LLMRegistryEntryConnector> for LLMConnectorType {
    fn from(value: LLMRegistryEntryConnector) -> Self {
        match value {
            LLMRegistryEntryConnector::GenericAPI => LLMConnectorType::GenericAPI,
            LLMRegistryEntryConnector::Ggml => LLMConnectorType::LLMrs,
            LLMRegistryEntryConnector::LLMrs => LLMConnectorType::LLMrs,
            LLMRegistryEntryConnector::OpenAI => LLMConnectorType::OpenAI,
        }
    }
}

/* Actually connect to the LLMs */
#[async_trait]
pub trait LLMInternalWrapper: Send + Sync {
    // async fn call_llm(self: &mut Self, msg: String, session_params: HashMap<String, Value>, params: HashMap<String, Value>, user: user::User, sender: mpsc::Sender<LLMEvent>) -> Result<Uuid, String>;
    async fn get_sessions(self: &Self, user: user::User) -> Result<Vec<LLMSession>, String>;
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
    async fn unload_llm(self: &Self) -> Result<(), String>; //called by shutdown
}

#[derive(Clone, serde::Serialize, Debug)]
pub struct LLMEvent {
    stream_id: Uuid,
    timestamp: DateTime<Utc>,
    call_timestamp: DateTime<Utc>,
    parameters: HashMap<String, Value>,
    input: String,
    llm_uuid: Uuid,
    session: LLMSession,
    event: LLMEventInternal,
}

#[derive(Clone, serde::Serialize, Debug)]
#[serde(tag = "type")]
pub enum LLMEventInternal {
    PromptProgress { previous: String, next: String }, // Next words of an LLM.
    PromptCompletion { previous: String },             // Finished the prompt
    PromptError { message: String },
    Other,
}
