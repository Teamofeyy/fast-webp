use crate::encode::{encode_rgb, encode_rgba};
use crate::error::WebpError;
use crate::options::WebpOptions;
use crate::pixel::{gray_alpha_to_rgba, gray_to_rgb};

pub fn encode_dynamic_image(
    image: &image::DynamicImage,
    options: WebpOptions,
) -> Result<Vec<u8>, WebpError> {
    match image {
        image::DynamicImage::ImageRgb8(rgb) => encode_rgb_image(rgb, options),
        image::DynamicImage::ImageRgba8(rgba) => encode_rgba_image(rgba, options),
        image::DynamicImage::ImageLuma8(gray) => {
            let rgb = gray_to_rgb(gray.as_raw());
            encode_rgb(&rgb, gray.width(), gray.height(), options)
        }
        image::DynamicImage::ImageLumaA8(gray_alpha) => {
            let rgba = gray_alpha_to_rgba(gray_alpha.as_raw());
            encode_rgba(&rgba, gray_alpha.width(), gray_alpha.height(), options)
        }
        image::DynamicImage::ImageRgb16(_) | image::DynamicImage::ImageRgb32F(_) => {
            let rgb = image.to_rgb8();
            encode_rgb_image(&rgb, options)
        }
        _ => {
            let rgba = image.to_rgba8();
            encode_rgba_image(&rgba, options)
        }
    }
}

pub fn encode_rgb_image(
    image: &image::RgbImage,
    options: WebpOptions,
) -> Result<Vec<u8>, WebpError> {
    encode_rgb(image.as_raw(), image.width(), image.height(), options)
}

pub fn encode_rgba_image(
    image: &image::RgbaImage,
    options: WebpOptions,
) -> Result<Vec<u8>, WebpError> {
    encode_rgba(image.as_raw(), image.width(), image.height(), options)
}
