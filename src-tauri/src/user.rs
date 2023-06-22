
use rand::Rng;
use uuid::Uuid;
use uuid::uuid;
use std::collections::HashMap;
use base64::{Engine as _, engine::{self, general_purpose}, alphabet};
const CUSTOM_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Permissions {
    roles: HashMap<String, bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String, //self identified, insecure
    pub api_key: String,
    pub permissions: Permissions,
}

impl User {
    pub fn new(name: String) -> User {
        User {
            id: Uuid::new_v4(),
            name: name,
            api_key: generate_api_key(),
            permissions: Permissions {
                roles: HashMap::new(),
            },
        }
    }
}

fn generate_api_key() -> String {
    let mut rng = rand::thread_rng();
    let key: [u8; 32] = rng.gen();
    CUSTOM_ENGINE.encode(&key)
}


pub fn get_local_user () -> User {
    User {
        id: uuid!("00000000-0000-0000-0000-000000000000"),
        name: "local".into(),
        api_key: "local".into(), //This isn't important because local calls skip the user auth layer
        permissions: Permissions {
            roles: HashMap::from([("super".into(), true)]),
        }

    }
}
