use chrono::DateTime;
use chrono::Utc;
use tauri::Manager;
use chrono::serde::ts_seconds_option;
use crate::connectors::llm_manager;
use crate::connectors::registry::LLMRegistryEntry;
use crate::error::PantryError;
use crate::llm;
use crate::llm::LLMWrapper;
use crate::connectors::registry;
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
    pub name: String,
    pub description: String,
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

impl From<&llm::LLM> for LLMAvailable {
    fn from(value: &llm::LLM) -> Self {
        let datetime: Option<DateTime<Utc>> = match value.last_called.read() {
            Ok(value) => value.clone(),
            Err(_) => None
        };
        LLMAvailable {
            llm_info: LLMInfo {
                id: value.id.clone(),
                name: value.name.clone(),
                description: value.description.clone()
            },
            downloaded: value.downloaded_reason.clone(),
            last_called: datetime,
        }
    }

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



#[tauri::command]
pub async fn get_requests(state: tauri::State<'_, state::GlobalState>) -> Result<CommandResponse<Vec<LLMRequest>>, String> {
    // let requests = state.get_requests().await;
    println!("received command get_reqs");
    let mock_llm =  LLMInfo {
        id: "llm_id".into(),
        name: "llmname".into(),
        description: "I'm a little llm, short and stout!".into(),
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
    let mock_llm =  LLMInfo {
        id: "llm_id".into(),
        name: "llmname".into(),
        description: "I'm a little llm, short and stout!".into(),
    };
    let mock = LLMRunning {
        llm_info: mock_llm,
        last_called: Option::Some(Utc::now()),
        downloaded: "dowwwn".into(),
        activated: "activvvvvv".into(),
    };
    active_llms.push(mock);
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
    let mock_llm =  LLMInfo {
        id: "llm_id".into(),
        name: "llmname".into(),
        description: "I'm a little llm, short and stout!".into(),
    };
    let mock = LLMAvailable {
        llm_info: mock_llm,
        last_called: Option::Some(Utc::now()),
        downloaded: "dowwwn".into(),
    };
    available_llms.push(mock);
    println!("responding");
    Ok(CommandResponse { data: available_llms })
}


#[tauri::command]
fn download_llm(llm_reg: LLMRegistryEntry, app: tauri::AppHandle, state: tauri::State<'_, state::GlobalState>) -> Result<CommandResponse<Vec<LLMAvailable>>, String> {
    let available_llms_iter = state.available_llms.iter();
    let mut available_llms = Vec::new();


    //Honestly idk wtf this code is even doing. It's definitely not downloading an LLM.
    let path = app.path_resolver().app_local_data_dir();
    match path {
        Some(pp) => {
            let mut p = pp.to_owned();
            p.push("llm_available.dat");

            let llm_iter = state.available_llms.iter();
            let llm_vec: Vec<llm::LLM> = llm_iter.map(|val| (**(val.value())).clone()).collect();

            llm::serialize_llms(p, &llm_vec);
            Ok(CommandResponse { data: available_llms })
        }
        None => {
            Err("Unable to save LLMs".into())
        }
    }
}

// This command refreshes the registry entries stored in state
#[tauri::command]
fn refresh_settings(app: tauri::AppHandle, stores: tauri::State<StoreCollection<Wry>>) -> Result<(), String>{

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
    match state.manager_addr.ask(llm_manager::Ping{}).await {
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
pub async fn load_llm(id: String, state: tauri::State<'_, state::GlobalState>) -> Result<(), String> {
    println!("Attempting to load an LLM");
    if (state.activated_llms.contains_key(&id)) {
        return Err("llm already loaded".into());
    }

    if let Some(llm) = state.available_llms.get(&id) {
        match llm::LLMActivated::activate_llm(llm.value().clone(), state.manager_addr.clone()).await {
            Ok(llm_activ) => {
                match llm_activ.load().await {
                    Ok(_) => {
                        state.activated_llms.insert(llm_activ.llm.id.clone(), llm_activ);
                        Ok(())
                    },
                    Err(err) => {
                        //For now we insert anyway and let the user call reload.
                        state.activated_llms.insert(llm_activ.llm.id.clone(), llm_activ);
                        Err(format!("LLM Actor Loaded, but wrapper failed to launch. You might want to try calling reload LLM. Failed with {}", err.to_string()))
                    }
                }
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

// #[tauri::command]
// pub async fn download_llm(id: String, state: tauri::State<'_, GlobalState>) -> Result<(), String> {
//     state.download_llm(id).await
// }

// #[tauri::command]
// pub async fn download_llm(id: String, state: tauri::State<'_, GlobalState>) -> Result<(), String> {
//     state.download_llm(id).await
// }


