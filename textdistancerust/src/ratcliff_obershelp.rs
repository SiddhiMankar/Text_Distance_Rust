use crate::traits::SimilarityMetric;
use crate::error::TextDistanceError;

pub struct RatcliffObershelp;

impl Default for RatcliffObershelp {
    fn default() -> Self {
        RatcliffObershelp
    }
}

impl RatcliffObershelp {
    pub fn new() -> Self {
        RatcliffObershelp::default()
    }
}

fn lcsstr_string<T: PartialEq + Clone>(s1: &[T], s2: &[T]) -> Vec<T> {
    if s1.is_empty() || s2.is_empty() {
        return vec![];
    }

    let rows = s1.len() + 1;
    let cols = s2.len() + 1;

    let mut dist_prev = vec![0; cols];
    let mut dist_cur = vec![0; cols];
    let mut max_len = 0;
    let mut best_i = 0;
    let mut best_j = 0;

    for i in 1..rows {
        for j in 1..cols {
            if s1[i - 1] == s2[j - 1] {
                let current_len = dist_prev[j - 1] + 1;
                dist_cur[j] = current_len;
                
                let start_i = i - current_len;
                let start_j = j - current_len;
                
                let is_better = if current_len > max_len {
                    true
                } else if current_len == max_len && current_len > 0 {
                    let best_start_i = best_i - max_len;
                    let best_start_j = best_j - max_len;
                    if start_i < best_start_i {
                        true
                    } else if start_i == best_start_i && start_j < best_start_j {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                
                if is_better {
                    max_len = current_len;
                    best_i = i;
                    best_j = j;
                }
            } else {
                dist_cur[j] = 0;
            }
        }
        dist_prev.copy_from_slice(&dist_cur);
    }

    if max_len == 0 {
        vec![]
    } else {
        s1[(best_i - max_len)..best_i].to_vec()
    }
}

fn find_subseq_index<T: PartialEq>(s: &[T], subseq: &[T]) -> Option<usize> {
    if subseq.is_empty() {
        return Some(0);
    }
    if s.len() < subseq.len() {
        return None;
    }
    for i in 0..=(s.len() - subseq.len()) {
        if &s[i..(i + subseq.len())] == subseq {
            return Some(i);
        }
    }
    None
}

fn ratcliff_obershelp_find<T: PartialEq + Clone>(s1: &[T], s2: &[T]) -> usize {
    let subseq = lcsstr_string(s1, s2);
    let length = subseq.len();
    if length == 0 {
        return 0;
    }
    
    let pos1 = find_subseq_index(s1, &subseq).unwrap();
    let pos2 = find_subseq_index(s2, &subseq).unwrap();
    
    let before1 = &s1[..pos1];
    let before2 = &s2[..pos2];
    
    let after1 = &s1[(pos1 + length)..];
    let after2 = &s2[(pos2 + length)..];
    
    ratcliff_obershelp_find(before1, before2) + length + ratcliff_obershelp_find(after1, after2)
}

impl<T: PartialEq + Clone> SimilarityMetric<T> for RatcliffObershelp {
    fn similarity(&self, s1: &[T], s2: &[T]) -> Result<f64, TextDistanceError> {
        let scount = 2.0;
        let ecount = (s1.len() + s2.len()) as f64;
        
        if ecount == 0.0 {
            return Ok(1.0);
        }
        
        let matches = ratcliff_obershelp_find(s1, s2) as f64;
        Ok(scount * matches / ecount)
    }

    fn maximum(&self, _s1: &[T], _s2: &[T]) -> f64 {
        1.0
    }
}
