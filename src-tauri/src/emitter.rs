use crate::connectors::LLMEvent;
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc;

#[derive(Clone, serde::Serialize, Debug)]
#[serde(tag = "type")]
pub enum EmitterEventPayload {
    LLMResponse(LLMEvent),
    DownloadProgress { progress: String },
    DownloadCompletion,
    DownloadError { message: String },
    ChannelClose, //Universally at the end of a channel
    Other,
}

#[derive(Clone, serde::Serialize, Debug)]
pub struct EmitterEvent {
    pub stream_id: String,
    pub event: EmitterEventPayload,
}

// Defines the conversion function type
type ConversionFunc<T> = fn(String, T) -> Result<EmitterEvent, String>;

pub async fn send_events<T: 'static>(
    channel: String,
    stream_id: String,
    mut rx: mpsc::Receiver<T>,
    app: AppHandle,
    convert: ConversionFunc<T>,
) {
    println!("STARTING RECEIVER");
    while let Some(payload_inner) = rx.recv().await {
        match convert(stream_id.to_string(), payload_inner) {
            Ok(payload) => {
                // println!("Emitting event {:?} on {:?}", payload, channel);
                app.emit_all(&channel, &payload).unwrap()
            }
            Err(_) => (),
        }
    }

    // Channel has closed, emit completion event
    let payload = EmitterEvent {
        stream_id: stream_id.to_string(),
        event: EmitterEventPayload::ChannelClose,
    };
    // println!("Emitting event {:?} on {:?}", payload, channel);
    app.emit_all(&channel, &payload).unwrap();
}
