use crate::connectors; //::LLMRegistryEntry;
use crate::connectors::llm_manager;
use crate::error::PantryError;
use crate::frontend::available_llms;
use crate::llm;
use crate::registry; //::LLMRegistryEntry;
use crate::request;
use crate::user;
use dashmap::{DashMap, DashSet};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use keyring;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use tiny_tokio_actor::*;

use uuid::Uuid;

#[derive(Debug)]
pub struct KeychainEntry {
    name: String,
    entry: keyring::Entry,
}

impl Clone for KeychainEntry {
    fn clone(&self) -> Self {
        KeychainEntry::new(&self.name.clone()).unwrap()
    }
}

impl KeychainEntry {
    pub fn new(name: &str) -> Result<Self, String> {
        let entry = keyring::Entry::new("pantry", name)
            .map_err(|err| format!("failed to load keychain: {:?}", err))?;
        Ok(Self {
            name: name.to_string(),
            entry,
        })
    }

    pub fn set_password(&self, password: &str) -> keyring::Result<()> {
        self.entry.set_password(password)
    }

    pub fn get_password(&self) -> keyring::Result<String> {
        self.entry.get_password()
    }
}

impl Serialize for KeychainEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Save the password into keychain, then serialize the name only.
        serializer.serialize_str(&self.name)
    }
}

struct KeychainEntryVisitor;

impl<'de> Visitor<'de> for KeychainEntryVisitor {
    type Value = KeychainEntry;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string representing the name of the key")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        KeychainEntry::new(value).map_err(de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for KeychainEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the name and create an Entry from it.
        deserializer.deserialize_str(KeychainEntryVisitor)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct UserSettings {
    location: PathBuf,
    pub settings_path: PathBuf,
    pub openai_key: KeychainEntry,
    pub use_gpu: bool,
    pub n_thread: usize,
    pub n_batch: usize,
}

impl UserSettings {
    pub fn new(app_location: PathBuf) -> UserSettings {
        let mut location = app_location.clone();
        location.push("user_settings.json");
        if location.exists() {
            match std::fs::read_to_string(&location) {
                Ok(contents) => {
                    // Attempt to deserialize from the existing file.
                    match serde_json::from_str::<UserSettings>(&contents) {
                        Ok(settings) => return settings,
                        Err(_) => {
                            println!("Failed to parse settings file. A new one will be created.")
                        }
                    }
                }
                Err(_) => println!("Failed to read settings file. A new one will be created."),
            }
        }

        // If the file does not exist or reading/parsing it failed, return a new default object.
        UserSettings {
            location,
            settings_path: app_location,
            openai_key: KeychainEntry::new("openai").unwrap(), // replace with actual default
            use_gpu: false,
            n_thread: 4,
            n_batch: 1,
        }
    }
    pub fn save(&self) -> Result<(), String> {
        let serialized = serde_json::to_string(self).map_err(|e| e.to_string())?;
        std::fs::write(&self.location, serialized).map_err(|e| e.to_string())
    }

    pub fn get_location(&self) -> PathBuf {
        self.location.clone()
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserSettingsInfo {
    pub use_gpu: bool,
    pub n_thread: usize,
    pub n_batch: usize,
}

impl From<&UserSettings> for UserSettingsInfo {
    fn from(user_settings: &UserSettings) -> Self {
        UserSettingsInfo {
            use_gpu: user_settings.use_gpu.clone(),
            n_thread: user_settings.n_thread.clone(),
            n_batch: user_settings.n_batch.clone(),
        }
    }
}

#[derive(Clone)]
pub struct GlobalStateWrapper {
    pub state: Arc<GlobalState>,
}

impl Deref for GlobalStateWrapper {
    type Target = GlobalState;

    fn deref(&self) -> &Self::Target {
        &*self.state
    }
}

//TODO: available, registered users, and requests eventually belong in a DB.
pub struct GlobalState {
    pub user_settings: RwLock<UserSettings>,
    pub manager_addr: ActorRef<connectors::SysEvent, llm_manager::LLMManagerActor>,
    // pub running_llms: DashSet<String>,
    pub activated_llms: DashMap<Uuid, llm::LLMActivated>,
    pub available_llms: DashMap<Uuid, Arc<llm::LLM>>,
    pub pool: Pool<ConnectionManager<SqliteConnection>>,
}

/*
 * Functions that modify our global state (activating/deactivating LLMs)
 * */
pub fn create_global_state(
    addr: ActorRef<connectors::SysEvent, llm_manager::LLMManagerActor>,
    activated_llms: DashMap<Uuid, llm::LLMActivated>,
    available_llms: DashMap<Uuid, Arc<llm::LLM>>,
    settings_path: PathBuf,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> GlobalStateWrapper {
    GlobalStateWrapper {
        state: Arc::new(GlobalState {
            manager_addr: addr,
            user_settings: RwLock::new(UserSettings::new(settings_path)), // We initialize user settings after global state
            activated_llms,
            pool: pool,
        }),
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
