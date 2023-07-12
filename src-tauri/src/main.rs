// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use crate::connectors::llm_manager;
use crate::llm::LLMWrapper;
use dashmap::DashMap;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::sqlite::Sqlite;
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use frontend::available_llms;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time;
use tauri::{
    window::WindowBuilder, CustomMenuItem, Manager, RunEvent, SystemTray, SystemTrayEvent,
    SystemTrayMenu, WindowEvent, WindowUrl, Wry,
};
use tauri_plugin_store::with_store;
use tauri_plugin_store::StoreCollection;
use tiny_tokio_actor::*;
use tokio;
use uuid::Uuid;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tauri::api::path::app_data_dir;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

mod connectors;
mod database;
mod database_types;
mod emitter;
mod error;
mod frontend;
mod listeners;
mod llm;
mod registry;
mod request;
mod schema;
mod server;
mod state;
mod user;

pub fn get_connection_pool() -> Pool<ConnectionManager<SqliteConnection>> {
    // let url = database_url_for_env();
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    // Refer to the `r2d2` documentation for more methods to use
    // when building a connection pool
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}

// pub fn establish_connection() -> SqliteConnection {
//     SqliteConnection::establish(&database_url)
//         .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
// }
fn run_migrations(
    connection: &mut impl MigrationHarness<Sqlite>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.
    connection.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}

#[tokio::main]
async fn main() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let tray_menu = SystemTrayMenu::new().add_item(CustomMenuItem::new("toggle", "Toggle"));

    let bus = EventBus::<connectors::SysEvent>::new(1000);

    let system = ActorSystem::new("pantry", bus);

    let mut pool = get_connection_pool();

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

        // Load user settings, then return running_llms.

        // Load available LLMs

        let mut path = app
            .path_resolver()
            .app_local_data_dir()
            .ok_or("no path no pantry")?;

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
            pool,
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
        ]);

    // build_server()

    app.run(context)
        .expect("error while running tauri application");
}
