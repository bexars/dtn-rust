use super::cla_handle::ClaHandle;

struct LoopbackCLA {
    handle: ClaHandle,
}

impl LoopbackCLA {
    pub fn new(handle: ClaHandle, conf: super::AdapterConfiguration) -> LoopbackCLA {
        
        
        Self { handle, }
    }
}