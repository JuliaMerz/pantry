use crate::connectors::{LLMInternalWrapper, LLMEvent, LLMEventInternal};
use crate::llm::{LLMSession, LLMHistoryItem};
use uuid::Uuid;
use crate::user::User;
use tiny_tokio_actor::*;
use std::path::PathBuf;
use tokio::sync::mpsc;
use serde_json::Value;
use std::collections::HashMap;
use llm::VocabularySource;
use std::sync::{Arc, RwLock};

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

    model: RwLock<Option<Box<dyn llm::Model>>>

}

impl LLMrsConnector {
    pub fn new(uuid: Uuid, data_path: PathBuf, config: HashMap<String, Value>) -> LLMrsConnector {
        LLMrsConnector {
            config: config,
            model: RwLock::new(None)
        }
    }
}

#[async_trait]
impl LLMInternalWrapper for LLMrsConnector {
    async fn call_llm(self: &mut Self, msg: String, params: HashMap<String, Value>, user: User) -> Result<(Uuid, mpsc::Receiver<LLMEvent>), String> {

        todo!()
    }
    async fn get_sessions(self: &Self, user: User) -> Result<Vec<LLMSession>, String> {
        todo!()
    }

    async fn create_session(self: &mut Self, params: HashMap<String, Value>, user: User) -> Result<Uuid, String> {

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

        let inf_pams = llm::InferenceParameters {
            n_threads: 8,
            n_batch: 8,
            sampler: Arc::new(sampler),
        };

        todo!()
    } //uuid
    async fn prompt_session(self: &mut Self, session_id: Uuid, msg: String, user: User) -> Result<mpsc::Receiver<LLMEvent>, String> {
        todo!()
    }

    async fn load_llm(self: &mut Self) -> Result<(), String> {
        let vocab_source: llm::VocabularySource = match (self.config.get("vocabulary_path"), self.config.get("vocabulary_repository")) {
            (Some(_), Some(_)) => {
                panic!("Cannot specify both --vocabulary-path and --vocabulary-repository");
            }
            (Some(path), None) => llm::VocabularySource::HuggingFaceTokenizerFile(PathBuf::from(path.to_string())),
            (None, Some(repo)) => llm::VocabularySource::HuggingFaceRemote(repo.to_string()),
            (None, None) => llm::VocabularySource::Model,
        };

        let now = std::time::Instant::now();
        let model_architecture = self.config.get("model_architecture")
            .ok_or("missing model architecture")?
            .to_string()
            .parse()
            .map_err(|err| {format!("unsupported model architecture")})?;
        let model_path = PathBuf::from(self.config.get("model_path")
            .ok_or("missing model path")?
            .to_string());

        let mut writer = self.model.write().unwrap();
        *writer = Some(llm::load_dynamic(
            model_architecture,
            &model_path,
            vocab_source,
            Default::default(),
            llm::load_progress_callback_stdout,
        ).map_err(|err| "fuckup loading")?);


        Ok(())
    }

    async fn unload_llm(self: &Self, ) -> Result<(), String> {
        todo!()

    }//called by shutdown
}
