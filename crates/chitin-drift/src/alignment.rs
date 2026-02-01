// crates/chitin-drift/src/alignment.rs
//
// Cross-model vector space alignment (linear projection) for the Chitin Protocol.

use serde::{Deserialize, Serialize};

/// A linear projection matrix for aligning two vector spaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentMatrix {
    /// Dimensionality of the source model's vectors.
    pub from_dim: u32,
    /// Dimensionality of the target model's vectors.
    pub to_dim: u32,
    /// Flattened projection matrix of size from_dim * to_dim (row-major).
    pub matrix: Vec<f64>,
}

/// Compute an alignment matrix from paired vector samples using gradient descent.
///
/// Given corresponding vectors from two models (same texts embedded by both),
/// learns a linear projection that minimizes MSE between `from * M` and `to`.
///
/// # Arguments
/// * `from_vectors` - Vectors from the source model (one per sample text).
/// * `to_vectors` - Vectors from the target model (one per sample text).
///
/// # Panics
/// Panics if `from_vectors` and `to_vectors` have different lengths or are empty.
pub fn compute_alignment(
    from_vectors: &[Vec<f32>],
    to_vectors: &[Vec<f32>],
) -> AlignmentMatrix {
    assert!(
        !from_vectors.is_empty(),
        "from_vectors must not be empty"
    );
    assert_eq!(
        from_vectors.len(),
        to_vectors.len(),
        "from and to vectors must have same count"
    );

    let n = from_vectors.len();
    let d_from = from_vectors[0].len();
    let d_to = to_vectors[0].len();

    // Initialize M as zeros (d_from x d_to)
    let mut matrix = vec![0.0f64; d_from * d_to];

    let lr = 0.01;
    let iterations = 1000;

    for _ in 0..iterations {
        // Compute gradient: dL/dM = (2/N) * sum_i( from_i^T * (from_i * M - to_i) )
        let mut grad = vec![0.0f64; d_from * d_to];

        for i in 0..n {
            // Compute predicted = from_i * M (1 x d_to)
            let mut predicted = vec![0.0f64; d_to];
            for j in 0..d_to {
                for k in 0..d_from {
                    predicted[j] += from_vectors[i][k] as f64 * matrix[k * d_to + j];
                }
            }

            // Compute error = predicted - to_i
            let mut error = vec![0.0f64; d_to];
            for j in 0..d_to {
                error[j] = predicted[j] - to_vectors[i][j] as f64;
            }

            // Accumulate gradient: from_i^T * error (d_from x d_to)
            for k in 0..d_from {
                for j in 0..d_to {
                    grad[k * d_to + j] += from_vectors[i][k] as f64 * error[j];
                }
            }
        }

        // Update M: M -= lr * (2/N) * grad
        let scale = 2.0 * lr / n as f64;
        for idx in 0..matrix.len() {
            matrix[idx] -= scale * grad[idx];
        }
    }

    AlignmentMatrix {
        from_dim: d_from as u32,
        to_dim: d_to as u32,
        matrix,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignment_matrix_dimensions() {
        let from = vec![vec![1.0f32, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
        let to = vec![vec![1.0f32, 2.0], vec![3.0, 4.0]];
        let mat = compute_alignment(&from, &to);
        assert_eq!(mat.from_dim, 3);
        assert_eq!(mat.to_dim, 2);
        assert_eq!(mat.matrix.len(), 6); // 3 * 2
    }

    #[test]
    fn alignment_reduces_error() {
        // Simple test: identity-like mapping (from 2D to 2D)
        let from = vec![
            vec![1.0f32, 0.0],
            vec![0.0, 1.0],
            vec![1.0, 1.0],
            vec![2.0, 3.0],
        ];
        let to = vec![
            vec![1.0f32, 0.0],
            vec![0.0, 1.0],
            vec![1.0, 1.0],
            vec![2.0, 3.0],
        ];
        let mat = compute_alignment(&from, &to);

        // Apply the learned matrix and check error is small
        let mut total_error = 0.0;
        for i in 0..from.len() {
            for j in 0..to[0].len() {
                let mut predicted = 0.0;
                for k in 0..from[0].len() {
                    predicted += from[i][k] as f64 * mat.matrix[k * to[0].len() + j];
                }
                let err = predicted - to[i][j] as f64;
                total_error += err * err;
            }
        }
        let mse = total_error / (from.len() * to[0].len()) as f64;
        assert!(
            mse < 0.1,
            "MSE should be small for identity mapping, got {}",
            mse
        );
    }

    #[test]
    fn identity_alignment() {
        // When from == to, the learned matrix should approximate identity
        let from = vec![
            vec![1.0f32, 0.0],
            vec![0.0, 1.0],
            vec![0.5, 0.5],
        ];
        let to = from.clone();
        let mat = compute_alignment(&from, &to);

        // Check diagonal elements are close to 1.0
        assert!(
            (mat.matrix[0] - 1.0).abs() < 0.3,
            "M[0,0] should be ~1.0, got {}",
            mat.matrix[0]
        );
        assert!(
            (mat.matrix[3] - 1.0).abs() < 0.3,
            "M[1,1] should be ~1.0, got {}",
            mat.matrix[3]
        );
    }
}
