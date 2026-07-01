set shell := ["bash", "-cu"]

setup:
    uv sync --all-extras

develop:
    uv run maturin develop --release

native-smoke:
    PYTHONPATH=python uv run python -m cape_demo.native_smoke

test:
    PYTHONPATH=python uv run pytest -q

notebook:
    PYTHONPATH=python uv run jupyter nbconvert --to notebook --execute notebooks/cape_pyo3_demo.ipynb --output cape_pyo3_demo.executed.ipynb --output-dir artifacts

preview:
    PYTHONPATH=python uv run python -m cape_demo.native_smoke --out artifacts/demo_preview.png

rust-check:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test

verify: rust-check setup develop test native-smoke notebook preview

package:
    mkdir -p artifacts
    zip -r artifacts/cape_rust_pyo3_submission.zip Cargo.toml Cargo.lock pyproject.toml uv.lock justfile README.md src python notebooks tests docs artifacts/demo_preview.png artifacts/cape_pyo3_demo.executed.ipynb
