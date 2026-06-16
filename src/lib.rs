//! Fast WebP encoding for the two common Rust workflows:
//!
//! - `image` crate buffers (`DynamicImage`, `RgbImage`, `RgbaImage`) to WebP.
//! - JPEG/PNG bytes from servers and proxies to WebP bytes.
//!
//! The low-level hot path is intentionally small:
//! RGB/RGBA bytes are validated and passed directly to `libwebp-sys`.
//!
//! ```rust
//! use fast_webp::{encode_rgb, WebpOptions};
//!
//! let rgb = [255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
//! let webp = encode_rgb(&rgb, 2, 2, WebpOptions::default())?;
//! # Ok::<(), fast_webp::WebpError>(())
//! ```

mod bytes;
mod encode;
mod error;
mod options;
mod pixel;

#[cfg(feature = "image")]
mod image_api;

pub use bytes::{convert_bytes_to_webp, detect_format, InputFormat};
pub use encode::{encode_rgb, encode_rgba};
pub use error::WebpError;
pub use options::WebpOptions;

#[cfg(feature = "image")]
pub use image_api::{encode_dynamic_image, encode_rgb_image, encode_rgba_image};

#[cfg(test)]
mod tests;
