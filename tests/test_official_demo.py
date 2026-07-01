from __future__ import annotations

from pathlib import Path

import numpy as np

from cape_demo.extract import extract_frame
from cape_demo.official_data import load_official_seq_example
from cape_demo.visualization import save_demo_figure


def test_official_seq_example_loader_reads_depth_rgb_and_intrinsics() -> None:
    frame = load_official_seq_example()
    assert frame.depth.shape == (480, 640)
    assert frame.depth.dtype == np.float64
    assert frame.rgb.shape == (480, 640, 3)
    assert frame.rgb.dtype == np.float64
    assert frame.depth.max() > 1000.0
    assert np.count_nonzero(frame.depth) > 200_000
    assert 570.0 < frame.fx < 580.0
    assert 570.0 < frame.fy < 580.0
    assert 310.0 < frame.cx < 320.0
    assert 225.0 < frame.cy < 235.0


def test_native_pyo3_extracts_official_sample_plane_and_cylinder() -> None:
    frame = load_official_seq_example()
    result = extract_frame(frame)

    labels = np.asarray(result["labels"])
    assert labels.shape == frame.depth.shape
    assert len(result["planes"]) >= 1
    assert len(result["cylinders"]) >= 1
    assert result["stats"]["planar_cells"] > 600

    cylinder = result["cylinders"][0]
    assert cylinder["cell_count"] > 400
    assert 1000.0 < cylinder["radius"] < 1800.0
    assert cylinder["extrusion_score"] > 100.0


def test_native_demo_preview_is_written(tmp_path: Path) -> None:
    frame = load_official_seq_example()
    result = extract_frame(frame)

    out = save_demo_figure(frame, result, tmp_path / "official_preview.png")
    assert out.exists()
    assert out.stat().st_size > 50_000
