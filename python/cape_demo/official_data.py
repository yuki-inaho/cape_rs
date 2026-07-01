from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import xml.etree.ElementTree as ET

import numpy as np
from PIL import Image


DEFAULT_SEQ_EXAMPLE_DIR = Path(__file__).resolve().parents[2] / "data" / "seq_example"


@dataclass(frozen=True)
class OfficialCapeFrame:
    """One frame from the official CAPE `Data/seq_example` directory."""

    depth: np.ndarray
    rgb: np.ndarray
    fx: float
    fy: float
    cx: float
    cy: float
    source_dir: Path
    depth_path: Path
    rgb_path: Path
    calib_path: Path


def load_official_seq_example(seq_dir: str | Path = DEFAULT_SEQ_EXAMPLE_DIR) -> OfficialCapeFrame:
    """Load the official CAPE sample depth/RGB frame and IR intrinsics.

    The original CAPE offline sample reads `depth_0.png` with OpenCV
    `IMREAD_ANYDEPTH` and uses `IR_intrinsic_params` for depth back-projection.
    This loader mirrors those inputs for the Rust/PyO3 notebook demo.
    """

    seq_dir = Path(seq_dir)
    depth_path = seq_dir / "depth_0.png"
    rgb_path = seq_dir / "rgb_0.png"
    calib_path = seq_dir / "calib_params.xml"
    for path in (depth_path, rgb_path, calib_path):
        if not path.exists():
            raise FileNotFoundError(f"official CAPE sample file is missing: {path}")

    depth = np.asarray(Image.open(depth_path), dtype=np.float64)
    rgb = np.asarray(Image.open(rgb_path).convert("RGB"), dtype=np.float64) / 255.0
    ir_intrinsics = _load_opencv_matrix(calib_path, "IR_intrinsic_params")
    return OfficialCapeFrame(
        depth=depth,
        rgb=rgb,
        fx=float(ir_intrinsics[0, 0]),
        fy=float(ir_intrinsics[1, 1]),
        cx=float(ir_intrinsics[0, 2]),
        cy=float(ir_intrinsics[1, 2]),
        source_dir=seq_dir,
        depth_path=depth_path,
        rgb_path=rgb_path,
        calib_path=calib_path,
    )


def _load_opencv_matrix(path: Path, name: str) -> np.ndarray:
    root = ET.parse(path).getroot()
    node = root.find(name)
    if node is None:
        raise ValueError(f"{name!r} not found in {path}")

    rows_node = node.find("rows")
    cols_node = node.find("cols")
    data_node = node.find("data")
    if rows_node is None or cols_node is None or data_node is None or data_node.text is None:
        raise ValueError(f"{name!r} in {path} is not a complete OpenCV matrix")

    rows = int(rows_node.text)
    cols = int(cols_node.text)
    values = np.fromstring(data_node.text, sep=" ", dtype=np.float64)
    if values.size != rows * cols:
        raise ValueError(f"{name!r} in {path} has {values.size} values, expected {rows * cols}")
    return values.reshape((rows, cols))
