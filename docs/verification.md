# Verification

Last local verification: `2026-07-01 14:26:20 JST+0900`

## Environment

- `uv 0.9.7`
- `rustc 1.92.0-nightly`
- `cargo 1.92.0-nightly`
- `just 1.40.0`
- Python is managed by `uv` in `.venv`.

## Commands

```bash
uv sync --all-extras
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
uv run maturin develop --release
PYTHONPATH=python uv run pytest -q
PYTHONPATH=python uv run python -m cape_demo.native_smoke
PYTHONPATH=python uv run jupyter nbconvert --to notebook --execute notebooks/cape_pyo3_demo.ipynb --output cape_pyo3_demo.executed.ipynb --output-dir artifacts
PYTHONPATH=python uv run python -m cape_demo.native_smoke --out artifacts/demo_preview.png
```

## Native Smoke Result

The native smoke command imports `cape_pyo3`, loads the official CAPE sample from `data/seq_example/`, runs the Rust/PyO3 extractor, and asserts that at least one plane and one cylinder are detected.

## Official Data Source

The tracked files under `data/seq_example/` are copied from the official CAPE repository directory:

`https://github.com/pedropro/CAPE/tree/master/Data/seq_example`
