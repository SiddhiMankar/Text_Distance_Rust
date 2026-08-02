use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::error::TextDistanceError;
use crate::traits::SimilarityMetric;

#[derive(Debug, Clone, Copy)]
pub struct Sorensen {
    pub qval: usize,
    pub as_set: bool,
}

impl Default for Sorensen {
    fn default() -> Self {
        Sorensen {
            qval: 1,
            as_set: false,
        }
    }
}

impl Sorensen {
    pub fn new() -> Self {
        Sorensen::default()
    }

    pub fn with_config(qval: usize, as_set: bool) -> Self {
        Sorensen { qval, as_set }
    }

    pub fn similarity_sequences<T: PartialEq + Eq + Hash + Clone>(
        &self,
        s1: &[T],
        s2: &[T],
    ) -> f64 {
        if s1.is_empty() || s2.is_empty() {
            return 0.0;
        }

        if self.as_set {
            let set1: HashSet<&T> = s1.iter().collect();
            let set2: HashSet<&T> = s2.iter().collect();
            let intersection = set1.intersection(&set2).count();
            let total_count = set1.len() + set2.len();
            if total_count == 0 {
                return 0.0;
            }
            2.0 * intersection as f64 / total_count as f64
        } else {
            let mut counts1: HashMap<&T, usize> = HashMap::new();
            for item in s1 {
                *counts1.entry(item).or_insert(0) += 1;
            }

            let mut counts2: HashMap<&T, usize> = HashMap::new();
            for item in s2 {
                *counts2.entry(item).or_insert(0) += 1;
            }

            let mut intersection = 0usize;
            let mut all_keys: HashSet<&T> = HashSet::new();
            all_keys.extend(counts1.keys());
            all_keys.extend(counts2.keys());

            for key in all_keys {
                let c1 = counts1.get(key).copied().unwrap_or(0);
                let c2 = counts2.get(key).copied().unwrap_or(0);
                intersection += c1.min(c2);
            }

            let count1: usize = counts1.values().sum();
            let count2: usize = counts2.values().sum();
            let total_count = count1 + count2;

            if total_count == 0 {
                return 0.0;
            }
            2.0 * intersection as f64 / total_count as f64
        }
    }
}

impl<T: PartialEq + Eq + Hash + Clone> SimilarityMetric<T> for Sorensen {
    fn similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        Ok(self.similarity_sequences(s1, s2))
    }

    fn maximum(&self, _s1: &[T], _s2: &[T]) -> f64 {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::to_char_vec;

    #[test]
    fn test_sorensen_same() {
        let s = Sorensen::new();
        let s1 = to_char_vec("hello");
        let s2 = to_char_vec("hello");
        assert_eq!(s.similarity(&s1, &s2).unwrap(), 1.0);
        assert_eq!(s.distance(&s1, &s2).unwrap(), 0.0);
    }

    #[test]
    fn test_sorensen_empty() {
        let s = Sorensen::new();
        let s1 = to_char_vec("");
        let s2 = to_char_vec("");
        assert_eq!(s.similarity(&s1, &s2).unwrap(), 0.0);
    }

    #[test]
    fn test_sorensen_asymmetric_empty() {
        let s = Sorensen::new();
        let s1 = to_char_vec("");
        let s2 = to_char_vec("abc");
        assert_eq!(s.similarity(&s1, &s2).unwrap(), 0.0);
        assert_eq!(s.distance(&s1, &s2).unwrap(), 1.0);
    }

    #[test]
    fn test_sorensen_cat_hat() {
        let s = Sorensen::new();
        let s1 = to_char_vec("cat");
        let s2 = to_char_vec("hat");
        let sim = s.similarity(&s1, &s2).unwrap();
        assert!((sim - 2.0 / 3.0).abs() < 1e-9);
    }
}
