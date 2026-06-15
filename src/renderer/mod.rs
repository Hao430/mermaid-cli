use crate::parser::Diagram;
use crate::svg::SvgBuilder;

pub struct Renderer {
    width: u32,
    height: u32,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            width: 800,
            height: 600,
        }
    }

    pub fn with_dimensions(width: u32, height: u32) -> Self {
        Renderer { width, height }
    }

    pub fn render(&self, diagram: &Diagram) -> Result<String, String> {
        let mut svg = SvgBuilder::new(self.width, self.height);

        // 获取所有节点和边
        let nodes = diagram.get_nodes();
        let edges = diagram.get_edges();

        // 计算简单的布局（从上到下）
        let node_height = 50.0;
        let node_width = 80.0;
        let spacing = 100.0;

        // 在 SVG 中绘制节点
        for (idx, node_id) in nodes.iter().enumerate() {
            let y = 50.0 + (idx as f32) * spacing;
            let x = (self.width as f32 - node_width) / 2.0;

            // 添加节点矩形
            svg.add_rect(
                x,
                y,
                node_width,
                node_height,
                "fill:white;stroke:black;stroke-width:2",
            );

            // 添加节点文本
            svg.add_text(
                x + node_width / 2.0,
                y + node_height / 2.0,
                node_id,
                "text-anchor:middle;dominant-baseline:middle;font-family:Arial",
            );
        }

        // 绘制边
        for (from, to) in edges {
            if let (Some(from_idx), Some(to_idx)) = (
                nodes.iter().position(|n| n == &from),
                nodes.iter().position(|n| n == &to),
            ) {
                let x1 = (self.width as f32) / 2.0;
                let y1 = 50.0 + (from_idx as f32) * spacing + 50.0;

                let x2 = (self.width as f32) / 2.0;
                let y2 = 50.0 + (to_idx as f32) * spacing;

                svg.add_line(x1, y1, x2, y2, "stroke:black;stroke-width:2");
                svg.add_arrow(x2, y2);
            }
        }

        Ok(svg.build())
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{DiagramType, NodeShape, Statement};

    #[test]
    fn test_renderer_creates_svg() {
        let diagram = Diagram {
            diagram_type: DiagramType::Flowchart,
            statements: vec![
                Statement::NodeDef {
                    id: "A".to_string(),
                    label: Some("Start".to_string()),
                    shape: NodeShape::Rect,
                },
                Statement::NodeDef {
                    id: "B".to_string(),
                    label: Some("End".to_string()),
                    shape: NodeShape::Rect,
                },
            ],
        };

        let renderer = Renderer::new();
        let result = renderer.render(&diagram);
        assert!(result.is_ok());

        let svg = result.unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("A"));
        assert!(svg.contains("B"));
    }

    #[test]
    fn test_renderer_custom_dimensions() {
        let diagram = Diagram {
            diagram_type: DiagramType::Flowchart,
            statements: vec![Statement::NodeDef {
                id: "A".to_string(),
                label: None,
                shape: NodeShape::Rect,
            }],
        };

        let renderer = Renderer::with_dimensions(1024, 768);
        let result = renderer.render(&diagram);
        assert!(result.is_ok());

        let svg = result.unwrap();
        assert!(svg.contains("1024"));
        assert!(svg.contains("768"));
    }
}
