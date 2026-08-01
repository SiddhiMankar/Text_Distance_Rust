use crate::error::TextDistanceError;
use crate::traits::SimilarityMetric;

#[derive(Debug, Clone, Copy, Default)]
pub struct Identity;

impl Identity {
    pub fn new() -> Self {
        Identity
    }
}

impl<T: PartialEq> SimilarityMetric<T> for Identity {
    fn similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        if s1 == s2 {
            Ok(1.0)
        } else {
            Ok(0.0)
        }
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
    fn test_identity_same() {
        let ident = Identity::new();
        let s1 = to_char_vec("hello");
        let s2 = to_char_vec("hello");
        assert_eq!(ident.similarity(&s1, &s2).unwrap(), 1.0);
        assert_eq!(ident.distance(&s1, &s2).unwrap(), 0.0);
        assert_eq!(ident.normalized_similarity(&s1, &s2).unwrap(), 1.0);
        assert_eq!(ident.normalized_distance(&s1, &s2).unwrap(), 0.0);
    }

    #[test]
    fn test_identity_different() {
        let ident = Identity::new();
        let s1 = to_char_vec("hello");
        let s2 = to_char_vec("world");
        assert_eq!(ident.similarity(&s1, &s2).unwrap(), 0.0);
        assert_eq!(ident.distance(&s1, &s2).unwrap(), 1.0);
        assert_eq!(ident.normalized_similarity(&s1, &s2).unwrap(), 0.0);
        assert_eq!(ident.normalized_distance(&s1, &s2).unwrap(), 1.0);
    }

    #[test]
    fn test_identity_empty() {
        let ident = Identity::new();
        let s1 = to_char_vec("");
        let s2 = to_char_vec("");
        assert_eq!(ident.similarity(&s1, &s2).unwrap(), 1.0);
        assert_eq!(ident.distance(&s1, &s2).unwrap(), 0.0);
        assert_eq!(ident.normalized_similarity(&s1, &s2).unwrap(), 1.0);
        assert_eq!(ident.normalized_distance(&s1, &s2).unwrap(), 0.0);
    }
}
