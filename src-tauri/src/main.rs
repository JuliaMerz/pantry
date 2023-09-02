// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![recursion_limit = "512"]
use crate::connectors::llm_manager;

use crate::state::KeychainEntry;
use dashmap::DashMap;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::Sqlite;
use diesel::sqlite::SqliteConnection;
use env_logger::Builder;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, error, info, warn, LevelFilter};
use prettytable::{Cell, Row, Table};
use std::sync::{Arc, Mutex};
use std::{thread, time};
use tauri::api::cli::{ArgData, Matches, SubcommandMatches};
use tauri_plugin_deep_link;
use tokio::sync::oneshot;
use url::{ParseError, Url};
use uuid::Uuid;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;

use tauri::{
    CustomMenuItem, Manager, RunEvent, SystemTray, SystemTrayEvent, SystemTrayMenu, WindowEvent,
    Wry,
};

use crate::llm::LLMWrapper;
use tauri_plugin_single_instance;
use tauri_plugin_store::StoreCollection;
use tiny_tokio_actor::*;
use tokio;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

mod cli;
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

#[derive(Debug)]
pub struct ConnectionOptions {
    pub enable_wal: bool,
    pub enable_foreign_keys: bool,
    pub busy_timeout: Option<Duration>,
}

impl diesel::r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error>
    for ConnectionOptions
{
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        (|| {
            if self.enable_wal {
                conn.batch_execute("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
            }
            if self.enable_foreign_keys {
                conn.batch_execute("PRAGMA foreign_keys = ON;")?;
            }
            if let Some(d) = self.busy_timeout {
                conn.batch_execute(&format!("PRAGMA busy_timeout = {};", d.as_millis()))?;
            }
            Ok(())
        })()
        .map_err(diesel::r2d2::Error::QueryError)
    }
}

pub fn get_connection_pool(db_url: String) -> Pool<ConnectionManager<SqliteConnection>> {
    // let url = database_url_for_env();

    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    // Refer to the `r2d2` documentation for more methods to use
    // when building a connection pool
    let pool = Pool::builder()
        .max_size(8)
        .connection_customizer(Box::new(ConnectionOptions {
            enable_wal: true,
            enable_foreign_keys: true,
            busy_timeout: Some(Duration::from_secs(10)),
        }))
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool");
    pool
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
    // debug!!("Mirations:\n{:?}", connection.revert_all_migrations(MIGRATIONS));
    // debug!!("Mirations:\n{:?}", connection.applied_migrations());
    connection.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}

#[derive(Clone, serde::Serialize, Debug)]
#[serde(tag = "type")]
pub enum DeepLinkEventPayload {
    URLError { message: String },
    DownloadEvent { base64: String },
    DebugEvent { debug1: String, debug2: String },
}

#[derive(Clone, serde::Serialize, Debug)]
pub struct DeepLinkEvent {
    pub raw: String,
    pub payload: DeepLinkEventPayload,
}

#[tokio::main]
async fn main() {
    tauri_plugin_deep_link::prepare("com.jmerz.pantry");
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    Builder::new()
        .filter(None, LevelFilter::Info) // Default log level set to `info`
        .init();

    // let _tray_menu = SystemTrayMenu::new().add_item(CustomMenuItem::new("toggle", "Toggle"));

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
        info!("listening for sys events");
        loop {
            match events.recv().await {
                Ok(event) => info!("Received sys event! {:?}", event),
                Err(err) => error!("Error receivng sys event!!! {:?}", err),
            }
        }
    });

    let context = tauri::generate_context!();

    let mut db_path = tauri::api::path::app_local_data_dir(context.config()).unwrap();
    let mut llm_path = tauri::api::path::local_data_dir().unwrap();
    llm_path.push("pantry");

    if !llm_path.exists() {
        fs::create_dir_all(&llm_path).unwrap();
    }

    if !db_path.exists() {
        fs::create_dir_all(&llm_path).unwrap();
    }

    let config = context.config().clone();

    db_path.push("local2.sqlite");

    let _ = diesel::sqlite::SqliteConnection::establish(
        &db_path.clone().into_os_string().into_string().unwrap(),
    );
    // we need to do this to ensure the database exists.

    let pool = get_connection_pool(db_path.into_os_string().into_string().unwrap());
    run_migrations(&mut pool.get().unwrap()).unwrap();

    //channels for shutting down the web servers
    let (server_shutdown_tx, server_shutdown_rx) = oneshot::channel();
    let mut server_shutdown_tx = Arc::new(Mutex::new(Some(server_shutdown_tx)));
    let mut server_shutdown_tx1 = server_shutdown_tx.clone();
    let (server_shutdown_confirm_tx, server_shutdown_confirm_rx) = oneshot::channel();
    let mut server_shutdown_confirm_rx = Arc::new(Mutex::new(Some(server_shutdown_confirm_rx)));
    let mut server_shutdown_confirm_rx1 = server_shutdown_confirm_rx.clone();
    // let server_shutdown_confirm_tx = Option(server_shutdown_confirm_tx)

    // Run early CLI commands—any that need to run before we shut down from dupes.
    let cli_conf = config.tauri.cli.clone().unwrap();
    let package_info = context.package_info();

    cli::cli_command_interpreter(cli_conf, package_info, pool.clone()).await;

    let builder = tauri::Builder::default().setup(move |app| {

        // TODO: break this out
        let handle = app.handle();
        tauri_plugin_deep_link::register(
            "pantry",
            move |request| {
              dbg!(&request);

              let url = Url::parse(&request);
              let payload = match url {
                  Ok(url_thing) => {
                      match url_thing.host_str() {
                          Some("download") => DeepLinkEvent {
                              raw: request,
                              payload: DeepLinkEventPayload::DownloadEvent { base64: url_thing.path()[1..].into() }
                          },
                          Some(other) => DeepLinkEvent {
                              raw: request,
                              payload: DeepLinkEventPayload::DebugEvent { debug1: url_thing.path().into(), debug2: other.into() }
                          },

                          None => DeepLinkEvent {
                              raw: request,
                              payload: DeepLinkEventPayload::URLError { message: "No path".into() }
                          }

                      }

                  }



                  Err(err) => DeepLinkEvent {
                      raw: request,
                      payload: DeepLinkEventPayload::URLError { message: err.to_string() }
                  }
              };


              //request is a string that reprentsnts the WHOLE url, pantry:// included
              //
              handle.emit_all("deep-link-request", payload).unwrap();
            },
          )
          .unwrap(/* If listening to the scheme is optional for your app, you don't want to unwrap here. */);
        #[cfg(debug_assertions)] // only include this code on debug builds
        {
            let window = app.get_window("main").unwrap();
            window.open_devtools();
            window.close_devtools();
        }




        let global_state = state::create_global_state(
            manager_addr,
            DashMap::new(),
            app.handle(),
            tauri::api::path::app_local_data_dir(&config).unwrap(),
            llm_path,
            pool,
        );



        app.manage(global_state.clone());
        tokio::spawn(async move {
            server_shutdown_confirm_tx.send(match server::build_server(global_state, server_shutdown_rx).await {
                Ok(_) => { info!("API server closed with okay.");
                Ok(())},
                Err(err) => {
                    error!("API server failure: {:?}", err);
                    Err(err)
                }
            });
          });


        let app_handle = app.handle();
        // SystemTray::new()
        //   .with_menu(
        //     SystemTrayMenu::new()
        //       .add_item(CustomMenuItem::new("quit", "Quit"))
        //       .add_item(CustomMenuItem::new("open", "Open"))
        //   )
        //   .on_event(move |event| {
        //     let tray_handle = app_handle.tray_handle();
        //   });

        Ok(())
    });

    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("quit", "Quit"))
        .add_item(CustomMenuItem::new("hide", "Hide"))
        .add_item(CustomMenuItem::new("open", "Open"));

    // async fn run_quit(shutdown_tx, confirm_rx) {
    async fn run_quit(
        app: tauri::AppHandle,
        shutdown_tx: oneshot::Sender<()>,
        confirm_rx: oneshot::Receiver<Result<(), String>>,
    ) {
        let stat: tauri::State<state::GlobalStateWrapper> = app.state();
        let manager_addr = stat.manager_addr.clone();
        shutdown_tx.send(());
        let uuids: Vec<Uuid> = stat
            .activated_llms
            .iter()
            .map(|pair| pair.key().clone())
            .collect();
        let mut futs = Vec::new();
        for uuid in uuids {
            futs.push(
                stat.activated_llms
                    .remove(&uuid)
                    .expect("beep")
                    .1
                    .unload_llm(manager_addr.clone()),
            );
        }
        let borrow_man = manager_addr.clone();
        confirm_rx.await;
        join_all(futs).await;
        info!("completed shutdown");
        std::process::exit(0);
    }

    let app = builder
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            debug!("{}, {argv:?}, {cwd}", app.package_info().name);
            cli::main_command_response(argv, app.state());
        }))
        .system_tray(SystemTray::new().with_menu(tray_menu))
        .on_system_tray_event(move |app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    if let Some(send) = server_shutdown_tx.lock().unwrap().take() {
                        let opt = server_shutdown_confirm_rx.lock().unwrap().take();
                        if let Some(recv) = opt {
                            let app_clone = app.clone();
                            tokio::spawn(async move {
                                run_quit(app_clone, send, recv).await;
                            });
                        }
                    }
                }
                "hide" => {
                    let window = app.get_window("main").unwrap();
                    window.hide().unwrap();
                }
                "open" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                }
                _ => {}
            },
            _ => {}
        })
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
            frontend::accept_request,
            frontend::reject_request,
        ]);

    // build_server()

    let app = app
        .build(context)
        .expect("error while running tauri application");
    app.run(move |app_handle, event| match event {
        RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        RunEvent::Exit => {
            if let Some(send) = server_shutdown_tx1.lock().unwrap().take() {
                let opt = server_shutdown_confirm_rx1.lock().unwrap().take();
                if let Some(recv) = opt {
                    let app_clone = app_handle.clone();
                    let join_hand = tokio::spawn(async move {
                        run_quit(app_clone, send, recv).await;
                    });
                    while !join_hand.is_finished() {
                        thread::sleep(time::Duration::from_millis(100));
                    }
                    info!("Ctrl q complete. Bye bye.");
                }
            }
        }
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } => {
            let window = app_handle.get_window("main").unwrap();
            api.prevent_close();
            window.hide().unwrap();
        }
        _ => {}
    });
}
