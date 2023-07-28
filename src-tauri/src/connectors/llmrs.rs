use crate::connectors::{LLMEvent, LLMEventInternal, LLMInternalWrapper};
use crate::database;
use crate::database_types::*;
use crate::llm::{LLMHistoryItem, LLMSession};
use crate::state;
use crate::user::User;
use bincode::{deserialize_from, serialize_into};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use llm::InferenceSession;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use tiny_tokio_actor::*;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
// use llm::llm_base::InferenceSnapshotRef;
use std::sync::{Arc, Mutex, RwLock};

//src/connectors/llmrs.rs

// REQUIRED CONFIGS:
// model_architecture
//
//
// The SYSTEM PROVIDES THESE CONFIGS:
// model_path
//
//
// Parameters (per session)
// OPTIONAL PARAMETERS
//     pub top_k: usize,
// top_p: f32,
// repeat_penalty: f32,
// temperature: f32,
// bias_tokens: TokenBias,
// repetition_penalty_last_n: usize,
//
pub struct LLMrsConnector {
    pub config: HashMap<String, Value>,
    model_path: PathBuf,
    model: RwLock<Option<Box<dyn llm::Model>>>,
    loaded_sessions: DashMap<Uuid, LLMrsSession>,
    data_path: PathBuf,
    uuid: Uuid,
    user_settings: state::UserSettings,
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl LLMrsConnector {
    pub fn new(
        uuid: Uuid,
        data_path: PathBuf,
        config: HashMap<String, Value>,
        model_path: PathBuf,
        user_settings: state::UserSettings,
        pool: Pool<ConnectionManager<SqliteConnection>>,
    ) -> LLMrsConnector {
        let mut path = data_path.clone();
        path.push(format!("llmrs-{}", uuid.to_string()));
        fs::create_dir_all(path.clone());
        let mut conn = LLMrsConnector {
            config,
            uuid,
            data_path: path,
            model_path,
            model: RwLock::new(None),
            loaded_sessions: DashMap::new(),
            user_settings,
            pool,
        };
        conn
    }

    fn rehydrate_session(&self, llm_sess: LLMSession) -> Result<Uuid, String> {
        println!("Rehyrdatring session {:?}", llm_sess.id);
        let mut path = self.data_path.clone();
        path.push(llm_sess.id.0.to_string());

        let mut reader =
            File::open(path).map_err(|err| format!("Failed to deserialize to file: {:?}", err))?;

        let snapshot = deserialize_from(reader)
            .map_err(|err| format!("Failed to deserialize to file: {:?}", err))?;
        // self.model.write().unwrap().ok_or("model not activated, cannot rehydrate".into())?;
        let model_tmp = self
            .model
            .read()
            .map_err(|err| format!("failed to get read lock on model {:?}", err))?;

        let model = model_tmp
            .as_ref()
            .expect("Model is not available (opt is None)");

        let inference = InferenceSession::from_snapshot(snapshot, model.as_ref())
            .map_err(|err| format!("Failed to rehydrate model: {:?}", err))?;
        let uuid = llm_sess.id.0.clone();

        let new_session = LLMrsSession {
            model_session: Arc::new(Mutex::new(inference)),
            llm_session: Arc::new(RwLock::new(llm_sess)),
        };

        self.loaded_sessions.insert(uuid.clone(), new_session);

        Ok(uuid.clone())
    }

    fn dehydrate_session(&self, llmrs_sess: LLMrsSession) -> Result<(), String> {
        let mut raw = llmrs_sess.model_session.lock().unwrap();
        println!("dehydrating session");

        // Safety: you can't use the model while get_snapshot is alive
        // The lock above us ensures this, and we're going to drop the model after anyway
        unsafe {
            let snapshot = raw.get_snapshot();
            let mut path = self.data_path.clone();
            path.push(llmrs_sess.llm_session.read().unwrap().id.0.to_string());
            let mut writer = File::create(path)
                .map_err(|err| format!("Failed to serialize to file: {:?}", err))?;
            let serialized = serialize_into(writer, &snapshot)
                .map_err(|err| format!("Failed to serialize to file: {:?}", err))?;
        }

        Ok(())
    }

    // Due to a combination of DashMap and borrow checking we can't actually
    // return get(&session_id). So instead we return sessionid and the end
    // user needs to use it. This, ironically, is _less_ typesafe but whatever.
    fn get_session_check(&self, session_id: &Uuid) -> Result<Uuid, String> {
        println!("Checking session {}", session_id.to_string());
        // if let Some(sess) = self.loaded_sessions.get(&session_id) {
        //     return Ok(sess.value());
        // }
        if let Some(sess) = self.loaded_sessions.get(&session_id) {
            return Ok(session_id.clone());
        }

        println!("Getting session from DB");
        let session: LLMSession = database::get_llm_session(session_id.clone(), self.pool.clone())
            .map_err(|err| format!("Database failure, probably not found: {:?}", err))?;

        if session.llm_uuid.0 == self.uuid {
            return self.rehydrate_session(session);
        } else {
            return Err("unable to find session".into());
        }
    }
}

#[derive(Clone)]
struct LLMrsSession {
    // We're doing inner mutex because we need to modify the llm_session even while holding
    // the lock for the model_session, and we need to do so in closures that would deadlock
    // otherwise.
    model_session: Arc<Mutex<llm::InferenceSession>>,
    llm_session: Arc<RwLock<LLMSession>>,
}

#[derive(Clone, serde::Serialize)]
struct LLMrsSessionSerial {
    // session_snapshot: llm::InferenceSnapshotRef,
    llm_session: LLMSession,
}

#[derive(Clone, serde::Deserialize)]
struct LLMrsSessionDeserial {
    session_snapshot: llm::InferenceSnapshot,
    llm_session: LLMSession,
}

#[async_trait]
impl LLMInternalWrapper for LLMrsConnector {
    // async fn call_llm(self: &mut Self, msg: String, session_params: HashMap<String, Value>, params: HashMap<String, Value>, user: User) -> Result<(Uuid, mpsc::Receiver<LLMEvent>), String> {

    //     println!("Triggered call llm for {:?} with \"{}\" and {:?}", user, msg, params);

    //     // Create a new session with the provided parameters
    //     let session_id = self.create_session(session_params, user.clone()).await?;
    //     println!("created a session");

    //     // Now that a new session is created, we need to prompt it immediately with the given message
    //     match self.prompt_session(session_id, msg, params, user).await {
    //         Ok(stream) => Ok((session_id, stream)),
    //         Err(e) => Err(e)
    //     }
    // }
    // async fn get_sessions(self: &Self, user: User) -> Result<Vec<LLMSession>, String> {
    //     let sesss: Vec<LLMSession> = self
    //         .sessions
    //         .iter()
    //         .filter(|ff| ff.value().llm_session.read().unwrap().user_id == user.id)
    //         .map(|ff| ff.value().llm_session.as_ref().read().unwrap().clone())
    //         .collect();
    //     return Ok(sesss);
    // }
    async fn maintenance(&mut self) -> Result<(), String> {
        println!("HERE: HERE: Running maintenance check...");
        if self.loaded_sessions.len() > 1 {
            // if self.loaded_sessions.len() > self.user_settings.preferred_active_sessions {
            let mut llm_list: Vec<(Uuid, DateTime<Utc>)> = self
                .loaded_sessions
                .iter()
                .map(|pair| {
                    (
                        pair.key().clone(),
                        pair.value().llm_session.read().unwrap().last_called.clone(),
                    )
                })
                .collect();

            llm_list.sort_by(|a, b| a.1.cmp(&b.1));
            // get the last uuid
            let uuid = llm_list.pop().ok_or("list empty")?.0;
            let llmrs_sess = self
                .loaded_sessions
                .remove(&uuid)
                .expect("we just grabbed this key")
                .1;

            self.dehydrate_session(llmrs_sess)?;
            println!(
                "now have this loaded session struct: {:?}",
                self.loaded_sessions.len()
            );
        }

        Ok(())
    }

    async fn create_session(
        self: &mut Self,
        params: HashMap<String, Value>,
        user: User,
    ) -> Result<Uuid, String> {
        let mut model_read = self
            .model
            .write()
            .map_err(|err| format!("probably rwgard: {:?}", err))?;
        //TODO: User settings for implementing gpu accel
        let inference = model_read
            .as_mut()
            .expect("model missing")
            .start_session(Default::default());
        let uuid = Uuid::new_v4();

        let new_session = LLMrsSession {
            model_session: Arc::new(Mutex::new(inference)),
            llm_session: Arc::new(RwLock::new(
                database::save_new_llm_session(
                    LLMSession {
                        id: DbUuid(uuid),
                        llm_uuid: DbUuid(self.uuid.clone()), // replace with actual llm_uuid
                        user_id: user.id,                    // replace with actual user_id
                        started: Utc::now(),
                        last_called: Utc::now(),
                        session_parameters: DbHashMap(params),
                    },
                    self.pool.clone(),
                )
                .map_err(|err| format!("Database failure: {:?}", err))?,
            )),
        };

        self.loaded_sessions.insert(uuid, new_session);

        Ok(uuid)
    } //uuid

    async fn prompt_session(
        self: &mut Self,
        session_id: Uuid,
        prompt: String,
        params: HashMap<String, Value>,
        _user: User,
        sender: mpsc::Sender<LLMEvent>,
        cancellation: CancellationToken,
    ) -> Result<(), String> {
        // The infer function is blocking, and once we start we can't move to another thread
        // which means we need to move to another thread NOW and return our sender.
        //
        // The alternative is acknowledging that we should have passed in the sender

        println!("Received call to prompt session");
        let mut sampler = llm::samplers::TopPTopK::default();
        if let Some(Value::Number(n)) = self.config.get("top_k") {
            if let Some(top_k) = n.as_u64() {
                sampler.top_k = top_k as usize;
            }
        }

        if let Some(Value::Number(n)) = self.config.get("top_p") {
            if let Some(top_p) = n.as_f64() {
                sampler.top_p = top_p as f32;
            }
        }

        if let Some(Value::Number(n)) = self.config.get("repeat_penalty") {
            if let Some(repeat_penalty) = n.as_f64() {
                sampler.repeat_penalty = repeat_penalty as f32;
            }
        }

        if let Some(Value::Number(n)) = self.config.get("temperature") {
            if let Some(temperature) = n.as_f64() {
                sampler.temperature = temperature as f32;
            }
        }

        if let Some(Value::String(s)) = self.config.get("bias_tokens") {
            if let Ok(bias_tokens) = s.parse() {
                sampler.bias_tokens = bias_tokens;
            }
        }

        if let Some(Value::Number(n)) = self.config.get("repetition_penalty_last_n") {
            if let Some(repetition_penalty_last_n) = n.as_u64() {
                sampler.repetition_penalty_last_n = repetition_penalty_last_n as usize;
            }
        }

        let _inf_params = llm::InferenceParameters {
            n_threads: self.user_settings.n_thread,
            n_batch: self.user_settings.n_batch,
            sampler: Arc::new(sampler),
        };
        self.get_session_check(&session_id)?;
        let session_wrapped = self
            .loaded_sessions
            .get(&session_id)
            .expect("missing session");
        let mut model_armed = session_wrapped
            .value()
            .model_session
            .as_ref()
            .lock()
            .map_err(|err| format!("failed to acquire lock: {:?}", err))?;

        let llm_session_armed = session_wrapped
            .llm_session
            .as_ref()
            .write()
            .map_err(|err| format!("failed to acquire lock: {:?}", err))?;

        // Do our own bookkeeping before calling the LLM.
        let item_id = Uuid::new_v4();
        let new_item = LLMHistoryItem {
            id: DbUuid(item_id.clone()),
            llm_session_id: llm_session_armed.id.clone(),
            updated_timestamp: Utc::now(),
            call_timestamp: Utc::now(),
            complete: false, // initially false, will be set to true once response is received
            parameters: DbHashMap(params.clone()),
            input: prompt.clone(),
            output: "".into(),
        };

        let new_item = database::save_new_llm_history(new_item, self.pool.clone())
            .map_err(|err| format!("Database failure: {:?}", err))?;

        println!("Attempting to infer");
        // Call the llm
        model_armed
            .infer::<mpsc::error::SendError<LLMEvent>>(
                self.model
                    .read()
                    .map_err(|err| format!("failed to get read lock on model {:?}", err))?
                    .as_ref()
                    .expect("Model is not available (opt is None)")
                    .as_ref(),
                &mut rand::thread_rng(),
                &llm::InferenceRequest {
                    prompt: (&prompt).into(),
                    parameters: &llm::InferenceParameters::default(),
                    play_back_previous_tokens: false,
                    maximum_token_count: None,
                },
                // OutputRequest
                &mut Default::default(),
                |r| match r {
                    llm::InferenceResponse::InferredToken(t) => {
                        print!("{t}");

                        let session_clone = llm_session_armed.clone();

                        let update_item =
                            database::get_llm_history(item_id, self.pool.clone()).unwrap();

                        let event = LLMEvent {
                            stream_id: item_id.clone(),
                            timestamp: Utc::now(),
                            call_timestamp: new_item.call_timestamp.clone(),
                            parameters: new_item.parameters.0.clone(),
                            input: prompt.clone(),
                            llm_uuid: self.uuid.clone(),
                            session: session_clone,
                            event: LLMEventInternal::PromptProgress {
                                previous: update_item.output.clone().into(),
                                next: t.clone(),
                            },
                        };

                        let send_clone = sender.clone();
                        let event_clone = event.clone();
                        tokio::task::spawn(async move {
                            let print_clone = event_clone.event.clone();
                            send_clone.send(event_clone).await;
                            println!("Sent an event from llmrs for {:?}", print_clone);
                        });

                        let cancel = cancellation.is_cancelled();
                        let update_item =
                            database::append_token(update_item, t, cancel, self.pool.clone())
                                .unwrap();
                        if cancel {
                            println!("SENT CONCLUSION");
                            let mut event_clone = event.clone();
                            event_clone.event = LLMEventInternal::PromptCompletion {
                                previous: update_item.output.clone().into(),
                            };
                            event_clone.timestamp = Utc::now();

                            let send_clone = sender.clone();
                            // we need to grab a new clone that includes the updated session
                            event_clone.session = llm_session_armed.clone();
                            tokio::task::spawn(async move {
                                let print_clone = event_clone.event.clone();
                                send_clone.send(event_clone).await;
                                println!("Sent an event from llmrs for {:?}", print_clone);
                            });
                        }
                        match cancellation.is_cancelled() {
                            true => Ok(llm::InferenceFeedback::Halt),
                            false => Ok(llm::InferenceFeedback::Continue),
                        }
                    }

                    llm::InferenceResponse::EotToken => {
                        print!("Received EOT from LLM");
                        let session_clone = llm_session_armed.clone();

                        let update_item =
                            database::get_llm_history(item_id, self.pool.clone()).unwrap();
                        let update_item =
                            database::append_token(update_item, "".into(), true, self.pool.clone())
                                .unwrap();

                        Ok(llm::InferenceFeedback::Halt)
                    }
                    _ => match cancellation.is_cancelled() {
                        // TODO: mark complete here
                        // TODO: mark complete final token whatever
                        true => Ok(llm::InferenceFeedback::Halt),
                        false => Ok(llm::InferenceFeedback::Continue),
                    },
                },
            )
            .map_err(|err| format!("failure to infer with {:?}", err))?;

        println!("Sending back receiver");
        Ok(())
    }

    async fn load_llm(self: &mut Self) -> Result<(), String> {
        let vocab_source: llm::TokenizerSource = match (
            self.config.get("vocabulary_path"),
            self.config.get("vocabulary_repository"),
        ) {
            (Some(_), Some(_)) => {
                panic!("Cannot specify both --vocabulary-path and --vocabulary-repository");
            }
            (Some(path), None) => {
                llm::TokenizerSource::HuggingFaceTokenizerFile(PathBuf::from(path.to_string()))
            }
            (None, Some(repo)) => llm::TokenizerSource::HuggingFaceRemote(repo.to_string()),
            (None, None) => llm::TokenizerSource::Embedded,
        };

        let _now = std::time::Instant::now();

        //llm.rs now supports infering model architecture, but we won't support it.
        let model_architecture: llm::ModelArchitecture = self
            .config
            .get("model_architecture")
            .ok_or("missing model architecture")?
            .to_string()
            .parse()
            .map_err(|_err| format!("unsupported model architecture"))?;

        let mut model_params: llm::ModelParameters = Default::default();
        model_params.use_gpu = self.user_settings.use_gpu;

        let mut writer = self.model.write().unwrap();
        *writer = Some(
            llm::load_dynamic(
                Some(model_architecture),
                &self.model_path,
                vocab_source,
                model_params,
                llm::load_progress_callback_stdout,
            )
            .map_err(|_err| "fuckup loading")?,
        );

        Ok(())
    }

    async fn pre_unload(self: &Self) -> Result<(), String> {
        let uuids: Vec<Uuid> = self
            .loaded_sessions
            .iter()
            .map(|pair| pair.key().clone())
            .collect();

        for uuid in uuids {
            self.dehydrate_session(self.loaded_sessions.remove(&uuid).expect("beep").1);
        }

        println!("successfully unloaded {:?}", self.uuid);

        //we don't remove because we're about to drain
        Ok(())
    } //called by shutdown
    async fn unload_llm(self: &Self) -> Result<(), String> {
        Ok(())
    }
}
