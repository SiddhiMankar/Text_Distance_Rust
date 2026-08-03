// src/ncd_base.rs
/// Shared logic for Normalized Compression Distance (NCD) algorithms.
/// All algorithms implement `SimilarityMetric<char>` and use this helper.
pub struct NcdBase;

impl NcdBase {
    /// Compute NCD distance given the minimal concatenated compressed size
    /// and the individual compressed sizes.
    /// Uses f64 for sizes to support non-integer compressed sizes (e.g. SqrtNCD).
    pub fn compute_distance_f64(min_concat: f64, sizes: &[f64]) -> f64 {
        let min_size = sizes.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_size = sizes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if max_size == 0.0 {
            return 0.0;
        }
        (min_concat - min_size * (sizes.len() as f64 - 1.0)) / max_size
    }

    /// Integer-based convenience wrapper (used by RLENCD and ArithNCD).
    pub fn compute_distance(min_concat: usize, sizes: &[usize]) -> f64 {
        let f_sizes: Vec<f64> = sizes.iter().map(|&s| s as f64).collect();
        Self::compute_distance_f64(min_concat as f64, &f_sizes)
    }
}
