use crate::database_types::DbUuid;
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use dashmap::DashMap;
use diesel::prelude::*;
use rand::Rng;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
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
        }
    }
}
// pub fn serialize_all(
//     path: PathBuf,
//     users: DashMap<Uuid, User>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let user_iter = users.iter();

//     let user_vec: Vec<User> = user_iter.map(|val| (*(val.value())).clone()).collect();
//     let mut file = File::create(path)?;
//     rmp_serde::encode::write_named(&mut file, &user_vec)?;
//     // file.write_all(&encoded)?;
//     Ok(())
// }

// pub fn deserialize_all(path: PathBuf) -> Result<DashMap<Uuid, User>, Box<dyn std::error::Error>> {
//     let mut file = File::open(path)?;
//     let users: Vec<User> = rmp_serde::decode::from_read(&file)?;
//     let blank_map = DashMap::new();
//     users
//         .into_iter()
//         .map(|val| blank_map.insert(val.id.clone(), val));

//     // let mut buffer = Vec::new();
//     // file.read_to_end(&mut buffer)?;
//     // let llms: Vec<LLM> = rmp_serde::deserialize(&buffer)?;
//     Ok(blank_map)
// }

fn generate_api_key() -> String {
    let mut rng = rand::thread_rng();
    let key: [u8; 32] = rng.gen();
    CUSTOM_ENGINE.encode(&key)
}

pub fn get_local_user() -> User {
    User {
        id: DbUuid(uuid!("00000000-0000-0000-0000-000000000000")),
        name: "local".into(),
        api_key: "local".into(), //This isn't important because local calls skip the user auth layer
        perm_superuser: true,
        perm_load_llm: false,
        perm_unload_llm: false,
        perm_download_llm: false,
        perm_session: false, //this is for create_sessioon AND prompt_session
        perm_request_download: false,
        perm_request_load: false,
        perm_request_unload: false,
    }
}
