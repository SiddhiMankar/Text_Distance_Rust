pub mod arith_ncd;
pub mod bag;
pub mod cosine;
pub mod damerau_levenshtein;
pub mod editex;
pub mod error;
pub mod hamming;
pub mod identity;
pub mod jaccard;
pub mod length;
pub mod matrix;
pub mod mra;
pub mod ncd_base;
pub mod overlap;
pub mod postfix;
pub mod prefix;
pub mod rlencd;
pub mod sorensen;
pub mod sqrt_ncd;
pub mod strcmp95;
pub mod tanimoto;
pub mod tokenizer;
pub mod traits;
pub mod tversky;

// monge_elkan module scoped out

pub use arith_ncd::ArithNcd;
pub use bag::Bag;
pub use cosine::Cosine;
pub use damerau_levenshtein::DamerauLevenshtein;
pub use editex::Editex;
pub use error::TextDistanceError;
pub use hamming::Hamming;
pub use identity::Identity;
pub use jaccard::Jaccard;
pub use length::Length;
pub use matrix::Matrix;
pub use mra::Mra;
pub use ncd_base::NcdBase;
pub use overlap::Overlap;
pub use postfix::Postfix;
pub use prefix::Prefix;
pub use rlencd::RlenCd;
pub use sorensen::Sorensen;
pub use sqrt_ncd::SqrtNcd;
pub use strcmp95::StrCmp95;
pub use tanimoto::Tanimoto;
pub use tokenizer::{find_ngrams, to_char_vec, to_word_vec};
pub use traits::{DistanceMetric, SimilarityMetric};
pub use tversky::Tversky;

// MongeElkan re-export removed – scoped out