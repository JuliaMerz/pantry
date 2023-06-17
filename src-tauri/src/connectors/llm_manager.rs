use crate::{connectors, error::PantryError};
use serde_json::Value;
use crate::connectors::{SysEvent};
use tiny_tokio_actor::*;
use std::collections::HashMap;
use crate::connectors::llm_actor::LLMActor;
use std::rc::Rc;

// Define some general bookkeeping for the actor framework



// Special actor that manages LLMActors
// This moves us out of the tauri state events thread, and into
// to be started.
#[derive(Default)]
pub struct LLMManagerActor {
    // This is the source of truth for running LLMs.
    pub active_llm_actors: HashMap<String, ActorRef<SysEvent, LLMActor>>,
}

impl Actor<SysEvent> for LLMManagerActor {}


// Message to create a new LLMActor
#[derive(Clone, Debug)]
pub struct CreateLLMActorMessage(pub String, pub connectors::LLMConnectorType, pub HashMap<String, Value>);
// id, connector type, config[]

impl Message for CreateLLMActorMessage {
    type Response = Result<ActorRef<SysEvent, LLMActor>, PantryError>;
}
// (llm_id, create_thread, config)

#[derive(Clone, Debug)]
pub struct PingMessage();
impl Message for PingMessage {
    type Response = Result<Vec<String>, PantryError>;
}


//#[derive(Clone, Debug)]
//pub struct GetLLMActorMessage(pub String);
////llm_id

//impl Message for GetLLMActorMessage {
//    type Response = Result<ActorRef<SysEvent, LLMActor>, PantryError>;
//}


#[async_trait]
impl Handler<SysEvent, CreateLLMActorMessage> for LLMManagerActor {
    async fn handle(&mut self, msg: CreateLLMActorMessage, ctx: &mut ActorContext<SysEvent>) -> Result<ActorRef<SysEvent, LLMActor>, PantryError> {
        println!("Running createllmactor handler");

        let conn: connectors::LLMConnectorType = msg.1.clone();
        let connection = connectors::get_new_llm_connector(conn.clone(), msg.2.clone());
        let llm_act = LLMActor {
            loaded: false, //LLM actors need to have init called on them
            llm_internal: connection,
            llm_connector: conn.clone(),
            config: msg.2.clone(),
        };

        match ctx.get_or_create_child(&msg.0, || llm_act).await {
            Ok(act_ref) => {
                println!("Created child");
                self.active_llm_actors.insert(msg.0.clone(), act_ref.clone());
                Ok(act_ref)
            }
            Err(act_er) => Err(PantryError::ActorFailure(act_er))
        }

    }
}

// #[async_trait]
// impl Handler<SysEvent, GetLLMActorMessage> for LLMManagerActor {
//     async fn handle(&mut self, msg: GetLLMActorMessage, ctx: &mut ActorContext<SysEvent>) -> Result<LLMConnector, PantryError> {
//         match self.active_llm_actors.get(&msg.0) {
//             Some(llm_conn) => Ok(llm_conn.clone()),
//             None => Err(PantryError::LLMNotRunning)
//         }

//     }
// }





#[async_trait]
impl Handler<SysEvent, PingMessage> for LLMManagerActor {
    async fn handle(&mut self, msg: PingMessage, _ctx: &mut ActorContext<SysEvent>) -> Result<Vec<String>, PantryError>{
        let mut ve:Vec<String> = Vec::new();
        for (key, _) in self.active_llm_actors.clone().into_iter() {
            ve.push(key.clone());
        }
        Ok(ve)
    }
}

