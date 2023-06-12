use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::connectors::llm_manager;
use crate::error::PantryError;
use crate::frontend::available_llms;
use crate::connectors::registry;//::LLMRegistryEntry;
use crate::connectors;//::LLMRegistryEntry;
use crate::llm;
use dashmap::{DashMap, DashSet};
use tiny_tokio_actor::*;


// Tauri state
//
// Using this safely:
//
//

// This needs to match frontend's user settings
#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserSettings {
}


pub struct GlobalState {
    pub user_settings: RwLock<UserSettings>,
    pub manager_addr: ActorRef<connectors::SysEvent, llm_manager::LLMManagerActor>,
    // pub running_llms: DashSet<String>,
    pub activated_llms: DashMap<String, llm::LLMActivated>,
    pub available_llms: DashMap<String, Arc<llm::LLM>>,
}

/*
 * Functions that modify our global state (activating/deactivating LLMs)
 * */
pub fn create_global_state(addr: ActorRef<connectors::SysEvent, llm_manager::LLMManagerActor>,
            activated_llms: DashMap<String, llm::LLMActivated>,
            available_llms: DashMap<String, Arc<llm::LLM>>,)
            -> GlobalState {
    GlobalState {
        manager_addr: addr,
        user_settings: RwLock::new(UserSettings {  }), // We initialize user settings after global state
        activated_llms: activated_llms,
        available_llms: available_llms,
        // running_llms: DashMap::new(),
        // available_llms: DashMap::new(),

    }
}

// Write this if we have to save LLMs in a second place.
// pub fn save_available_llms_state(path:PathBuf, available_llms: DashMap<String, Arc<llm::LLM>>) -> Result<(), PantryError>



// pub fn load_llm(id:String, state:GlobalState) -> Result<(), String> {

//     let llm_avail = state.available_llms.get(&id).unwrap().clone();

//     state.running_llms.insert(id);
//     Ok(())
// }

// pub fn unload_llm(id:String, state:GlobalState) -> Result<(), String> {

//     state.running_llms.remove(&id);

//     Ok(())
// }
