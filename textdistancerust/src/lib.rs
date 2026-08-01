pub mod error;
pub mod identity;
pub mod length;
pub mod matrix;
pub mod postfix;
pub mod prefix;
pub mod tokenizer;
pub mod traits;

pub use error::TextDistanceError;
pub use identity::Identity;
pub use length::Length;
pub use matrix::Matrix;
pub use postfix::Postfix;
pub use prefix::Prefix;
pub use tokenizer::{find_ngrams, to_char_vec, to_word_vec};
pub use traits::{DistanceMetric, SimilarityMetric};
