use crate::connectors::{LLMEvent, LLMEventInternal, LLMInternalWrapper};
use crate::database;
use crate::database_types::*;
use crate::emitter;
use crate::llm::{LLMHistoryItem, LLMSession};
use crate::state;
use crate::user::User;
use bincode::{deserialize_from, serialize_into};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

use llm::{InferenceError, InferenceFeedback, InferenceSession};
use log::{debug, error, info, warn};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

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
    id: String, //we use this for better debug displays
    uuid: Uuid,
    user_settings: state::UserSettings,
    pool: Pool<ConnectionManager<SqliteConnection>>,
    notification_emitter: emitter::NotificationEmitter,
}

impl LLMrsConnector {
    pub fn new(
        id: String,
        uuid: Uuid,
        data_path: PathBuf,
        config: HashMap<String, Value>,
        model_path: PathBuf,
        user_settings: state::UserSettings,
        pool: Pool<ConnectionManager<SqliteConnection>>,
        notification_emitter: emitter::NotificationEmitter,
    ) -> LLMrsConnector {
        let mut path = data_path.clone();
        path.push(format!("llmrs-{}", uuid.to_string()));
        fs::create_dir_all(path.clone());
        let conn = LLMrsConnector {
            config,
            id,
            uuid,
            data_path: path,
            model_path,
            model: RwLock::new(None),
            loaded_sessions: DashMap::new(),
            user_settings,
            pool,
            notification_emitter,
        };
        conn
    }

    fn rehydrate_session(&self, llm_sess: LLMSession) -> Result<Uuid, String> {
        self.notification_emitter.send_notification(
            self.uuid.to_string(),
            format!("Rehydrating session for {}", self.id.to_string()),
        );
        info!("Rehyrdatring session {:?}", llm_sess.id);
        let mut path = self.data_path.clone();
        path.push(llm_sess.id.0.to_string());

        let reader =
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
        info!("dehydrating session");
        self.notification_emitter.send_notification(
            self.uuid.to_string(),
            format!(
                "Maintenance: Dehydrating session for {}",
                self.id.to_string()
            ),
        );

        // Safety: you can't use the model while get_snapshot is alive
        // The lock above us ensures this, and we're going to drop the model after anyway
        unsafe {
            let snapshot = raw.get_snapshot();
            let mut path = self.data_path.clone();
            path.push(llmrs_sess.llm_session.read().unwrap().id.0.to_string());
            let writer = File::create(path)
                .map_err(|err| format!("Failed to serialize to file: {:?}", err))?;
            let _serialized = serialize_into(writer, &snapshot)
                .map_err(|err| format!("Failed to serialize to file: {:?}", err))?;
        }

        Ok(())
    }

    // Due to a combination of DashMap and borrow checking we can't actually
    // return get(&session_id). So instead we return sessionid and the end
    // user needs to use it. This, ironically, is _less_ typesafe but whatever.
    fn get_session_check(&self, session_id: &Uuid) -> Result<Uuid, String> {
        debug!("Checking session {}", session_id.to_string());
        // if let Some(sess) = self.loaded_sessions.get(&session_id) {
        //     return Ok(sess.value());
        // }
        if let Some(_sess) = self.loaded_sessions.get(&session_id) {
            return Ok(session_id.clone());
        }

        debug!("Getting session from DB");
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

    async fn maintenance(&mut self) -> Result<(), String> {
        debug!("Running maintenance check...");
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
            debug!(
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
        self.notification_emitter.send_notification(
            self.uuid.to_string(),
            format!("Creating session for {}", self.id.to_string()),
        );
        let mut model_read = self
            .model
            .write()
            .map_err(|err| format!("probably rwgard: {:?}", err))?;
        let mut inference_session_config: llm::InferenceSessionConfig = Default::default();
        inference_session_config.n_threads = self.user_settings.n_thread;
        inference_session_config.n_batch = self.user_settings.n_batch;

        //TODO: User settings for implementing gpu accel
        let mut inference = model_read
            .as_mut()
            .expect("model missing")
            .start_session(inference_session_config);
        let uuid = Uuid::new_v4();

        if let Some(Value::String(s)) = params.get("system_prompt") {
            match inference.feed_prompt(
                self.model
                    .read()
                    .map_err(|err| format!("failed to get read lock on model {:?}", err))?
                    .as_ref()
                    .expect("Model is not available (opt is None)")
                    .as_ref(),
                s,
                &mut Default::default(),
                |_| -> Result<InferenceFeedback, InferenceError> {
                    Ok(InferenceFeedback::Continue)
                },
            ) {
                Ok(_) => (),
                Err(e) => error!("Failed to feed system prompt: {:?}", e),
            };
        }

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

        debug!("Received call to prompt session");
        let mut sampler = llm::samplers::default_samplers();

        if let Some(Value::String(s)) = params.get("sampler_string") {
            // if let Ok(sampler_str) = s.parse() {
            if let Ok(configured_samplers) = llm::samplers::ConfiguredSamplers::from_str(&s) {
                // let new_sampler: dyn llm::samplers::Sampler<llm::TokenId, f32> =
                //     configured_samplers;
                sampler = Arc::new(Mutex::new(configured_samplers.builder.into_chain()));
            }
        }
        // if let Some(Value::Number(n)) = params.get("top_k") {
        //     if let Some(top_k) = n.as_u64() {
        //         sampler.top_k = top_k as usize;
        //     }
        // }

        // if let Some(Value::Number(n)) = params.get("top_p") {
        //     if let Some(top_p) = n.as_f64() {
        //         sampler.top_p = top_p as f32;
        //     }
        // }

        // if let Some(Value::Number(n)) = params.get("repeat_penalty") {
        //     if let Some(repeat_penalty) = n.as_f64() {
        //         sampler.repeat_penalty = repeat_penalty as f32;
        //     }
        // }

        // if let Some(Value::Number(n)) = params.get("temperature") {
        //     if let Some(temperature) = n.as_f64() {
        //         sampler.temperature = temperature as f32;
        //     }
        // }

        // if let Some(Value::String(s)) = params.get("bias_tokens") {
        //     if let Ok(bias_tokens) = s.parse() {
        //         sampler.bias_tokens = bias_tokens;
        //     }
        // }

        // if let Some(Value::Number(n)) = params.get("repetition_penalty_last_n") {
        //     if let Some(repetition_penalty_last_n) = n.as_u64() {
        //         sampler.repetition_penalty_last_n = repetition_penalty_last_n as usize;
        //     }
        // }

        let mut stop_sequence = None;
        let mut processed_prompt = prompt;
        if let Some(Value::String(s)) = params.get("pre_prompt") {
            if let pre_prompt = s {
                stop_sequence = Some(pre_prompt.clone());
                processed_prompt = pre_prompt.to_owned() + &processed_prompt;
            }
        }

        if let Some(Value::String(s)) = params.get("post_prompt") {
            if let post_prompt = s {
                processed_prompt = processed_prompt + post_prompt;
            }
        }

        let _inf_params = llm::InferenceParameters { sampler: sampler };
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

        // let n_vocab = model_armed.n_vocab();

        // llm::samplers::build_sampler(n_vocab, &[], args)

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
            input: processed_prompt.clone(),
            output: "".into(),
        };

        let new_item = database::save_new_llm_history(new_item, self.pool.clone())
            .map_err(|err| format!("Database failure: {:?}", err))?;

        let mut stop_sequence_buf = String::new();

        self.notification_emitter.send_notification(
            self.uuid.to_string(),
            format!("Beginning inference for {}", self.id.to_string()),
        );
        debug!("Attempting to infer");
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
                    prompt: (&processed_prompt).into(),
                    parameters: &llm::InferenceParameters::default(),
                    play_back_previous_tokens: false,
                    maximum_token_count: None,
                },
                // OutputRequest
                &mut Default::default(),
                |r| match r {
                    llm::InferenceResponse::InferredToken(t) => {
                        print!("{t}");

                        self.notification_emitter.send_notification(
                            self.uuid.to_string(),
                            format!("{}: Inferred '{}'", self.id.to_string(), t.clone()),
                        );
                        let mut cancel = false;

                        if let Some(stop_seq) = stop_sequence.clone() {
                            let mut buf = stop_sequence_buf.clone();
                            buf.push_str(&t);
                            if buf.starts_with(&stop_seq) {
                                // We've generated the stop sequence, so we're done.
                                // Note that this will contain the extra tokens that were generated after the stop sequence,
                                // which may affect generation. This is non-ideal, but it's the best we can do without
                                // modifying the model.
                                stop_sequence_buf.clear();
                                cancel = true;
                            } else if stop_seq.starts_with(&buf) {
                                // We've generated a prefix of the stop sequence, so we need to keep buffering.
                                stop_sequence_buf = buf;
                            }

                            // We've generated a token that isn't part of the stop sequence, so we can
                            // pass it to the callback.
                            stop_sequence_buf.clear();
                        }

                        let session_clone = llm_session_armed.clone();

                        let update_item =
                            database::get_llm_history(item_id, self.pool.clone()).unwrap();

                        let event = LLMEvent {
                            stream_id: item_id.clone(),
                            timestamp: Utc::now(),
                            call_timestamp: new_item.call_timestamp.clone(),
                            parameters: new_item.parameters.0.clone(),
                            input: processed_prompt.clone(),
                            llm_uuid: self.uuid.clone(),
                            session: (&session_clone).into(),
                            event: LLMEventInternal::PromptProgress {
                                previous: update_item.output.clone().into(),
                                next: t.clone(),
                            },
                        };

                        let send_clone = sender.clone();
                        let event_clone = event.clone();
                        let cancel_token = cancellation.clone();
                        tokio::task::spawn(async move {
                            let print_clone = event_clone.event.clone();
                            if let Err(_e) = send_clone.send(event_clone).await {
                                warn!("Error sending, so cancelling.");
                                cancel_token.cancel();
                            }
                            debug!("Sent an event from llmrs for {:?}", print_clone);
                        });
                        if cancellation.is_cancelled() {
                            cancel = true;
                        }

                        let update_item =
                            database::append_token(update_item, t, cancel, self.pool.clone())
                                .unwrap();
                        if cancel {
                            debug!("SENT CONCLUSION");
                            let mut event_clone = event.clone();
                            event_clone.event = LLMEventInternal::PromptCompletion {
                                previous: update_item.output.clone().into(),
                            };
                            event_clone.timestamp = Utc::now();

                            let send_clone = sender.clone();
                            // we need to grab a new clone that includes the updated session
                            event_clone.session = (&*llm_session_armed).into();
                            tokio::task::spawn(async move {
                                let print_clone = event_clone.event.clone();
                                send_clone.send(event_clone).await;
                                debug!("Sent an event from llmrs for {:?}", print_clone);
                            });
                        }
                        match cancel {
                            true => Ok(llm::InferenceFeedback::Halt),
                            false => Ok(llm::InferenceFeedback::Continue),
                        }
                    }

                    llm::InferenceResponse::EotToken => {
                        print!("Received EOT from LLM");
                        let _session_clone = llm_session_armed.clone();

                        let update_item =
                            database::get_llm_history(item_id, self.pool.clone()).unwrap();
                        let _update_item =
                            database::append_token(update_item, "".into(), true, self.pool.clone())
                                .unwrap();

                        Ok(llm::InferenceFeedback::Halt)
                    }
                    _ => {
                        debug!("got other");
                        match cancellation.is_cancelled() {
                            // TODO: mark complete here
                            // TODO: mark complete final token whatever
                            true => Ok(llm::InferenceFeedback::Halt),
                            false => Ok(llm::InferenceFeedback::Continue),
                        }
                    }
                },
            )
            .map_err(|err| format!("failure to infer with {:?}", err))?;

        debug!("Sending back receiver");
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

        info!("successfully unloaded {:?}", self.uuid);

        //we don't remove because we're about to drain
        Ok(())
    } //called by shutdown
    async fn unload_llm(self: &Self) -> Result<(), String> {
        Ok(())
    }
}
