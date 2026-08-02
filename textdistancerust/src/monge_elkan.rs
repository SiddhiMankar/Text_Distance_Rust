// src/monge_elkan.rs
use crate::error::TextDistanceError;
use crate::traits::{SimilarityMetric, DistanceMetric};
use std::fmt::Debug;

/// Monge‑Elkan similarity metric.
/// Defaults to a restricted Damerau‑Levenshtein inner metric.
/// Replicates the known double‑division bug from the Python implementation.
#[derive(Debug, Clone, Copy)]
pub struct MongeElkan<A> {
    algorithm: A,
    symmetric: bool,
    qval: usize,
    external: bool,
}

impl<A> MongeElkan<A>
where
    A: SimilarityMetric<char> + DistanceMetric<char> + Default + Clone,
{
    pub fn new() -> Self {
        MongeElkan {
            algorithm: A::default(),
            symmetric: false,
            qval: 1,
            external: true,
        }
    }
}

impl<A> SimilarityMetric<char> for MongeElkan<A>
where
    A: SimilarityMetric<char> + DistanceMetric<char> + Default + Clone,
{
    fn similarity(&self, s1: &[char], s2: &[char]) -> Result<f64, TextDistanceError> {
        self._calc(&[s1, s2])
    }

    fn maximum(&self, s1: &[char], s2: &[char]) -> f64 {
        // Forward to inner metric's maximum for each sequence.
        // Monge‑Elkan's maximum is defined as the maximum of the inner metric's maximums.
        let max1 = DistanceMetric::maximum(&self.algorithm, s1, s1);
        let max2 = DistanceMetric::maximum(&self.algorithm, s2, s2);
        max1.max(max2)
    }
}

impl<A> MongeElkan<A>
where
    A: SimilarityMetric<char> + DistanceMetric<char> + Default + Clone,
{
    fn _calc(&self, sequences: &[&[char]]) -> Result<f64, TextDistanceError> {
        if sequences.is_empty() {
            return Ok(0.0);
        }
        // `seq` is the primary (first) sequence.
        let seq = sequences[0];
        if seq.is_empty() {
            return Ok(0.0);
        }
        let mut maxes: Vec<f64> = Vec::new();
        for &c1 in seq {
            for &other in &sequences[1..] {
                let mut max_sim = f64::NEG_INFINITY;
                for &c2 in other {
                    let sim = SimilarityMetric::similarity(&self.algorithm, &[c1], &[c2])?;
                    if sim > max_sim {
                        max_sim = sim;
                    }
                }
                maxes.push(max_sim);
            }
        }
        // BUG: double division (len(seq) appears twice)
        let len_seq = seq.len() as f64;
        let len_maxes = maxes.len() as f64;
        if len_seq == 0.0 || len_maxes == 0.0 {
            return Ok(0.0);
        }
        Ok(maxes.iter().sum::<f64>() / len_seq / len_maxes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::damerau_levenshtein::DamerauLevenshtein;
    use crate::traits::SimilarityMetric;
    
    #[test]
    fn test_monge_elkan_simple() {
        let me = MongeElkan::<DamerauLevenshtein>::new();
        let s1: Vec<char> = "cat".chars().collect();
        let s2: Vec<char> = "hat".chars().collect();
        let sim = me.similarity(&s1, &s2).unwrap();
        assert!(sim >= 0.0 && sim <= 1.0);
    }
}
