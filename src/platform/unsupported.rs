use crate::model::{KillError, PortProcess};

pub fn find_processes(_port: u16, _listen_only: bool) -> Vec<PortProcess> {
    Vec::new()
}

pub fn kill_pid(_pid: u32, _force: bool) -> Result<(), KillError> {
    Err(KillError::Other("Unsupported platform".to_string()))
}

pub fn kill_on_port(_port: u16, _force: bool) -> Result<bool, KillError> {
    Err(KillError::Other("Unsupported platform".to_string()))
}
