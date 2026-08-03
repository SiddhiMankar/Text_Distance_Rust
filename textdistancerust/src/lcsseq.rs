use crate::traits::SimilarityMetric;
use crate::error::TextDistanceError;

pub struct LcsSeq;

impl Default for LcsSeq {
    fn default() -> Self {
        LcsSeq
    }
}

impl LcsSeq {
    pub fn new() -> Self {
        LcsSeq::default()
    }
}

impl<T: PartialEq> SimilarityMetric<T> for LcsSeq {
    fn similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        let rows = s1.len() + 1;
        let cols = s2.len() + 1;

        let mut dist_prev = vec![0; cols];
        let mut dist_cur = vec![0; cols];

        for i in 1..rows {
            for j in 1..cols {
                if s1[i - 1] == s2[j - 1] {
                    dist_cur[j] = dist_prev[j - 1] + 1;
                } else {
                    dist_cur[j] = dist_cur[j - 1].max(dist_prev[j]);
                }
            }
            dist_prev.copy_from_slice(&dist_cur);
        }

        Ok(dist_prev[cols - 1] as f64)
    }

    fn maximum(&self, s1: &[T], s2: &[T]) -> f64 {
        s1.len().max(s2.len()) as f64
    }
}
