use crate::{connectors, error::PantryError};
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
    // This _might_ eventually have some utility, BUT architecture wise at
    // the moment, an LLM->LLMConnector stores the Addr<LLMActor> and SHOULD
    // be the only place that it gets accessed. T
    pub llms: HashMap<String, ActorRef<SysEvent, LLMActor>>,
}

impl Actor<SysEvent> for LLMManagerActor {}


// Message to create a new LLMActor
#[derive(Clone, Debug)]
pub struct CreateLLMActorMessage(pub String, pub connectors::LLMConnectorType, pub Vec<(String, String)>);
// id, connector type, config[]

impl Message for CreateLLMActorMessage {
    type Response = Result<ActorRef<SysEvent, LLMActor>, PantryError>;
}
// (llm_id, create_thread, config)

#[derive(Clone, Debug)]
pub struct Ping();
impl Message for Ping {
    type Response = Result<Vec<String>, PantryError>;
}

#[async_trait]
impl Handler<SysEvent, CreateLLMActorMessage> for LLMManagerActor {
    async fn handle(&mut self, msg: CreateLLMActorMessage, ctx: &mut ActorContext<SysEvent>) -> Result<ActorRef<SysEvent, LLMActor>, PantryError> {
        println!("Running createllmactor handler");

        let conn: connectors::LLMConnectorType = msg.1.clone();
        let connection = connectors::get_new_llm_connector(conn.clone());
        let llm_act = LLMActor {
            llm_internal: connection,
            llm_connector: conn.clone(),
        };

        match ctx.get_or_create_child(&msg.0, || llm_act).await {
            Ok(act_ref) => {
                println!("Created child");
                self.llms.insert(msg.0.clone(), act_ref.clone());
                Ok(act_ref)
            }
            Err(act_er) => Err(PantryError::LLMStartupFailed(act_er))
        }

    }
}

#[async_trait]
impl Handler<SysEvent, Ping> for LLMManagerActor {
    async fn handle(&mut self, msg: Ping, _ctx: &mut ActorContext<SysEvent>) -> Result<Vec<String>, PantryError>{
        let mut ve:Vec<String> = Vec::new();
        for (key, _) in self.llms.clone().into_iter() {
            ve.push(key.clone());
        }
        Ok(ve)
    }
}

