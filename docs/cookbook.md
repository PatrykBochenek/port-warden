# Cookbook

Practical recipes built on the portly API.

## Pick a free port for a dev server

Use `find_free` to grab a free port, then hand it to your server. Prefer a
specific port and fall back to a free one when it is busy:

```python
import portly

port = portly.find_free(8000)
print(f"Starting server on {port}")
```

To constrain the search to a known range (and reserve several ports at once):

```python
import portly

port = portly.find_free_in_range(8000, 8100)          # a single free port
ports = portly.find_free_in_range(8000, 8100, count=3)  # three distinct ports
```

## Wait for a service to come up (`wait-for-it`)

Block until a server is *actually listening* on a port — the classic deploy /
compose pattern:

```python
import portly

if portly.wait_for_server(8000, timeout=30):
    print("Server is ready!")
else:
    print("Server did not come up in time")
    raise SystemExit(1)
```

## Wait for a port to free up

The inverse: block until a process releases a port.

```python
import portly

if portly.wait_until_free(5432, timeout=30):
    print("Port 5432 is free")
```

## Inspect and kill what is holding a port

```python
import portly

info = portly.get_info(8000)
if info is not None:
    print(f"pid={info['pid']} name={info['name']} cmd={info['cmd']}")

# Kill it (SIGTERM); use force=True for SIGKILL.
portly.kill(8000)
```

## Handle errors with the typed hierarchy

```python
import portly
from portly import PortlyError, PortlyPortError

try:
    portly.find_free_in_range(1, 1)
except PortlyPortError as exc:
    # Most specific type; also caught by `except OSError`.
    print(f"No free port: {exc}")
except PortlyError as exc:
    # Any portly failure.
    print(f"portly error: {exc}")
```

!!! tip "TOCTOU"
    A port returned by `find_free`/`find_free_in_range` can be taken between
    the check and your `bind()`. For a race-free reservation, bind to port `0`
    and let the OS choose.