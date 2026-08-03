// src/arith_ncd.rs
//! Arithmetic coding based Normalized Compression Distance.
//! Mirrors Python textdistance ArithNCD exactly.

use crate::error::TextDistanceError;
use crate::ncd_base::NcdBase;
use crate::traits::SimilarityMetric;
use num::bigint::BigUint;
use num::rational::Ratio;
use num::traits::{One, ToPrimitive, Zero};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Default)]
pub struct ArithNcd {
    base: u32,
    terminator: Option<char>,
}

impl ArithNcd {
    pub fn new() -> Self {
        ArithNcd {
            base: 2,
            terminator: None,
        }
    }

    /// Build probability table from a data string.
    /// Maps each char to (cumulative_start_fraction, width_fraction).
    ///
    /// CRITICAL: Must replicate Python's Counter.most_common() ordering:
    /// - Sort by count descending
    /// - For equal counts, preserve insertion order (order of first appearance in data)
    fn make_probs(&self, data: &[char]) -> HashMap<char, (Ratio<BigUint>, Ratio<BigUint>)> {
        // Track counts AND insertion order (first appearance index)
        let mut counter: HashMap<char, usize> = HashMap::new();
        let mut insertion_order: Vec<char> = Vec::new();
        for &c in data {
            if !counter.contains_key(&c) {
                insertion_order.push(c);
            }
            *counter.entry(c).or_insert(0) += 1;
        }
        if let Some(t) = self.terminator {
            if !counter.contains_key(&t) {
                insertion_order.push(t);
            }
            counter.insert(t, 1);
        }
        let total: usize = counter.values().sum();

        // Sort by decreasing count, preserving insertion order for ties
        // (matching Python Counter.most_common() behavior)
        let mut items: Vec<(char, usize, usize)> = insertion_order
            .iter()
            .enumerate()
            .map(|(idx, &ch)| (ch, counter[&ch], idx))
            .collect();
        items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.2.cmp(&b.2)));

        let mut probs: HashMap<char, (Ratio<BigUint>, Ratio<BigUint>)> = HashMap::new();
        let mut cumulative = BigUint::zero();
        let total_big = BigUint::from(total as u64);
        for (ch, cnt, _) in items {
            let start = Ratio::new(cumulative.clone(), total_big.clone());
            let width = Ratio::new(BigUint::from(cnt as u64), total_big.clone());
            probs.insert(ch, (start, width));
            cumulative += BigUint::from(cnt as u64);
        }
        probs
    }

    /// Compute the arithmetic coding range [start, end) for the data.
    fn get_range(
        &self,
        data: &[char],
        probs: &HashMap<char, (Ratio<BigUint>, Ratio<BigUint>)>,
    ) -> (Ratio<BigUint>, Ratio<BigUint>) {
        let mut d = data.to_vec();
        if let Some(t) = self.terminator {
            d.retain(|&c| c != t);
            d.push(t);
        }
        let mut start: Ratio<BigUint> = Ratio::zero();
        let mut width: Ratio<BigUint> = Ratio::one();
        for &c in &d {
            let (prob_start, prob_width) = probs.get(&c).expect("character not in probs");
            start += prob_start.clone() * width.clone();
            width *= prob_width.clone();
        }
        let end = start.clone() + width;
        (start, end)
    }

    /// Perform arithmetic compression and return a fraction representing the compressed value.
    /// Matches Python's _compress exactly.
    fn compress_fraction(&self, data: &[char]) -> Ratio<BigUint> {
        if data.is_empty() {
            return Ratio::zero();
        }
        let probs = self.make_probs(data);
        let (start, end) = self.get_range(data, &probs);

        let mut output_fraction: Ratio<BigUint> = Ratio::zero();
        let mut output_denominator = BigUint::one();

        while !(start <= output_fraction && output_fraction < end) {
            let output_numerator =
                BigUint::one() + (start.numer() * &output_denominator) / start.denom();
            output_fraction = Ratio::new(output_numerator, output_denominator.clone());
            output_denominator *= BigUint::from(2u32);
        }
        output_fraction
    }

    /// Get the "size" of compressed data = ceil(log_base(numerator)).
    /// Matches Python's math.ceil(math.log(numerator, self.base)) exactly by
    /// replicating CPython's frexp-based log computation:
    /// `((bits + log2(mantissa)) * ln(2)) / ln(base)`.
    fn get_size(&self, data: &[char]) -> usize {
        let fraction = self.compress_fraction(data);
        let numerator = fraction.numer().clone();
        if numerator.is_zero() {
            return 0;
        }
        let bits = numerator.bits(); // = floor(log2(n)) + 1
        let mantissa = if bits <= 53 {
            let top_f = numerator.to_f64().unwrap();
            top_f / (2.0f64.powi(bits as i32))
        } else {
            let shift = bits - 53;
            let top = (&numerator >> shift).to_f64().unwrap();
            top / (2.0f64.powi(53))
        };
        let log_x = (bits as f64 + mantissa.log2()) * std::f64::consts::LN_2;
        let log_base = log_x / (self.base as f64).ln();
        log_base.ceil() as usize
    }
}

impl SimilarityMetric<char> for ArithNcd {
    fn similarity(&self, s1: &[char], s2: &[char]) -> Result<f64, TextDistanceError> {
        Ok(1.0 - self.distance(s1, s2)?)
    }

    fn distance(&self, s1: &[char], s2: &[char]) -> Result<f64, TextDistanceError> {
        let size1 = self.get_size(s1);
        let size2 = self.get_size(s2);
        // Concatenate raw data, then compress the concatenation.
        let concat1: Vec<char> = s1.iter().chain(s2.iter()).copied().collect();
        let concat2: Vec<char> = s2.iter().chain(s1.iter()).copied().collect();
        let size_concat1 = self.get_size(&concat1);
        let size_concat2 = self.get_size(&concat2);
        let min_concat = std::cmp::min(size_concat1, size_concat2);
        Ok(NcdBase::compute_distance(min_concat, &[size1, size2]))
    }

    fn maximum(&self, _s1: &[char], _s2: &[char]) -> f64 {
        1.0
    }
}
