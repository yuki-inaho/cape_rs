use crate::geometry::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Eigen3 {
    /// Eigenvalues sorted in ascending order.
    pub values: [f64; 3],
    /// Eigenvectors corresponding to `values`, sorted in the same order.
    pub vectors: [Vec3; 3],
}

/// Deterministic Jacobi eigen decomposition for real symmetric 3x3 matrices.
///
/// The implementation is intentionally small and dependency-free. It is adequate
/// for the covariance matrices used by this CAPE MVP. For production numeric
/// workloads, replacing this module with nalgebra or faer would be reasonable.
#[allow(clippy::needless_range_loop)]
pub fn symmetric_eigen_3x3(input: [[f64; 3]; 3]) -> Eigen3 {
    let mut a = input;
    let mut v = [[0.0_f64; 3]; 3];
    for i in 0..3 {
        v[i][i] = 1.0;
    }

    for _ in 0..64 {
        let mut p = 0;
        let mut q = 1;
        let mut max_value = a[0][1].abs();
        for i in 0..3 {
            for j in (i + 1)..3 {
                let candidate = a[i][j].abs();
                if candidate > max_value {
                    max_value = candidate;
                    p = i;
                    q = j;
                }
            }
        }

        if max_value < 1.0e-10 {
            break;
        }

        let app = a[p][p];
        let aqq = a[q][q];
        let apq = a[p][q];
        if apq.abs() < 1.0e-20 {
            continue;
        }

        let tau = (aqq - app) / (2.0 * apq);
        let sign = if tau >= 0.0 { 1.0 } else { -1.0 };
        let t = sign / (tau.abs() + (1.0 + tau * tau).sqrt());
        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = t * c;

        for k in 0..3 {
            if k != p && k != q {
                let akp = a[k][p];
                let akq = a[k][q];
                a[k][p] = c * akp - s * akq;
                a[p][k] = a[k][p];
                a[k][q] = s * akp + c * akq;
                a[q][k] = a[k][q];
            }
        }

        a[p][p] = c * c * app - 2.0 * s * c * apq + s * s * aqq;
        a[q][q] = s * s * app + 2.0 * s * c * apq + c * c * aqq;
        a[p][q] = 0.0;
        a[q][p] = 0.0;

        for row in 0..3 {
            let vip = v[row][p];
            let viq = v[row][q];
            v[row][p] = c * vip - s * viq;
            v[row][q] = s * vip + c * viq;
        }
    }

    let mut pairs = [
        (a[0][0], Vec3::new(v[0][0], v[1][0], v[2][0])),
        (a[1][1], Vec3::new(v[0][1], v[1][1], v[2][1])),
        (a[2][2], Vec3::new(v[0][2], v[1][2], v[2][2])),
    ];
    pairs.sort_by(|left, right| left.0.total_cmp(&right.0));

    Eigen3 {
        values: [pairs[0].0, pairs[1].0, pairs[2].0],
        vectors: [
            pairs[0].1.normalized().unwrap_or(Vec3::new(1.0, 0.0, 0.0)),
            pairs[1].1.normalized().unwrap_or(Vec3::new(0.0, 1.0, 0.0)),
            pairs[2].1.normalized().unwrap_or(Vec3::new(0.0, 0.0, 1.0)),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::symmetric_eigen_3x3;

    #[test]
    fn diagonal_matrix_eigenvalues_are_sorted() {
        let eigen = symmetric_eigen_3x3([[3.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 2.0]]);
        assert!((eigen.values[0] - 1.0).abs() < 1.0e-8);
        assert!((eigen.values[1] - 2.0).abs() < 1.0e-8);
        assert!((eigen.values[2] - 3.0).abs() < 1.0e-8);
    }
}
