use std::collections::HashMap;
use std::hash::Hash;

use crate::error::TextDistanceError;
use crate::traits::SimilarityMetric;

#[derive(Debug, Clone)]
pub struct Matrix<T> {
    pub mat: Option<HashMap<(T, T), f64>>,
    pub match_cost: f64,
    pub mismatch_cost: f64,
    pub symmetric: bool,
}

impl<T> Default for Matrix<T> {
    fn default() -> Self {
        Matrix {
            mat: None,
            match_cost: 1.0,
            mismatch_cost: 0.0,
            symmetric: true,
        }
    }
}

impl<T> Matrix<T> {
    pub fn new() -> Self {
        Matrix::default()
    }

    pub fn with_config(
        mat: Option<HashMap<(T, T), f64>>,
        match_cost: f64,
        mismatch_cost: f64,
        symmetric: bool,
    ) -> Self {
        Matrix {
            mat,
            match_cost,
            mismatch_cost,
            symmetric,
        }
    }
}

impl<T: PartialEq + Eq + Hash + Clone> SimilarityMetric<T> for Matrix<T> {
    fn similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        let is_ident = s1 == s2;

        if let Some(ref map) = self.mat {
            if map.is_empty() {
                if is_ident {
                    return Ok(self.match_cost);
                } else {
                    return Ok(self.mismatch_cost);
                }
            }

            if s1.len() == 1 && s2.len() == 1 {
                let pair = (s1[0].clone(), s2[0].clone());
                if let Some(&val) = map.get(&pair) {
                    return Ok(val);
                }
                if self.symmetric {
                    let rev_pair = (s2[0].clone(), s1[0].clone());
                    if let Some(&val) = map.get(&rev_pair) {
                        return Ok(val);
                    }
                }
            }
        }

        if is_ident {
            Ok(self.match_cost)
        } else {
            Ok(self.mismatch_cost)
        }
    }

    fn maximum(&self, _s1: &[T], _s2: &[T]) -> f64 {
        self.match_cost
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
    fn test_matrix_default_same() {
        let m = Matrix::<char>::new();
        let s1 = to_char_vec("a");
        let s2 = to_char_vec("a");
        assert_eq!(m.similarity(&s1, &s2).unwrap(), 1.0);
        assert_eq!(m.distance(&s1, &s2).unwrap(), 0.0);
        assert_eq!(m.normalized_similarity(&s1, &s2).unwrap(), 1.0);
        assert_eq!(m.normalized_distance(&s1, &s2).unwrap(), 0.0);
    }

    #[test]
    fn test_matrix_default_different() {
        let m = Matrix::<char>::new();
        let s1 = to_char_vec("a");
        let s2 = to_char_vec("b");
        assert_eq!(m.similarity(&s1, &s2).unwrap(), 0.0);
        assert_eq!(m.distance(&s1, &s2).unwrap(), 1.0);
        assert_eq!(m.normalized_similarity(&s1, &s2).unwrap(), 0.0);
        assert_eq!(m.normalized_distance(&s1, &s2).unwrap(), 1.0);
    }

    #[test]
    fn test_matrix_empty() {
        let m = Matrix::<char>::new();
        let s1 = to_char_vec("");
        let s2 = to_char_vec("");
        assert_eq!(m.similarity(&s1, &s2).unwrap(), 1.0);
        assert_eq!(m.distance(&s1, &s2).unwrap(), 0.0);
        assert_eq!(m.normalized_similarity(&s1, &s2).unwrap(), 1.0);
        assert_eq!(m.normalized_distance(&s1, &s2).unwrap(), 0.0);
    }

    #[test]
    fn test_matrix_custom_map() {
        let mut map = HashMap::new();
        map.insert(('a', 'b'), 0.8);

        let m = Matrix::with_config(Some(map), 1.0, 0.0, true);
        let s1 = to_char_vec("a");
        let s2 = to_char_vec("b");
        assert_eq!(m.similarity(&s1, &s2).unwrap(), 0.8);

        let s3 = to_char_vec("b");
        let s4 = to_char_vec("a");
        assert_eq!(m.similarity(&s3, &s4).unwrap(), 0.8);
    }
}
