use tauri::{AppHandle, Wry, Manager};
use tokio::sync::mpsc;

#[derive(Clone, serde::Serialize)]
#[serde(tag="type")]
pub enum EventType {
  PromptProgress{previous: String, next: String}, // Next words of an LLM.
  PromptCompletion{previous: String}, // Finished the prompt
  PromptError{message: String},
  DownloadProgress{progress: String},
  DownloadCompletion,
  DownloadError{message: String},
  ChannelClose,  //Universally at the end of a channel
  Other,
}

#[derive(Clone, serde::Serialize)]
pub struct EventPayload {
  pub stream_id: String,
  pub event: EventType,
}

// Defines the conversion function type
type ConversionFunc<T> = fn(String, T) -> Result<EventPayload, String>;

pub async fn send_events<T: 'static>(
    channel: String,
    stream_id: String,
    mut rx: mpsc::Receiver<T>,
    app: AppHandle,
    convert: ConversionFunc<T>)
{
    while let Some(payload_inner) = rx.recv().await {
        match convert(stream_id.to_string(), payload_inner){
            Ok(payload) => app.emit_all(&channel, &payload).unwrap(),
            Err(_) => ()
        }

    }

    // Channel has closed, emit completion event
    let payload = EventPayload {
        stream_id: channel.to_string(),
        event: EventType::ChannelClose,
    };
    app.emit_all(&channel, &payload).unwrap();
}

