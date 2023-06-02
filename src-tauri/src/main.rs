// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::collections::HashMap;
use std::thread;
use std::time;
use tokio;
use tiny_tokio_actor::*;
use std::sync::{Arc, RwLock};
use dashmap::{DashMap, DashSet};
use frontend::available_llms;
use crate::connectors::registry;//::LLMRegistryEntry;
use crate::connectors::llm_manager;
use crate::llm::LLMWrapper;

use tauri::api::path::app_data_dir;


use tauri::{
  window::WindowBuilder, CustomMenuItem, Manager,
  RunEvent, SystemTray, SystemTrayEvent, SystemTrayMenu, WindowEvent, WindowUrl, Wry
};


use tauri_plugin_store::with_store;
use tauri_plugin_store::StoreCollection;
use std::path::PathBuf;
use serde::Serialize;


mod llm;
mod frontend;
mod state;
mod connectors;
mod error;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tokio::main]
async fn main() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());



    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("toggle", "Toggle"));


    let bus = EventBus::<connectors::SysEvent>::new(1000);

    let system = ActorSystem::new("pantry", bus);

    let man_act = connectors::llm_manager::LLMManagerActor {
        llms: HashMap::new()
    };

    let manager_addr: ActorRef<connectors::SysEvent, llm_manager::LLMManagerActor> = system.create_actor("llm_manager", man_act).await.unwrap();



    let builder = tauri::Builder::default()

        .setup(|app| {

            // Load up the state
            let state: tauri::State<state::GlobalState> = app.state();

            let stores = app.state::<StoreCollection<Wry>>();
            let path = PathBuf::from(".settings.dat");

            // Load user settings, then return running_llms.
            let running_llms_vec:Result<Vec<String>, _>  = with_store(app.handle(), stores, path, |store| {
                // let user_settings_json:Option<Arc> =
                match store.get("userSettings") {
                    Some(val) => {
                        println!("Found user settings");
                        match serde_json::from_value(val.to_owned()) {
                            Ok(value) => {
                                println!("Found user settings and deserialized");
                                let mut inner = state.user_settings.write().unwrap();
                                *inner = value;
                            },
                            Err(_) => {
                                println!("Deserialization error, using empty settings.");
                                let mut inner = state.user_settings.write().unwrap();
                                *inner = state::UserSettings {} ;
                            }
                        }
                    },
                    None => {
                        println!("No settings found, using empty settings");
                        let mut inner = state.user_settings.write().unwrap();
                        *inner = state::UserSettings {} ;
                    }
                };

                // let running_llms_json:Vec<LLM> =
                match store.get("running_llms") {
                    Some(val2) => {
                        println!("Found running_llms attempting to deserialize");
                        match serde_json::from_value(val2.to_owned()) {
                            Err(_) => Ok(Vec::new()),
                            Ok(value) => Ok(value)
                        }
                    },
                    None => Ok(Vec::new())
                }
            });

            // Load available LLMs

            let path = app.path_resolver()
                .app_local_data_dir();
            match path {
                Some(mut p) => {
                    p.push("llm_available.dat");
                    match llm::deserialize_llms(p) {
                        Ok(llms) => {
                            println!("Found llm_available.dat, loading");
                            llms.into_iter()
                                .map(|val| state.available_llms.insert(val.id.clone(), Arc::new(val))).for_each(drop);
                        },
                        Err(_) => {
                            println!("Didn't find llm_available.dat, using factory config");
                            connectors::factory::factory_llms().into_iter()
                                .map(|val| state.available_llms.insert(val.id.clone(), Arc::new(val))).for_each(drop); }
                    }
                },
                None => {
                    //We can't find a path?
                    println!("Didn't find app_local_data_dir, panicking.");
                    panic!("Can't find data path")
                }
            }

            // match running_llms_json {
            //     Ok(val) => {
            //         match serde_json::from_value(*val) {
            //             Ok(value) => {running_llms = value;}
            //             Err(_) => { running_llms = Vec::new() }
            //         }
            //     },
            //     Err(_) => {state.user_settings = None}
            // }

            // If the LLM exists and is available we boot it up!
            if running_llms_vec.is_ok() {
                println!("processing running LLMs");
                for val in running_llms_vec.unwrap().into_iter() {
                    if let Some(new_llm) = state.available_llms.get(&val) {
                        // new_llm.load();
                        println!("Inserting {val} into running LLMs");
                        state.running_llms.insert(val);
                    }
                }
            }

            Ok(())
        });



    let app = builder
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(state::create_global_state(manager_addr, DashSet::new(), DashMap::new()))
        .invoke_handler(tauri::generate_handler![
            frontend::get_requests,
            frontend::active_llms,
            frontend::available_llms,
            frontend::ping,
            frontend::load_llm,
            // frontend::unload_llm,
            // frontend::download_llm
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

}

