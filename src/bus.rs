
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleMsgEnum {
    MsgProcessing,   
    MsgClaManager,   
    MsgCLI,          
    MsgLogging,      
    MsgStorage,      
    MsgAppAgent,     
    MsgRouting,      
    MsgConf(crate::conf::ConfMessage),
    MsgSystem(crate::router::SystemMessage),
    ShutdownNow,
    MsgOk(String),
    MsgErr(String),  
}
