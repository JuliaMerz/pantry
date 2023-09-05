use crate::database;

use crate::registry::{download_and_write_llm, LLMRegistryEntry};
use crate::state::GlobalStateWrapper;
use crate::state::KeychainEntry;
use crate::user;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;

use log::{error, info, warn};
use pantry_rs::PantryClient;
use prettytable::{row, Table};
use serde_json::{from_value, Value};
use std::env;

use tauri::api::cli::Matches;
use tauri::{AppHandle, PackageInfo, State};

use uuid::uuid;
use uuid::Uuid;

// We currently handle the CLI entirely through the API, so this is a noop.
pub fn main_command_response(_argv: Vec<String>, _state: State<GlobalStateWrapper>) {}

pub async fn cli_command_interpreter(
    app: AppHandle,
    matches: Matches,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<(), String> {
    let local_user = uuid!("00000000-0000-0000-0000-000000000000");
    let cli_user_evn = Uuid::parse_str(&(env::var("PANTRY_CLI_USER").unwrap_or("".into())));
    let cli_key_evn = env::var("PANTRY_CLI_KEY").unwrap_or("".into());

    // Set up local superuser for API integration
    let (cli_user, cli_key) = match (cli_user_evn, cli_key_evn) {
        (Ok(a), b) => (Some(a), Some(b)),
        (_, _) => match KeychainEntry::new("superuser_key") {
            Ok(pw_entry) => match pw_entry.get_password() {
                Ok(pw) => {
                    info!("Loaded superuser PW");
                    (Some(local_user), Some(pw))
                }
                Err(keyring::Error::NoEntry) => {
                    info!("No superuser key detected, generating...");

                    // We use the same local user, but give it an API key in the db.
                    // This has the advantage of giving us an easy way of finding the user
                    // so we only need to store the API key in the keystore.
                    let mut superuser = user::get_local_user();
                    let api_key = user::generate_api_key();
                    superuser.api_key = api_key.clone();
                    match database::save_new_user(superuser, pool.clone()) {
                        Ok(_user) => match pw_entry.set_password(&api_key) {
                            Ok(_) => {
                                info!("Created local superuser");
                                (Some(local_user), Some(api_key))
                            }
                            Err(e) => {
                                error!("E: {:?}", e);
                                error!("Failed to create superuser. CLI will fail.");
                                (None, None)
                            }
                        },
                        Err(e) => {
                            error!("E: {:?}", e);
                            error!("Failed to create superuser. CLI will fail.");
                            (None, None)
                        }
                    }
                }
                Err(e) => {
                    error!("E: {:?}", e);
                    error!("Failed to access secure api_key storage, CLI will fail.");
                    (None, None)
                }
            },
            Err(e) => {
                error!("E: {:?}", e);
                error!("Failed to load keychain, CLI will fail.");
                (None, None)
            }
        },
    };

    if cli_user.is_none() || cli_key.is_none() {
        error!("Unable to detect cli user/password");
        return Err("Unable to detect cli user/password".into());
    }

    let cli_target = match env::var("PANTRY_CLI_TARGET") {
        Ok(t) => Some(t),
        Err(_) => None,
    };

    let pantry_api = PantryClient::login(cli_user.unwrap(), cli_key.unwrap(), cli_target);
    if let Some(help_text) = matches.args.get("help") {
        println!("{}", help_text.value.as_str().unwrap_or(""));
    }

    if let Some(subcommand) = &matches.subcommand {
        match subcommand.name.as_str() {
            "list" => handle_list_subcommand_cli(&subcommand.matches, pool, pantry_api).await,
            "activate" => {
                handle_activate_subcommand_cli(&subcommand.matches, pool, pantry_api).await;
            }
            "deactivate" => {
                handle_deactivate_subcommand_cli(&subcommand.matches, pool, pantry_api).await;
            }
            "path" => {
                handle_path_subcommand_cli(&subcommand.matches, pool, pantry_api).await;
            }
            "download" => {
                match handle_download_subcommand_cli(&subcommand.matches, pool, pantry_api).await {
                    Ok(_) => {}
                    Err(e) => error!("Download failed: {:?}", e),
                }
            }
            "status" => {
                match handle_status_subcommand_cli(&subcommand.matches, pool, pantry_api).await {
                    Ok(_) => {}
                    Err(e) => error!("Status request failed: {:?}", e),
                }
            }
            "new_cli_user" => {
                match handle_new_cli_user_subcommand(&subcommand.matches, pool).await {
                    Ok(_) => {}
                    Err(e) => error!("New user request failed: {:?}", e),
                }
            }
            _ => {
                error!("Unrecognized command");
            }
        }
    } else {
        // app.print_help();
    };
    Ok(())
}

// Stub function to handle the 'list' subcommand
async fn handle_list_subcommand_cli(
    matches: &Matches,
    _pool: Pool<ConnectionManager<SqliteConnection>>,
    client: PantryClient,
) {
    if let Some(help_text) = matches.args.get("help") {
        println!("{}", help_text.value.as_str().unwrap_or(""));
    }
    if let Some(subcommand) = &matches.subcommand {
        match subcommand.name.as_str() {
            "running" => match client.get_running_llms().await {
                Ok(llms) => {
                    let mut table = Table::new();
                    table.add_row(row![b->"UUID", b->"ID", b->"Name"]);
                    for entry in llms.iter() {
                        table.add_row(row![entry.uuid, entry.id, entry.name]);
                    }
                    table.printstd();
                }
                Err(e) => {
                    error!("Failed to get running LLMs: {:?}", e);
                }
            },
            "available" => match client.get_available_llms().await {
                Ok(llms) => {
                    let mut table = Table::new();
                    table.add_row(row![b->"UUID", b->"ID", b->"Name"]);
                    for entry in llms.iter() {
                        table.add_row(row![entry.uuid, entry.id, entry.name]);
                    }
                    table.printstd();
                }
                Err(e) => {
                    error!("Failed to get running LLMs: {:?}", e);
                }
            },
            "downloadable" => match downloadable_llms_default().await {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to get downloadable LLMS: {:?}", e);
                }
            },
            _ => {
                warn!("No subcommand supplied, defaulting to 'running'");
            }
        }
    } else {
        error!("Handling 'list' with default behavior");
    };
}

// Stub function to handle the 'activate' subcommand
async fn handle_deactivate_subcommand_cli(
    matches: &Matches,
    _pool: Pool<ConnectionManager<SqliteConnection>>,
    client: PantryClient,
) {
    if let Some(help_text) = matches.args.get("help") {
        println!("{}", help_text.value.as_str().unwrap_or(""));
    }
    if let Some(arg_data) = matches.args.get("llm_id") {
        if let Value::String(llm_id) = &arg_data.value {
            info!("Handling 'deactivate' with llm_id: {}", llm_id);
            match client.unload_llm(llm_id.clone()).await {
                Ok(_) => {
                    info!("Initiated LLM deactivate");
                }
                Err(e) => {
                    error!("Failed to initate llm deactivate: {:?}", e);
                }
            }
        }
    }
}

async fn downloadable_llms_default() -> Result<(), String> {
    let response =
        // TODO: implement the same registry add functionlity of the UI.
        reqwest::get("https://raw.githubusercontent.com/JuliaMerz/pantry/master/models/index.json")
            .await
            .map_err(|e| format!("{:?}", e))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| format!("{:?}", e))?;

    // Navigate JSON and deserialize to Vec<LLMRegistryEntry>
    if let Some(models) = response.get("models") {
        if let serde_json::Value::Object(map) = models {
            let mut llms: Vec<pantry_rs::interface::LLMRegistryEntry> = Vec::new();
            for (k, v) in map {
                match serde_json::from_value::<pantry_rs::interface::LLMRegistryEntry>(v.clone()) {
                    Ok(entry) => llms.push(entry),
                    Err(e) => error!("Failed to parse {}: {:?}", k, e),
                }
            }

            // Print using a table
            let mut table = Table::new();
            table.add_row(row![b->"ID", b->"Name", b->"Website"]);
            for entry in llms.iter() {
                table.add_row(row![entry.id, entry.name, entry.homepage]);
            }
            table.printstd();
        }
    };
    Ok(())
}

// Stub function to handle the 'deactivate' subcommand
async fn handle_activate_subcommand_cli(
    matches: &Matches,
    _pool: Pool<ConnectionManager<SqliteConnection>>,
    client: PantryClient,
) {
    if let Some(help_text) = matches.args.get("help") {
        println!("{}", help_text.value.as_str().unwrap_or(""));
    }
    if let Some(arg_data) = matches.args.get("llm_id") {
        if let Value::String(llm_id) = &arg_data.value {
            match client.load_llm(llm_id.clone()).await {
                Ok(status) => {
                    info!("Sent activation command for LLM: {}", status.llm_info.name);
                }
                Err(e) => {
                    error!("Failed to activate, due to error: {:?}", e);
                }
            }
        }
    }
}

async fn handle_path_subcommand_cli(
    matches: &Matches,
    _pool: Pool<ConnectionManager<SqliteConnection>>,
    client: PantryClient,
) {
    if let Some(help_text) = matches.args.get("help") {
        println!("{}", help_text.value.as_str().unwrap_or(""));
    }
    if let Some(arg_data) = matches.args.get("llm_id") {
        if let Value::String(llm_id) = &arg_data.value {
            match client
                .bare_model_flex(
                    Some(pantry_rs::LLMFilter {
                        llm_id: Some(llm_id.to_owned()),
                        llm_uuid: None,
                        family_id: None,
                        local: None,
                        minimum_capabilities: None,
                    }),
                    None,
                )
                .await
            {
                Ok(status) => {
                    info!("LLM with bare model! {:?}", status);
                    let mut table = Table::new();
                    table.add_row(row![b->"ID", b->"Name", b->"path"]);
                    table.add_row(row![status.0.id, status.0.name, status.1]);
                    table.printstd();
                }
                Err(e) => {
                    error!("Failed to get bare model path, due to error: {:?}", e);
                }
            }
        }
    }
}

async fn handle_download_subcommand_cli(
    matches: &Matches,
    _pool: Pool<ConnectionManager<SqliteConnection>>,
    client: PantryClient,
) -> Result<(), String> {
    if let Some(help_text) = matches.args.get("help") {
        println!("{}", help_text.value.as_str().unwrap_or(""));
    }
    if let Some(arg_data) = matches.args.get("llm_id") {
        if let Value::String(llm_id) = &arg_data.value {
            let response =
        // TODO: implement the same registry add functionlity of the UI.
        reqwest::get("https://raw.githubusercontent.com/JuliaMerz/pantry/master/models/index.json")
            .await
            .map_err(|e| format!("{:?}", e))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| format!("{:?}", e))?;

            // Navigate JSON and deserialize to Vec<LLMRegistryEntry>
            let models = response.get("models").ok_or("Failed to get registry.")?;
            if let serde_json::Value::Object(map) = models {
                let reg_ent_val = map.get(llm_id).ok_or("Model is not part of the default registry. For non-default registry or manual adds, please use the UI.")?;

                let reg_ent =
                    from_value::<pantry_rs::interface::LLMRegistryEntry>(reg_ent_val.clone())
                        .map_err(|e| format!("Deserialization failure: {:?}", e))?;

                let uuid = client
                    .download_llm(reg_ent)
                    .await
                    .map_err(|e| format!("Error initiating download: {:?}", e))?;
                info!(
                    "Download begun. You can check progress with `pantry status {}`",
                    uuid
                );
            };
        };
    } else {
        error!("llm_id is mandatory");
    }
    Ok(())
}

async fn handle_status_subcommand_cli(
    matches: &Matches,
    _pool: Pool<ConnectionManager<SqliteConnection>>,
    client: PantryClient,
) -> Result<(), String> {
    if let Some(help_text) = matches.args.get("help") {
        println!("{}", help_text.value.as_str().unwrap_or(""));
    }
    if let Some(arg_data) = matches.args.get("llm_id") {
        if let Value::String(llm_id) = &arg_data.value {
            let llm_uuid =
                Uuid::parse_str(&llm_id).map_err(|e| format!("Argument must be a valid UUID."))?;
            match client.llm_status(llm_uuid.clone()).await {
                Ok(status) => {
                    info!("Obtained status for LLM: {}", status.name);
                    if (status.running) {
                        info!("{} is currently running.", status.name);
                    }
                    info!("Full status info:");
                    println!("{:#?}", status)
                }
                Err(e) => {
                    error!("Failed to obtain status, due to error: {:?}", e);
                }
            }
        }
    };
    Ok(())
}

async fn handle_new_cli_user_subcommand(
    matches: &Matches,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<(), String> {
    let mut u = user::User::new("cli_user".into());
    u.perm_superuser = true;
    let user_info: user::UserInfo = (&u).into();
    database::save_new_user(u, pool.clone())
        .map_err(|e| format!("Failed to safe user: {:?}", e))?;
    println!("PANTRY_CLI_USER={}", user_info.id.to_string());
    println!("PANTRY_CLI_KEY={}", user_info.api_key);
    Ok(())
}
