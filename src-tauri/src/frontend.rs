use crate::connectors::llm_manager;
use crate::database;
use crate::emitter;
use crate::llm;
use crate::llm::LLMWrapper;
use crate::registry;
use crate::request;
use crate::state;
use crate::user;
use chrono::serde::ts_seconds_option;
use chrono::DateTime;
use chrono::Utc;
use log::{debug, info};
use serde_json::Value;
use std::collections::HashMap;
use tauri::AppHandle;
use tauri::Manager;

use uuid::Uuid;

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
    pub capabilities: HashMap<String, i32>,
    pub requirements: String,
    pub tags: Vec<String>,

    pub url: String,

    pub local: bool, //TODO: Rename this, it basically means local now
    pub connector_type: String,
    pub config: HashMap<String, Value>, // Connector Configs Parameters

    //These aren't _useful_ to the user, but we include them for advanced users
    //to get details.
    pub parameters: HashMap<String, Value>, // Hardcoded Parameters
    pub user_parameters: Vec<String>,       //User Parameters
    pub session_parameters: HashMap<String, Value>, // Hardcoded Parameters
    pub user_session_parameters: Vec<String>, //User Parameters
}

#[derive(serde::Serialize, Debug)]
pub struct LLMRunningInfo {
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
pub struct LLMAvailableInfo {
    pub llm_info: LLMInfo,
    pub downloaded: String,
    #[serde(with = "ts_seconds_option")]
    pub last_called: Option<DateTime<Utc>>,
    pub uuid: String,
}

#[derive(serde::Serialize)]
pub struct LLMRequestInfo {
    pub id: String,
    pub user_id: String,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
    pub originator: String,
    pub request: request::UserRequestType,
    pub complete: bool,
    pub accepted: bool,
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
            parameters: value.parameters.0.clone(),
            user_parameters: value.user_parameters.0.clone(),
            session_parameters: value.session_parameters.0.clone(),
            user_session_parameters: value.user_session_parameters.0.clone(),

            capabilities: value.capabilities.0.clone(),
            homepage: value.homepage.clone(),
            license: value.license.clone(),
            requirements: value.requirements.clone(),
            url: value.url.clone(),
            tags: value.tags.0.clone(),
            local: value.local.clone(),

            connector_type: value.connector_type.to_string(),
            config: value.config.0.clone(),
        }
    }
}

impl From<&llm::LLMActivated> for LLMRunningInfo {
    fn from(value: &llm::LLMActivated) -> Self {
        LLMRunningInfo {
            llm_info: (&value.llm).into(),
            download_reason: format!(
                "Downloaded {} for {}",
                value.llm.downloaded_date.format("%b %e %T %Y"),
                value.llm.downloaded_reason
            ),
            downloaded_date: value.llm.downloaded_date,
            last_called: value.llm.last_called.clone(),
            activated: format!(
                "Activated {} for {}",
                value.activated_time.format("%b %e %T %Y"),
                value.activated_reason
            ),
            uuid: value.llm.uuid.to_string(),
        }
    }
}

impl From<&llm::LLM> for LLMAvailableInfo {
    fn from(value: &llm::LLM) -> Self {
        LLMAvailableInfo {
            llm_info: value.into(),
            uuid: value.uuid.to_string(),
            downloaded: value.downloaded_reason.clone(),
            last_called: value.last_called.clone(),
        }
    }
}

impl From<&request::UserRequest> for LLMRequestInfo {
    fn from(value: &request::UserRequest) -> Self {
        LLMRequestInfo {
            id: value.id.0.clone().to_string(),
            user_id: value.user_id.0.clone().to_string(),
            reason: value.reason.clone(),
            timestamp: value.timestamp.clone(),
            originator: value.originator.clone(),
            request: value.request.clone(),
            complete: value.complete.clone(),
            accepted: value.accepted.clone(),
        }
    }
}

#[tauri::command]
pub async fn get_requests(
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<Vec<LLMRequestInfo>>, String> {
    // let requests = state.get_requests().await;
    info!("received command get_reqs");
    let reqs = database::get_requests(state.pool.clone())
        .map_err(|err| format!("Database failure: {:?}", err))?;
    // let mut available_llms: Vec<LLMAvailable> = Vec::new();
    // for val in available_llms_iter {
    //     available_llms.push(val.value().clone().as_ref().into())
    // }
    Ok(CommandResponse {
        data: reqs.iter().map(|req| req.into()).collect(),
    })
    // Err("boop".into())
}

#[tauri::command]
pub async fn accept_request(
    request_id: String,
    state: tauri::State<'_, state::GlobalStateWrapper>,
    app: tauri::AppHandle,
) -> Result<CommandResponse<()>, String> {
    let req_uuid = Uuid::parse_str(&request_id).map_err(|e| e.to_string())?;

    let req = database::get_request(req_uuid, state.pool.clone())
        .map_err(|err| format!("Request not found: {:?}", err))?;

    match req.request {
        request::UserRequestType::DownloadRequest(dlr) => {
            let uuid = Uuid::new_v4();
            let llm_reg = dlr.llm_registry_entry;

            let _id = llm_reg.id.clone();

            tokio::spawn(async move {
                registry::download_and_write_llm(llm_reg, uuid, app.clone()).await;
            });

            database::mark_request_complete(req_uuid, true, state.pool.clone())
                .map_err(|err| format!("Databse failure: {:?}", err))?;

            Ok(CommandResponse { data: () })
        }
        request::UserRequestType::PermissionRequest(pr) => {
            database::update_permissions(
                req.user_id.0,
                pr.requested_permissions,
                state.pool.clone(),
            )
            .map_err(|err| format!("Databse failure: {:?}", err))?;

            database::mark_request_complete(req_uuid, true, state.pool.clone())
                .map_err(|err| format!("Databse failure: {:?}", err))?;

            Ok(CommandResponse { data: () })
        }
        request::UserRequestType::LoadRequest(lr) => {
            load_llm(lr.llm_id, app, state.clone()).await?;
            database::mark_request_complete(req_uuid, true, state.pool.clone())
                .map_err(|err| format!("Databse failure: {:?}", err))?;
            Ok(CommandResponse { data: () })
        }
        request::UserRequestType::UnloadRequest(ur) => {
            unload_llm(ur.llm_id, app, state.clone()).await?;
            database::mark_request_complete(req_uuid, true, state.pool.clone())
                .map_err(|err| format!("Databse failure: {:?}", err))?;
            Ok(CommandResponse { data: () })
        }
    }
}

#[tauri::command]
pub async fn reject_request(
    request_id: String,
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<()>, String> {
    let req_uuid = Uuid::parse_str(&request_id).map_err(|e| e.to_string())?;
    database::mark_request_complete(req_uuid, false, state.pool.clone())
        .map_err(|err| format!("Databse failure: {:?}", err))?;

    Ok(CommandResponse { data: () })
}

#[tauri::command]
pub async fn active_llms(
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<Vec<LLMRunningInfo>>, String> {
    let active_llms_iter = state.activated_llms.iter();
    info!("received command active_llms");
    let mut active_llms: Vec<LLMRunningInfo> = Vec::new();
    for pair in active_llms_iter {
        debug!("attempting to add an active");
        let llm = pair.value();
        active_llms.push(llm.into_llm_running());
    }
    Ok(CommandResponse { data: active_llms })
}

#[tauri::command]
pub async fn available_llms(
    _app: tauri::AppHandle,
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<Vec<LLMAvailableInfo>>, String> {
    info!("received command available_llms");
    let available_llms_iter = database::get_available_llms(state.pool.clone())
        .map_err(|err| format!("Database failure: {:?}", err))?;
    // let mut available_llms: Vec<LLMAvailable> = Vec::new();
    // for val in available_llms_iter {
    //     available_llms.push(val.value().clone().as_ref().into())
    // }
    Ok(CommandResponse {
        data: available_llms_iter.iter().map(|llm| llm.into()).collect(),
    })
}

#[derive(serde::Serialize)]
pub struct DownloadResponse {
    pub uuid: String,
    pub stream: String,
}

#[tauri::command]
pub fn download_llm(
    llm_reg: registry::LLMRegistryEntry,
    app: tauri::AppHandle,
    _state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<DownloadResponse>, String> {
    let uuid = Uuid::new_v4();

    let id = llm_reg.id.clone();

    tokio::spawn(async move {
        registry::download_and_write_llm(llm_reg, uuid, app.clone()).await;
    });

    Ok(CommandResponse {
        data: DownloadResponse {
            uuid: uuid.to_string(),
            stream: format!("{}-{}", id, uuid),
        },
    })
}

#[tauri::command]
pub fn get_user_settings(
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<state::UserSettingsInfo, String> {
    let user_settings = state.user_settings.read().unwrap();
    Ok(state::UserSettingsInfo::from(&*user_settings)) // Convert UserSettings to UserSettingsInfo
}

#[tauri::command]
pub fn set_user_setting(
    key: String,
    value: serde_json::Value,
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<(), String> {
    let mut user_settings = state.user_settings.write().unwrap();

    match key.as_str() {
        "use_gpu" => {
            user_settings.use_gpu = value.as_bool().ok_or("Invalid value for 'use_gpu'")?
        }
        "n_thread" => {
            user_settings.n_thread = value.as_u64().ok_or("Invalid value for 'n_thread'")? as usize
        }
        "preferred_active_sessions" => {
            user_settings.preferred_active_sessions = value
                .as_u64()
                .ok_or("Invalid value for 'preferred_active_sessions'")?
                as usize
        }
        "dedup_downloads" => {
            user_settings.dedup_downloads = value
                .as_bool()
                .ok_or("Invalid value for 'dedup_downloads'")?
        }
        "n_batch" => {
            user_settings.n_batch = value.as_u64().ok_or("Invalid value for 'n_batch'")? as usize
        }
        "openai_key" => {
            // Assuming 'value' is a string containing the new password
            let new_password = value.as_str().ok_or("Invalid value for 'openai_key'")?;
            user_settings
                .openai_key
                .set_password(new_password)
                .map_err(|e| e.to_string())?
        }
        _ => return Err(format!("Unknown setting '{}'", key)),
    }

    user_settings.save()
}

#[tauri::command]
pub async fn ping(
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<Vec<String>, String> {
    match state.manager_addr.ask(llm_manager::PingMessage {}).await {
        Ok(val) => match val {
            Ok(va) => {
                debug!("pingok");
                Ok(va)
            }
            Err(ma_err) => Err(ma_err.to_string()),
        },
        Err(ma_err) => Err(ma_err.to_string()),
    }
}

#[tauri::command]
pub async fn load_llm(
    uuid: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<(), String> {
    // let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let uuid = Uuid::parse_str(&uuid).map_err(|e| e.to_string())?;

    info!("Attempting to load an LLM");
    if state.activated_llms.contains_key(&uuid) {
        return Err("llm already loaded".into());
    }

    let manager_addr_copy = state.manager_addr.clone();

    let new_llm = database::get_llm(uuid, state.pool.clone())
        .map_err(|err| format!("Database failure: {:?}", err))?;

    let path = app
        .path_resolver()
        .app_local_data_dir()
        .ok_or("no path no llms")?;
    let settings = state.user_settings.read().unwrap().clone();
    let result = llm::LLMActivated::activate_llm(
        new_llm.clone(),
        manager_addr_copy,
        path,
        settings,
        state.pool.clone(),
        app.clone(),
    )
    .await;
    // new_llm.load();
    match result {
        Ok(running) => {
            debug!("Inserting {uuid} into running LLMs");
            state.activated_llms.insert(uuid, running);
            Ok(())
        }
        Err(_err) => Err("failed to launch {id} skipping".into()),
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

#[derive(Debug, serde::Serialize)]
pub struct CallLLMResponse {
    pub session_id: String,
    pub parameters: HashMap<String, Value>,
    // I don't think we use this, we don't need it.
    // pub llm_info: LLMInfo,
}

// Define the response structure for the prompt_session command.
#[derive(Debug, serde::Serialize)]
pub struct PromptSessionResponse {
    // I don't think we use this, we don't need it.
    // pub llm_info: LLMInfo,
}

#[derive(Debug, serde::Serialize)]
pub struct CreateSessionResponse {
    pub session_parameters: HashMap<String, Value>,
    // I don't think we use this, we don't need it.
    // pub llm_info: LLMInfo,
    pub session_id: String,
}

#[tauri::command]
pub async fn get_sessions(
    llm_uuid: String,
    _app: AppHandle,
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<Vec<(llm::LLMSession, Vec<llm::LLMHistoryItem>)>>, String> {
    let uuid = Uuid::parse_str(&llm_uuid).map_err(|e| e.to_string())?;
    info!(
        "Frontend called get_sessions with LLM UUID {:?} and user {:?}",
        llm_uuid,
        user::get_local_user()
    );
    if let Some(llm) = state.activated_llms.get(&uuid) {
        match llm.value().get_sessions(user::get_local_user()).await {
            Ok(sessions) => Ok(CommandResponse { data: sessions }),
            Err(err) => Err(err.to_string()),
        }
    } else {
        Err(format!("LLM with UUID {} not found", llm_uuid))
    }
}

#[tauri::command]
pub async fn create_session(
    llm_uuid: String,
    user_session_parameters: HashMap<String, Value>,
    _app: AppHandle,
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<CreateSessionResponse>, String> {
    info!(
        "Frontend called create_session for {} with parameters {:?} and user {:?}",
        llm_uuid,
        user_session_parameters,
        user::get_local_user()
    );
    let uuid = Uuid::parse_str(&llm_uuid).map_err(|e| e.to_string())?;
    if let Some(llm) = state.activated_llms.get(&uuid) {
        match llm
            .value()
            .create_session(user_session_parameters, user::get_local_user())
            .await
        {
            Ok(resp) => Ok(CommandResponse {
                data: CreateSessionResponse {
                    session_parameters: resp.session_parameters,
                    session_id: resp.session_id.to_string(),
                    // llm_info: llm.llm.as_ref().into(),
                },
            }),
            Err(err) => Err(err.to_string()),
        }
    } else {
        Err(format!("LLM with UUID {} not found", uuid))
    }
}

#[tauri::command]
pub async fn prompt_session(
    llm_uuid: String,
    session_id: Uuid,
    prompt: String,
    parameters: HashMap<String, Value>,
    app: AppHandle,
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<PromptSessionResponse>, String> {
    info!(
        "Frontend called prompt_session with session_id {:?}, prompt {:?}, and user {:?}",
        session_id,
        prompt,
        user::get_local_user()
    );
    let uuid = Uuid::parse_str(&llm_uuid).map_err(|e| e.to_string())?;
    if let Some(llm) = state.activated_llms.get(&uuid) {
        match llm
            .value()
            .prompt_session(session_id, prompt, parameters, user::get_local_user())
            .await
        {
            Ok(prompt_response) => {
                tokio::spawn(async move {
                    emitter::send_events(
                        "llm_response".into(),
                        session_id.to_string(),
                        prompt_response.stream,
                        app,
                        |stream_id, event| {
                            let event = emitter::EmitterEventPayload::LLMResponse(event);

                            Ok(emitter::EmitterEvent {
                                stream_id: stream_id,
                                event: event,
                            })
                        },
                    )
                    .await
                });

                Ok(CommandResponse {
                    data: PromptSessionResponse {},
                })
            }
            Err(err) => Err(err.to_string()),
        }
    } else {
        Err(format!("LLM with UUID {} not found", uuid))
    }
}

#[tauri::command]
pub async fn call_llm(
    llm_uuid: String,
    prompt: String,
    user_session_parameters: HashMap<String, Value>,
    user_parameters: HashMap<String, Value>,
    app: AppHandle,
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<CallLLMResponse>, String> {
    let uuid = Uuid::parse_str(&llm_uuid).map_err(|e| e.to_string())?;
    info!(
        "frontend called {} with {} and params {:?}",
        uuid, prompt, user_parameters
    );
    if let Some(llm) = state.activated_llms.get(&uuid) {
        let _uuid = Uuid::new_v4();
        match state.manager_addr.ask(llm_manager::PingMessage()).await {
            Ok(result) => debug!("ping result: {:?}", result),
            Err(err) => debug!("ping error: {:?}", err),
        }

        debug!("{:?}", llm.value().ping().await);

        match llm
            .value()
            .call_llm(
                &prompt,
                user_session_parameters,
                user_parameters,
                user::get_local_user(),
            )
            .await
        {
            Ok(llm_resp) => {
                tokio::spawn(async move {
                    emitter::send_events(
                        "llm_response".into(),
                        llm_resp.session_id.to_string(),
                        llm_resp.stream,
                        app,
                        |stream_id, blah| {
                            let event = emitter::EmitterEventPayload::LLMResponse(blah);

                            Ok(emitter::EmitterEvent {
                                stream_id: stream_id,
                                event: event,
                            })
                        },
                    )
                    .await
                });

                Ok(CommandResponse {
                    data: CallLLMResponse {
                        session_id: llm_resp.session_id.to_string(),
                        parameters: llm_resp.parameters,
                        // llm_info: llm.llm.as_ref().into(),
                    },
                })
            }
            Err(err) => Err(err.to_string()),
        }
    } else {
        Err("Couldn't find LLM".into())
    }
}
// #[tauri::command]
// pub async fn unload_llm(id: String, state: tauri::State<'_, GlobalStateWrapper>) -> Result<(), String> {
//     state.unload_llm(id).await
// }
//
#[tauri::command]
pub async fn unload_llm(
    uuid: String,
    _app: tauri::AppHandle,
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&uuid).map_err(|e| e.to_string())?;
    info!("Attempting to unload an LLM");

    if let Some(_running_llm) = state.activated_llms.remove(&uuid) {
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
pub async fn delete_llm(
    uuid: String,
    _app: tauri::AppHandle,
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&uuid).map_err(|e| e.to_string())?;
    info!("Attempting to delete an LLM");

    if let Some(_running_llm) = state.activated_llms.remove(&uuid) {
        let unload_message = llm_manager::UnloadLLMActorMessage { uuid };
        let manager_addr = state.manager_addr.clone();

        manager_addr
            .ask(unload_message)
            .await
            .map_err(|err| {
                format!(
                    "Failed to send unload message to LLMManagerActor: {:?}",
                    err
                )
            })?
            .map_err(|err| format!("Failed to unload: {:?}", err))?;
    }
    match database::delete_llm(uuid, state.pool.clone()) {
        Ok(_) => Ok(()),
        Err(_err) => Err("Unable to find and delete llm".into()),
    }
}

#[tauri::command]
pub async fn interrupt_session(
    llm_uuid: String,
    session_id: String,
    _app: AppHandle,
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<bool>, String> {
    info!(
        "Frontend called interrupt_session for LLM UUID {}, session ID {}",
        llm_uuid, session_id
    );
    let llm_uuid = Uuid::parse_str(&llm_uuid).map_err(|e| e.to_string())?;
    let session_uuid = Uuid::parse_str(&session_id).map_err(|e| e.to_string())?;

    if let Some(llm) = state.activated_llms.get(&llm_uuid) {
        match llm
            .value()
            .interrupt_session(session_uuid, user::get_local_user())
            .await
        {
            Ok(interrupted) => Ok(CommandResponse { data: interrupted }),
            Err(err) => Err(err.to_string()),
        }
    } else {
        Err(format!("LLM with UUID {} not found", llm_uuid))
    }
}

#[tauri::command]
pub async fn exec_path(app: AppHandle) -> Result<CommandResponse<String>, String> {
    Ok(CommandResponse {
        data: tauri::utils::platform::current_exe()
            .map_err(|e| format!("Failed to get current path."))?
            .into_os_string()
            .into_string()
            .map_err(|e| format!("Failed to convert current path."))?,
    })
}

#[tauri::command]
pub async fn new_cli_user(
    app: AppHandle,
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<user::UserInfo>, String> {
    let mut u = user::User::new("cli_user".into());
    u.perm_superuser = true;
    let user_info = (&u).into();
    database::save_new_user(u, state.pool.clone());
    Ok(CommandResponse { data: user_info })
}
