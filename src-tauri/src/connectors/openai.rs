use crate::connectors::LLMInternalWrapper;
use std::path::PathBuf;
use tokio::sync::mpsc;
use serde_json::Value;
use std::collections::HashMap;
use llm::VocabularySource;
use crate::connectors::LLMEvent;


pub struct OpenAIConnector {
    config: HashMap<String, Value>

}

impl OpenAIConnector {
    pub fn new(config: HashMap<String, Value>) -> OpenAIConnector {
       OpenAIConnector {
            config: config,
        }
    }
}

impl LLMInternalWrapper for OpenAIConnector {
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
        // We don't natively support vocabulary, but if you want to set some yourself...

        todo!()
    }
    fn unload_llm(self: &Self, ) -> Result<(), String> {
        todo!()
    }//called by shutdown
}
