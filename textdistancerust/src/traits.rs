use crate::error::TextDistanceError;

pub trait DistanceMetric<T> {
    fn distance(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError>;
    fn maximum(&self, s1: &[T], s2: &[T]) -> f64;

    fn similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        Ok(self.maximum(s1, s2) - self.distance(s1, s2)?)
    }

    fn normalized_distance(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        let max = self.maximum(s1, s2);
        if max == 0.0 {
            return Ok(0.0);
        }
        Ok(self.distance(s1, s2)? / max)
    }

    fn normalized_similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        Ok(1.0 - self.normalized_distance(s1, s2)?)
    }
}

pub trait SimilarityMetric<T> {
    fn similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError>;
    fn maximum(&self, s1: &[T], s2: &[T]) -> f64;

    fn distance(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        Ok(self.maximum(s1, s2) - self.similarity(s1, s2)?)
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
