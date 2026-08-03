use crate::traits::SimilarityMetric;
use crate::error::TextDistanceError;

pub struct SmithWaterman {
    pub gap_cost: f64,
}

impl Default for SmithWaterman {
    fn default() -> Self {
        SmithWaterman { gap_cost: 1.0 }
    }
}

impl SmithWaterman {
    pub fn new() -> Self {
        SmithWaterman::default()
    }
}

impl<T: PartialEq> SimilarityMetric<T> for SmithWaterman {
    fn similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        if s1 == s2 {
            return Ok(self.maximum(s1, s2));
        }
        
        let rows = s1.len() + 1;
        let cols = s2.len() + 1;

        let mut prev = vec![0.0; cols];
        let mut cur = vec![0.0; cols];

        for i in 1..rows {
            cur[0] = 0.0; // In SmithWaterman, initialization is 0.0 everywhere
            for j in 1..cols {
                let match_score = prev[j - 1] + if s1[i - 1] == s2[j - 1] { 1.0 } else { 0.0 };
                let delete = prev[j] - self.gap_cost;
                let insert = cur[j - 1] - self.gap_cost;
                cur[j] = 0f64.max(match_score).max(delete).max(insert);
            }
            prev.copy_from_slice(&cur);
        }

        Ok(prev[cols - 1])
    }

    fn maximum(&self, s1: &[T], s2: &[T]) -> f64 {
        s1.len().min(s2.len()) as f64
    }
}
