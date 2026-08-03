use crate::traits::DistanceMetric;
use crate::error::TextDistanceError;

#[derive(Default)]
pub struct Hamming;

impl Hamming {
    pub fn new() -> Self {
        Hamming
    }
}

impl<T: PartialEq> DistanceMetric<T> for Hamming {
    fn distance(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        let max_len = s1.len().max(s2.len());
        let mut matches = 0;
        let min_len = s1.len().min(s2.len());
        
        for i in 0..min_len {
            if s1[i] == s2[i] {
                matches += 1;
            }
        }
        
        Ok((max_len - matches) as f64)
    }

    fn maximum(&self, s1: &[T], s2: &[T]) -> f64 {
        s1.len().max(s2.len()) as f64
    }
}
