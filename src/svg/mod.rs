pub struct SvgBuilder {
    width: u32,
    height: u32,
    elements: Vec<String>,
    /// 追踪所有元素的最大 x, y 坐标（用于自动调整画布大小）
    max_x: f32,
    max_y: f32,
    background_color: String,
    custom_css: Option<String>,
}

impl SvgBuilder {
    pub fn new(width: u32, height: u32) -> Self {
        SvgBuilder {
            width,
            height,
            elements: Vec::new(),
            max_x: 0.0,
            max_y: 0.0,
            background_color: "white".to_string(),
            custom_css: None,
        }
    }

    pub fn with_background_color(mut self, color: &str) -> Self {
        self.background_color = color.to_string();
        self
    }

    pub fn with_custom_css(mut self, css: &str) -> Self {
        self.custom_css = Some(css.to_string());
        self
    }

    pub fn with_custom_css_if(mut self, css: Option<&str>) -> Self {
        self.custom_css = css.map(|s| s.to_string());
        self
    }

    fn track_bounds(&mut self, x: f32, y: f32) {
        if x > self.max_x {
            self.max_x = x;
        }
        if y > self.max_y {
            self.max_y = y;
        }
    }

    pub fn add_rect(&mut self, x: f32, y: f32, width: f32, height: f32, style: &str) {
        self.track_bounds(x + width, y + height);
        let element = format!(
            r#"  <rect x="{}" y="{}" width="{}" height="{}" style="{}"/>"#,
            x, y, width, height, style
        );
        self.elements.push(element);
    }

    pub fn add_circle(&mut self, cx: f32, cy: f32, r: f32, style: &str) {
        self.track_bounds(cx + r, cy + r);
        let element = format!(
            r#"  <circle cx="{}" cy="{}" r="{}" style="{}"/>"#,
            cx, cy, r, style
        );
        self.elements.push(element);
    }

    pub fn add_ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, style: &str) {
        self.track_bounds(cx + rx, cy + ry);
        let element = format!(
            r#"  <ellipse cx="{}" cy="{}" rx="{}" ry="{}" style="{}"/>"#,
            cx, cy, rx, ry, style
        );
        self.elements.push(element);
    }

    pub fn add_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, style: &str) {
        self.track_bounds(x1.max(x2), y1.max(y2));
        let element = format!(
            r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" style="{}"/>"#,
            x1, y1, x2, y2, style
        );
        self.elements.push(element);
    }

    pub fn add_text(&mut self, x: f32, y: f32, text: &str, style: &str) {
        self.track_bounds(x, y);
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
        self.track_bounds(x + 8.0, y);
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
        // 从 path 数据中粗略提取最大坐标
        for part in d.split_whitespace() {
            if let Ok(val) = part.parse::<f32>() {
                self.track_bounds(val, val);
            }
        }
        let element = format!(r#"  <path d="{}" style="{}"/>"#, d, style);
        self.elements.push(element);
    }

    pub fn add_polyline(&mut self, points: &[(f32, f32)], style: &str) {
        for &(x, y) in points {
            self.track_bounds(x, y);
        }
        let points_str: Vec<String> = points.iter().map(|(x, y)| format!("{},{}", x, y)).collect();
        let element = format!(
            r#"  <polyline points="{}" style="{}"/>"#,
            points_str.join(" "),
            style
        );
        self.elements.push(element);
    }

    /// Add a filled polygon from a list of (x, y) points.
    pub fn add_polygon(&mut self, points: &[(f32, f32)], style: &str) {
        for &(x, y) in points {
            self.track_bounds(x, y);
        }
        let points_str: Vec<String> = points.iter()
            .map(|(x, y)| format!("{},{}", x, y))
            .collect();
        let element = format!(
            r#"  <polygon points="{}" style="{}"/>"#,
            points_str.join(" "),
            style
        );
        self.elements.push(element);
    }

    pub fn build(self) -> String {
        let padding = 20.0;
        // 自动扩展画布以适应内容
        let width = self.width.max((self.max_x + padding) as u32);
        let height = self.height.max((self.max_y + padding) as u32);

        let mut svg = String::new();
        let extra_css = if let Some(ref css) = self.custom_css {
            format!("\n    /* Custom CSS */\n{}", css)
        } else {
            String::new()
        };

        svg.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
  <style>
    text {{ font-family: Arial, sans-serif; font-size: 14px; }}
  </style>{}
  <rect width="100%" height="100%" fill="{}"/>
"#,
            width, height, extra_css, self.background_color
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
    fn test_svg_builder_empty() {
        let svg = SvgBuilder::new(800, 600);
        let result = svg.build();

        assert!(result.contains("<svg"));
        assert!(result.contains("</svg>"));
        assert!(result.contains("800"));
        assert!(result.contains("600"));
        assert!(result.contains("xmlns"));
    }

    #[test]
    fn test_svg_ellipse_element() {
        let mut svg = SvgBuilder::new(800, 600);
        svg.add_ellipse(100.0, 100.0, 50.0, 30.0, "fill:blue");
        let result = svg.build();

        assert!(result.contains("<ellipse"));
        assert!(result.contains("cx=\"100\""));
        assert!(result.contains("cy=\"100\""));
        assert!(result.contains("rx=\"50\""));
        assert!(result.contains("ry=\"30\""));
    }

    #[test]
    fn test_svg_arrow_element() {
        let mut svg = SvgBuilder::new(800, 600);
        svg.add_arrow(100.0, 100.0);
        let result = svg.build();

        assert!(result.contains("<polygon"));
        assert!(result.contains("fill:black"));
    }

    #[test]
    fn test_svg_path_element() {
        let mut svg = SvgBuilder::new(800, 600);
        svg.add_path("M10 10 L100 100", "fill:none;stroke:black");
        let result = svg.build();

        assert!(result.contains("<path"));
        assert!(result.contains("M10 10 L100 100"));
    }

    #[test]
    fn test_svg_element_order() {
        let mut svg = SvgBuilder::new(800, 600);
        svg.add_rect(10.0, 10.0, 50.0, 50.0, "");
        svg.add_circle(100.0, 100.0, 20.0, "");
        svg.add_text(10.0, 10.0, "Order", "");
        let result = svg.build();

        let rect_pos = result.find("<rect").unwrap();
        let circle_pos = result.find("<circle").unwrap();
        let text_pos = result.find("Order").unwrap();

        assert!(rect_pos < circle_pos, "rect should come before circle");
        assert!(circle_pos < text_pos, "circle should come before text");
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

    #[test]
    fn test_svg_builder_background_color() {
        let svg = SvgBuilder::new(800, 600)
            .with_background_color("#f0f0f0");
        let result = svg.build();
        assert!(result.contains("#f0f0f0"), "Should include background color");
    }

    #[test]
    fn test_svg_builder_custom_css() {
        let svg = SvgBuilder::new(800, 600)
            .with_custom_css(".highlight { fill: red; }");
        let result = svg.build();
        assert!(result.contains("Custom CSS"), "Should include custom CSS section");
        assert!(result.contains(".highlight"), "Should include CSS content");
    }

    #[test]
    fn test_svg_builder_custom_css_if() {
        let svg = SvgBuilder::new(800, 600)
            .with_custom_css_if(Some(".test { }"));
        let result = svg.build();
        assert!(result.contains(".test"), "Should include conditional CSS");
    }

    #[test]
    fn test_svg_builder_custom_css_if_none() {
        let svg = SvgBuilder::new(800, 600)
            .with_custom_css_if(None);
        let result = svg.build();
        assert!(!result.contains("Custom CSS"), "Should not include CSS section when None");
    }

    #[test]
    fn test_svg_builder_multiple_adds() {
        let mut svg = SvgBuilder::new(800, 600);
        for i in 0..50 {
            svg.add_rect(i as f32 * 5.0, i as f32 * 5.0, 10.0, 10.0, "");
        }
        let result = svg.build();
        let rect_count = result.matches("<rect").count();
        assert_eq!(rect_count, 51, "Should have 50 added rects + 1 background rect");
    }

    #[test]
    fn test_svg_builder_polyline_with_points() {
        let mut svg = SvgBuilder::new(800, 600);
        svg.add_polyline(&[(10.0, 10.0), (100.0, 100.0), (200.0, 50.0)],
            "stroke:black;fill:none");
        let result = svg.build();
        assert!(result.contains("<polyline"), "Should have polyline element");
    }

    #[test]
    fn test_svg_builder_dimensions_auto_expand() {
        let mut svg = SvgBuilder::new(100, 100);
        svg.add_rect(0.0, 0.0, 500.0, 400.0, "");
        let result = svg.build();
        // Should auto-expand to fit content
        assert!(result.contains("width=\"520\""), "Width should expand for content");
        assert!(result.contains("height=\"420\""), "Height should expand for content");
    }

    #[test]
    fn test_svg_builder_xml_declaration() {
        let svg = SvgBuilder::new(800, 600);
        let result = svg.build();
        assert!(result.starts_with("<?xml"), "SVG should start with XML declaration");
    }

    #[test]
    fn test_svg_escape_xml_special_chars() {
        let escaped = escape_xml("A & B < C > D \"quoted\" 'single'");
        assert!(escaped.contains("&amp;"));
        assert!(escaped.contains("&lt;"));
        assert!(escaped.contains("&gt;"));
        assert!(escaped.contains("&quot;"));
        assert!(escaped.contains("&apos;"));
    }

    #[test]
    fn test_svg_all_element_types() {
        let mut svg = SvgBuilder::new(800, 600);
        svg.add_rect(10.0, 10.0, 50.0, 30.0, "fill:red");
        svg.add_circle(100.0, 50.0, 20.0, "fill:blue");
        svg.add_ellipse(160.0, 50.0, 30.0, 15.0, "fill:green");
        svg.add_line(10.0, 80.0, 200.0, 80.0, "stroke:black");
        svg.add_text(100.0, 120.0, "Hello SVG", "font-size:14px");
        svg.add_path("M10 130 L200 130", "stroke:black");
        svg.add_arrow(100.0, 200.0);
        let result = svg.build();
        assert!(result.contains("<rect"));
        assert!(result.contains("<circle"));
        assert!(result.contains("<ellipse"));
        assert!(result.contains("<line"));
        assert!(result.contains("<text"));
        assert!(result.contains("<path"));
        assert!(result.contains("<polygon"), "Arrow should create polygon");
    }

    #[test]
    fn test_svg_escape_xml_all_chars() {
        let escaped = escape_xml("A & B < C > D \"quoted\" 'single'");
        assert!(escaped.contains("&amp;"));
        assert!(escaped.contains("&lt;"));
        assert!(escaped.contains("&gt;"));
        assert!(escaped.contains("&quot;"));
        assert!(escaped.contains("&apos;"));
    }

    #[test]
    fn test_svg_multiple_text_special_chars() {
        let mut svg = SvgBuilder::new(800, 600);
        svg.add_text(50.0, 50.0, "A & B < C > D", "");
        svg.add_text(50.0, 80.0, "He said \"hello\"", "");
        svg.add_text(50.0, 110.0, "It's fine 'yes'", "");
        let result = svg.build();
        assert!(result.contains("&amp;"));
        assert!(result.contains("&lt;"));
        assert!(result.contains("&gt;"));
        assert!(result.contains("&quot;"));
        assert!(result.contains("&apos;"));
    }

    #[test]
    fn test_svg_builder_element_order_preserved() {
        let mut svg = SvgBuilder::new(800, 600);
        svg.add_rect(10.0, 10.0, 50.0, 50.0, "");
        svg.add_circle(200.0, 200.0, 30.0, "");
        svg.add_text(10.0, 10.0, "Last", "");
        let result = svg.build();
        let rect_pos = result.find("<rect").unwrap();
        let circle_pos = result.find("<circle").unwrap();
        let text_pos = result.find("Last").unwrap();
        assert!(rect_pos < circle_pos, "rect before circle");
        assert!(circle_pos < text_pos, "circle before text");
    }

    #[test]
    fn test_svg_builder_many_elements() {
        let mut svg = SvgBuilder::new(800, 600);
        for i in 0..30 {
            svg.add_rect(i as f32 * 10.0, i as f32 * 10.0, 8.0, 8.0, "fill:gray");
        }
        let result = svg.build();
        let rect_count = result.matches("<rect").count();
        assert_eq!(rect_count, 31, "Should have 30 rect elements plus background rect");
    }
}
