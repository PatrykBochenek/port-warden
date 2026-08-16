"""Tests for the portly library."""

from __future__ import annotations

import os
import socket
import subprocess
import threading
import time

import portly as pw


class TestIsAvailable:
    """Tests for is_available."""

    def test_free_port_returns_true(self, free_port: int) -> None:
        assert pw.is_available(free_port) is True

    def test_busy_port_returns_false(self, busy_port: int) -> None:
        assert pw.is_available(busy_port) is False

    def test_port_zero_is_not_available(self) -> None:
        assert pw.is_available(0) is False

    def test_upper_bound(self) -> None:
        assert isinstance(pw.is_available(65535), bool)


class TestFindFree:
    """Tests for find_free."""

    def test_find_any_free_port(self) -> None:
        port = pw.find_free()
        assert isinstance(port, int)
        assert 1 <= port <= 65535
        assert pw.is_available(port) is True

    def test_preferred_port_when_free(self, free_port: int) -> None:
        assert pw.find_free(free_port) == free_port

    def test_different_port_when_preferred_busy(self, busy_port: int) -> None:
        result = pw.find_free(busy_port)
        assert result != busy_port
        assert pw.is_available(result) is True

    def test_preferred_zero_returns_any_free_port(self) -> None:
        result = pw.find_free(0)
        assert 1 <= result <= 65535


class TestWaitUntilFree:
    """Tests for wait_until_free."""

    def test_port_already_free(self, free_port: int) -> None:
        assert pw.wait_until_free(free_port, timeout=1) is True

    def test_wait_for_port_to_become_free(self) -> None:
        server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        server.bind(("127.0.0.1", 0))
        server.listen(1)
        port = int(server.getsockname()[1])

        def close_server() -> None:
            time.sleep(1)
            server.close()

        thread = threading.Thread(target=close_server)
        thread.start()
        try:
            assert pw.wait_until_free(port, timeout=5) is True
        finally:
            thread.join()

    def test_timeout_returns_false(self) -> None:
        server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        server.bind(("127.0.0.1", 0))
        server.listen(1)
        port = int(server.getsockname()[1])
        try:
            assert pw.wait_until_free(port, timeout=1) is False
        finally:
            server.close()


class TestGetInfo:
    """Tests for get_info."""

    def test_free_port_returns_none(self, free_port: int) -> None:
        assert pw.get_info(free_port) is None

    def test_busy_port_returns_info(self, temp_server: tuple[socket.socket, int]) -> None:
        _server, port = temp_server
        info = pw.get_info(port)

        assert info is not None
        assert "pid" in info
        assert "name" in info
        assert "cmd" in info
        assert isinstance(info["pid"], int)
        assert info["pid"] > 0
        # The socket lives in this very process.
        assert info["pid"] == os.getpid()
        assert isinstance(info["name"], str) and info["name"]
        assert isinstance(info["cmd"], str)


class TestKill:
    """Tests for kill."""

    def test_kill_on_free_port(self, free_port: int) -> None:
        assert pw.kill(free_port) is True

    def test_kill_on_busy_port(
        self, subprocess_server: tuple[subprocess.Popen[bytes], int]
    ) -> None:
        proc, port = subprocess_server
        assert pw.kill(port) is True
        assert pw.is_available(port) is True
        proc.wait(timeout=5)
        assert proc.poll() is not None

    def test_kill_force_on_busy_port(
        self, subprocess_server: tuple[subprocess.Popen[bytes], int]
    ) -> None:
        proc, port = subprocess_server
        assert pw.kill(port, force=True) is True
        assert pw.is_available(port) is True
        proc.wait(timeout=5)
        assert proc.poll() is not None


class TestScan:
    """Tests for scan."""

    def test_scan_empty_list(self) -> None:
        assert pw.scan([]) == {}

    def test_scan_mix_of_ports(self) -> None:
        server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        server.bind(("127.0.0.1", 0))
        server.listen(1)
        busy = int(server.getsockname()[1])
        free = pw.find_free()
        try:
            result = pw.scan([busy, free])
            assert busy in result
            assert free in result
            assert result[busy] is not None  # Busy port has info
            assert result[free] is None  # Free port has no info
        finally:
            server.close()


class TestIntegration:
    """Integration tests for complete workflows."""

    def test_full_workflow(self, subprocess_server: tuple[subprocess.Popen[bytes], int]) -> None:
        """Verify, inspect, kill, verify."""
        proc, port = subprocess_server

        assert not pw.is_available(port)
        info = pw.get_info(port)
        assert info is not None
        assert "pid" in info
        assert info["pid"] == proc.pid

        pw.kill(port)

        assert pw.is_available(port)
        proc.wait(timeout=5)

    def test_find_and_wait_workflow(self) -> None:
        server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        server.bind(("127.0.0.1", 0))
        server.listen(1)
        port = int(server.getsockname()[1])

        def close() -> None:
            time.sleep(0.5)
            server.close()

        thread = threading.Thread(target=close)
        thread.start()
        try:
            assert pw.wait_until_free(port, timeout=5) is True
        finally:
            thread.join()


class TestVersion:
    """Test version is available."""

    def test_version_exists(self) -> None:
        assert isinstance(pw.__version__, str)
