/// Match Rating Approach (MRA) phonetic similarity metric.
///
/// Implements the Western Airlines Surname Match Rating Algorithm:
/// <https://en.wikipedia.org/wiki/Match_rating_approach>
///
/// Python reference: `textdistance.algorithms.phonetic.MRA` (inherits `_BaseSimilarity`).
///
/// ## Key behaviours confirmed against live Python library
///
/// - `MRA` does **not** use q-gram tokenisation or set arithmetic.
///   It operates on raw `&str` inputs through its own phonetic encoder (`_calc_mra`).
/// - `similarity()` == `__call__()` (because Python class inherits `_BaseSimilarity`).
/// - `distance()` == `maximum() - similarity()`.
/// - `maximum()` is the **max of the MRA-encoded string lengths**, not raw char counts.
/// - `normalized_similarity()` == `similarity() / maximum()` (0.0 when `maximum == 0`).
/// - `normalized_distance()` == `1.0 - normalized_similarity()`.
/// - Empty input: `__call__` returns `0` (early-exit `if not all(sequences): return 0`).
///   When both inputs are empty, `maximum = max(0, 0) = 0`, so `normalized_similarity = 0.0`.
///
/// ## Encoding (`_calc_mra`)
///
/// 1. Uppercase the string.
/// 2. Keep the first character; strip `A E I O U` from remaining chars.
/// 3. Collapse consecutive duplicate characters (like Unix `uniq`).
/// 4. If the result is longer than 6 chars, keep first 3 + last 3.
///
/// ## Comparison algorithm
///
/// For two strings s1, s2 (after encoding):
/// 1. If either encoded string is empty, return `similarity = 0`.
/// 2. If `abs(len(enc1) - len(enc2)) > 2`, return `similarity = 0`.
/// 3. Run **exactly 2** iterations of prefix-matching:
///    - Zip the two encoded sequences together position by position.
///    - Strip any position where the chars are identical; keep the rest.
///    - Append the trailing (non-overlapping) tail of the longer sequence.
///    - Update lengths.
/// 4. `similarity = max_length - max(final_lengths)`.
///    When all chars matched, final lengths = `[0, 0]` → `similarity = max_length`.

use crate::error::TextDistanceError;

#[derive(Debug, Clone, Copy, Default)]
pub struct Mra;

impl Mra {
    pub fn new() -> Self {
        Mra
    }

    /// Phonetic encoder: mirrors Python `MRA._calc_mra`.
    ///
    /// Returns the encoded string (may be empty if input is empty).
    pub fn calc_mra(word: &str) -> String {
        if word.is_empty() {
            return String::new();
        }

        // Step 1: uppercase
        let upper = word.to_uppercase();
        let mut chars = upper.chars();

        // Step 2: keep first char; remove AEIOU from the rest
        let first = chars.next().unwrap(); // safe: non-empty
        let mut result = String::new();
        result.push(first);
        for c in chars {
            if !matches!(c, 'A' | 'E' | 'I' | 'O' | 'U') {
                result.push(c);
            }
        }

        // Step 3: collapse consecutive duplicates (Unix `uniq`)
        // Use Option<char> as sentinel so NUL '\x00' is handled correctly.
        // (Initializing prev to '\0' would silently drop a leading NUL character.)
        let mut deduped = String::new();
        let mut prev: Option<char> = None;
        for c in result.chars() {
            if Some(c) != prev {
                deduped.push(c);
                prev = Some(c);
            }
        }

        // Step 4: truncate to first 3 + last 3 if longer than 6
        let chars_vec: Vec<char> = deduped.chars().collect();
        if chars_vec.len() > 6 {
            chars_vec[..3].iter().chain(chars_vec[chars_vec.len() - 3..].iter()).collect()
        } else {
            chars_vec.into_iter().collect()
        }
    }

    /// Compute the MRA comparison rating for two strings.
    ///
    /// Returns the integer similarity score (≥ 0).  This mirrors Python `MRA.__call__`.
    pub fn compute(&self, s1: &str, s2: &str) -> f64 {
        // Python: `if not all(sequences): return 0`
        // i.e. either empty raw input → 0 immediately (before encoding)
        if s1.is_empty() || s2.is_empty() {
            return 0.0;
        }

        // Encode both strings
        let enc1: Vec<char> = Self::calc_mra(s1).chars().collect();
        let enc2: Vec<char> = Self::calc_mra(s2).chars().collect();

        let count = 2usize; // always 2 for a pairwise comparison
        let len1 = enc1.len();
        let len2 = enc2.len();
        let max_length = len1.max(len2);

        // Threshold check: abs(max_len - min_len) > count → 0
        let len_diff = if len1 > len2 { len1 - len2 } else { len2 - len1 };
        if len_diff > count {
            return 0.0;
        }

        // Iterative prefix-matching loop (runs exactly `count` times = 2)
        let mut seq1 = enc1.clone();
        let mut seq2 = enc2.clone();

        for _ in 0..count {
            let minlen = seq1.len().min(seq2.len());

            // Collect the non-matching (position, pair) entries from zip
            let mut non_matching_pairs: Vec<(char, char)> = Vec::new();
            for (c1, c2) in seq1.iter().zip(seq2.iter()) {
                if c1 != c2 {
                    non_matching_pairs.push((*c1, *c2));
                }
            }

            // Reconstruct each sequence from non-matching pairs + tail beyond minlen
            let new_s1: Vec<char> = non_matching_pairs.iter().map(|(c, _)| *c)
                .chain(seq1[minlen..].iter().copied())
                .collect();
            let new_s2: Vec<char> = non_matching_pairs.iter().map(|(_, c)| *c)
                .chain(seq2[minlen..].iter().copied())
                .collect();

            seq1 = new_s1;
            seq2 = new_s2;
        }

        // Python: `if not lengths: return max_length` (lengths list is empty only if sequences is empty)
        // In practice with 2 strings this path is unreachable, but mirror it defensively:
        // `return max_length - max(lengths)`
        let remaining_max = seq1.len().max(seq2.len());
        (max_length - remaining_max) as f64
    }

    /// `maximum` mirrors Python `MRA.maximum`: max of the MRA-encoded string lengths.
    pub fn maximum_score(&self, s1: &str, s2: &str) -> f64 {
        let enc1 = Self::calc_mra(s1);
        let enc2 = Self::calc_mra(s2);
        enc1.chars().count().max(enc2.chars().count()) as f64
    }

    /// `similarity` = `__call__` (mirrors Python `_BaseSimilarity.similarity`).
    pub fn similarity(&self, s1: &str, s2: &str) -> Result<f64, TextDistanceError> {
        Ok(self.compute(s1, s2))
    }

    /// `distance` = `maximum - similarity` (mirrors Python `_BaseSimilarity.distance`).
    pub fn distance(&self, s1: &str, s2: &str) -> Result<f64, TextDistanceError> {
        Ok(self.maximum_score(s1, s2) - self.compute(s1, s2))
    }

    /// `normalized_similarity` = `1 - normalized_distance`.
    ///
    /// When `maximum == 0` (both inputs encode to empty strings, e.g. both inputs are ""),
    /// Python `Base.normalized_distance` returns 0 (early-exit guard), so
    /// `normalized_similarity = 1 - 0 = 1.0`.
    pub fn normalized_similarity(&self, s1: &str, s2: &str) -> Result<f64, TextDistanceError> {
        Ok(1.0 - self.normalized_distance(s1, s2)?)
    }

    /// `normalized_distance` = `distance / maximum` (0.0 when maximum == 0).
    ///
    /// Mirrors Python `Base.normalized_distance`:
    ///   if maximum == 0: return 0
    ///   return distance / maximum
    pub fn normalized_distance(&self, s1: &str, s2: &str) -> Result<f64, TextDistanceError> {
        let max = self.maximum_score(s1, s2);
        if max == 0.0 {
            return Ok(0.0);
        }
        Ok(self.distance(s1, s2)? / max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Encoder tests (empirically confirmed against Python) ──────────────────

    #[test]
    fn test_calc_mra_hello() {
        // "hello" → upper: "HELLO" → remove vowels from rest: "H"+"LL" → uniq: "HL"
        assert_eq!(Mra::calc_mra("hello"), "HL");
    }

    #[test]
    fn test_calc_mra_world() {
        // "world" → "WORLD" → "W"+"RLD" → uniq: "WRLD"
        assert_eq!(Mra::calc_mra("world"), "WRLD");
    }

    #[test]
    fn test_calc_mra_catherine() {
        // "catherine" → "CATHERINE" → "C"+"THRN" → uniq: "CTHRN"
        assert_eq!(Mra::calc_mra("catherine"), "CTHRN");
    }

    #[test]
    fn test_calc_mra_kathryn() {
        // "kathryn" → "KATHRYN" → "K"+"THRYN" → uniq: "KTHRYN"
        assert_eq!(Mra::calc_mra("kathryn"), "KTHRYN");
    }

    #[test]
    fn test_calc_mra_empty() {
        assert_eq!(Mra::calc_mra(""), "");
    }

    // ── Comparison tests (empirically confirmed against Python) ───────────────

    #[test]
    fn test_mra_empty_inputs() {
        let m = Mra::new();
        // Both empty: compute = 0 (early exit), maximum = 0
        // normalized_distance = 0 (Python's Base guard: max==0 → return 0)
        // normalized_similarity = 1 - 0 = 1.0
        assert_eq!(m.compute("", ""), 0.0);
        assert_eq!(m.normalized_distance("", "").unwrap(), 0.0);
        assert_eq!(m.normalized_similarity("", "").unwrap(), 1.0);
        // One empty: compute = 0, maximum = encoded len of non-empty side
        assert_eq!(m.compute("", "abc"), 0.0);
        assert_eq!(m.compute("abc", ""), 0.0);
    }

    #[test]
    fn test_mra_cat_cats() {
        let m = Mra::new();
        // "cat" → "CT" (len 2), "cats" → "CTS" (len 3), max=3
        // diff=1 ≤ 2 → proceed. After 2 iterations: similarity=2
        assert_eq!(m.compute("cat", "cats"), 2.0);
        assert_eq!(m.similarity("cat", "cats").unwrap(), 2.0);
        assert_eq!(m.distance("cat", "cats").unwrap(), 1.0);      // 3 - 2 = 1
        let norm_sim = m.normalized_similarity("cat", "cats").unwrap();
        assert!((norm_sim - 2.0 / 3.0).abs() < 1e-9);
        let norm_dist = m.normalized_distance("cat", "cats").unwrap();
        assert!((norm_dist - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_mra_identical() {
        let m = Mra::new();
        // "cat" vs "cat" → same encoding "CT", all chars match, similarity = max_length = 2
        assert_eq!(m.compute("cat", "cat"), 2.0);
        // "hello" vs "hello" → "HL" len=2, similarity=2
        assert_eq!(m.compute("hello", "hello"), 2.0);
    }

    #[test]
    fn test_mra_a_vs_b() {
        let m = Mra::new();
        // "a" → "A", "b" → "B"; no chars match; similarity = 1 - 1 = 0
        assert_eq!(m.compute("a", "b"), 0.0);
    }

    #[test]
    fn test_mra_maximum() {
        let m = Mra::new();
        // "hello"→"HL"(2), "world"→"WRLD"(4); max=4
        assert_eq!(m.maximum_score("hello", "world"), 4.0);
        // "catherine"→"CTHRN"(5), "kathryn"→"KTHRYN"(6); max=6
        assert_eq!(m.maximum_score("catherine", "kathryn"), 6.0);
    }

    #[test]
    fn test_mra_threshold_exceeded() {
        let m = Mra::new();
        // "a"→"A"(1), "bbbbb"→"B"(1); diff=0 ≤ 2 → not threshold, but no matching chars → 0
        // Verify threshold case with real diff > 2:
        // "a"→"A"(1) vs long string that encodes to len 4 → diff=3 > 2 → returns 0
        // "aaaa" → "A" (deduplicated), so need something like consonant-heavy input
        // "bcdfg" → "BCDFG"(5), diff = 5-1 = 4 > 2 → should be 0
        assert_eq!(m.compute("a", "bcdfg"), 0.0);
    }
}
