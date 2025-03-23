pub mod css;
pub mod dom;
pub mod html;
pub mod layout;
pub mod paint;
pub mod style;

use dom::*;

mod tests {
    use fontdue::Font;
    use image::{Rgb, RgbImage};

    #[test]
    fn test_font_dude() {
        let font_data = include_bytes!("../testfiles/NotoSerif-Bold.ttf") as &[u8];
        let font = Font::from_bytes(font_data, fontdue::FontSettings::default())
            .expect("font load failed");

        let font_size = 40.0;
        let text = "Hello, wwww, iiiii, fffffffooooonnnnnnntttttttddddddduuuuueeeee, fontdue!";

        let mut width = 0;
        let mut max_height = 0;

        let glyphs: Vec<_> = text
            .chars()
            .map(|c| {
                let (metrics, _) = font.rasterize(c, font_size);
                width += metrics.advance_width.ceil() as usize;
                if metrics.height > max_height {
                    max_height = metrics.height;
                }
                (c, metrics)
            })
            .collect();

        let mut image = RgbImage::from_pixel(
            (width + 10) as u32,
            (max_height + 10) as u32,
            Rgb([255, 255, 255]),
        );

        let mut x_cursor = 5;
        for (c, metrics) in &glyphs {
            let (metrics, bitmap) = font.rasterize(*c, font_size);
            let y_offset = max_height - metrics.height + 5;

            for y in 0..metrics.height {
                for x in 0..metrics.width {
                    let alpha = bitmap[y * metrics.width + x];
                    let val = 255 - alpha;
                    if x_cursor + x < image.width() as usize
                        && y + y_offset < image.height() as usize
                    {
                        image.put_pixel(
                            (x_cursor + x) as u32,
                            (y + y_offset) as u32,
                            Rgb([val, val, val]),
                        );
                    }
                }
            }

            x_cursor += metrics.advance_width.ceil() as usize;
        }

        image
            .save("output-font-due.bmp")
            .expect("failed to save as bmp");
    }
}
