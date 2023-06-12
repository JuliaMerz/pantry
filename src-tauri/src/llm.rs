use chrono::DateTime;
use chrono::Utc;
use tiny_tokio_actor::*;

use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;
use std::collections::HashMap;

use crate::connectors::llm_actor;
use crate::connectors::llm_manager;
use crate::frontend;
use crate::error::PantryError;
use crate::connectors::llm_actor::LLMActor;
use crate::connectors::registry;
use crate::connectors;

use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};
use bincode;


// src/llm.rs

//Potentially unecessary wrapper class, written to give us space.
#[async_trait]
pub trait LLMWrapper {
    fn get_info(&self) -> frontend::LLMStatus;
    async fn status(&self) -> frontend::LLMStatus;
    async fn load(&self) -> Result<(), PantryError>;
    async fn reload(&self) -> Result<(), PantryError>;
    async fn unload(&self) -> Result<(), String>;
    async fn call(&mut self, message: &str, parameters_json: &str) -> Result<String, &'static str>;
    fn into_llm_running(&self) -> frontend::LLMRunning;
}

// Implements LLMWrapper
#[derive(serde::Serialize, serde::Deserialize)]
pub struct LLM {
    // Info
    pub id: String,
    pub name: String,
    pub description: String,
    pub downloaded_reason: String,
    pub downloaded_date: DateTime<Utc>,
    pub last_called: RwLock<Option<DateTime<Utc>>>,


    // Functionality
    pub create_thread: bool, // Is it an API connector?
    pub connector_type: connectors::LLMConnectorType, // which connector to use
    pub config: Vec<(String, String)>, //Configs used by the connector
    pub parameters: Vec<(String, String)>, // Hardcoded Parameters
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
            name: self.name.clone(),
            description: self.description.clone(),
            downloaded_reason: self.downloaded_reason.clone(),
            downloaded_date: self.downloaded_date.clone(),
            last_called: RwLock::new(*self.last_called.read().unwrap()),  // clone inner value
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
            Err(err) => Err(PantryError::LLMStartupFailed(err))
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

    async fn load(&self) -> Result<(), PantryError> {
        println!("Sending LLM bootup message");
        match self.actor.ask(llm_actor::Load {}).await {
            Ok(val) => {
                Ok(())
            },
            Err(err) => Err(PantryError::LLMStartupFailed(err))
        }
        // // Get the address of the LLMManagerActor (this might come from your GlobalState, for example)
    }

    async fn reload(&self) -> Result<(), PantryError> {
        todo!()
    }

    async fn unload(&self) -> Result<(), String> {
        todo!()
    }

    // kinda ugly that we need mutability here for a potentially long call, for a short mut.
    async fn call(&mut self, message: &str, parameters_json: &str) -> Result<String, &'static str> {
        todo!()
    }

    fn into_llm_running(&self) -> frontend::LLMRunning {
        frontend::LLMRunning {
            llm_info: frontend::LLMInfo {
                id: self.llm.id.clone(),
                name: self.llm.name.clone(),
                description: self.llm.description.clone()
            },
            downloaded: format!("Downloaded {} for {}", self.llm.downloaded_date, self.llm.downloaded_reason),
            last_called: self.llm.last_called.read().unwrap().clone(),
            activated: format!("Activated {} for {}", self.activated_time, self.activated_reason)
        }
        //THEN IMPLEMENT CALL()
    }
}




