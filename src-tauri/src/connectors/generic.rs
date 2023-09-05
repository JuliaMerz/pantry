use crate::connectors::{LLMEvent, LLMInternalWrapper};

use crate::state;
use crate::user::User;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde_json::Value;
use std::{collections::HashMap, path::PathBuf};
use tiny_tokio_actor::*;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct GenericAPIConnector {
    config: HashMap<String, Value>,
    user_settings: state::UserSettings,
    pub pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl GenericAPIConnector {
    pub fn new(
        _uuid: Uuid,
        _data_path: PathBuf,
        config: HashMap<String, Value>,
        user_settings: state::UserSettings,
        pool: Pool<ConnectionManager<SqliteConnection>>,
    ) -> GenericAPIConnector {
        GenericAPIConnector {
            config,
            user_settings,
            pool,
        }
    }
}

#[async_trait]
impl LLMInternalWrapper for GenericAPIConnector {
    // async fn call_llm(self: &mut Self, msg: String, session_params: HashMap<String, Value>, params: HashMap<String, Value>, user:User) -> Result<(Uuid, mpsc::Receiver<LLMEvent>), String> {
    // todo!()
    // }
    async fn maintenance(self: &mut Self) -> Result<(), String> {
        Ok(())
    }

    async fn create_session(
        self: &mut Self,
        _params: HashMap<String, Value>,
        _user: User,
    ) -> Result<Uuid, String> {
        todo!()
    } //uuid
    async fn prompt_session(
        self: &mut Self,
        _session_id: Uuid,
        _msg: String,
        _params: HashMap<String, Value>,
        _user: User,
        _sender: mpsc::Sender<LLMEvent>,
        _cancellation: CancellationToken,
    ) -> Result<(), String> {
        todo!()
    }
    async fn load_llm(self: &mut Self) -> Result<(), String> {
        todo!()
    }
    async fn pre_unload(self: &Self) -> Result<(), String> {
        todo!()
    } //called before shutdown
    async fn unload_llm(self: &Self) -> Result<(), String> {
        todo!()
    } //called by shutdown
}
