use crate::connectors::{LLMEvent, LLMEventInternal, LLMInternalWrapper};
use crate::llm::{LLMHistoryItem, LLMSession};
use crate::state;
use crate::user::User;
use serde_json::Value;
use std::{collections::HashMap, path::PathBuf};
use tiny_tokio_actor::*;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct GenericAPIConnector {
    config: HashMap<String, Value>,
    user_settings: state::UserSettings,
}

impl GenericAPIConnector {
    pub fn new(
        uuid: Uuid,
        data_path: PathBuf,
        config: HashMap<String, Value>,
        user_settings: state::UserSettings,
    ) -> GenericAPIConnector {
        GenericAPIConnector {
            config: config,
            user_settings: user_settings,
        }
    }
}

#[async_trait]
impl LLMInternalWrapper for GenericAPIConnector {
    // async fn call_llm(self: &mut Self, msg: String, session_params: HashMap<String, Value>, params: HashMap<String, Value>, user:User) -> Result<(Uuid, mpsc::Receiver<LLMEvent>), String> {
    // todo!()
    // }
    async fn get_sessions(self: &Self, user: User) -> Result<Vec<LLMSession>, String> {
        todo!()
    }

    async fn create_session(
        self: &mut Self,
        params: HashMap<String, Value>,
        user: User,
    ) -> Result<Uuid, String> {
        todo!()
    } //uuid
    async fn prompt_session(
        self: &mut Self,
        session_id: Uuid,
        msg: String,
        params: HashMap<String, Value>,
        user: User,
        sender: mpsc::Sender<LLMEvent>,
        cancellation: CancellationToken,
    ) -> Result<(), String> {
        todo!()
    }
    async fn load_llm(self: &mut Self) -> Result<(), String> {
        todo!()
    }
    async fn unload_llm(self: &Self) -> Result<(), String> {
        todo!()
    } //called by shutdown
}
