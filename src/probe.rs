use crate::model::PortProcess;
use crate::platform;

/// Find processes using `port`.
///
/// When `listen_only` is true, only listening sockets are considered (used by
/// `get_info`); otherwise TCP and UDP sockets in any state are included (used
/// by `kill`).
pub fn find_processes(port: u16, listen_only: bool) -> Vec<PortProcess> {
    platform::find_processes(port, listen_only)
}

#[cfg(test)]
#[cfg(any(target_os = "linux", target_os = "macos", windows))]
mod tests {
    use std::net::TcpListener;

    use crate::probe::find_processes;

    #[test]
    fn finds_listener_in_current_process() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let processes = find_processes(port, true);
        assert_eq!(
            processes.len(),
            1,
            "expected exactly one process on the port"
        );
        assert_eq!(processes[0].pid, std::process::id());
        assert!(!processes[0].name.is_empty());
    }

    #[test]
    fn fresh_port_has_no_processes() {
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let processes = find_processes(port, true);
        assert!(processes.is_empty());
    }
}
