#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PixelLayout {
    Rgb,
    Rgba,
}

impl PixelLayout {
    pub(crate) const fn channels(self) -> usize {
        match self {
            Self::Rgb => 3,
            Self::Rgba => 4,
        }
    }
}

#[cfg(feature = "image")]
pub(crate) fn gray_to_rgb(gray: &[u8]) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(gray.len() * 3);
    for &g in gray {
        rgb.extend_from_slice(&[g, g, g]);
    }
    rgb
}

#[cfg(feature = "image")]
pub(crate) fn gray_alpha_to_rgba(gray_alpha: &[u8]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(gray_alpha.len() * 2);
    for pixel in gray_alpha.chunks_exact(2) {
        rgba.extend_from_slice(&[pixel[0], pixel[0], pixel[0], pixel[1]]);
    }
    rgba
}
