//server.rs
use crate::connectors::LLMEvent;
use crate::database;
use crate::database_types::DbUuid;
use crate::listeners::create_listeners;
use crate::llm::{LLMActivated, LLMWrapper, LLM};
use crate::llm_manager;
use crate::registry;
use crate::request;
use crate::request::{UserRequest, UserRequestType};
use crate::schema;
use crate::state;
use crate::user;
use axum::{extract::State, Json};
use axum::{
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
    routing::post,
    Router,
};
use axum_macros;
use chrono::DateTime;
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use futures_util::stream::Stream;
use hyper::StatusCode;
use serde;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::{convert::Infallible, time::Duration};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio_stream::{wrappers::ReceiverStream, StreamExt as _};
use uuid::Uuid;

// We're deliberately disconnecting these datastructures from frontend.rs
// because the frontend is a superuser panel, and this is a regular API.
#[derive(serde::Serialize, Debug)]
pub struct LLMStatus {
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

    pub local: bool, //rename from local,
    pub connector_type: String,
    pub config: HashMap<String, Value>, // Connector Configs Parameters

    //These aren't _useful_ to the user, but we include them for advanced users
    //to get details.
    pub parameters: HashMap<String, Value>, // Hardcoded Parameters
    pub user_parameters: Vec<String>,       //User Parameters
    pub session_parameters: HashMap<String, Value>, // Hardcoded Parameters
    pub user_session_parameters: Vec<String>, //User Parameters

    //non llminfo fields
    pub uuid: String, // All LLMStatus are downloaded,
    pub running: bool,
}

//This is a lot like frontend::LLMRunningInfo, but limited for non-superusers
#[derive(serde::Serialize, Debug)]
pub struct LLMRunningStatus {
    pub llm_info: LLMStatus,
    pub uuid: String,
    // #[serde(skip_serializing)]
    // pub llm: dyn LLMWrapper + Send + Sync
}

#[derive(serde::Serialize, Debug)]
pub struct LLMAvailableStatus {
    pub llm_info: LLMStatus,
    pub uuid: String,
}

impl From<&LLM> for LLMStatus {
    fn from(llm: &LLM) -> Self {
        LLMStatus {
            id: llm.id.clone(),
            family_id: llm.family_id.clone(),
            organization: llm.organization.clone(),
            name: llm.name.clone(),
            homepage: llm.homepage.clone(),
            license: llm.license.clone(),
            description: llm.description.clone(),
            capabilities: llm.capabilities.0.clone(),
            requirements: llm.requirements.clone(),
            tags: llm.tags.0.clone(),
            url: llm.url.clone(),
            local: llm.local.clone(),
            connector_type: llm.connector_type.to_string(),
            config: llm.config.0.clone(),
            parameters: llm.parameters.0.clone(),
            user_parameters: llm.user_parameters.0.clone(),
            session_parameters: llm.session_parameters.0.clone(),
            user_session_parameters: llm.user_session_parameters.0.clone(),
            uuid: llm.uuid.to_string(),
            running: false,
        }
    }
}

impl From<&LLMActivated> for LLMStatus {
    fn from(llm: &LLMActivated) -> Self {
        LLMStatus {
            id: llm.llm.id.clone(),
            family_id: llm.llm.family_id.clone(),
            organization: llm.llm.organization.clone(),
            name: llm.llm.name.clone(),
            homepage: llm.llm.homepage.clone(),
            license: llm.llm.license.clone(),
            description: llm.llm.description.clone(),
            capabilities: llm.llm.capabilities.0.clone(),
            requirements: llm.llm.requirements.clone(),
            tags: llm.llm.tags.0.clone(),
            url: llm.llm.url.clone(),
            local: llm.llm.local.clone(),
            connector_type: llm.llm.connector_type.to_string(),
            config: llm.llm.config.0.clone(),
            parameters: llm.llm.parameters.0.clone(),
            user_parameters: llm.llm.user_parameters.0.clone(),
            session_parameters: llm.llm.session_parameters.0.clone(),
            user_session_parameters: llm.llm.user_session_parameters.0.clone(),
            uuid: llm.llm.uuid.to_string(),
            running: true,
        }
    }
}

impl From<&LLMActivated> for LLMRunningStatus {
    fn from(llm: &LLMActivated) -> Self {
        LLMRunningStatus {
            llm_info: llm.into(),
            uuid: llm.llm.uuid.to_string(),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserRequestStatus {
    pub id: Uuid,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub request: UserRequestType,
    pub complete: bool,
    pub accepted: bool,
}
impl From<&UserRequest> for UserRequestStatus {
    fn from(llm_req: &UserRequest) -> Self {
        UserRequestStatus {
            id: llm_req.id.0.clone(),
            user_id: llm_req.user_id.0.clone(),
            timestamp: llm_req.timestamp.clone(),
            request: llm_req.request.clone(),
            complete: llm_req.complete.clone(),
            accepted: llm_req.accepted.clone(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum CapabilityType {
    General,
    Assistant,
    Writing,
    Coding,
}

impl fmt::Display for CapabilityType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CapabilityType::General => write!(f, "general"),
            CapabilityType::Assistant => write!(f, "assistant"),
            CapabilityType::Writing => write!(f, "writing"),
            CapabilityType::Coding => write!(f, "coding"),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct CapabilityFilter {
    pub capability: CapabilityType,
    pub value: i32,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LLMFilter {
    pub llm_uuid: Option<Uuid>,
    pub llm_id: Option<String>,
    pub family_id: Option<String>,
    pub local: Option<bool>,
    pub minimum_capabilities: Option<Vec<CapabilityFilter>>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LLMPreference {
    pub llm_uuid: Option<Uuid>,
    pub llm_id: Option<String>,
    pub local: Option<bool>,
    pub family_id: Option<String>,
    pub capability_type: Option<CapabilityType>,
}

fn user_permission_check(
    required: &str,
    api_key: String,
    // user: &user::User,
    user_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<user::User, (StatusCode, String)> {
    let user = database::get_user(user_id, pool)
        .map_err(|_err| (StatusCode::UNAUTHORIZED, "Not a Valid User {:?}".into()))?;
    if user.api_key != api_key {
        return Err((StatusCode::UNAUTHORIZED, "Incorrect API Key".into()));
    };
    if user.perm_superuser.clone() {
        return Ok(user);
    }
    let auth = match required {
        "" => true,
        "load_llm" => user.perm_load_llm.clone(),
        "unload_llm" => user.perm_unload_llm.clone(),
        "download_llm" => user.perm_download_llm.clone(),
        "session" => user.perm_session.clone(), //this is for create_sessioon AND prompt_session
        "request_download" => user.perm_request_download.clone(),
        "request_load" => user.perm_request_load.clone(),
        "request_unload" => user.perm_request_unload.clone(),
        "view_llms" => user.perm_view_llms.clone(),
        "bare_model" => user.perm_bare_model.clone(),
        &_ => false,
    };
    match auth {
        true => Ok(user),
        false => Err((StatusCode::UNAUTHORIZED, "Incorrect Permissions".into())),
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct RegisterUserRequest {
    user_name: String,
}

#[axum_macros::debug_handler]
async fn register_user(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<RegisterUserRequest>,
) -> Result<Json<user::UserInfo>, (StatusCode, String)> {
    println!("Called register_user from API.");
    let user = user::User::new(payload.user_name);
    match database::save_new_user(user, state.pool.clone()) {
        Ok(user) => Ok(Json((&user).into())),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error creating user".into(),
        )),
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct RequestPermissionRequest {
    user_id: String,
    api_key: String,
    requested_permissions: user::Permissions,
}
#[axum_macros::debug_handler]
async fn request_permissions(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<RequestPermissionRequest>,
) -> Result<Json<UserRequest>, (StatusCode, String)> {
    println!("Called request_permissions from API.");
    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let user = user_permission_check("", payload.api_key, user_uuid, state.pool.clone())?;

    let request = UserRequest {
        id: DbUuid(Uuid::new_v4()),
        reason: "reason".into(),
        timestamp: chrono::Utc::now(),
        originator: user.name.clone(),
        user_id: user.id,
        request: UserRequestType::PermissionRequest(request::PermissionRequest {
            requested_permissions: payload.requested_permissions,
        }),
        complete: false,
        accepted: false,
    };

    let req = database::save_new_request(request, state.pool.clone()).map_err(|err| {
        println!("failed to save to database because... {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error saving new request.".into(),
        )
    })?;
    Ok(Json(req))
    // Ok(Json(request.id.to_string()))
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct RequestDownloadRequest {
    user_id: String,
    api_key: String,
    llm_registry_entry: registry::LLMRegistryEntry,
}
#[axum_macros::debug_handler]
async fn request_download(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<RequestDownloadRequest>,
) -> Result<Json<UserRequest>, (StatusCode, String)> {
    println!("Called request_download from API.");
    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let user = user_permission_check(
        "request_download",
        payload.api_key,
        user_uuid,
        state.pool.clone(),
    )?;

    let request = UserRequest {
        id: DbUuid(Uuid::new_v4()),
        timestamp: chrono::Utc::now(),
        reason: "reason".into(),
        originator: user.name.clone(),
        user_id: user.id,
        request: UserRequestType::DownloadRequest(request::DownloadRequest {
            llm_registry_entry: payload.llm_registry_entry,
        }),
        complete: false,
        accepted: false,
    };
    let req = database::save_new_request(request, state.pool.clone()).map_err(|err| {
        println!("failed to save to database because... {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error saving new request.".into(),
        )
    })?;
    Ok(Json(req))
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct RequestLoadRequest {
    user_id: String,
    api_key: String,
    llm_id: String,
}

#[axum_macros::debug_handler]
async fn request_load(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<RequestLoadRequest>,
) -> Result<Json<UserRequest>, (StatusCode, String)> {
    println!("Called request_load from API.");
    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let user = user_permission_check(
        "request_load_llm",
        payload.api_key,
        user_uuid,
        state.pool.clone(),
    )?;

    let request = UserRequest {
        id: DbUuid(Uuid::new_v4()),
        reason: "reason".into(),
        timestamp: chrono::Utc::now(),
        originator: user.name.clone(),
        user_id: DbUuid(user.id.0),
        request: UserRequestType::LoadRequest(request::LoadRequest {
            llm_id: payload.llm_id,
        }),
        complete: false,
        accepted: false,
    };

    let req = database::save_new_request(request, state.pool.clone()).map_err(|err| {
        println!("failed to save to database because... {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error saving new request.".into(),
        )
    })?;
    Ok(Json(req))
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct RequestUnloadRequest {
    user_id: String,
    api_key: String,
    llm_id: String,
}
#[axum_macros::debug_handler]
async fn request_unload(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<RequestUnloadRequest>,
) -> Result<Json<UserRequest>, (StatusCode, String)> {
    println!("Called request_unload from API.");
    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let user = user_permission_check(
        "request_unload_llm",
        payload.api_key,
        user_uuid,
        state.pool.clone(),
    )?;

    let request = UserRequest {
        id: DbUuid(Uuid::new_v4()),
        reason: "reason".into(),
        timestamp: chrono::Utc::now(),
        originator: user.name.clone(),
        user_id: DbUuid(user.id.0),
        request: UserRequestType::UnloadRequest(request::UnloadRequest {
            llm_id: payload.llm_id,
        }),
        complete: false,
        accepted: false,
    };
    let req = database::save_new_request(request, state.pool.clone()).map_err(|err| {
        println!("failed to save to database because... {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error saving new request.".into(),
        )
    })?;
    Ok(Json(req))
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct RequestStatusRequest {
    user_id: String,
    api_key: String,
    request_id: String,
}

#[axum_macros::debug_handler]
async fn request_status(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<RequestStatusRequest>,
) -> Result<Json<UserRequestStatus>, (StatusCode, String)> {

    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let request_uuid =
        Uuid::parse_str(&payload.request_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let user = user_permission_check(
        "",
        payload.api_key.clone(),
        user_uuid,
        state.pool.clone(),
    )?;

    let req = database::get_request(request_uuid, state.pool.clone()).map_err(|err| {
        println!("didn't find {:?}", request_uuid);
        (
            StatusCode::NOT_FOUND,
            "Request Not Found".into(),
        )
    })?;
    if user_uuid != req.user_id.0 {
        println!("uuid didn't match find {:?} vs {:?}", user_uuid, req.user_id.0);
        return Err( (
            StatusCode::NOT_FOUND,
            "Request Not Found".into(),
        ))
    }

    Ok(Json((&req).into()))
}



#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct RequestLoadFlexRequest {
    user_id: String,
    api_key: String,
    llm_id: String,
    filter: Option<LLMFilter>,
    preference: Option<LLMPreference>,
}

#[axum_macros::debug_handler]
async fn request_load_flex(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<RequestLoadFlexRequest>,
) -> Result<Json<UserRequest>, (StatusCode, String)> {
    println!("Called request_load_flex from API.");
    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let user = user_permission_check(
        "request_load_llm",
        payload.api_key.clone(),
        user_uuid,
        state.pool.clone(),
    )?;

    let mut llms = database::get_available_llms(state.pool.clone())
        .map_err(|err| {
        println!("failed to save to database because... {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".into())})?;
    // let mut llms: Vec<Uuid> = state
    //     .activated_llms
    //     .iter()
    //     .map(|pair| (pair.key()).clone())
    //     .collect();

    // let llms = llms.into_iter().filter(|llm| )

    if let Some(filter) = payload.filter {
        if let Some(llm_uuid_filter) = filter.llm_uuid {
            llms = llms
                .into_iter()
                .filter(|llm| llm.uuid.0 == llm_uuid_filter)
                .collect();
        }
        if let Some(llm_id_filter) = filter.llm_id {
            llms = llms
                .into_iter()
                .filter(|llm| llm.id == llm_id_filter)
                .collect();
        }
        if let Some(family_id_filter) = filter.family_id {
            llms = llms
                .into_iter()
                .filter(|llm| llm.family_id == family_id_filter)
                .collect();
        }

        if let Some(local_filter) = filter.local {
            llms = llms
                .into_iter()
                .filter(|llm| llm.local == local_filter)
                .collect();
        }

        if let Some(capabilities_filter) = filter.minimum_capabilities {
            for cap_fil in capabilities_filter.into_iter() {
                let capability_name = cap_fil.capability;
                let capability_min = cap_fil.value;
                llms = llms
                    .into_iter()
                    .filter(|llm| {
                        llm.capabilities
                            .0
                            .get(&capability_name.to_string())
                            .is_some_and(|x| x.clone() >= capability_min.into())
                    })
                    .collect()
            }
        }
    }

    println!("Filtered LLMS: {:?}", llms);

    if llms.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            "No running LLM matching requirements.".into(),
        ));
    } else if llms.len() == 1 {
        return request_load(
            state,
            Json(RequestLoadRequest {
                user_id: payload.user_id,
                api_key: payload.api_key,
                llm_id: llms.pop().unwrap().uuid.0.to_string(),
            }),
        )
        .await;
    }

    let mut capability_pref = CapabilityType::General;
    if let Some(preference) = payload.preference {
        // uuid is a singular preference. if we find it, we go.
        if let Some(uuid_pref) = preference.llm_uuid {
            if let Some(found) = llms.iter().find(|llm| llm.uuid.0 == uuid_pref) {
                return request_load(
                    state,
                    Json(RequestLoadRequest {
                        user_id: payload.user_id,
                        api_key: payload.api_key,
                        llm_id: llms.pop().unwrap().uuid.0.to_string(),
                    }),
                )
                .await;
            }
        }

        // id is a singular preference. if we find it, we go.
        if let Some(id_pref) = preference.llm_id {
            if let Some(found) = llms.iter().find(|llm| llm.id == id_pref) {
                return request_load(
                    state,
                    Json(RequestLoadRequest {
                        user_id: payload.user_id,
                        api_key: payload.api_key,
                        llm_id: llms.pop().unwrap().uuid.0.to_string(),
                    }),
                )
                .await;
            }
        }

        if let Some(local_pref) = preference.local {
            let count = llms.iter().filter(|llm| llm.local == local_pref).count();
            if count > 0 {
                llms = llms
                    .into_iter()
                    .filter(|llm| llm.local == local_pref)
                    .collect();
            }
        }

        if let Some(family_pref) = preference.family_id {
            let count = llms
                .iter()
                .filter(|llm| llm.family_id == family_pref)
                .count();
            if count > 0 {
                llms = llms
                    .into_iter()
                    .filter(|llm| llm.family_id == family_pref)
                    .collect();
            }
        }

        if let Some(cap_pref) = preference.capability_type {
            capability_pref = cap_pref;
        }
    }

    // Finally we select the most capability model, based on their preference
    // or in general

    llms.sort_by(|a, b| {
        a.capabilities
            .get(&capability_pref.to_string())
            .unwrap_or(&-1)
            .cmp(
                b.capabilities
                    .get(&capability_pref.to_string())
                    .unwrap_or(&-1),
            )
    });

    if llms.is_empty() {
        println!("Major malfunction, LLMs empty should be impossible here.");
        //fail gracefully
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failure in sorting code, please contact support".into(),
        ));
    }
    return request_load(
        state,
        Json(RequestLoadRequest {
            user_id: payload.user_id,
            api_key: payload.api_key,
            llm_id: llms.pop().unwrap().uuid.0.to_string(),
        }),
    )
    .await;
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GetLLMStatusRequest {
    user_id: String,
    api_key: String,
    llm_id: String,
}

#[axum_macros::debug_handler]
async fn get_llm_status(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<GetLLMStatusRequest>,
) -> Result<Json<LLMStatus>, (StatusCode, String)> {
    println!("Called get_llm_status from API.");
    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let llm_id =
        Uuid::parse_str(&payload.llm_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let user = user_permission_check("view_llms", payload.api_key, user_uuid, state.pool.clone())?;
    let llm = database::get_llm(llm_id, state.pool.clone())
        .map_err(|err| {
            println!("Failed to database: {:?}", err.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".into())})?;

    Ok(Json((&llm).into()))
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GetAvailableLLMRequest {
    user_id: String,
    api_key: String,
}

#[axum_macros::debug_handler]
async fn get_available_llms(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<GetAvailableLLMRequest>,
) -> Result<Json<Vec<LLMStatus>>, (StatusCode, String)> {
    println!("Called get_available_llms from API.");
    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let user = user_permission_check("view_llms", payload.api_key, user_uuid, state.pool.clone())?;
    let llms = database::get_available_llms(state.pool.clone())
        .map_err(|err| {
            println!("Failed to database: {:?}", err.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".into())})?;

    Ok(Json(llms.iter().map(|val| (val).into()).collect()))
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GetRunningLLMRequest {
    user_id: String,
    api_key: String,
}

#[axum_macros::debug_handler]
async fn get_running_llms(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<GetRunningLLMRequest>,
) -> Result<Json<Vec<LLMStatus>>, (StatusCode, String)> {
    println!("Called get_running_llms from API.");
    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let user = user_permission_check("view_llms", payload.api_key, user_uuid, state.pool.clone())?;
    let llms: Vec<LLMStatus> = state
        .activated_llms
        .iter()
        .map(|pair| pair.value().into())
        .collect();
    Ok(Json(llms))
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct InterruptSessionRequest {
    user_id: String,
    api_key: String,
    llm_uuid: String,
    session_id: String,
}

#[axum_macros::debug_handler]
async fn interrupt_session(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<InterruptSessionRequest>,
) -> Result<Json<LLMRunningStatus>, (StatusCode, String)> {
    println!("Called interrupt_session from API.");
    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let llm_id =
        Uuid::parse_str(&payload.llm_uuid).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let session_id = Uuid::parse_str(&payload.session_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let user = user_permission_check(
        "session",
        payload.api_key,
        user_uuid,
        state.pool.clone(),
    )?;
    let llm = state
        .activated_llms
        .get(&llm_id)
        .ok_or(format!("LLM not running"))
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    llm.value()
        .interrupt_session(session_id, user)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json((llm.value()).into()))
}

/* Once a function has selected an LLM, this function isolates the work to actually boot it up */
async fn llm_loading_assistant(
    state: State<state::GlobalStateWrapper>,
    new_llm: LLM,
) -> Result<Json<LLMRunningStatus>, (StatusCode, String)> {
    println!("Called llm_loading_assistant from API.");
    if state.activated_llms.contains_key(&new_llm.uuid) {
        return Ok(Json(state.activated_llms.get(&new_llm.uuid).unwrap().value().into()))
        // return Err((StatusCode::OK, "LLM Already Activated".into()));
    }

    let manager_addr_copy = state.manager_addr.clone();

    let path = state.local_path.clone();
    let settings = state.user_settings.read().unwrap().clone();
    let result = LLMActivated::activate_llm(
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
            let status = (&running).into();
            state.activated_llms.insert(running.llm.uuid.0, running);
            Ok(Json(status))
        }
        Err(_err) => Err((
            StatusCode::INSUFFICIENT_STORAGE,
            "Failed to launch llm: {id}".into(),
        )),
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct LoadLLMRequest {
    user_id: String,
    api_key: String,
    llm_id: String,
}
#[axum_macros::debug_handler]
async fn load_llm(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<LoadLLMRequest>,
) -> Result<Json<LLMRunningStatus>, (StatusCode, String)> {
    println!("Called load_llm from API.");
    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let _user = user_permission_check("load_llm", payload.api_key, user_uuid, state.pool.clone())?;

    let count = database::count_llm_by_pub_id(payload.llm_id.clone(), state.pool.clone())
        .map_err(|err| {
            println!("Failed to database: {:?}", err.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".into())})?;

    let new_llm: LLM;
    if count == 1 {
        new_llm = database::get_llm_pub_id(payload.llm_id, state.pool.clone())
            .map_err(|err| {
                println!("Failed to database: {:?}", err.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".into())})?;
    } else {
        let llm_uuid = Uuid::parse_str(&payload.llm_id)
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        new_llm = database::get_llm(llm_uuid, state.pool.clone()).map_err(|err| match err {
            diesel::result::Error::NotFound => (StatusCode::NOT_FOUND, "Unable to find LLM".into()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".into()),
        })?;
    };
    llm_loading_assistant(state, new_llm).await
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct LoadLLMFlexRequest {
    user_id: String,
    api_key: String,
    filter: Option<LLMFilter>,
    preference: Option<LLMPreference>,
}

#[axum_macros::debug_handler]
async fn load_llm_flex(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<LoadLLMFlexRequest>,
) -> Result<Json<LLMRunningStatus>, (StatusCode, String)> {
    println!("Called load_llm_flex from API.");
    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let user = user_permission_check("load_llm", payload.api_key, user_uuid, state.pool.clone())?;
    // We should use currently running LLMs.
    let mut llms = database::get_available_llms(state.pool.clone())
        .map_err(|err| {
            println!("Failed to database: {:?}", err.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".into())})?;
    // let mut llms: Vec<Uuid> = state
    //     .activated_llms
    //     .iter()
    //     .map(|pair| (pair.key()).clone())
    //     .collect();

    // let llms = llms.into_iter().filter(|llm| )

    if let Some(filter) = payload.filter {
        if let Some(llm_uuid_filter) = filter.llm_uuid {
            llms = llms
                .into_iter()
                .filter(|llm| llm.uuid.0 == llm_uuid_filter)
                .collect();
        }
        if let Some(llm_id_filter) = filter.llm_id {
            llms = llms
                .into_iter()
                .filter(|llm| llm.id == llm_id_filter)
                .collect();
        }
        if let Some(family_id_filter) = filter.family_id {
            llms = llms
                .into_iter()
                .filter(|llm| llm.family_id == family_id_filter)
                .collect();
        }

        if let Some(local_filter) = filter.local {
            llms = llms
                .into_iter()
                .filter(|llm| llm.local == local_filter)
                .collect();
        }

        if let Some(capabilities_filter) = filter.minimum_capabilities {
            for cap_fil in capabilities_filter.into_iter() {
                let capability_name = cap_fil.capability;
                let capability_min = cap_fil.value;
                llms = llms
                    .into_iter()
                    .filter(|llm| {
                        llm.capabilities
                            .0
                            .get(&capability_name.to_string())
                            .is_some_and(|x| x.clone() >= capability_min.into())
                    })
                    .collect()
            }
        }
    }

    println!("Filtered LLMS: {:?}", llms);

    if llms.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            "No running LLM matching requirements.".into(),
        ));
    } else if llms.len() == 1 {
        return llm_loading_assistant(state, llms.pop().unwrap()).await;
    }

    let mut capability_pref = CapabilityType::General;
    if let Some(preference) = payload.preference {
        // uuid is a singular preference. if we find it, we go.
        if let Some(uuid_pref) = preference.llm_uuid {
            if let Some(found) = llms.iter().find(|llm| llm.uuid.0 == uuid_pref) {
                return llm_loading_assistant(state, llms.pop().unwrap()).await;
            }
        }

        // id is a singular preference. if we find it, we go.
        if let Some(id_pref) = preference.llm_id {
            if let Some(found) = llms.iter().find(|llm| llm.id == id_pref) {
                return llm_loading_assistant(state, llms.pop().unwrap()).await;
            }
        }

        if let Some(local_pref) = preference.local {
            let count = llms.iter().filter(|llm| llm.local == local_pref).count();
            if count > 0 {
                llms = llms
                    .into_iter()
                    .filter(|llm| llm.local == local_pref)
                    .collect();
            }
        }

        if let Some(family_pref) = preference.family_id {
            let count = llms
                .iter()
                .filter(|llm| llm.family_id == family_pref)
                .count();
            if count > 0 {
                llms = llms
                    .into_iter()
                    .filter(|llm| llm.family_id == family_pref)
                    .collect();
            }
        }

        if let Some(cap_pref) = preference.capability_type {
            capability_pref = cap_pref;
        }
    }

    // Finally we select the most capability model, based on their preference
    // or in general

    llms.sort_by(|a, b| {
        a.capabilities
            .get(&capability_pref.to_string())
            .unwrap_or(&-1)
            .cmp(
                b.capabilities
                    .get(&capability_pref.to_string())
                    .unwrap_or(&-1),
            )
    });

    if llms.is_empty() {
        println!("Major malfunction, LLMs empty should be impossible here.");
        //fail gracefully
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failure in sorting code, please contact support".into(),
        ));
    }
    llm_loading_assistant(state, llms.pop().unwrap()).await
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct UnloadLLMRequest {
    user_id: String,
    api_key: String,
    llm_id: String,
}
#[axum_macros::debug_handler]
async fn unload_llm(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<UnloadLLMRequest>,
) -> Result<Json<LLMStatus>, (StatusCode, String)> {
    println!("Called unload_llm from API.");
    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    // let user = state
    let user = user_permission_check("unload_llm", payload.api_key, user_uuid, state.pool.clone())?;
    let llm_uuid =
        Uuid::parse_str(&payload.llm_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    if let Some(running_llm) = state.activated_llms.remove(&llm_uuid) {
        let unload_message = llm_manager::UnloadLLMActorMessage { uuid: llm_uuid };
        let manager_addr = state.manager_addr.clone();

        let result = manager_addr.ask(unload_message).await;

        match result {
            Ok(_) => Ok(Json((&running_llm.1).into())),
            Err(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error unloading LLM".into(),
            )),
        }
    } else {
        Err((
            StatusCode::NOT_FOUND,
            format!("LLM with ID '{}' not found", payload.llm_id),
        ))
    }
    // format!("LLM with ID '{}' loaded successfully", llm_id) todo!()
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct DownloadLLMRequest {
    user_id: String,
    api_key: String,
    llm_registry_entry: registry::LLMRegistryEntry,
}
#[axum_macros::debug_handler]
async fn download_llm(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<DownloadLLMRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    println!("Called download_llm from API.");
    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let user = user_permission_check(
        "download_llm",
        payload.api_key,
        user_uuid,
        state.pool.clone(),
    )?;

    let uuid = Uuid::new_v4();

    let id = payload.llm_registry_entry.id.clone();

    tokio::spawn(async move {
        registry::download_and_write_llm(payload.llm_registry_entry, uuid, state.handle.clone())
            .await;
    });

    Ok(Json(uuid.to_string().into()))
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct CreateSessionRequest {
    user_id: String,
    api_key: String,
    user_session_parameters: HashMap<String, Value>,
}

#[axum_macros::debug_handler]
async fn create_session(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<CreateSessionRequest>,
) -> Result<Json<CreateSessionResponse>, (StatusCode, String)> {
    println!("Called create_session from API.");
    let _user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    // let user = state
    //     .registered_users
    //     .get(&user_uuid)
    //     .ok_or((StatusCode::UNAUTHORIZED, "Invalid User".into()))?;
    // user_permission_check("", api_key, &user.value())?;

    // create_session_flex(state, user_id, api_key, None, None, user_session_parameters).await
    create_session_flex(
        state,
        Json(CreateSessionFlexRequest {
            user_id: payload.user_id,
            api_key: payload.api_key,
            filter: None,
            preference: None,
            user_session_parameters: payload.user_session_parameters,
        }),
    )
    .await
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct CreateSessionIdRequest {
    user_id: String,
    api_key: String,
    llm_id: String,
    user_session_parameters: HashMap<String, Value>,
}
#[axum_macros::debug_handler]
async fn create_session_id(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<CreateSessionIdRequest>,
) -> Result<Json<CreateSessionResponse>, (StatusCode, String)> {
    println!("Called create_session_id from API.");
    //Try to match on uuid. if it's not a valid uuid, treat it as a regular id.
    //Edge case: someone names their LLM a uuid.
    match Uuid::parse_str(&payload.llm_id) {
        Ok(uuid) => {
            create_session_flex(
                state,
                Json(CreateSessionFlexRequest {
                    user_id: payload.user_id,
                    api_key: payload.api_key,
                    filter: Some(LLMFilter {
                        llm_uuid: Some(uuid),
                        llm_id: None,
                        family_id: None,
                        minimum_capabilities: None,
                        local: None,
                    }),
                    preference: None,
                    user_session_parameters: payload.user_session_parameters,
                }),
            )
            .await
        }
        Err(err) => {
            create_session_flex(
                state,
                Json(CreateSessionFlexRequest {
                    user_id: payload.user_id,
                    api_key: payload.api_key,
                    filter: Some(LLMFilter {
                        llm_id: Some(payload.llm_id),
                        llm_uuid: None,
                        family_id: None,
                        minimum_capabilities: None,
                        local: None,
                    }),
                    preference: None,
                    user_session_parameters: payload.user_session_parameters,
                }),
            )
            .await
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct CreateSessionFlexRequest {
    user_id: String,
    api_key: String,
    filter: Option<LLMFilter>,
    preference: Option<LLMPreference>,
    user_session_parameters: HashMap<String, Value>,
}

#[axum_macros::debug_handler]
async fn create_session_flex(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<CreateSessionFlexRequest>,
) -> Result<Json<CreateSessionResponse>, (StatusCode, String)> {
    println!("Called create_session_flex from API.");
    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let user = user_permission_check("session", payload.api_key, user_uuid, state.pool.clone())?;
    // We should use currently running LLMs.
    let mut llms: Vec<Uuid> = state
        .activated_llms
        .iter()
        .map(|pair| (pair.key()).clone())
        .collect();

    // let llms = llms.into_iter().filter(|llm| )

    if let Some(filter) = payload.filter {
        if let Some(llm_uuid_filter) = filter.llm_uuid {
            llms = llms
                .into_iter()
                .filter(|llm| {
                    state.activated_llms.get(llm).unwrap().value().llm.uuid.0 == llm_uuid_filter
                })
                .collect();
        }
        if let Some(llm_id_filter) = filter.llm_id {
            llms = llms
                .into_iter()
                .filter(|llm| {
                    state.activated_llms.get(llm).unwrap().value().llm.id == llm_id_filter
                })
                .collect();
        }
        if let Some(family_id_filter) = filter.family_id {
            llms = llms
                .into_iter()
                .filter(|llm| {
                    state.activated_llms.get(llm).unwrap().value().llm.family_id == family_id_filter
                })
                .collect();
        }

        if let Some(local_filter) = filter.local {
            llms = llms
                .into_iter()
                .filter(|llm| {
                    state.activated_llms.get(llm).unwrap().value().llm.local == local_filter
                })
                .collect();
        }

        if let Some(capabilities_filter) = filter.minimum_capabilities {
            for cap_fil in capabilities_filter.into_iter() {
                let capability_name = cap_fil.capability;
                let capability_min = cap_fil.value;
                llms = llms
                    .into_iter()
                    .filter(|llm| {
                        state
                            .activated_llms
                            .get(llm)
                            .unwrap()
                            .value()
                            .llm
                            .capabilities
                            .0
                            .get(&capability_name.to_string())
                            .is_some_and(|x| x.clone() >= capability_min.into())
                    })
                    .collect()
            }
        }
    }

    println!("Filtered LLMS: {:?}", llms);

    if llms.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            "No running LLM matching requirements.".into(),
        ));
    } else if llms.len() == 1 {
        return create_session_internal(
            state.clone(),
            user,
            state
                .activated_llms
                .get(&llms.pop().unwrap())
                .unwrap()
                .value(),
            payload.user_session_parameters,
        )
        .await;
    }

    let mut capability_pref = CapabilityType::General;
    if let Some(preference) = payload.preference {
        // uuid is a singular preference. if we find it, we go.
        if let Some(uuid_pref) = preference.llm_uuid {
            if let Some(found) = llms
                .iter()
                .find(|llm| state.activated_llms.get(llm).unwrap().value().llm.uuid.0 == uuid_pref)
            {
                return create_session_internal(
                    state.clone(),
                    user,
                    state.activated_llms.get(found).unwrap().value(),
                    payload.user_session_parameters,
                )
                .await;
            }
        }

        // id is a singular preference. if we find it, we go.
        if let Some(id_pref) = preference.llm_id {
            if let Some(found) = llms
                .iter()
                .find(|llm| state.activated_llms.get(llm).unwrap().value().llm.id == id_pref)
            {
                return create_session_internal(
                    state.clone(),
                    user,
                    state.activated_llms.get(found).unwrap().value(),
                    payload.user_session_parameters,
                )
                .await;
            }
        }

        if let Some(local_pref) = preference.local {
            let count = llms
                .iter()
                .filter(|llm| {
                    state.activated_llms.get(llm).unwrap().value().llm.local == local_pref
                })
                .count();
            if count > 0 {
                llms = llms
                    .into_iter()
                    .filter(|llm| {
                        state.activated_llms.get(llm).unwrap().value().llm.local == local_pref
                    })
                    .collect();
            }
        }

        if let Some(family_pref) = preference.family_id {
            let count = llms
                .iter()
                .filter(|llm| {
                    state.activated_llms.get(llm).unwrap().value().llm.family_id == family_pref
                })
                .count();
            if count > 0 {
                llms = llms
                    .into_iter()
                    .filter(|llm| {
                        state.activated_llms.get(llm).unwrap().value().llm.family_id == family_pref
                    })
                    .collect();
            }
        }

        if let Some(cap_pref) = preference.capability_type {
            capability_pref = cap_pref;
        }
    }

    // Finally we select the most capability model, based on their preference
    // or in general

    llms.sort_by(|a, b| {
        state
            .activated_llms
            .get(a)
            .unwrap()
            .value()
            .llm
            .capabilities
            .get(&capability_pref.to_string())
            .unwrap_or(&-1)
            .cmp(
                state
                    .activated_llms
                    .get(b)
                    .unwrap()
                    .value()
                    .llm
                    .capabilities
                    .get(&capability_pref.to_string())
                    .unwrap_or(&-1),
            )
    });

    if llms.is_empty() {
        println!("Major malfunction, LLMs empty should be impossible here.");
        //fail gracefully
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failure in sorting code, please contact support".into(),
        ));
    }
    create_session_internal(
        state.clone(),
        user,
        state
            .activated_llms
            .get(&llms.first().unwrap())
            .unwrap()
            .value(),
        payload.user_session_parameters,
    )
    .await

    //Preference, in order,
    //first llm
    //
    //second local
    //
    //third family
    //
    //
    //finally capability.
    //if no capability! general capability.
}

#[derive(Debug, serde::Serialize)]
pub struct CreateSessionResponse {
    pub session_parameters: HashMap<String, Value>,
    pub llm_status: LLMStatus,
    pub session_id: String,
}
async fn create_session_internal(
    state: State<state::GlobalStateWrapper>,
    user: user::User,
    llm: &LLMActivated,
    user_session_parameters: HashMap<String, Value>,
) -> Result<Json<CreateSessionResponse>, (StatusCode, String)> {
    println!("Called create_session_internal from API.");
    match llm.create_session(user_session_parameters, user).await {
        Ok(resp) => Ok(Json(CreateSessionResponse {
            session_parameters: resp.session_parameters,
            llm_status: llm.into(),
            session_id: resp.session_id.to_string(),
            // llm_info: llm.llm.as_ref().into(),
        })),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error creating session.".into(),
        )),
    }
}
#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct PromptSessionStreamRequest {
    user_id: String,
    api_key: String,
    session_id: String,
    llm_uuid: String,
    prompt: String,
    parameters: HashMap<String, Value>,
}

#[axum_macros::debug_handler]
async fn prompt_session_stream(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<PromptSessionStreamRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, serde_json::Error>>>, (StatusCode, String)> {
    println!("Called prompt_session_stream from API.");
    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let llm_uuid =
        Uuid::parse_str(&payload.llm_uuid).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let session_id = Uuid::parse_str(&payload.session_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let user = user_permission_check("session", payload.api_key, user_uuid, state.pool.clone())?;

    if let Some(llm) = state.activated_llms.get(&llm_uuid) {
        match llm
            .value()
            .prompt_session(session_id, payload.prompt, payload.parameters, user)
            .await
        {
            Ok(prompt_response) => {
                let receiver_stream = ReceiverStream::new(prompt_response.stream);

                let event_stream =
                    receiver_stream.map(|llm_event| Event::default().json_data(llm_event));

                let stream = Sse::new(event_stream).keep_alive(KeepAlive::default());
                Ok(stream)
            }
            Err(err) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".into(),
            )),
        }
    } else {
        Err((
            StatusCode::NOT_FOUND,
            format!(
                "LLM with UUID {} not currently active. Either load it, or create a new session.",
                llm_uuid
            ),
        ))
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct BareModelFlexRequest {
    user_id: String,
    api_key: String,
    filter: Option<LLMFilter>,
    preference: Option<LLMPreference>,
}

#[derive(Debug, serde::Serialize)]
struct BareModelResponse {
    model: LLMStatus,
    path: String,

}
async fn bare_model_flex(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<BareModelFlexRequest>,
) -> Result<Json<BareModelResponse>, (StatusCode, String)> {
    println!("Called bare_mode_flex from API.");
    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let user = user_permission_check("bare_model", payload.api_key, user_uuid, state.pool.clone())?;
    let mut llms = database::get_available_llms(state.pool.clone())
        .map_err(|err| {
            println!("Failed to database: {:?}", err.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".into())})?;

    llms = llms.into_iter().filter(|llm| llm.model_path.is_some()).collect();
    // let mut llms: Vec<Uuid> = state
    //     .activated_llms
    //     .iter()
    //     .map(|pair| (pair.key()).clone())
    //     .collect();

    // let llms = llms.into_iter().filter(|llm| )

    if let Some(filter) = payload.filter {
        if let Some(llm_uuid_filter) = filter.llm_uuid {
            llms = llms
                .into_iter()
                .filter(|llm| llm.uuid.0 == llm_uuid_filter)
                .collect();
        }
        if let Some(llm_id_filter) = filter.llm_id {
            llms = llms
                .into_iter()
                .filter(|llm| llm.id == llm_id_filter)
                .collect();
        }
        if let Some(family_id_filter) = filter.family_id {
            llms = llms
                .into_iter()
                .filter(|llm| llm.family_id == family_id_filter)
                .collect();
        }

        if let Some(local_filter) = filter.local {
            llms = llms
                .into_iter()
                .filter(|llm| llm.local == local_filter)
                .collect();
        }

        if let Some(capabilities_filter) = filter.minimum_capabilities {
            for cap_fil in capabilities_filter.into_iter() {
                let capability_name = cap_fil.capability;
                let capability_min = cap_fil.value;
                llms = llms
                    .into_iter()
                    .filter(|llm| {
                        llm.capabilities
                            .0
                            .get(&capability_name.to_string())
                            .is_some_and(|x| x.clone() >= capability_min.into())
                    })
                    .collect()
            }
        }
    }

    println!("Filtered LLMS: {:?}", llms);

    if llms.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            "No running LLM matching requirements.".into(),
        ));
    } else if llms.len() == 1 {
        let llm = llms.pop().unwrap();
        let resp = BareModelResponse {
            model: (&llm).into(),
            path: llm.model_path.0.clone().unwrap().into_os_string().into_string().map_err(|osstr|
                (StatusCode::INTERNAL_SERVER_ERROR, "Path Error".into())
            )?
        };
        return Ok(Json(resp));
    }

    let mut capability_pref = CapabilityType::General;
    if let Some(preference) = payload.preference {
        // uuid is a singular preference. if we find it, we go.
        if let Some(uuid_pref) = preference.llm_uuid {
            if let Some(found) = llms.iter().find(|llm| llm.uuid.0 == uuid_pref) {
        let llm = llms.pop().unwrap();
        let resp = BareModelResponse {
            model: (&llm).into(),
            path: llm.model_path.0.unwrap().into_os_string().into_string().map_err(|osstr|
                (StatusCode::INTERNAL_SERVER_ERROR, "Path Error".into())
            )?
        };
        return Ok(Json(resp));
            }
        }

        // id is a singular preference. if we find it, we go.
        if let Some(id_pref) = preference.llm_id {
            if let Some(found) = llms.iter().find(|llm| llm.id == id_pref) {
        let llm = llms.pop().unwrap();
        let resp = BareModelResponse {
            model: (&llm).into(),
            path: llm.model_path.0.clone().unwrap().into_os_string().into_string().map_err(|osstr|
                (StatusCode::INTERNAL_SERVER_ERROR, "Path Error".into())
            )?
        };
        return Ok(Json(resp));
            }
        }

        if let Some(local_pref) = preference.local {
            let count = llms.iter().filter(|llm| llm.local == local_pref).count();
            if count > 0 {
                llms = llms
                    .into_iter()
                    .filter(|llm| llm.local == local_pref)
                    .collect();
            }
        }

        if let Some(family_pref) = preference.family_id {
            let count = llms
                .iter()
                .filter(|llm| llm.family_id == family_pref)
                .count();
            if count > 0 {
                llms = llms
                    .into_iter()
                    .filter(|llm| llm.family_id == family_pref)
                    .collect();
            }
        }

        if let Some(cap_pref) = preference.capability_type {
            capability_pref = cap_pref;
        }
    }
    llms.sort_by(|a, b| {
        a.capabilities
            .get(&capability_pref.to_string())
            .unwrap_or(&-1)
            .cmp(
                b.capabilities
                    .get(&capability_pref.to_string())
                    .unwrap_or(&-1),
            )
    });

    if llms.is_empty() {
        println!("Major malfunction, LLMs empty should be impossible here.");
        //fail gracefully
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failure in sorting code, please contact support".into(),
        ));
    }
        let llm = llms.pop().unwrap();
        let resp = BareModelResponse {
            model: (&llm).into(),
            path: llm.model_path.0.clone().unwrap().into_os_string().into_string().map_err(|osstr|
                (StatusCode::INTERNAL_SERVER_ERROR, "Path Error".into())
            )?
        };
        return Ok(Json(resp));

    // let user = state
    //     .registered_users
    //     .get(&user_uuid)
    //     .ok_or((StatusCode::UNAUTHORIZED, "Invalid User".into()))?;
    // user_permission_check("", api_key, &user.value())?;
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct BareModelRequest {
    user_id: String,
    api_key: String,
    llm_id: String,
}
#[axum_macros::debug_handler]
async fn bare_model(
    state: State<state::GlobalStateWrapper>,
    Json(payload): Json<BareModelRequest>,
) -> Result<Json<BareModelResponse>, (StatusCode, String)> {
    println!("Called load_llm from API.");
    let user_uuid =
        Uuid::parse_str(&payload.user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let llm_uuid =
        Uuid::parse_str(&payload.llm_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let _user = user_permission_check("bare_model", payload.api_key, user_uuid, state.pool.clone())?;

    let llm = database::get_llm(llm_uuid, state.pool.clone()).map_err(|err|
            (StatusCode::NOT_FOUND, "Unable to find LLM".into())
    )?;
    let resp = BareModelResponse {
        model: (&llm).into(),
            path: llm.model_path.0.clone().unwrap().into_os_string().into_string().map_err(|osstr|
                (StatusCode::INTERNAL_SERVER_ERROR, "Path Error".into())
            )?
    };
    return Ok(Json(resp))
}


pub async fn build_server(
    global_state: state::GlobalStateWrapper,
    rx: oneshot::Receiver<()>,
) -> Result<(), String> {
    // Define your API routes
    fn routes(state: state::GlobalStateWrapper) -> Router {
        Router::new()
            .route("/register_user", post(register_user))
            .route("/request_permissions", post(request_permissions))
            .route("/request_download", post(request_download))
            .route("/request_load", post(request_load))
            .route("/request_unload", post(request_unload))
            .route("/request_load_flex", post(request_load_flex))
            .route("/get_request_status", post(request_status))
            .route("/get_llm_status", post(get_llm_status))
            .route("/get_available_llms", post(get_available_llms))
            .route("/get_running_llms", post(get_running_llms))
            .route("/interrupt_session", post(interrupt_session))
            // .route("/load_session_id", post(load_session_id))
            .route("/load_llm", post(load_llm))
            .route("/load_llm_flex", post(load_llm_flex))
            .route("/unload_llm", post(unload_llm))
            .route("/download_llm", post(download_llm))
            .route("/create_session", post(create_session))
            .route("/create_session_id", post(create_session_id))
            .route("/create_session_flex", post(create_session_flex))
            .route("/prompt_session_stream", post(prompt_session_stream))
            .route("/bare_model", post(bare_model))
            .route("/bare_model_flex", post(bare_model_flex))
            .with_state(state)
    }
    let app = routes(global_state);

    create_listeners(app, rx).await
}
