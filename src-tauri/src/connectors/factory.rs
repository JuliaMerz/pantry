use crate::connectors;
use crate::database_types::*;
use crate::llm;

use chrono::Utc;
use serde_json::json;

use std::collections::HashMap;

use uuid::Uuid;

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
            last_called: None,
            requirements: "An OpenAI Api Key".into(),
            url: "".into(),
            homepage: "https://platform.openai.com/docs/introduction".into(),

            capabilities: DbHashMapInt(HashMap::from([
                ("TEXT_COMPLETION".into(), 2),
                ("CONVERSATION".into(), 2),
            ])),
            tags: DbVec(vec!["openai".into()]),

            uuid: DbUuid(Uuid::new_v4()),

            create_thread: false, //eventually false, true for testing
            connector_type: connectors::LLMConnectorType::OpenAI,
            config: DbHashMap(HashMap::from([
                ("endpoint".to_string(), json!("completions")),
                ("model".to_string(), json!("text-davinci-ada")),
            ])),
            parameters: DbHashMap(HashMap::from([
                ("temperature".into(), json!(0.5)),
                ("color".into(), json!("red")),
            ])),
            user_parameters: DbVec(vec!["temperature".into()]),
            session_parameters: DbHashMap(HashMap::from([])),
            user_session_parameters: DbVec(vec![]),
            model_path: DbOptionPathbuf(None),
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
            last_called: None,
            requirements: "An OpenAI Api Key".into(),
            url: "".into(),
            homepage: "https://platform.openai.com/docs/introduction".into(),

            capabilities: DbHashMapInt(HashMap::from([
                ("TEXT_COMPLETION".into(), 10),
                ("CONVERSATION".into(), 10),
            ])),
            tags: DbVec(vec!["openai".into()]),

            uuid: DbUuid(Uuid::new_v4()),

            create_thread: true, //eventually false, true for testing
            connector_type: connectors::LLMConnectorType::OpenAI,
            config: DbHashMap(HashMap::from([
                ("endpoint".to_string(), json!("completions")),
                ("model".to_string(), json!("text-davinci-ada")),
            ])),
            session_parameters: DbHashMap(HashMap::from([])),
            user_session_parameters: DbVec(vec![]),
            parameters: DbHashMap(HashMap::from([])),
            user_parameters: DbVec(vec![]),
            model_path: DbOptionPathbuf(None),
        },
    ]
}
