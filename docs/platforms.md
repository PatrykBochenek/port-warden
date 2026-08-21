# Platform support

portly ships pre-built wheels for Linux, macOS, and Windows (multiple
architectures per OS). The process lookups are implemented natively per
platform — **no subprocesses** (`lsof`, `netstat`, `tasklist`, `wmic`, …) are
spawned.

| Platform | Socket lookup | Process mapping | Notes |
| --- | --- | --- | --- |
| Linux | `/proc/net/{tcp,tcp6,udp,udp6}` | `/proc/<pid>/fd` scan via `procfs` | Relies on `procfs`; works in minimal containers |
| macOS | `libproc` (`proc_pidinfo`) | `libproc` | |
| Windows | `GetExtended{Tcp,Udp}Table` | ToolHelp32 snapshots via `windows-sys` | |

## Permission caveats

- **Visibility** — process info and kills only cover processes visible to your
  user. Other users' listeners may report as "free"/unreachable without
  elevated privileges (same limitation as `lsof`).
- **`kill`** — sends `SIGTERM` unless `force=True` (`SIGKILL`). On Windows the
  process is always terminated forcefully. Killing another user's process
  requires elevated privileges and raises `PermissionError` (or
  `PortlyPermissionError`) otherwise.

## Behavior notes

- **`get_info`** — `cmd` is the full command line on Linux, the executable
  path on macOS, and the executable name on Windows.
- **`find_free`** — the returned port can be taken between the check and your
  `bind()` (TOCTOU). If you need a race-free reservation, bind to port `0` and
  let the OS choose.

!!! info "FreeBSD"
    FreeBSD support is tracked in the [roadmap issue][roadmap]; today the
    package requires one of the three platforms above.

[roadmap]: https://github.com/PatrykBochenek/portly/issues/20