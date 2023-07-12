use crate::connectors;
use crate::connectors::llm_manager;
use crate::database;
use crate::emitter;
use crate::llm;
use crate::llm::LLMWrapper;
use crate::registry;
use crate::state;
use crate::user;
use chrono::serde::ts_seconds_option;
use chrono::DateTime;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use tauri::Manager;
use tauri::{AppHandle, Wry};
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
    pub capabilities: HashMap<String, isize>,
    pub requirements: String,
    pub tags: Vec<String>,

    pub url: String,

    pub create_thread: bool, //TODO: Rename this, it basically means local now
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
    pub status: String,
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
            create_thread: value.create_thread.clone(),

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

impl From<&llm::LLM> for LLMAvailable {
    fn from(value: &llm::LLM) -> Self {
        LLMAvailable {
            llm_info: value.into(),
            uuid: value.uuid.to_string(),
            downloaded: value.downloaded_reason.clone(),
            last_called: value.last_called.clone(),
        }
    }
}

#[tauri::command]
pub async fn get_requests(
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<Vec<LLMRequest>>, String> {
    // let requests = state.get_requests().await;
    println!("received command get_reqs");

    let mock_llm = LLMInfo {
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
        requester: "fake".into(),
    };
    Ok(CommandResponse { data: vec![mock] })
    // Err("boop".into())
}

#[tauri::command]
pub async fn active_llms(
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<Vec<LLMRunningInfo>>, String> {
    let active_llms_iter = state.activated_llms.iter();
    println!("received command active_llms");
    let mut active_llms: Vec<LLMRunningInfo> = Vec::new();
    for pair in active_llms_iter {
        println!("attempting to add an active");
        let llm = pair.value();
        active_llms.push(llm.into_llm_running());
    }
    Ok(CommandResponse { data: active_llms })
}

#[tauri::command]
pub async fn available_llms(
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<Vec<LLMAvailable>>, String> {
    println!("received command available_llms");
    let available_llms_iter = database::get_available_llms(state.pool.clone())?;
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
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<DownloadResponse>, String> {
    let uuid = Uuid::new_v4();

    let id = llm_reg.id.clone();

    tokio::spawn(async move {
        registry::download_and_write_llm(llm_reg, uuid, app.clone()).await;
    });
    // Here we need to download llm_reg.url

    //Honestly idk wtf this code is even doing. It's definitely not downloading an LLM.
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
                println!("pingok");
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
    println!("Attempting to load an LLM");
    if (state.activated_llms.contains_key(&uuid)) {
        return Err("llm already loaded".into());
    }

    let manager_addr_copy = state.manager_addr.clone();

    let new_llm = database::get_llm(uuid, state.pool.clone())?;

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
    )
    .await;
    // new_llm.load();
    match result {
        Ok(running) => {
            println!("Inserting {uuid} into running LLMs");
            state.activated_llms.insert(uuid, running);
            Ok(())
        }
        Err(err) => Err("failed to launch {id} skipping".into()),
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
    app: AppHandle,
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<Vec<llm::LLMSession>>, String> {
    let uuid = Uuid::parse_str(&llm_uuid).map_err(|e| e.to_string())?;
    println!(
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
    app: AppHandle,
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<CreateSessionResponse>, String> {
    println!(
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
    println!(
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
                            let event = emitter::EventType::LLMResponse(event);

                            Ok(emitter::EventPayload {
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
    println!(
        "frontend called {} with {} and params {:?}",
        uuid, prompt, user_parameters
    );
    if let Some(llm) = state.activated_llms.get(&uuid) {
        let uuid = Uuid::new_v4();
        match state.manager_addr.ask(llm_manager::PingMessage()).await {
            Ok(result) => println!("ping result: {:?}", result),
            Err(err) => println!("ping error: {:?}", err),
        }

        println!("{:?}", llm.value().ping().await);

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
                            let event = emitter::EventType::LLMResponse(blah);

                            Ok(emitter::EventPayload {
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
    app: tauri::AppHandle,
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<(), String> {
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
pub async fn delete_llm(
    uuid: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&uuid).map_err(|e| e.to_string())?;
    println!("Attempting to delete an LLM");

    if let Some(running_llm) = state.activated_llms.remove(&uuid) {
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
        Err(err) => Err("Unable to find and delte llm".into()),
    }
}

#[tauri::command]
pub async fn interrupt_session(
    llm_uuid: String,
    session_id: String,
    app: AppHandle,
    state: tauri::State<'_, state::GlobalStateWrapper>,
) -> Result<CommandResponse<bool>, String> {
    println!(
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
