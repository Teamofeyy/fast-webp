use std::env;

use fast_webp::{encode_rgb, WebpOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = env::args()
        .nth(1)
        .unwrap_or_else(|| "raw-buffer.webp".to_string());

    let width = 2;
    let height = 2;
    let rgb = [
        255, 0, 0, // red
        0, 255, 0, // green
        0, 0, 255, // blue
        255, 255, 255, // white
    ];

    let webp = encode_rgb(&rgb, width, height, WebpOptions::default())?;
    std::fs::write(output, webp)?;
    Ok(())
}
