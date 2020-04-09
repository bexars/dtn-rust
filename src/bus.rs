
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
    SystemMsg(crate::router::SystemMessage),
    ShutdownNow,
    Error(String),  
}
