use crate::traits::{DistanceMetric, SimilarityMetric};
use crate::error::TextDistanceError;
use crate::hamming::Hamming;

pub struct Mlipns {
    pub threshold: f64,
    pub maxmismatches: i64,
}

impl Default for Mlipns {
    fn default() -> Self {
        Mlipns {
            threshold: 0.25,
            maxmismatches: 2,
        }
    }
}

impl Mlipns {
    pub fn new() -> Self {
        Mlipns::default()
    }
}

impl<T: PartialEq> SimilarityMetric<T> for Mlipns {
    fn similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        if s1.is_empty() && s2.is_empty() {
            return Ok(1.0);
        }
        if s1 == s2 {
            return Ok(1.0);
        }

        let mut mismatches = 0;
        let hamming_metric = Hamming::new();
        let mut ham = hamming_metric.distance(s1, s2)?;
        let mut maxlen = s1.len().max(s2.len()) as f64;

        while !s1.is_empty() && !s2.is_empty() && mismatches <= self.maxmismatches {
            if maxlen == 0.0 {
                return Ok(1.0);
            }
            if 1.0 - (maxlen - ham) / maxlen <= self.threshold {
                return Ok(1.0);
            }
            mismatches += 1;
            ham -= 1.0;
            maxlen -= 1.0;
        }

        if maxlen == 0.0 {
            return Ok(1.0);
        }
        Ok(0.0)
    }

    fn maximum(&self, _s1: &[T], _s2: &[T]) -> f64 {
        1.0
    }
}
