# Definition of Done for CAPE Rust/PyO3 MVP Port

## Target completion criteria

| ID | Criterion | Evidence path | Current status |
| --- | --- | --- | --- |
| DoD-1 | CAPE extraction-core Rust project exists with PyO3 wrapping metadata. | `Cargo.toml`, `pyproject.toml`, `src/lib.rs` | Satisfied as source artifact. |
| DoD-2 | Rust source contains cell fitting, histogram, region growing, plane fitting and cylinder fitting modules. | `src/geometry.rs`, `src/linalg.rs`, `src/models.rs`, `src/histogram.rs`, `src/extractor.rs` | Satisfied as source artifact. |
| DoD-3 | Python/Jupyter demo utilities call the native PyO3 module directly. | `python/cape_demo/*`, `notebooks/cape_pyo3_demo.ipynb` | Satisfied. |
| DoD-4 | Demo can visualize the official CAPE sample frame. | `data/seq_example/*`, `artifacts/demo_preview.png`, `artifacts/cape_pyo3_demo.executed.ipynb` | Satisfied through the native Rust/PyO3 backend. |
| DoD-5 | PyO3-backed Python tests pass. | `tests/test_official_demo.py` | Satisfied: `PYTHONPATH=python uv run pytest -q` passes. |
| DoD-6 | Native PyO3 build and smoke execution complete. | `Cargo.lock`, `python/cape_demo/native_smoke.py`, release `maturin develop` output | Satisfied: `uv run maturin develop --release` and `PYTHONPATH=python uv run python -m cape_demo.native_smoke` pass. |
| DoD-7 | Current verification commands and environment are recorded. | `docs/verification.md` | Satisfied. |
| DoD-8 | Generated build/runtime directories are excluded while source and demo artifacts can be staged. | `.gitignore`, `git status --short --ignored -- .` | Satisfied. |

## Native verification status

Native Rust/PyO3 compilation completes in this checkout. The notebook imports the PyO3-backed helper directly, so execution fails fast if the extension has not been built.

## Reproduction commands

```bash
cd /home/inaho-omen/Project/cape_rs
uv sync --all-extras
uv run maturin develop --release
PYTHONPATH=python uv run pytest -q
PYTHONPATH=python uv run python -m cape_demo.native_smoke
PYTHONPATH=python uv run jupyter nbconvert --to notebook --execute notebooks/cape_pyo3_demo.ipynb --output cape_pyo3_demo.executed.ipynb --output-dir artifacts
PYTHONPATH=python uv run python -m cape_demo.native_smoke --out artifacts/demo_preview.png
```

## Scope notes

The MVP implements the extraction path. The following are future work:

1. Full sequential RANSAC parity with the original C++ `CylinderSeg.cpp`.
2. Morphological boundary refinement and plane merging parity.
3. Probabilistic cylinder fitting and visual odometry integration from the paper's later sections.
4. Numpy zero-copy input/output for the PyO3 API; the current MVP uses nested Python sequences for simpler dependency management.
