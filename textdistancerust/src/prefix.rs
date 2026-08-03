use crate::error::TextDistanceError;
use crate::traits::SimilarityMetric;

#[derive(Debug, Clone, Copy)]
pub struct Prefix {
    pub qval: usize,
}

impl Default for Prefix {
    fn default() -> Self {
        Prefix { qval: 1 }
    }
}

impl Prefix {
    pub fn new() -> Self {
        Prefix::default()
    }

    pub fn with_qval(qval: usize) -> Self {
        Prefix { qval }
    }

    pub fn prefix<'a, T: PartialEq>(&self, s1: &'a [T], s2: &'a [T]) -> &'a [T] {
        let len = s1.iter().zip(s2.iter()).take_while(|(a, b)| a == b).count();
        &s1[..len]
    }
}

impl<T: PartialEq> SimilarityMetric<T> for Prefix {
    fn similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        Ok(self.prefix(s1, s2).len() as f64)
    }

    fn maximum(&self, s1: &[T], s2: &[T]) -> f64 {
        (s1.len() as f64).max(s2.len() as f64)
    }

    fn normalized_distance(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        let max = self.maximum(s1, s2);
        if max == 0.0 {
            return Ok(0.0);
        }
        Ok(self.distance(s1, s2)? / max)
    }

    fn normalized_similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        let max = self.maximum(s1, s2);
        if max == 0.0 {
            return Ok(1.0);
        }
        Ok(self.similarity(s1, s2)? / max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::to_char_vec;

    #[test]
    fn test_prefix_matching() {
        let p = Prefix::new();
        let s1 = to_char_vec("hello");
        let s2 = to_char_vec("help");
        let pref = p.prefix(&s1, &s2);
        assert_eq!(pref, &['h', 'e', 'l']);
        assert_eq!(p.similarity(&s1, &s2).unwrap(), 3.0);
        assert_eq!(p.maximum(&s1, &s2), 5.0);
        assert_eq!(p.distance(&s1, &s2).unwrap(), 2.0);
        assert_eq!(p.normalized_similarity(&s1, &s2).unwrap(), 3.0 / 5.0);
        assert_eq!(p.normalized_distance(&s1, &s2).unwrap(), 2.0 / 5.0);
    }

    #[test]
    fn test_prefix_different() {
        let p = Prefix::new();
        let s1 = to_char_vec("cat");
        let s2 = to_char_vec("dog");
        let pref = p.prefix(&s1, &s2);
        assert_eq!(pref, &[]);
        assert_eq!(p.similarity(&s1, &s2).unwrap(), 0.0);
        assert_eq!(p.distance(&s1, &s2).unwrap(), 3.0);
        assert_eq!(p.normalized_similarity(&s1, &s2).unwrap(), 0.0);
        assert_eq!(p.normalized_distance(&s1, &s2).unwrap(), 1.0);
    }

    #[test]
    fn test_prefix_empty() {
        let p = Prefix::new();
        let s1 = to_char_vec("");
        let s2 = to_char_vec("");
        let pref = p.prefix(&s1, &s2);
        assert_eq!(pref, &[]);
        assert_eq!(p.similarity(&s1, &s2).unwrap(), 0.0);
        assert_eq!(p.maximum(&s1, &s2), 0.0);
        assert_eq!(p.distance(&s1, &s2).unwrap(), 0.0);
        assert_eq!(p.normalized_similarity(&s1, &s2).unwrap(), 1.0);
        assert_eq!(p.normalized_distance(&s1, &s2).unwrap(), 0.0);
    }
}
