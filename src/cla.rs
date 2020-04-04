pub mod cla_handle;
pub mod cla_manager;
pub mod stcp_server;

pub enum ClaRW {
    R,
    RW,
    W,
}

pub enum ClaType {
    StcpListener,
    StcpSender,
    LoopBack,
}