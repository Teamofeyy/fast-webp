#[derive(Clone, Copy, Debug)]
pub struct WebpOptions {
    pub quality: f32,
    pub lossless: bool,
}

impl Default for WebpOptions {
    fn default() -> Self {
        Self {
            quality: 75.0,
            lossless: false,
        }
    }
}
