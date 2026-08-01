use crate::error::TextDistanceError;
use crate::traits::DistanceMetric;

#[derive(Debug, Clone, Copy, Default)]
pub struct Length;

impl Length {
    pub fn new() -> Self {
        Length
    }
}

impl<T> DistanceMetric<T> for Length {
    fn distance(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        let l1 = s1.len() as f64;
        let l2 = s2.len() as f64;
        Ok((l1 - l2).abs())
    }

    fn maximum(&self, s1: &[T], s2: &[T]) -> f64 {
        (s1.len() as f64).max(s2.len() as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::to_char_vec;

    #[test]
    fn test_length_same() {
        let len_metric = Length::new();
        let s1 = to_char_vec("hello");
        let s2 = to_char_vec("world");
        assert_eq!(len_metric.distance(&s1, &s2).unwrap(), 0.0);
        assert_eq!(len_metric.maximum(&s1, &s2), 5.0);
        assert_eq!(len_metric.normalized_distance(&s1, &s2).unwrap(), 0.0);
        assert_eq!(len_metric.normalized_similarity(&s1, &s2).unwrap(), 1.0);
    }

    #[test]
    fn test_length_different() {
        let len_metric = Length::new();
        let s1 = to_char_vec("cat");
        let s2 = to_char_vec("elephant");
        assert_eq!(len_metric.distance(&s1, &s2).unwrap(), 5.0);
        assert_eq!(len_metric.maximum(&s1, &s2), 8.0);
        assert_eq!(len_metric.normalized_distance(&s1, &s2).unwrap(), 5.0 / 8.0);
        assert_eq!(len_metric.normalized_similarity(&s1, &s2).unwrap(), 3.0 / 8.0);
    }

    #[test]
    fn test_length_empty() {
        let len_metric = Length::new();
        let s1 = to_char_vec("");
        let s2 = to_char_vec("");
        assert_eq!(len_metric.distance(&s1, &s2).unwrap(), 0.0);
        assert_eq!(len_metric.maximum(&s1, &s2), 0.0);
        assert_eq!(len_metric.normalized_distance(&s1, &s2).unwrap(), 0.0);
        assert_eq!(len_metric.normalized_similarity(&s1, &s2).unwrap(), 1.0);
    }
}
