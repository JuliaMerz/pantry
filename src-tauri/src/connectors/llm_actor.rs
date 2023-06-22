use crate::connectors;
use crate::llm;
use crate::user::User;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tiny_tokio_actor::*;
use connectors::LLMInternalWrapper;

use uuid::Uuid;



//src/connectors/llm_actor.rs

pub struct LLMActor {
    pub loaded: bool,
    pub uuid: Uuid,
    pub llm_connector: connectors::LLMConnectorType,
    pub llm_internal: Box<dyn connectors::LLMInternalWrapper>,
    pub config: HashMap<String, Value>,
    pub data_path: PathBuf,
}


#[async_trait]
impl Actor<connectors::SysEvent> for LLMActor {
    async fn pre_start(&mut self, ctx: &mut ActorContext<connectors::SysEvent>) -> Result<(), ActorError> {
        ctx.system.publish(connectors::SysEvent(format!("Actor '{}' started.", ctx.path)));
        match self.llm_internal.load_llm().await {
            Ok(_) => Ok(()),
            Err(err) => Err(ActorError::CreateError(err))
        }
    }

    async fn pre_restart(&mut self, ctx: &mut ActorContext<connectors::SysEvent>, error: Option<&ActorError>) -> Result<(), ActorError> {
        ctx.system.publish(connectors::SysEvent(format!("Actor '{}' is restarting due to {:#?}", ctx.path, error)));
        self.pre_start(ctx).await
    }

    async fn post_stop(&mut self, ctx: &mut ActorContext<connectors::SysEvent>) {
        match self.llm_internal.unload_llm().await {
            Ok(_) => ctx.system.publish(connectors::SysEvent(format!("Actor '{}' stopped.", ctx.path))),
            Err(err) => ctx.system.publish(connectors::SysEvent(format!("Actor '{}' failed to stop cleanly: {}.", ctx.path, err.to_string()))),
        }

    }

}

// This API NEEEDS
// Status
// CallLLM
// CreateSession
// PromptSession
//
// Actor also needs:
// Bootup
// Shutdown
//
#[derive(Clone, Debug)]
pub struct IDMessage();
impl Message for IDMessage {
    type Response = Result<String, String>;
}

#[derive(Clone, Debug)]
struct StatusMessage();
impl Message for StatusMessage{
    type Response = Result<String, String>;
}



#[derive(Clone, Debug)]
pub struct CallLLMMessage(pub String, pub HashMap<String, Value>, pub User);
 // strin 1 is text, string 2 is parameters_json
impl Message for CallLLMMessage {
    type Response = Result<mpsc::Receiver<connectors::LLMEvent>, String>;
}

#[derive(Clone, Debug)]
pub struct CreateSessionMessage(pub HashMap<String, Value>, pub User);
 // initial prompt (may be empty), HashMap of Params.
impl Message for CreateSessionMessage {
    // Return session_id
    type Response = Result<Uuid, String>;
}

#[derive(Clone, Debug)]
pub struct PromptSessionMessage(pub Uuid, pub String, pub User);
 // session_id, prompt
impl Message for PromptSessionMessage {
    type Response = Result<mpsc::Receiver<connectors::LLMEvent>, String>;
}


#[derive(Clone, Debug)]
pub struct GetLLMSessionsMessage {
    pub user: User,
}

impl Message for GetLLMSessionsMessage {
    type Response = Result<Vec<llm::LLMSession>, String>;
}

#[async_trait]
impl Handler<connectors::SysEvent, GetLLMSessionsMessage> for LLMActor {
    async fn handle(&mut self, msg: GetLLMSessionsMessage, ctx: &mut ActorContext<connectors::SysEvent>) -> Result<Vec<llm::LLMSession>, String> {
        self.llm_internal.as_ref().get_sessions(msg.user).await
    }
}



#[async_trait]
impl Handler<connectors::SysEvent, IDMessage> for LLMActor {
    async fn handle(&mut self, msg: IDMessage, ctx: &mut ActorContext<connectors::SysEvent>) -> Result<String, String> {
        // Err("ba".into())
        Ok(ctx.path.clone().to_string())
    }
}

#[async_trait]
impl Handler<connectors::SysEvent, StatusMessage> for LLMActor {
    async fn handle(&mut self, msg: StatusMessage, ctx: &mut ActorContext<connectors::SysEvent>) -> Result<String, String> {
        Err("ba".into())
    }
}


#[async_trait]
impl Handler<connectors::SysEvent, CallLLMMessage> for LLMActor {
    async fn handle(&mut self, msg: CallLLMMessage, ctx: &mut ActorContext<connectors::SysEvent>) -> Result<mpsc::Receiver<connectors::LLMEvent>, String> {
        self.llm_internal.call_llm(msg.0, msg.1, msg.2).await
    }

}

#[async_trait]
impl Handler<connectors::SysEvent, CreateSessionMessage> for LLMActor {
    async fn handle(&mut self, msg: CreateSessionMessage, ctx: &mut ActorContext<connectors::SysEvent>) -> Result<Uuid, String> {
        self.llm_internal.create_session(msg.0, msg.1).await
    }
}


#[async_trait]
impl Handler<connectors::SysEvent, PromptSessionMessage> for LLMActor {
    async fn handle(&mut self, msg: PromptSessionMessage, ctx: &mut ActorContext<connectors::SysEvent>) -> Result<mpsc::Receiver<connectors::LLMEvent>, String> {
        self.llm_internal.prompt_session(msg.0, msg.1, msg.2).await
    }
}
