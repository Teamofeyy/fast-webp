use std::fmt;

#[derive(Debug)]
pub enum WebpError {
    UnsupportedFormat,

    InvalidQuality(f32),

    InvalidDimensions { width: u32, height: u32 },

    InvalidBufferLength { expected: usize, actual: usize },

    DecodeJpeg(String),

    DecodePng(String),

    EncodeWebp,
}

impl fmt::Display for WebpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFormat => write!(f, "unsupported input format"),
            Self::InvalidQuality(quality) => write!(f, "invalid quality: {quality}"),
            Self::InvalidDimensions { width, height } => {
                write!(f, "invalid dimensions: {width}x{height}")
            }
            Self::InvalidBufferLength { expected, actual } => {
                write!(
                    f,
                    "invalid buffer length: expected {expected}, got {actual}"
                )
            }
            Self::DecodeJpeg(message) => write!(f, "jpeg decode failed: {message}"),
            Self::DecodePng(message) => write!(f, "png decode failed: {message}"),
            Self::EncodeWebp => write!(f, "webp encode failed"),
        }
    }
}

impl std::error::Error for WebpError {}
