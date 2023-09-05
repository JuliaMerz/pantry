use crate::database_types::DbUuid;
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use diesel::prelude::*;

use rand::Rng;
use uuid::uuid;
use uuid::Uuid;

const CUSTOM_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Permissions {
    // We flatten these in here for easier DB storage.
    pub perm_superuser: bool,
    pub perm_load_llm: bool,
    pub perm_unload_llm: bool,
    pub perm_download_llm: bool,
    pub perm_session: bool, //this is for create_sessioon AND prompt_session
    pub perm_request_download: bool,
    pub perm_request_load: bool,
    pub perm_request_unload: bool,
    pub perm_view_llms: bool,
    pub perm_bare_model: bool,
}

impl From<&User> for Permissions {
    fn from(user: &User) -> Self {
        Permissions {
            perm_superuser: user.perm_superuser.clone(),
            perm_load_llm: user.perm_load_llm.clone(),
            perm_unload_llm: user.perm_unload_llm.clone(),
            perm_download_llm: user.perm_download_llm.clone(),
            perm_session: user.perm_session.clone(),
            perm_request_download: user.perm_request_download.clone(),
            perm_request_load: user.perm_request_load.clone(),
            perm_request_unload: user.perm_request_unload.clone(),
            perm_view_llms: user.perm_view_llms.clone(),
            perm_bare_model: user.perm_bare_model.clone(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::user)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: DbUuid,
    pub name: String, //self identified, insecure
    pub api_key: String,

    // We flatten these in here for easier DB storage.
    pub perm_superuser: bool,
    pub perm_load_llm: bool,
    pub perm_unload_llm: bool,
    pub perm_download_llm: bool,
    pub perm_session: bool, //this is for create_sessioon AND prompt_session
    pub perm_request_download: bool,
    pub perm_request_load: bool,
    pub perm_request_unload: bool,
    pub perm_view_llms: bool,
    pub perm_bare_model: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub name: String, //self identified, insecure
    pub api_key: String,

    pub perm_superuser: bool,
    pub perm_load_llm: bool,
    pub perm_unload_llm: bool,
    pub perm_download_llm: bool,
    pub perm_session: bool, //this is for create_sessioon AND prompt_session
    pub perm_request_download: bool,
    pub perm_request_load: bool,
    pub perm_request_unload: bool,
    pub perm_view_llms: bool,
    pub perm_bare_model: bool,
}

impl From<&User> for UserInfo {
    fn from(user: &User) -> Self {
        UserInfo {
            id: user.id.0.clone(),
            name: user.name.clone(),
            api_key: user.api_key.clone(),

            perm_superuser: user.perm_superuser.clone(),
            perm_load_llm: user.perm_load_llm.clone(),
            perm_unload_llm: user.perm_unload_llm.clone(),
            perm_download_llm: user.perm_download_llm.clone(),
            perm_session: user.perm_session.clone(), //this is for create_sessioon AND prompt_session
            perm_request_download: user.perm_request_download.clone(),
            perm_request_load: user.perm_request_load.clone(),
            perm_request_unload: user.perm_request_unload.clone(),
            perm_view_llms: user.perm_view_llms.clone(),
            perm_bare_model: user.perm_bare_model.clone(),
        }
    }
}

impl User {
    pub fn new(name: String) -> User {
        User {
            id: DbUuid(Uuid::new_v4()),
            name: name,
            api_key: generate_api_key(),
            perm_superuser: false,
            perm_load_llm: false,
            perm_unload_llm: false,
            perm_download_llm: false,
            perm_session: false, //this is for create_sessioon AND prompt_session
            perm_request_download: false,
            perm_request_load: false,
            perm_request_unload: false,
            perm_view_llms: false,
            perm_bare_model: false,
        }
    }
}

// The first time the user gets generated, this API key is in clear text.
// It gets hashed into the DB, then hashed every time it gets checked in the API layer
// before being compared to the saved DB value.
pub fn generate_api_key() -> String {
    let mut rng = rand::thread_rng();
    let key: [u8; 32] = rng.gen();
    CUSTOM_ENGINE.encode(&key)
}

pub fn get_local_user() -> User {
    User {
        id: DbUuid(uuid!("00000000-0000-0000-0000-000000000000")),
        name: "local".into(),
        api_key: "".into(), //This isn't important because local calls skip the user auth layer
        perm_superuser: true,
        perm_load_llm: false,
        perm_unload_llm: false,
        perm_download_llm: false,
        perm_session: false, //this is for create_sessioon AND prompt_session
        perm_request_download: false,
        perm_request_load: false,
        perm_request_unload: false,
        perm_view_llms: false,
        perm_bare_model: false,
    }
}
