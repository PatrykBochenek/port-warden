"""Examples: portly playground — a small FastAPI app to try out the library.

Run from the repo root with:

    .venv/bin/uvicorn examples.main:app --reload --port 8712

Then open http://127.0.0.1:8712 (or use the auto docs at /docs).
"""

from __future__ import annotations

import subprocess
import sys
import time
from contextlib import asynccontextmanager
from pathlib import Path

from fastapi import FastAPI, HTTPException, Query
from fastapi.responses import FileResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel, Field

import portly

STATIC_DIR = Path(__file__).resolve().parent / "static"

# Demo servers started from the UI: port -> Popen
_demo_servers: dict[int, subprocess.Popen] = {}

DEMO_SCRIPT = """
import socket, time
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", {port}))
s.listen(1)
print("ready", flush=True)
while True:
    time.sleep(3600)
"""


@asynccontextmanager
async def lifespan(_: FastAPI):
    yield
    # Clean up any demo servers still running when the app shuts down.
    for proc in _demo_servers.values():
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=3)
            except subprocess.TimeoutExpired:
                proc.kill()
    _demo_servers.clear()


app = FastAPI(title="portly playground", lifespan=lifespan)
app.mount("/static", StaticFiles(directory=STATIC_DIR), name="static")


class PortBody(BaseModel):
    port: int = Field(ge=1, le=65535)


class KillBody(PortBody):
    force: bool = False


class WaitBody(PortBody):
    timeout: int = Field(default=30, ge=1, le=300)


class ScanBody(BaseModel):
    ports: list[int] = Field(max_length=100)


@app.get("/", include_in_schema=False)
def index() -> FileResponse:
    return FileResponse(STATIC_DIR / "index.html")


@app.get("/api/version")
def version() -> dict:
    return {"version": portly.__version__}


@app.get("/api/available")
def available(port: int = Query(ge=1, le=65535)) -> dict:
    return {"port": port, "available": portly.is_available(port)}


@app.get("/api/find-free")
def find_free(preferred: int | None = Query(default=None, ge=1, le=65535)) -> dict:
    try:
        port = portly.find_free(preferred)
    except OSError as e:
        raise HTTPException(status_code=503, detail=str(e)) from e
    return {"port": port}


@app.get("/api/info")
def info(port: int = Query(ge=1, le=65535)) -> dict:
    return {"port": port, "info": portly.get_info(port)}


@app.post("/api/kill")
def kill(body: KillBody) -> dict:
    try:
        killed = portly.kill(body.port, force=body.force)
    except PermissionError as e:
        raise HTTPException(status_code=403, detail=str(e)) from e
    except OSError as e:
        raise HTTPException(status_code=500, detail=str(e)) from e
    return {"port": body.port, "killed": killed, "force": body.force}


@app.post("/api/wait")
def wait_free(body: WaitBody) -> dict:
    start = time.monotonic()
    became_free = portly.wait_until_free(body.port, timeout=body.timeout)
    waited_ms = int((time.monotonic() - start) * 1000)
    return {"port": body.port, "became_free": became_free, "waited_ms": waited_ms}


@app.post("/api/scan")
def scan(body: ScanBody) -> dict:
    for p in body.ports:
        if not 1 <= p <= 65535:
            raise HTTPException(status_code=422, detail=f"Port {p} is out of range")
    return {"results": portly.scan(body.ports)}


@app.post("/api/demo/start")
def demo_start(body: PortBody) -> dict:
    """Spawn a real Python subprocess that listens on the given port."""
    if not portly.is_available(body.port):
        raise HTTPException(status_code=409, detail=f"Port {body.port} is already in use")
    proc = subprocess.Popen(
        [sys.executable, "-c", DEMO_SCRIPT.format(port=body.port)],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    try:
        line = proc.stdout.readline() or ""
    except ValueError:
        line = ""
    if "ready" not in line:
        proc.kill()
        proc.wait()
        raise HTTPException(status_code=500, detail=f"Demo server failed to start: {line!r}")
    _demo_servers[body.port] = proc
    return {"port": body.port, "pid": proc.pid}


@app.post("/api/demo/stop")
def demo_stop(body: PortBody) -> dict:
    proc = _demo_servers.pop(body.port, None)
    if proc is None:
        raise HTTPException(status_code=404, detail=f"No demo server on port {body.port}")
    proc.terminate()
    try:
        proc.wait(timeout=3)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait(timeout=3)
    return {"port": body.port, "stopped": True}
