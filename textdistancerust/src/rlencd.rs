// src/rlencd.rs
//! Run-Length Encoding based Normalized Compression Distance.
//! Mirrors Python textdistance RLENCD exactly.

use crate::error::TextDistanceError;
use crate::ncd_base::NcdBase;
use crate::traits::SimilarityMetric;

#[derive(Debug, Clone, Copy, Default)]
pub struct RlenCd;

impl RlenCd {
    pub fn new() -> Self {
        RlenCd
    }

    /// Run-length encode a char sequence.
    /// Matches Python's RLENCD._compress using itertools.groupby logic:
    ///   n > 2  => str(n) + char
    ///   n == 1 => char
    ///   n == 2 => char * 2
    fn compress(&self, data: &[char]) -> String {
        let mut result = String::new();
        let mut i = 0;
        while i < data.len() {
            let c = data[i];
            let mut count = 1usize;
            while i + count < data.len() && data[i + count] == c {
                count += 1;
            }
            if count > 2 {
                result.push_str(&format!("{}{}", count, c));
            } else if count == 1 {
                result.push(c);
            } else {
                // count == 2
                result.push(c);
                result.push(c);
            }
            i += count;
        }
        result
    }

    /// Get the compressed size of a char sequence.
    fn get_size(&self, data: &[char]) -> usize {
        self.compress(data).chars().count()
    }
}

impl SimilarityMetric<char> for RlenCd {
    fn similarity(&self, s1: &[char], s2: &[char]) -> Result<f64, TextDistanceError> {
        Ok(1.0 - self.distance(s1, s2)?)
    }

    fn distance(&self, s1: &[char], s2: &[char]) -> Result<f64, TextDistanceError> {
        // Compressed sizes for individual sequences.
        let size1 = self.get_size(s1);
        let size2 = self.get_size(s2);
        // Concatenate RAW data (not compressed), then compress the concatenation.
        // This mirrors Python's _NCDBase.__call__ which joins raw sequences,
        // then calls _get_size (which calls _compress) on the joined data.
        let concat1: Vec<char> = s1.iter().chain(s2.iter()).copied().collect();
        let concat2: Vec<char> = s2.iter().chain(s1.iter()).copied().collect();
        let size_concat1 = self.get_size(&concat1);
        let size_concat2 = self.get_size(&concat2);
        let min_concat = size_concat1.min(size_concat2);
        Ok(NcdBase::compute_distance(min_concat, &[size1, size2]))
    }

    fn maximum(&self, _s1: &[char], _s2: &[char]) -> f64 {
        1.0
    }
}
