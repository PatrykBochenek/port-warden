//! portly — cross-platform Python port management made simple.
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

mod bindings;
mod kill;
mod model;
mod platform;
mod ports;
mod probe;

use pyo3::prelude::*;

/// Python module for port management.
#[pymodule]
fn _lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    bindings::register(m)
}
