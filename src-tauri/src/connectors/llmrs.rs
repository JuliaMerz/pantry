use crate::connectors::{LLMEvent, LLMEventInternal, LLMInternalWrapper};
use crate::database_types::*;
use crate::llm::{LLMHistoryItem, LLMSession};
use crate::state;
use crate::user::User;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{error::Error, fs::File};
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
    sessions: DashMap<Uuid, LLMrsSession>,
    data_path: PathBuf,
    uuid: Uuid,
    user_settings: state::UserSettings,
}

impl LLMrsConnector {
    pub fn new(
        uuid: Uuid,
        data_path: PathBuf,
        config: HashMap<String, Value>,
        model_path: PathBuf,
        user_settings: state::UserSettings,
    ) -> LLMrsConnector {
        let mut path = data_path.clone();
        path.push(format!("llmrs-{}", uuid.to_string()));
        let mut conn = LLMrsConnector {
            config: config,
            uuid: uuid,
            data_path: data_path,
            model_path: model_path,
            model: RwLock::new(None),
            sessions: DashMap::new(),
            user_settings: user_settings,
        };
        conn.deserialize_sessions();
        conn
    }

    //TODO: This is expensive as all hell, we should be doing this by individual sessions
    // Utility functions to serialize and deserialize sessions
    pub fn serialize_sessions(&self) -> Result<(), Box<dyn std::error::Error>> {
        return Ok(()); //We skip serializing because inferencesessionref is missing in llmrs.
                       // let mut file = File::create(&self.data_path)?;

        // let vec: Vec<LLMrsSessionSerial> = self.sessions.into_iter().map(|sess| {
        //     let sess_guard = sess.1.lock().unwrap();
        //     // We're immediately taking ownership and writing, and we're in the mutex,
        //     // see https://github.com/rustformers/llm/blob/9123171ab1aa436fcfa45b9aaef90211534f9fdb/crates/llm-base/src/inference_session.rs#L540C1-L565C1
        //     let session_snapshot = unsafe {sess_guard.model_session.get_snapshot()};
        //     let llm_session = sess_guard.llm_session.clone();

        //     LLMrsSessionSerial {
        //         session_snapshot,
        //         llm_session,
        //     }
        // }).collect();

        // rmp_serde::encode::write_named(&mut file, &vec)?;
        // Ok(())
    }

    pub fn deserialize_sessions(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        return Ok(());
        //we skip deserializing because we skipped serializing
        // let mut file = File::open(&self.data_path)?;
        // let sessions_serial: Vec<LLMrsSessionSerial> = rmp_serde::decode::from_read(&file)?;
        // let sessions: Vec<LLMrsSession> = sessions_serial.into_iter().map(|sess| sess.into()).collect();
        // self.sessions = DashMap::new();
        // sessions.into_iter().map(|sess| {
        //     self.sessions.insert(sess.llm_session.id, Arc::new(Mutex::new(sess)));
        // });
        // Ok(())
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
    async fn get_sessions(self: &Self, user: User) -> Result<Vec<LLMSession>, String> {
        let sesss: Vec<LLMSession> = self
            .sessions
            .iter()
            .filter(|ff| ff.value().llm_session.read().unwrap().user_id == user.id)
            .map(|ff| ff.value().llm_session.as_ref().read().unwrap().clone())
            .collect();
        return Ok(sesss);
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
            llm_session: Arc::new(RwLock::new(LLMSession {
                id: DbUuid(uuid),
                llm_uuid: DbUuid(self.uuid.clone()), // replace with actual llm_uuid
                user_id: user.id,                    // replace with actual user_id
                started: Utc::now(),
                last_called: Utc::now(),
                session_parameters: DbHashMap(params),
                items: DbVec(vec![]),
            })),
        };

        self.sessions.insert(uuid, new_session);

        Ok(uuid)
    } //uuid

    async fn prompt_session(
        self: &mut Self,
        session_id: Uuid,
        prompt: String,
        params: HashMap<String, Value>,
        user: User,
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

        let inf_params = llm::InferenceParameters {
            n_threads: self.user_settings.n_thread,
            n_batch: self.user_settings.n_batch,
            sampler: Arc::new(sampler),
        };
        let session_wrapped = self.sessions.get(&session_id).ok_or("session not found")?;
        let mut model_armed = session_wrapped
            .model_session
            .as_ref()
            .lock()
            .map_err(|err| format!("failed to acquire lock: {:?}", err))?;

        let mut llm_session_armed = session_wrapped
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

        diesel::insert_into(schema::llm_history)
            .values(new_item)
            .execute(conn);

        llm_session_armed.items.push(new_item.clone());
        llm_session_armed.last_called = Utc::now();

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

                        let update_item = llm_session_armed
                            .items
                            .iter_mut()
                            .find(|item| item.id.0 == item_id)
                            .ok_or("couldn't find session we just made")
                            .unwrap();

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

                        // TODO: send a prompt complete here
                        update_item.output.push_str(&t);
                        if cancellation.is_cancelled() {
                            println!("SENT CONCLUSION");
                            update_item.complete = true;
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
                    _ => match cancellation.is_cancelled() {
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

        let now = std::time::Instant::now();

        //llm.rs now supports infering model architecture, but we won't support it.
        let model_architecture: llm::ModelArchitecture = self
            .config
            .get("model_architecture")
            .ok_or("missing model architecture")?
            .to_string()
            .parse()
            .map_err(|err| format!("unsupported model architecture"))?;

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
            .map_err(|err| "fuckup loading")?,
        );

        Ok(())
    }

    async fn unload_llm(self: &Self) -> Result<(), String> {
        todo!()
    } //called by shutdown
}
