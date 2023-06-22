use crate::connectors::{LLMInternalWrapper, LLMEvent, LLMEventInternal};
use crate::llm::{LLMSession, LLMHistoryItem};
use crate::user::User;
use uuid::Uuid;
use std::{collections::HashMap, path::PathBuf};
use tiny_tokio_actor::*;
use tokio::sync::mpsc;
use serde_json::Value;

pub struct GenericAPIConnector {
    config: HashMap<String, Value>

}

impl GenericAPIConnector {
    pub fn new(uuid: Uuid, data_path: PathBuf, config: HashMap<String, Value>) -> GenericAPIConnector {
        GenericAPIConnector {
            config: config,
        }
    }
}

#[async_trait]
impl LLMInternalWrapper for GenericAPIConnector {
    async fn call_llm(self: &mut Self, msg: String, params: HashMap<String, Value>, user:User) -> Result<mpsc::Receiver<LLMEvent>, String> {
        todo!()
    }
    async fn get_sessions(self: &Self, user: User) -> Result<Vec<LLMSession>, String> {
        todo!()
    }

    async fn create_session(self: &mut Self, params: HashMap<String, Value>, user: User) -> Result<Uuid, String> {
        todo!()
    } //uuid
    async fn prompt_session(self: &mut Self, session_id: Uuid, msg: String, user: User) -> Result<mpsc::Receiver<LLMEvent>, String> {
        todo!()
    }
    async fn load_llm(self: &mut Self, ) -> Result<(), String> {
        todo!()
    }
    async fn unload_llm(self: &Self, ) -> Result<(), String> {
        todo!()
    }//called by shutdown

}
