"""portly — cross-platform Python port management made simple.

A Rust-powered library to inspect, scan and free TCP/UDP ports.

Example:
    >>> import portly as pw
    >>> pw.is_available(8000)
    True
    >>> port = pw.find_free(8000)
    >>> pw.wait_until_free(5432, timeout=30)
    True
"""

from portly._lib import (
    PortlyError,
    PortlyPortError,
    __version__,
    _register_permission_error,
    find_free,
    get_info,
    is_available,
    kill,
    scan,
    wait_until_free,
)


class PortlyPermissionError(PortlyError, PermissionError):
    """Permission denied when inspecting or killing a process.

    Multiple inheritance keeps two common error-handling patterns working:

    * ``except PortlyError`` catches every portly-specific failure.
    * ``except PermissionError`` (and therefore ``except OSError``) keeps
      existing callers working unchanged.
    """


# Register the Python-side class so the Rust extension can raise it directly.
_register_permission_error(PortlyPermissionError)

__all__ = [
    "PortlyError",
    "PortlyPermissionError",
    "PortlyPortError",
    "__version__",
    "find_free",
    "get_info",
    "is_available",
    "kill",
    "scan",
    "wait_until_free",
]
