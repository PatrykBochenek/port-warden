# Contributing to port-warden

Thanks for your interest in contributing! port-warden is a hybrid Python +
Rust package: a thin PyO3 extension over a pure-Rust core. Contributions of
all kinds are welcome — bug reports, docs, tests, and code.

## Development setup

Requirements: Python 3.10+, a Rust toolchain (rustc 1.85+), and
[uv](https://docs.astral.sh/uv/) (or pip).

```bash
# Clone the repo, then install dev dependencies.
uv sync --group dev

# Build and install the extension into the current environment.
maturin develop

# Run the tests.
pytest tests/
```

When you change Rust code you need to re-run `maturin develop` for the
Python tests to pick it up.

## Quality gates

All of the following must pass before merging. The CI runs the same checks.

```bash
# Rust
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test

# Python
ruff check python/ tests/ examples/
ruff format --check python/ tests/ examples/
mypy python/port_warden tests

# Stubs must match the compiled extension.
maturin develop
python -m mypy.stubtest port_warden._lib --allowlist tests/stubtest-allowlist.txt
```

There is also a pre-commit config; install it with
`pre-commit install` if you use pre-commit.

## Project layout

- `src/lib.rs` — the Rust extension. Platform-specific process lookup lives
  in `mod platform` (Linux: `procfs`, macOS: `libproc`, Windows:
  `windows-sys`). There are **no subprocess calls** — do not add `lsof`,
  `netstat`, `wmic`, or friends.
- `python/port_warden/` — the Python package: a thin re-export shim
  (`__init__.py`), the type stubs (`_lib.pyi`), and `py.typed`.
- `tests/` — pytest suite (run against the installed module) plus the
  stubtest allowlist.
- `examples/` — the FastAPI playground demo.

## Versioning and releases

port-warden follows [SemVer](https://semver.org/). The version is
single-sourced in `Cargo.toml`; `pyproject.toml` uses `dynamic = ["version"]`.

Releases are cut from the `main` branch:

1. Update `CHANGELOG.md` (keep a changelog format), commit, and push.
2. Create an annotated tag matching the `Cargo.toml` version and push it:

   ```bash
   git tag -a v0.1.1 -m "port-warden v0.1.1"
   git push origin v0.1.1
   ```

3. The `release.yml` workflow builds wheels for all platforms, publishes to
   PyPI via Trusted Publishing, and creates a GitHub Release with release
   notes.

## Questions?

Open a discussion in the GitHub repo, or ask in your PR/issue.