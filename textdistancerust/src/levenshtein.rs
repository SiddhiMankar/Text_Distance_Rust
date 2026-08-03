use crate::traits::DistanceMetric;
use crate::error::TextDistanceError;

#[derive(Default)]
pub struct Levenshtein;

impl Levenshtein {
    pub fn new() -> Self {
        Levenshtein
    }
}

impl<T: PartialEq> DistanceMetric<T> for Levenshtein {
    fn distance(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        let rows = s1.len() + 1;
        let cols = s2.len() + 1;
        
        if s1.is_empty() {
            return Ok(s2.len() as f64);
        }
        if s2.is_empty() {
            return Ok(s1.len() as f64);
        }

        let mut cur: Vec<usize> = (0..cols).collect();
        let mut prev = vec![0; cols];

        for r in 1..rows {
            std::mem::swap(&mut prev, &mut cur);
            cur[0] = r;
            for c in 1..cols {
                let deletion = prev[c] + 1;
                let insertion = cur[c - 1] + 1;
                let cost = if s1[r - 1] == s2[c - 1] { 0 } else { 1 };
                let edit = prev[c - 1] + cost;
                
                cur[c] = deletion.min(insertion).min(edit);
            }
        }
        
        Ok(cur[cols - 1] as f64)
    }

    fn maximum(&self, s1: &[T], s2: &[T]) -> f64 {
        s1.len().max(s2.len()) as f64
    }
}
