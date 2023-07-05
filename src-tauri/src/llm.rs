use crate::state;
use chrono::prelude::*;
use chrono::DateTime;
use chrono::Utc;
use dashmap::DashMap;
use rmp_serde;
use serde_json::json;
use serde_json::Value;
use tiny_tokio_actor::*;

use uuid::Uuid;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;

use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::connectors;
use crate::connectors::llm_actor;
use crate::connectors::llm_actor::LLMActor;
use crate::connectors::llm_manager;
use crate::error::PantryError;
use crate::frontend;
use crate::registry;
use crate::user;
use tokio::sync::mpsc;

use bincode;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

// src/llm.rs

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct HistoryItem {
    pub caller: String,
    pub request: String,
    pub response: String,
    pub timestamp: DateTime<Utc>,
}

// #[derive(serde::Serialize)]
pub struct CallLLMResponse {
    pub session_id: Uuid,
    pub session_parameters: HashMap<String, Value>, //Final parameters used.
    pub parameters: HashMap<String, Value>,         //Final parameters used.
    pub stream: mpsc::Receiver<connectors::LLMEvent>,
}

pub struct CreateSessionResponse {
    pub session_id: Uuid,
    pub session_parameters: HashMap<String, Value>,
}

pub struct PromptSessionResponse {
    pub stream: mpsc::Receiver<connectors::LLMEvent>,
    pub parameters: HashMap<String, Value>,
}
//Potentially unecessary wrapper class, written to give us space.
#[async_trait]
pub trait LLMWrapper {
    fn get_info(&self) -> frontend::LLMStatus;
    async fn status(&self) -> frontend::LLMStatus;
    async fn ping(&self) -> Result<String, String>;
    async fn reload(&self) -> Result<(), PantryError>;
    async fn get_sessions(&self, user: user::User) -> Result<Vec<LLMSession>, PantryError>;
    async fn create_session(
        &self,
        params: HashMap<String, Value>,
        user: user::User,
    ) -> Result<CreateSessionResponse, PantryError>;
    async fn prompt_session(
        &self,
        session_id: Uuid,
        prompt: String,
        parameters: HashMap<String, Value>,
        user: user::User,
    ) -> Result<PromptSessionResponse, PantryError>;
    async fn call_llm(
        &self,
        message: &str,
        session_parameters: HashMap<String, Value>,
        parameters: HashMap<String, Value>,
        user: user::User,
    ) -> Result<CallLLMResponse, PantryError>;
    async fn interrupt_session(
        &self,
        session_id: Uuid,
        user: user::User,
    ) -> Result<bool, PantryError>;
    fn into_llm_running(&self) -> frontend::LLMRunning;
}

// Implements LLMWrapper
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct LLM {
    // Machine Info
    pub id: String, // Maybe?: https://github.com/alexanderatallah/window.ai/blob/main/packages/lib/README.md
    pub family_id: String, // Whole sets of models: Example's are GPT, LLaMA
    pub organization: String, // May be "None"

    // Human Info
    pub name: String,
    pub homepage: String,
    pub description: String,
    pub license: String,

    // Fields only in LLM Available
    pub downloaded_reason: String,
    pub downloaded_date: DateTime<Utc>,
    pub last_called: RwLock<Option<DateTime<Utc>>>,

    // 0 is not capable, -1 is not evaluated.
    pub capabilities: HashMap<String, isize>,
    pub tags: Vec<String>,
    pub requirements: String,

    pub uuid: Uuid,
    pub url: String,

    pub history: Vec<HistoryItem>,

    // Functionality
    pub create_thread: bool, // Is it not an API connector?
    pub connector_type: connectors::LLMConnectorType, // which connector to use
    // Configs used by the connector for setup.
    pub config: HashMap<String, Value>, //Configs used by the connector
    pub model_path: Option<PathBuf>,

    // Parameters — these are submitted at call time.
    // these are the same, except one is configurable by users (programs or direct).
    // Hard coded parameters exist so repositories can deploy simple user friendly models
    // with preset configurations.
    pub parameters: HashMap<String, Value>, // Hardcoded Parameters
    pub user_parameters: Vec<String>,       //User Parameters

    //These are the same, but for whole sessions.
    //This is largely forward thinking, the only place we would implement
    //this now would be useGPU.
    //But we'll need ot eventually.
    pub session_parameters: HashMap<String, Value>, // Hardcoded Parameters
    pub user_session_parameters: Vec<String>,
}

pub struct LLMActivated {
    pub llm: Arc<LLM>,
    pub activated_reason: String,
    pub activated_time: DateTime<Utc>,
    // This is a map of session id to interrupt tokens. (session_id, user_id)
    actor: ActorRef<connectors::SysEvent, llm_actor::LLMActor>,
    pub interrupts: Arc<DashMap<(Uuid, Uuid), Vec<CancellationToken>>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct LLMHistoryItem {
    pub id: Uuid,
    pub updated_timestamp: DateTime<Utc>,
    pub call_timestamp: DateTime<Utc>,
    pub complete: bool,
    pub parameters: HashMap<String, Value>,
    pub input: String,
    pub output: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct LLMSession {
    pub id: Uuid, //this is a uuid
    pub started: DateTime<Utc>,
    pub last_called: DateTime<Utc>,
    pub user_id: Uuid,
    pub llm_uuid: Uuid,
    pub session_parameters: HashMap<String, Value>,
    pub items: Vec<LLMHistoryItem>,
}

impl Clone for LLM {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            family_id: self.family_id.clone(),
            organization: self.organization.clone(),
            name: self.name.clone(),
            license: self.license.clone(),
            description: self.description.clone(),
            downloaded_reason: self.downloaded_reason.clone(),
            downloaded_date: self.downloaded_date.clone(),
            last_called: RwLock::new(*self.last_called.read().unwrap()), // clone inner value

            url: self.url.clone(),
            homepage: self.homepage.clone(),

            uuid: self.uuid.clone(),

            capabilities: self.capabilities.clone(),
            tags: self.tags.clone(),
            history: self.history.clone(),

            requirements: self.requirements.clone(),

            create_thread: self.create_thread.clone(),
            connector_type: self.connector_type.clone(), // assuming this type is also Clone
            config: self.config.clone(),
            model_path: self.model_path.clone(),
            parameters: self.parameters.clone(),
            user_parameters: self.user_parameters.clone(),
            session_parameters: self.session_parameters.clone(),
            user_session_parameters: self.user_session_parameters.clone(),
        }
    }
}

pub fn serialize_llms(path: PathBuf, llms: &Vec<LLM>) -> Result<(), Box<dyn std::error::Error>> {
    // let encoded = rmp_serde::serialize(llms)?;
    let mut file = File::create(path)?;
    rmp_serde::encode::write_named(&mut file, llms)?;
    // file.write_all(&encoded)?;
    Ok(())
}

pub fn deserialize_llms(path: PathBuf) -> Result<Vec<LLM>, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let llms: Vec<LLM> = rmp_serde::decode::from_read(&file)?;
    // let mut buffer = Vec::new();
    // file.read_to_end(&mut buffer)?;
    // let llms: Vec<LLM> = rmp_serde::deserialize(&buffer)?;
    Ok(llms)
}

impl LLMActivated {
    pub async fn activate_llm(
        llm: Arc<LLM>,
        manager_addr: ActorRef<connectors::SysEvent, llm_manager::LLMManagerActor>,
        data_path: PathBuf,
        user_settings: state::UserSettings,
    ) -> Result<LLMActivated, PantryError> {
        match manager_addr
            .ask(llm_manager::CreateLLMActorMessage {
                id: llm.id.clone(),
                uuid: llm.uuid.clone(),
                connector: llm.connector_type.clone(),
                config: llm.config.clone(),
                data_path: data_path,
                model_path: llm.model_path.clone(),
                user_settings,
            })
            .await
        {
            Ok(result) => {
                match result {
                    // At this point we've created the LLM actor.
                    Ok(val) => Ok(LLMActivated {
                        llm: llm,
                        activated_reason: "User request".into(),
                        activated_time: chrono::offset::Utc::now(),
                        actor: val,
                        interrupts: Arc::new(DashMap::new()),
                    }),
                    Err(err) => Err(err),
                }
            }
            Err(err) => Err(PantryError::ActorFailure(err)),
        }
    }
}

#[async_trait]
impl LLMWrapper for LLMActivated {
    fn get_info(&self) -> frontend::LLMStatus {
        frontend::LLMStatus {
            status: format!(
                "ID: {}, Name: {}, Description: {}",
                self.llm.id, self.llm.name, self.llm.description
            ),
        }
    }

    async fn status(&self) -> frontend::LLMStatus {
        todo!()
    }

    async fn ping(&self) -> Result<String, String> {
        match self.actor.ask(llm_actor::IDMessage()).await {
            Ok(result) => Ok(format!("id result: {:?}", result)),
            Err(err) => Err(format!(
                "ID error—This likely means the LLM is dead: {:?}",
                err
            )),
        }
    }

    async fn reload(&self) -> Result<(), PantryError> {
        todo!()
    }

    async fn get_sessions(&self, user: user::User) -> Result<Vec<LLMSession>, PantryError> {
        println!(
            "Called get_sessions with LLM UUID {} and user {:?}",
            self.llm.uuid, user
        );

        match self
            .actor
            .ask(llm_actor::GetLLMSessionsMessage { user: user.into() })
            .await
        {
            Ok(result) => match result {
                Ok(sessions) => Ok(sessions),
                Err(err) => Err(PantryError::OtherFailure(err)),
            },
            Err(err) => Err(PantryError::ActorFailure(err)),
        }
    }

    async fn create_session(
        &self,
        params: HashMap<String, Value>,
        user: user::User,
    ) -> Result<CreateSessionResponse, PantryError> {
        println!(
            "Called create_session with LLM UUID {} and user {:?}",
            self.llm.uuid, user
        );
        // Reconcile Parameters
        let mut armed_params = self.llm.session_parameters.clone();
        for param in self.llm.user_session_parameters.iter() {
            if let Some(val) = params.get(param) {
                armed_params.insert(param.clone(), json!(val.clone()));
            }
        }
        match self
            .actor
            .ask(llm_actor::CreateSessionMessage {
                session_params: armed_params.clone(),
                user: user.into(),
            })
            .await
        {
            Ok(result) => match result {
                Ok(session_id) => Ok(CreateSessionResponse {
                    session_id: session_id,
                    session_parameters: armed_params,
                }),
                Err(err) => Err(PantryError::OtherFailure(err)),
            },
            Err(err) => Err(PantryError::ActorFailure(err)),
        }
    }

    async fn prompt_session(
        &self,
        session_id: Uuid,
        prompt: String,
        parameters: HashMap<String, Value>,
        user: user::User,
    ) -> Result<PromptSessionResponse, PantryError> {
        println!(
            "Called prompt_session with LLM UUID {} and user {:?}",
            self.llm.uuid, user
        );

        // Reconcile Parameters
        let mut armed_params = self.llm.parameters.clone();
        for param in self.llm.user_parameters.iter() {
            if let Some(val) = parameters.get(param) {
                armed_params.insert(param.clone(), json!(val.clone()));
            }
        }

        let (sender, receiver): (
            mpsc::Sender<connectors::LLMEvent>,
            mpsc::Receiver<connectors::LLMEvent>,
        ) = mpsc::channel(100);

        let msg: String = prompt.clone().into();
        let act = self.actor.clone();

        let cloned_params = armed_params.clone();

        let interrupts_clone = self.interrupts.clone();

        let token = CancellationToken::new();
        let cloned_token = token.clone();
        let key = (session_id.clone(), user.id.clone());
        if self.interrupts.contains_key(&key) {
            self.interrupts
                .get_mut(&key)
                .expect("We just checked that it contains the key.")
                .push(token);
        } else {
            self.interrupts.insert(key, vec![token]);
        }

        tokio::spawn(async move {
            let result = act
                .ask(llm_actor::PromptSessionMessage {
                    session_id: session_id.clone(),
                    prompt: msg,
                    prompt_params: armed_params.clone(),
                    user: user.into(),
                    sender: sender,
                    cancellation_token: cloned_token,
                })
                .await;

            match result {
                Ok(res) => match res {
                    Ok(()) => {
                        println!("Completed inference successfully.");
                    }
                    Err(err) => {
                        println!("Failed to complete inference: {:?}", err);
                    }
                },
                Err(err) => {
                    println!("Failed to send inference message: {:?}", err);
                }
            }
        });

        Ok(PromptSessionResponse {
            stream: receiver,
            parameters: cloned_params,
        })
    }

    // kinda ugly that we need mutability here for a potentially long call, for a short mut.
    async fn call_llm(
        &self,
        message: &str,
        session_parameters: HashMap<String, Value>,
        parameters: HashMap<String, Value>,
        user: user::User,
    ) -> Result<CallLLMResponse, PantryError> {
        let create_sess_response = self
            .create_session(session_parameters, user.clone())
            .await?;
        let prompt_response = self
            .prompt_session(
                create_sess_response.session_id,
                message.into(),
                parameters,
                user.clone(),
            )
            .await?;

        Ok(CallLLMResponse {
            session_id: create_sess_response.session_id,
            parameters: prompt_response.parameters,
            session_parameters: create_sess_response.session_parameters,
            stream: prompt_response.stream,
        })

        //NOTE: This separate implementation DOES NOT include anything past
        //the addition of interrupts or interrupts themselves. Slated for deletion
        //once confirmed that the simplified redirect implementation works.

        // Reconcile Parameters
        // let mut armed_session_params = self.llm.session_parameters.clone();
        // for param in self.llm.user_session_parameters.iter() {
        //     if let Some(val) = session_parameters.get(param) {
        //         armed_session_params.insert(param.clone(), json!(val.clone()));
        //     }
        // }

        // let mut armed_params = self.llm.parameters.clone();
        // for param in self.llm.user_parameters.iter() {
        //     if let Some(val) = parameters.get(param) {
        //         armed_params.insert(param.clone(), json!(val.clone()));
        //     }
        // }

        // // We need to create the sender here, so we can stick the non-async LLM
        // // compute into a separate thread.
        // let (sender, receiver): (
        //     mpsc::Sender<connectors::LLMEvent>,
        //     mpsc::Receiver<connectors::LLMEvent>,
        // ) = mpsc::channel(100);

        // println!(
        //     "Called {} with {} using {:?} and params {:?}",
        //     self.llm.id, message, self.actor, armed_params
        // );

        // let session_uuid: Uuid = self
        //     .actor
        //     .ask(llm_actor::CreateSessionMessage {
        //         session_params: armed_session_params.clone(),
        //         user: user.clone(),
        //     })
        //     .await
        //     .map_err(|err| format!("Failed to send: {:?}", err))?
        //     .map_err(|err| format!("Failed to create session: {:?}", err))?;

        // let resp = CallLLMResponse {
        //     session_id: session_uuid.clone(),
        //     stream: receiver,
        //     parameters: armed_params.clone(),
        // };
        // let act = self.actor.clone();
        // let msg: String = message.clone().into();

        // tokio::spawn(async move {
        //     match act
        //         .ask(llm_actor::PromptSessionMessage {
        //             session_uuid: session_uuid,
        //             prompt: msg.into(),
        //             prompt_params: armed_params.clone(),
        //             user: user,
        //             sender: sender,
        //         })
        //         .await
        //     {
        //         Ok(result) => match result {
        //             Ok(()) => {
        //                 println!("Completed inference successfully.");
        //             }
        //             Err(err) => {
        //                 println!("Failed to complete inference: {:?}", err);
        //             }
        //         },
        //         Err(err) => {
        //             println!("Failed to send inference message: {:?}", err);
        //         }
        //     }
        // });

        // Ok(resp)
    }

    async fn interrupt_session(
        &self,
        session_id: Uuid,
        user: user::User,
    ) -> Result<bool, PantryError> {
        //api layer should verify user == user
        let key = (session_id.clone(), user.id.clone());

        println!("Attempting to interrupt session");
        let key = (session_id.clone(), user.id.clone());

        println!("For fuck sake: {:?}", self.interrupts);

        let res = self
            .interrupts
            .get_mut(&key)
            .ok_or(format!("Cancellation token missing"))?
            .value()
            .iter()
            .map(|x| x.cancel())
            .count();

        match res {
            0 => Ok(false),
            _ => {
                println!("we interrupted");
                self.interrupts
                    .get_mut(&key)
                    .expect("We've already confirmed this exists.")
                    .clear();

                println!("We're returning");
                Ok(true)
            }
        }

        // let result = self
        //     .actor
        //     .ask(llm_actor::InterruptSessionMessage {
        //         session_id: session_id.clone(),
        //         user: user.into(),
        //     })
        //     .await;

        // match result {
        //     Ok(res) => match res {
        //         Ok(true) => {
        //             println!("Disrupted inference.");
        //             Ok(true)
        //         }
        //         Ok(false) => {
        //             println!("No inference to interrupt.");
        //             Ok(false)
        //         }
        //         Err(err) => {
        //             println!("Failed to interrupt inference: {:?}", err);
        //             Err(PantryError::OtherFailure(err))
        //         }
        //     },
        //     Err(err) => {
        //         println!("Failed to send interrupt message: {:?}", err);
        //         Err(PantryError::OtherFailure(err.to_string()))
        //     }
        // }
    }

    fn into_llm_running(&self) -> frontend::LLMRunning {
        self.into()
    }
}
