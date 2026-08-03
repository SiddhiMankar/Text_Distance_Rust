use crate::traits::SimilarityMetric;
use crate::error::TextDistanceError;

#[derive(Default)]
pub struct JaroWinkler {
    pub long_tolerance: bool,
    pub winklerize: bool,
    pub prefix_weight: f64,
}

impl JaroWinkler {
    pub fn new() -> Self {
        JaroWinkler {
            long_tolerance: false,
            winklerize: true,
            prefix_weight: 0.1,
        }
    }

    pub fn jaro() -> Self {
        JaroWinkler {
            long_tolerance: false,
            winklerize: false,
            prefix_weight: 0.1,
        }
    }
}

impl<T: PartialEq> SimilarityMetric<T> for JaroWinkler {
    fn similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        if s1 == s2 {
            return Ok(1.0);
        }

        let s1_len = s1.len();
        let s2_len = s2.len();

        if s1_len == 0 || s2_len == 0 {
            return Ok(0.0);
        }

        let min_len = s1_len.min(s2_len);
        let max_len = s1_len.max(s2_len);
        let search_range = if max_len == 0 { 0 } else { (max_len / 2).saturating_sub(1) };

        let mut s1_flags = vec![false; s1_len];
        let mut s2_flags = vec![false; s2_len];

        let mut common_chars = 0;
        for (i, s1_ch) in s1.iter().enumerate() {
            let low = i.saturating_sub(search_range);
            let hi = (i + search_range).min(s2_len.saturating_sub(1));
            
            for j in low..=hi {
                if !s2_flags[j] && s2[j] == *s1_ch {
                    s1_flags[i] = true;
                    s2_flags[j] = true;
                    common_chars += 1;
                    break;
                }
            }
        }

        if common_chars == 0 {
            return Ok(0.0);
        }

        let mut k = 0;
        let mut trans_count = 0;
        for (i, s1_f) in s1_flags.iter().enumerate() {
            if *s1_f {
                let mut j = k;
                while j < s2_len {
                    if s2_flags[j] {
                        k = j + 1;
                        break;
                    }
                    j += 1;
                }
                if s1[i] != s2[j] {
                    trans_count += 1;
                }
            }
        }
        trans_count /= 2;

        let common = common_chars as f64;
        let mut weight = (common / s1_len as f64 + common / s2_len as f64 + (common - trans_count as f64) / common) / 3.0;

        if !self.winklerize || weight <= 0.7 {
            return Ok(weight);
        }

        let max_prefix = min_len.min(4);
        let mut p = 0;
        while p < max_prefix && s1[p] == s2[p] {
            p += 1;
        }

        if p > 0 {
            weight += p as f64 * self.prefix_weight * (1.0 - weight);
        }

        if !self.long_tolerance || min_len <= 4 {
            return Ok(weight);
        }

        if common_chars <= p + 1 || 2 * common_chars < min_len + p {
            return Ok(weight);
        }

        let tmp = (common_chars as f64 - p as f64 - 1.0) / (s1_len as f64 + s2_len as f64 - (p * 2) as f64 + 2.0);
        weight += (1.0 - weight) * tmp;

        Ok(weight)
    }

    fn maximum(&self, _s1: &[T], _s2: &[T]) -> f64 {
        1.0
    }
}

pub struct Jaro;
impl Jaro {
    pub fn new() -> JaroWinkler {
        JaroWinkler::jaro()
    }
}
