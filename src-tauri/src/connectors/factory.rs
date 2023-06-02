use crate::llm;
use crate::connectors;
use chrono::DateTime;
use chrono::Utc;
use std::sync::{Arc, RwLock};

pub fn factory_llms() -> Vec<llm::LLM> {
    vec![
        llm::LLM {
            id: "openai_ada".to_string(),
            name: "OpenAI Ada".to_string(),
            description: "openAi's Ada model".to_string(),
            downloaded_reason: "Factory Include".to_string(),
            downloaded_date: Utc::now(),
            last_called: RwLock::new(Utc::now()),

            create_thread: true, //eventually false, true for testing
            connector_type: connectors::LLMConnectorType::OpenAI,
            config: vec![
                ("endpoint".to_string(), "completions".to_string()),
                ("model".to_string(), "text-davinci-ada".to_string())],
            parameters: vec![],
            user_parameters: vec![],

            connector: Option::None,

        },
        llm::LLM {
            id: "openai_gpt4".to_string(),
            name: "OpenAI GPT-4".to_string(),
            description: "openAi's GPT-4 model".to_string(),
            downloaded_reason: "Factory Include".to_string(),
            downloaded_date: Utc::now(),
            last_called: RwLock::new(Utc::now()),

            create_thread: true, //eventually false, true for testing
            connector_type: connectors::LLMConnectorType::OpenAI,
            config: vec![
                ("endpoint".to_string(), "completions".to_string()),
                ("model".to_string(), "text-davinci-ada".to_string())],
            parameters: vec![],
            user_parameters: vec![],

            connector: Option::None,
        },
    ]
}

