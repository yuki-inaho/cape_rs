# CAPE Rust/PyO3 port design

## 1. Purpose

This project ports the extraction core of CAPE, a cell-based plane and cylinder extractor for organized depth images, into a Rust library exposed to Python through PyO3. The immediate user-facing objective is a Jupyter Notebook demo that runs the native PyO3 module on the official CAPE `Data/seq_example` frame.

## 2. Non-goals

The following are intentionally excluded from the MVP implementation and are not claimed as complete:

- End-to-end visual odometry integration.
- Probabilistic RGB-D odometry, depth fusion, feature matching and pose optimization.
- Exact one-to-one behavioural parity with every C++ heuristic.
- Environments without `rustc`/`cargo`.

## 3. Architecture

```text
cape_rust_pyo3/
├── Cargo.toml                  # Rust crate and PyO3 extension metadata
├── pyproject.toml              # uv/maturin Python project metadata
├── src/
│   ├── lib.rs                  # PyO3 public module: CapeParams, extract_depth
│   ├── extractor.rs            # CAPE extraction orchestration
│   ├── geometry.rs             # Vec3 and primitive math
│   ├── histogram.rs            # Quantized normal histogram
│   ├── linalg.rs               # Symmetric 3x3 Jacobi eigen solver
│   └── models.rs               # PlaneCell, PlaneModel, CylinderModel fitting
├── python/cape_demo/
│   ├── official_data.py        # Official CAPE sample loader
│   ├── extract.py              # Thin PyO3 demo helper
│   ├── visualization.py        # Matplotlib visualizations
│   └── native_smoke.py         # CLI smoke check for the official sample
├── notebooks/
│   └── cape_pyo3_demo.ipynb    # Jupyter demo
├── tests/
│   └── test_official_demo.py   # PyO3-backed official sample tests
└── docs/
    ├── CAPE_algorithm_notes.md
    ├── CAPE_rust_port_design.md
    └── DoD.md
```

## 4. Data contract

### Input

`extract_depth(depth, fx, fy, cx, cy, params=None)` accepts:

- `depth`: nested Python sequence of finite positive depth values. Unit is user-defined but must be consistent with camera intrinsics and thresholds. The demo uses millimetres.
- `fx`, `fy`, `cx`, `cy`: pinhole intrinsics.
- `params`: `CapeParams`, optional. The official sample demo uses `max_merge_dist=50.0` and `extrusion_score_min=100.0` to match the original offline sample setup.

### Output

The native API returns a Python dictionary with:

```python
{
    "labels": [[int, ...], ...],
    "planes": [
        {"id": int, "normal": [float, float, float], "d": float, "mse": float,
         "score": float, "cell_count": int, "point_count": int},
    ],
    "cylinders": [
        {"id": int, "axis": [float, float, float], "center": [float, float, float],
         "radius": float, "mse": float, "cell_count": int},
    ],
    "stats": {"height": int, "width": int, "planar_cells": int, "segments": int}
}
```

Labels use `0` as background. Plane labels start from `1`; cylinder labels start from `1000`. This avoids collision in visualization.

## 5. Rust module responsibilities

### `geometry.rs`

- Provides a compact `Vec3` type.
- Implements dot, norm, normalization and arithmetic.

### `linalg.rs`

- Provides deterministic symmetric 3x3 Jacobi eigen decomposition.
- Sorts eigenpairs ascending by eigenvalue.
- Replaces the C++ Eigen dependency for the MVP.

### `models.rs`

- `PlaneCell` stores raw moments and fitted plane data for a grid cell.
- `PlaneModel` fits aggregate cell moments.
- `CylinderModel` fits an extruded cylinder using stacked-normal PCA and direct projected normal equations.

### `histogram.rs`

- Quantizes normal spherical coordinates.
- Tracks per-cell bin assignment.
- Returns all currently unassigned cells in the most frequent bin.

### `extractor.rs`

- Validates input depth shape and intrinsics.
- Back-projects depth to 3D points.
- Fits cells, builds histogram, performs region growing, classifies plane/cylinder segments.
- Produces per-pixel labels by assigning whole cells to detected models.

### `lib.rs`

- Exposes `CapeParams` and `extract_depth` with PyO3.
- Performs explicit input conversion from Python nested sequences.
- Exposes only the native Rust implementation.

## 6. Error handling policy

- Empty depth input returns a `ValueError`.
- Ragged depth rows return a `ValueError`.
- Non-positive or non-finite intrinsics return a `ValueError`.
- Non-finite or non-positive depth values are treated as missing points, not as an exception.
- Degenerate models are skipped and recorded only through stats.

## 7. Testing strategy

- Python tests import the built `cape_pyo3` module and run it on the official sample frame.
- Native Rust checks run in an environment with `rustc` and `cargo`:
  - `cargo test`
  - `cargo fmt --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `uv run maturin develop --release`
- `PYTHONPATH=python uv run python -m cape_demo.native_smoke` imports the built PyO3 extension and verifies the official CAPE sample frame.
- Notebook tests run the native PyO3 path after `maturin develop`.

## 8. Build strategy

Preferred setup:

```bash
uv sync --all-extras
uv run maturin develop --release
PYTHONPATH=python uv run pytest -q
PYTHONPATH=python uv run python -m cape_demo.native_smoke
PYTHONPATH=python uv run jupyter nbconvert --to notebook --execute notebooks/cape_pyo3_demo.ipynb --output cape_pyo3_demo.executed.ipynb --output-dir artifacts
```

The submitted project contains a `justfile` with equivalent recipes. If `just` is unavailable, commands can be copied directly from the recipe file.

## 9. Acceptance notes

This checkout has been validated with the native PyO3 backend. `uv run maturin develop --release` builds and installs `cape_pyo3`, `python -m cape_demo.native_smoke` detects plane/cylinder segments in the official sample frame, and the notebook executes through the same PyO3 path.
