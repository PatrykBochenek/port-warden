from typing import TypedDict

class PortInfo(TypedDict):
    """Information about the process using a port.

    Note:
        `cmd` is the full command line on Linux, the executable path on
        macOS, and the executable name on Windows.
    """

    pid: int
    name: str
    cmd: str

class PortlyError(OSError):
    """Base class for all portly errors.

    Subclasses `OSError` so existing ``except OSError`` blocks continue to
    catch portly failures.
    """

class PortlyPortError(PortlyError):
    """Port-related failure (no free port found, port could not be freed, ...)."""

def _register_permission_error(exc: type[BaseException]) -> None:
    """Register the Python-side permission exception class with Rust."""
    ...

def is_available(port: int) -> bool:
    """Return True if *port* is free on localhost, False otherwise."""
    ...

def find_free(preferred: int | None = None) -> int:
    """Return a free port, preferring *preferred* when it is free.

    Raises:
        PortlyPortError: If no free port could be found.
    """
    ...

def wait_until_free(port: int, timeout: int = 30) -> bool:
    """Wait up to *timeout* seconds for *port* to become free.

    Returns True if the port became free, False on timeout.
    """
    ...

def get_info(port: int) -> PortInfo | None:
    """Return information about the process listening on *port*.

    Returns None if the port is free (or its owner is not visible to us).
    """
    ...

def kill(port: int, force: bool = False) -> bool:
    """Kill the process(es) using *port*.

    Sends SIGTERM unless *force* is True (SIGKILL on Unix; TerminateProcess
    on Windows regardless of *force*). Returns True if the port became free.

    Raises:
        PortlyPermissionError: If we lack permission to kill the process.
        PortlyPortError: If the process(es) could not be killed for another reason.
    """
    ...

def scan(ports: list[int]) -> dict[int, PortInfo | None]:
    """Return process info for each port in *ports* (None when free)."""
    ...

__version__: str

__all__ = [
    "PortlyError",
    "PortlyPortError",
    "__version__",
    "find_free",
    "get_info",
    "is_available",
    "kill",
    "scan",
    "wait_until_free",
]
