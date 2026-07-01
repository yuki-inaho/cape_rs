# CAPE algorithm notes for Rust/PyO3 port

## Source inputs

- Paper: `Fast_Cylinder_and_Plane_Extraction_from_Depth_Came.pdf`.
- Existing C++ implementation inspected from the unpacked draft source: `temp/CAPE-master/CAPE-master/CAPE` in this checkout.
- Core C++ files: `CAPE.cpp`, `PlaneSeg.cpp`, `CylinderSeg.cpp`, `Histogram.cpp`, `run_cape_offline.cpp`.

## Paper-level workflow

The paper describes CAPE as a fast cylinder and plane extraction method for organized point clouds. The extraction pipeline is:

1. Planar cell fitting on a regular image grid.
2. Normal histogram construction from planar cell normals.
3. Cell-wise region growing guided by the most frequent normal-bin seed.
4. Plane and cylinder model fitting.
5. Plane merging and approximate boundary refinement.

The paper reports that the extractor operates on 640x480 depth images by using a grid of planar cells and targets consistent low latency. It also uses cylinder primitives downstream in visual odometry, but the available repository primarily implements extraction and offline visualization.

## Existing C++ implementation observations

### `PlaneSeg.cpp`

- Stores raw first and second moments: `x_acc`, `y_acc`, `z_acc`, `xx_acc`, `xy_acc`, etc.
- Classifies a cell as non-planar if fewer than half of points are valid.
- Performs a horizontal and vertical cross-scan through the cell center to reject depth discontinuities.
- Fits a plane by 3x3 covariance eigen decomposition.
- Sets plane normal orientation so `d > 0` for the plane equation `n · p + d = 0`.
- Uses `MSE = smallest_eigenvalue / nr_pts` and `score = second_eigenvalue / smallest_eigenvalue`.

### `Histogram.cpp`

- Quantizes polar and azimuth angles of cell normals.
- Retrieves the most frequent bin as seed candidates.
- Removes assigned cells from the histogram during region growing.

### `CAPE.cpp`

- Creates a regular grid of `cell_width x cell_height` cells.
- Fits planar cells first.
- Builds a normal histogram.
- Repeatedly selects seed cells from the most frequent bin and grows via 4-neighbour traversal.
- Neighbour acceptance conditions are:
  - cell is still unassigned;
  - normal dot product is above `min_cos_angle_4_merge`;
  - centroid-to-current-plane squared distance is below an adaptive threshold.
- If the grown segment is sufficiently planar, stores it as a plane.
- Otherwise, delegates to `CylinderSeg` when cylinder detection is enabled.
- Performs a cell-grid boundary refinement in the original implementation. The MVP Rust port initially exposes cell-level labels plus per-pixel rectangular assignment; full morphological refinement is listed as an extension point.

### `CylinderSeg.cpp`

- Performs PCA on stacked normals `[N, -N]` to detect extrusion and obtain cylinder axis.
- Projects centroids and normals onto the plane perpendicular to the axis.
- Uses a direct radius/center solution based on cell normals.
- Embeds the direct solution in sequential RANSAC in the original C++ implementation.
- Stores axis, center, radius and segment inliers.

## MVP scope for this delivery

Implemented in the Rust project:

- Back-projection from depth image to 3D points.
- Cell-wise raw-moment accumulation and plane fitting.
- 3x3 symmetric Jacobi eigen decomposition implemented without Eigen.
- Normal histogram seed selection.
- 4-neighbour cell region growing.
- Segment-level plane fitting.
- Segment-level cylinder fitting based on PCA of stacked normals and direct projected normal equations.
- PyO3 API exposing `CapeParams` and `extract_depth`.
- Jupyter-oriented Python helper utilities that call the PyO3 module directly.

Out of MVP scope, documented for future work:

- Full sequential RANSAC parity with all original C++ heuristics.
- Plane merging after initial classification.
- Morphological boundary refinement equivalent to the original C++ code.
- Probabilistic cylinder fitting and visual odometry pose optimization from the latter part of the paper.

## Traceability matrix

| Paper / C++ concept | Rust module target | Python/Jupyter exposure |
| --- | --- | --- |
| Planar cell fitting | `src/models.rs`, `src/extractor.rs` | returned plane list and label map |
| Normal histogram | `src/histogram.rs` | stats only |
| Region growing | `src/extractor.rs` | label map |
| Plane fitting | `src/models.rs` | `planes` list |
| Cylinder direct fitting | `src/models.rs` | `cylinders` list |
| PyO3 wrapper | `src/lib.rs` | `cape_pyo3.extract_depth(...)` |
| Demo visualization | `python/cape_demo/*`, `notebooks/*.ipynb` | notebook cells |
