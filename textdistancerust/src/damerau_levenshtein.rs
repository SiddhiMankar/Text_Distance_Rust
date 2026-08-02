// src/damerau_levenshtein.rs
use crate::error::TextDistanceError;
use crate::traits::{DistanceMetric, SimilarityMetric};

#[derive(Debug, Clone, Copy)]
pub struct DamerauLevenshtein;

impl DamerauLevenshtein {
    pub fn new() -> Self {
        DamerauLevenshtein
    }
}

impl Default for DamerauLevenshtein {
    fn default() -> Self { DamerauLevenshtein }
}

impl SimilarityMetric<char> for DamerauLevenshtein {
    fn similarity(&self, s1: &[char], s2: &[char]) -> Result<f64, TextDistanceError> {
        // Compute similarity using distance and maximum from DistanceMetric
        let dist = DistanceMetric::distance(self, s1, s2)?;
        let max = DistanceMetric::maximum(self, s1, s2);
        if max == 0.0 { return Ok(1.0); }
        Ok(1.0 - dist / max)
    }

    fn maximum(&self, s1: &[char], s2: &[char]) -> f64 {
        std::cmp::max(s1.len(), s2.len()) as f64
    }
}
impl<T> DistanceMetric<T> for DamerauLevenshtein
where
    T: PartialEq + Clone,
{
    fn distance(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        // Restricted Damerau-Levenshtein: Levenshtein DP + adjacent transposition check.
        let len1 = s1.len();
        let len2 = s2.len();
        if len1 == 0 {
            return Ok(len2 as f64);
        }
        if len2 == 0 {
            return Ok(len1 as f64);
        }
        // DP matrix (len1+1) x (len2+1)
        let mut dp = vec![vec![0usize; len2 + 1]; len1 + 1];
        for i in 0..=len1 {
            dp[i][0] = i;
        }
        for j in 0..=len2 {
            dp[0][j] = j;
        }
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };
                let deletion = dp[i - 1][j] + 1;
                let insertion = dp[i][j - 1] + 1;
                let substitution = dp[i - 1][j - 1] + cost;
                let mut val = deletion.min(insertion).min(substitution);
                // Adjacent transposition check (restricted)
                if i > 1 && j > 1 && s1[i - 1] == s2[j - 2] && s1[i - 2] == s2[j - 1] {
                    val = val.min(dp[i - 2][j - 2] + 1);
                }
                dp[i][j] = val;
            }
        }
        Ok(dp[len1][len2] as f64)
    }

    fn maximum(&self, s1: &[T], s2: &[T]) -> f64 {
        std::cmp::max(s1.len(), s2.len()) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::DistanceMetric;

    #[test]
    fn test_basic() {
        let dl = DamerauLevenshtein::new();
        assert_eq!(dl.distance(&["a"], &["a"]).unwrap(), 0.0);
        assert_eq!(dl.distance(&["a"], &["b"]).unwrap(), 1.0);
        assert_eq!(dl.distance(&["ab"], &["ba"]).unwrap(), 1.0); // transposition
        assert_eq!(dl.maximum(&["ab"], &["ba"]), 2.0);
    }
}
