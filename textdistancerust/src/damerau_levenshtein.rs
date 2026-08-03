// src/damerau_levenshtein.rs
use crate::error::TextDistanceError;
use crate::traits::{DistanceMetric, SimilarityMetric};

#[derive(Debug, Clone, Copy, Default)]
pub struct DamerauLevenshtein;

impl DamerauLevenshtein {
    pub fn new() -> Self {
        DamerauLevenshtein
    }
}

impl DistanceMetric<char> for DamerauLevenshtein {
    fn distance(&self, s1: &[char], s2: &[char]) -> Result<f64, TextDistanceError> {
        // Restricted Damerau-Levenshtein (optimal string alignment)
        let len1 = s1.len();
        let len2 = s2.len();
        if len1 == 0 {
            return Ok(len2 as f64);
        }
        if len2 == 0 {
            return Ok(len1 as f64);
        }
        let mut dp = vec![vec![0usize; len2 + 1]; len1 + 1];
        for (i, row) in dp.iter_mut().enumerate().take(len1 + 1) {
            row[0] = i;
        }
        for (j, val) in dp[0].iter_mut().enumerate().take(len2 + 1) {
            *val = j;
        }
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };
                let deletion = dp[i - 1][j] + 1;
                let insertion = dp[i][j - 1] + 1;
                let substitution = dp[i - 1][j - 1] + cost;
                let mut val = deletion.min(insertion).min(substitution);
                if i > 1 && j > 1 && s1[i - 1] == s2[j - 2] && s1[i - 2] == s2[j - 1] {
                    // transposition
                    val = val.min(dp[i - 2][j - 2] + 1);
                }
                dp[i][j] = val;
            }
        }
        Ok(dp[len1][len2] as f64)
    }

    fn maximum(&self, s1: &[char], s2: &[char]) -> f64 {
        // Maximum possible distance is the longer length
        (s1.len().max(s2.len())) as f64
    }
}

impl SimilarityMetric<char> for DamerauLevenshtein {
    fn similarity(&self, s1: &[char], s2: &[char]) -> Result<f64, TextDistanceError> {
        Ok(DistanceMetric::maximum(self, s1, s2) - DistanceMetric::distance(self, s1, s2)?)
    }

    fn maximum(&self, s1: &[char], s2: &[char]) -> f64 {
        DistanceMetric::maximum(self, s1, s2)
    }
}
