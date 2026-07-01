# Unofficial Rust/PyO3 Implementation for CAPE

This repository is a Rust/PyO3 reimplementation of CAPE, the cell-based plane and cylinder extraction method described in *Fast Cylinder and Plane Extraction from Depth Cameras for Visual Odometry*.

Official Repository: [github.com/pedropro/CAPE](https://github.com/pedropro/CAPE)

## Contents

- `src/`: Rust implementation and PyO3 module.
- `python/cape_demo/`: small Python utilities for loading the official sample, running the PyO3 module, and visualizing results.
- `data/seq_example/`: official CAPE sample frame copied from `pedropro/CAPE`.
- `notebooks/cape_pyo3_demo.ipynb`: official CAPE sample depth/RGB demo.
- `tests/`: PyO3-backed tests for the official sample demo.
- `docs/`: onboarding notes for future LLM agents.
- `justfile`: repeatable commands.

## Build and development

Preferred workflow:

```bash
uv sync --all-extras
uv run maturin develop --release
PYTHONPATH=python uv run pytest -q
PYTHONPATH=python uv run python -m cape_demo.native_smoke
PYTHONPATH=python uv run jupyter nbconvert --to notebook --execute notebooks/cape_pyo3_demo.ipynb --output cape_pyo3_demo.executed.ipynb --output-dir artifacts
```

If `just` is installed, the equivalent commands are:

```bash
just setup
just develop
just test
just native-smoke
just notebook
```

## Native API

After `maturin develop`, use:

```python
import cape_pyo3

params = cape_pyo3.CapeParams(cell_width=20, cell_height=20)
result = cape_pyo3.extract_depth(depth.tolist(), fx, fy, cx, cy, params)
labels = result["labels"]
planes = result["planes"]
cylinders = result["cylinders"]
```

Build the extension before using `cape_pyo3`; importing it fails if the native module has not been installed.

## Demo

The notebook defaults to the native Rust/PyO3 backend and reads the official CAPE `Data/seq_example` files in `data/seq_example/`. To run it:

1. Install Rust and Cargo.
2. Run `uv run maturin develop --release`.
3. Open `notebooks/cape_pyo3_demo.ipynb`.
4. Execute the notebook, or run `just notebook`.

## Current execution environment note

This checkout was verified with `uv`, Python 3.13, Rust/Cargo, `maturin`, `pytest`, and `nbconvert`. The release PyO3 extension builds through `uv run maturin develop --release`, the native smoke check detects plane/cylinder segments in the official CAPE sample frame, and the notebook executes through the PyO3 path.
