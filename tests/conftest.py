"""Shared pytest fixtures for portly tests."""

from __future__ import annotations

import socket
import subprocess
import sys
from collections.abc import Iterator

import pytest

import portly

SERVER_SCRIPT = """
import socket
import time

s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", {port}))
s.listen(1)
print("ready", flush=True)
time.sleep(120)
"""


@pytest.fixture
def free_port() -> int:
    """Get a port that is currently free."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


@pytest.fixture
def busy_port() -> Iterator[int]:
    """Get a port that is in use (yields, then closes)."""
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind(("127.0.0.1", 0))
    server.listen(1)
    yield int(server.getsockname()[1])
    server.close()


@pytest.fixture
def temp_server() -> Iterator[tuple[socket.socket, int]]:
    """Start a temporary server; yields (socket, port)."""
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind(("127.0.0.1", 0))
    server.listen(1)
    yield server, int(server.getsockname()[1])
    server.close()


@pytest.fixture
def subprocess_server() -> Iterator[tuple[subprocess.Popen[bytes], int]]:
    """Spawn a subprocess holding a port; yields (proc, port).

    The port is held by a SEPARATE process on purpose — the kill()
    tests must never kill the pytest process itself.
    """
    port = portly.find_free()
    proc = subprocess.Popen(
        [sys.executable, "-c", SERVER_SCRIPT.format(port=port)],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    # Wait until the child reports it is listening.
    assert proc.stdout is not None
    line = proc.stdout.readline()
    assert line.strip() == b"ready", f"server subprocess failed: {line!r}"
    # Sanity: the port must actually be busy now.
    assert not portly.is_available(port)
    yield proc, port
    if proc.poll() is None:
        proc.kill()
        proc.wait(timeout=5)
