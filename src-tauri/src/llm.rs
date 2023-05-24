use chrono::DateTime;
use chrono::serde::ts_seconds_option;
use chrono::Utc;
use actix::prelude::*;

use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;
use std::collections::HashMap;

// src/llm.rs

#[derive(serde::Serialize)]
pub struct LLMInfo {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(serde::Serialize)]
pub struct SerializableTS {
    #[serde(with = "ts_seconds_option")]
    pub time: Option<DateTime<Utc>>
}

#[derive(serde::Serialize)]
pub struct LLMRunning {
    pub llm_info: LLMInfo,
    pub downloaded: String,
    pub last_called: RwLock<SerializableTS>,
    pub activated: String,
    // #[serde(skip_serializing)]
    // pub llm: dyn LLMWrapper + Send + Sync

}

#[derive(serde::Serialize)]
pub struct LLMAvailable {
    pub llm_info: LLMInfo,
    pub downloaded: String,
    pub last_called: RwLock<SerializableTS>,
}

#[derive(serde::Serialize)]
pub struct LLMRequest {
    pub llm_info: LLMInfo,
    pub source: String, //For compatibility with the string based enum in typescript
    pub requester: String,
}

#[derive(serde::Serialize)]
pub struct LLMStatus {
    pub status: String
}


pub trait LLMActor: LLMWrapper {

}


pub trait LLMWrapper {
    fn get_info(&self) -> LLMStatus;
    fn load(&self) -> Result<(), &'static str>;
    fn unload(&self) -> Result<(), &'static str>;
    fn call(&self, message: &str, parameters_json: &str) -> Result<String, &'static str>;
    fn status(&self) -> LLMStatus;
}


