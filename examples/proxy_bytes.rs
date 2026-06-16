use std::env;

use fast_webp::{convert_bytes_to_webp, WebpOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = env::args()
        .nth(1)
        .unwrap_or_else(|| "input.jpg".to_string());
    let output = env::args()
        .nth(2)
        .unwrap_or_else(|| "output.webp".to_string());

    let response_body = std::fs::read(input)?;
    let webp = convert_bytes_to_webp(
        &response_body,
        WebpOptions {
            quality: 75.0,
            lossless: false,
        },
    )?;

    std::fs::write(output, webp)?;
    Ok(())
}
