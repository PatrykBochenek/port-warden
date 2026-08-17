/// A process found to be using a port.
#[derive(Debug, Clone)]
pub struct PortProcess {
    pub pid: u32,
    pub name: String,
    pub cmd: String,
}

/// Why terminating a process failed.
#[derive(Debug)]
pub enum KillError {
    /// We lack the privileges required to terminate the process.
    Permission,
    /// Any other failure, with a human-readable reason.
    Other(String),
}
