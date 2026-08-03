use crate::traits::SimilarityMetric;
use crate::error::TextDistanceError;

pub struct NeedlemanWunsch {
    pub gap_cost: f64,
}

impl Default for NeedlemanWunsch {
    fn default() -> Self {
        NeedlemanWunsch { gap_cost: 1.0 }
    }
}

impl NeedlemanWunsch {
    pub fn new() -> Self {
        NeedlemanWunsch::default()
    }
}

impl<T: PartialEq> SimilarityMetric<T> for NeedlemanWunsch {
    fn similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        let rows = s1.len() + 1;
        let cols = s2.len() + 1;

        let mut prev = vec![0.0; cols];
        let mut cur = vec![0.0; cols];

        for j in 0..cols {
            prev[j] = -(j as f64 * self.gap_cost);
        }

        for i in 1..rows {
            cur[0] = -(i as f64 * self.gap_cost);
            for j in 1..cols {
                let match_score = prev[j - 1] + if s1[i - 1] == s2[j - 1] { 1.0 } else { 0.0 };
                let delete = prev[j] - self.gap_cost;
                let insert = cur[j - 1] - self.gap_cost;
                cur[j] = match_score.max(delete).max(insert);
            }
            prev.copy_from_slice(&cur);
        }

        Ok(prev[cols - 1])
    }

    fn maximum(&self, s1: &[T], s2: &[T]) -> f64 {
        s1.len().max(s2.len()) as f64
    }

    fn distance(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        Ok(-self.similarity(s1, s2)?)
    }

    fn normalized_distance(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        let max = self.maximum(s1, s2);
        if max == 0.0 {
            return Ok(0.0);
        }
        let min = -max * self.gap_cost;
        Ok((self.distance(s1, s2)? - min) / (max - min))
    }

    fn normalized_similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        let max = self.maximum(s1, s2);
        if max == 0.0 {
            return Ok(1.0);
        }
        let min = -max * self.gap_cost;
        Ok((self.similarity(s1, s2)? - min) / (max * 2.0))
    }
}
