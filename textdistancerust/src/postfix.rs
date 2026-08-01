use crate::error::TextDistanceError;
use crate::traits::SimilarityMetric;

#[derive(Debug, Clone, Copy)]
pub struct Postfix {
    pub qval: usize,
}

impl Default for Postfix {
    fn default() -> Self {
        Postfix { qval: 1 }
    }
}

impl Postfix {
    pub fn new() -> Self {
        Postfix::default()
    }

    pub fn with_qval(qval: usize) -> Self {
        Postfix { qval }
    }

    pub fn postfix<'a, T: PartialEq>(&self, s1: &'a [T], s2: &'a [T]) -> &'a [T] {
        let len = s1
            .iter()
            .rev()
            .zip(s2.iter().rev())
            .take_while(|(a, b)| a == b)
            .count();
        &s1[s1.len() - len..]
    }
}

impl<T: PartialEq> SimilarityMetric<T> for Postfix {
    fn similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        Ok(self.postfix(s1, s2).len() as f64)
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
    fn test_postfix_matching() {
        let p = Postfix::new();
        let s1 = to_char_vec("testing");
        let s2 = to_char_vec("resting");
        let post = p.postfix(&s1, &s2);
        assert_eq!(post, &['e', 's', 't', 'i', 'n', 'g']);
        assert_eq!(p.similarity(&s1, &s2).unwrap(), 6.0);
        assert_eq!(p.maximum(&s1, &s2), 7.0);
        assert_eq!(p.distance(&s1, &s2).unwrap(), 1.0);
        assert_eq!(p.normalized_similarity(&s1, &s2).unwrap(), 6.0 / 7.0);
        assert_eq!(p.normalized_distance(&s1, &s2).unwrap(), 1.0 / 7.0);
    }

    #[test]
    fn test_postfix_different() {
        let p = Postfix::new();
        let s1 = to_char_vec("cat");
        let s2 = to_char_vec("dog");
        let post = p.postfix(&s1, &s2);
        assert_eq!(post, &[]);
        assert_eq!(p.similarity(&s1, &s2).unwrap(), 0.0);
        assert_eq!(p.distance(&s1, &s2).unwrap(), 3.0);
        assert_eq!(p.normalized_similarity(&s1, &s2).unwrap(), 0.0);
        assert_eq!(p.normalized_distance(&s1, &s2).unwrap(), 1.0);
    }

    #[test]
    fn test_postfix_empty() {
        let p = Postfix::new();
        let s1 = to_char_vec("");
        let s2 = to_char_vec("");
        let post = p.postfix(&s1, &s2);
        assert_eq!(post, &[]);
        assert_eq!(p.similarity(&s1, &s2).unwrap(), 0.0);
        assert_eq!(p.maximum(&s1, &s2), 0.0);
        assert_eq!(p.distance(&s1, &s2).unwrap(), 0.0);
        assert_eq!(p.normalized_similarity(&s1, &s2).unwrap(), 1.0);
        assert_eq!(p.normalized_distance(&s1, &s2).unwrap(), 0.0);
    }
}
