use chrono::DateTime;
use serde_json::Value;
use std::collections::HashMap;
use chrono::Utc;
use crate::user;
use tauri::Manager;
use uuid::{Uuid, uuid};
use chrono::serde::ts_seconds_option;
use crate::connectors;
use crate::connectors::llm_manager;
use crate::llm;
use crate::emitter;
use crate::llm::LLMWrapper;
use crate::registry;
use crate::state;
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

#[derive(serde::Serialize, Debug)]
pub struct LLMInfo {
    pub id: String,
    pub family_id: String,
    pub organization: String,

    pub name: String,
    pub homepage: String,
    pub license: String,
    pub description: String,

    // 0 is not capable, -1 is not evaluated.
    pub capabilities: HashMap<String, isize>,
    pub requirements: String,
    pub tags: Vec<String>,

    pub url: String,

    pub create_thread: bool,
    pub connector_type: String,
    pub config: HashMap<String, Value>, // Connector Configs Parameters

    //These aren't _useful_ to the user, but we include them for advanced users
    //to get details.
    pub parameters: HashMap<String, Value>, // Hardcoded Parameters
    pub user_parameters: Vec<String>, //User Parameters
    pub session_parameters: HashMap<String, Value>, // Hardcoded Parameters
    pub user_session_parameters: Vec<String>, //User Parameters


}


#[derive(serde::Serialize, Debug)]
pub struct LLMRunning {
    pub llm_info: LLMInfo,
    pub download_reason: String,
    pub downloaded_date: DateTime<Utc>,
    #[serde(with = "ts_seconds_option")]
    pub last_called: Option<DateTime<Utc>>,
    pub activated: String,
    pub uuid: String,
    // #[serde(skip_serializing)]
    // pub llm: dyn LLMWrapper + Send + Sync

}

#[derive(serde::Serialize, Debug)]
pub struct LLMAvailable {
    pub llm_info: LLMInfo,
    pub downloaded: String,
    #[serde(with = "ts_seconds_option")]
    pub last_called: Option<DateTime<Utc>>,
    pub uuid: String,
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
                session_parameters: value.session_parameters.clone(),
                user_session_parameters: value.user_session_parameters.clone(),

                capabilities: value.capabilities.clone(),
                homepage: value.homepage.clone(),
                license: value.license.clone(),
                requirements: value.requirements.clone(),
                url: value.url.clone(),
                tags: value.tags.clone(),
                create_thread: value.create_thread.clone(),



                connector_type: value.connector_type.to_string(),
                config: value.config.clone()
            }
    }
}


impl From<&llm::LLMActivated> for LLMRunning {
    fn from(value: &llm::LLMActivated) -> Self {
        LLMRunning {
            llm_info: value.llm.as_ref().into(),
            download_reason: format!("Downloaded {} for {}", value.llm.downloaded_date.format("%b %e %T %Y"), value.llm.downloaded_reason),
            downloaded_date: value.llm.downloaded_date,
            last_called: value.llm.last_called.read().unwrap().clone(),
            activated: format!("Activated {} for {}", value.activated_time.format("%b %e %T %Y"), value.activated_reason),
            uuid: value.llm.as_ref().uuid.to_string(),
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
            uuid: value.uuid.to_string(),
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
        session_parameters: HashMap::from([("session".into(), "red".into())]),
        user_session_parameters: vec!["session".into()],
        connector_type: connectors::LLMConnectorType::GenericAPI.to_string(),
        capabilities: HashMap::from([("TEXT_COMPLETION".into(), 10), ("CONVERSATION".into(), 10)]),
        config: HashMap::from([]),
        url: "".into(),
        homepage: "https://platform.openai.com/docs/introduction".into(),
        license: "commercial".into(),
        tags: vec!["test".into(), "request".into()],
        create_thread: false,
        requirements: "openai api key".into(),
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
    println!("responding {:?}", available_llms);
    Ok(CommandResponse { data: available_llms })
}

#[derive(serde::Serialize)]
pub struct DownloadResponse {
    pub uuid: String,
    pub stream: String
}


#[tauri::command]
pub fn download_llm(llm_reg: registry::LLMRegistryEntry, app: tauri::AppHandle, state: tauri::State<'_, state::GlobalState>) -> Result<CommandResponse<DownloadResponse>, String> {

    let uuid = Uuid::new_v4();

    let id = llm_reg.id.clone();

    tokio::spawn(async move {
      registry::download_and_write_llm(llm_reg, uuid, app.clone()).await;
    });
    // Here we need to download llm_reg.url

    //Honestly idk wtf this code is even doing. It's definitely not downloading an LLM.
    Ok(CommandResponse { data: DownloadResponse {uuid: uuid.to_string(), stream: format!("{}-{}", id, uuid)}})
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
pub async fn load_llm(uuid: String, app: tauri::AppHandle, state: tauri::State<'_, state::GlobalState>) -> Result<(), String> {
    // let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let uuid = Uuid::parse_str(&uuid).map_err(|e| e.to_string())?;
    println!("Attempting to load an LLM");
    if (state.activated_llms.contains_key(&uuid)) {
        return Err("llm already loaded".into());
    }

    let manager_addr_copy = state.manager_addr.clone();

    if let Some(new_llm) = state.available_llms.get(&uuid) {
        let path = app.path_resolver().app_local_data_dir().ok_or("no path no llms")?;
        let result = llm::LLMActivated::activate_llm(new_llm.value().clone(), manager_addr_copy, path).await;
        // new_llm.load();
        match result {
            Ok(running) => {
                println!("Inserting {uuid} into running LLMs");
                state.activated_llms.insert(uuid, running);
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


/*
 * For responses, we need to include the LLM info because in a real api
 * the caller might not know which LLM gets triggered.
 * We also include a complete set of parameters, since those also aren't necessarily
 * known.
 *
 * For Session Create/Prompt we include a partial info.
 */

#[derive(Debug, serde::Serialize )]
pub struct CallLLMResponse {
    pub session_id: String,
    pub parameters: HashMap<String, Value>,
    pub llm_info: LLMInfo,
}

// Define the response structure for the prompt_session command.
#[derive(Debug, serde::Serialize )]
pub struct PromptSessionResponse {
    pub llm_info: LLMInfo,  // assuming you have defined LLMInfo elsewhere.
}

#[derive(Debug, serde::Serialize )]
pub struct CreateSessionResponse {
    pub session_parameters: HashMap<String, Value>,
    pub llm_info: LLMInfo,
    pub session_id: String,
}


#[tauri::command]
pub async fn get_sessions(llm_uuid: String, app: AppHandle, state: tauri::State<'_, state::GlobalState>) -> Result<CommandResponse<Vec<llm::LLMSession>>, String> {
    let uuid = Uuid::parse_str(&llm_uuid).map_err(|e| e.to_string())?;
    println!("Frontend called get_sessions with LLM UUID {:?} and user {:?}", llm_uuid, user::get_local_user());
    if let Some(llm) = state.activated_llms.get(&uuid) {
        match llm.value().get_sessions(user::get_local_user()).await {
            Ok(sessions) => {
                Ok(CommandResponse {
                    data: sessions,
                })
            },
            Err(err) => Err(err.to_string()),
        }
    } else {
        Err(format!("LLM with UUID {} not found", llm_uuid))
    }
}

#[tauri::command]
pub async fn create_session(llm_uuid: String, user_session_parameters: HashMap<String, Value>, app: AppHandle, state: tauri::State<'_, state::GlobalState>) -> Result<CommandResponse<CreateSessionResponse>, String> {
    println!("Frontend called create_session for {} with parameters {:?} and user {:?}", llm_uuid, user_session_parameters, user::get_local_user());
    let uuid = Uuid::parse_str(&llm_uuid).map_err(|e| e.to_string())?;
    if let Some(llm) = state.activated_llms.get(&uuid) {
        match llm.value().create_session(user_session_parameters, user::get_local_user()).await {
            Ok(resp) => {
                Ok(CommandResponse {
                    data: CreateSessionResponse {
                        session_parameters: resp.session_parameters,
                        session_id: resp.session_id.to_string(),
                        llm_info: llm.llm.as_ref().into()
                    }

                })
            },
            Err(err) => Err(err.to_string()),
        }
    } else {
        Err(format!("LLM with UUID {} not found", uuid))
    }
}

#[tauri::command]
pub async fn prompt_session(llm_uuid: String, session_id: Uuid, prompt: String, parameters:HashMap<String, Value>, app: AppHandle, state: tauri::State<'_, state::GlobalState>) -> Result<CommandResponse<PromptSessionResponse>, String> {
    println!("Frontend called prompt_session with session_id {:?}, prompt {:?}, and user {:?}", session_id, prompt, user::get_local_user());
    let uuid = Uuid::parse_str(&llm_uuid).map_err(|e| e.to_string())?;
    if let Some(llm) = state.activated_llms.get(&uuid) {
        match llm.value().prompt_session(session_id, prompt, parameters, user::get_local_user()).await {
            Ok(prompt_response) => {
                tokio::spawn(async move {
                    emitter::send_events("llm_response".into(), session_id.to_string(), prompt_response.stream, app, |stream_id, event| {
                        let event = emitter::EventType::LLMResponse(event);

                        Ok(emitter::EventPayload {
                            stream_id: stream_id,
                            event: event,
                        })

                    }).await
                });

                Ok(CommandResponse {
                    data: PromptSessionResponse {
                        llm_info: llm.llm.as_ref().into()
                    }
                })
            },
            Err(err) => Err(err.to_string()),
        }
    } else {
        Err(format!("LLM with UUID {} not found", uuid))
    }
}

#[tauri::command]
pub async fn call_llm(llm_uuid: String, prompt: String, user_session_parameters: HashMap<String, Value>, user_parameters: HashMap<String, Value>, app: AppHandle, state: tauri::State<'_, state::GlobalState>) -> Result<CommandResponse<CallLLMResponse>, String> {
    let uuid = Uuid::parse_str(&llm_uuid).map_err(|e| e.to_string())?;
    println!("frontend called {} with {} and params {:?}", uuid, prompt, user_parameters);
    if let Some(llm) = state.activated_llms.get(&uuid) {
        let uuid = Uuid::new_v4();
        match state.manager_addr.ask(llm_manager::PingMessage()).await {
            Ok(result) => println!("ping result: {:?}", result),
            Err(err) => println!("ping error: {:?}", err)
        }

        println!("{:?}", llm.value().ping().await);

        match llm.value().call_llm(&prompt, user_session_parameters, user_parameters, user::get_local_user()).await {
            Ok(llm_resp) => {

                    tokio::spawn(async move {
                        emitter::send_events("llm_response".into(), llm_resp.session_id.to_string(), llm_resp.stream, app, |stream_id, blah| {
                            let event = emitter::EventType::LLMResponse(blah);

                            Ok(emitter::EventPayload {
                                stream_id: stream_id,
                                event: event,
                            })

                        }).await
                    });

                    Ok(CommandResponse {
                    data: CallLLMResponse {
                        session_id: llm_resp.session_id.to_string(),
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
//
#[tauri::command]
pub async fn unload_llm(uuid: String, app: tauri::AppHandle, state: tauri::State<'_, state::GlobalState>) -> Result<(), String> {
    let uuid = Uuid::parse_str(&uuid).map_err(|e| e.to_string())?;
    println!("Attempting to unload an LLM");

    if let Some(running_llm) = state.activated_llms.remove(&uuid) {
        let unload_message = llm_manager::UnloadLLMActorMessage { uuid };
        let manager_addr = state.manager_addr.clone();

        let result = manager_addr.ask(unload_message).await;

        match result {
            Ok(_) => Ok(()),
            Err(_) => Err("Failed to send unload message to LLMManagerActor".into()),
        }
    } else {
        Err("LLM not found or already unloaded".into())
    }
}



#[tauri::command]
pub async fn delete_llm(uuid: String, app: tauri::AppHandle, state: tauri::State<'_, state::GlobalState>) -> Result<(), String> {
    let uuid = Uuid::parse_str(&uuid).map_err(|e| e.to_string())?;
    println!("Attempting to delete an LLM");

    if let Some(running_llm) = state.activated_llms.remove(&uuid) {
        let unload_message = llm_manager::UnloadLLMActorMessage { uuid };
        let manager_addr = state.manager_addr.clone();

        manager_addr.ask(unload_message).await.map_err(|err|format!("Failed to send unload message to LLMManagerActor: {:?}", err))?.map_err(|err| format!("Failed to unload: {:?}", err))?;
    }
    if let Some(llm) = state.available_llms.remove(&uuid) {
        if let Some(model_path) = llm.1.as_ref().model_path.clone() {
            if let Err(err) = std::fs::remove_file(&model_path) {
                return Err(format!("Failed to delete LLM file: {}", err));
            }
        }

        let path = app.path_resolver().app_local_data_dir().ok_or("Failed to get data directory path")?;
        let available_llms_path = path.join("llm_available.dat");

        let llm_iter = state.available_llms.iter();
        let llm_vec: Vec<llm::LLM> = llm_iter.map(|val| (**(val.value())).clone()).collect();

        if let Err(err) = llm::serialize_llms(available_llms_path, &llm_vec) {
            return Err(format!("Failed to serialize available LLMs: {}", err));
        }

        println!("Successfully deleted {} — {}", llm.1.id, llm.1.uuid);

        Ok(())
    } else {
        return Err(format!("Unable to find LLM"))
    }


}
