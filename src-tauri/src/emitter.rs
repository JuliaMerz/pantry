use crate::connectors::LLMEvent;
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc;

#[derive(Clone, serde::Serialize, Debug)]
#[serde(tag = "type")]
pub enum EventType {
    LLMResponse(LLMEvent),
    DownloadProgress { progress: String },
    DownloadCompletion,
    DownloadError { message: String },
    ChannelClose, //Universally at the end of a channel
    Other,
}

#[derive(Clone, serde::Serialize, Debug)]
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
    let payload = EventPayload {
        stream_id: stream_id.to_string(),
        event: EventType::ChannelClose,
    };
    // println!("Emitting event {:?} on {:?}", payload, channel);
    app.emit_all(&channel, &payload).unwrap();
}
