// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

mod llm;
mod frontend;
mod state;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        .manage(state::GlobalState {
            running_llms: HashMap::new(),
            available_llms: HashMap::new(),

        })
        .invoke_handler(tauri::generate_handler![
            frontend::get_requests,
            frontend::active_llms,
            // frontend::available_llms,
            // frontend::load_llm,
            // frontend::unload_llm,
            // frontend::download_llm
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

