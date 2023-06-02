use chrono::DateTime;
use chrono::Utc;
use tiny_tokio_actor::*;

use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;
use std::collections::HashMap;

use crate::frontend::{LLMInfo, LLMAvailable, LLMRunning, LLMStatus};
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
pub trait LLMWrapper {
    fn llm_running(&self) -> bool;
    fn get_info(&self) -> LLMStatus;
    fn status(&self) -> LLMStatus;
    fn load(&self) -> Result<(), String>;
    fn unload(&self) -> Result<(), String>;
    fn call(&self, message: &str, parameters_json: &str) -> Result<String, &'static str>;
    fn into_llm_running(&self) -> Result<LLMRunning, PantryError>;
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
    pub last_called: RwLock<DateTime<Utc>>,


    // Functionality
    pub create_thread: bool, // Is it an API connector?
    pub connector_type: connectors::LLMConnectorType, // which connector to use
    pub config: Vec<(String, String)>, //Configs used by the connector
    pub parameters: Vec<(String, String)>, // Hardcoded Parameters
    pub user_parameters: Vec<String>, //User Parameters



    #[serde(skip)]
    pub connector: Option<LLMConnector>,
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
            connector: None,  // always clone as None
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

pub struct LLMConnector {
    pub activated: String,
    pub actor: ActorRef<connectors::SysEvent, LLMActor>,
}



impl LLMWrapper for LLM {
    fn llm_running(&self) -> bool {
        self.connector.is_some()
    }

    fn get_info(&self) -> LLMStatus {
        LLMStatus {
            status: format!("ID: {}, Name: {}, Description: {}", self.id, self.name, self.description),
        }
    }

    fn status(&self) -> LLMStatus {
        todo!()
    }

    fn load(&self) -> Result<(), String> {
        todo!()
        // // Get the address of the LLMManagerActor (this might come from your GlobalState, for example)
    }

    fn unload(&self) -> Result<(), String> {
        todo!()
    }

    fn call(&self, message: &str, parameters_json: &str) -> Result<String, &'static str> {
        todo!()
    }

    fn into_llm_running(&self) -> Result<LLMRunning, PantryError> {
        //IMPLEMENT HERE
        //THEN IMPLEMENT CALL()
    }
}




