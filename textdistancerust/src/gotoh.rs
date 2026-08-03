use crate::traits::SimilarityMetric;
use crate::error::TextDistanceError;

pub struct Gotoh {
    pub gap_open: f64,
    pub gap_ext: f64,
}

impl Default for Gotoh {
    fn default() -> Self {
        Gotoh {
            gap_open: 1.0,
            gap_ext: 0.4,
        }
    }
}

impl Gotoh {
    pub fn new() -> Self {
        Gotoh::default()
    }
}

impl<T: PartialEq> SimilarityMetric<T> for Gotoh {
    fn similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        let rows = s1.len() + 1;
        let cols = s2.len() + 1;

        let mut d_prev = vec![0.0; cols];
        let mut d_cur = vec![0.0; cols];
        let mut p_prev = vec![0.0; cols];
        let mut p_cur = vec![0.0; cols];
        let mut q_prev = vec![0.0; cols];
        let mut q_cur = vec![0.0; cols];

        d_prev[0] = 0.0;
        p_prev[0] = std::f64::NEG_INFINITY;
        q_prev[0] = std::f64::NEG_INFINITY;
        
        for j in 1..cols {
            d_prev[j] = std::f64::NEG_INFINITY;
            p_prev[j] = std::f64::NEG_INFINITY;
            q_prev[j] = -self.gap_open - self.gap_ext * (j as f64 - 1.0);
        }

        for i in 1..rows {
            d_cur[0] = std::f64::NEG_INFINITY;
            p_cur[0] = -self.gap_open - self.gap_ext * (i as f64 - 1.0);
            q_cur[0] = std::f64::NEG_INFINITY;
            
            for j in 1..cols {
                let sim_val = if s1[i - 1] == s2[j - 1] { 1.0 } else { 0.0 };
                
                d_cur[j] = (d_prev[j - 1] + sim_val)
                    .max(p_prev[j - 1] + sim_val)
                    .max(q_prev[j - 1] + sim_val);
                    
                p_cur[j] = (d_prev[j] - self.gap_open)
                    .max(p_prev[j] - self.gap_ext);
                    
                q_cur[j] = (d_cur[j - 1] - self.gap_open)
                    .max(q_cur[j - 1] - self.gap_ext);
            }
            
            d_prev.copy_from_slice(&d_cur);
            p_prev.copy_from_slice(&p_cur);
            q_prev.copy_from_slice(&q_cur);
        }

        Ok(d_prev[cols - 1].max(p_prev[cols - 1]).max(q_prev[cols - 1]))
    }

    fn maximum(&self, s1: &[T], s2: &[T]) -> f64 {
        s1.len().min(s2.len()) as f64
    }

    fn distance(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        Ok(-self.similarity(s1, s2)?)
    }

    fn normalized_distance(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        let max = self.maximum(s1, s2);
        if max == 0.0 {
            return Ok(0.0);
        }
        let min = -max;
        Ok((self.distance(s1, s2)? - min) / (max - min))
    }

    fn normalized_similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        let max = self.maximum(s1, s2);
        if max == 0.0 {
            return Ok(1.0);
        }
        let min = -max;
        Ok((self.similarity(s1, s2)? - min) / (max * 2.0))
    }
}
