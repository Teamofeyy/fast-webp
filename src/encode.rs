use std::mem::MaybeUninit;

use libwebp_sys::{
    WebPConfig, WebPConfigInitInternal, WebPConfigLosslessPreset, WebPEncode, WebPMemoryWrite,
    WebPMemoryWriter, WebPMemoryWriterClear, WebPMemoryWriterInit, WebPPicture, WebPPictureFree,
    WebPPictureImportRGB, WebPPictureImportRGBA, WebPPictureInitInternal, WebPPreset,
    WebPValidateConfig, WEBP_ENCODER_ABI_VERSION, WEBP_MAX_DIMENSION,
};

use crate::error::WebpError;
use crate::options::WebpOptions;
use crate::pixel::PixelLayout;

const WEBP_METHOD_FASTEST: i32 = 0;

pub fn encode_rgb(
    rgb: &[u8],
    width: u32,
    height: u32,
    options: WebpOptions,
) -> Result<Vec<u8>, WebpError> {
    let geometry = checked_geometry(rgb.len(), width, height, PixelLayout::Rgb, options)?;
    encode_with_libwebp(rgb, geometry, PixelLayout::Rgb, options)
}

pub fn encode_rgba(
    rgba: &[u8],
    width: u32,
    height: u32,
    options: WebpOptions,
) -> Result<Vec<u8>, WebpError> {
    let geometry = checked_geometry(rgba.len(), width, height, PixelLayout::Rgba, options)?;
    encode_with_libwebp(rgba, geometry, PixelLayout::Rgba, options)
}

struct Geometry {
    width: i32,
    height: i32,
    stride: i32,
}

fn checked_geometry(
    actual_len: usize,
    width: u32,
    height: u32,
    layout: PixelLayout,
    options: WebpOptions,
) -> Result<Geometry, WebpError> {
    if !options.quality.is_finite() || !(0.0..=100.0).contains(&options.quality) {
        return Err(WebpError::InvalidQuality(options.quality));
    }

    if width == 0 || height == 0 || width > WEBP_MAX_DIMENSION || height > WEBP_MAX_DIMENSION {
        return Err(WebpError::InvalidDimensions { width, height });
    }

    let channels = layout.channels();
    let expected = usize::try_from(width)
        .ok()
        .and_then(|w| usize::try_from(height).ok().and_then(|h| w.checked_mul(h)))
        .and_then(|pixels| pixels.checked_mul(channels))
        .ok_or(WebpError::InvalidDimensions { width, height })?;

    if actual_len != expected {
        return Err(WebpError::InvalidBufferLength {
            expected,
            actual: actual_len,
        });
    }

    let width_i32 =
        i32::try_from(width).map_err(|_| WebpError::InvalidDimensions { width, height })?;
    let height_i32 =
        i32::try_from(height).map_err(|_| WebpError::InvalidDimensions { width, height })?;
    let stride = width_i32
        .checked_mul(channels as i32)
        .ok_or(WebpError::InvalidDimensions { width, height })?;

    Ok(Geometry {
        width: width_i32,
        height: height_i32,
        stride,
    })
}

fn encode_with_libwebp(
    pixels: &[u8],
    geometry: Geometry,
    layout: PixelLayout,
    options: WebpOptions,
) -> Result<Vec<u8>, WebpError> {
    let config = create_fast_config(options)?;

    unsafe {
        let mut writer = MemoryWriter::new();
        let mut picture = Picture::new(geometry.width, geometry.height)?;
        picture.as_mut().writer = Some(WebPMemoryWrite);
        picture.as_mut().custom_ptr = writer.as_mut_ptr().cast();

        let imported = match layout {
            PixelLayout::Rgb => {
                WebPPictureImportRGB(picture.as_mut(), pixels.as_ptr(), geometry.stride) != 0
            }
            PixelLayout::Rgba => {
                WebPPictureImportRGBA(picture.as_mut(), pixels.as_ptr(), geometry.stride) != 0
            }
        };
        if !imported {
            return Err(WebpError::EncodeWebp);
        }

        if WebPEncode(&config, picture.as_mut()) == 0 {
            return Err(WebpError::EncodeWebp);
        }

        writer.to_vec()
    }
}

fn create_fast_config(options: WebpOptions) -> Result<WebPConfig, WebpError> {
    let mut config = MaybeUninit::<WebPConfig>::uninit();
    let ok = unsafe {
        WebPConfigInitInternal(
            config.as_mut_ptr(),
            WebPPreset::WEBP_PRESET_DEFAULT,
            options.quality,
            WEBP_ENCODER_ABI_VERSION as i32,
        )
    } != 0;
    if !ok {
        return Err(WebpError::EncodeWebp);
    }

    let mut config = unsafe { config.assume_init() };
    config.lossless = i32::from(options.lossless);
    config.quality = options.quality;
    config.method = WEBP_METHOD_FASTEST;
    config.pass = 1;
    config.thread_level = 1;
    config.alpha_quality = 90;

    if options.lossless {
        unsafe {
            WebPConfigLosslessPreset(&mut config, WEBP_METHOD_FASTEST);
        }
        config.method = WEBP_METHOD_FASTEST;
        config.thread_level = 1;
        config.pass = 1;
    }

    if unsafe { WebPValidateConfig(&config) } == 0 {
        return Err(WebpError::EncodeWebp);
    }

    Ok(config)
}

struct Picture {
    picture: WebPPicture,
}

impl Picture {
    fn new(width: i32, height: i32) -> Result<Self, WebpError> {
        let mut picture = MaybeUninit::<WebPPicture>::uninit();
        let ok = unsafe {
            WebPPictureInitInternal(picture.as_mut_ptr(), WEBP_ENCODER_ABI_VERSION as i32)
        } != 0;
        if !ok {
            return Err(WebpError::EncodeWebp);
        }

        let mut picture = unsafe { picture.assume_init() };
        picture.width = width;
        picture.height = height;
        Ok(Self { picture })
    }

    fn as_mut(&mut self) -> &mut WebPPicture {
        &mut self.picture
    }
}

impl Drop for Picture {
    fn drop(&mut self) {
        unsafe {
            WebPPictureFree(&mut self.picture);
        }
    }
}

struct MemoryWriter {
    writer: WebPMemoryWriter,
}

impl MemoryWriter {
    unsafe fn new() -> Self {
        let mut writer = MaybeUninit::<WebPMemoryWriter>::uninit();
        WebPMemoryWriterInit(writer.as_mut_ptr());
        Self {
            writer: writer.assume_init(),
        }
    }

    fn as_mut_ptr(&mut self) -> *mut WebPMemoryWriter {
        &mut self.writer
    }

    unsafe fn to_vec(&self) -> Result<Vec<u8>, WebpError> {
        if self.writer.mem.is_null() || self.writer.size == 0 {
            return Err(WebpError::EncodeWebp);
        }

        Ok(std::slice::from_raw_parts(self.writer.mem, self.writer.size).to_vec())
    }
}

impl Drop for MemoryWriter {
    fn drop(&mut self) {
        unsafe {
            WebPMemoryWriterClear(&mut self.writer);
        }
    }
}
