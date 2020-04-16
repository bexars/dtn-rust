#[derive(Debug, Clone)]
pub enum ModuleMsgEnum {
    MsgProcessing(crate::processor::ProcessorMsg),   
    MsgClaManager,   
    MsgCla(crate::cla::ClaMessage),
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
