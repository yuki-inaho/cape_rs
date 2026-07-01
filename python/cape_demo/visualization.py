from __future__ import annotations

from pathlib import Path
from typing import Any

import matplotlib.pyplot as plt
import numpy as np

def labels_to_rgb(labels: np.ndarray) -> np.ndarray:
    """Map integer labels to a deterministic RGB image."""

    labels = np.asarray(labels)
    out = np.zeros(labels.shape + (3,), dtype=np.float64)
    unique = [int(label) for label in np.unique(labels) if int(label) != 0]
    for label in unique:
        rng = np.random.default_rng(label)
        out[labels == label] = rng.uniform(0.25, 1.0, size=3)
    return out


def overlay_labels(rgb: np.ndarray, labels: np.ndarray, alpha: float = 0.45) -> np.ndarray:
    base = np.asarray(rgb, dtype=np.float64)
    if base.max() > 1.0:
        base = base / 255.0
    label_rgb = labels_to_rgb(labels)
    mask = np.asarray(labels) > 0
    out = base.copy()
    out[mask] = (1.0 - alpha) * base[mask] + alpha * label_rgb[mask]
    return np.clip(out, 0.0, 1.0)


def save_demo_figure(scene: Any, result: dict[str, Any], path: str | Path) -> Path:
    """Save a compact visual audit figure for the CAPE demo."""

    path = Path(path)
    labels = np.asarray(result["labels"])
    overlay = overlay_labels(scene.rgb, labels)

    fig = plt.figure(figsize=(12, 7))
    ax1 = fig.add_subplot(2, 2, 1)
    depth_plot = ax1.imshow(scene.depth, cmap="viridis")
    ax1.set_title("Depth (mm)")
    ax1.axis("off")
    fig.colorbar(depth_plot, ax=ax1, fraction=0.046, pad=0.04)

    ax2 = fig.add_subplot(2, 2, 2)
    ax2.imshow(labels, cmap="tab20")
    ax2.set_title("Extracted labels")
    ax2.axis("off")

    ax3 = fig.add_subplot(2, 2, 3)
    ax3.imshow(overlay)
    ax3.set_title("Overlay")
    ax3.axis("off")

    ax4 = fig.add_subplot(2, 2, 4)
    ax4.axis("off")
    lines = [
        "CAPE Rust/PyO3 demo result",
        f"planes: {len(result['planes'])}",
        f"cylinders: {len(result['cylinders'])}",
        f"planar cells: {result['stats']['planar_cells']}",
        f"grown segments: {result['stats']['grown_segments']}",
    ]
    source_dir = getattr(scene, "source_dir", None)
    if source_dir is not None:
        lines.insert(1, f"source: {Path(source_dir).name}")
    for plane in result["planes"][:3]:
        normal = np.array(plane["normal"])
        lines.append(f"plane {plane['id']}: n={np.round(normal, 3).tolist()}, cells={plane['cell_count']}")
    for cyl in result["cylinders"][:3]:
        axis = np.array(cyl["axis"])
        lines.append(f"cylinder {cyl['id']}: r={cyl['radius']:.1f}, axis={np.round(axis, 3).tolist()}")
    ax4.text(0.0, 1.0, "\n".join(lines), va="top", family="monospace")

    fig.tight_layout()
    path.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(path, dpi=160)
    plt.close(fig)
    return path
