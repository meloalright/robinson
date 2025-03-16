use super::css::*;
use super::layout::*;

type DisplayList = Vec<DisplayCommand>;

enum DisplayCommand {
    SolidColor(Color, Rect),
    // insert more commands here
}

fn build_display_list(layout_root: &LayoutBox) -> DisplayList {
    let mut list = Vec::new();
    render_layout_box(&mut list, layout_root);
    return list;
}

fn render_layout_box(list: &mut DisplayList, layout_box: &LayoutBox) {
    render_background(list, layout_box);
    render_borders(list, layout_box);
    // TODO: render text

    for child in &layout_box.children {
        render_layout_box(list, child);
    }
}

fn render_background(list: &mut DisplayList, layout_box: &LayoutBox) {
    get_color(layout_box, "background").map(|color| {
        list.push(DisplayCommand::SolidColor(
            color,
            layout_box.dimensions.border_box(),
        ))
    });
}

// Return the specified color for CSS property `name`, or None if no color was specified.
fn get_color(layout_box: &LayoutBox, name: &str) -> Option<Color> {
    match layout_box.box_type {
        BoxType::BlockNode(style) | BoxType::InlineNode(style) => match style.value(name) {
            Some(Value::ColorValue(color)) => Some(color),
            _ => None,
        },
        BoxType::AnonymousBlock => None,
    }
}

fn render_borders(list: &mut DisplayList, layout_box: &LayoutBox) {
    let color = match get_color(layout_box, "border-color") {
        Some(color) => color,
        _ => return, // bail out if no border-color is specified
    };

    let d = &layout_box.dimensions;
    let border_box = d.border_box();

    // Left border
    list.push(DisplayCommand::SolidColor(
        color,
        Rect {
            x: border_box.x,
            y: border_box.y,
            width: d.border.left,
            height: border_box.height,
        },
    ));

    // Right border
    list.push(DisplayCommand::SolidColor(
        color,
        Rect {
            x: border_box.x + border_box.width - d.border.right,
            y: border_box.y,
            width: d.border.right,
            height: border_box.height,
        },
    ));

    // Top border
    list.push(DisplayCommand::SolidColor(
        color,
        Rect {
            x: border_box.x,
            y: border_box.y,
            width: border_box.width,
            height: d.border.top,
        },
    ));

    // Bottom border
    list.push(DisplayCommand::SolidColor(
        color,
        Rect {
            x: border_box.x,
            y: border_box.y + border_box.height - d.border.bottom,
            width: border_box.width,
            height: d.border.bottom,
        },
    ));
}

struct Canvas {
    pixels: Vec<Color>,
    width: usize,
    height: usize,
}

impl Canvas {
    // Create a blank canvas
    fn new(width: usize, height: usize) -> Canvas {
        let white = Color {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        };
        Canvas {
            pixels: vec![white; width * height],
            width,
            height,
        }
    }
    // ...

    fn paint_item(&mut self, item: &DisplayCommand) {
        match item {
            &DisplayCommand::SolidColor(color, rect) => {
                // Clip the rectangle to the canvas boundaries.
                let x0 = rect.x.clamp(0.0, self.width as f32) as usize;
                let y0 = rect.y.clamp(0.0, self.height as f32) as usize;
                let x1 = (rect.x + rect.width).clamp(0.0, self.width as f32) as usize;
                let y1 = (rect.y + rect.height).clamp(0.0, self.height as f32) as usize;

                for y in (y0..y1) {
                    for x in (x0..x1) {
                        // TODO: alpha compositing with existing pixel
                        self.pixels[x + y * self.width] = color;
                    }
                }
            }
        }
    }
}

// Paint a tree of LayoutBoxes to an array of pixels.
fn paint(layout_root: &LayoutBox, bounds: Rect) -> Canvas {
    let display_list = build_display_list(layout_root);
    let mut canvas = Canvas::new(bounds.width as usize, bounds.height as usize);
    for item in display_list {
        canvas.paint_item(&item);
    }
    return canvas;
}

mod tests {
    use super::super::css;
    use super::super::html;
    use super::super::layout;
    use super::super::style::*;
    use super::*;

    extern crate image;
    use std::fs::File;
    use std::io::{BufWriter, Read};

    #[test]
    fn test_rasterization() {
        let root = html::parse(
            "<div class=\"a\">
  <div class=\"b\">
    <div class=\"c\">
      <div class=\"d\">
        <div class=\"e\">
          <div class=\"f\">
            <div class=\"g\">
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>"
                .to_string(),
        );
        let css = css::parse(
            "* { display: block; padding: 12px; }
.a { background: #ff0000; }
.b { background: #ffa500; }
.c { background: #ffff00; }
.d { background: #008000; }
.e { background: #0000ff; }
.f { background: #4b0082; }
.g { background: #800080; }"
                .to_owned(),
        );

        let styled_tree = style_tree(&root, &css);

        let mut layout_tree = build_layout_tree(&styled_tree);

        let mut dimension = Dimensions::default();

        dimension.content.width = 800.0;

        layout_tree.layout(dimension);

        let canvas = paint(
            &layout_tree,
            Rect {
                x: 0.,
                y: 0.,
                width: 1000.,
                height: 1000.,
            },
        );
        let (w, h) = (canvas.width as u32, canvas.height as u32);

        let img = image::ImageBuffer::from_fn(w, h, move |x, y| {
            let color = canvas.pixels[(y * w + x) as usize];
            image::Rgba([color.r, color.g, color.b, color.a])
        });

        let mut file = BufWriter::new(File::create("output.png").unwrap());
        image::DynamicImage::ImageRgba8(img).write_to(&mut file, image::ImageFormat::Png);
    }
    #[test]
    fn test_rasterization2() {
        let root = html::parse(
            "<div><div class=\"a\">
  <div class=\"b\">
    <div class=\"c\">
      <div class=\"d\">
        <div class=\"e\">
          <div class=\"f\">
            <div class=\"g\">
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div></div>"
                .to_string(),
        );
        let css = css::parse(
            "* { display: block; padding: 12px; }
.a { background: #ff0000; }
.b { background: #ffa500; }
.c { background: #ffff00; }
.d { background: #008000; }
.e { background: #0000ff; }
.f { background: #4b0082; }
.g { background: #800080; }"
                .to_owned(),
        );

        let styled_tree = style_tree(&root, &css);

        let mut layout_tree = build_layout_tree(&styled_tree);

        let mut dimension = Dimensions::default();

        dimension.content.width = 800.0;

        layout_tree.layout2(Value::Length(800.0, Unit::Px), Value::Length(800.0, Unit::Px));
        layout_tree.calc_position();

        let canvas = paint(
            &layout_tree,
            Rect {
                x: 0.,
                y: 0.,
                width: 1000.,
                height: 1000.,
            },
        );
        let (w, h) = (canvas.width as u32, canvas.height as u32);

        let img = image::ImageBuffer::from_fn(w, h, move |x, y| {
            let color = canvas.pixels[(y * w + x) as usize];
            image::Rgba([color.r, color.g, color.b, color.a])
        });

        let mut file = BufWriter::new(File::create("output2.png").unwrap());
        image::DynamicImage::ImageRgba8(img).write_to(&mut file, image::ImageFormat::Png);
    }
}
