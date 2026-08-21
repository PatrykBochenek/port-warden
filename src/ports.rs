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

/// Find `count` distinct free ports within `[lo, hi]` (inclusive), scanning
/// upward from `lo`. Returns fewer than `count` (possibly empty) if the range
/// is exhausted — the caller decides whether that is an error.
pub fn find_free_ports_in_range(lo: u16, hi: u16, count: usize) -> Vec<u16> {
    let mut found = Vec::new();
    if lo == 0 || lo > hi || count == 0 {
        return found;
    }

    let mut cursor = lo;
    loop {
        // Port 0 is never a usable free port; skip it if it appears.
        if cursor != 0 && port_check::is_local_port_free(cursor) {
            found.push(cursor);
            if found.len() == count {
                break;
            }
        }
        if cursor == hi {
            break;
        }
        cursor += 1;
    }
    found
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
