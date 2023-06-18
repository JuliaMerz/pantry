use chrono::DateTime;
use serde_json::Value;
use std::collections::HashMap;
use chrono::Utc;
use tauri::Manager;
use uuid::Uuid;
use chrono::serde::ts_seconds_option;
use crate::connectors;
use crate::connectors::LLMEvent;
use crate::connectors::llm_manager;
use crate::error::PantryError;
use crate::llm;
use crate::emitter;
use crate::llm::LLM;
use crate::llm::LLMWrapper;
use crate::registry;
use crate::state;
use std::{sync::{Arc, RwLock}, ops::Deref};
use serde_json::json;
use tauri::{AppHandle, Wry};
use tauri_plugin_store::with_store;
use tauri_plugin_store::StoreCollection;
use dashmap::DashMap;
use std::path::PathBuf;
use keyring::Entry;



//
// These types are just for API interfacing.
//

#[derive(serde::Serialize)]
pub struct CommandResponse<T> {
    data: T,
}

#[derive(serde::Serialize)]
pub struct LLMInfo {
    pub id: String,
    pub family_id: String,
    pub organization: String,
    pub name: String,
    pub description: String,
    pub user_parameters: Vec<String>, //User Parameters

    //These aren't _useful_ to the user, but we include them for advanced users
    //to get details.
    pub parameters: HashMap<String, Value>, // Hardcoded Parameters

    // 0 is not capable, -1 is not evaluated.
    pub capabilities: HashMap<String, isize>,

    pub connector_type: String,
    pub config: HashMap<String, Value>, // Connector Configs Parameters
}


#[derive(serde::Serialize)]
pub struct LLMRunning {
    pub llm_info: LLMInfo,
    pub downloaded: String,
    #[serde(with = "ts_seconds_option")]
    pub last_called: Option<DateTime<Utc>>,
    pub activated: String,
    // #[serde(skip_serializing)]
    // pub llm: dyn LLMWrapper + Send + Sync

}

#[derive(serde::Serialize)]
pub struct LLMAvailable {
    pub llm_info: LLMInfo,
    pub downloaded: String,
    #[serde(with = "ts_seconds_option")]
    pub last_called: Option<DateTime<Utc>>,
}


#[derive(serde::Serialize)]
pub struct LLMRequest {
    pub llm_info: LLMInfo,
    pub source: String, //For compatibility with the string based enum in typescript
    pub requester: String,
}

#[derive(serde::Serialize)]
pub struct LLMStatus {
    pub status: String
}

// so far, we allow three conversions:
// From LLM to LLMInfo
//
// From LLMActivated to LLMRunning
// From LLM to LLMAvailable

impl From<&llm::LLM> for LLMInfo {
    fn from(value: &llm::LLM) -> Self {
        LLMInfo {
                id: value.id.clone(),
                family_id: value.family_id.clone(),
                organization: value.organization.clone(),
                name: value.name.clone(),
                description: value.description.clone(),
                parameters: value.parameters.clone(),
                user_parameters: value.user_parameters.clone(),

                capabilities: value.capabilities.clone(),



                connector_type: value.connector_type.to_string(),
                config: value.config.clone()
            }
    }
}


impl From<&llm::LLMActivated> for LLMRunning {
    fn from(value: &llm::LLMActivated) -> Self {
        LLMRunning {
            llm_info: value.llm.as_ref().into(),
            downloaded: format!("Downloaded {} for {}", value.llm.downloaded_date.format("%b %e %T %Y"), value.llm.downloaded_reason),
            last_called: value.llm.last_called.read().unwrap().clone(),
            activated: format!("Activated {} for {}", value.activated_time.format("%b %e %T %Y"), value.activated_reason)
        }
    }

}

impl From<&llm::LLM> for LLMAvailable {
    fn from(value: &llm::LLM) -> Self {
        let datetime: Option<DateTime<Utc>> = match value.last_called.read() {
            Ok(value) => value.clone(),
            Err(_) => None
        };
        LLMAvailable {
            llm_info: value.into() ,
            downloaded: value.downloaded_reason.clone(),
            last_called: datetime,
        }
    }

}



#[tauri::command]
pub async fn get_requests(state: tauri::State<'_, state::GlobalState>) -> Result<CommandResponse<Vec<LLMRequest>>, String> {
    // let requests = state.get_requests().await;
    println!("received command get_reqs");
    let mock_llm =  LLMInfo {
        id: "llm_id".into(),
        family_id: "family_id".into(),
        organization: "openai".into(),
        name: "llmname".into(),
        description: "I'm a little llm, short and stout!".into(),
        parameters: HashMap::from([("color".into(), "green".into())]),
        user_parameters: vec!["shape".into()],
        connector_type: connectors::LLMConnectorType::GenericAPI.to_string(),
        capabilities: HashMap::from([("TEXT_COMPLETION".into(), 10), ("CONVERSATION".into(), 10)]),
        config: HashMap::from([]),
    };
    let mock = LLMRequest {
        llm_info: mock_llm,
        source: "mock".into(),
        requester: "fake".into()
    };
    Ok(CommandResponse { data: vec![mock]})
    // Err("boop".into())
}

#[tauri::command]
pub async fn active_llms(state: tauri::State<'_, state::GlobalState>) -> Result<CommandResponse<Vec<LLMRunning>>, String> {
    let active_llms_iter = state.activated_llms.iter();
    println!("received command active_llms");
    let mut active_llms: Vec<LLMRunning> = Vec::new();
    for pair in active_llms_iter {
        println!("attempting to add an active");
        let llm = pair.value();
        active_llms.push(llm.into_llm_running());
    }
    Ok(CommandResponse { data: active_llms })
}

#[tauri::command]
pub async fn available_llms(state: tauri::State<'_, state::GlobalState>) -> Result<CommandResponse<Vec<LLMAvailable>>, String> {
    println!("received command available_llms");
    let available_llms_iter = state.available_llms.iter();
    let mut available_llms: Vec<LLMAvailable> = Vec::new();
    for val in available_llms_iter {
        available_llms.push(val.value().clone().as_ref().into())
    }
    println!("responding");
    Ok(CommandResponse { data: available_llms })
}


#[tauri::command]
pub fn download_llm(llm_reg: registry::LLMRegistryEntry, app: tauri::AppHandle, state: tauri::State<'_, state::GlobalState>) -> Result<CommandResponse<String>, String> {

    let uuid = Uuid::new_v4();

    let id = llm_reg.id.clone();

    tokio::spawn(async move {
      registry::download_and_write_llm(llm_reg, uuid, app.clone()).await;
    });
    // Here we need to download llm_reg.url

    //Honestly idk wtf this code is even doing. It's definitely not downloading an LLM.
    Ok(CommandResponse { data: format!("{}-{}", id, uuid)})
}

// This command refreshes the registry entries stored in state
#[tauri::command]
pub fn refresh_settings(app: tauri::AppHandle, stores: tauri::State<StoreCollection<Wry>>) -> Result<(), String>{

    let path = PathBuf::from(".settings.dat");

    // We need to do some voodoo to compensate for the fact that with_store
    // does not allow for custom errors, which includes errors for missing
    // keys.
    let key_found = false;

    let path = PathBuf::from(".settings.dat");
    with_store(app.clone(), stores, path, |store| {
        // let user_settings_json:Option<Arc> =
        let state: tauri::State<state::GlobalState> = app.state();
        match store.get("userSettings") {
            Some(val) => {
                match serde_json::from_value(val.to_owned()) {
                    Ok(value) => {
                        let mut inner = state.user_settings.write().unwrap();
                        *inner = value;
                    },
                    Err(_) => {
                        let mut inner = state.user_settings.write().unwrap();
                        *inner = state::UserSettings {} ;
                    }
                }
            },
            None => {
                let mut inner = state.user_settings.write().unwrap();
                *inner = state::UserSettings {} ;
            }
        };

        Ok(())
    });
    if key_found{
        Ok(())
    } else {
        Err("User Settings Refresh Failed".into())
    }
}
#[tauri::command]
fn save_key(key_id: String, key_value: String, app: tauri::AppHandle, stores: tauri::State<StoreCollection<Wry>>) -> Result<(), String>{

    let path = PathBuf::from(".settings.dat");


    // TODO: Think about key safety to give custom API's the option of keys
    let entry = Entry::new("pantry-llm", &key_id);
    let entr = entry.map_err(|err| err.to_string())?;
    entr.set_password(&key_value).map_err(|err| err.to_string())?;
    Ok(())

    // match entry {
    //     Ok(entr) => {
    //         match entr.set_password(&key_value) {
    //             Ok(_) => Ok(()),
    //             Err(err) => Err(err.to_string())
    //         }
    //     }
    //     Err(err) => Err(err.to_string())
    // }

    // with_store(app, stores, path, |store| {



    // });
    // Ok(())
}

#[tauri::command]
pub async fn ping(state: tauri::State<'_, state::GlobalState>) -> Result<Vec<String>, String> {
    match state.manager_addr.ask(llm_manager::PingMessage{}).await {
        Ok(val) => match val {
            Ok(va) => {
                println!("pingok");
                Ok(va)
            },
            Err(ma_err) => Err(ma_err.to_string())
        },
        Err(ma_err) => Err(ma_err.to_string())
    }
}


#[tauri::command]
pub async fn load_llm(id: String, app: tauri::AppHandle, state: tauri::State<'_, state::GlobalState>) -> Result<(), String> {
    println!("Attempting to load an LLM");
    if (state.activated_llms.contains_key(&id)) {
        return Err("llm already loaded".into());
    }

    let manager_addr_copy = state.manager_addr.clone();

    if let Some(new_llm) = state.available_llms.get(&id) {
        let result = llm::LLMActivated::activate_llm(new_llm.value().clone(), manager_addr_copy).await;
        // new_llm.load();
        match result {
            Ok(running) => {
                println!("Inserting {id} into running LLMs");
                state.activated_llms.insert(id, running);
                Ok(())
            },
            Err(err) => {
                Err("failed to launch {id} skipping".into())
            }
        }
    } else {
        Err("couldn't find matching llm".into())

    }

    //if let Some(llm) = state.available_llms.get(&id) {
    //    match llm::LLMActivated::activate_llm(llm.value().clone(), state.manager_addr.clone()).await {
    //        Ok(llm_activ) => {
    //            match llm_activ.load().await {
    //                Ok(_) => {
    //                    state.activated_llms.insert(llm_activ.llm.id.clone(), llm_activ);
    //                    Ok(())
    //                },
    //                Err(err) => {
    //                    //For now we insert anyway and let the user call reload.
    //                    state.activated_llms.insert(llm_activ.llm.id.clone(), llm_activ);
    //                    Err(format!("LLM Actor Loaded, but wrapper failed to launch. You might want to try calling reload LLM. Failed with {}", err.to_string()))
    //                }
    //            }
    //        },
    //        Err(err) => Err(err.to_string())
    //    }
    //} else {
    //    Err("Couldn't find LLM".into())
    //}
}


#[derive(serde::Serialize)]
pub struct CallLLMResponse {
    pub session_id: String,
    pub parameters: HashMap<String, Value>,
    pub llm_info: LLMInfo,
}


#[tauri::command]
pub async fn call_llm(id: String, message: String, user_parameters: HashMap<String, Value>, app: AppHandle, state: tauri::State<'_, state::GlobalState>) -> Result<CommandResponse<CallLLMResponse>, String> {
    println!("frontend called {} with {} and params {:?}", id, message, user_parameters);
    if let Some(llm) = state.activated_llms.get(&id) {
        let uuid = Uuid::new_v4();
        match llm.value().call_llm(&message, user_parameters).await {
            Ok(llm_resp) => {

                    tokio::spawn(async move {
                        emitter::send_events("llm_response".into(), uuid.to_string(), llm_resp.stream.unwrap(), app, |stream_id, blah| {
                            let event: emitter::EventType  = match blah {
                                connectors::LLMEvent::PromptProgress { previous, next } => {
                                    Ok(emitter::EventType::PromptProgress {
                                        previous: previous,
                                        next: next
                                    })
                                },
                                connectors::LLMEvent::PromptCompletion { previous } => {
                                    Ok(emitter::EventType::PromptCompletion {
                                        previous: previous,
                                    })
                                },
                                connectors::LLMEvent::PromptError { message } => {
                                    Ok(emitter::EventType::PromptError {
                                        message: message
                                    })
                                }

                                other => {
                                    Err("invalid event type")
                                }

                            }?;

                            Ok(emitter::EventPayload {
                                stream_id: stream_id,
                                event: event,
                            })

                        })
                    });

                    Ok(CommandResponse {
                    data: CallLLMResponse {
                        session_id: uuid.to_string(),
                        parameters: llm_resp.parameters,
                        llm_info: llm.llm.as_ref().into()
                    }})
            },
            Err(err) => Err(err.to_string())
        }
    } else {
        Err("Couldn't find LLM".into())
    }
}
// #[tauri::command]
// pub async fn unload_llm(id: String, state: tauri::State<'_, GlobalState>) -> Result<(), String> {
//     state.unload_llm(id).await
// }



