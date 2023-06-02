use crate::connectors::registry::LLMRegistryEntryConnector;
use tiny_tokio_actor::*;
use std::sync::{Arc, RwLock};


use crate::error::PantryError;

pub mod registry;
pub mod factory;
pub mod llm_manager;
pub mod llm_actor;

pub mod generic;
pub mod llmrs;
pub mod openai;

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

pub fn get_new_llm_connector(connector_type: LLMConnectorType) -> Box<dyn LLMInternalWrapper> {

    match connector_type {
        LLMConnectorType::GenericAPI => Box::new(generic::GenericAPIConnector{}),
        LLMConnectorType::OpenAI => Box::new(openai::OpenAIConnector{}),
        LLMConnectorType::LLMrs => Box::new(llmrs::LLMrsConnector{})
    }

}

// Conversion from the format used by the index into our internal typing
impl From<LLMRegistryEntryConnector> for LLMConnectorType {
    fn from(value: LLMRegistryEntryConnector) -> Self {
        match value {
            LLMRegistryEntryConnector::GenericAPI => LLMConnectorType::GenericAPI,
            LLMRegistryEntryConnector::Ggml => LLMConnectorType::LLMrs,
            LLMRegistryEntryConnector::OpenAI => LLMConnectorType::OpenAI,
        }
    }

}

/* Actually connect to the LLMs */
pub trait LLMInternalWrapper: Send + Sync{


}

