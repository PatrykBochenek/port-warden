#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
mod unsupported;
#[cfg(windows)]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::{find_processes, kill_pid};
#[cfg(target_os = "macos")]
pub use macos::{find_processes, kill_pid};
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
pub use unsupported::{find_processes, kill_on_port, kill_pid};
#[cfg(windows)]
pub use windows::{find_processes, kill_pid};
