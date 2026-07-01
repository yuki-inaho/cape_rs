mod extractor;
mod geometry;
mod histogram;
mod linalg;
mod models;

use extractor::{extract_depth_core, CapeParamsCore};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PySequence};

#[pyclass(name = "CapeParams")]
#[derive(Debug, Clone)]
pub struct PyCapeParams {
    #[pyo3(get, set)]
    pub cell_width: usize,
    #[pyo3(get, set)]
    pub cell_height: usize,
    #[pyo3(get, set)]
    pub enable_cylinders: bool,
    #[pyo3(get, set)]
    pub min_valid_fraction: f64,
    #[pyo3(get, set)]
    pub max_depth_jump: f64,
    #[pyo3(get, set)]
    pub min_cos_angle: f64,
    #[pyo3(get, set)]
    pub max_merge_dist: f64,
    #[pyo3(get, set)]
    pub min_merge_dist: f64,
    #[pyo3(get, set)]
    pub depth_sigma_coeff: f64,
    #[pyo3(get, set)]
    pub depth_sigma_margin: f64,
    #[pyo3(get, set)]
    pub plane_score_min: f64,
    #[pyo3(get, set)]
    pub extrusion_score_min: f64,
    #[pyo3(get, set)]
    pub min_seed_bin_count: usize,
    #[pyo3(get, set)]
    pub min_segment_cells: usize,
    #[pyo3(get, set)]
    pub min_cylinder_cells: usize,
    #[pyo3(get, set)]
    pub histogram_bins: usize,
}

impl Default for PyCapeParams {
    fn default() -> Self {
        CapeParamsCore::default().into()
    }
}

impl From<CapeParamsCore> for PyCapeParams {
    fn from(value: CapeParamsCore) -> Self {
        Self {
            cell_width: value.cell_width,
            cell_height: value.cell_height,
            enable_cylinders: value.enable_cylinders,
            min_valid_fraction: value.min_valid_fraction,
            max_depth_jump: value.max_depth_jump,
            min_cos_angle: value.min_cos_angle,
            max_merge_dist: value.max_merge_dist,
            min_merge_dist: value.min_merge_dist,
            depth_sigma_coeff: value.depth_sigma_coeff,
            depth_sigma_margin: value.depth_sigma_margin,
            plane_score_min: value.plane_score_min,
            extrusion_score_min: value.extrusion_score_min,
            min_seed_bin_count: value.min_seed_bin_count,
            min_segment_cells: value.min_segment_cells,
            min_cylinder_cells: value.min_cylinder_cells,
            histogram_bins: value.histogram_bins,
        }
    }
}

impl From<&PyCapeParams> for CapeParamsCore {
    fn from(value: &PyCapeParams) -> Self {
        Self {
            cell_width: value.cell_width,
            cell_height: value.cell_height,
            enable_cylinders: value.enable_cylinders,
            min_valid_fraction: value.min_valid_fraction,
            max_depth_jump: value.max_depth_jump,
            min_cos_angle: value.min_cos_angle,
            max_merge_dist: value.max_merge_dist,
            min_merge_dist: value.min_merge_dist,
            depth_sigma_coeff: value.depth_sigma_coeff,
            depth_sigma_margin: value.depth_sigma_margin,
            plane_score_min: value.plane_score_min,
            extrusion_score_min: value.extrusion_score_min,
            min_seed_bin_count: value.min_seed_bin_count,
            min_segment_cells: value.min_segment_cells,
            min_cylinder_cells: value.min_cylinder_cells,
            histogram_bins: value.histogram_bins,
        }
    }
}

#[pymethods]
impl PyCapeParams {
    #[new]
    #[pyo3(signature = (
        cell_width = 20,
        cell_height = 20,
        enable_cylinders = true,
        min_valid_fraction = 0.50,
        max_depth_jump = 100.0,
        min_cos_angle = 0.965_925_826_289_068_3,
        max_merge_dist = 900.0,
        min_merge_dist = 20.0,
        depth_sigma_coeff = 0.000_001_425,
        depth_sigma_margin = 10.0,
        plane_score_min = 100.0,
        extrusion_score_min = 50.0,
        min_seed_bin_count = 3,
        min_segment_cells = 3,
        min_cylinder_cells = 4,
        histogram_bins = 20
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        cell_width: usize,
        cell_height: usize,
        enable_cylinders: bool,
        min_valid_fraction: f64,
        max_depth_jump: f64,
        min_cos_angle: f64,
        max_merge_dist: f64,
        min_merge_dist: f64,
        depth_sigma_coeff: f64,
        depth_sigma_margin: f64,
        plane_score_min: f64,
        extrusion_score_min: f64,
        min_seed_bin_count: usize,
        min_segment_cells: usize,
        min_cylinder_cells: usize,
        histogram_bins: usize,
    ) -> Self {
        Self {
            cell_width,
            cell_height,
            enable_cylinders,
            min_valid_fraction,
            max_depth_jump,
            min_cos_angle,
            max_merge_dist,
            min_merge_dist,
            depth_sigma_coeff,
            depth_sigma_margin,
            plane_score_min,
            extrusion_score_min,
            min_seed_bin_count,
            min_segment_cells,
            min_cylinder_cells,
            histogram_bins,
        }
    }
}

#[pyfunction]
#[pyo3(signature = (depth, fx, fy, cx, cy, params = None))]
fn extract_depth<'py>(
    py: Python<'py>,
    depth: &Bound<'py, PyAny>,
    fx: f64,
    fy: f64,
    cx: f64,
    cy: f64,
    params: Option<PyRef<'py, PyCapeParams>>,
) -> PyResult<Bound<'py, PyDict>> {
    let depth_rows = depth_to_vec(depth)?;
    let params_core = params
        .as_deref()
        .map(CapeParamsCore::from)
        .unwrap_or_default();
    let result = extract_depth_core(&depth_rows, fx, fy, cx, cy, &params_core)
        .map_err(PyValueError::new_err)?;

    let out = PyDict::new(py);
    out.set_item("labels", result.labels)?;

    let planes = PyList::empty(py);
    for plane in result.planes {
        let item = PyDict::new(py);
        item.set_item("id", plane.id)?;
        item.set_item("normal", plane.model.normal.as_array())?;
        item.set_item("d", plane.model.d)?;
        item.set_item("mean", plane.model.mean.as_array())?;
        item.set_item("mse", plane.model.mse)?;
        item.set_item("score", plane.model.score)?;
        item.set_item("cell_count", plane.cell_ids.len())?;
        item.set_item("point_count", plane.model.point_count)?;
        planes.append(item)?;
    }
    out.set_item("planes", planes)?;

    let cylinders = PyList::empty(py);
    for cylinder in result.cylinders {
        let item = PyDict::new(py);
        item.set_item("id", cylinder.id)?;
        item.set_item("axis", cylinder.model.axis.as_array())?;
        item.set_item("center", cylinder.model.center.as_array())?;
        item.set_item("radius", cylinder.model.radius)?;
        item.set_item("mse", cylinder.model.mse)?;
        item.set_item("extrusion_score", cylinder.model.extrusion_score)?;
        item.set_item("cell_count", cylinder.cell_ids.len())?;
        cylinders.append(item)?;
    }
    out.set_item("cylinders", cylinders)?;

    let stats = PyDict::new(py);
    stats.set_item("height", result.stats.height)?;
    stats.set_item("width", result.stats.width)?;
    stats.set_item("grid_rows", result.stats.grid_rows)?;
    stats.set_item("grid_cols", result.stats.grid_cols)?;
    stats.set_item("planar_cells", result.stats.planar_cells)?;
    stats.set_item("grown_segments", result.stats.grown_segments)?;
    stats.set_item(
        "skipped_small_segments",
        result.stats.skipped_small_segments,
    )?;
    out.set_item("stats", stats)?;
    Ok(out)
}

fn depth_to_vec(depth: &Bound<'_, PyAny>) -> PyResult<Vec<Vec<f64>>> {
    let rows = depth.downcast::<PySequence>()?;
    let row_count = rows.len()?;
    let mut out = Vec::with_capacity(row_count);
    let mut expected_width = None;
    for row_idx in 0..row_count {
        let row_obj = rows.get_item(row_idx)?;
        let row = row_obj.downcast::<PySequence>()?;
        let width = row.len()?;
        if let Some(expected) = expected_width {
            if expected != width {
                return Err(PyValueError::new_err(format!(
                    "depth image is ragged at row {row_idx}: expected {expected}, got {width}"
                )));
            }
        } else {
            expected_width = Some(width);
        }
        let mut values = Vec::with_capacity(width);
        for col_idx in 0..width {
            let value = row.get_item(col_idx)?.extract::<f64>().map_err(|err| {
                PyValueError::new_err(format!(
                    "depth[{row_idx}][{col_idx}] is not a float-compatible value: {err}"
                ))
            })?;
            values.push(value);
        }
        out.push(values);
    }
    Ok(out)
}

#[pymodule]
fn cape_pyo3(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyCapeParams>()?;
    m.add_function(wrap_pyfunction!(extract_depth, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
