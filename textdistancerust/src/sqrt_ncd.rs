// src/sqrt_ncd.rs
//! Square Root based Normalized Compression Distance.
//! Mirrors Python textdistance SqrtNCD exactly.
//! Size = sum of sqrt(count) for each unique element.

use crate::error::TextDistanceError;
use crate::ncd_base::NcdBase;
use crate::traits::SimilarityMetric;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Default)]
pub struct SqrtNcd;

impl SqrtNcd {
    pub fn new() -> Self {
        SqrtNcd
    }

    /// Get the "compressed size" of a sequence: sum of sqrt(count) for each element.
    /// Returns f64 to match Python which does NOT round.
    fn get_size(&self, data: &[char]) -> f64 {
        let mut counter: HashMap<char, usize> = HashMap::new();
        for &c in data {
            *counter.entry(c).or_insert(0) += 1;
        }
        counter.values().map(|&cnt| (cnt as f64).sqrt()).sum()
    }
}

impl SimilarityMetric<char> for SqrtNcd {
    fn similarity(&self, s1: &[char], s2: &[char]) -> Result<f64, TextDistanceError> {
        Ok(1.0 - self.distance(s1, s2)?)
    }

    fn distance(&self, s1: &[char], s2: &[char]) -> Result<f64, TextDistanceError> {
        let size1 = self.get_size(s1);
        let size2 = self.get_size(s2);
        // Concatenate raw data, compute size of concatenations.
        let concat1: Vec<char> = s1.iter().chain(s2.iter()).copied().collect();
        let concat2: Vec<char> = s2.iter().chain(s1.iter()).copied().collect();
        let size_concat1 = self.get_size(&concat1);
        let size_concat2 = self.get_size(&concat2);
        let min_concat = size_concat1.min(size_concat2);
        Ok(NcdBase::compute_distance_f64(min_concat, &[size1, size2]))
    }

    fn maximum(&self, _s1: &[char], _s2: &[char]) -> f64 {
        1.0
    }
}
