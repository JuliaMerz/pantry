use crate::llm;
use chrono::Utc;
use crate::llm::SerializableTS;
use crate::state;
use std::sync::{Arc, RwLock};
#[derive(serde::Serialize)]
pub struct CommandResponse<T> {
    data: T,
}


#[tauri::command]
pub async fn get_requests(state: tauri::State<'_, state::GlobalState>) -> Result<CommandResponse<Vec<llm::LLMRequest>>, String> {
    // let requests = state.get_requests().await;
    let mock_llm =  llm::LLMInfo {
        id: "llm_id".into(),
        name: "llmname".into(),
        description: "I'm a little llm, short and stout!".into(),
    };
    let mock = llm::LLMRequest {
        llm_info: mock_llm,
        source: "mock".into(),
        requester: "fake".into()
    };
    Ok(CommandResponse { data: vec![mock]})
    // Err("boop".into())
}

#[tauri::command]
pub async fn active_llms(state: tauri::State<'_, state::GlobalState>) -> Result<CommandResponse<Vec<Arc<llm::LLMRunning>>>, String> {
    let active_llms_iter = state.running_llms.clone().into_values();//.collect();
    let mut active_llms = Vec::new();
    for val in active_llms_iter {
        active_llms.push(val);

    }
    let mock_llm =  llm::LLMInfo {
        id: "llm_id".into(),
        name: "llmname".into(),
        description: "I'm a little llm, short and stout!".into(),
    };
    let mock = llm::LLMRunning {
        llm_info: mock_llm,
        last_called: RwLock::new(llm::SerializableTS{time: Option::Some(Utc::now())}),
        downloaded: "dowwwn".into(),
        activated: "activvvvvv".into(),
    };
    active_llms.push(Arc::new(mock));
    Ok(CommandResponse { data: active_llms })
}

// #[tauri::command]
// pub async fn available_llms(state: tauri::State<'_, llm::GlobalState>) -> Result<CommandResponse<Vec<llm::LLMAvailable>>, String> {
//     let available_llms = state.available_llms().await;
//     Ok(CommandResponse { data: available_llms })
// }

// #[tauri::command]
// pub async fn load_llm(id: String, state: tauri::State<'_, llm::GlobalState>) -> Result<(), String> {
//     state.load_llm(id).await
// }

// #[tauri::command]
// pub async fn unload_llm(id: String, state: tauri::State<'_, llm::GlobalState>) -> Result<(), String> {
//     state.unload_llm(id).await
// }

// #[tauri::command]
// pub async fn download_llm(id: String, state: tauri::State<'_, llm::GlobalState>) -> Result<(), String> {
//     state.download_llm(id).await
// }

// #[tauri::command]
// pub async fn download_llm(id: String, state: tauri::State<'_, llm::GlobalState>) -> Result<(), String> {
//     state.download_llm(id).await
// }


