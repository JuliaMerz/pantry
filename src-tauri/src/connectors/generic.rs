use crate::connectors::LLMInternalWrapper;
use tokio::sync::mpsc;
use serde_json::Value;
use std::collections::HashMap;

use crate::connectors::LLMEvent;
pub struct GenericAPIConnector {
    config: HashMap<String, Value>

}

impl GenericAPIConnector {
    pub fn new(config: HashMap<String, Value>) -> GenericAPIConnector {
        GenericAPIConnector {
            config: config,
        }
    }
}

impl LLMInternalWrapper for GenericAPIConnector {
    fn call_llm(self: &Self, msg: String, params: HashMap<String, Value>) -> Result<mpsc::Receiver<LLMEvent>, String> {
        todo!()
    }
    fn create_session(self: &mut Self, params: HashMap<String, Value>) -> Result<String, String> {
        todo!()
    } //uuid
    fn prompt_session(self: &mut Self, session_id: String, msg: String) -> Result<mpsc::Receiver<LLMEvent>, String> {
        todo!()
    }
    fn load_llm(self: &mut Self, ) -> Result<(), String> {
        todo!()
    }
    fn unload_llm(self: &Self, ) -> Result<(), String> {
        todo!()
    }//called by shutdown

}
