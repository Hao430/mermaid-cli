use crate::parser::{Diagram, NodeShape, Statement};
use crate::svg::SvgBuilder;
use std::collections::{HashMap, HashSet};

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

        // 构建节点信息：ID → (label, shape)
        let mut node_info: HashMap<String, (String, NodeShape)> = HashMap::new();
        for stmt in &diagram.statements {
            if let Statement::NodeDef { id, label, shape } = stmt {
                let display = label.clone().unwrap_or_else(|| id.clone());
                node_info.insert(id.clone(), (display, shape.clone()));
            }
        }
        // 添加只有 EdgeDef 引用但没有 NodeDef 的节点
        for stmt in &diagram.statements {
            if let Statement::EdgeDef { from, to, .. } = stmt {
                node_info
                    .entry(from.clone())
                    .or_insert_with(|| (from.clone(), NodeShape::Rect));
                node_info
                    .entry(to.clone())
                    .or_insert_with(|| (to.clone(), NodeShape::Rect));
            }
        }

        let edges = diagram.get_edges();
        let edges_with_labels = diagram.get_edges_with_labels();

        // 计算节点层级（拓扑排序）
        let node_ids: Vec<String> = node_info.keys().cloned().collect();
        let layers = compute_layers(&node_ids, &edges);

        // 布局参数
        let node_width = 100.0;
        let node_height = 50.0;
        let h_spacing = 60.0; // 水平间距
        let v_spacing = 100.0; // 垂直间距
        let padding = 50.0; // 画布边距

        let is_horizontal = diagram.direction.as_deref() == Some("LR")
            || diagram.direction.as_deref() == Some("RL");

        // 计算每层节点的位置
        let mut node_positions: HashMap<String, (f32, f32)> = HashMap::new();

        for (layer_idx, layer_nodes) in layers.iter().enumerate() {
            let count = layer_nodes.len() as f32;
            let total_width = count * node_width + (count - 1.0) * h_spacing;
            let start_x = (self.width as f32 - total_width) / 2.0;

            for (pos_idx, node_id) in layer_nodes.iter().enumerate() {
                let (x, y) = if is_horizontal {
                    // 从左到右布局：层 → X 轴
                    let x = padding + layer_idx as f32 * (node_width + v_spacing);
                    let y = start_x + pos_idx as f32 * (node_height + h_spacing);
                    (x, y)
                } else {
                    // 从上到下布局（默认）：层 → Y 轴
                    let x = start_x + pos_idx as f32 * (node_width + h_spacing);
                    let y = padding + layer_idx as f32 * (node_height + v_spacing);
                    (x, y)
                };
                node_positions.insert(node_id.clone(), (x, y));
            }
        }

        // 绘制边（在节点之前，以便边在节点下方）
        for (from, to, label) in &edges_with_labels {
            if let (Some(&(x1, y1)), Some(&(x2, y2))) =
                (node_positions.get(from), node_positions.get(to))
            {
                let cx1 = x1 + node_width / 2.0;
                let cy1 = y1 + node_height / 2.0;
                let cx2 = x2 + node_width / 2.0;
                let cy2 = y2 + node_height / 2.0;

                svg.add_line(cx1, cy1, cx2, cy2, "stroke:black;stroke-width:2");
                svg.add_arrow(cx2, cy2);

                // 绘制边标签
                if let Some(edge_label) = label {
                    let label_x = (cx1 + cx2) / 2.0 - 20.0;
                    let label_y = (cy1 + cy2) / 2.0 - 8.0;
                    svg.add_rect(label_x, label_y, 40.0, 16.0, "fill:white;stroke:none;");
                    svg.add_text(
                        label_x + 20.0,
                        label_y + 8.0,
                        edge_label,
                        "text-anchor:middle;dominant-baseline:middle;font-size:10px;font-family:Arial",
                    );
                }
            }
        }

        // 绘制 subgraph 背景
        for sg in &diagram.subgraphs {
            let sg_stmts = &sg.statements;
            if sg_stmts.is_empty() {
                continue;
            }
            // 收集 subgraph 内所有节点的位置来计算边界框
            let mut min_x = self.width as f32;
            let mut min_y = self.height as f32;
            let mut max_x = 0.0_f32;
            let mut max_y = 0.0_f32;
            let mut has_node = false;

            for stmt in sg_stmts {
                if let Statement::NodeDef { id, .. } = stmt {
                    if let Some(&(nx, ny)) = node_positions.get(id) {
                        min_x = min_x.min(nx);
                        min_y = min_y.min(ny);
                        max_x = max_x.max(nx + node_width);
                        max_y = max_y.max(ny + node_height);
                        has_node = true;
                    }
                }
            }

            if has_node {
                let sg_padding = 10.0;
                let sg_x = min_x - sg_padding;
                let sg_y = min_y - sg_padding;
                let sg_w = max_x - min_x + 2.0 * sg_padding;
                let sg_h = max_y - min_y + 2.0 * sg_padding;

                svg.add_rect(
                    sg_x,
                    sg_y,
                    sg_w,
                    sg_h,
                    "fill:#f0f0f0;stroke:#999;stroke-width:1;rx:5;",
                );
                // subgraph 标题
                if let Some(title) = &sg.title {
                    svg.add_text(
                        sg_x + 10.0,
                        sg_y + 15.0,
                        title,
                        "font-size:12px;font-weight:bold;font-family:Arial",
                    );
                }
            }
        }

        // 绘制节点
        for (node_id, (display_label, shape)) in &node_info {
            if let Some(&(x, y)) = node_positions.get(node_id) {
                let cx = x + node_width / 2.0;
                let cy = y + node_height / 2.0;
                let style = "fill:white;stroke:black;stroke-width:2";

                match shape {
                    NodeShape::Rect => {
                        svg.add_rect(x, y, node_width, node_height, style);
                    }
                    NodeShape::Circle => {
                        svg.add_rect(x, y, node_width, node_height, &format!("{};rx:10", style));
                    }
                    NodeShape::Diamond => {
                        let mid_x = cx;
                        let mid_y = cy;
                        let hw = node_width / 2.0;
                        let hh = node_height / 2.0;
                        svg.add_path(
                            &format!(
                                "M {} {} L {} {} L {} {} L {} {} Z",
                                mid_x,
                                mid_y - hh,
                                mid_x + hw,
                                mid_y,
                                mid_x,
                                mid_y + hh,
                                mid_x - hw,
                                mid_y
                            ),
                            style,
                        );
                    }
                    NodeShape::Rounded => {
                        svg.add_rect(x, y, node_width, node_height, &format!("{};rx:20", style));
                    }
                    NodeShape::Subroutine => {
                        svg.add_rect(x, y, node_width, node_height, style);
                        let inset = 6.0;
                        svg.add_rect(
                            x + inset,
                            y + inset,
                            node_width - 2.0 * inset,
                            node_height - 2.0 * inset,
                            "fill:white;stroke:black;stroke-width:1",
                        );
                    }
                    NodeShape::Cylinder => {
                        svg.add_rect(x, y + 10.0, node_width, node_height - 10.0, style);
                        svg.add_ellipse(cx, y + 10.0, node_width / 2.0, 10.0, style);
                    }
                    NodeShape::DoubleCircle => {
                        let outer_r = node_height / 2.0;
                        let inner_r = outer_r - 5.0;
                        svg.add_circle(cx, cy, outer_r, style);
                        svg.add_circle(cx, cy, inner_r, "fill:white;stroke:black;stroke-width:1");
                    }
                    NodeShape::Flag => {
                        svg.add_rect(x + 15.0, y, node_width - 15.0, node_height, style);
                        svg.add_path(
                            &format!(
                                "M {} {} L {} {} L {} {} Z",
                                x + 15.0,
                                y + node_height / 2.0,
                                x,
                                y,
                                x,
                                y + node_height
                            ),
                            style,
                        );
                    }
                }

                // 文本
                svg.add_text(
                    cx,
                    cy,
                    display_label,
                    "text-anchor:middle;dominant-baseline:middle;font-family:Arial",
                );
            }
        }

        Ok(svg.build())
    }
}

/// 使用 Kahn 算法进行拓扑排序分层
///
/// 每次移除入度为 0 的节点，记录为一层，然后更新剩余节点入度。
/// 返回 `Vec<Vec<String>>`，每个内部 Vec 是一层中的节点 ID。
fn compute_layers(node_ids: &[String], edges: &[(String, String)]) -> Vec<Vec<String>> {
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut all_nodes: HashSet<&str> = HashSet::new();

    for id in node_ids {
        in_degree.entry(id).or_insert(0);
        adjacency.entry(id).or_default();
        all_nodes.insert(id.as_str());
    }

    for (from, to) in edges {
        if all_nodes.contains(from.as_str()) && all_nodes.contains(to.as_str()) {
            adjacency
                .entry(from.as_str())
                .or_default()
                .push(to.as_str());
            *in_degree.entry(to.as_str()).or_insert(0) += 1;
        }
    }

    let mut remaining: HashSet<&str> = all_nodes.clone();
    let mut layers: Vec<Vec<String>> = Vec::new();

    while !remaining.is_empty() {
        // 找出当前入度为 0 的节点 → 组成一层
        let zero_in: Vec<&str> = remaining
            .iter()
            .filter(|id| *in_degree.get(*id).unwrap_or(&0) == 0)
            .cloned()
            .collect();

        if zero_in.is_empty() {
            // 有环：将所有剩余节点放入最后一层
            let layer: Vec<String> = remaining.iter().map(|s| s.to_string()).collect();
            layers.push(layer);
            break;
        }

        let layer: Vec<String> = zero_in.iter().map(|s| s.to_string()).collect();
        layers.push(layer);

        // 移除这些节点并更新邻接节点的入度
        for id in &zero_in {
            remaining.remove(id);
            if let Some(neighbors) = adjacency.get(id) {
                for next in neighbors {
                    if remaining.contains(next) {
                        if let Some(deg) = in_degree.get_mut(next) {
                            *deg = deg.saturating_sub(1);
                        }
                    }
                }
            }
        }
    }

    layers
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{DiagramType, Statement};

    #[test]
    fn test_renderer_creates_svg() {
        let diagram = Diagram {
            diagram_type: DiagramType::Flowchart,
            direction: None,
            subgraphs: vec![],
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
        assert!(svg.contains("Start"), "SVG should contain label 'Start'");
        assert!(svg.contains("End"), "SVG should contain label 'End'");
    }

    #[test]
    fn test_renderer_custom_dimensions() {
        let diagram = Diagram {
            diagram_type: DiagramType::Flowchart,
            direction: None,
            subgraphs: vec![],
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

    #[test]
    fn test_renderer_displays_label() {
        let diagram = Diagram {
            diagram_type: DiagramType::Flowchart,
            direction: None,
            subgraphs: vec![],
            statements: vec![Statement::NodeDef {
                id: "A".to_string(),
                label: Some("Start".to_string()),
                shape: NodeShape::Rect,
            }],
        };

        let renderer = Renderer::new();
        let result = renderer.render(&diagram).unwrap();
        assert!(result.contains("Start"), "SVG should contain label 'Start'");
        assert!(
            !result.contains(">A<"),
            "SVG should not display node ID 'A'"
        );
    }

    #[test]
    fn test_renderer_falls_back_to_id() {
        let diagram = Diagram {
            diagram_type: DiagramType::Flowchart,
            direction: None,
            subgraphs: vec![],
            statements: vec![Statement::NodeDef {
                id: "A".to_string(),
                label: None,
                shape: NodeShape::Rect,
            }],
        };

        let renderer = Renderer::new();
        let result = renderer.render(&diagram).unwrap();
        assert!(result.contains("A"), "SVG should display node ID 'A'");
    }

    #[test]
    fn test_renderer_renders_diamond() {
        let diagram = Diagram {
            diagram_type: DiagramType::Flowchart,
            direction: None,
            subgraphs: vec![],
            statements: vec![Statement::NodeDef {
                id: "A".to_string(),
                label: Some("Decision".to_string()),
                shape: NodeShape::Diamond,
            }],
        };

        let renderer = Renderer::new();
        let svg = renderer.render(&diagram).unwrap();
        assert!(svg.contains("path"), "Diamond should use path element");
        assert!(svg.contains("Decision"));
    }

    #[test]
    fn test_renderer_renders_rounded() {
        let diagram = Diagram {
            diagram_type: DiagramType::Flowchart,
            direction: None,
            subgraphs: vec![],
            statements: vec![Statement::NodeDef {
                id: "A".to_string(),
                label: Some("Rounded".to_string()),
                shape: NodeShape::Circle,
            }],
        };

        let renderer = Renderer::new();
        let svg = renderer.render(&diagram).unwrap();
        assert!(svg.contains("rx:10"), "Rounded should have rx attribute");
        assert!(svg.contains("Rounded"));
    }

    #[test]
    fn test_renderer_renders_cylinder() {
        let diagram = Diagram {
            diagram_type: DiagramType::Flowchart,
            direction: None,
            subgraphs: vec![],
            statements: vec![Statement::NodeDef {
                id: "A".to_string(),
                label: Some("DB".to_string()),
                shape: NodeShape::Cylinder,
            }],
        };

        let renderer = Renderer::new();
        let svg = renderer.render(&diagram).unwrap();
        assert!(
            svg.contains("ellipse"),
            "Cylinder should use ellipse element"
        );
        assert!(svg.contains("DB"));
    }

    #[test]
    fn test_renderer_renders_double_circle() {
        let diagram = Diagram {
            diagram_type: DiagramType::Flowchart,
            direction: None,
            subgraphs: vec![],
            statements: vec![Statement::NodeDef {
                id: "A".to_string(),
                label: Some("Start".to_string()),
                shape: NodeShape::DoubleCircle,
            }],
        };

        let renderer = Renderer::new();
        let svg = renderer.render(&diagram).unwrap();
        // Should have two circle elements
        let circle_count = svg.matches("circle").count();
        assert!(
            circle_count >= 2,
            "DoubleCircle should have at least 2 circles, found {}",
            circle_count
        );
        assert!(svg.contains("Start"));
    }

    #[test]
    fn test_renderer_renders_subroutine() {
        let diagram = Diagram {
            diagram_type: DiagramType::Flowchart,
            direction: None,
            subgraphs: vec![],
            statements: vec![Statement::NodeDef {
                id: "A".to_string(),
                label: Some("Proc".to_string()),
                shape: NodeShape::Subroutine,
            }],
        };

        let renderer = Renderer::new();
        let svg = renderer.render(&diagram).unwrap();
        // Should have two rect elements (outer + inner)
        let rect_count = svg.matches("rect").count();
        assert!(
            rect_count >= 2,
            "Subroutine should have at least 2 rects, found {}",
            rect_count
        );
        assert!(svg.contains("Proc"));
    }

    #[test]
    fn test_renderer_renders_flag() {
        let diagram = Diagram {
            diagram_type: DiagramType::Flowchart,
            direction: None,
            subgraphs: vec![],
            statements: vec![Statement::NodeDef {
                id: "A".to_string(),
                label: Some("Flag".to_string()),
                shape: NodeShape::Flag,
            }],
        };

        let renderer = Renderer::new();
        let svg = renderer.render(&diagram).unwrap();
        assert!(svg.contains("path"), "Flag should use path element");
        assert!(svg.contains("Flag"));
    }
}
