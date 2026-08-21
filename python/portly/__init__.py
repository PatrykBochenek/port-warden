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
    __version__,
    find_free,
    find_free_in_range,
    get_info,
    is_available,
    kill,
    scan,
    wait_for_server,
    wait_until_free,
)

__all__ = [
    "__version__",
    "find_free",
    "find_free_in_range",
    "get_info",
    "is_available",
    "kill",
    "scan",
    "wait_for_server",
    "wait_until_free",
]
