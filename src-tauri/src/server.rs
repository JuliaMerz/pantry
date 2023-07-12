//server.rs
use crate::database;

use crate::llm::{LLMActivated, LLM};
use crate::registry;
use crate::request::{UserRequest, UserRequestType};
use crate::schema;
use crate::state;
use crate::user;
use axum::{extract::State, Json};
use chrono::DateTime;
use chrono::Utc;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{prelude::*};
use hyper::StatusCode;

use serde_json::Value;
use std::collections::HashMap;

use uuid::Uuid;

// We're deliberately disconnecting these datastructures from frontend.rs
// because the frontend is a superuser panel, and this is a regular API.
struct LLMStatus {
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

    pub local: bool, //rename from create_thread,
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
            local: llm.create_thread.clone(),
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
            local: llm.llm.create_thread.clone(),
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

pub struct LLMRequestStatus {
    pub id: Uuid,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub request: UserRequestType,
}
impl From<&UserRequest> for LLMRequestStatus {
    fn from(llm_req: &UserRequest) -> Self {
        LLMRequestStatus {
            id: llm_req.id.0.clone(),
            user_id: llm_req.user_id.0.clone(),
            timestamp: llm_req.timestamp.clone(),
            request: llm_req.request.clone(),
        }
    }
}

fn user_permission_check(
    required: &str,
    api_key: String,
    // user: &user::User,
    user_id: Uuid,
    pool: Pool<ConnectionManager<SqliteConnection>>,
) -> Result<(), (StatusCode, String)> {
    let user = database::get_user(user_id, pool)
        .map_err(|_err| (StatusCode::UNAUTHORIZED, "Not a Valid User {:?}".into()))?;
    if user.api_key != api_key {
        return Err((StatusCode::UNAUTHORIZED, "Incorrect API Key".into()));
    };
    if user.perm_superuser.clone() {
        return Ok(());
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
        &_ => false,
    };
    match auth {
        true => Ok(()),
        false => Err((StatusCode::UNAUTHORIZED, "Incorrect Permissions".into())),
    }
}

async fn hello_world() -> Json<Value> {
    Json("Hello, World!".into())
}

fn register_user(
    _state: State<state::GlobalStateWrapper>,
    user_name: String,
) -> Json<user::UserInfo> {
    let user = user::User::new(user_name);
    diesel::insert_into(schema::user::table).values(&user);
    Json((&user).into())
}

fn request_permissions(
    state: State<state::GlobalStateWrapper>,
    user_id: String,
    api_key: String,
    _requested_permissions: user::Permissions,
) -> Result<Json<Value>, (StatusCode, String)> {
    let user_uuid =
        Uuid::parse_str(&user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let _user = user_permission_check("", api_key, user_uuid, state.pool.clone());
    // state
    // .registered_users
    // .get(&user_uuid)
    // .ok_or((StatusCode::UNAUTHORIZED, "Invalid User".into()))?;

    // let request = LLMRequest {
    //     id: Uuid::new_v4(),
    //     reason: "reason".into(),
    //     timestamp: chrono::Utc::now(),
    //     user_id: user.id,
    //     request: LLMRequestType::PermissionRequest(request::PermissionRequest {
    //         requested_permissions,
    //     }),
    // };
    // state.requests.insert(request.id, request);
    // request::serialize_all(
    //     state.user_settings.read().unwrap().settings_path.clone(),
    //     state.requests.clone(),
    // )
    // .unwrap();
    todo!()
    // Ok(Json(request.id.to_string()))
}

fn request_download(
    _state: State<state::GlobalStateWrapper>,
    user_id: String,
    _api_key: String,
    _llm_registry_entry: registry::LLMRegistryEntry,
) -> Result<Json<Value>, (StatusCode, String)> {
    let _user_uuid =
        Uuid::parse_str(&user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    // let user = state
    //     .registered_users
    //     .get(&user_uuid)
    //     .ok_or((StatusCode::UNAUTHORIZED, "Invalid User".into()))?;
    // user_permission_check("request_download", api_key, &user.value())?;

    // let request = LLMRequest {
    //     id: DbUuid(Uuid::new_v4()),
    //     timestamp: chrono::Utc::now(),
    //     reason: "reason".into(),
    //     user_id: user.id,
    //     request: LLMRequestType::DownloadRequest(request::DownloadRequest { llm_registry_entry }),
    // };
    // state.requests.insert(request.id, request);
    // request::serialize_all(
    //     state.user_settings.read().unwrap().settings_path.clone(),
    //     state.requests.clone(),
    // )
    // .unwrap();
    // request.id.to_string()
    todo!()
}

fn request_load(
    _state: State<state::GlobalStateWrapper>,
    user_id: String,
    _api_key: String,
    _llm_id: String,
) -> Result<Json<Value>, (StatusCode, String)> {
    let _user_uuid =
        Uuid::parse_str(&user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    // let user = state
    //     .registered_users
    //     .get(&user_uuid)
    //     .ok_or((StatusCode::UNAUTHORIZED, "Invalid User".into()))?;
    // user_permission_check("request_load", api_key, &user.value())?;

    // let request = LLMRequest {
    //     id: DbUuid(Uuid::new_v4()),
    //     reason: "reason".into(),
    //     timestamp: chrono::Utc::now(),
    //     user_id: DbUuid(user.id),
    //     request: LLMRequestType::LoadRequest(request::LoadRequest { llm_id }),
    // };
    // state.requests.insert(request.id, request);
    // request::serialize_all(
    //     state.user_settings.read().unwrap().settings_path.clone(),
    //     state.requests.clone(),
    // )
    // .unwrap();
    // request.id.to_string()
    todo!()
}

fn request_unload(
    _state: State<state::GlobalStateWrapper>,
    user_id: String,
    _api_key: String,
    _llm_id: String,
) -> Result<Json<Value>, (StatusCode, String)> {
    let _user_uuid =
        Uuid::parse_str(&user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    // let user = state
    //     .registered_users
    //     .get(&user_uuid)
    //     .ok_or((StatusCode::UNAUTHORIZED, "Invalid User".into()))?;
    // user_permission_check("request_unload", api_key, &user.value())?;

    // let request = LLMRequest {
    //     id: DbUuid(Uuid::new_v4()),
    //     reason: "reason".into(),
    //     timestamp: chrono::Utc::now(),
    //     user_id: DbUuid(user.id),
    //     request: LLMRequestType::UnloadRequest(request::UnloadRequest { llm_id }),
    // };
    // state.requests.insert(request.id, request);
    // request::serialize_all(
    //     state.user_settings.read().unwrap().settings_path.clone(),
    //     state.requests.clone(),
    // )
    // .unwrap();
    // request.id.to_string()
    todo!()
}

async fn load_llm(
    state: State<state::GlobalStateWrapper>,
    user_id: String,
    _api_key: String,
    llm_id: String,
) -> Result<Json<Value>, (StatusCode, String)> {
    let _user_uuid =
        Uuid::parse_str(&user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    // let user = state
    //     .registered_users
    //     .get(&user_uuid)
    //     .ok_or((StatusCode::UNAUTHORIZED, "Invalid User".into()))?;
    // user_permission_check("", api_key, &user.value())?;

    let llm_uuid =
        Uuid::parse_str(&llm_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    if state.activated_llms.contains_key(&llm_uuid) {
        return Err((StatusCode::OK, "LLM Already Activated".into()));
    }

    let _manager_addr_copy = state.manager_addr.clone();

    todo!()

    // if let Some(new_llm) = state.available_llms.get(&uuid) {
    //     let path = app
    //         .path_resolver()
    //         .app_local_data_dir()
    //         .ok_or("no path no llms")?;
    //     let settings = state.user_settings.read().unwrap().clone();
    //     let result = llm::LLMActivated::activate_llm(
    //         new_llm.value().clone(),
    //         manager_addr_copy,
    //         path,
    //         settings,
    //     )
    //     .await;
    //     // new_llm.load();
    //     match result {
    //         Ok(running) => {
    //             println!("Inserting {uuid} into running LLMs");
    //             state.activated_llms.insert(uuid, running);
    //             Ok(())
    //         }
    //         Err(err) => Err("failed to launch {id} skipping".into()),
    //     }
    // } else {
    //     Err("couldn't find matching llm".into())
    // }

    // // Perform the necessary operations to load the LLM

    // format!("LLM with ID '{}' loaded successfully", llm_id)
}

fn unload_llm(
    _state: State<state::GlobalStateWrapper>,
    user_id: String,
    _api_key: String,
    _llm_id: String,
) -> Result<Json<Value>, (StatusCode, String)> {
    let _user_uuid =
        Uuid::parse_str(&user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    // let user = state
    //     .registered_users
    //     .get(&user_uuid)
    //     .ok_or((StatusCode::UNAUTHORIZED, "Invalid User".into()))?;
    // user_permission_check("unload_llm", api_key, &user.value())?;

    // let llm = match state.available_llms.get(&llm_id) {
    //     Some(llm) => llm.clone(),
    //     None => return format!("LLM with ID '{}' not found", llm_id),
    // };

    // Perform the necessary operations to load the LLM

    // format!("LLM with ID '{}' loaded successfully", llm_id) todo!()
    todo!()
}

fn download_llm(
    _state: State<state::GlobalStateWrapper>,
    user_id: String,
    _api_key: String,
    _llm_id: String,
) -> Result<Json<Value>, (StatusCode, String)> {
    let _user_uuid =
        Uuid::parse_str(&user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    // let user = state
    //     .registered_users
    //     .get(&user_uuid)
    //     .ok_or((StatusCode::UNAUTHORIZED, "Invalid User".into()))?;
    // user_permission_check("", api_key, &user.value())?;

    // let llm = match state.available_llms.get(&llm_id) {
    //     Some(llm) => llm.clone(),
    //     None => return format!("LLM with ID '{}' not found", llm_id),
    // };

    // Perform the necessary operations to load the LLM
    //
    todo!()

    // format!("LLM with ID '{}' loaded successfully", llm_id)
}

fn create_session(
    _state: State<state::GlobalStateWrapper>,
    user_id: String,
    _api_key: String,
    _llm_id: String,
) -> Result<Json<Value>, (StatusCode, String)> {
    let _user_uuid =
        Uuid::parse_str(&user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    // let user = state
    //     .registered_users
    //     .get(&user_uuid)
    //     .ok_or((StatusCode::UNAUTHORIZED, "Invalid User".into()))?;
    // user_permission_check("", api_key, &user.value())?;

    todo!()
}

fn create_session_id(
    _state: State<state::GlobalStateWrapper>,
    user_id: String,
    _api_key: String,
    _llm_id: String,
) -> Result<Json<Value>, (StatusCode, String)> {
    let _user_uuid =
        Uuid::parse_str(&user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    // let user = state
    //     .registered_users
    //     .get(&user_uuid)
    //     .ok_or((StatusCode::UNAUTHORIZED, "Invalid User".into()))?;
    // user_permission_check("", api_key, &user.value())?;
    todo!()
}

struct CapabilityFilter {
    capability: String,
    value: isize,
}

struct LLMFilter {
    llm: Option<String>,
    family_id: Option<String>,
    minimum_capabilities: Option<CapabilityFilter>,
}

struct LLMPreference {
    llm: Option<String>,
    family_id: Option<String>,
    capability_type: Option<String>,
}

fn create_session_flex(
    _state: State<state::GlobalStateWrapper>,
    user_id: String,
    _api_key: String,
    _filter: LLMFilter,
    _preference: LLMPreference,
) -> Result<Json<Value>, (StatusCode, String)> {
    let _user_uuid =
        Uuid::parse_str(&user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    // let user = state
    //     .registered_users
    //     .get(&user_uuid)
    //     .ok_or((StatusCode::UNAUTHORIZED, "Invalid User".into()))?;
    // user_permission_check("", api_key, &user.value())?;

    todo!()
}

fn prompt_session(
    _state: State<state::GlobalStateWrapper>,
    user_id: String,
    _api_key: String,
    _session_id: String,
    _prompt: String,
) -> Result<Json<Value>, (StatusCode, String)> {
    let _user_uuid =
        Uuid::parse_str(&user_id).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    // let user = state
    //     .registered_users
    //     .get(&user_uuid)
    //     .ok_or((StatusCode::UNAUTHORIZED, "Invalid User".into()))?;
    // user_permission_check("", api_key, &user.value())?;
    todo!()
}

pub fn build_server(_global_state: state::GlobalStateWrapper) -> Result<(), String> {
    // Define your API routes
    // let app = Router::new(); //.with_state(global_state);

    // app.route("/", get(hello_world));

    // app.with_state(global_state);
    Ok(())
}
