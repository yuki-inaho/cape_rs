use crate::geometry::Vec3;

#[derive(Debug, Clone)]
pub struct NormalHistogram {
    bins_per_coord: usize,
    assignments: Vec<Option<usize>>,
}

impl NormalHistogram {
    pub fn new(normals: &[Option<Vec3>], bins_per_coord: usize) -> Self {
        let bins_per_coord = bins_per_coord.max(1);
        let assignments = normals
            .iter()
            .map(|normal| normal.and_then(|n| Self::bin_for_normal(n, bins_per_coord)))
            .collect();
        Self {
            bins_per_coord,
            assignments,
        }
    }

    pub fn most_frequent_cells(&self, remaining: &[bool]) -> Vec<usize> {
        let bin_count = self.bins_per_coord * self.bins_per_coord;
        let mut counts = vec![0_usize; bin_count];
        for (idx, assignment) in self.assignments.iter().enumerate() {
            if remaining.get(idx).copied().unwrap_or(false) {
                if let Some(bin) = assignment {
                    counts[*bin] += 1;
                }
            }
        }
        let Some((best_bin, best_count)) = counts
            .iter()
            .enumerate()
            .max_by_key(|(_, count)| **count)
            .map(|(idx, count)| (idx, *count))
        else {
            return Vec::new();
        };
        if best_count == 0 {
            return Vec::new();
        }
        self.assignments
            .iter()
            .enumerate()
            .filter_map(|(idx, assignment)| {
                if remaining.get(idx).copied().unwrap_or(false) && *assignment == Some(best_bin) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }

    fn bin_for_normal(normal: Vec3, bins_per_coord: usize) -> Option<usize> {
        let n = normal.normalized()?;
        let polar = (-n.z).clamp(-1.0, 1.0).acos();
        let polar_bin = quantize(polar, 0.0, std::f64::consts::PI, bins_per_coord);
        let azimuth_bin = if polar_bin == 0 {
            0
        } else {
            let horizontal_norm = (n.x * n.x + n.y * n.y).sqrt();
            if horizontal_norm <= 1.0e-12 {
                0
            } else {
                let azimuth = (n.x / horizontal_norm).atan2(n.y / horizontal_norm);
                quantize(
                    azimuth,
                    -std::f64::consts::PI,
                    std::f64::consts::PI,
                    bins_per_coord,
                )
            }
        };
        Some(azimuth_bin * bins_per_coord + polar_bin)
    }
}

fn quantize(value: f64, min_value: f64, max_value: f64, bins: usize) -> usize {
    if bins <= 1 {
        return 0;
    }
    let normalized = ((value - min_value) / (max_value - min_value)).clamp(0.0, 1.0);
    (normalized * (bins - 1) as f64).floor() as usize
}
