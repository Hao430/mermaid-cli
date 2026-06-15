pub struct SvgBuilder {
    width: u32,
    height: u32,
    elements: Vec<String>,
}

impl SvgBuilder {
    pub fn new(width: u32, height: u32) -> Self {
        SvgBuilder {
            width,
            height,
            elements: Vec::new(),
        }
    }

    pub fn add_rect(&mut self, x: f32, y: f32, width: f32, height: f32, style: &str) {
        let element = format!(
            r#"  <rect x="{}" y="{}" width="{}" height="{}" style="{}"/>"#,
            x, y, width, height, style
        );
        self.elements.push(element);
    }

    pub fn add_circle(&mut self, cx: f32, cy: f32, r: f32, style: &str) {
        let element = format!(
            r#"  <circle cx="{}" cy="{}" r="{}" style="{}"/>"#,
            cx, cy, r, style
        );
        self.elements.push(element);
    }

    pub fn add_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, style: &str) {
        let element = format!(
            r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" style="{}"/>"#,
            x1, y1, x2, y2, style
        );
        self.elements.push(element);
    }

    pub fn add_text(&mut self, x: f32, y: f32, text: &str, style: &str) {
        let element = format!(
            r#"  <text x="{}" y="{}" style="{}">{}</text>"#,
            x,
            y,
            style,
            escape_xml(text)
        );
        self.elements.push(element);
    }

    pub fn add_arrow(&mut self, x: f32, y: f32) {
        // 简单的箭头：三角形
        let size = 8.0;
        let points = format!(
            "{},{} {},{} {},{}",
            x,
            y,
            x - size,
            y - size,
            x + size,
            y - size
        );
        let element = format!(r#"  <polygon points="{}" style="fill:black"/>"#, points);
        self.elements.push(element);
    }

    pub fn add_path(&mut self, d: &str, style: &str) {
        let element = format!(r#"  <path d="{}" style="{}"/>"#, d, style);
        self.elements.push(element);
    }

    pub fn build(self) -> String {
        let mut svg = String::new();
        svg.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
  <style>
    text {{ font-family: Arial, sans-serif; font-size: 14px; }}
  </style>
"#,
            self.width, self.height
        ));

        for element in self.elements {
            svg.push_str(&element);
            svg.push('\n');
        }

        svg.push_str("</svg>");
        svg
    }
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_builder_basic() {
        let mut svg = SvgBuilder::new(800, 600);
        svg.add_rect(10.0, 10.0, 100.0, 50.0, "fill:white;stroke:black");
        let result = svg.build();

        assert!(result.contains("800"));
        assert!(result.contains("600"));
        assert!(result.contains("<rect"));
        assert!(result.contains("</svg>"));
    }

    #[test]
    fn test_svg_builder_with_text() {
        let mut svg = SvgBuilder::new(800, 600);
        svg.add_text(100.0, 100.0, "Hello", "font-size:14px");
        let result = svg.build();

        assert!(result.contains("Hello"));
        assert!(result.contains("<text"));
    }

    #[test]
    fn test_svg_escape_xml() {
        let mut svg = SvgBuilder::new(800, 600);
        svg.add_text(100.0, 100.0, "A & B < C", "");
        let result = svg.build();

        assert!(result.contains("&amp;"));
        assert!(result.contains("&lt;"));
    }

    #[test]
    fn test_svg_builder_with_shapes() {
        let mut svg = SvgBuilder::new(800, 600);
        svg.add_rect(10.0, 10.0, 100.0, 50.0, "");
        svg.add_circle(200.0, 200.0, 20.0, "");
        svg.add_line(10.0, 10.0, 200.0, 200.0, "");
        let result = svg.build();

        assert!(result.contains("<rect"));
        assert!(result.contains("<circle"));
        assert!(result.contains("<line"));
    }
}
