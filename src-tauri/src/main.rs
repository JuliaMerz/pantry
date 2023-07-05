// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use crate::connectors::llm_manager;
use crate::llm::LLMWrapper;
use dashmap::DashMap;
use frontend::available_llms;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time;
use tiny_tokio_actor::*;
use tokio;
use uuid::Uuid;

use tauri::api::path::app_data_dir;

use tauri::{
    window::WindowBuilder, CustomMenuItem, Manager, RunEvent, SystemTray, SystemTrayEvent,
    SystemTrayMenu, WindowEvent, WindowUrl, Wry,
};

use serde::Serialize;
use std::path::PathBuf;
use tauri_plugin_store::with_store;
use tauri_plugin_store::StoreCollection;

mod connectors;
mod emitter;
mod error;
mod frontend;
mod listeners;
mod llm;
mod registry;
mod server;
mod state;
mod user;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tokio::main]
async fn main() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let tray_menu = SystemTrayMenu::new().add_item(CustomMenuItem::new("toggle", "Toggle"));

    let bus = EventBus::<connectors::SysEvent>::new(1000);

    let system = ActorSystem::new("pantry", bus);

    let man_act = connectors::llm_manager::LLMManagerActor {
        active_llm_actors: HashMap::new(),
    };

    let manager_addr: ActorRef<connectors::SysEvent, llm_manager::LLMManagerActor> =
        system.create_actor("llm_manager", man_act).await.unwrap();

    // Listen for events on the system event bus
    let mut events: EventReceiver<connectors::SysEvent> = system.events();
    tokio::spawn(async move {
        println!("listening for sys events");
        loop {
            match events.recv().await {
                Ok(event) => println!("Received sys event! {:?}", event),
                Err(err) => println!("Error receivng sys event!!! {:?}", err),
            }
        }
    });

    let manager_addr_clone = manager_addr.clone();

    let builder = tauri::Builder::default().setup(move |app| {
        #[cfg(debug_assertions)] // only include this code on debug builds
        {
            let window = app.get_window("main").unwrap();
            window.open_devtools();
            window.close_devtools();
        }

        // Load up the state
        let state: tauri::State<state::GlobalStateWrapper> = app.state();

        let stores = app.state::<StoreCollection<Wry>>();
        let path = PathBuf::from(".settings.dat");

        // Load user settings, then return running_llms.
        let running_llms_vec: Result<Vec<Uuid>, _> =
            with_store(app.handle(), stores, path, |store| {
                // let user_settings_json:Option<Arc> =

                // let running_llms_json:Vec<LLM> =
                match store.get("active_llms") {
                    Some(val2) => {
                        println!("Found active_llms attempting to deserialize");
                        match serde_json::from_value(val2.to_owned()) {
                            Err(_) => Ok(Vec::new()),
                            Ok(value) => Ok(value),
                        }
                    }
                    None => Ok(Vec::new()),
                }
            });

        // Load available LLMs

        let mut path = app
            .path_resolver()
            .app_local_data_dir()
            .ok_or("no path no pantry")?;

        path.push("llm_available.dat");
        println!("path used: {:?}", path);
        match llm::deserialize_llms(path.clone()) {
            Ok(llms) => {
                println!("Found llm_available.dat, loading");
                llms.into_iter()
                    .map(|val| state.available_llms.insert(val.uuid.clone(), Arc::new(val)))
                    .for_each(drop);
            }
            Err(err) => {
                println!("Error finding llm, using factory. Err: {:?}", err);
                connectors::factory::factory_llms()
                    .into_iter()
                    .map(|val| state.available_llms.insert(val.uuid.clone(), Arc::new(val)))
                    .for_each(drop);

                // mostly test
                let llm_iter = state.available_llms.iter();

                let llm_vec: Vec<llm::LLM> =
                    llm_iter.map(|val| (**(val.value())).clone()).collect();
                match llm::serialize_llms(path, &llm_vec) {
                    Ok(res) => {
                        println!("serialized successfully! {:?}", res);
                    }
                    Err(err) => {
                        println!("failed serialize test: {:?}", err);
                    }
                }
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
        let new_path = app
            .path_resolver()
            .app_local_data_dir()
            .ok_or("no path no pantry")?;

        // If running LLMs exist, we need to boot them up.
        let app_handle = app.handle();
        if running_llms_vec.is_ok() {
            println!("processing running LLMs");
            tokio::spawn(async move {
                let state_pointer: Arc<tauri::State<state::GlobalStateWrapper>> =
                    Arc::new(app_handle.state());
                for val in running_llms_vec.unwrap().into_iter() {
                    let manager_addr_copy = manager_addr_clone.clone();
                    if let Some(new_llm) = state_pointer.available_llms.get(&val) {
                        let result = llm::LLMActivated::activate_llm(
                            new_llm.value().clone(),
                            manager_addr_copy,
                            new_path.clone(),
                            state::UserSettings::new(
                                app_handle.path_resolver().app_local_data_dir().unwrap(),
                            ),
                        )
                        .await;
                        // new_llm.load();
                        match result {
                            Ok(running) => {
                                println!("Inserting {val} into running LLMs");
                                state_pointer.activated_llms.insert(val, running);
                            }
                            Err(err) => {
                                println!("failed to launch {val} skipping");
                            }
                        }
                    }
                }
            });
        }

        Ok(())
    });

    let context = tauri::generate_context!();

    let app = builder
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(state::create_global_state(
            manager_addr,
            DashMap::new(),
            DashMap::new(),
            tauri::api::path::app_local_data_dir(context.config()).unwrap(),
        ))
        .invoke_handler(tauri::generate_handler![
            frontend::get_requests,
            frontend::active_llms,
            frontend::available_llms,
            frontend::ping,
            frontend::load_llm,
            frontend::call_llm,
            frontend::get_sessions,
            frontend::create_session,
            frontend::prompt_session,
            frontend::unload_llm,
            frontend::download_llm,
            frontend::delete_llm,
            frontend::set_user_setting,
            frontend::get_user_settings,
            frontend::interrupt_session,
        ])
        .run(context)
        .expect("error while running tauri application");
}
