# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-08-16

### Added

- Initial release of **portly**: a cross-platform, Rust-powered library
  for checking, finding, scanning, waiting on, and freeing TCP/UDP ports.
- API: `is_available`, `find_free`, `wait_until_free`, `get_info`, `kill`,
  `scan`, `__version__`.
- Native (subprocess-free) process lookup on all platforms:
  - Linux: `/proc/net/{tcp,tcp6,udp,udp6}` + `/proc/<pid>/fd` via `procfs`.
  - macOS: `libproc` (`proc_pidinfo`).
  - Windows: `GetExtended{Tcp,Udp}Table` + ToolHelp32 via `windows-sys`
    (no dependency on the deprecated `wmic`).
- Type stubs (`_lib.pyi`) validated by `mypy.stubtest`; `py.typed` marker.
- CI: lint/typecheck gates, cross-platform test matrix (incl. free-threaded
  3.14t), wheel + sdist install tests, and a release workflow using PyPI
  Trusted Publishing with PEP 740 attestations.

[Unreleased]: https://github.com/PatrykBochenek/portly/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/PatrykBochenek/portly/releases/tag/v0.1.0