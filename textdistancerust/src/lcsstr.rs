use crate::traits::SimilarityMetric;
use crate::error::TextDistanceError;

pub struct LcsStr;

impl Default for LcsStr {
    fn default() -> Self {
        LcsStr
    }
}

impl LcsStr {
    pub fn new() -> Self {
        LcsStr::default()
    }
}

impl<T: PartialEq> SimilarityMetric<T> for LcsStr {
    fn similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        if s1.is_empty() || s2.is_empty() {
            return Ok(0.0);
        }

        let rows = s1.len() + 1;
        let cols = s2.len() + 1;

        let mut dist_prev = vec![0; cols];
        let mut dist_cur = vec![0; cols];
        let mut max_len = 0;

        for i in 1..rows {
            for j in 1..cols {
                if s1[i - 1] == s2[j - 1] {
                    dist_cur[j] = dist_prev[j - 1] + 1;
                    if dist_cur[j] > max_len {
                        max_len = dist_cur[j];
                    }
                } else {
                    dist_cur[j] = 0;
                }
            }
            dist_prev.copy_from_slice(&dist_cur);
        }

        Ok(max_len as f64)
    }

    fn maximum(&self, s1: &[T], s2: &[T]) -> f64 {
        s1.len().max(s2.len()) as f64
    }
}
