use crate::geometry::{add_outer, scale_matrix, Vec3};
use crate::linalg::symmetric_eigen_3x3;

const EPS: f64 = 1.0e-12;

#[derive(Debug, Clone, Default)]
pub struct RawMoments {
    pub count: usize,
    pub sum: Vec3,
    pub xx: f64,
    pub yy: f64,
    pub zz: f64,
    pub xy: f64,
    pub xz: f64,
    pub yz: f64,
}

impl RawMoments {
    pub fn add_point(&mut self, point: Vec3) {
        self.count += 1;
        self.sum += point;
        self.xx += point.x * point.x;
        self.yy += point.y * point.y;
        self.zz += point.z * point.z;
        self.xy += point.x * point.y;
        self.xz += point.x * point.z;
        self.yz += point.y * point.z;
    }

    pub fn merge(&mut self, rhs: &Self) {
        self.count += rhs.count;
        self.sum += rhs.sum;
        self.xx += rhs.xx;
        self.yy += rhs.yy;
        self.zz += rhs.zz;
        self.xy += rhs.xy;
        self.xz += rhs.xz;
        self.yz += rhs.yz;
    }

    pub fn mean(&self) -> Option<Vec3> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }

    pub fn covariance_sum(&self) -> Option<[[f64; 3]; 3]> {
        let n = self.count as f64;
        if self.count < 3 {
            return None;
        }
        Some([
            [
                self.xx - self.sum.x * self.sum.x / n,
                self.xy - self.sum.x * self.sum.y / n,
                self.xz - self.sum.x * self.sum.z / n,
            ],
            [
                self.xy - self.sum.x * self.sum.y / n,
                self.yy - self.sum.y * self.sum.y / n,
                self.yz - self.sum.y * self.sum.z / n,
            ],
            [
                self.xz - self.sum.x * self.sum.z / n,
                self.yz - self.sum.y * self.sum.z / n,
                self.zz - self.sum.z * self.sum.z / n,
            ],
        ])
    }
}

#[derive(Debug, Clone)]
pub struct PlaneModel {
    pub normal: Vec3,
    pub d: f64,
    pub mean: Vec3,
    pub mse: f64,
    pub score: f64,
    pub point_count: usize,
}

impl PlaneModel {
    pub fn fit(moments: &RawMoments) -> Option<Self> {
        let covariance_sum = moments.covariance_sum()?;
        let eigen = symmetric_eigen_3x3(covariance_sum);
        let mean = moments.mean()?;
        let mut normal = eigen.vectors[0].normalized()?;
        let mut d = -normal.dot(mean);
        if d <= 0.0 {
            normal = normal * -1.0;
            d = -d;
        }
        let smallest = eigen.values[0].max(EPS);
        let second = eigen.values[1].max(EPS);
        Some(Self {
            normal,
            d,
            mean,
            mse: eigen.values[0].max(0.0) / moments.count as f64,
            score: second / smallest,
            point_count: moments.count,
        })
    }

    pub fn signed_distance(&self, point: Vec3) -> f64 {
        self.normal.dot(point) + self.d
    }
}

#[derive(Debug, Clone)]
pub struct PlaneCell {
    pub moments: RawMoments,
    pub model: PlaneModel,
}

#[derive(Debug, Clone)]
pub struct CylinderModel {
    pub axis: Vec3,
    pub center: Vec3,
    pub radius: f64,
    pub mse: f64,
    pub extrusion_score: f64,
}

impl CylinderModel {
    /// Fits one cylinder to a grown cell segment using the direct projected-normal
    /// solution from CAPE. This is the deterministic MVP path; full sequential
    /// RANSAC parity is documented as follow-up work.
    pub fn fit_direct(cells: &[&PlaneCell], min_extrusion_score: f64) -> Option<Self> {
        if cells.len() < 3 {
            return None;
        }

        let mut normal_cov = [[0.0_f64; 3]; 3];
        for cell in cells {
            add_outer(&mut normal_cov, cell.model.normal, cell.model.normal);
            add_outer(
                &mut normal_cov,
                cell.model.normal * -1.0,
                cell.model.normal * -1.0,
            );
        }
        let denom = (2 * cells.len()).saturating_sub(1).max(1) as f64;
        normal_cov = scale_matrix(normal_cov, 1.0 / denom);
        let eigen = symmetric_eigen_3x3(normal_cov);
        let lambda_min = eigen.values[0].abs().max(EPS);
        let lambda_max = eigen.values[2].abs().max(EPS);
        let extrusion_score = lambda_max / lambda_min;
        if extrusion_score < min_extrusion_score {
            return None;
        }
        let axis = eigen.vectors[0].normalized()?;

        let mut projected_points = Vec::with_capacity(cells.len());
        let mut projected_normals = Vec::with_capacity(cells.len());
        for cell in cells {
            let p = cell.model.mean;
            let n = cell.model.normal;
            let p_projected = p - axis * axis.dot(p);
            let n_projected = (n - axis * axis.dot(n)).normalized()?;
            projected_points.push(p_projected);
            projected_normals.push(n_projected);
        }

        let m = projected_points.len() as f64;
        let p_mean = projected_points
            .iter()
            .copied()
            .fold(Vec3::ZERO, |acc, p| acc + p)
            / m;
        let n_mean = projected_normals
            .iter()
            .copied()
            .fold(Vec3::ZERO, |acc, n| acc + n)
            / m;

        let denom = 1.0 - projected_normals.iter().map(|n| n.dot(n_mean)).sum::<f64>() / m;
        if denom.abs() < EPS {
            return None;
        }
        let numer = projected_normals
            .iter()
            .zip(projected_points.iter())
            .map(|(n, p)| n.dot(*p - p_mean))
            .sum::<f64>()
            / m;
        let signed_radius = numer / denom;
        if !signed_radius.is_finite() || signed_radius.abs() < EPS {
            return None;
        }
        let center = projected_points
            .iter()
            .zip(projected_normals.iter())
            .map(|(p, n)| *p - *n * signed_radius)
            .fold(Vec3::ZERO, |acc, value| acc + value)
            / m;
        let mse = projected_points
            .iter()
            .zip(projected_normals.iter())
            .map(|(p, n)| (*p - *n * signed_radius - center).norm_squared())
            .sum::<f64>()
            / m;

        Some(Self {
            axis,
            center,
            radius: signed_radius.abs(),
            mse,
            extrusion_score,
        })
    }
}
