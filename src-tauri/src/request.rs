//request.rs
use crate::database_types::*;
use crate::registry;
use crate::state;
use crate::user;
use chrono::DateTime;
use chrono::Utc;
use dashmap::DashMap;
use diesel::deserialize::FromSql;
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sqlite::{Sqlite, SqliteValue};
use diesel::*;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, FromSqlRow, AsExpression)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct DownloadRequest {
    pub llm_registry_entry: registry::LLMRegistryEntry,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, FromSqlRow, AsExpression)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct PermissionRequest {
    pub requested_permissions: user::Permissions,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, FromSqlRow, AsExpression)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct LoadRequest {
    pub llm_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, FromSqlRow, AsExpression)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct UnloadRequest {
    pub llm_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, FromSqlRow, AsExpression)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum UserRequestType {
    DownloadRequest(DownloadRequest),
    PermissionRequest(PermissionRequest),
    LoadRequest(LoadRequest),
    UnloadRequest(UnloadRequest),
}

impl FromSql<diesel::sql_types::Text, Sqlite> for UserRequestType {
    fn from_sql(bytes: SqliteValue<'_, '_, '_>) -> diesel::deserialize::Result<Self> {
        let str = <String as FromSql<diesel::sql_types::Text, Sqlite>>::from_sql(bytes)?;
        let value: UserRequestType = serde_json::from_str(&str)?;
        Ok(value)
    }
}

impl ToSql<diesel::sql_types::Text, Sqlite> for UserRequestType {
    fn to_sql<'W>(&'W self, out: &mut Output<'W, '_, Sqlite>) -> serialize::Result {
        let str = serde_json::to_string(self)?;
        out.set_value(str);
        Ok(serialize::IsNull::No)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::llm_requests)]
#[diesel(belongs_to(user::User))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserRequest {
    pub id: DbUuid,
    pub user_id: DbUuid,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
    pub request: UserRequestType,
}

// pub fn serialize_all(
//     path: PathBuf,
//     requests: DashMap<Uuid, LLMRequest>,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let request_iter = requests.iter();

//     let request_vec: Vec<LLMRequest> = request_iter.map(|val| (*(val.value())).clone()).collect();
//     let mut file = File::create(path)?;
//     rmp_serde::encode::write_named(&mut file, &request_vec)?;
//     // file.write_all(&encoded)?;
//     Ok(())
// }

// pub fn deserialize_all(
//     path: PathBuf,
// ) -> Result<DashMap<Uuid, LLMRequest>, Box<dyn std::error::Error>> {
//     let mut file = File::open(path)?;
//     let requests: Vec<LLMRequest> = rmp_serde::decode::from_read(&file)?;
//     let blank_map = DashMap::new();
//     requests
//         .into_iter()
//         .map(|val| blank_map.insert(val.id.clone(), val));

//     // let mut buffer = Vec::new();
//     // file.read_to_end(&mut buffer)?;
//     // let llms: Vec<LLM> = rmp_serde::deserialize(&buffer)?;
//     Ok(blank_map)
// }
