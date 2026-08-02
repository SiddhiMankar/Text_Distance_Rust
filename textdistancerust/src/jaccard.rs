use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::error::TextDistanceError;
use crate::traits::SimilarityMetric;

#[derive(Debug, Clone, Copy)]
pub struct Jaccard {
    pub qval: usize,
    pub as_set: bool,
}

impl Default for Jaccard {
    fn default() -> Self {
        Jaccard {
            qval: 1,
            as_set: false,
        }
    }
}

impl Jaccard {
    pub fn new() -> Self {
        Jaccard::default()
    }

    pub fn with_config(qval: usize, as_set: bool) -> Self {
        Jaccard { qval, as_set }
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
            let union = set1.union(&set2).count();
            if union == 0 {
                return 0.0;
            }
            intersection as f64 / union as f64
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
            let mut union = 0usize;

            let mut all_keys: HashSet<&T> = HashSet::new();
            all_keys.extend(counts1.keys());
            all_keys.extend(counts2.keys());

            for key in all_keys {
                let c1 = counts1.get(key).copied().unwrap_or(0);
                let c2 = counts2.get(key).copied().unwrap_or(0);
                intersection += c1.min(c2);
                union += c1.max(c2);
            }

            if union == 0 {
                return 0.0;
            }
            intersection as f64 / union as f64
        }
    }
}

impl<T: PartialEq + Eq + Hash + Clone> SimilarityMetric<T> for Jaccard {
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
    fn test_jaccard_same() {
        let j = Jaccard::new();
        let s1 = to_char_vec("hello");
        let s2 = to_char_vec("hello");
        assert_eq!(j.similarity(&s1, &s2).unwrap(), 1.0);
        assert_eq!(j.distance(&s1, &s2).unwrap(), 0.0);
    }

    #[test]
    fn test_jaccard_empty() {
        let j = Jaccard::new();
        let s1 = to_char_vec("");
        let s2 = to_char_vec("");
        assert_eq!(j.similarity(&s1, &s2).unwrap(), 0.0);
    }

    #[test]
    fn test_jaccard_asymmetric_empty() {
        let j = Jaccard::new();
        let s1 = to_char_vec("");
        let s2 = to_char_vec("abc");
        assert_eq!(j.similarity(&s1, &s2).unwrap(), 0.0);
        assert_eq!(j.distance(&s1, &s2).unwrap(), 1.0);
    }

    #[test]
    fn test_jaccard_cat_hat() {
        let j = Jaccard::new();
        let s1 = to_char_vec("cat");
        let s2 = to_char_vec("hat");
        assert_eq!(j.similarity(&s1, &s2).unwrap(), 0.5);
        assert_eq!(j.distance(&s1, &s2).unwrap(), 0.5);
    }

    #[test]
    fn test_jaccard_as_set() {
        let j_multi = Jaccard::with_config(1, false);
        let j_set = Jaccard::with_config(1, true);
        let s1 = to_char_vec("aa");
        let s2 = to_char_vec("a");
        assert_eq!(j_multi.similarity(&s1, &s2).unwrap(), 0.5);
        assert_eq!(j_set.similarity(&s1, &s2).unwrap(), 1.0);
    }
}
