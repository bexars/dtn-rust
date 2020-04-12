
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
    MsgSystem(crate::system::SystemMessage),
    ShutdownNow,
    MsgOk(String),
    MsgErr(String),  
}
