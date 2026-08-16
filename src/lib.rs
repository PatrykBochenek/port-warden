//! port-warden — cross-platform Python port management made simple.
//!
//! Thin PyO3 bindings around the [`port_check`] crate for TCP port
//! availability, plus native process introspection and termination for the
//! process(es) bound to a given port.
//!
//! The process lookups are implemented natively per platform — no
//! subprocesses (`lsof`, `netstat`, `tasklist`, `wmic`, …) are spawned:
//!
//! * **Linux** — `/proc/net/{tcp,tcp6,udp,udp6}` plus `/proc/<pid>/fd`
//!   scanning via the [`procfs`] crate.
//! * **macOS** — `libproc` (`proc_pidinfo`) via the [`libproc`] crate.
//! * **Windows** — `GetExtended{Tcp,Udp}Table` plus ToolHelp32 snapshots via
//!   [`windows-sys`].
//!
//! The platform-specific socket→process logic is adapted from the
//! MIT-licensed [`killport`](https://github.com/jkfran/killport) project.

use pyo3::prelude::*;
use pyo3::types::PyModule;
use pyo3::IntoPyObjectExt;
use std::collections::HashMap;

/// A process found to be using a port.
#[derive(Debug, Clone)]
struct PortProcess {
    pid: u32,
    name: String,
    cmd: String,
}

/// Why terminating a process failed.
#[derive(Debug)]
enum KillError {
    /// We lack the privileges required to terminate the process.
    Permission,
    /// Any other failure, with a human-readable reason.
    Other(String),
}

/// Convert an infallible Rust value into a Python object.
fn to_pyobj<'py>(py: Python<'py>, value: impl IntoPyObject<'py>) -> Py<PyAny> {
    value.into_py_any(py).expect("conversion is infallible")
}

/// Build the public `{"pid", "name", "cmd"}` info dict for a process.
fn info_dict(py: Python<'_>, process: &PortProcess) -> HashMap<String, Py<PyAny>> {
    let mut info = HashMap::new();
    info.insert("pid".to_string(), to_pyobj(py, process.pid));
    info.insert("name".to_string(), to_pyobj(py, process.name.clone()));
    info.insert("cmd".to_string(), to_pyobj(py, process.cmd.clone()));
    info
}

/// Wait up to ~1s for the port to become free, polling every 50ms.
///
/// Returns `true` once the port is actually free, so callers report the
/// truth instead of blindly claiming success.
fn wait_for_port_free(port: u16) -> bool {
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

// =============================================================================
// PORT AVAILABILITY
// =============================================================================

/// Check if a port is available on localhost.
///
/// Args:
///     port: Port number to check (1-65535)
///
/// Returns:
///     True if port is available, False if in use
///
/// Example:
///     >>> import port_warden
///     >>> port_warden.is_available(8000)
///     True
#[pyfunction]
fn is_available(port: u16) -> bool {
    if port == 0 {
        return false;
    }
    port_check::is_local_port_free(port)
}

// =============================================================================
// FIND FREE PORT
// =============================================================================

/// Find a free port on localhost.
///
/// Args:
///     preferred: Optional preferred port number. If provided and free, returns it.
///                If occupied, returns a different free port.
///
/// Returns:
///     A free port number
///
/// Raises:
///     OSError: If no free port could be found
///
/// Note:
///     The returned port can be taken by another process between this check
///     and when you actually bind to it (TOCTOU). Prefer binding to port 0
///     and letting the OS pick if you need a race-free reservation.
///
/// Example:
///     >>> port = port_warden.find_free(8000)
///     >>> port = port_warden.find_free()  # Any free port
#[pyfunction]
#[pyo3(signature = (preferred=None))]
fn find_free(preferred: Option<u16>) -> PyResult<u16> {
    if let Some(p) = preferred {
        if p != 0 && port_check::is_local_port_free(p) {
            return Ok(p);
        }
    }
    port_check::free_local_port()
        .ok_or_else(|| pyo3::exceptions::PyOSError::new_err("Could not find a free port"))
}

// =============================================================================
// WAIT UNTIL FREE
// =============================================================================

/// Wait for a port to become free.
///
/// Args:
///     port: Port number to wait for
///     timeout: Maximum seconds to wait (default: 30)
///
/// Returns:
///     True if port became free, False if timeout reached
///
/// Example:
///     >>> port_warden.wait_until_free(5432, timeout=30)
///     True
#[pyfunction]
#[pyo3(signature = (port, timeout=30))]
fn wait_until_free(py: Python<'_>, port: u16, timeout: u64) -> bool {
    if port == 0 {
        return false;
    }
    // Release the GIL while polling so other Python threads can run
    // (e.g. the thread that will free the port we are waiting for).
    py.detach(move || {
        use std::time::{Duration, Instant};

        let deadline = Instant::now() + Duration::from_secs(timeout);

        while Instant::now() < deadline {
            if port_check::is_local_port_free(port) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        // Final check
        port_check::is_local_port_free(port)
    })
}

// =============================================================================
// GET PROCESS INFO
// =============================================================================

/// Get information about the process using a port.
///
/// Args:
///     port: Port number to investigate
///
/// Returns:
///     Dict with pid, name, and cmd keys, or None if the port is free
///     (or its owner is not visible to us, e.g. it belongs to another user).
///
/// Note:
///     `cmd` is the full command line on Linux, the executable path on
///     macOS, and the executable name on Windows.
///
/// Example:
///     >>> port_warden.get_info(8000)
///     {'pid': 1234, 'name': 'python', 'cmd': 'python app.py'}
///     >>> port_warden.get_info(9999)
///     None
#[pyfunction]
fn get_info(py: Python<'_>, port: u16) -> Option<HashMap<String, Py<PyAny>>> {
    if port == 0 || port_check::is_local_port_free(port) {
        return None;
    }

    // Socket-table scans can block on /proc or process enumeration; release
    // the GIL so other Python threads can run while we look.
    let process = py.detach(move || platform::find_processes(port, true).into_iter().next());

    process.map(|p| info_dict(py, &p))
}

// =============================================================================
// KILL PROCESS
// =============================================================================

/// Kill the process(es) using a port.
///
/// Args:
///     port: Port number
///     force: If True, use SIGKILL (SIGTERM if False). Default: False.
///            On Windows the process is always terminated forcefully.
///
/// Returns:
///     True if the port became free (or was already free)
///
/// Raises:
///     PermissionError: If insufficient permissions
///     OSError: If kill failed for another reason
///
/// Example:
///     >>> port_warden.kill(8000)
///     True
///     >>> port_warden.kill(8000, force=True)
///     True
#[pyfunction]
#[pyo3(signature = (port, force=false))]
fn kill(py: Python<'_>, port: u16, force: bool) -> PyResult<bool> {
    if port == 0 || port_check::is_local_port_free(port) {
        return Ok(true);
    }

    let freed = py.detach(move || platform::kill_on_port(port, force));

    match freed {
        Ok(true) => Ok(true),
        Ok(false) => Err(pyo3::exceptions::PyOSError::new_err(format!(
            "Could not free port {port}: no matching process was found or it did not exit"
        ))),
        Err(KillError::Permission) => Err(pyo3::exceptions::PyPermissionError::new_err(format!(
            "Permission denied to kill the process(es) using port {port} \
             (try running with elevated privileges)"
        ))),
        Err(KillError::Other(msg)) => Err(pyo3::exceptions::PyOSError::new_err(msg)),
    }
}

// =============================================================================
// SCAN MULTIPLE PORTS
// =============================================================================

/// Scan multiple ports and return info for each.
///
/// Args:
///     ports: List of port numbers
///
/// Returns:
///     Dict mapping port numbers to process info (or None if free)
///
/// Example:
///     >>> port_warden.scan([8000, 8001, 5432])
///     {8000: {'pid': 1234, 'name': 'python', 'cmd': 'python app.py'}, 8001: None, 5432: None}
#[pyfunction]
fn scan(py: Python<'_>, ports: Vec<u16>) -> HashMap<u16, Option<HashMap<String, Py<PyAny>>>> {
    ports.into_iter().map(|p| (p, get_info(py, p))).collect()
}

// =============================================================================
// UNIX PLATFORM (Linux: procfs, macOS: libproc)
// =============================================================================

#[cfg(unix)]
fn kill_pid_unix(pid: u32, force: bool) -> Result<(), KillError> {
    use nix::errno::Errno;
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    let signal = if force {
        Signal::SIGKILL
    } else {
        Signal::SIGTERM
    };
    match kill(Pid::from_raw(pid as i32), signal) {
        Ok(()) => Ok(()),
        Err(Errno::ESRCH) => Ok(()), // process already gone
        Err(Errno::EPERM) => Err(KillError::Permission),
        Err(e) => Err(KillError::Other(e.to_string())),
    }
}

#[cfg(unix)]
mod platform {
    use super::{kill_pid_unix, wait_for_port_free, KillError, PortProcess};

    // ------------------------------------------------------------------
    // Socket-table probes
    // ------------------------------------------------------------------

    #[cfg(target_os = "linux")]
    mod probe {
        use super::PortProcess;
        use procfs::process::FDTarget;
        use std::collections::HashSet;

        /// Collect the `/proc` socket inodes for `port`.
        ///
        /// When `listen_only` is true, only TCP sockets in the LISTEN state
        /// are considered (used by `get_info`); otherwise TCP and UDP sockets
        /// in any state are included (used by `kill`).
        fn socket_inodes(port: u16, listen_only: bool) -> HashSet<u64> {
            let mut inodes = HashSet::new();

            for table in [procfs::net::tcp(), procfs::net::tcp6()] {
                if let Ok(entries) = table {
                    for entry in entries {
                        if entry.local_address.port() == port
                            && (!listen_only || entry.state == procfs::net::TcpState::Listen)
                        {
                            inodes.insert(entry.inode);
                        }
                    }
                }
            }

            if !listen_only {
                for table in [procfs::net::udp(), procfs::net::udp6()] {
                    if let Ok(entries) = table {
                        for entry in entries {
                            if entry.local_address.port() == port {
                                inodes.insert(entry.inode);
                            }
                        }
                    }
                }
            }

            inodes
        }

        /// Map socket inodes to their owning processes in a single pass over
        /// `/proc/<pid>/fd`.
        fn processes_for_inodes(inodes: &HashSet<u64>) -> Vec<PortProcess> {
            let mut out = Vec::new();
            if inodes.is_empty() {
                return out;
            }

            let Ok(processes) = procfs::process::all_processes() else {
                return out;
            };

            'next: for process in processes.flatten() {
                let Ok(fds) = process.fd() else { continue };

                for fd in fds.flatten() {
                    if let FDTarget::Socket(inode) = fd.target {
                        if inodes.contains(&inode) {
                            out.push(PortProcess {
                                pid: process.pid() as u32,
                                name: process.stat().map(|s| s.comm).unwrap_or_default(),
                                cmd: process
                                    .cmdline()
                                    .ok()
                                    .map(|parts| parts.join(" "))
                                    .unwrap_or_default(),
                            });
                            // One entry per process, even when several fds match.
                            continue 'next;
                        }
                    }
                }
            }
            out
        }

        pub fn find_processes(port: u16, listen_only: bool) -> Vec<PortProcess> {
            processes_for_inodes(&socket_inodes(port, listen_only))
        }
    }

    #[cfg(target_os = "macos")]
    mod probe {
        use super::PortProcess;
        use libproc::libproc::bsd_info::BSDInfo;
        use libproc::libproc::file_info::{pidfdinfo, ListFDs, ProcFDType};
        use libproc::libproc::net_info::{SocketFDInfo, SocketInfoKind, TcpSIState};
        use libproc::libproc::proc_pid::{listpidinfo, name, pidinfo, pidpath};
        use libproc::processes::{pids_by_type, ProcFilter};
        use std::collections::HashSet;

        pub fn find_processes(port: u16, listen_only: bool) -> Vec<PortProcess> {
            let mut out = Vec::new();
            let mut seen: HashSet<i32> = HashSet::new();

            let Ok(procs) = pids_by_type(ProcFilter::All) else {
                return out;
            };

            'next: for p in procs {
                let pid = p as i32;
                if seen.contains(&pid) {
                    continue;
                }
                // Size the fd list from the process's actual fd count with a
                // little headroom (a fixed cap silently misses sockets in
                // fd-heavy processes); fall back to a generous default.
                let max_fds = pidinfo::<BSDInfo>(pid, 0)
                    .map(|info| info.pbi_nfiles as usize + 32)
                    .unwrap_or(4096);
                let Ok(fds) = listpidinfo::<ListFDs>(pid, max_fds) else {
                    continue; // EPERM for processes owned by other users
                };
                for fd in &fds {
                    if !matches!(ProcFDType::from(fd.proc_fdtype), ProcFDType::Socket) {
                        continue;
                    }
                    let Ok(socket) = pidfdinfo::<SocketFDInfo>(pid, fd.proc_fd) else {
                        continue;
                    };
                    let kind = SocketInfoKind::from(socket.psi.soi_kind);
                    let (local_port, is_listening) = match kind {
                        SocketInfoKind::In => unsafe {
                            (socket.psi.soi_proto.pri_in.insi_lport as u16, false)
                        },
                        SocketInfoKind::Tcp => unsafe {
                            let state = TcpSIState::from(socket.psi.soi_proto.pri_tcp.tcpsi_state);
                            (
                                socket.psi.soi_proto.pri_tcp.tcpsi_ini.insi_lport as u16,
                                matches!(state, TcpSIState::Listen),
                            )
                        },
                        _ => continue,
                    };
                    if u16::from_be(local_port) != port {
                        continue;
                    }
                    if listen_only && !is_listening {
                        continue;
                    }
                    // The process can exit between socket enumeration and the
                    // name lookup; a vanished process must not fail the scan.
                    let Ok(process_name) = name(pid) else {
                        continue 'next;
                    };
                    seen.insert(pid);
                    out.push(PortProcess {
                        pid: pid as u32,
                        name: process_name,
                        cmd: pidpath(pid).unwrap_or_default(),
                    });
                    continue 'next;
                }
            }
            out
        }
    }

    // ------------------------------------------------------------------
    // Shared Unix logic
    // ------------------------------------------------------------------

    pub fn find_processes(port: u16, listen_only: bool) -> Vec<PortProcess> {
        probe::find_processes(port, listen_only)
    }

    fn kill_pid(pid: u32, force: bool) -> Result<(), KillError> {
        kill_pid_unix(pid, force)
    }

    pub fn kill_on_port(port: u16, force: bool) -> Result<bool, KillError> {
        let processes = find_processes(port, false);
        if processes.is_empty() {
            // The port is busy but no matching process was found (for example
            // it is owned by another user and hidden from us). We cannot kill.
            return Ok(false);
        }
        let mut denied = false;
        let mut killed_any = false;
        for process in &processes {
            match kill_pid(process.pid, force) {
                Ok(()) => killed_any = true,
                Err(KillError::Permission) => denied = true,
                Err(KillError::Other(_)) => {}
            }
        }
        if denied && !killed_any {
            return Err(KillError::Permission);
        }
        // Give the kernel a moment and verify the port actually freed up
        // instead of blindly reporting success.
        Ok(wait_for_port_free(port))
    }

    #[cfg(test)]
    mod tests {
        use std::net::TcpListener;

        #[test]
        fn finds_listener_in_current_process() {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            let processes = super::find_processes(port, true);
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
            let processes = super::find_processes(port, true);
            assert!(processes.is_empty());
        }
    }
}

// =============================================================================
// WINDOWS PLATFORM (windows-sys)
// =============================================================================

#[cfg(windows)]
mod platform {
    use super::{wait_for_port_free, KillError, PortProcess};
    use std::collections::{HashMap, HashSet};
    use std::ptr::addr_of;
    use std::slice;
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_INSUFFICIENT_BUFFER, FALSE, HANDLE,
        INVALID_HANDLE_VALUE, NO_ERROR,
    };
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GetExtendedTcpTable, GetExtendedUdpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCP6TABLE_OWNER_PID,
        MIB_TCPROW_OWNER_PID, MIB_TCPTABLE_OWNER_PID, MIB_TCP_STATE_LISTEN, MIB_UDP6ROW_OWNER_PID,
        MIB_UDP6TABLE_OWNER_PID, MIB_UDPROW_OWNER_PID, MIB_UDPTABLE_OWNER_PID,
        TCP_TABLE_OWNER_PID_ALL, UDP_TABLE_OWNER_PID,
    };
    use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

    /// Scan one Win32 "extended table" for rows matching `port` and record
    /// their owning PIDs in `pids`.
    ///
    /// `$matches` receives each row and decides whether it belongs to `port`
    /// (and, for TCP, honours the `listen_only` filter).
    macro_rules! scan_table {
        ($getter:path, $family:expr, $class:expr, $table_ty:ty, $row_ty:ty, $port:expr, $pids:expr, $matches:expr) => {{
            let mut size: u32 = 0;
            // u32 buffer: guarantees the 4-byte alignment the MIB tables need.
            let mut buf: Vec<u32> = Vec::new();
            let result = loop {
                let result =
                    unsafe { $getter(buf.as_mut_ptr().cast(), &mut size, 0, $family, $class, 0) };
                if result == NO_ERROR {
                    break result;
                }
                if result == ERROR_INSUFFICIENT_BUFFER {
                    buf.resize((size as usize).div_ceil(4) + 1, 0);
                    continue;
                }
                return Err(format!("GetExtendedTable failed: {result:#x}"));
            };
            debug_assert_eq!(result, NO_ERROR);
            unsafe {
                let table = buf.as_ptr() as *const $table_ty;
                let count = addr_of!((*table).dwNumEntries).read_unaligned() as usize;
                let rows: &[$row_ty] =
                    slice::from_raw_parts(addr_of!((*table).table).cast(), count);
                for row in rows {
                    if $matches(row) {
                        let pid = row.dwOwningPid;
                        // PID 0 ([System Process]) owns TIME_WAIT rows and PID 4
                        // (System) owns kernel sockets; neither can nor should be
                        // terminated.
                        if pid != 0 && pid != 4 {
                            $pids.insert(pid);
                        }
                    }
                }
            }
        }};
    }

    fn collect_pids(port: u16, listen_only: bool, pids: &mut HashSet<u32>) -> Result<(), String> {
        scan_table!(
            GetExtendedTcpTable,
            AF_INET as u32,
            TCP_TABLE_OWNER_PID_ALL,
            MIB_TCPTABLE_OWNER_PID,
            MIB_TCPROW_OWNER_PID,
            port,
            pids,
            |row: &MIB_TCPROW_OWNER_PID| {
                u16::from_be(row.dwLocalPort as u16) == port
                    && (!listen_only || row.dwState == MIB_TCP_STATE_LISTEN as u32)
            }
        );
        scan_table!(
            GetExtendedTcpTable,
            AF_INET6 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            MIB_TCP6TABLE_OWNER_PID,
            MIB_TCP6ROW_OWNER_PID,
            port,
            pids,
            |row: &MIB_TCP6ROW_OWNER_PID| {
                u16::from_be(row.dwLocalPort as u16) == port
                    && (!listen_only || row.dwState == MIB_TCP_STATE_LISTEN as u32)
            }
        );
        if !listen_only {
            scan_table!(
                GetExtendedUdpTable,
                AF_INET as u32,
                UDP_TABLE_OWNER_PID,
                MIB_UDPTABLE_OWNER_PID,
                MIB_UDPROW_OWNER_PID,
                port,
                pids,
                |row: &MIB_UDPROW_OWNER_PID| u16::from_be(row.dwLocalPort as u16) == port
            );
            scan_table!(
                GetExtendedUdpTable,
                AF_INET6 as u32,
                UDP_TABLE_OWNER_PID,
                MIB_UDP6TABLE_OWNER_PID,
                MIB_UDP6ROW_OWNER_PID,
                port,
                pids,
                |row: &MIB_UDP6ROW_OWNER_PID| u16::from_be(row.dwLocalPort as u16) == port
            );
        }
        Ok(())
    }

    /// Snapshot of process-id → executable name (ToolHelp32).
    fn snapshot_names() -> HashMap<u32, String> {
        let mut names = HashMap::new();
        unsafe {
            let handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if handle == INVALID_HANDLE_VALUE {
                return names;
            }
            let mut entry: PROCESSENTRY32 = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
            if Process32First(handle, &mut entry) == FALSE {
                CloseHandle(handle);
                return names;
            }
            loop {
                let len = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                // windows-sys exposes the WCHAR image name as raw bytes.
                let bytes: Vec<u8> = entry.szExeFile[..len].iter().map(|&c| c as u8).collect();
                let name = String::from_utf8_lossy(&bytes).into_owned();
                names.insert(entry.th32ProcessID, name);
                if Process32Next(handle, &mut entry) == FALSE {
                    break;
                }
            }
            CloseHandle(handle);
        }
        names
    }

    pub fn find_processes(port: u16, listen_only: bool) -> Vec<PortProcess> {
        let mut pids = HashSet::new();
        if collect_pids(port, listen_only, &mut pids).is_err() {
            return Vec::new();
        }
        if pids.is_empty() {
            return Vec::new();
        }
        let names = snapshot_names();
        pids.into_iter()
            .map(|pid| {
                let name = names
                    .get(&pid)
                    .cloned()
                    .unwrap_or_else(|| "Unknown".to_string());
                // Windows provides no cheap, dependency-free full command
                // line; the executable name is the best we can offer.
                PortProcess {
                    pid,
                    cmd: name.clone(),
                    name,
                }
            })
            .collect()
    }

    fn kill_pid(pid: u32) -> Result<(), KillError> {
        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
            if handle.is_null() {
                // The process may have exited between the scan and now.
                if !snapshot_names().contains_key(&pid) {
                    return Ok(());
                }
                let err = GetLastError();
                return Err(if err == ERROR_ACCESS_DENIED {
                    KillError::Permission
                } else {
                    KillError::Other(format!("Failed to open process {pid}: {err:#x}"))
                });
            }
            let result = TerminateProcess(handle, 0);
            CloseHandle(handle);
            if result == FALSE {
                let err = GetLastError();
                return Err(if err == ERROR_ACCESS_DENIED {
                    KillError::Permission
                } else {
                    KillError::Other(format!("Failed to terminate process {pid}: {err:#x}"))
                });
            }
        }
        Ok(())
    }

    pub fn kill_on_port(port: u16, _force: bool) -> Result<bool, KillError> {
        let mut pids = HashSet::new();
        collect_pids(port, false, &mut pids).map_err(KillError::Other)?;
        if pids.is_empty() {
            return Ok(false);
        }
        let mut denied = false;
        let mut killed_any = false;
        for pid in pids {
            match kill_pid(pid) {
                Ok(()) => killed_any = true,
                Err(KillError::Permission) => denied = true,
                Err(KillError::Other(_)) => {}
            }
        }
        if denied && !killed_any {
            return Err(KillError::Permission);
        }
        Ok(wait_for_port_free(port))
    }

    #[cfg(test)]
    mod tests {
        use std::net::TcpListener;

        #[test]
        fn finds_listener_in_current_process() {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            let processes = super::find_processes(port, true);
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
            let processes = super::find_processes(port, true);
            assert!(processes.is_empty());
        }
    }
}

// =============================================================================
// UNSUPPORTED PLATFORMS
// =============================================================================

#[cfg(not(any(unix, windows)))]
mod platform {
    use super::{KillError, PortProcess};

    pub fn find_processes(_port: u16, _listen_only: bool) -> Vec<PortProcess> {
        Vec::new()
    }

    pub fn kill_on_port(_port: u16, _force: bool) -> Result<bool, KillError> {
        Err(KillError::Other("Unsupported platform".to_string()))
    }
}

// =============================================================================
// MODULE REGISTRATION
// =============================================================================

/// Python module for port management.
#[pymodule]
fn _lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(is_available, m)?)?;
    m.add_function(wrap_pyfunction!(find_free, m)?)?;
    m.add_function(wrap_pyfunction!(wait_until_free, m)?)?;
    m.add_function(wrap_pyfunction!(get_info, m)?)?;
    m.add_function(wrap_pyfunction!(kill, m)?)?;
    m.add_function(wrap_pyfunction!(scan, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
