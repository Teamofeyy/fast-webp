use crate::error::WebpError;
#[cfg(feature = "image-codecs")]
use crate::image_api::encode_dynamic_image;
use crate::options::WebpOptions;

pub(crate) const PNG_SIG: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputFormat {
    Jpeg,
    Png,
}

pub fn detect_format(input: &[u8]) -> Option<InputFormat> {
    if input.starts_with(PNG_SIG) {
        Some(InputFormat::Png)
    } else if input.len() >= 3 && input[0] == 0xff && input[1] == 0xd8 && input[2] == 0xff {
        Some(InputFormat::Jpeg)
    } else {
        None
    }
}

pub fn convert_bytes_to_webp(input: &[u8], options: WebpOptions) -> Result<Vec<u8>, WebpError> {
    convert_bytes_to_webp_impl(input, options)
}

#[cfg(feature = "image-codecs")]
fn convert_bytes_to_webp_impl(input: &[u8], options: WebpOptions) -> Result<Vec<u8>, WebpError> {
    let image_format = match detect_format(input).ok_or(WebpError::UnsupportedFormat)? {
        InputFormat::Jpeg => image::ImageFormat::Jpeg,
        InputFormat::Png => image::ImageFormat::Png,
    };
    let image =
        image::load_from_memory_with_format(input, image_format).map_err(
            |err| match image_format {
                image::ImageFormat::Jpeg => WebpError::DecodeJpeg(err.to_string()),
                image::ImageFormat::Png => WebpError::DecodePng(err.to_string()),
                _ => WebpError::UnsupportedFormat,
            },
        )?;

    encode_dynamic_image(&image, options)
}

#[cfg(not(feature = "image-codecs"))]
fn convert_bytes_to_webp_impl(_input: &[u8], _options: WebpOptions) -> Result<Vec<u8>, WebpError> {
    Err(WebpError::UnsupportedFormat)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_formats() {
        assert_eq!(detect_format(PNG_SIG), Some(InputFormat::Png));
        assert_eq!(
            detect_format(&[0xff, 0xd8, 0xff, 0xdb]),
            Some(InputFormat::Jpeg)
        );
        assert_eq!(detect_format(b"not an image"), None);
    }
}
