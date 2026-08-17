/// Check if a port is available on localhost.
pub fn is_port_free(port: u16) -> bool {
    port_check::is_local_port_free(port)
}

/// Find a free port on localhost, optionally preferring a specific port.
pub fn find_free_port(preferred: Option<u16>) -> Option<u16> {
    if let Some(p) = preferred {
        if p != 0 && port_check::is_local_port_free(p) {
            return Some(p);
        }
    }
    port_check::free_local_port()
}

/// Wait up to ~1s for the port to become free, polling every 50ms.
///
/// Returns `true` once the port is actually free, so callers report the
/// truth instead of blindly claiming success.
pub fn wait_for_port_free(port: u16) -> bool {
    use std::time::{Duration, Instant};

    let deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < deadline {
        if port_check::is_local_port_free(port) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}
