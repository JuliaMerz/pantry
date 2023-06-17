use chrono::DateTime;
use chrono::Utc;
use serde_json::Value;
use serde_json::json;
use tiny_tokio_actor::*;

use uuid::Uuid;

use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;
use std::collections::HashMap;

use crate::connectors::llm_actor;
use crate::connectors::llm_manager;
use crate::frontend;
use crate::error::PantryError;
use crate::connectors::llm_actor::LLMActor;
use crate::registry;
use crate::connectors;
use tokio::sync::mpsc;

use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};
use bincode;


// src/llm.rs


#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct HistoryItem {
    pub caller: String,
    pub request: String,
    pub response: String,
    pub timestamp: DateTime<Utc>
}

// #[serde(tag="type")]
#[derive(Clone)]
pub enum LLMResponseType {
    Str {string: String},
    Stream { }
}

// #[derive(serde::Serialize)]
pub struct LLMResponse {
    pub response: LLMResponseType, // Event poll for info.
    pub parameters: HashMap<String, Value>, //Final parameters used.
    // We can't put a stream into an enum because of clone, so this is our workaround.
    pub stream: Option<mpsc::Receiver<connectors::LLMEvent>>,
}

//Potentially unecessary wrapper class, written to give us space.
#[async_trait]
pub trait LLMWrapper {
    fn get_info(&self) -> frontend::LLMStatus;
    async fn status(&self) -> frontend::LLMStatus;
    async fn reload(&self) -> Result<(), PantryError>;
    async fn call_llm(&self, message: &str, parameters: HashMap<String, Value>) -> Result<LLMResponse, PantryError>;
    fn into_llm_running(&self) -> frontend::LLMRunning;
}

// Implements LLMWrapper
#[derive(serde::Serialize, serde::Deserialize)]
pub struct LLM {
    // Machine Info
    pub id: String, // Maybe?: https://github.com/alexanderatallah/window.ai/blob/main/packages/lib/README.md
    pub family_id: String, // Whole sets of models: Example's are GPT, LLaMA
    pub organization: String, // May be "None"

    // Human Info
    pub name: String,
    pub description: String,
    pub downloaded_reason: String,
    pub downloaded_date: DateTime<Utc>,
    pub last_called: RwLock<Option<DateTime<Utc>>>,

    // 0 is not capable, -1 is not evaluated.
    pub capabilities: HashMap<String, isize>,
    pub tags: Vec<String>,
    pub requirements: String,

    pub uuid: Uuid,

    pub history: Vec<HistoryItem>,

    // Functionality
    pub create_thread: bool, // Is it not an API connector?
    pub connector_type: connectors::LLMConnectorType, // which connector to use

    // Configs used by the connector for setup.
    pub config: HashMap<String, Value>, //Configs used by the connector


    // Parameters — these are submitted at call time.
    // these are the same, except one is configurable by users (programs or direct).
    // Hard coded parameters exist so repositories can deploy simple user friendly models
    // with preset configurations.
    pub parameters: HashMap<String, Value>, // Hardcoded Parameters
    pub user_parameters: Vec<String>, //User Parameters

    // #[serde(skip)]
    // pub connector: RwLock<Option<LLMConnector>>,
}

pub struct LLMActivated {
    pub llm: Arc<LLM>,
    pub activated_reason: String,
    pub activated_time: DateTime<Utc>,
    actor: ActorRef<connectors::SysEvent, llm_actor::LLMActor>
}

impl Clone for LLM {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            family_id: self.family_id.clone(),
            organization: self.organization.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            downloaded_reason: self.downloaded_reason.clone(),
            downloaded_date: self.downloaded_date.clone(),
            last_called: RwLock::new(*self.last_called.read().unwrap()),  // clone inner value

            uuid: self.uuid.clone(),

            capabilities: self.capabilities.clone(),
            tags: self.tags.clone(),
            history: self.history.clone(),

            requirements: self.requirements.clone(),

            create_thread: self.create_thread.clone(),
            connector_type: self.connector_type.clone(), // assuming this type is also Clone
            config: self.config.clone(),
            parameters: self.parameters.clone(),
            user_parameters: self.user_parameters.clone(),
        }
    }
}


pub fn serialize_llms(path: PathBuf, llms: &Vec<LLM>) -> Result<(), Box<dyn std::error::Error>> {
    let encoded = bincode::serialize(llms)?;
    let mut file = File::create(path)?;
    file.write_all(&encoded)?;
    Ok(())
}

pub fn deserialize_llms(path: PathBuf) -> Result<Vec<LLM>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let llms: Vec<LLM> = bincode::deserialize(&buffer)?;
    Ok(llms)
}


impl LLMActivated {
    pub async fn activate_llm(llm: Arc<LLM>, manager_addr: ActorRef<connectors::SysEvent, llm_manager::LLMManagerActor>) -> Result<LLMActivated, PantryError> {
        match manager_addr.ask(llm_manager::CreateLLMActorMessage(llm.id.clone(), llm.connector_type.clone(), llm.config.clone())).await {
            Ok(result) => {
                match result {
                    // At this point we've created the LLM actor.
                    Ok(val) => Ok(LLMActivated {
                        llm: llm,
                        activated_reason: "User request".into(),
                        activated_time: chrono::offset::Utc::now(),
                        actor: val
                    }),
                    Err(err) => Err(err)
                }

            },
            Err(err) => Err(PantryError::ActorFailure(err))
        }
    }
}

#[async_trait]
impl LLMWrapper for LLMActivated {
    fn get_info(&self) -> frontend::LLMStatus {
        frontend::LLMStatus {
            status: format!("ID: {}, Name: {}, Description: {}", self.llm.id, self.llm.name, self.llm.description),
        }
    }

    async fn status(&self) -> frontend::LLMStatus {
        todo!()
    }

    async fn reload(&self) -> Result<(), PantryError> {
        todo!()
    }

    // kinda ugly that we need mutability here for a potentially long call, for a short mut.
    async fn call_llm(&self, message: &str, parameters: HashMap<String, Value>) -> Result<LLMResponse, PantryError> {


        // Reconcile Parameters
        let mut armed_params = self.llm.parameters.clone();
        for param in self.llm.user_parameters.iter() {
            if let Some(val) = parameters.get(param) {
                armed_params.insert(param.clone(), json!(val.clone()));
            }
        }

        println!("Called {} with {} and params {:?}", self.llm.id, message, armed_params);


        // WE CREATE A THREAD
        // THREAD EMITS EVENTS WHILE RECEIVER
        // EMITS SPECIAL END EVENT WHEN RECEIVE DONE
        // response is CODE for SPECIAL EVENT

        match self.actor.ask(llm_actor::CallLLMMessage(message.into(), armed_params.clone())).await {
            Ok(result) => match result {
                Ok(stream) => Ok(
                  LLMResponse {
                        response: LLMResponseType::Stream {},
                        stream: Some(stream),
                        parameters: armed_params,
                    }),
                Err(err) => Err(PantryError::OtherFailure(err))
            },
            Err(err) => Err(PantryError::ActorFailure(err))

        }



    }

    fn into_llm_running(&self) -> frontend::LLMRunning {
        self.into()
    }
}




