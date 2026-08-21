# Development

portly is a hybrid Python + Rust package: a thin PyO3 extension over a
pure-Rust core. See [CONTRIBUTING.md][contributing] for the full contributor
guide; the essentials are below.

## Setup

Requirements: Python 3.10+, a Rust toolchain (rustc 1.85+), and
[uv](https://docs.astral.sh/uv/) (or pip).

```bash
# Install dev dependencies.
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
mypy python/portly tests

# Stubs must match the compiled extension.
python -m mypy.stubtest portly._lib --allowlist tests/stubtest-allowlist.txt
```

## Project layout

- `src/lib.rs` — the Rust extension. Platform-specific process lookup lives in
  `mod platform` (Linux: `procfs`, macOS: `libproc`, Windows: `windows-sys`).
  There are **no subprocess calls** — do not add `lsof`, `netstat`, `wmic`, or
  friends.
- `python/portly/` — the Python package: a thin re-export shim
  (`__init__.py`), the type stubs (`_lib.pyi`), and `py.typed`.
- `tests/` — pytest suite (run against the installed module) plus the stubtest
  allowlist.
- `examples/` — the FastAPI playground demo.
- `docs/` — this documentation site (mkdocs + mkdocstrings).

## Building the docs locally

```bash
uv sync --group docs
maturin develop     # mkdocstrings imports the package, so the extension is needed
mkdocs serve        # or: mkdocs build
```

## Versioning and releases

portly follows [SemVer](https://semver.org/). The version is single-sourced in
`Cargo.toml`; `pyproject.toml` uses `dynamic = ["version"]`. Releases are
driven by [release-please](https://github.com/googleapis/release-please), which
parses conventional commits and maintains release PRs. See
[CONTRIBUTING.md][contributing] for details.

[contributing]: https://github.com/PatrykBochenek/portly/blob/main/CONTRIBUTING.md