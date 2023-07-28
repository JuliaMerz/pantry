use crate::connectors::{LLMEvent, LLMInternalWrapper};
use crate::database_types::*;
use crate::llm::LLMSession;
use crate::state;
use crate::user::User;
use chrono::Utc;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde_json::Value;
use std::collections::HashMap;

use std::path::PathBuf;

use tiny_tokio_actor::*;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct OpenAIConnector {
    config: HashMap<String, Value>,
    uuid: Uuid,
    data_path: PathBuf,
    user_settings: state::UserSettings,
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl OpenAIConnector {
    pub fn new(
        uuid: Uuid,
        data_path: PathBuf,
        config: HashMap<String, Value>,
        user_settings: state::UserSettings,
        pool: Pool<ConnectionManager<SqliteConnection>>,
    ) -> OpenAIConnector {
        let mut path = data_path.clone();
        path.push(format!("openai-{}", uuid.to_string()));
        let conn = OpenAIConnector {
            config,
            data_path: path,
            uuid,
            user_settings,
            pool,
        };
        conn
    }
}

#[async_trait]
impl LLMInternalWrapper for OpenAIConnector {
    // async fn call_llm(&mut self, msg: String, session_params: HashMap<String, Value>, params: HashMap<String, Value>, user: User) -> Result<(Uuid, mpsc::Receiver<LLMEvent>), String> {
    //     println!("Triggered call llm for {:?} with \"{}\" and {:?}", user, msg, params);

    //     // Create a new session with the provided parameters
    //     let session_id = self.create_session(session_params, user.clone()).await?;
    //     println!("created a session");

    //     // Now that a new session is created, we need to prompt it immediately with the given message
    //     match self.prompt_session(session_id, msg, params, user).await {
    //         Ok(stream) => Ok((session_id, stream)),
    //         Err(e) => Err(e)
    //     }
    // }

    async fn maintenance(self: &mut Self) -> Result<(), String> {
        Ok(())
    }

    async fn create_session(
        self: &mut Self,
        params: HashMap<String, Value>,
        user: User,
    ) -> Result<Uuid, String> {
        // Here we create a new LLMSession, and push it to our sessions vector
        let _new_session = LLMSession {
            id: DbUuid(Uuid::new_v4()),
            started: Utc::now(),
            last_called: Utc::now(),
            user_id: user.id,                    // replace with actual user_id
            llm_uuid: DbUuid(self.uuid.clone()), // replace with actual llm_uuid
            session_parameters: DbHashMap(params),
        };

        // After adding the new session to our vector, we serialize the sessions vector to disk
        // Replace "sessions_path" with the actual path
        todo!()
    } //uuid
    async fn prompt_session(
        &mut self,
        _session_id: Uuid,
        _msg: String,
        _params: HashMap<String, Value>,
        _user: User,
        _sender: mpsc::Sender<LLMEvent>,
        _cancellation: CancellationToken,
    ) -> Result<(), String> {
        // Here we find the session by ID in our sessions vector
        println!("attempting to find session");
        todo!();
        Ok(())
    }
    async fn load_llm(self: &mut Self) -> Result<(), String> {
        return Ok(());
    }

    async fn pre_unload(self: &Self) -> Result<(), String> {
        todo!()
    } //called by shutdown

    async fn unload_llm(self: &Self) -> Result<(), String> {
        todo!()
    } //called by shutdown
}
