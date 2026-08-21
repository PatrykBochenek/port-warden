# portly

**Cross-platform Python port management made simple.**

Check whether a port is free, find a free port, wait for one to become free,
inspect what is listening on it, scan a list of ports, or kill whatever is
holding one — fast, and on Linux, macOS, and Windows.

Everything is implemented natively (no `lsof`, `netstat`, `wmic`, or other
subprocesses), so it works in minimal containers and is safe from shell
escaping issues.

## Installation

```bash
pip install portly
```

Requires Python 3.10+. Pre-built wheels are available for Linux (manylinux &
musllinux), macOS (Intel & Apple Silicon), and Windows (x64 & arm64).

## Quick Start

```python
import portly

# Check if a port is available
if portly.is_available(8000):
    print("Port 8000 is free!")

# Find a free port (preferring 8000 if it happens to be free)
port = portly.find_free(8000)

# Wait for a port to become free (up to 30s)
portly.wait_until_free(5432, timeout=30)

# Kill the process using a port
portly.kill(8000)

# Get process info
info = portly.get_info(8000)
# {'pid': 1234, 'name': 'python', 'cmd': 'python app.py'}  (or None)

# Scan multiple ports at once
results = portly.scan([8000, 8001, 5432])
```

## Why portly?

- **Cross-platform** — Linux, macOS, and Windows, with per-arch wheels.
- **Fast** — Rust core; no Python socket gymnastics, no subprocess spawning.
- **Correct** — native APIs (`/proc`, `libproc`, `GetExtendedTcpTable`), with
  graceful handling of zombies, permissions, and races.
- **Typed** — first-party type stubs checked against the compiled module.
- **Small** — a handful of functions, one dependency-free story.

See the [API Reference](api.md), [Platform support](platforms.md),
[Cookbook](cookbook.md), and [Development](development.md) pages for details.