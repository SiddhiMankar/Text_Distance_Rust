/// Jaro-Winkler strcmp95 variant similarity metric.
///
/// Ref: `textdistance.algorithms.edit_based.StrCmp95`
/// Standard strcmp95 algorithm (Winkler 1995 with optional long_strings adjustment).
///
/// Python reference: inherits `_BaseSimilarity`.
/// - `maximum` = 1.0
/// - `similarity` = `__call__`
/// - `distance` = `maximum - similarity` = `1.0 - similarity`
/// - `normalized_similarity` = `similarity / maximum` = `similarity`
/// - `normalized_distance` = `1.0 - normalized_similarity`
use crate::error::TextDistanceError;

const SP_MX: &[(char, char)] = &[
    ('A', 'E'),
    ('A', 'I'),
    ('A', 'O'),
    ('A', 'U'),
    ('B', 'V'),
    ('E', 'I'),
    ('E', 'O'),
    ('E', 'U'),
    ('I', 'O'),
    ('I', 'U'),
    ('O', 'U'),
    ('I', 'Y'),
    ('E', 'Y'),
    ('C', 'G'),
    ('E', 'F'),
    ('W', 'U'),
    ('W', 'V'),
    ('X', 'K'),
    ('S', 'Z'),
    ('X', 'S'),
    ('Q', 'C'),
    ('U', 'V'),
    ('M', 'N'),
    ('L', 'I'),
    ('Q', 'O'),
    ('P', 'R'),
    ('I', 'J'),
    ('2', 'Z'),
    ('5', 'S'),
    ('8', 'B'),
    ('1', 'I'),
    ('1', 'L'),
    ('0', 'O'),
    ('0', 'Q'),
    ('C', 'K'),
    ('G', 'J'),
];

#[derive(Debug, Clone, Copy, Default)]
pub struct StrCmp95 {
    pub long_strings: bool,
}

impl StrCmp95 {
    pub fn new() -> Self {
        StrCmp95::default()
    }

    pub fn with_config(long_strings: bool) -> Self {
        StrCmp95 { long_strings }
    }

    fn in_range(c: char) -> bool {
        let code = c as u32;
        code > 0 && code < 91
    }

    fn get_adjwt(c1: char, c2: char) -> Option<f64> {
        for &(a, b) in SP_MX {
            if (c1 == a && c2 == b) || (c1 == b && c2 == a) {
                return Some(3.0);
            }
        }
        None
    }

    fn is_python_whitespace(c: char) -> bool {
        c.is_whitespace() || matches!(c as u32, 0x1C..=0x1F)
    }

    fn py_trim(s: &str) -> &str {
        s.trim_matches(Self::is_python_whitespace)
    }

    pub fn compute(&self, s1: &str, s2: &str) -> f64 {
        let s1_clean: Vec<char> = Self::py_trim(s1).to_uppercase().chars().collect();
        let s2_clean: Vec<char> = Self::py_trim(s2).to_uppercase().chars().collect();

        // Quick answer checks
        if s1_clean == s2_clean {
            return 1.0;
        }
        if s1_clean.is_empty() || s2_clean.is_empty() {
            return 0.0;
        }

        let len_s1 = s1_clean.len();
        let len_s2 = s2_clean.len();

        let (search_range_initial, minv) = if len_s1 > len_s2 {
            (len_s1, len_s2)
        } else {
            (len_s2, len_s1)
        };

        let mut s1_flag = vec![0i32; search_range_initial];
        let mut s2_flag = vec![0i32; search_range_initial];

        let search_range = if search_range_initial / 2 > 0 {
            search_range_initial / 2 - 1
        } else {
            0
        };

        // Looking only within the search range, count and flag the matched pairs.
        let mut num_com = 0usize;
        let yl1 = len_s2 - 1;
        for i in 0..len_s1 {
            let sc1 = s1_clean[i];
            let lowlim = i.saturating_sub(search_range);
            let hilim = (i + search_range).min(yl1);
            for j in lowlim..=hilim {
                if s2_flag[j] == 0 && s2_clean[j] == sc1 {
                    s2_flag[j] = 1;
                    s1_flag[i] = 1;
                    num_com += 1;
                    break;
                }
            }
        }

        // If no characters in common - return
        if num_com == 0 {
            return 0.0;
        }

        // Count transpositions
        let mut k = 0usize;
        let mut n_trans = 0usize;
        for i in 0..len_s1 {
            if s1_flag[i] == 0 {
                continue;
            }
            let mut j = k;
            while j < len_s2 {
                if s2_flag[j] != 0 {
                    k = j + 1;
                    break;
                }
                j += 1;
            }
            if s1_clean[i] != s2_clean[j] {
                n_trans += 1;
            }
        }
        n_trans /= 2;

        // Adjust for similarities in unmatched characters
        let mut n_simi = 0.0;
        if minv > num_com {
            for i in 0..len_s1 {
                if s1_flag[i] != 0 {
                    continue;
                }
                if !Self::in_range(s1_clean[i]) {
                    continue;
                }
                for j in 0..len_s2 {
                    if s2_flag[j] != 0 {
                        continue;
                    }
                    if !Self::in_range(s2_clean[j]) {
                        continue;
                    }
                    if let Some(wt) = Self::get_adjwt(s1_clean[i], s2_clean[j]) {
                        n_simi += wt;
                        s2_flag[j] = 2;
                        break;
                    }
                }
            }
        }

        let num_sim = n_simi / 10.0 + num_com as f64;

        // Main weight computation
        let mut weight = (num_sim / len_s1 as f64) + (num_sim / len_s2 as f64);
        weight += (num_com as f64 - n_trans as f64) / num_com as f64;
        weight /= 3.0;

        // Continue to boost the weight if strings are similar
        if weight <= 0.7 {
            return weight;
        }

        // Adjust for up to first 4 characters in common
        let j = minv.min(4);
        let mut i = 0usize;
        for (sc1, sc2) in s1_clean.iter().zip(s2_clean.iter()) {
            if i >= j {
                break;
            }
            if sc1 != sc2 {
                break;
            }
            if sc1.is_ascii_digit() {
                break;
            }
            i += 1;
        }

        if i > 0 {
            weight += (i as f64) * 0.1 * (1.0 - weight);
        }

        // Optionally adjust for long strings
        if !self.long_strings {
            return weight;
        }
        if minv <= 4 {
            return weight;
        }
        if num_com <= i + 1 || 2 * num_com < minv + i {
            return weight;
        }
        if s1_clean[0].is_ascii_digit() {
            return weight;
        }

        let res = (num_com as f64 - i as f64 - 1.0)
            / (len_s1 as f64 + len_s2 as f64 - (i * 2) as f64 + 2.0);
        weight += (1.0 - weight) * res;
        weight
    }

    pub fn maximum_score(&self, _s1: &str, _s2: &str) -> f64 {
        1.0
    }

    pub fn similarity(&self, s1: &str, s2: &str) -> Result<f64, TextDistanceError> {
        Ok(self.compute(s1, s2))
    }

    pub fn distance(&self, s1: &str, s2: &str) -> Result<f64, TextDistanceError> {
        Ok(1.0 - self.compute(s1, s2))
    }

    pub fn normalized_similarity(&self, s1: &str, s2: &str) -> Result<f64, TextDistanceError> {
        Ok(self.compute(s1, s2))
    }

    pub fn normalized_distance(&self, s1: &str, s2: &str) -> Result<f64, TextDistanceError> {
        Ok(1.0 - self.compute(s1, s2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strcmp95_basic() {
        let s = StrCmp95::new();
        assert_eq!(s.compute("", ""), 1.0);
        assert_eq!(s.compute("", "a"), 0.0);
        assert_eq!(s.compute("a", ""), 0.0);
        assert_eq!(s.compute("a", "a"), 1.0);
        assert_eq!(s.compute("a", "b"), 0.0);
    }

    #[test]
    fn test_strcmp95_known_pairs() {
        let s = StrCmp95::new();
        let sim = s.compute("cat", "cats");
        assert!((sim - 0.9416666666666667).abs() < 1e-7);

        let sim2 = s.compute("MARTHA", "MARHTA");
        assert!((sim2 - 0.9611111111111111).abs() < 1e-7);
    }

    #[test]
    fn test_strcmp95_long_strings() {
        let sl = StrCmp95::with_config(true);
        let sim = sl.compute("shackleford", "shackelford");
        assert!((sim - 0.9886363636363636).abs() < 1e-7);
    }
}
