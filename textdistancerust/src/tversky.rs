use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::error::TextDistanceError;
use crate::traits::SimilarityMetric;

#[derive(Debug, Clone, Copy)]
pub struct Tversky {
    pub alpha: f64,
    pub beta: f64,
    pub bias: Option<f64>,
    pub qval: usize,
    pub as_set: bool,
}

impl Default for Tversky {
    fn default() -> Self {
        Tversky {
            alpha: 1.0,
            beta: 1.0,
            bias: None,
            qval: 1,
            as_set: false,
        }
    }
}

impl Tversky {
    pub fn new() -> Self {
        Tversky::default()
    }

    pub fn with_config(
        alpha: f64,
        beta: f64,
        bias: Option<f64>,
        qval: usize,
        as_set: bool,
    ) -> Self {
        Tversky {
            alpha,
            beta,
            bias,
            qval,
            as_set,
        }
    }

    pub fn similarity_sequences<T: PartialEq + Eq + Hash + Clone>(
        &self,
        s1: &[T],
        s2: &[T],
    ) -> f64 {
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

        if let Some(bias) = self.bias {
            let a_val = (count1.min(count2)) as f64;
            let b_val = (count1.max(count2)) as f64;
            let c_val = (intersection as f64) + bias;
            let result = self.alpha * self.beta * (a_val - b_val) + b_val * self.beta;
            let denom = result + c_val;
            if denom == 0.0 {
                0.0
            } else {
                c_val / denom
            }
        } else {
            if count1 == 0 || count2 == 0 {
                return 0.0;
            }
            let inter_f = intersection as f64;
            let diff1 = (count1 - intersection) as f64;
            let diff2 = (count2 - intersection) as f64;
            let denom = inter_f + self.alpha * diff1 + self.beta * diff2;
            if denom == 0.0 {
                0.0
            } else {
                inter_f / denom
            }
        }
    }
}

impl<T: PartialEq + Eq + Hash + Clone> SimilarityMetric<T> for Tversky {
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
    fn test_tversky_default_jaccard_parity() {
        let t = Tversky::new();
        let s1 = to_char_vec("cat");
        let s2 = to_char_vec("hat");
        assert_eq!(t.similarity(&s1, &s2).unwrap(), 0.5);
    }

    #[test]
    fn test_tversky_dice_parity() {
        let t = Tversky::with_config(0.5, 0.5, None, 1, false);
        let s1 = to_char_vec("cat");
        let s2 = to_char_vec("hat");
        let sim = t.similarity(&s1, &s2).unwrap();
        assert!((sim - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_tversky_bias() {
        let t = Tversky::with_config(1.0, 1.0, Some(0.5), 1, false);
        let s1 = to_char_vec("cat");
        let s2 = to_char_vec("hat");
        let sim = t.similarity(&s1, &s2).unwrap();
        assert!((sim - 0.45454545454545453).abs() < 1e-9);
    }
}
