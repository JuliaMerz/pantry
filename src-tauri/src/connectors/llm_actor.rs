use crate::connectors;
use std::sync::{Arc, RwLock};
use tiny_tokio_actor::*;



//src/llm_actor.rs

pub struct LLMActor {
    pub llm_connector: connectors::LLMConnectorType,
    pub llm_internal: Box<dyn connectors::LLMInternalWrapper>,
}

impl Actor<connectors::SysEvent> for LLMActor {
}


#[async_trait]
impl Handler<connectors::SysEvent, ID> for LLMActor {
    async fn handle(&mut self, msg: ID, ctx: &mut ActorContext<connectors::SysEvent>) -> String {
        ctx.path.clone().to_string()
    }
}


////equivalent of call
//#[derive(Message)]
//#[rtype(result = "String")]
//struct Call(String, String);
//// strin 1 is text, string 2 is parameters_json


////equivalent of status
//#[derive(Message)]
//#[rtype(result = "String")]
//struct Status();


//equivalent of load
#[derive(Clone, Debug)]
struct ID();

impl Message for ID {
    type Response = String;
}


////equivalent of unload
//#[derive(Message)]
//#[rtype(result = "bool")]
//struct Shutdown();

