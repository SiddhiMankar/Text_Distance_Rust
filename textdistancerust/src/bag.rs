use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::error::TextDistanceError;
use crate::traits::DistanceMetric;

#[derive(Debug, Clone, Copy)]
pub struct Bag {
    pub qval: usize,
    pub as_set: bool,
}

impl Default for Bag {
    fn default() -> Self {
        Bag {
            qval: 1,
            as_set: false,
        }
    }
}

impl Bag {
    pub fn new() -> Self {
        Bag::default()
    }

    pub fn with_config(qval: usize, as_set: bool) -> Self {
        Bag { qval, as_set }
    }

    pub fn distance_sequences<T: PartialEq + Eq + Hash + Clone>(&self, s1: &[T], s2: &[T]) -> f64 {
        if s1.is_empty() && s2.is_empty() {
            return 0.0;
        }

        let (intersection, count1, count2) = if self.as_set {
            let set1: HashSet<&T> = s1.iter().collect();
            let set2: HashSet<&T> = s2.iter().collect();
            let inter = set1.intersection(&set2).count();
            (inter, set1.len(), set2.len())
        } else {
            let mut counts1: HashMap<&T, usize> = HashMap::new();
            for item in s1 {
                *counts1.entry(item).or_insert(0) += 1;
            }

            let mut counts2: HashMap<&T, usize> = HashMap::new();
            for item in s2 {
                *counts2.entry(item).or_insert(0) += 1;
            }

            let mut inter = 0usize;
            let mut all_keys: HashSet<&T> = HashSet::new();
            all_keys.extend(counts1.keys());
            all_keys.extend(counts2.keys());

            for key in all_keys {
                let c1 = counts1.get(key).copied().unwrap_or(0);
                let c2 = counts2.get(key).copied().unwrap_or(0);
                inter += c1.min(c2);
            }

            let c1: usize = counts1.values().sum();
            let c2: usize = counts2.values().sum();
            (inter, c1, c2)
        };

        let diff1 = count1.saturating_sub(intersection);
        let diff2 = count2.saturating_sub(intersection);
        diff1.max(diff2) as f64
    }
}

impl<T: PartialEq + Eq + Hash + Clone> DistanceMetric<T> for Bag {
    fn distance(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        Ok(self.distance_sequences(s1, s2))
    }

    fn maximum(&self, s1: &[T], s2: &[T]) -> f64 {
        s1.len().max(s2.len()) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::to_char_vec;

    #[test]
    fn test_bag_same() {
        let b = Bag::new();
        let s1 = to_char_vec("cat");
        let s2 = to_char_vec("cat");
        assert_eq!(b.distance(&s1, &s2).unwrap(), 0.0);
        assert_eq!(b.similarity(&s1, &s2).unwrap(), 3.0);
    }

    #[test]
    fn test_bag_empty() {
        let b = Bag::new();
        let s1 = to_char_vec("");
        let s2 = to_char_vec("");
        assert_eq!(b.distance(&s1, &s2).unwrap(), 0.0);
    }

    #[test]
    fn test_bag_cat_hat() {
        let b = Bag::new();
        let s1 = to_char_vec("cat");
        let s2 = to_char_vec("hat");
        assert_eq!(b.distance(&s1, &s2).unwrap(), 1.0);
        assert_eq!(b.similarity(&s1, &s2).unwrap(), 2.0);
        assert_eq!(b.maximum(&s1, &s2), 3.0);
        let norm_dist = b.normalized_distance(&s1, &s2).unwrap();
        assert!((norm_dist - 1.0 / 3.0).abs() < 1e-9);
    }
}
