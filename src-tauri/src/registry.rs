use std::collections::HashMap;
use std::io::prelude::*;


use crate::database;
use crate::database_types::*;
use crate::emitter;
use crate::llm;
use crate::state;
use futures_util::StreamExt;
use serde_json::Value;
use std::fs::File;
use std::str::FromStr;
use tauri::{Manager};

use uuid::Uuid;

// connectors/registry.rs

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LLMRegistryEntryConnector {
    Ggml,
    LLMrs,
    OpenAI,
    GenericAPI,
}

impl FromStr for LLMRegistryEntryConnector {
    type Err = ();

    fn from_str(input: &str) -> Result<LLMRegistryEntryConnector, Self::Err> {
        match input {
            "ggml" => Ok(LLMRegistryEntryConnector::Ggml),
            "llmrs" => Ok(LLMRegistryEntryConnector::LLMrs),
            "api" => Ok(LLMRegistryEntryConnector::GenericAPI),
            "openai" => Ok(LLMRegistryEntryConnector::OpenAI),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, serde::Deserialize)]
pub enum LLMRegistryEntryInstallStep {
    Download,
}

impl FromStr for LLMRegistryEntryInstallStep {
    type Err = ();

    fn from_str(input: &str) -> Result<LLMRegistryEntryInstallStep, Self::Err> {
        match input {
            "download" => Ok(LLMRegistryEntryInstallStep::Download),
            _ => Err(()),
        }
    }
}

//We don't store these locally.
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct LLMRegistryEntry {
    pub id: String,
    pub family_id: String,
    pub organization: String,

    pub name: String,
    pub license: String,
    pub description: String,
    pub homepage: String,

    pub capabilities: HashMap<String, isize>,
    pub tags: Vec<String>,
    pub requirements: String,

    pub backend_uuid: String,
    pub url: String,

    pub config: HashMap<String, Value>,
    pub create_thread: bool,
    pub connector_type: LLMRegistryEntryConnector,

    pub parameters: HashMap<String, Value>,
    pub user_parameters: Vec<String>,

    pub session_parameters: HashMap<String, Value>,
    pub user_session_parameters: Vec<String>,
}

pub async fn download_and_write_llm(
    llm_reg: LLMRegistryEntry,
    uuid: Uuid,
    app: tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create the request client.
    let client = reqwest::Client::new();

    let mut path = app
        .path_resolver()
        .app_local_data_dir()
        .ok_or(format!("failed to find local data"))?;
    path.push(format!("{}-{}", llm_reg.id, uuid.to_string()));

    // Create the file to write into.
    let mut file = File::create(&path)?;

    // Create the request.
    let response = client.get(llm_reg.url.clone()).send().await?;

    // Get the total size if available.
    let total_size_opt = response.content_length();

    let mut downloaded: u64 = 0;

    let mut stream = response.bytes_stream();

    let stream_id = format!("{}-{}", llm_reg.id, uuid.to_string());

    //TODO: download progress for specific downloads
    let mut update_counter = 0;
    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        update_counter += 1;
        if update_counter % 1000 != 1 {
            continue;
        }

        // If the total size of the object is known, calculate the percentage.
        if let Some(total_size) = total_size_opt {
            let percent = (downloaded as f32 / total_size as f32) * 100.0;
            println!("Downloading {} at {}", llm_reg.id, percent);
            app.emit_all(
                "downloads",
                emitter::EventPayload {
                    stream_id: stream_id.clone(),
                    event: emitter::EventType::DownloadProgress {
                        progress: percent.to_string(),
                    },
                },
            )?;
        } else {
            println!("Downloading {} at {}", llm_reg.id, downloaded);
            // otherwise, just emit the downloaded amount.
            app.emit_all(
                "downloads",
                emitter::EventPayload {
                    stream_id: stream_id.clone(),
                    event: emitter::EventType::DownloadProgress {
                        progress: downloaded.to_string(),
                    },
                },
            )?;
        }
    }

    let state: tauri::State<'_, state::GlobalState> = app.state();

    let new_llm: llm::LLM = llm::LLM {
        id: llm_reg.id.clone(),
        family_id: llm_reg.family_id.clone(),
        organization: llm_reg.organization.clone(),
        name: llm_reg.name.clone(),
        license: llm_reg.license.clone(),
        description: llm_reg.description.clone(),
        downloaded_reason: "some kind of user input".into(), //TODO: make this dynamic at some point
        downloaded_date: chrono::offset::Utc::now(),
        last_called: None, // clone inner value
        url: llm_reg.url.clone(),
        homepage: llm_reg.homepage.clone(),

        uuid: DbUuid(uuid),

        capabilities: DbHashMapInt(llm_reg.capabilities.clone()),
        tags: DbVec(llm_reg.tags.clone()),

        requirements: llm_reg.requirements.clone(),

        create_thread: llm_reg.create_thread.clone(),
        connector_type: llm_reg.connector_type.clone().into(), // assuming this type is also Clone
        config: DbHashMap(llm_reg.config.clone()),
        parameters: DbHashMap(llm_reg.parameters.clone()),
        user_parameters: DbVec(llm_reg.user_parameters.clone()),
        session_parameters: DbHashMap(llm_reg.session_parameters.clone()),
        user_session_parameters: DbVec(llm_reg.user_session_parameters.clone()),
        model_path: DbOptionPathbuf(Some(path.clone())),
    };

    match database::save_new_llm(new_llm, state.pool.clone()) {
        Ok(_) => {
            println!("Successful download, llms serialized");
            app.emit_all(
                "downloads",
                emitter::EventPayload {
                    stream_id: stream_id.clone(),
                    event: emitter::EventType::DownloadCompletion {},
                },
            )?;
        }
        Err(_) => {
            println!("Failed to save download");
            app.emit_all(
                "downloads",
                emitter::EventPayload {
                    stream_id: stream_id.clone(),
                    event: emitter::EventType::DownloadError {
                        message: "failed to save llm".into(),
                    },
                },
            )?;
        }
    }
    app.emit_all(
        "downloads",
        emitter::EventPayload {
            stream_id: stream_id.clone(),
            event: emitter::EventType::ChannelClose {},
        },
    )?;
    Ok(())
}
