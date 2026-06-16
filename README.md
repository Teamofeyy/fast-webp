# fast-webp

`fast-webp` is a small WebP encoder crate for Rust applications that need the
ergonomics of the `webp` crate and a faster hot path.

It is intentionally narrow:

- `RgbImage`, `RgbaImage`, `DynamicImage` -> WebP bytes
- RGB/RGBA byte buffers -> WebP bytes
- optional JPEG/PNG response bytes -> WebP bytes via `image` codecs

It does not try to be an image processing framework. Resize, crop, filters,
metadata editing, animation, and WebP decoding are out of scope for the first
version.

## Why this exists

The upstream [`webp`](https://github.com/jaredforth/webp) crate has a convenient
`Encoder::from_image(&image).encode(quality)` API. Internally it is organized as
a small wrapper around `libwebp-sys`: `src/lib.rs` re-exports modules,
`src/encoder.rs` builds an encoder/picture pipeline, and `src/shared.rs` owns the
WebP memory wrappers.

That shape is good for a general wrapper, but for reverse proxies and HTTP image
conversion we want a shorter path:

```text
DynamicImage/RGB/RGBA/JPEG/PNG -> RGB/RGBA bytes -> WebPEncodeRGB/RGBA -> Vec<u8>
```

So `fast-webp` keeps the public API simple and pushes the encoding hot path
through `libwebp-sys` with a fast encoder configuration:

- `WebPConfig` with `method = 0`
- `pass = 1`
- `thread_level = 1`
- `WebPPictureImportRGB`
- `WebPPictureImportRGBA`
- `WebPEncode`
- `WebPFree`

## Install

```toml
[dependencies]
fast-webp = "0.1.1"
```

Runtime dependencies are deliberately close to `webp`:

```toml
[dependencies]
libwebp-sys = "0.9.3"
image = { version = "^0.25.0", default-features = false, optional = true }
```

Minimal encoder-only build:

```toml
[dependencies]
fast-webp = { version = "0.1.1", default-features = false }
```

Only `image` buffer integration:

```toml
[dependencies]
fast-webp = { version = "0.1.1", default-features = false, features = ["image"] }
```

JPEG/PNG byte conversion through `image` codecs:

```toml
[dependencies]
fast-webp = { version = "0.1.1", features = ["image-codecs"] }
```

If your proxy already decodes PNG/JPEG with another fast decoder, prefer feeding
the resulting RGB/RGBA buffer into `encode_rgb` or `encode_rgba`. That is the
fast path this crate optimizes.

## Public API

### Raw Buffers

Use this when you already have tightly packed RGB/RGBA pixels.

```rust
use fast_webp::{encode_rgb, encode_rgba, WebpOptions};

let rgb = vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
let webp = encode_rgb(&rgb, 2, 2, WebpOptions::default())?;

let rgba = vec![
    255, 0, 0, 255,
    0, 255, 0, 128,
    0, 0, 255, 255,
    255, 255, 255, 0,
];
let webp_with_alpha = encode_rgba(&rgba, 2, 2, WebpOptions::default())?;
```

The encoder validates:

- quality is finite and in `0.0..=100.0`
- width and height are non-zero
- dimensions are within `WEBP_MAX_DIMENSION`
- `width * height * channels` does not overflow
- input length exactly matches the expected packed buffer length

### image crate

Enable the `image` feature. It is enabled by default.

```rust
use fast_webp::{encode_dynamic_image, WebpOptions};

let image = image::open("input.jpg")?;
let webp = encode_dynamic_image(
    &image,
    WebpOptions {
        quality: 75.0,
        lossless: false,
    },
)?;

std::fs::write("output.webp", webp)?;
```

Direct `RgbImage` / `RgbaImage` entry points avoid unnecessary conversion:

```rust
use fast_webp::{encode_rgb_image, encode_rgba_image, WebpOptions};

let rgb = image::open("photo.jpg")?.to_rgb8();
let webp = encode_rgb_image(&rgb, WebpOptions::default())?;

let rgba = image::open("icon.png")?.to_rgba8();
let webp = encode_rgba_image(&rgba, WebpOptions::default())?;
```

`DynamicImage` handling is deliberately conservative:

- `ImageRgb8` -> RGB encode
- `ImageRgba8` -> RGBA encode
- grayscale -> RGB
- grayscale + alpha -> RGBA
- higher-depth RGB -> `to_rgb8`
- everything else -> `to_rgba8`

### JPEG/PNG bytes

Enable `image-codecs` for this convenience API. This keeps the default crate
small while still allowing a single-call proxy workflow when you want it.

```rust
use fast_webp::{convert_bytes_to_webp, WebpOptions};

let webp = convert_bytes_to_webp(
    &response_body,
    WebpOptions {
        quality: 75.0,
        lossless: false,
    },
)?;
```

Format detection is magic-byte based:

```rust
use fast_webp::{detect_format, InputFormat};

assert_eq!(detect_format(b"\xff\xd8\xff..."), Some(InputFormat::Jpeg));
```

Decoder path:

- JPEG -> `image` JPEG decoder -> `DynamicImage` -> RGB/RGBA encode
- PNG -> `image` PNG decoder -> `DynamicImage` -> RGB/RGBA encode

The 800 ms target for a 4 MB PNG depends mostly on PNG decode time and CPU. The
part controlled by this crate is the WebP encode path after decode: it avoids
the heavier `WebPPicture` setup used by generic wrappers and calls the simple
`WebPEncode*` functions directly. For the fastest proxy path, decode with the
fastest decoder available in your application and call `encode_rgb`/`encode_rgba`.
- PNG grayscale -> RGB
- PNG grayscale + alpha -> RGBA

## Options

```rust
use fast_webp::WebpOptions;

let lossy = WebpOptions {
    quality: 82.0,
    lossless: false,
};

let lossless = WebpOptions {
    quality: 100.0,
    lossless: true,
};
```

For lossless mode, `quality` is still validated for API consistency, but the
simple libwebp lossless functions do not use it.

## File Layout

```text
src/lib.rs          public facade and crate docs
src/options.rs      WebpOptions
src/error.rs        WebpError
src/encode.rs       hot path: validation + fast WebPConfig + libwebp-sys FFI
src/bytes.rs        detect_format + optional image-codecs bytes API
src/image_api.rs    image crate integration
src/pixel.rs        internal pixel layout and grayscale helpers
src/tests.rs        unit tests for public behavior
benches/convert.rs  Criterion comparison with webp crate
examples/           runnable API examples
```

## Examples

```sh
cargo run --example dynamic_image -- input.jpg output.webp
cargo run --features image-codecs --example proxy_bytes -- input.jpg output.webp
cargo run --example raw_buffers -- output.webp
```

## Benchmarks

```sh
cargo bench --features image-codecs
```

Current benchmark groups:

- `webp` crate `DynamicImage -> WebP`
- `fast-webp` `DynamicImage -> WebP`
- `fast-webp` JPEG bytes -> WebP
- `fast-webp` PNG bytes -> WebP

The benchmark generates images in memory so it is reproducible in CI. For proxy
tuning, add your production JPEG/PNG corpus and run the same groups against those
files.
