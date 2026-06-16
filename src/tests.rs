use super::*;
use crate::bytes::PNG_SIG;

fn assert_webp(bytes: &[u8]) {
    assert!(bytes.len() >= 12);
    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WEBP");
}

#[test]
fn encodes_rgb_buffer() {
    let rgb = [255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
    let webp = encode_rgb(&rgb, 2, 2, WebpOptions::default()).unwrap();
    assert_webp(&webp);
}

#[test]
fn encodes_rgba_buffer() {
    let rgba = [
        255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 255, 255, 255, 255, 0,
    ];
    let webp = encode_rgba(&rgba, 2, 2, WebpOptions::default()).unwrap();
    assert_webp(&webp);
}

#[test]
fn rejects_invalid_quality() {
    let err = encode_rgb(
        &[0, 0, 0],
        1,
        1,
        WebpOptions {
            quality: f32::NAN,
            lossless: false,
        },
    )
    .unwrap_err();

    assert!(matches!(err, WebpError::InvalidQuality(_)));
}

#[test]
fn rejects_invalid_dimensions() {
    let err = encode_rgb(&[], 0, 1, WebpOptions::default()).unwrap_err();
    assert!(matches!(err, WebpError::InvalidDimensions { .. }));
}

#[test]
fn rejects_invalid_buffer_length() {
    let err = encode_rgb(&[0, 0, 0], 2, 2, WebpOptions::default()).unwrap_err();
    assert!(matches!(err, WebpError::InvalidBufferLength { .. }));
}

#[test]
fn detects_formats() {
    assert_eq!(detect_format(PNG_SIG), Some(InputFormat::Png));
    assert_eq!(
        detect_format(&[0xff, 0xd8, 0xff, 0xdb]),
        Some(InputFormat::Jpeg)
    );
    assert_eq!(detect_format(b"not an image"), None);
}

#[cfg(feature = "image-codecs")]
fn make_png_rgba() -> Vec<u8> {
    let image = image::RgbaImage::from_raw(
        2,
        2,
        vec![
            255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 255, 255, 255, 255, 0,
        ],
    )
    .unwrap();
    let mut out = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

#[cfg(feature = "image-codecs")]
#[test]
fn converts_png_bytes_to_webp() {
    let png = make_png_rgba();
    let webp = convert_bytes_to_webp(&png, WebpOptions::default()).unwrap();
    assert_webp(&webp);
}

#[cfg(feature = "image-codecs")]
#[test]
fn converts_jpeg_bytes_to_webp() {
    let mut jpeg = Vec::new();
    let image = image::RgbImage::from_fn(8, 8, |x, y| {
        image::Rgb([(x * 31) as u8, (y * 29) as u8, 180])
    });
    image
        .write_to(
            &mut std::io::Cursor::new(&mut jpeg),
            image::ImageFormat::Jpeg,
        )
        .unwrap();

    let webp = convert_bytes_to_webp(&jpeg, WebpOptions::default()).unwrap();
    assert_webp(&webp);
}

#[cfg(feature = "image")]
#[test]
fn encodes_image_crate_buffers() {
    let rgb = image::RgbImage::from_fn(2, 2, |x, y| {
        image::Rgb([(x * 120) as u8, (y * 120) as u8, 64])
    });
    let rgba = image::RgbaImage::from_fn(2, 2, |x, y| {
        image::Rgba([(x * 120) as u8, (y * 120) as u8, 64, 180])
    });

    assert_webp(&encode_rgb_image(&rgb, WebpOptions::default()).unwrap());
    assert_webp(&encode_rgba_image(&rgba, WebpOptions::default()).unwrap());
    assert_webp(
        &encode_dynamic_image(&image::DynamicImage::ImageRgb8(rgb), WebpOptions::default())
            .unwrap(),
    );
}

#[test]
fn malformed_input_returns_error() {
    let err = convert_bytes_to_webp(b"bad", WebpOptions::default()).unwrap_err();
    assert!(matches!(err, WebpError::UnsupportedFormat));
}

#[cfg(feature = "image-codecs")]
#[test]
fn malformed_png_returns_error() {
    let err = convert_bytes_to_webp(PNG_SIG, WebpOptions::default()).unwrap_err();
    assert!(matches!(err, WebpError::DecodePng(_)));
}

#[cfg(feature = "image-codecs")]
#[test]
fn malformed_jpeg_returns_error() {
    let err = convert_bytes_to_webp(&[0xff, 0xd8, 0xff], WebpOptions::default()).unwrap_err();
    assert!(matches!(err, WebpError::DecodeJpeg(_)));
}
