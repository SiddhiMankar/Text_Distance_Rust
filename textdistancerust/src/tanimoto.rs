use std::hash::Hash;

use crate::error::TextDistanceError;
use crate::jaccard::Jaccard;
use crate::traits::SimilarityMetric;

#[derive(Debug, Clone, Copy, Default)]
pub struct Tanimoto {
    pub jaccard: Jaccard,
}

impl Tanimoto {
    pub fn new() -> Self {
        Tanimoto::default()
    }

    pub fn with_config(qval: usize, as_set: bool) -> Self {
        Tanimoto {
            jaccard: Jaccard::with_config(qval, as_set),
        }
    }

    pub fn similarity_sequences<T: PartialEq + Eq + Hash + Clone>(
        &self,
        s1: &[T],
        s2: &[T],
    ) -> f64 {
        if s1.is_empty() || s2.is_empty() {
            return f64::NEG_INFINITY;
        }

        let j_sim = self.jaccard.similarity_sequences(s1, s2);
        if j_sim == 0.0 {
            f64::NEG_INFINITY
        } else {
            j_sim.log2()
        }
    }
}

impl<T: PartialEq + Eq + Hash + Clone> SimilarityMetric<T> for Tanimoto {
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
    fn test_tanimoto_same() {
        let t = Tanimoto::new();
        let s1 = to_char_vec("hello");
        let s2 = to_char_vec("hello");
        assert_eq!(t.similarity(&s1, &s2).unwrap(), 0.0);
        assert_eq!(t.distance(&s1, &s2).unwrap(), 1.0);
    }

    #[test]
    fn test_tanimoto_empty() {
        let t = Tanimoto::new();
        let s1 = to_char_vec("");
        let s2 = to_char_vec("");
        assert_eq!(t.similarity(&s1, &s2).unwrap(), f64::NEG_INFINITY);
    }

    #[test]
    fn test_tanimoto_disjoint() {
        let t = Tanimoto::new();
        let s1 = to_char_vec("abc");
        let s2 = to_char_vec("def");
        assert_eq!(t.similarity(&s1, &s2).unwrap(), f64::NEG_INFINITY);
    }

    #[test]
    fn test_tanimoto_cat_hat() {
        let t = Tanimoto::new();
        let s1 = to_char_vec("cat");
        let s2 = to_char_vec("hat");
        assert_eq!(t.similarity(&s1, &s2).unwrap(), -1.0);
        assert_eq!(t.distance(&s1, &s2).unwrap(), 2.0);
    }
}
