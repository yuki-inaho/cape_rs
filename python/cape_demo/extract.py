from __future__ import annotations

from typing import Any

import numpy as np

import cape_pyo3

from .official_data import OfficialCapeFrame


def official_sample_params() -> cape_pyo3.CapeParams:
    """Return parameters matching the official CAPE offline sample setup."""

    return cape_pyo3.CapeParams(max_merge_dist=50.0, extrusion_score_min=100.0)


def extract_frame(frame: OfficialCapeFrame, params: cape_pyo3.CapeParams | None = None) -> dict[str, Any]:
    """Run the native PyO3 extractor on one loaded CAPE frame."""

    result = cape_pyo3.extract_depth(
        frame.depth.tolist(),
        frame.fx,
        frame.fy,
        frame.cx,
        frame.cy,
        params or official_sample_params(),
    )
    result["labels"] = np.asarray(result["labels"], dtype=np.uint16)
    return result
