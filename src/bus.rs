use crate::system::SystemModules;

#[derive(Debug, Clone)]
pub enum ModuleMsgEnum {
    MsgProcessing,   
    MsgClaManager,   
    MsgCLI,          
    MsgLogging,      
    MsgStorage,      
    MsgAppAgent,     
    MsgRouting(crate::routing::RoutingMessage),  // not for actual bundles      
    MsgConf(crate::conf::ConfMessage),
    MsgSystem(crate::system::SystemMessage),
    ShutdownNow,
    MsgOk(String),
    MsgErr(String),  
}
