use crate::geometry::Vec3;
use crate::histogram::NormalHistogram;
use crate::models::{CylinderModel, PlaneCell, PlaneModel, RawMoments};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct CapeParamsCore {
    pub cell_width: usize,
    pub cell_height: usize,
    pub enable_cylinders: bool,
    pub min_valid_fraction: f64,
    pub max_depth_jump: f64,
    pub min_cos_angle: f64,
    pub max_merge_dist: f64,
    pub min_merge_dist: f64,
    pub depth_sigma_coeff: f64,
    pub depth_sigma_margin: f64,
    pub plane_score_min: f64,
    pub extrusion_score_min: f64,
    pub min_seed_bin_count: usize,
    pub min_segment_cells: usize,
    pub min_cylinder_cells: usize,
    pub histogram_bins: usize,
}

impl Default for CapeParamsCore {
    fn default() -> Self {
        Self {
            cell_width: 20,
            cell_height: 20,
            enable_cylinders: true,
            min_valid_fraction: 0.50,
            max_depth_jump: 100.0,
            min_cos_angle: 0.965_925_826_289_068_3,
            max_merge_dist: 900.0,
            min_merge_dist: 20.0,
            depth_sigma_coeff: 0.000_001_425,
            depth_sigma_margin: 10.0,
            plane_score_min: 100.0,
            extrusion_score_min: 50.0,
            min_seed_bin_count: 3,
            min_segment_cells: 3,
            min_cylinder_cells: 4,
            histogram_bins: 20,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlaneDetection {
    pub id: u16,
    pub model: PlaneModel,
    pub cell_ids: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct CylinderDetection {
    pub id: u16,
    pub model: CylinderModel,
    pub cell_ids: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct ExtractionStats {
    pub height: usize,
    pub width: usize,
    pub grid_rows: usize,
    pub grid_cols: usize,
    pub planar_cells: usize,
    pub grown_segments: usize,
    pub skipped_small_segments: usize,
}

#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub labels: Vec<Vec<u16>>,
    pub planes: Vec<PlaneDetection>,
    pub cylinders: Vec<CylinderDetection>,
    pub stats: ExtractionStats,
}

pub fn extract_depth_core(
    depth: &[Vec<f64>],
    fx: f64,
    fy: f64,
    cx: f64,
    cy: f64,
    params: &CapeParamsCore,
) -> Result<ExtractionResult, String> {
    validate_inputs(depth, fx, fy, params)?;
    let height = depth.len();
    let width = depth[0].len();
    let grid_rows = height / params.cell_height;
    let grid_cols = width / params.cell_width;
    if grid_rows == 0 || grid_cols == 0 {
        return Err("depth image is smaller than the configured CAPE cell size".to_string());
    }

    let points = back_project(depth, fx, fy, cx, cy);
    let (cells, distance_tolerances) = fit_cells(depth, &points, grid_rows, grid_cols, params);
    let normals: Vec<Option<Vec3>> = cells
        .iter()
        .map(|cell| cell.as_ref().map(|value| value.model.normal))
        .collect();
    let histogram = NormalHistogram::new(&normals, params.histogram_bins);
    let mut remaining: Vec<bool> = cells.iter().map(Option::is_some).collect();
    let planar_cells = remaining.iter().filter(|flag| **flag).count();
    let mut labels = vec![vec![0_u16; width]; height];
    let mut planes = Vec::new();
    let mut cylinders = Vec::new();
    let mut grown_segments = 0_usize;
    let mut skipped_small_segments = 0_usize;

    loop {
        let seed_candidates = histogram.most_frequent_cells(&remaining);
        if seed_candidates.len() < params.min_seed_bin_count {
            break;
        }
        let Some(seed_id) = seed_candidates.iter().copied().min_by(|left, right| {
            let left_mse = cells[*left]
                .as_ref()
                .map(|cell| cell.model.mse)
                .unwrap_or(f64::INFINITY);
            let right_mse = cells[*right]
                .as_ref()
                .map(|cell| cell.model.mse)
                .unwrap_or(f64::INFINITY);
            left_mse.total_cmp(&right_mse)
        }) else {
            break;
        };

        let active = grow_segment(
            seed_id,
            grid_rows,
            grid_cols,
            &cells,
            &remaining,
            &distance_tolerances,
            params.min_cos_angle,
        );
        if active.is_empty() {
            remaining[seed_id] = false;
            continue;
        }
        for cell_id in &active {
            remaining[*cell_id] = false;
        }
        grown_segments += 1;

        if active.len() < params.min_segment_cells {
            skipped_small_segments += 1;
            continue;
        }

        let segment_cells: Vec<&PlaneCell> =
            active.iter().filter_map(|id| cells[*id].as_ref()).collect();
        let aggregate = aggregate_cell_moments(&segment_cells);
        let Some(plane_model) = PlaneModel::fit(&aggregate) else {
            continue;
        };

        if plane_model.score >= params.plane_score_min {
            let id = checked_label_id(planes.len() + 1)?;
            mark_cells(&mut labels, grid_cols, params, &active, id);
            planes.push(PlaneDetection {
                id,
                model: plane_model,
                cell_ids: active,
            });
            continue;
        }

        if params.enable_cylinders && segment_cells.len() >= params.min_cylinder_cells {
            if let Some(cylinder_model) =
                CylinderModel::fit_direct(&segment_cells, params.extrusion_score_min)
            {
                let id = checked_label_id(1000 + cylinders.len())?;
                mark_cells(&mut labels, grid_cols, params, &active, id);
                cylinders.push(CylinderDetection {
                    id,
                    model: cylinder_model,
                    cell_ids: active,
                });
            }
        }
    }

    Ok(ExtractionResult {
        labels,
        planes,
        cylinders,
        stats: ExtractionStats {
            height,
            width,
            grid_rows,
            grid_cols,
            planar_cells,
            grown_segments,
            skipped_small_segments,
        },
    })
}

fn validate_inputs(
    depth: &[Vec<f64>],
    fx: f64,
    fy: f64,
    params: &CapeParamsCore,
) -> Result<(), String> {
    if depth.is_empty() {
        return Err("depth image must contain at least one row".to_string());
    }
    let width = depth[0].len();
    if width == 0 {
        return Err("depth image must contain at least one column".to_string());
    }
    for (idx, row) in depth.iter().enumerate() {
        if row.len() != width {
            return Err(format!("depth image is ragged at row {idx}"));
        }
    }
    if !fx.is_finite() || !fy.is_finite() || fx <= 0.0 || fy <= 0.0 {
        return Err("fx and fy must be finite positive values".to_string());
    }
    if params.cell_width == 0 || params.cell_height == 0 {
        return Err("cell_width and cell_height must be positive".to_string());
    }
    if !(0.0..=1.0).contains(&params.min_valid_fraction) {
        return Err("min_valid_fraction must be within [0, 1]".to_string());
    }
    Ok(())
}

fn back_project(depth: &[Vec<f64>], fx: f64, fy: f64, cx: f64, cy: f64) -> Vec<Option<Vec3>> {
    let height = depth.len();
    let width = depth[0].len();
    let mut points = Vec::with_capacity(height * width);
    for (row_idx, row) in depth.iter().enumerate() {
        for (col_idx, z) in row.iter().enumerate() {
            if z.is_finite() && *z > 0.0 {
                let x = (col_idx as f64 - cx) / fx * *z;
                let y = (row_idx as f64 - cy) / fy * *z;
                points.push(Some(Vec3::new(x, y, *z)));
            } else {
                points.push(None);
            }
        }
    }
    points
}

fn fit_cells(
    depth: &[Vec<f64>],
    points: &[Option<Vec3>],
    grid_rows: usize,
    grid_cols: usize,
    params: &CapeParamsCore,
) -> (Vec<Option<PlaneCell>>, Vec<f64>) {
    let image_width = depth[0].len();
    let total_cells = grid_rows * grid_cols;
    let mut cells = vec![None; total_cells];
    let mut tolerances = vec![0.0_f64; total_cells];
    let min_valid_points = ((params.cell_width * params.cell_height) as f64
        * params.min_valid_fraction)
        .ceil() as usize;
    let sin_angle = (1.0 - params.min_cos_angle * params.min_cos_angle)
        .max(0.0)
        .sqrt();

    for cell_row in 0..grid_rows {
        for cell_col in 0..grid_cols {
            let cell_id = cell_row * grid_cols + cell_col;
            if has_depth_discontinuity(depth, cell_row, cell_col, params) {
                continue;
            }
            let mut moments = RawMoments::default();
            let mut min_point = Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
            let mut max_point = Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
            for local_row in 0..params.cell_height {
                let row = cell_row * params.cell_height + local_row;
                for local_col in 0..params.cell_width {
                    let col = cell_col * params.cell_width + local_col;
                    let point_idx = row * image_width + col;
                    if let Some(point) = points[point_idx] {
                        moments.add_point(point);
                        min_point.x = min_point.x.min(point.x);
                        min_point.y = min_point.y.min(point.y);
                        min_point.z = min_point.z.min(point.z);
                        max_point.x = max_point.x.max(point.x);
                        max_point.y = max_point.y.max(point.y);
                        max_point.z = max_point.z.max(point.z);
                    }
                }
            }
            if moments.count < min_valid_points {
                continue;
            }
            let Some(model) = PlaneModel::fit(&moments) else {
                continue;
            };
            let depth_sigma =
                params.depth_sigma_coeff * model.mean.z * model.mean.z + params.depth_sigma_margin;
            if model.mse > depth_sigma * depth_sigma {
                continue;
            }
            let cell_diameter = if min_point.x.is_finite() && max_point.x.is_finite() {
                min_point.distance_squared(max_point).sqrt()
            } else {
                params.min_merge_dist
            };
            let distance_tol = (cell_diameter * sin_angle)
                .max(params.min_merge_dist)
                .min(params.max_merge_dist);
            tolerances[cell_id] = distance_tol * distance_tol;
            cells[cell_id] = Some(PlaneCell { moments, model });
        }
    }
    (cells, tolerances)
}

fn has_depth_discontinuity(
    depth: &[Vec<f64>],
    cell_row: usize,
    cell_col: usize,
    params: &CapeParamsCore,
) -> bool {
    if params.max_depth_jump <= 0.0 {
        return false;
    }
    let row_offset = cell_row * params.cell_height;
    let col_offset = cell_col * params.cell_width;
    let mid_row = row_offset + params.cell_height / 2;
    let mid_col = col_offset + params.cell_width / 2;

    let mut horizontal_jumps = 0_usize;
    let mut last_valid: Option<f64> = None;
    for &z in depth[mid_row]
        .iter()
        .skip(col_offset)
        .take(params.cell_width)
    {
        if z.is_finite() && z > 0.0 {
            if let Some(last) = last_valid {
                if (z - last).abs() > params.max_depth_jump {
                    horizontal_jumps += 1;
                }
            }
            last_valid = Some(z);
        }
    }
    if horizontal_jumps > 1 {
        return true;
    }

    let mut vertical_jumps = 0_usize;
    last_valid = None;
    for row in depth.iter().skip(row_offset).take(params.cell_height) {
        let z = row[mid_col];
        if z.is_finite() && z > 0.0 {
            if let Some(last) = last_valid {
                if (z - last).abs() > params.max_depth_jump {
                    vertical_jumps += 1;
                }
            }
            last_valid = Some(z);
        }
    }
    vertical_jumps > 1
}

fn grow_segment(
    seed_id: usize,
    grid_rows: usize,
    grid_cols: usize,
    cells: &[Option<PlaneCell>],
    remaining: &[bool],
    distance_tolerances: &[f64],
    min_cos_angle: f64,
) -> Vec<usize> {
    if !remaining.get(seed_id).copied().unwrap_or(false) || cells[seed_id].is_none() {
        return Vec::new();
    }
    let mut active = vec![false; cells.len()];
    let mut queue = VecDeque::new();
    active[seed_id] = true;
    queue.push_back(seed_id);

    while let Some(current_id) = queue.pop_front() {
        for neighbor_id in neighbors_4(current_id, grid_rows, grid_cols) {
            if active[neighbor_id] || !remaining[neighbor_id] {
                continue;
            }
            let Some(current_cell) = cells[current_id].as_ref() else {
                continue;
            };
            let Some(neighbor_cell) = cells[neighbor_id].as_ref() else {
                continue;
            };
            let normal_dot = current_cell.model.normal.dot(neighbor_cell.model.normal);
            if normal_dot < min_cos_angle {
                continue;
            }
            let distance = current_cell.model.signed_distance(neighbor_cell.model.mean);
            if distance * distance > distance_tolerances[neighbor_id] {
                continue;
            }
            active[neighbor_id] = true;
            queue.push_back(neighbor_id);
        }
    }

    active
        .iter()
        .enumerate()
        .filter_map(|(idx, is_active)| if *is_active { Some(idx) } else { None })
        .collect()
}

fn neighbors_4(cell_id: usize, grid_rows: usize, grid_cols: usize) -> Vec<usize> {
    let row = cell_id / grid_cols;
    let col = cell_id % grid_cols;
    let mut out = Vec::with_capacity(4);
    if col > 0 {
        out.push(cell_id - 1);
    }
    if col + 1 < grid_cols {
        out.push(cell_id + 1);
    }
    if row > 0 {
        out.push(cell_id - grid_cols);
    }
    if row + 1 < grid_rows {
        out.push(cell_id + grid_cols);
    }
    out
}

fn aggregate_cell_moments(cells: &[&PlaneCell]) -> RawMoments {
    let mut aggregate = RawMoments::default();
    for cell in cells {
        aggregate.merge(&cell.moments);
    }
    aggregate
}

fn checked_label_id(value: usize) -> Result<u16, String> {
    u16::try_from(value).map_err(|_| "too many detected segments for u16 label map".to_string())
}

fn mark_cells(
    labels: &mut [Vec<u16>],
    grid_cols: usize,
    params: &CapeParamsCore,
    cell_ids: &[usize],
    label: u16,
) {
    let height = labels.len();
    let width = labels[0].len();
    for cell_id in cell_ids {
        let cell_row = cell_id / grid_cols;
        let cell_col = cell_id % grid_cols;
        let row_start = cell_row * params.cell_height;
        let col_start = cell_col * params.cell_width;
        let row_end = (row_start + params.cell_height).min(height);
        let col_end = (col_start + params.cell_width).min(width);
        for row in labels.iter_mut().take(row_end).skip(row_start) {
            for value in row.iter_mut().take(col_end).skip(col_start) {
                *value = label;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_depth_core, CapeParamsCore};

    #[test]
    fn rejects_empty_depth() {
        let err =
            extract_depth_core(&[], 1.0, 1.0, 0.0, 0.0, &CapeParamsCore::default()).unwrap_err();
        assert!(err.contains("at least one row"));
    }
}
