use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TextDistanceError {
    InvalidParameter(String),
    EmptyInputSequence,
    CalculationOverflow,
    IncompatibleLength,
}

impl fmt::Display for TextDistanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextDistanceError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            TextDistanceError::EmptyInputSequence => write!(f, "Empty input sequence"),
            TextDistanceError::CalculationOverflow => write!(f, "Calculation overflow"),
            TextDistanceError::IncompatibleLength => write!(f, "Incompatible sequence lengths"),
        }
    }
}

impl std::error::Error for TextDistanceError {}
