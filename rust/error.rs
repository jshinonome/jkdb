use std::fmt;

#[derive(Debug, Clone)]
pub enum KError {
    UnsupportedKType(u8),
    UnsupportedKList(u8),
    NotAnArray,
    NotSameSizeColumn(String),
    BufferTooShort,
    InvalidMessage(String),
}

impl fmt::Display for KError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KError::UnsupportedKType(t) => write!(f, "UNSUPPORTED_K_TYPE - {t}"),
            KError::UnsupportedKList(t) => write!(f, "UNSUPPORTED_K_LIST - {t}"),
            KError::NotAnArray => write!(f, "NOT_AN_ARRAY"),
            KError::NotSameSizeColumn(c) => write!(f, "NOT_SAME_SIZE_COLUMN - {c}"),
            KError::BufferTooShort => write!(f, "BUFFER_TOO_SHORT"),
            KError::InvalidMessage(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for KError {}
