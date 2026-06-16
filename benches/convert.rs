use std::io::Cursor;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fast_webp::{convert_bytes_to_webp, encode_dynamic_image, WebpOptions};
use image::{DynamicImage, ImageFormat, Rgb, RgbImage, Rgba, RgbaImage};

fn rgb_sample(width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgb8(RgbImage::from_fn(width, height, |x, y| {
        Rgb([
            ((x * 13 + y * 7) & 0xff) as u8,
            ((x * 3 + y * 17) & 0xff) as u8,
            ((x * 11 + y * 5) & 0xff) as u8,
        ])
    }))
}

fn rgba_sample(width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(RgbaImage::from_fn(width, height, |x, y| {
        Rgba([
            ((x * 13 + y * 7) & 0xff) as u8,
            ((x * 3 + y * 17) & 0xff) as u8,
            ((x * 11 + y * 5) & 0xff) as u8,
            (96 + ((x + y) & 0x7f)) as u8,
        ])
    }))
}

fn noisy_rgb_sample(width: u32, height: u32) -> DynamicImage {
    DynamicImage::ImageRgb8(RgbImage::from_fn(width, height, |x, y| {
        let mut n = x
            .wrapping_mul(1_664_525)
            .wrapping_add(y.wrapping_mul(1_013_904_223));
        n ^= n >> 16;
        n = n.wrapping_mul(2_246_822_519);
        Rgb([n as u8, (n >> 8) as u8, (n >> 16) as u8])
    }))
}

fn encode_input(image: &DynamicImage, format: ImageFormat) -> Vec<u8> {
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), format)
        .unwrap();
    bytes
}

fn bench_dynamic_image(c: &mut Criterion) {
    let options = WebpOptions::default();
    let samples = [
        ("rgb_512", rgb_sample(512, 512)),
        ("rgba_512", rgba_sample(512, 512)),
        ("rgb_1920", rgb_sample(1920, 1080)),
    ];

    let mut group = c.benchmark_group("dynamic_image_to_webp");
    for (name, image) in samples {
        group.bench_with_input(BenchmarkId::new("webp_crate", name), &image, |b, image| {
            b.iter(|| {
                let encoder = webp::Encoder::from_image(black_box(image)).unwrap();
                black_box(encoder.encode(75.0).len())
            });
        });

        group.bench_with_input(BenchmarkId::new("fast_webp", name), &image, |b, image| {
            b.iter(|| {
                let webp = encode_dynamic_image(black_box(image), options).unwrap();
                black_box(webp.len())
            });
        });
    }
    group.finish();
}

fn bench_bytes(c: &mut Criterion) {
    let options = WebpOptions::default();
    let rgb = rgb_sample(1280, 720);
    let rgba = rgba_sample(1024, 768);
    let noisy_rgb = noisy_rgb_sample(1400, 1000);
    let jpeg = encode_input(&rgb, ImageFormat::Jpeg);
    let png_rgb = encode_input(&rgb, ImageFormat::Png);
    let png_rgba = encode_input(&rgba, ImageFormat::Png);
    let png_noise = encode_input(&noisy_rgb, ImageFormat::Png);

    let samples = vec![
        ("jpeg_rgb_1280x720".to_string(), jpeg),
        ("png_rgb_1280x720".to_string(), png_rgb),
        ("png_rgba_1024x768".to_string(), png_rgba),
        (
            format!("png_rgb_noise_{}kb", png_noise.len() / 1024),
            png_noise,
        ),
    ];

    let mut group = c.benchmark_group("bytes_to_webp");
    for (name, bytes) in samples {
        group.bench_with_input(BenchmarkId::new("fast_webp", &name), &bytes, |b, bytes| {
            b.iter(|| {
                let webp = convert_bytes_to_webp(black_box(bytes), options).unwrap();
                black_box(webp.len())
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_dynamic_image, bench_bytes);
criterion_main!(benches);
