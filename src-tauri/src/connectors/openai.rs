use crate::connectors::{LLMInternalWrapper, LLMEvent, LLMEventInternal};
use crate::llm::{LLMSession, LLMHistoryItem};
use crate::user::User;
use dashmap::DashMap;
use tiny_tokio_actor::*;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use tokio::sync::mpsc;
use serde_json::Value;
use std::collections::HashMap;
use llm::VocabularySource;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::fs::File;
use tiny_tokio_actor::*;



pub struct OpenAIConnector {
    config: HashMap<String, Value>,
    uuid: Uuid,
    data_path: PathBuf,
    sessions: DashMap<Uuid, LLMSession>

}

impl OpenAIConnector {
    pub fn new(uuid: Uuid, data_path: PathBuf, config: HashMap<String, Value>) -> OpenAIConnector {
        let mut path = data_path.clone();
        path.push(format!("openai-{}", uuid.to_string()));
        let mut conn = OpenAIConnector {
            config: config,
            data_path: path,
            uuid: uuid,
            sessions: DashMap::new(),
        };
        conn.deserialize_sessions();
        conn
    }

    // Utility functions to serialize and deserialize sessions
    pub fn serialize_sessions(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(&self.data_path)?;
        rmp_serde::encode::write_named(&mut file, &self.sessions)?;
        Ok(())
    }

    pub fn deserialize_sessions(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(&self.data_path)?;
        let sessions: Vec<LLMSession> = rmp_serde::decode::from_read(&file)?;
        self.sessions = DashMap::new();
        sessions.into_iter().map(|sess| {
            self.sessions.insert(sess.id, sess);
        });
        Ok(())
    }
}

#[async_trait]
impl LLMInternalWrapper for OpenAIConnector {
    async fn call_llm(&mut self, msg: String, params: HashMap<String, Value>, user: User) -> Result<mpsc::Receiver<LLMEvent>, String> {

        // Create a new session with the provided parameters
        let session_id = self.create_session(params, user.clone()).await?;

        // Now that a new session is created, we need to prompt it immediately with the given message
        self.prompt_session(session_id, msg, user).await
    }

    async fn get_sessions(&self, user: User) -> Result<Vec<LLMSession>, String> {
        // Filter sessions by user ID and clone them into a new vector
        let user_sessions = self.sessions.clone()
            .into_iter()
            .filter(|(uuid, session)| session.user_id == user.id)
            .map(|(uuid, llm_sess)| llm_sess)
            .collect();

        Ok(user_sessions)
    }


    async fn create_session(self: &mut Self, params: HashMap<String, Value>, user: User) -> Result<Uuid, String> {
                // Here we create a new LLMSession, and push it to our sessions vector
        let new_session = LLMSession {
            id: Uuid::new_v4(),
            started: Utc::now(),
            user_id: user.id, // replace with actual user_id
            llm_uuid: self.uuid.clone(), // replace with actual llm_uuid
            parameters: params,
            items: vec![],
        };

        self.sessions.insert(new_session.id, new_session.clone());

        // After adding the new session to our vector, we serialize the sessions vector to disk
        // Replace "sessions_path" with the actual path
        match self.serialize_sessions() {
            Ok(_) => Ok(new_session.id), // return the session ID
            Err(err) => Err(err.to_string()),
        }

    } //uuid
    async fn prompt_session(&mut self, session_id: Uuid, msg: String, user: User) -> Result<mpsc::Receiver<LLMEvent>, String> {
        // Here we find the session by ID in our sessions vector
        match self.sessions.iter_mut().find(|session| session.id == session_id) {
            Some(mut session) => {
                // If the session is found, we add a new history item
                let item_id = Uuid::new_v4();
                let new_item = LLMHistoryItem {
                    id: item_id.clone(),
                    updated_timestamp: Utc::now(),
                    call_timestamp: Utc::now(),
                    complete: false, // initially false, will be set to true once response is received
                    parameters: session.parameters.clone(),
                    input: msg.clone(),
                    output: "".into(),
                };

                session.items.push(new_item.clone());

                // Here you should actually make the API call using the session and msg.
                // eventual opani api calls

                let (sender, receiver):(mpsc::Sender<LLMEvent>, mpsc::Receiver<LLMEvent>) = mpsc::channel(100);
                sender.send(LLMEvent{
                    stream_id: item_id.clone(),
                    timestamp: new_item.updated_timestamp.clone(),
                    call_timestamp: new_item.call_timestamp.clone(),
                    parameters: new_item.parameters.clone(),
                    input: msg.clone(),
                    llm_uuid: self.uuid.clone(),
                    session: session.clone(),
                    event:LLMEventInternal::PromptProgress { previous: "".into(), next: "boop".into() }
                }).await;
                sender.send(LLMEvent{
                    stream_id: item_id.clone(),
                    timestamp: new_item.updated_timestamp.clone(),
                    call_timestamp: new_item.call_timestamp.clone(),
                    parameters: new_item.parameters.clone(),
                    input: msg.clone(),
                    llm_uuid: self.uuid.clone(),
                    session: session.clone(),
                    event:LLMEventInternal::PromptCompletion { previous: "boop".into() }
                }).await;

                let update_item = session.items.iter_mut().find(|item| item.id == item_id).ok_or("couldn't find session we just made")?;
                update_item.output = "boop".into();
                update_item.complete = true;



                Ok(receiver)
            },
            None => Err(format!("Session with id {} not found.", session_id)),
        }
    }
    async fn load_llm(self: &mut Self) -> Result<(), String> {

        return Ok(())

    }

    async fn unload_llm(self: &Self, ) -> Result<(), String> {
        todo!()
    }//called by shutdown
}
