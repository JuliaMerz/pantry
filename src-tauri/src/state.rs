use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::llm;

// Tauri state
pub struct GlobalState {
    pub running_llms: HashMap<String, Arc<llm::LLMRunning>>,
    pub available_llms: HashMap<String, Arc<llm::LLMAvailable>>,

    // pub running_llms: Arc<Mutex<HashMap<String, Box<dyn LLMWrapper + Send>>>>,
    // pub all_llms: Arc<Mutex<HashMap<String, LLMActive>>>,
}
