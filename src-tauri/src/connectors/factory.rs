use crate::llm;
use std::collections::HashMap;
use serde_json::value::Value;
use serde_json::json;
use crate::connectors;
use chrono::DateTime;
use uuid::Uuid;
use chrono::Utc;
use std::sync::{Arc, RwLock};

pub fn factory_llms() -> Vec<llm::LLM> {
    vec![
        llm::LLM {
            id: "openai_ada".to_string(),
            organization: "openai".into(),
            family_id: "gpt".into(),
            name: "OpenAI Ada".to_string(),
            license: "Closed Source".to_string(),
            description: "openAi's Ada model".to_string(),
            downloaded_reason: "Factory Include".to_string(),
            downloaded_date: Utc::now(),
            last_called: RwLock::new(Option::Some(Utc::now())),
            requirements: "An OpenAI Api Key".into(),
            url: "".into(),
            homepage: "https://platform.openai.com/docs/introduction".into(),


            capabilities: HashMap::from([("TEXT_COMPLETION".into(), 2), ("CONVERSATION".into(), 2)]),
            history: Vec::new(),
            tags: vec!["openai".into()],

            uuid: Uuid::new_v4(),

            create_thread: true, //eventually false, true for testing
            connector_type: connectors::LLMConnectorType::OpenAI,
            config: HashMap::from([
                ("endpoint".to_string(), json!("completions")),
                ("model".to_string(), json!("text-davinci-ada"))]),
            parameters: HashMap::from([("temperature".into(), json!(0.5)), ("color".into(), json!("red"))]),
            user_parameters: vec!["temperature".into()],
            model_path: None,

        },
        llm::LLM {
            id: "openai_gpt4".to_string(),
            organization: "openai".into(),
            family_id: "gpt".into(),
            name: "OpenAI GPT-4".to_string(),
            license: "Closed Source".to_string(),
            description: "openAi's GPT-4 model".to_string(),
            downloaded_reason: "Factory Include".to_string(),
            downloaded_date: Utc::now(),
            last_called: RwLock::new(Option::Some(Utc::now())),
            requirements: "An OpenAI Api Key".into(),
            url: "".into(),
            homepage: "https://platform.openai.com/docs/introduction".into(),

            capabilities: HashMap::from([("TEXT_COMPLETION".into(), 10), ("CONVERSATION".into(), 10)]),
            tags: vec!["openai".into()],
            history: Vec::new(),

            uuid: Uuid::new_v4(),

            create_thread: true, //eventually false, true for testing
            connector_type: connectors::LLMConnectorType::OpenAI,
            config: HashMap::from([
                ("endpoint".to_string(), json!("completions")),
                ("model".to_string(), json!("text-davinci-ada"))]),
            parameters: HashMap::from([]),
            user_parameters: vec![],
            model_path: None,
        },
    ]
}

