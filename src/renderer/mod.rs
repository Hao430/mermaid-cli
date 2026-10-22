use crate::parser::{ArrowType, ClassVisibility, DiagramType, NotePosition};
use crate::parser::{Diagram, NodeShape, Statement, Subgraph};
use crate::svg::SvgBuilder;
use std::collections::{HashMap, HashSet};

pub struct Renderer {
    width: u32,
    height: u32,
    theme: String,
    background_color: String,
    scale: f32,
    custom_css: Option<String>,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            width: 800,
            height: 600,
            theme: "default".to_string(),
            background_color: "white".to_string(),
            scale: 1.0,
            custom_css: None,
        }
    }

    pub fn with_dimensions(width: u32, height: u32) -> Self {
        Renderer {
            width,
            height,
            theme: "default".to_string(),
            background_color: "white".to_string(),
            scale: 1.0,
            custom_css: None,
        }
    }

    pub fn with_theme(mut self, theme: &str) -> Self {
        self.theme = theme.to_string();
        self
    }

    pub fn with_background_color(mut self, color: &str) -> Self {
        self.background_color = color.to_string();
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_custom_css(mut self, css: &str) -> Self {
        self.custom_css = Some(css.to_string());
        self
    }

    fn setup_svg_builder(&self) -> SvgBuilder {
        let mut svg =
            SvgBuilder::new(self.width, self.height).with_background_color(&self.background_color);
        if let Some(ref css) = self.custom_css {
            svg = svg.with_custom_css(css);
        }
        svg
    }

    pub fn render(&self, diagram: &Diagram) -> Result<String, String> {
        match diagram.diagram_type {
            DiagramType::Sequence => Ok(self.render_sequence(diagram)),
            DiagramType::Pie => Ok(self.render_pie(diagram)),
            DiagramType::Class => Ok(self.render_class(diagram)),
            DiagramType::State => Ok(self.render_state(diagram)),
            DiagramType::Er => Ok(self.render_er(diagram)),
            DiagramType::Gantt => Ok(self.render_gantt(diagram)),
            DiagramType::Mindmap => Ok(self.render_mindmap(diagram)),
            DiagramType::GitGraph => Ok(self.render_gitgraph(diagram)),
            DiagramType::Timeline => Ok(self.render_timeline(diagram)),
            DiagramType::Journey => Ok(self.render_journey(diagram)),
            DiagramType::Kanban => Ok(self.render_kanban(diagram)),
            DiagramType::Venn => Ok(self.render_venn(diagram)),
            DiagramType::Packet => Ok(self.render_packet(diagram)),
            DiagramType::Radar => Ok(self.render_radar(diagram)),
            DiagramType::Ishikawa => Ok(self.render_ishikawa(diagram)),
            DiagramType::Quadrant => Ok(self.render_quadrant(diagram)),
            DiagramType::ZenUml => Ok(self.render_zenuml(diagram)),
            DiagramType::Requirement => Ok(self.render_requirement(diagram)),
            DiagramType::Block => Ok(self.render_block(diagram)),
            DiagramType::C4 => Ok(self.render_c4(diagram)),
            DiagramType::Architecture => Ok(self.render_architecture(diagram)),
            DiagramType::XyChart => Ok(self.render_xychart(diagram)),
            DiagramType::Sankey => Ok(self.render_sankey(diagram)),
            DiagramType::Treemap => Ok(self.render_treemap(diagram)),
            _ => self.render_flowchart(diagram),
        }
    }

    fn render_flowchart(&self, diagram: &Diagram) -> Result<String, String> {
        let mut svg = self.setup_svg_builder();

        // 构建节点信息：ID → (label, shape) 以及样式定义
        let mut node_info: HashMap<String, (String, NodeShape)> = HashMap::new();
        let mut class_defs: HashMap<String, String> = HashMap::new();
        let mut node_classes: HashMap<String, String> = HashMap::new();

        let mut process_stmt = |stmt: &Statement| match stmt {
            Statement::NodeDef { id, label, shape } => {
                if let Some(existing) = node_info.get_mut(id) {
                    if let Some(lbl) = label {
                        existing.0 = lbl.clone();
                    }
                    if *shape != NodeShape::Rect {
                        existing.1 = shape.clone();
                    }
                } else {
                    let display = label.clone().unwrap_or_else(|| id.clone());
                    node_info.insert(id.clone(), (display, shape.clone()));
                }
            }
            Statement::EdgeDef { from, to, .. } => {
                node_info
                    .entry(from.clone())
                    .or_insert_with(|| (from.clone(), NodeShape::Rect));
                node_info
                    .entry(to.clone())
                    .or_insert_with(|| (to.clone(), NodeShape::Rect));
            }
            Statement::StyleClassDef { name, styles } => {
                class_defs.insert(name.clone(), styles.clone());
            }
            Statement::ClassAssign { nodes, class_name } => {
                for node in nodes {
                    node_classes.insert(node.clone(), class_name.clone());
                }
            }
            _ => {}
        };

        for stmt in &diagram.statements {
            process_stmt(stmt);
        }
        for sg in &diagram.subgraphs {
            for stmt in &sg.statements {
                process_stmt(stmt);
            }
        }

        let edges = diagram.get_edges();
        let edges_with_labels = diagram.get_edges_with_labels();

        // ponytail: use tree-aware layout for org charts; fallback to topological sort for DAGs
        let node_ids: Vec<String> = node_info.keys().cloned().collect();
        let layers = compute_tree_layers(&node_ids, &edges, &diagram.subgraphs);

        // 布局参数（紧凑树形布局以最小化连线交叉）
        let node_width = 130.0;
        let node_height = 65.0;
        let h_spacing = 40.0;
        let v_spacing = 80.0; // 减小垂直间距让线更短
        let padding = 40.0;

        // ponytail: calculate required canvas size based on content, not viewport
        let max_nodes_in_layer = layers.iter().map(|l| l.len()).max().unwrap_or(1) as f32;
        let num_layers = layers.len() as f32;

        let required_width = max_nodes_in_layer * node_width
            + (max_nodes_in_layer - 1.0) * h_spacing
            + 2.0 * padding;
        let required_height =
            num_layers * node_height + (num_layers - 1.0) * v_spacing + 2.0 * padding;

        let effective_width = required_width.max(self.width as f32) as u32;
        let effective_height = required_height.max(self.height as f32) as u32;

        // ponytail: Barycenter method - sort nodes by parent positions to minimize crossing
        let mut parent_map: HashMap<String, String> = HashMap::new();
        let mut children_map: HashMap<String, Vec<String>> = HashMap::new();

        for (from, to) in &edges {
            parent_map.insert(to.clone(), from.clone());
            children_map
                .entry(from.clone())
                .or_default()
                .push(to.clone());
        }

        let mut node_positions: HashMap<String, (f32, f32)> = HashMap::new();

        for (layer_idx, layer_nodes) in layers.iter().enumerate() {
            let y = padding + layer_idx as f32 * (node_height + v_spacing);

            // Sort nodes in this layer by parent positions (barycenter method)
            let mut sorted_nodes = layer_nodes.clone();
            sorted_nodes.sort_by(|a, b| {
                let a_parent_x = parent_map
                    .get(a)
                    .and_then(|p| node_positions.get(p).map(|&(x, _)| x))
                    .unwrap_or(f32::INFINITY);

                let b_parent_x = parent_map
                    .get(b)
                    .and_then(|p| node_positions.get(p).map(|&(x, _)| x))
                    .unwrap_or(f32::INFINITY);

                a_parent_x
                    .partial_cmp(&b_parent_x)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let count = sorted_nodes.len() as f32;
            let total_width = count * node_width + (count - 1.0) * h_spacing;
            let start_x = (effective_width as f32 - total_width) / 2.0;

            for (idx, node_id) in sorted_nodes.iter().enumerate() {
                let x = start_x + idx as f32 * (node_width + h_spacing);
                node_positions.insert(node_id.clone(), (x, y));
            }
        }

        // 更新 SVG 构建器的尺寸
        svg.set_dimensions(effective_width, effective_height);

        // 绘制 subgraph 背景（在边和节点之前绘制）
        for sg in &diagram.subgraphs {
            let mut sg_node_ids = HashSet::new();
            for stmt in &sg.statements {
                match stmt {
                    Statement::NodeDef { id, .. } => {
                        sg_node_ids.insert(id.clone());
                    }
                    Statement::EdgeDef { from, to, .. } => {
                        sg_node_ids.insert(from.clone());
                        sg_node_ids.insert(to.clone());
                    }
                    _ => {}
                }
            }

            let mut min_x = f32::INFINITY;
            let mut min_y = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;
            let mut max_y = f32::NEG_INFINITY;
            let mut has_node = false;

            for id in &sg_node_ids {
                if let Some(&(nx, ny)) = node_positions.get(id) {
                    min_x = min_x.min(nx);
                    min_y = min_y.min(ny);
                    max_x = max_x.max(nx + node_width);
                    max_y = max_y.max(ny + node_height);
                    has_node = true;
                }
            }

            if has_node {
                let padding_x = 20.0;
                let padding_bottom = 20.0;
                let padding_top = if sg.title.is_some() { 36.0 } else { 20.0 };

                let sg_x = min_x - padding_x;
                let sg_y = min_y - padding_top;
                let sg_w = max_x - min_x + 2.0 * padding_x;
                let sg_h = max_y - min_y + padding_top + padding_bottom;

                svg.add_rect(
                    sg_x,
                    sg_y,
                    sg_w,
                    sg_h,
                    "fill:#f8fafc;stroke:#cbd5e1;stroke-width:1.5;stroke-dasharray:4;rx:8;ry:8",
                );

                if let Some(title) = &sg.title {
                    svg.add_text(
                        sg_x + 15.0,
                        sg_y + 18.0,
                        title,
                        "text-anchor:start;dominant-baseline:middle;font-size:13px;font-weight:bold;fill:#475569;font-family:Arial, sans-serif",
                    );
                }
            }
        }

        // 绘制边
        for (from, to, label) in &edges_with_labels {
            if let (Some(&(x1, y1)), Some(&(x2, y2))) =
                (node_positions.get(from), node_positions.get(to))
            {
                let cx1 = x1 + node_width / 2.0;
                let cy1 = y1 + node_height / 2.0;
                let cx2 = x2 + node_width / 2.0;
                let cy2 = y2 + node_height / 2.0;

                svg.add_line(cx1, cy1, cx2, cy2, "stroke:#475569;stroke-width:1.5");
                svg.add_arrow(cx2, cy2);

                if let Some(edge_label) = label {
                    let text_chars = edge_label.chars().count() as f32;
                    let lw = (text_chars * 12.0 + 14.0).max(36.0);
                    let label_x = (cx1 + cx2) / 2.0 - lw / 2.0;
                    let label_y = (cy1 + cy2) / 2.0 - 9.0;
                    svg.add_rect(
                        label_x,
                        label_y,
                        lw,
                        18.0,
                        "fill:#ffffff;stroke:#cbd5e1;stroke-width:1;rx:4;ry:4",
                    );
                    svg.add_text(
                        (cx1 + cx2) / 2.0,
                        (cy1 + cy2) / 2.0 + 4.0,
                        edge_label,
                        "text-anchor:middle;dominant-baseline:middle;font-size:11px;fill:#475569;font-family:Arial, sans-serif",
                    );
                }
            }
        }

        // 绘制节点
        for (node_id, (display_label, shape)) in &node_info {
            if let Some(&(x, y)) = node_positions.get(node_id) {
                let cx = x + node_width / 2.0;
                let cy = y + node_height / 2.0;

                let class_name = node_classes.get(node_id).map(|s| s.as_str()).or_else(|| {
                    if class_defs.contains_key("default") {
                        Some("default")
                    } else {
                        None
                    }
                });

                let (shape_style, text_fill) = if let Some(cname) = class_name {
                    if let Some(style_decl) = class_defs.get(cname) {
                        let (props, others) = parse_class_style(style_decl);
                        let fill = props
                            .get("fill")
                            .cloned()
                            .unwrap_or_else(|| "#ffffff".to_string());
                        let stroke = props
                            .get("stroke")
                            .cloned()
                            .unwrap_or_else(|| "#334155".to_string());
                        let stroke_width = props
                            .get("stroke-width")
                            .cloned()
                            .unwrap_or_else(|| "1.5".to_string());
                        let color = props
                            .get("color")
                            .cloned()
                            .unwrap_or_else(|| "#0f172a".to_string());
                        let mut s = format!(
                            "fill:{};stroke:{};stroke-width:{}",
                            fill, stroke, stroke_width
                        );
                        if !others.is_empty() {
                            s.push(';');
                            s.push_str(&others.join(";"));
                        }
                        (s, color)
                    } else {
                        (
                            "fill:#ffffff;stroke:#334155;stroke-width:1.5".to_string(),
                            "#0f172a".to_string(),
                        )
                    }
                } else {
                    (
                        "fill:#ffffff;stroke:#334155;stroke-width:1.5".to_string(),
                        "#0f172a".to_string(),
                    )
                };

                match shape {
                    NodeShape::Rect => {
                        svg.add_rect(
                            x,
                            y,
                            node_width,
                            node_height,
                            &format!("{};rx:6;ry:6", shape_style),
                        );
                    }
                    NodeShape::Circle => {
                        svg.add_rect(
                            x,
                            y,
                            node_width,
                            node_height,
                            &format!("{};rx:10;ry:10", shape_style),
                        );
                    }
                    NodeShape::Diamond => {
                        svg.add_path(
                            &format!(
                                "M {} {} L {} {} L {} {} L {} {} Z",
                                cx,
                                cy - node_height / 2.0,
                                cx + node_width / 2.0,
                                cy,
                                cx,
                                cy + node_height / 2.0,
                                cx - node_width / 2.0,
                                cy
                            ),
                            &shape_style,
                        );
                    }
                    NodeShape::Rounded => {
                        let rx = (node_height / 2.0).min(32.5);
                        svg.add_rect(
                            x,
                            y,
                            node_width,
                            node_height,
                            &format!("{};rx:{};ry:{}", shape_style, rx, rx),
                        );
                    }
                    NodeShape::Subroutine => {
                        svg.add_rect(
                            x,
                            y,
                            node_width,
                            node_height,
                            &format!("{};rx:4", shape_style),
                        );
                        let inset = 6.0;
                        svg.add_rect(
                            x + inset,
                            y + inset,
                            node_width - 2.0 * inset,
                            node_height - 2.0 * inset,
                            "fill:#ffffff;stroke:#94a3b8;stroke-width:1",
                        );
                    }
                    NodeShape::Cylinder => {
                        svg.add_rect(x, y + 10.0, node_width, node_height - 10.0, &shape_style);
                        svg.add_ellipse(cx, y + 10.0, node_width / 2.0, 10.0, &shape_style);
                    }
                    NodeShape::DoubleCircle => {
                        let outer_r = node_height / 2.0;
                        let inner_r = outer_r - 5.0;
                        svg.add_circle(cx, cy, outer_r, &shape_style);
                        svg.add_circle(
                            cx,
                            cy,
                            inner_r,
                            "fill:#ffffff;stroke:#94a3b8;stroke-width:1",
                        );
                    }
                    NodeShape::Flag => {
                        svg.add_rect(x + 15.0, y, node_width - 15.0, node_height, &shape_style);
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
                            &shape_style,
                        );
                    }
                }

                svg.add_text(
                    cx,
                    cy,
                    display_label,
                    &format!(
                        "text-anchor:middle;dominant-baseline:middle;font-size:13px;font-weight:500;fill:{};font-family:Arial, sans-serif",
                        text_fill
                    ),
                );
            }
        }

        Ok(svg.build())
    }

    fn render_sequence(&self, diagram: &Diagram) -> String {
        let mut svg = self.setup_svg_builder();

        let box_width = 120.0;
        let box_height = 40.0;
        let spacing = 100.0;
        let top_margin = 40.0;
        let start_x = 60.0;

        // Collect participants and display names
        let mut participants: Vec<(String, String)> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        for stmt in &diagram.statements {
            match stmt {
                Statement::Participant { id, label } => {
                    if seen.insert(id.clone()) {
                        let display = label.clone().unwrap_or_else(|| id.clone());
                        participants.push((id.clone(), display));
                    }
                }
                Statement::Message { from, to, .. } => {
                    if seen.insert(from.clone()) {
                        participants.push((from.clone(), from.clone()));
                    }
                    if seen.insert(to.clone()) {
                        participants.push((to.clone(), to.clone()));
                    }
                }
                Statement::Note { target, .. } => {
                    if seen.insert(target.clone()) {
                        participants.push((target.clone(), target.clone()));
                    }
                }
                Statement::Activate { participant } | Statement::Deactivate { participant } => {
                    if seen.insert(participant.clone()) {
                        participants.push((participant.clone(), participant.clone()));
                    }
                }
                Statement::Block { statements, .. } => {
                    for s in flatten_seq_stmts(statements) {
                        if let Statement::Message { from, to, .. } = &s {
                            if seen.insert(from.clone()) {
                                participants.push((from.clone(), from.clone()));
                            }
                            if seen.insert(to.clone()) {
                                participants.push((to.clone(), to.clone()));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        let centers: Vec<(String, f32)> = participants
            .iter()
            .enumerate()
            .map(|(i, (id, _))| {
                (
                    id.clone(),
                    start_x + box_width / 2.0 + i as f32 * (box_width + spacing),
                )
            })
            .collect();

        let center_map: HashMap<&str, f32> =
            centers.iter().map(|(id, x)| (id.as_str(), *x)).collect();

        let lifeline_top = top_margin + box_height + 20.0;
        let mut y_pos = lifeline_top + 30.0;
        let line_height = 45.0;
        let mut activations: HashMap<String, f32> = HashMap::new();

        // Process sequence statements with proper block handling
        process_seq_stmts(
            &mut svg,
            &center_map,
            &diagram.statements,
            &mut y_pos,
            line_height,
            &mut activations,
            box_width,
            start_x,
        );

        // Draw lifelines (dashed vertical lines)
        for (_, cx_val) in &centers {
            svg.add_line(
                *cx_val,
                lifeline_top,
                *cx_val,
                y_pos + 10.0,
                "stroke:#94a3b8;stroke-width:1.5;stroke-dasharray:6,4",
            );
        }

        // Draw participant boxes on top
        for ((_, display_name), (id, cx)) in participants.iter().zip(centers.iter()) {
            let bx = cx - box_width / 2.0;
            svg.add_rect(
                bx,
                top_margin,
                box_width,
                box_height,
                "fill:#f8fafc;stroke:#64748b;stroke-width:1.5;rx:6",
            );
            svg.add_text(
                *cx,
                top_margin + box_height / 2.0 + 1.0,
                display_name,
                "text-anchor:middle;dominant-baseline:middle;font-size:13px;font-weight:600;fill:#0f172a;font-family:Arial, sans-serif",
            );
            if *display_name != *id {
                svg.add_text(
                    *cx,
                    top_margin + box_height + 12.0,
                    id,
                    "text-anchor:middle;font-size:10px;fill:#64748b;font-family:Arial, sans-serif",
                );
            }
        }

        svg.build()
    }

    fn render_pie(&self, diagram: &Diagram) -> String {
        let mut svg = self.setup_svg_builder();

        let mut slices: Vec<(String, f64)> = Vec::new();
        for stmt in &diagram.statements {
            if let Statement::PieSlice { label, value } = stmt {
                slices.push((label.clone(), *value));
            }
        }

        let total: f64 = slices.iter().map(|(_, v)| v).sum();

        if let Some(ref title) = diagram.title {
            svg.add_text(
                self.width as f32 / 2.0,
                30.0,
                title,
                "text-anchor:middle;font-size:16px;font-weight:bold;font-family:Arial",
            );
        }

        let cx = self.width as f32 / 2.0;
        let cy = if diagram.title.is_some() {
            260.0
        } else {
            250.0
        };
        let radius = 180.0;

        let colors = [
            "#ff6b6b", "#4ecdc4", "#45b7d1", "#96ceb4", "#ffeaa7", "#dfe6e9", "#a29bfe", "#fd79a8",
            "#00cec9", "#fdcb6e",
        ];

        let mut start_angle = -90.0_f64;

        for (i, (label, value)) in slices.iter().enumerate() {
            if total <= 0.0 {
                continue;
            }
            let fraction = value / total;
            let end_angle = start_angle + 360.0 * fraction;

            let start_rad = start_angle.to_radians();
            let end_rad = end_angle.to_radians();

            let x1 = cx + radius * start_rad.cos() as f32;
            let y1 = cy + radius * start_rad.sin() as f32;
            let x2 = cx + radius * end_rad.cos() as f32;
            let y2 = cy + radius * end_rad.sin() as f32;

            let large_arc = if fraction > 0.5 { 1 } else { 0 };
            let color = colors[i % colors.len()];

            let d = format!(
                "M {} {} L {} {} A {} {} 0 {} 1 {} {} Z",
                cx, cy, x1, y1, radius, radius, large_arc, x2, y2
            );
            svg.add_path(&d, &format!("fill:{};stroke:white;stroke-width:2", color));

            let mid_angle = start_angle + 360.0 * fraction / 2.0;
            let mid_rad = mid_angle.to_radians();
            let label_r = radius + 25.0;
            let lx = cx + label_r * mid_rad.cos() as f32;
            let ly = cy + label_r * mid_rad.sin() as f32;
            let pct = (fraction * 100.0).round() as u32;
            svg.add_text(
                lx,
                ly,
                &format!("{} {}%", label, pct),
                "text-anchor:middle;dominant-baseline:middle;font-size:12px;font-family:Arial",
            );

            start_angle = end_angle;
        }

        svg.build()
    }

    #[allow(clippy::type_complexity)]
    fn render_class(&self, diagram: &Diagram) -> String {
        let mut svg = self.setup_svg_builder();

        let mut classes: HashMap<String, (Option<String>, Vec<(String, String, String)>)> =
            HashMap::new();

        for stmt in &diagram.statements {
            match stmt {
                Statement::ClassDef { name, stereotype } => {
                    classes
                        .entry(name.clone())
                        .or_insert_with(|| (stereotype.clone(), Vec::new()));
                }
                Statement::ClassMember {
                    class_name,
                    visibility,
                    name,
                    type_annotation,
                    ..
                } => {
                    let vis_symbol = match visibility {
                        ClassVisibility::Public => "+",
                        ClassVisibility::Private => "-",
                        ClassVisibility::Protected => "#",
                        ClassVisibility::Package => "~",
                    };
                    let type_str = type_annotation
                        .as_deref()
                        .map(|t| format!(": {}", t))
                        .unwrap_or_default();
                    let member_text = format!("{}{}{}", vis_symbol, name, type_str);
                    classes
                        .entry(class_name.clone())
                        .or_insert_with(|| (None, Vec::new()))
                        .1
                        .push(("".to_string(), member_text, "".to_string()));
                }
                Statement::ClassRelation { from, to, .. } => {
                    classes
                        .entry(from.clone())
                        .or_insert_with(|| (None, Vec::new()));
                    classes
                        .entry(to.clone())
                        .or_insert_with(|| (None, Vec::new()));
                }
                _ => {}
            }
        }

        let class_w = 180.0;
        let header_h = 32.0;
        let member_h = 22.0;
        let spacing_x = 70.0;
        let spacing_y = 60.0;
        let start_x = 50.0;
        let start_y = 50.0;

        let mut class_list: Vec<&String> = classes.keys().collect();
        class_list.sort();
        let cols = ((class_list.len() as f32).sqrt().ceil() as usize).max(1);

        // Compute row max heights
        let mut row_heights: Vec<f32> = Vec::new();
        for (i, class_name) in class_list.iter().enumerate() {
            let row = i / cols;
            let members = &classes[*class_name].1;
            let h = header_h + members.len() as f32 * member_h + 8.0;
            if row >= row_heights.len() {
                row_heights.push(h);
            } else {
                row_heights[row] = row_heights[row].max(h);
            }
        }

        let mut row_y_starts: Vec<f32> = Vec::new();
        let mut curr_y = start_y;
        for &rh in &row_heights {
            row_y_starts.push(curr_y);
            curr_y += rh + spacing_y;
        }

        let mut class_rects: HashMap<String, (f32, f32, f32, f32)> = HashMap::new();

        for (i, class_name) in class_list.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            let x = start_x + col as f32 * (class_w + spacing_x);
            let y = row_y_starts[row];

            let (_stereotype, members) = &classes[*class_name];
            let total_h = header_h + members.len() as f32 * member_h + 8.0;
            class_rects.insert((*class_name).clone(), (x, y, class_w, total_h));

            // Outer class card with modern styling and rounded corners
            svg.add_rect(
                x,
                y,
                class_w,
                total_h,
                "fill:#ffffff;stroke:#64748b;stroke-width:1.5;rx:6",
            );

            // Header background pill/block
            svg.add_rect(x, y, class_w, header_h, "fill:#f1f5f9;stroke:none;rx:6");
            // Patch bottom corners of header to be square
            svg.add_rect(
                x,
                y + header_h - 6.0,
                class_w,
                6.0,
                "fill:#f1f5f9;stroke:none",
            );
            // Header divider line
            svg.add_line(
                x,
                y + header_h,
                x + class_w,
                y + header_h,
                "stroke:#cbd5e1;stroke-width:1",
            );

            svg.add_text(
                x + class_w / 2.0,
                y + header_h / 2.0 + 1.0,
                class_name,
                "text-anchor:middle;dominant-baseline:middle;font-size:13px;font-weight:bold;fill:#0f172a;font-family:Arial, sans-serif",
            );

            for (j, (_, member_text, _)) in members.iter().enumerate() {
                svg.add_text(
                    x + 10.0,
                    y + header_h + 6.0 + j as f32 * member_h + member_h / 2.0,
                    member_text,
                    "dominant-baseline:middle;font-size:11px;fill:#334155;font-family:monospace",
                );
            }
        }

        // Adjust canvas dimensions to fit classes
        let max_w = start_x + cols as f32 * (class_w + spacing_x);
        let max_h = curr_y + 40.0;
        svg.set_dimensions(
            max_w.max(self.width as f32) as u32,
            max_h.max(self.height as f32) as u32,
        );

        // Draw relationships with obstacle-avoiding boundary connections
        for stmt in &diagram.statements {
            if let Statement::ClassRelation {
                from, to, label, ..
            } = stmt
            {
                if let (Some(&(x1, y1, w1, h1)), Some(&(x2, y2, w2, h2))) =
                    (class_rects.get(from), class_rects.get(to))
                {
                    let cx1 = x1 + w1 / 2.0;
                    let cy1 = y1 + h1 / 2.0;
                    let cx2 = x2 + w2 / 2.0;
                    let cy2 = y2 + h2 / 2.0;

                    // Choose boundary connection points based on relative position
                    let (start_pt, end_pt) = if (y2 + h2) <= y1 + 5.0 {
                        // `to` is above `from`
                        ((cx1, y1), (cx2, y2 + h2))
                    } else if y2 >= (y1 + h1 - 5.0) {
                        // `to` is below `from`
                        ((cx1, y1 + h1), (cx2, y2))
                    } else if cx2 > cx1 {
                        // `to` is to the right
                        ((x1 + w1, cy1), (x2, cy2))
                    } else {
                        // `to` is to the left
                        ((x1, cy1), (x2 + w2, cy2))
                    };

                    let dx = end_pt.0 - start_pt.0;
                    let dy = end_pt.1 - start_pt.1;
                    let len = (dx * dx + dy * dy).sqrt().max(1.0);
                    let ux = dx / len;
                    let uy = dy / len;
                    let nx = -uy;
                    let ny = ux;

                    let arrow_len = 10.0;
                    let arrow_w = 5.0;
                    let line_end_x = end_pt.0 - ux * arrow_len;
                    let line_end_y = end_pt.1 - uy * arrow_len;

                    svg.add_line(
                        start_pt.0,
                        start_pt.1,
                        line_end_x,
                        line_end_y,
                        "stroke:#475569;stroke-width:1.5;fill:none",
                    );

                    if let Some(lbl) = label {
                        svg.add_text(
                            (start_pt.0 + end_pt.0) / 2.0,
                            (start_pt.1 + end_pt.1) / 2.0 - 6.0,
                            lbl,
                            "text-anchor:middle;font-size:10px;fill:#475569;font-family:Arial, sans-serif",
                        );
                    }

                    // Open inheritance triangle at target boundary
                    let p1 = end_pt;
                    let p2 = (
                        end_pt.0 - ux * arrow_len + nx * arrow_w,
                        end_pt.1 - uy * arrow_len + ny * arrow_w,
                    );
                    let p3 = (
                        end_pt.0 - ux * arrow_len - nx * arrow_w,
                        end_pt.1 - uy * arrow_len - ny * arrow_w,
                    );
                    svg.add_polygon(
                        &[p1, p2, p3],
                        "fill:#ffffff;stroke:#475569;stroke-width:1.5",
                    );
                }
            }
        }

        svg.build()
    }

    fn render_state(&self, diagram: &Diagram) -> String {
        let mut svg = self.setup_svg_builder();

        let mut state_labels: HashMap<String, Option<String>> = HashMap::new();
        let mut transitions: Vec<(String, String, Option<String>)> = Vec::new();

        for stmt in &diagram.statements {
            match stmt {
                Statement::StateDef { id, label } => {
                    state_labels.insert(id.clone(), label.clone());
                }
                Statement::StateTransition { from, to, label } => {
                    transitions.push((from.clone(), to.clone(), label.clone()));
                    state_labels
                        .entry(from.clone())
                        .or_insert_with(|| Some(from.clone()));
                    state_labels
                        .entry(to.clone())
                        .or_insert_with(|| Some(to.clone()));
                }
                _ => {}
            }
        }

        let state_w = 140.0;
        let state_h = 45.0;
        let circle_r = 20.0;
        let start_x = 80.0;
        let start_y = 80.0;

        let state_list: Vec<&String> = state_labels.keys().filter(|k| *k != "[*]").collect();
        let mut positions: HashMap<String, (f32, f32)> = HashMap::new();

        for (i, name) in state_list.iter().enumerate() {
            let col = i % 3;
            let row = i / 3;
            positions.insert(
                (*name).clone(),
                (
                    start_x + col as f32 * (state_w + 60.0),
                    start_y + row as f32 * (state_h + 60.0),
                ),
            );
        }

        let has_start = transitions.iter().any(|(f, _, _)| f == "[*]");
        let has_end = transitions.iter().any(|(_, t, _)| t == "[*]");

        for (from, to, label) in &transitions {
            let (x1, y1) = if from == "[*]" {
                (start_x - 30.0, start_y + state_h / 2.0)
            } else if let Some(&pos) = positions.get(from.as_str()) {
                (pos.0 + state_w / 2.0, pos.1 + state_h / 2.0)
            } else {
                continue;
            };

            let (x2, y2) = if to == "[*]" {
                if !state_list.is_empty() {
                    let last = state_list.last().unwrap();
                    let lp = positions.get(last.as_str()).unwrap();
                    (lp.0 + state_w + 30.0, lp.1 + state_h / 2.0)
                } else {
                    continue;
                }
            } else if let Some(&pos) = positions.get(to.as_str()) {
                (pos.0 + state_w / 2.0, pos.1 + state_h / 2.0)
            } else {
                continue;
            };

            svg.add_line(x1, y1, x2, y2, "stroke:black;stroke-width:1;fill:none");

            if let Some(lbl) = label {
                svg.add_text(
                    (x1 + x2) / 2.0,
                    (y1 + y2) / 2.0 - 6.0,
                    lbl,
                    "text-anchor:middle;font-size:10px;font-family:Arial",
                );
            }
        }

        if has_start {
            svg.add_circle(
                start_x - 30.0,
                start_y + state_h / 2.0,
                circle_r,
                "fill:black;stroke:black;stroke-width:1",
            );
        }

        for name in state_labels.keys() {
            if name == "[*]" {
                continue;
            }
            if let Some(&pos) = positions.get(name.as_str()) {
                let display = state_labels
                    .get(name)
                    .and_then(|l| l.clone())
                    .unwrap_or_else(|| name.clone());
                svg.add_rect(
                    pos.0,
                    pos.1,
                    state_w,
                    state_h,
                    "fill:white;stroke:black;stroke-width:1;rx:8",
                );
                svg.add_text(
                    pos.0 + state_w / 2.0,
                    pos.1 + state_h / 2.0,
                    &display,
                    "text-anchor:middle;dominant-baseline:middle;font-size:13px;font-family:Arial",
                );
            }
        }

        if has_end && !state_list.is_empty() {
            let last = state_list.last().unwrap();
            let lp = positions.get(last.as_str()).unwrap();
            svg.add_circle(
                lp.0 + state_w + 30.0,
                lp.1 + state_h / 2.0,
                circle_r,
                "fill:white;stroke:black;stroke-width:2",
            );
            svg.add_circle(
                lp.0 + state_w + 30.0,
                lp.1 + state_h / 2.0,
                circle_r * 0.4,
                "fill:black;stroke:none",
            );
        }

        svg.build()
    }

    fn render_er(&self, diagram: &Diagram) -> String {
        let mut svg = self.setup_svg_builder();

        let mut entities: HashMap<String, Vec<(String, String, bool, bool)>> = HashMap::new();

        for stmt in &diagram.statements {
            match stmt {
                Statement::ErEntity { name } => {
                    entities.entry(name.clone()).or_default();
                }
                Statement::ErAttribute {
                    entity,
                    name,
                    type_annotation,
                    is_pk,
                    is_null,
                } => {
                    let type_str = type_annotation.as_deref().unwrap_or("").to_string();
                    entities.entry(entity.clone()).or_default().push((
                        name.clone(),
                        type_str,
                        *is_pk,
                        *is_null,
                    ));
                }
                Statement::ErRelation { from, to, .. } => {
                    entities.entry(from.clone()).or_default();
                    entities.entry(to.clone()).or_default();
                }
                _ => {}
            }
        }

        let entity_w = 180.0;
        let header_h = 30.0;
        let attr_h = 20.0;
        let spacing_x = 80.0;
        let spacing_y = 60.0;
        let start_x = 50.0;
        let start_y = 50.0;

        let entity_list: Vec<&String> = entities.keys().collect();
        let cols = ((entity_list.len() as f32).sqrt().ceil() as usize).max(1);

        for (i, name) in entity_list.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            let x = start_x + col as f32 * (entity_w + spacing_x);
            let y = start_y + row as f32 * (spacing_y);

            let attrs = &entities[name.to_owned()];
            let total_h = header_h + attrs.len() as f32 * attr_h;

            svg.add_rect(
                x,
                y,
                entity_w,
                total_h,
                "fill:white;stroke:black;stroke-width:1",
            );
            svg.add_text(
                x + entity_w / 2.0, y + header_h / 2.0, name,
                "text-anchor:middle;dominant-baseline:middle;font-size:13px;font-weight:bold;font-family:Arial",
            );
            svg.add_line(
                x,
                y + header_h,
                x + entity_w,
                y + header_h,
                "stroke:black;stroke-width:1",
            );

            for (j, (attr_name, attr_type, is_pk, _is_null)) in attrs.iter().enumerate() {
                let prefix = if *is_pk { "PK " } else { "" };
                let type_part = if !attr_type.is_empty() {
                    format!(" {}", attr_type)
                } else {
                    String::new()
                };
                svg.add_text(
                    x + 8.0,
                    y + header_h + j as f32 * attr_h + attr_h / 2.0,
                    &format!("{}{}{}", prefix, attr_name, type_part),
                    "dominant-baseline:middle;font-size:11px;font-family:monospace",
                );
            }
        }

        for stmt in &diagram.statements {
            if let Statement::ErRelation {
                from, to, label, ..
            } = stmt
            {
                let pos_from = entity_list.iter().position(|n| *n == from);
                let pos_to = entity_list.iter().position(|n| *n == to);
                if let (Some(ifrom), Some(ito)) = (pos_from, pos_to) {
                    let col_from = ifrom % cols;
                    let row_from = ifrom / cols;
                    let col_to = ito % cols;
                    let row_to = ito / cols;
                    let x1 = start_x + col_from as f32 * (entity_w + spacing_x) + entity_w / 2.0;
                    let y1 = start_y + row_from as f32 * (spacing_y) + header_h;
                    let x2 = start_x + col_to as f32 * (entity_w + spacing_x) + entity_w / 2.0;
                    let y2 = start_y + row_to as f32 * (spacing_y) + header_h;
                    svg.add_line(x1, y1, x2, y2, "stroke:black;stroke-width:1;fill:none");
                    if let Some(lbl) = label {
                        svg.add_text(
                            (x1 + x2) / 2.0,
                            (y1 + y2) / 2.0 - 6.0,
                            lbl,
                            "text-anchor:middle;font-size:10px;font-family:Arial;font-style:italic",
                        );
                    }
                }
            }
        }

        svg.build()
    }

    #[allow(clippy::type_complexity)]
    fn render_gantt(&self, diagram: &Diagram) -> String {
        let mut svg = self.setup_svg_builder();

        let mut sections: Vec<(String, Vec<(String, String, String)>)> = Vec::new();
        let mut current_tasks: Vec<(String, String, String)> = Vec::new();
        let mut current_section = "__default".to_string();

        for stmt in &diagram.statements {
            match stmt {
                Statement::GanttSection { name } => {
                    if !current_tasks.is_empty() {
                        sections.push((current_section.clone(), current_tasks));
                        current_tasks = Vec::new();
                    }
                    current_section = name.clone();
                }
                Statement::GanttTask { name, id, .. } => {
                    let id_str = id.clone().unwrap_or_default();
                    current_tasks.push((name.clone(), id_str, String::new()));
                }
                _ => {}
            }
        }
        if !current_tasks.is_empty() {
            sections.push((current_section.clone(), current_tasks));
        }

        let mut y = 40.0;
        let label_x = 20.0;
        let bar_start_x = 200.0;
        let bar_width = 400.0;
        let bar_h = 24.0;
        let row_h = 35.0;

        if let Some(ref t) = diagram.title {
            svg.add_text(
                self.width as f32 / 2.0,
                20.0,
                t,
                "text-anchor:middle;font-size:16px;font-weight:bold;font-family:Arial",
            );
            y = 50.0;
        }

        let colors = ["#45b7d1", "#96ceb4", "#ffeaa7", "#ff6b6b", "#a29bfe"];

        for (section_name, tasks) in &sections {
            if section_name != "__default" {
                svg.add_text(
                    label_x,
                    y,
                    section_name,
                    "font-size:13px;font-weight:bold;font-family:Arial",
                );
                y += row_h;
            }
            for (i, (task_name, _, _)) in tasks.iter().enumerate() {
                svg.add_text(
                    label_x,
                    y + bar_h / 2.0,
                    task_name,
                    "dominant-baseline:middle;font-size:12px;font-family:Arial",
                );
                let color = colors[i % colors.len()];
                svg.add_rect(
                    bar_start_x,
                    y,
                    bar_width * 0.7,
                    bar_h,
                    &format!("fill:{};stroke:#333;stroke-width:1;rx:4", color),
                );
                y += row_h;
            }
        }

        svg.build()
    }

    fn render_mindmap(&self, diagram: &Diagram) -> String {
        let mut svg = self.setup_svg_builder();

        struct NodePos {
            label: String,
            depth: usize,
            x: f32,
            y: f32,
            w: f32,
            h: f32,
            children: Vec<usize>,
        }

        fn layout_tree(
            node: &Statement,
            depth: usize,
            leaf_y: &mut f32,
            nodes: &mut Vec<NodePos>,
        ) -> usize {
            if let Statement::MindmapNode {
                label, children, ..
            } = node
            {
                let text_chars = label.chars().count() as f32;
                let w = (text_chars * 13.0 + 36.0).clamp(100.0, 220.0);
                let h = 34.0;
                let x = 60.0 + depth as f32 * 210.0;
                let my_idx = nodes.len();
                nodes.push(NodePos {
                    label: label.clone(),
                    depth,
                    x,
                    y: 0.0,
                    w,
                    h,
                    children: Vec::new(),
                });

                if children.is_empty() {
                    let y = *leaf_y;
                    *leaf_y += 50.0;
                    nodes[my_idx].y = y;
                } else {
                    let mut child_indices = Vec::new();
                    let mut sum_y = 0.0;
                    for child in children {
                        let c_idx = layout_tree(child, depth + 1, leaf_y, nodes);
                        child_indices.push(c_idx);
                        sum_y += nodes[c_idx].y;
                    }
                    nodes[my_idx].children = child_indices;
                    nodes[my_idx].y = sum_y / children.len() as f32;
                }
                my_idx
            } else {
                0
            }
        }

        let mut nodes: Vec<NodePos> = Vec::new();
        let mut leaf_y = 60.0;

        for stmt in &diagram.statements {
            if matches!(stmt, Statement::MindmapNode { .. }) {
                layout_tree(stmt, 0, &mut leaf_y, &mut nodes);
            }
        }

        if nodes.is_empty() {
            let cx = self.width as f32 / 2.0;
            svg.add_rect(
                cx - 80.0,
                self.height as f32 / 2.0 - 25.0,
                160.0,
                50.0,
                "fill:#f8fafc;stroke:#64748b;stroke-width:1.5;rx:8",
            );
            svg.add_text(
                cx,
                self.height as f32 / 2.0 + 4.0,
                "Mindmap",
                "text-anchor:middle;font-size:14px;font-weight:bold;fill:#0f172a;font-family:Arial, sans-serif",
            );
            return svg.build();
        }

        // Adjust dimensions to fit content
        let max_x = nodes.iter().map(|n| n.x + n.w).fold(0.0_f32, f32::max) + 60.0;
        let max_y = leaf_y.max(self.height as f32);
        svg.set_dimensions(max_x.max(self.width as f32) as u32, max_y as u32);

        // Draw curved connector lines from parents to children
        for node in &nodes {
            let p_x = node.x + node.w;
            let p_y = node.y;
            for &c_idx in &node.children {
                let c_node = &nodes[c_idx];
                let c_x = c_node.x;
                let c_y = c_node.y;
                let mid_x = (p_x + c_x) / 2.0;
                let path = format!(
                    "M {} {} C {} {}, {} {}, {} {}",
                    p_x, p_y, mid_x, p_y, mid_x, c_y, c_x, c_y
                );
                svg.add_path(&path, "stroke:#94a3b8;stroke-width:1.5;fill:none");
            }
        }

        // Draw nodes with modern color hierarchy
        for node in &nodes {
            let (fill, stroke, text_color, font_size, font_weight, rx) = match node.depth {
                0 => ("#ea580c", "#c2410c", "#ffffff", "13px", "bold", "17"),
                1 => ("#fff7ed", "#ea580c", "#9a3412", "12px", "bold", "8"),
                _ => ("#ffffff", "#e4e4e7", "#27272a", "11px", "normal", "6"),
            };

            let rect_style = format!("fill:{};stroke:{};stroke-width:1.5;rx:{}", fill, stroke, rx);
            svg.add_rect(node.x, node.y - node.h / 2.0, node.w, node.h, &rect_style);

            let text_style = format!(
                "text-anchor:middle;dominant-baseline:middle;font-size:{};font-weight:{};fill:{};font-family:Arial, sans-serif",
                font_size, font_weight, text_color
            );
            svg.add_text(node.x + node.w / 2.0, node.y, &node.label, &text_style);
        }

        svg.build()
    }

    fn render_gitgraph(&self, diagram: &Diagram) -> String {
        let mut svg = self.setup_svg_builder();

        let start_x = 100.0;
        let start_y = 70.0;
        let spacing_y = 48.0;
        let branch_colors = ["#ea580c", "#10b981", "#f59e0b", "#475569", "#0d9488"];

        let mut branches: HashMap<String, usize> = HashMap::new();
        branches.insert("main".to_string(), 0);
        let mut current_branch = "main".to_string();
        let mut commit_count: usize = 0;
        let mut last_commit_y: HashMap<String, f32> = HashMap::new();

        // First pass: collect branches
        for stmt in &diagram.statements {
            if let Statement::GitBranch { name } = stmt {
                if !branches.contains_key(name) {
                    let col = branches.len();
                    branches.insert(name.clone(), col);
                }
            }
        }

        // Draw branch header badges at the top
        for (name, &col) in &branches {
            let x = start_x + col as f32 * 90.0;
            let color = branch_colors[col % branch_colors.len()];
            let badge_w = (name.len() as f32 * 8.0 + 20.0).max(56.0);
            svg.add_rect(
                x - badge_w / 2.0,
                24.0,
                badge_w,
                22.0,
                &format!(
                    "fill:{};fill-opacity:0.12;stroke:{};stroke-width:1;rx:4",
                    color, color
                ),
            );
            svg.add_text(
                x,
                35.0,
                name,
                &format!("text-anchor:middle;dominant-baseline:middle;font-size:11px;font-weight:bold;fill:{};font-family:monospace", color),
            );
            // Subtle branch guideline
            svg.add_line(
                x,
                46.0,
                x,
                start_y,
                &format!(
                    "stroke:{};stroke-width:1;stroke-dasharray:3,3;stroke-opacity:0.4",
                    color
                ),
            );
        }

        for stmt in &diagram.statements {
            match stmt {
                Statement::GitCommit { id, tag, .. } => {
                    let col = *branches.get(&current_branch).unwrap_or(&0);
                    let x = start_x + col as f32 * 90.0;
                    let y = start_y + commit_count as f32 * spacing_y;
                    let color = branch_colors[col % branch_colors.len()];

                    // Line from previous commit on same branch
                    if let Some(&prev_y) = last_commit_y.get(&current_branch) {
                        svg.add_line(x, prev_y, x, y, &format!("stroke:{};stroke-width:2", color));
                    }
                    last_commit_y.insert(current_branch.clone(), y);

                    // Commit node
                    svg.add_circle(
                        x,
                        y,
                        10.0,
                        &format!("fill:{};stroke:#ffffff;stroke-width:2", color),
                    );

                    if let Some(tag_str) = tag {
                        svg.add_text(
                            x + 18.0,
                            y + 4.0,
                            tag_str,
                            "font-size:10px;font-family:Arial, sans-serif;fill:#475569;font-weight:bold",
                        );
                    }
                    if let Some(id_str) = id {
                        svg.add_text(
                            x - 16.0,
                            y + 4.0,
                            id_str,
                            "text-anchor:end;font-size:10px;font-family:monospace;fill:#64748b",
                        );
                    }
                    commit_count += 1;
                }
                Statement::GitBranch { name } => {
                    // Fork branch from parent's last commit
                    if let Some(&parent_y) = last_commit_y.get(&current_branch) {
                        let parent_col = *branches.get(&current_branch).unwrap_or(&0);
                        let parent_x = start_x + parent_col as f32 * 90.0;
                        let child_col = *branches.get(name).unwrap_or(&0);
                        let child_x = start_x + child_col as f32 * 90.0;
                        let child_y = start_y + commit_count as f32 * spacing_y;
                        let color = branch_colors[child_col % branch_colors.len()];
                        svg.add_line(
                            parent_x,
                            parent_y,
                            child_x,
                            child_y,
                            &format!("stroke:{};stroke-width:1.5;stroke-dasharray:4,3", color),
                        );
                        last_commit_y.insert(name.clone(), child_y);
                    }
                }
                Statement::GitCheckout { name } => {
                    current_branch = name.clone();
                }
                Statement::GitMerge { branch, .. } => {
                    let col_from = *branches.get(branch).unwrap_or(&0);
                    let col_to = *branches.get(&current_branch).unwrap_or(&0);
                    let x_from = start_x + col_from as f32 * 90.0;
                    let x_to = start_x + col_to as f32 * 90.0;
                    let y = start_y + commit_count as f32 * spacing_y;

                    let from_y = last_commit_y.get(branch).copied().unwrap_or(y - spacing_y);
                    svg.add_line(
                        x_from,
                        from_y,
                        x_to,
                        y,
                        "stroke:#64748b;stroke-width:2;stroke-dasharray:4,3",
                    );

                    // Commit node on destination branch
                    let to_color = branch_colors[col_to % branch_colors.len()];
                    if let Some(&prev_y) = last_commit_y.get(&current_branch) {
                        svg.add_line(
                            x_to,
                            prev_y,
                            x_to,
                            y,
                            &format!("stroke:{};stroke-width:2", to_color),
                        );
                    }
                    last_commit_y.insert(current_branch.clone(), y);

                    svg.add_circle(
                        x_to,
                        y,
                        10.0,
                        &format!("fill:{};stroke:#ffffff;stroke-width:2", to_color),
                    );
                    commit_count += 1;
                }
                _ => {}
            }
        }

        // Adjust dimensions dynamically
        let max_w = start_x + (branches.len() as f32) * 90.0 + 80.0;
        let max_h = start_y + (commit_count as f32) * spacing_y + 40.0;
        svg.set_dimensions(
            max_w.max(self.width as f32) as u32,
            max_h.max(self.height as f32) as u32,
        );

        svg.build()
    }

    fn render_timeline(&self, diagram: &Diagram) -> String {
        let mut svg = self.setup_svg_builder();

        let mut events: Vec<(String, String)> = Vec::new();
        let mut current_section: Option<String> = None;

        for stmt in &diagram.statements {
            match stmt {
                Statement::TimelineSection { name } => {
                    current_section = Some(name.clone());
                }
                Statement::TimelineEvent { time, description } => {
                    events.push((time.clone(), description.clone()));
                }
                _ => {}
            }
        }

        if events.is_empty() {
            svg.add_text(
                self.width as f32 / 2.0,
                self.height as f32 / 2.0,
                "timeline",
                "text-anchor:middle;font-size:14px;font-family:Arial",
            );
            return svg.build();
        }

        let mid_y = self.height as f32 / 2.0;
        let spacing = (self.width as f32 - 100.0) / events.len() as f32;
        let start_x = 80.0;

        svg.add_line(
            50.0,
            mid_y,
            self.width as f32 - 50.0,
            mid_y,
            "stroke:#999;stroke-width:2",
        );

        for (i, (time, desc)) in events.iter().enumerate() {
            let x = start_x + i as f32 * spacing;
            svg.add_circle(x, mid_y, 8.0, "fill:#45b7d1;stroke:#333;stroke-width:1");
            svg.add_text(
                x,
                mid_y - 20.0,
                time,
                "text-anchor:middle;font-size:12px;font-weight:bold;font-family:Arial",
            );
            svg.add_text(
                x,
                mid_y + 24.0,
                desc,
                "text-anchor:middle;font-size:12px;font-family:Arial",
            );
        }

        if let Some(section) = current_section {
            svg.add_text(
                self.width as f32 / 2.0,
                20.0,
                &section,
                "text-anchor:middle;font-size:14px;font-weight:bold;font-family:Arial",
            );
        }

        svg.build()
    }

    #[allow(clippy::type_complexity)]
    fn render_journey(&self, diagram: &Diagram) -> String {
        let mut svg = self.setup_svg_builder();

        let mut sections: Vec<(String, Vec<(String, f64, Vec<String>)>)> = Vec::new();
        let mut current_tasks: Vec<(String, f64, Vec<String>)> = Vec::new();
        let mut current_section: Option<String> = None;

        for stmt in &diagram.statements {
            match stmt {
                Statement::JourneySection { name } => {
                    if !current_tasks.is_empty() {
                        sections.push((current_section.clone().unwrap_or_default(), current_tasks));
                        current_tasks = Vec::new();
                    }
                    current_section = Some(name.clone());
                }
                Statement::JourneyTask {
                    name,
                    score,
                    actors,
                } => {
                    current_tasks.push((name.clone(), *score, actors.clone()));
                }
                _ => {}
            }
        }
        if !current_tasks.is_empty() {
            sections.push((current_section.clone().unwrap_or_default(), current_tasks));
        }

        let mut y = 30.0;
        let task_w = 140.0;
        let task_h = 40.0;
        let spacing = 30.0;
        let start_x = 30.0;

        for (section_name, tasks) in &sections {
            if !section_name.is_empty() {
                svg.add_text(
                    start_x,
                    y,
                    section_name,
                    "font-size:14px;font-weight:bold;font-family:Arial",
                );
                y += 25.0;
            }

            let colors = ["#45b7d1", "#96ceb4", "#ffeaa7", "#ff6b6b", "#a29bfe"];

            for (i, (name, score, actors)) in tasks.iter().enumerate() {
                let x = start_x + i as f32 * (task_w + spacing);
                let color = colors[i % colors.len()];

                svg.add_rect(
                    x,
                    y,
                    task_w,
                    task_h,
                    &format!("fill:{};stroke:#333;stroke-width:1;rx:4", color),
                );
                svg.add_text(
                    x + task_w / 2.0,
                    y + task_h / 2.0 - 4.0,
                    name,
                    "text-anchor:middle;font-size:12px;font-weight:bold;font-family:Arial",
                );

                let score_str = format!("Score: {}", score);
                svg.add_text(
                    x + task_w / 2.0,
                    y + task_h / 2.0 + 12.0,
                    &score_str,
                    "text-anchor:middle;font-size:9px;font-family:Arial;fill:#555",
                );

                if !actors.is_empty() {
                    let actors_str = actors.join(", ");
                    svg.add_text(
                        x + task_w / 2.0,
                        y + task_h + 14.0,
                        &actors_str,
                        "text-anchor:middle;font-size:10px;font-family:Arial;fill:#666",
                    );
                }
            }
            y += task_h + 40.0;
        }

        svg.build()
    }

    #[allow(clippy::type_complexity)]
    fn render_kanban(&self, diagram: &Diagram) -> String {
        let mut svg = self.setup_svg_builder();

        let mut columns: Vec<(String, Vec<(String, Option<String>)>)> = Vec::new();
        let mut current_tasks: Vec<(String, Option<String>)> = Vec::new();
        let mut current_col: Option<String> = None;

        for stmt in &diagram.statements {
            match stmt {
                Statement::KanbanColumn { name } => {
                    if !current_tasks.is_empty() {
                        columns.push((current_col.clone().unwrap_or_default(), current_tasks));
                        current_tasks = Vec::new();
                    }
                    current_col = Some(name.clone());
                }
                Statement::KanbanTask { name, description } => {
                    current_tasks.push((name.clone(), description.clone()));
                }
                _ => {}
            }
        }
        if !current_tasks.is_empty() {
            columns.push((current_col.clone().unwrap_or_default(), current_tasks));
        }

        if columns.is_empty() {
            svg.add_text(
                self.width as f32 / 2.0,
                self.height as f32 / 2.0,
                "kanban",
                "text-anchor:middle;font-size:14px;font-family:Arial",
            );
            return svg.build();
        }

        let col_w = (self.width as f32 - 60.0) / columns.len() as f32;
        let start_x = 20.0;
        let start_y = 40.0;

        for (i, (col_name, tasks)) in columns.iter().enumerate() {
            let x = start_x + i as f32 * col_w;
            let header_h = 36.0;

            svg.add_rect(
                x,
                start_y,
                col_w - 10.0,
                header_h,
                "fill:#e8f4f8;stroke:#45b7d1;stroke-width:1;rx:4",
            );
            svg.add_text(x + (col_w - 10.0) / 2.0, start_y + header_h / 2.0, col_name,
                         "text-anchor:middle;dominant-baseline:middle;font-size:13px;font-weight:bold;font-family:Arial");

            let card_h = 50.0;
            let card_spacing = 10.0;
            for (j, (task_name, desc)) in tasks.iter().enumerate() {
                let ty = start_y + header_h + 10.0 + j as f32 * (card_h + card_spacing);
                svg.add_rect(
                    x + 5.0,
                    ty,
                    col_w - 20.0,
                    card_h,
                    "fill:white;stroke:#ccc;stroke-width:1;rx:4",
                );
                svg.add_text(
                    x + (col_w - 20.0) / 2.0,
                    ty + card_h / 2.0 - 4.0,
                    task_name,
                    "text-anchor:middle;font-size:12px;font-weight:bold;font-family:Arial",
                );
                if let Some(d) = desc {
                    svg.add_text(
                        x + (col_w - 20.0) / 2.0,
                        ty + card_h / 2.0 + 12.0,
                        d,
                        "text-anchor:middle;font-size:10px;font-family:Arial;fill:#666",
                    );
                }
            }
        }

        svg.build()
    }

    fn render_venn(&self, diagram: &Diagram) -> String {
        let mut svg = self.setup_svg_builder();

        let mut sets: Vec<(String, String)> = Vec::new();
        for stmt in &diagram.statements {
            if let Statement::VennSet { id, label } = stmt {
                sets.push((id.clone(), label.clone()));
            }
        }

        let cx = self.width as f32 / 2.0;
        let cy = self.height as f32 / 2.0;
        let r = 120.0;
        let colors = ["#ff6b6b", "#4ecdc4", "#45b7d1", "#a29bfe", "#fdcb6e"];

        match sets.len() {
            0 => {}
            1 => {
                svg.add_circle(
                    cx,
                    cy,
                    r,
                    &format!(
                        "fill:{};stroke:#333;stroke-width:1;fill-opacity:0.3",
                        colors[0]
                    ),
                );
                svg.add_text(
                    cx,
                    cy + 4.0,
                    &sets[0].1,
                    "text-anchor:middle;font-size:18px;font-weight:bold;font-family:Arial",
                );
            }
            2 => {
                let offset = r * 0.35;
                svg.add_circle(
                    cx - offset,
                    cy,
                    r,
                    &format!(
                        "fill:{};stroke:#333;stroke-width:1;fill-opacity:0.3",
                        colors[0]
                    ),
                );
                svg.add_circle(
                    cx + offset,
                    cy,
                    r,
                    &format!(
                        "fill:{};stroke:#333;stroke-width:1;fill-opacity:0.3",
                        colors[1]
                    ),
                );
                svg.add_text(
                    cx - offset - r * 0.5,
                    cy,
                    &sets[0].1,
                    "text-anchor:middle;font-size:14px;font-family:Arial",
                );
                svg.add_text(
                    cx + offset + r * 0.5,
                    cy,
                    &sets[1].1,
                    "text-anchor:middle;font-size:14px;font-family:Arial",
                );
                if sets.len() > 2 {
                    svg.add_text(
                        cx,
                        cy,
                        &sets[2].1,
                        "text-anchor:middle;font-size:12px;font-family:Arial",
                    );
                }
            }
            _ => {
                let angles: Vec<f64> = (0..sets.len())
                    .map(|i| 2.0 * std::f64::consts::PI * i as f64 / sets.len() as f64)
                    .collect();
                for (i, (_, label)) in sets.iter().enumerate() {
                    let angle = angles[i];
                    let scx = cx + (r * 0.4) * angle.cos() as f32;
                    let scy = cy + (r * 0.4) * angle.sin() as f32;
                    svg.add_circle(
                        scx,
                        scy,
                        r * 0.65,
                        &format!(
                            "fill:{};stroke:#333;stroke-width:1;fill-opacity:0.3",
                            colors[i % colors.len()]
                        ),
                    );
                    let ld = r * 0.85;
                    let lx = cx + ld * angle.cos() as f32;
                    let ly = cy + ld * angle.sin() as f32;
                    svg.add_text(
                        lx,
                        ly,
                        label,
                        "text-anchor:middle;font-size:13px;font-family:Arial",
                    );
                }
            }
        }

        svg.build()
    }

    fn render_packet(&self, diagram: &Diagram) -> String {
        let mut svg = self.setup_svg_builder();

        let mut fields: Vec<(u32, u32, String)> = Vec::new();
        for stmt in &diagram.statements {
            if let Statement::PacketField {
                start_bit,
                end_bit,
                label,
            } = stmt
            {
                fields.push((*start_bit, *end_bit, label.clone()));
            }
        }

        if fields.is_empty() {
            return svg.build();
        }

        let total_bits = fields
            .iter()
            .map(|(s, e, _)| e.max(s) + 1)
            .max()
            .unwrap_or(32);
        let bit_width = (self.width as f32 - 80.0) / total_bits as f32;
        let x_start = 40.0;
        let y_start = self.height as f32 / 2.0 - 30.0;
        let field_h = 60.0;
        let label_h = 20.0;
        let colors = [
            "#45b7d1", "#96ceb4", "#ffeaa7", "#ff6b6b", "#a29bfe", "#fdcb6e",
        ];

        for (i, (start_bit, end_bit, label)) in fields.iter().enumerate() {
            let fw = (end_bit - start_bit + 1) as f32 * bit_width;
            let fx = x_start + *start_bit as f32 * bit_width;
            let color = colors[i % colors.len()];

            svg.add_rect(
                fx,
                y_start,
                fw,
                field_h,
                &format!("fill:{};stroke:#333;stroke-width:1;rx:2", color),
            );
            svg.add_text(
                fx + fw / 2.0,
                y_start + field_h / 2.0 + 4.0,
                label,
                "text-anchor:middle;font-size:11px;font-weight:bold;font-family:monospace",
            );
            svg.add_text(
                fx + fw / 2.0,
                y_start + field_h + label_h - 4.0,
                &format!("{}-{}", start_bit, end_bit),
                "text-anchor:middle;font-size:9px;font-family:monospace;fill:#666",
            );
        }

        svg.build()
    }

    fn render_radar(&self, diagram: &Diagram) -> String {
        let mut svg = self.setup_svg_builder();

        let mut axes: Vec<(String, f64)> = Vec::new();
        for stmt in &diagram.statements {
            if let Statement::RadarAxis { label, value } = stmt {
                axes.push((label.clone(), *value));
            }
        }

        if axes.is_empty() {
            return svg.build();
        }

        let cx = self.width as f32 / 2.0;
        let cy = self.height as f32 / 2.0;
        let max_r = (self.width.min(self.height) as f32 / 2.0) * 0.65;
        let n = axes.len();

        for ring_pct in &[0.25, 0.5, 0.75, 1.0] {
            let r = max_r * ring_pct;
            let mut points = Vec::new();
            for i in 0..n {
                let angle =
                    2.0 * std::f64::consts::PI * i as f64 / n as f64 - std::f64::consts::PI / 2.0;
                let px = cx + r * angle.cos() as f32;
                let py = cy + r * angle.sin() as f32;
                points.push((px, py));
            }
            svg.add_polygon(&points, "fill:none;stroke:#ddd;stroke-width:1");
        }

        for (i, (axis_name, _)) in axes.iter().take(n).enumerate() {
            let angle =
                2.0 * std::f64::consts::PI * i as f64 / n as f64 - std::f64::consts::PI / 2.0;
            let ex = cx + max_r * angle.cos() as f32;
            let ey = cy + max_r * angle.sin() as f32;
            svg.add_line(cx, cy, ex, ey, "stroke:#ccc;stroke-width:1");

            let lr = max_r + 25.0;
            let lx = cx + lr * angle.cos() as f32;
            let ly = cy + lr * angle.sin() as f32;
            svg.add_text(lx, ly, axis_name,
                         "text-anchor:middle;dominant-baseline:middle;font-size:12px;font-weight:bold;font-family:Arial");
        }

        let mut data_points = Vec::new();
        for (i, (_, val)) in axes.iter().take(n).enumerate() {
            let angle =
                2.0 * std::f64::consts::PI * i as f64 / n as f64 - std::f64::consts::PI / 2.0;
            let value = val.clamp(0.0, 100.0);
            let r = max_r * (value / 100.0) as f32;
            let px = cx + r * angle.cos() as f32;
            let py = cy + r * angle.sin() as f32;
            data_points.push((px, py));
            svg.add_circle(px, py, 4.0, "fill:#ff6b6b;stroke:#c0392b;stroke-width:1");
        }

        if !data_points.is_empty() {
            svg.add_polygon(
                &data_points,
                "fill:#ff6b6b;fill-opacity:0.2;stroke:#ff6b6b;stroke-width:2",
            );
        }

        svg.build()
    }

    fn render_ishikawa(&self, diagram: &Diagram) -> String {
        let mut svg = self.setup_svg_builder();

        let mut root: Option<String> = None;
        let mut categories: Vec<(String, Vec<String>)> = Vec::new();
        let mut current_category: Option<String> = None;
        let mut current_causes: Vec<String> = Vec::new();

        for stmt in &diagram.statements {
            match stmt {
                Statement::IshikawaRoot { label } => {
                    root = Some(label.clone());
                }
                Statement::IshikawaCategory { label } => {
                    if let Some(cat) = current_category.take() {
                        categories.push((cat, current_causes));
                        current_causes = Vec::new();
                    }
                    current_category = Some(label.clone());
                }
                Statement::IshikawaCause { label } => {
                    current_causes.push(label.clone());
                }
                _ => {}
            }
        }
        if let Some(cat) = current_category.take() {
            categories.push((cat, current_causes));
        }

        let cy = self.height as f32 / 2.0;
        let spine_end_x = self.width as f32 * 0.65;

        // Draw spine
        svg.add_line(50.0, cy, spine_end_x, cy, "stroke:#333;stroke-width:3");

        // Draw fish head
        let head_size = 30.0;
        let head_path = format!(
            "M {} {} L {} {} L {} {} L {} {} Z",
            spine_end_x,
            cy,
            spine_end_x + head_size,
            cy - head_size,
            spine_end_x + head_size + 10.0,
            cy,
            spine_end_x + head_size,
            cy + head_size,
        );
        svg.add_path(&head_path, "fill:#45b7d1;stroke:#333;stroke-width:2");

        if let Some(ref root_label) = root {
            svg.add_text(
                spine_end_x + head_size + 5.0,
                cy + 4.0,
                root_label,
                "text-anchor:middle;font-size:12px;font-weight:bold;font-family:Arial;fill:#fff",
            );
        }

        let n = categories.len().max(2) as f32;
        let bone_spacing = (self.height as f32 - 100.0) / n;

        for (i, (cat_name, causes)) in categories.iter().enumerate() {
            let cat_y = 50.0 + (i as f32 + 0.5) * bone_spacing;
            let bone_x = 60.0 + (spine_end_x - 60.0) * ((i as f32 + 0.5) / n);

            svg.add_line(
                bone_x,
                cy,
                bone_x + 80.0,
                cat_y,
                "stroke:#666;stroke-width:2",
            );
            svg.add_text(
                bone_x + 85.0,
                cat_y,
                cat_name,
                "font-size:12px;font-weight:bold;font-family:Arial",
            );

            for (j, cause) in causes.iter().enumerate() {
                let cause_x = bone_x + 20.0 + j as f32 * 40.0;
                let cause_y = cat_y + 20.0;
                svg.add_line(
                    cause_x,
                    cat_y,
                    cause_x + 10.0,
                    cause_y,
                    "stroke:#999;stroke-width:1",
                );
                svg.add_text(
                    cause_x + 5.0,
                    cause_y + 14.0,
                    cause,
                    "text-anchor:middle;font-size:10px;font-family:Arial;fill:#555",
                );
            }
        }

        svg.build()
    }

    fn render_quadrant(&self, diagram: &Diagram) -> String {
        let mut svg = self.setup_svg_builder();

        let cx = self.width as f32 / 2.0;
        let cy = self.height as f32 / 2.0;
        let size = (self.width.min(self.height) as f32) * 0.35;

        // Collect data
        let mut title = String::new();
        let mut x_label = String::from("X-Axis");
        let mut y_label = String::from("Y-Axis");
        let mut quad_labels: [String; 4] = [
            String::from("Quadrant 1"),
            String::from("Quadrant 2"),
            String::from("Quadrant 3"),
            String::from("Quadrant 4"),
        ];
        let mut points: Vec<(String, f32, f32)> = Vec::new();

        for stmt in &diagram.statements {
            match stmt {
                Statement::QuadrantTitle(t) => title = t.clone(),
                Statement::QuadrantXAxis(t) => x_label = t.to_string(),
                Statement::QuadrantYAxis(t) => y_label = t.to_string(),
                Statement::QuadrantLabel { quadrant, label } => {
                    if *quadrant >= 1 && *quadrant <= 4 {
                        quad_labels[*quadrant as usize - 1] = label.clone();
                    }
                }
                Statement::QuadrantPoint { label, x, y } => {
                    points.push((label.clone(), *x as f32, *y as f32));
                }
                _ => {}
            }
        }

        // Draw quadrant backgrounds
        let qcolors = ["#e8f5e9", "#e3f2fd", "#fce4ec", "#fff3e0"];
        svg.add_rect(
            cx,
            cy,
            size,
            size,
            &format!("fill:{};stroke:none", qcolors[0]),
        );
        svg.add_rect(
            cx - size,
            cy,
            size,
            size,
            &format!("fill:{};stroke:none", qcolors[1]),
        );
        svg.add_rect(
            cx - size,
            cy - size,
            size,
            size,
            &format!("fill:{};stroke:none", qcolors[2]),
        );
        svg.add_rect(
            cx,
            cy - size,
            size,
            size,
            &format!("fill:{};stroke:none", qcolors[3]),
        );

        // Axis lines
        svg.add_line(cx - size, cy, cx + size, cy, "stroke:#333;stroke-width:2");
        svg.add_line(cx, cy - size, cx, cy + size, "stroke:#333;stroke-width:2");

        // Axis labels
        svg.add_text(
            cx,
            cy + size + 20.0,
            &x_label,
            "text-anchor:middle;font-size:12px;font-family:Arial",
        );
        svg.add_text(
            cx - size - 10.0,
            cy,
            &y_label,
            "text-anchor:end;dominant-baseline:middle;font-size:12px;font-family:Arial",
        );

        // Quadrant labels
        let qpos = [
            (cx + size * 0.35, cy + size * 0.35),
            (cx - size * 0.35, cy + size * 0.35),
            (cx - size * 0.35, cy - size * 0.35),
            (cx + size * 0.35, cy - size * 0.35),
        ];
        for i in 0..4 {
            svg.add_text(qpos[i].0, qpos[i].1, &quad_labels[i],
                "text-anchor:middle;dominant-baseline:middle;font-size:11px;font-family:Arial;fill:#666");
        }

        // Plot points
        for (label, x, y) in &points {
            let px = cx - size + x * 2.0 * size;
            let py = cy + size - y * 2.0 * size;
            svg.add_circle(px, py, 5.0, "fill:#4A90D9;stroke:white;stroke-width:2");
            svg.add_text(
                px + 10.0,
                py,
                label,
                "text-anchor:start;dominant-baseline:middle;font-size:11px;font-family:Arial",
            );
        }

        // Title
        if !title.is_empty() {
            svg.add_text(
                self.width as f32 / 2.0,
                20.0,
                &title,
                "text-anchor:middle;font-size:16px;font-weight:bold;font-family:Arial",
            );
        }

        svg.build()
    }

    fn render_zenuml(&self, diagram: &Diagram) -> String {
        let mut svg =
            SvgBuilder::new(self.width, self.height).with_background_color(&self.background_color);

        // Collect participants in order of first appearance from Message statements
        let mut participants: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::<String>::new();
        for stmt in &diagram.statements {
            if let Statement::Message { from, to, .. } = stmt {
                if seen.insert(from.clone()) {
                    participants.push(from.clone());
                }
                if seen.insert(to.clone()) {
                    participants.push(to.clone());
                }
            }
        }
        if participants.is_empty() {
            return svg.build();
        }

        let box_w = 120.0;
        let box_h = 40.0;
        let gap = 60.0;
        let margin = 40.0;
        let spacing = 50.0;
        let header_end = margin + box_h + 20.0;

        let total_w = participants.len() as f32 * box_w + (participants.len() as f32 - 1.0) * gap;
        let start_x = (self.width as f32 - total_w) / 2.0;

        let mut pos: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
        for (i, name) in participants.iter().enumerate() {
            pos.insert(name.clone(), start_x + i as f32 * (box_w + gap));
        }

        let bottom = margin + spacing * (diagram.statements.len() + 1) as f32;

        // Lifelines
        for name in &participants {
            if let Some(&x) = pos.get(name) {
                let cx = x + box_w / 2.0;
                svg.add_line(
                    cx,
                    header_end,
                    cx,
                    bottom,
                    "stroke:#b0b0b0;stroke-width:1;stroke-dasharray:4,4",
                );
            }
        }

        // Participant boxes
        for name in &participants {
            if let Some(&x) = pos.get(name) {
                svg.add_rect(
                    x,
                    margin,
                    box_w,
                    box_h,
                    "fill:#e8f4fd;stroke:#5b9bd5;stroke-width:2;rx:4",
                );
                svg.add_text(x + box_w / 2.0, margin + box_h / 2.0, name,
                    "text-anchor:middle;dominant-baseline:middle;font-size:13px;font-family:Arial;fill:#333");
            }
        }

        // Messages
        for (idx, stmt) in diagram.statements.iter().enumerate() {
            if let Statement::Message {
                from, to, label, ..
            } = stmt
            {
                let y = header_end + (idx + 1) as f32 * spacing;
                if let (Some(&x1), Some(&x2)) = (pos.get(from), pos.get(to)) {
                    let cx1 = x1 + box_w / 2.0;
                    let cx2 = x2 + box_w / 2.0;
                    svg.add_line(cx1, y, cx2, y, "stroke:#5b9bd5;stroke-width:2");

                    // Arrowhead
                    let asz = 8.0;
                    let (bx, by, cx, cy) = if cx1 < cx2 {
                        (cx2 - asz, y - asz / 2.0, cx2 - asz, y + asz / 2.0)
                    } else {
                        (cx2 + asz, y - asz / 2.0, cx2 + asz, y + asz / 2.0)
                    };
                    svg.add_path(
                        &format!("M {} {} L {} {} L {} {} Z", cx2, y, bx, by, cx, cy),
                        "fill:#5b9bd5;stroke:none",
                    );

                    // Label
                    svg.add_text(
                        (cx1 + cx2) / 2.0,
                        y - 10.0,
                        label,
                        "text-anchor:middle;font-size:12px;font-family:Arial;fill:#333",
                    );
                }
            }
        }

        svg.build()
    }

    fn render_requirement(&self, diagram: &Diagram) -> String {
        let mut svg =
            SvgBuilder::new(self.width, self.height).with_background_color(&self.background_color);

        let mut reqs: Vec<(String, String)> = Vec::new();
        let mut els: Vec<(String, String)> = Vec::new();
        let mut rels: Vec<(String, String, String)> = Vec::new();

        for stmt in &diagram.statements {
            match stmt {
                Statement::RequirementDef { name, text, .. } => {
                    reqs.push((name.clone(), text.clone()));
                }
                Statement::RequirementElement { name, element_type } => {
                    els.push((name.clone(), element_type.clone()));
                }
                Statement::RequirementRelation {
                    from,
                    to,
                    relation_type,
                } => {
                    rels.push((from.clone(), to.clone(), relation_type.clone()));
                }
                _ => {}
            }
        }

        let nw = 180.0;
        let nh = 60.0;
        let hs = 40.0;
        let vs = 20.0;
        let sx = 30.0;
        let sy = 30.0;

        let items: Vec<String> = reqs
            .iter()
            .map(|(n, _)| n.clone())
            .chain(els.iter().map(|(n, _)| n.clone()))
            .collect();
        let cols = ((items.len() as f32).sqrt().ceil() as usize).max(1);

        let mut pos: std::collections::HashMap<String, (f32, f32)> =
            std::collections::HashMap::new();
        for (i, name) in items.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            pos.insert(
                name.clone(),
                (sx + col as f32 * (nw + hs), sy + row as f32 * (nh + vs)),
            );
        }

        // Draw relations
        for (from, to, rel_type) in &rels {
            if let (Some(&(x1, y1)), Some(&(x2, y2))) = (pos.get(from), pos.get(to)) {
                svg.add_line(
                    x1 + nw / 2.0,
                    y1 + nh / 2.0,
                    x2 + nw / 2.0,
                    y2 + nh / 2.0,
                    "stroke:#666;stroke-width:2",
                );
                let asz = 8.0;
                svg.add_path(
                    &format!(
                        "M {} {} L {} {} L {} {} Z",
                        x2 + nw / 2.0,
                        y2 + nh / 2.0,
                        x2 + nw / 2.0 - asz,
                        y2 + nh / 2.0 - 4.0,
                        x2 + nw / 2.0 - asz,
                        y2 + nh / 2.0 + 4.0
                    ),
                    "fill:#666;stroke:none",
                );
                // Relation label
                svg.add_text(
                    (x1 + x2 + nw) / 2.0,
                    (y1 + y2 + nh) / 2.0 - 10.0,
                    rel_type,
                    "text-anchor:middle;font-size:10px;font-family:Arial;fill:#666",
                );
            }
        }

        // Draw requirements (yellow) and elements (blue)
        for (name, _text) in &reqs {
            if let Some(&(x, y)) = pos.get(name) {
                svg.add_rect(
                    x,
                    y,
                    nw,
                    nh,
                    "fill:#fff9c4;stroke:#f9a825;stroke-width:2;rx:6",
                );
                svg.add_text(x + nw/2.0, y + 18.0, name, "text-anchor:middle;font-size:12px;font-weight:bold;font-family:Arial;fill:#333");
                svg.add_text(
                    x + nw / 2.0,
                    y + 40.0,
                    "Requirement",
                    "text-anchor:middle;font-size:10px;font-family:Arial;fill:#666",
                );
            }
        }
        for (name, _typ) in &els {
            if let Some(&(x, y)) = pos.get(name) {
                svg.add_rect(
                    x,
                    y,
                    nw,
                    nh,
                    "fill:#e3f2fd;stroke:#1976d2;stroke-width:2;rx:6",
                );
                svg.add_text(x + nw/2.0, y + 18.0, name, "text-anchor:middle;font-size:12px;font-weight:bold;font-family:Arial;fill:#333");
                svg.add_text(
                    x + nw / 2.0,
                    y + 40.0,
                    "Element",
                    "text-anchor:middle;font-size:10px;font-family:Arial;fill:#666",
                );
            }
        }

        svg.build()
    }

    fn render_block(&self, diagram: &Diagram) -> String {
        let mut svg =
            SvgBuilder::new(self.width, self.height).with_background_color(&self.background_color);

        let bw = 160.0;
        let bh = 40.0;
        let indent = 60.0;
        let colors = ["#e8f4f8", "#ffe8cc", "#d5f5e3", "#f5e6f0", "#e8e8f8"];

        #[allow(clippy::too_many_arguments)]
        fn render_node(
            svg: &mut SvgBuilder,
            stmts: &[Statement],
            x: f32,
            y: &mut f32,
            d: usize,
            bw: f32,
            bh: f32,
            indent: f32,
            colors: &[&str],
        ) {
            for stmt in stmts {
                if let Statement::BlockNode {
                    label, children, ..
                } = stmt
                {
                    let c = colors[d % colors.len()];
                    svg.add_rect(
                        x,
                        *y,
                        bw,
                        bh,
                        &format!("fill:{};stroke:#333;stroke-width:1;rx:6", c),
                    );
                    svg.add_text(x + bw/2.0, *y + bh/2.0, label,
                        "text-anchor:middle;dominant-baseline:middle;font-size:13px;font-family:Arial");
                    *y += bh + 15.0;
                    if !children.is_empty() {
                        let cy = *y;
                        render_node(svg, children, x + indent, y, d + 1, bw, bh, indent, colors);
                        if *y > cy {
                            *y += 10.0;
                        }
                    }
                }
            }
        }

        let mut y = 30.0;
        render_node(
            &mut svg,
            &diagram.statements,
            30.0,
            &mut y,
            0,
            bw,
            bh,
            indent,
            &colors,
        );
        svg.build()
    }

    fn render_c4(&self, diagram: &Diagram) -> String {
        let mut svg =
            SvgBuilder::new(self.width, self.height).with_background_color(&self.background_color);

        struct El {
            alias: String,
            label: String,
            etype: String,
        }
        let mut els: Vec<El> = Vec::new();
        let mut rels: Vec<(String, String, String)> = Vec::new();

        for stmt in &diagram.statements {
            match stmt {
                Statement::C4Person { alias, label, .. } => els.push(El {
                    alias: alias.clone(),
                    label: label.clone(),
                    etype: "person".into(),
                }),
                Statement::C4System { alias, label, .. } => els.push(El {
                    alias: alias.clone(),
                    label: label.clone(),
                    etype: "system".into(),
                }),
                Statement::C4Container { alias, label, .. } => els.push(El {
                    alias: alias.clone(),
                    label: label.clone(),
                    etype: "container".into(),
                }),
                Statement::C4Component { alias, label, .. } => els.push(El {
                    alias: alias.clone(),
                    label: label.clone(),
                    etype: "component".into(),
                }),
                Statement::C4Rel { from, to, label } => {
                    rels.push((from.clone(), to.clone(), label.clone()))
                }
                _ => {}
            }
        }

        let ew = 150.0;
        let eh = 60.0;
        let hs = 40.0;
        let vs = 30.0;
        let cols = 3usize;

        let mut pos: std::collections::HashMap<String, (f32, f32)> =
            std::collections::HashMap::new();
        for (i, el) in els.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            pos.insert(
                el.alias.clone(),
                (40.0 + col as f32 * (ew + hs), 40.0 + row as f32 * (eh + vs)),
            );
        }

        for (from, to, label) in &rels {
            if let (Some(&(x1, y1)), Some(&(x2, y2))) = (pos.get(from), pos.get(to)) {
                svg.add_line(
                    x1 + ew / 2.0,
                    y1 + eh / 2.0,
                    x2 + ew / 2.0,
                    y2 + eh / 2.0,
                    "stroke:#666;stroke-width:2",
                );
                svg.add_text(
                    (x1 + x2 + ew) / 2.0,
                    (y1 + y2 + eh) / 2.0 - 8.0,
                    label,
                    "text-anchor:middle;font-size:10px;font-family:Arial;fill:#666",
                );
            }
        }

        for el in &els {
            if let Some(&(x, y)) = pos.get(&el.alias) {
                let (color, stroke) = match el.etype.as_str() {
                    "person" => ("#fff9c4", "#f9a825"),
                    "system" => ("#e3f2fd", "#1976d2"),
                    "container" => ("#fce4ec", "#c62828"),
                    _ => ("#f3e5f5", "#7b1fa2"),
                };
                if el.etype == "person" {
                    svg.add_circle(
                        x + ew / 2.0,
                        y + 20.0,
                        12.0,
                        &format!("fill:{};stroke:{};stroke-width:2", color, stroke),
                    );
                    svg.add_text(
                        x + ew / 2.0,
                        y + eh - 10.0,
                        &el.label,
                        "text-anchor:middle;font-size:11px;font-family:Arial;fill:#333",
                    );
                } else {
                    svg.add_rect(
                        x,
                        y,
                        ew,
                        eh,
                        &format!("fill:{};stroke:{};stroke-width:2;rx:6", color, stroke),
                    );
                    svg.add_text(x + ew/2.0, y + eh/2.0, &el.label, "text-anchor:middle;dominant-baseline:middle;font-size:12px;font-family:Arial;fill:#333");
                }
            }
        }

        svg.build()
    }

    fn render_architecture(&self, diagram: &Diagram) -> String {
        let mut svg =
            SvgBuilder::new(self.width, self.height).with_background_color(&self.background_color);

        let mut node_info: std::collections::HashMap<String, (String, String)> =
            std::collections::HashMap::new();
        let mut edges: Vec<(String, String)> = Vec::new();

        for stmt in &diagram.statements {
            match stmt {
                Statement::ArchService { id, label } => {
                    node_info.insert(id.clone(), (label.clone(), "service".into()));
                }
                Statement::ArchDatabase { id, label } => {
                    node_info.insert(id.clone(), (label.clone(), "database".into()));
                }
                Statement::ArchQueue { id, label } => {
                    node_info.insert(id.clone(), (label.clone(), "queue".into()));
                }
                Statement::ArchRelation { from, to } => {
                    edges.push((from.clone(), to.clone()));
                }
                _ => {}
            }
        }

        let nw = 140.0;
        let nh = 50.0;
        let hs = 60.0;
        let vs = 80.0;
        let node_ids: Vec<String> = node_info.keys().cloned().collect();
        let layers = compute_layers(&node_ids, &edges);

        let mut pos: std::collections::HashMap<String, (f32, f32)> =
            std::collections::HashMap::new();
        for (li, layer) in layers.iter().enumerate() {
            let cnt = layer.len() as f32;
            let th = cnt * nh + (cnt - 1.0) * hs;
            let sy = (self.height as f32 - th) / 2.0;
            for (pi, nid) in layer.iter().enumerate() {
                pos.insert(
                    nid.clone(),
                    (40.0 + li as f32 * (nw + vs), sy + pi as f32 * (nh + hs)),
                );
            }
        }

        for (from, to) in &edges {
            if let (Some(&(x1, y1)), Some(&(x2, y2))) = (pos.get(from), pos.get(to)) {
                svg.add_line(
                    x1 + nw / 2.0,
                    y1 + nh / 2.0,
                    x2 + nw / 2.0,
                    y2 + nh / 2.0,
                    "stroke:#666;stroke-width:2",
                );
            }
        }

        for (id, (label, etype)) in &node_info {
            if let Some(&(x, y)) = pos.get(id) {
                let (color, stroke) = match etype.as_str() {
                    "database" => ("#e8f5e9", "#4caf50"),
                    "queue" => ("#fff3e0", "#ff9800"),
                    _ => ("#e3f2fd", "#2196f3"),
                };
                svg.add_rect(
                    x,
                    y,
                    nw,
                    nh,
                    &format!("fill:{};stroke:{};stroke-width:2;rx:8", color, stroke),
                );
                svg.add_text(
                    x + nw / 2.0,
                    y + nh / 2.0,
                    label,
                    "text-anchor:middle;dominant-baseline:middle;font-size:12px;font-family:Arial",
                );
            }
        }

        svg.build()
    }

    fn render_xychart(&self, diagram: &Diagram) -> String {
        let mut svg =
            SvgBuilder::new(self.width, self.height).with_background_color(&self.background_color);

        let mut title = String::new();
        let mut x_label = String::new();
        let mut cats: Vec<String> = Vec::new();
        let mut y_label = String::new();
        let mut y_max = 100.0;
        let mut bars: Vec<Vec<f64>> = Vec::new();
        let mut lines: Vec<Vec<f64>> = Vec::new();

        for stmt in &diagram.statements {
            match stmt {
                Statement::XyTitle(t) => title = t.clone(),
                Statement::XyXAxis { label, categories } => {
                    x_label = label.clone();
                    cats = categories.clone();
                }
                Statement::XyYAxis { label, min: _, max } => {
                    y_label = label.clone();
                    y_max = *max;
                }
                Statement::XyBar { data } => bars.push(data.clone()),
                Statement::XyLine { data } => lines.push(data.clone()),
                _ => {}
            }
        }

        let m = 50.0;
        let pw = self.width as f32 - 2.0 * m;
        let ph = self.height as f32 - 2.0 * m;
        let n = cats.len().max(1);
        let bw = pw / n as f32;

        // Y axis
        svg.add_line(m, m, m, m + ph, "stroke:#333;stroke-width:2");
        if !y_label.is_empty() {
            svg.add_text(
                15.0,
                m + ph / 2.0,
                &y_label,
                "text-anchor:middle;font-size:12px;font-family:Arial;fill:#666",
            );
        }

        // X axis
        svg.add_line(m, m + ph, m + pw, m + ph, "stroke:#333;stroke-width:2");

        // Bars
        for series in &bars {
            for (i, val) in series.iter().enumerate() {
                if i >= n {
                    break;
                }
                let x = m + i as f32 * bw + bw * 0.15;
                let w = bw * 0.7;
                let h = (*val as f32 / y_max as f32) * ph;
                svg.add_rect(x, m + ph - h, w, h, "fill:#4A90D9;stroke:none");
            }
        }

        // Lines
        for series in &lines {
            let pts: Vec<(f32, f32)> = series
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    (
                        m + i as f32 * bw + bw / 2.0,
                        m + ph - (*v as f32 / y_max as f32) * ph,
                    )
                })
                .collect();
            for win in pts.windows(2) {
                svg.add_line(
                    win[0].0,
                    win[0].1,
                    win[1].0,
                    win[1].1,
                    "stroke:#e74c3c;stroke-width:2;fill:none",
                );
            }
            for pt in &pts {
                svg.add_circle(pt.0, pt.1, 4.0, "fill:#e74c3c;stroke:white;stroke-width:2");
            }
        }

        // X-axis title
        if !x_label.is_empty() {
            svg.add_text(
                m + pw / 2.0,
                m + ph + 45.0,
                &x_label,
                "text-anchor:middle;font-size:11px;font-family:Arial;fill:#666",
            );
        }

        // X-axis category labels
        for (i, cat) in cats.iter().enumerate() {
            svg.add_text(
                m + i as f32 * bw + bw / 2.0,
                m + ph + 25.0,
                cat,
                "text-anchor:middle;font-size:10px;font-family:Arial;fill:#666",
            );
        }

        // Title
        if !title.is_empty() {
            svg.add_text(
                self.width as f32 / 2.0,
                20.0,
                &title,
                "text-anchor:middle;font-size:16px;font-weight:bold;font-family:Arial",
            );
        }

        svg.build()
    }

    fn render_sankey(&self, diagram: &Diagram) -> String {
        let mut svg =
            SvgBuilder::new(self.width, self.height).with_background_color(&self.background_color);

        let mut links: Vec<(String, String, f64)> = Vec::new();
        for stmt in &diagram.statements {
            if let Statement::SankeyLink {
                source,
                target,
                value,
            } = stmt
            {
                links.push((source.clone(), target.clone(), *value));
            }
        }

        if links.is_empty() {
            return svg.build();
        }

        let total: f64 = links.iter().map(|(_, _, v)| v).sum();
        let colors = [
            "#4A90D9", "#5CB85C", "#F0AD4E", "#7B68EE", "#f44336", "#00BCD4", "#FF9800", "#9C27B0",
        ];
        let lw = 200.0;
        let gap = 10.0;
        let left_x = 80.0;
        let right_x = self.width as f32 - 80.0 - lw;
        let h = self.height as f32 - 20.0;

        let mut left_y = 10.0;
        let mut right_y = 10.0;
        let _prev_left = 0.0f32;

        for (i, (source, target, value)) in links.iter().enumerate() {
            let frac = (*value / total) as f32;
            let bw = h * frac;
            let c = colors[i % colors.len()];

            // Left node
            svg.add_rect(
                left_x,
                left_y,
                lw,
                bw,
                &format!("fill:{};stroke:#333;stroke-width:1;rx:4", c),
            );
            svg.add_text(left_x + lw/2.0, left_y + bw/2.0, source,
                "text-anchor:middle;dominant-baseline:middle;font-size:11px;font-family:Arial;fill:white;font-weight:bold");

            // Right node
            svg.add_rect(
                right_x,
                right_y,
                lw,
                bw,
                &format!("fill:{};stroke:#333;stroke-width:1;rx:4", c),
            );
            svg.add_text(right_x + lw/2.0, right_y + bw/2.0, target,
                "text-anchor:middle;dominant-baseline:middle;font-size:11px;font-family:Arial;fill:white;font-weight:bold");

            // Flow band (simple polygon)
            svg.add_polygon(
                &[
                    (left_x + lw, left_y),
                    (left_x + lw, left_y + bw),
                    (right_x, right_y + bw),
                    (right_x, right_y),
                ],
                &format!("fill:{};fill-opacity:0.3;stroke:none", c),
            );

            // Value label
            let mid_y = left_y + bw / 2.0;
            svg.add_text((left_x + lw + right_x) / 2.0, mid_y, &format!("{:.0}", value),
                "text-anchor:middle;dominant-baseline:middle;font-size:10px;font-family:Arial;fill:#333");

            left_y += bw + gap;
            right_y += bw + gap;
        }

        svg.build()
    }

    fn render_treemap(&self, diagram: &Diagram) -> String {
        let mut svg =
            SvgBuilder::new(self.width, self.height).with_background_color(&self.background_color);

        let mut items: Vec<(String, f64)> = Vec::new();
        for stmt in &diagram.statements {
            if let Statement::TreemapItem { label, value } = stmt {
                items.push((label.clone(), *value));
            }
        }
        if items.is_empty() {
            return svg.build();
        }

        let total: f64 = items.iter().map(|(_, v)| v).sum();
        let colors = [
            "#4A90D9", "#5CB85C", "#F0AD4E", "#7B68EE", "#f44336", "#00BCD4", "#FF9800", "#9C27B0",
            "#607D8B", "#795548",
        ];
        let m = 10.0;
        let w = self.width as f32 - 2.0 * m;
        let h = self.height as f32 - 2.0 * m;
        let mut y = m;
        let min_h = 30.0;

        for (i, (label, value)) in items.iter().enumerate() {
            let frac = (*value / total) as f32;
            let ih = ((h * frac) * items.len() as f32)
                .max(min_h)
                .min(h - (y - m));
            let c = colors[i % colors.len()];
            svg.add_rect(
                m,
                y,
                w,
                ih,
                &format!("fill:{};stroke:white;stroke-width:2", c),
            );
            svg.add_text(m + 10.0, y + ih/2.0, label,
                "text-anchor:start;dominant-baseline:middle;font-size:14px;font-family:Arial;fill:white;font-weight:bold");
            let val_str = format!("{:.0}", value);
            svg.add_text(m + w - 10.0, y + ih/2.0, &val_str,
                "text-anchor:end;dominant-baseline:middle;font-size:12px;font-family:Arial;fill:white;fill-opacity:0.8");
            y += ih + 2.0;
        }

        svg.build()
    }

    #[cfg(feature = "png")]
    pub fn render_png(&self, diagram: &Diagram) -> Result<Vec<u8>, String> {
        let svg_data = self.render(diagram)?;
        // Ensure generic fallback font families are available for cross-platform rendering
        let svg_data = svg_data
            .replace("font-family:Arial", "font-family:Arial, sans-serif")
            .replace("font-family: Arial", "font-family: Arial, sans-serif");

        // Load system fonts for text rendering in PNG output
        let mut fontdb = usvg::fontdb::Database::new();
        fontdb.load_system_fonts();

        // Select best available sans-serif font (preferring CJK fonts for Chinese support)
        let mut sans_family = "sans-serif".to_string();
        for candidate in &[
            "Noto Sans CJK SC",
            "Noto Sans SC",
            "PingFang SC",
            "Microsoft YaHei",
            "WenQuanYi Micro Hei",
            "DejaVu Sans",
            "Arial",
        ] {
            if fontdb
                .faces()
                .any(|f| f.families.iter().any(|(fam, _)| fam == candidate))
            {
                fontdb.set_sans_serif_family(*candidate);
                sans_family = (*candidate).to_string();
                break;
            }
        }

        for candidate in &[
            "Noto Sans Mono CJK SC",
            "DejaVu Sans Mono",
            "Courier New",
            "monospace",
        ] {
            if fontdb
                .faces()
                .any(|f| f.families.iter().any(|(fam, _)| fam == candidate))
            {
                fontdb.set_monospace_family(*candidate);
                break;
            }
        }

        let mut opt = usvg::Options {
            fontdb: std::sync::Arc::new(fontdb),
            ..Default::default()
        };
        opt.font_family = sans_family;

        let rtree =
            usvg::Tree::from_str(&svg_data, &opt).map_err(|e| format!("SVG parse error: {}", e))?;

        // Use actual SVG dimensions from the tree, not the renderer's default
        let svg_size = rtree.size();
        let w = (svg_size.width() * self.scale).max(1.0) as u32;
        let h = (svg_size.height() * self.scale).max(1.0) as u32;
        let mut pixmap = tiny_skia::Pixmap::new(w, h).ok_or("Failed to create Pixmap")?;

        resvg::render(
            &rtree,
            tiny_skia::Transform::from_scale(self.scale, self.scale),
            &mut pixmap.as_mut(),
        );

        pixmap
            .encode_png()
            .map_err(|e| format!("PNG encode error: {}", e))
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_class_style(style_str: &str) -> (HashMap<String, String>, Vec<String>) {
    let mut props = HashMap::new();
    let mut other_styles = Vec::new();
    for part in style_str.split(&[',', ';'][..]) {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            let key = k.trim().to_lowercase();
            let val = v.trim().to_string();
            match key.as_str() {
                "fill" | "stroke" | "stroke-width" | "color" => {
                    props.insert(key, val);
                }
                _ => {
                    other_styles.push(format!("{}:{}", key, val));
                }
            }
        }
    }
    (props, other_styles)
}

/// Compute layers using tree-aware depth algorithm for better org chart layout.
fn compute_tree_layers(
    node_ids: &[String],
    edges: &[(String, String)],
    subgraphs: &[Subgraph],
) -> Vec<Vec<String>> {
    let node_set: HashSet<&str> = node_ids.iter().map(|s| s.as_str()).collect();

    // 1. Detect back-edges (cycles) using DFS
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for (from, to) in edges {
        if node_set.contains(from.as_str()) && node_set.contains(to.as_str()) {
            adj.entry(from.as_str()).or_default().push(to.as_str());
        }
    }

    let mut visited: HashSet<&str> = HashSet::new();
    let mut on_stack: HashSet<&str> = HashSet::new();
    let mut back_edges: HashSet<(&str, &str)> = HashSet::new();

    fn dfs<'a>(
        u: &'a str,
        adj: &HashMap<&str, Vec<&'a str>>,
        visited: &mut HashSet<&'a str>,
        on_stack: &mut HashSet<&'a str>,
        back_edges: &mut HashSet<(&'a str, &'a str)>,
    ) {
        visited.insert(u);
        on_stack.insert(u);
        if let Some(neighbors) = adj.get(u) {
            for &v in neighbors {
                if on_stack.contains(v) {
                    back_edges.insert((u, v));
                } else if !visited.contains(v) {
                    dfs(v, adj, visited, on_stack, back_edges);
                }
            }
        }
        on_stack.remove(u);
    }

    // Start DFS from in-degree 0 nodes first
    let mut in_degrees: HashMap<&str, usize> = HashMap::new();
    for id in node_ids {
        in_degrees.insert(id.as_str(), 0);
    }
    for (from, to) in edges {
        if node_set.contains(from.as_str()) && node_set.contains(to.as_str()) {
            *in_degrees.entry(to.as_str()).or_insert(0) += 1;
        }
    }
    let mut roots: Vec<&str> = node_ids
        .iter()
        .map(|s| s.as_str())
        .filter(|&id| *in_degrees.get(id).unwrap_or(&0) == 0)
        .collect();

    if roots.is_empty() {
        roots = node_ids.iter().map(|s| s.as_str()).collect();
    }

    for &r in &roots {
        if !visited.contains(r) {
            dfs(r, &adj, &mut visited, &mut on_stack, &mut back_edges);
        }
    }
    for id in node_ids {
        if !visited.contains(id.as_str()) {
            dfs(
                id.as_str(),
                &adj,
                &mut visited,
                &mut on_stack,
                &mut back_edges,
            );
        }
    }

    // 2. Build forward DAG (excluding back-edges)
    let mut dag_adj: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut dag_in_degree: HashMap<&str, usize> = HashMap::new();
    for id in node_ids {
        dag_in_degree.insert(id.as_str(), 0);
        dag_adj.entry(id.as_str()).or_default();
    }
    for (from, to) in edges {
        if node_set.contains(from.as_str())
            && node_set.contains(to.as_str())
            && !back_edges.contains(&(from.as_str(), to.as_str()))
        {
            dag_adj.entry(from.as_str()).or_default().push(to.as_str());
            *dag_in_degree.entry(to.as_str()).or_insert(0) += 1;
        }
    }

    // 3. Topological sort on DAG to compute longest path depths
    let mut zero_in: std::collections::VecDeque<&str> = std::collections::VecDeque::new();
    for (id, deg) in &dag_in_degree {
        if *deg == 0 {
            zero_in.push_back(*id);
        }
    }

    let mut depth_map: HashMap<&str, usize> = HashMap::new();
    for &id in &zero_in {
        depth_map.insert(id, 0);
    }

    while let Some(u) = zero_in.pop_front() {
        let u_depth = *depth_map.get(u).unwrap_or(&0);
        if let Some(neighbors) = dag_adj.get(u) {
            for &v in neighbors {
                let v_depth = depth_map.entry(v).or_insert(0);
                *v_depth = (*v_depth).max(u_depth + 1);

                let deg = dag_in_degree.get_mut(v).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    zero_in.push_back(v);
                }
            }
        }
    }

    // Ensure all nodes have a depth
    for id in node_ids {
        depth_map.entry(id.as_str()).or_insert(0);
    }

    // 4. Align subgraphs so downstream subgraphs start after upstream finishes
    if !subgraphs.is_empty() {
        let mut node_to_sg: HashMap<&str, usize> = HashMap::new();
        let mut sg_nodes: HashMap<usize, Vec<&str>> = HashMap::new();
        for (sg_idx, sg) in subgraphs.iter().enumerate() {
            for stmt in &sg.statements {
                match stmt {
                    Statement::NodeDef { id, .. } => {
                        node_to_sg.insert(id.as_str(), sg_idx);
                        sg_nodes.entry(sg_idx).or_default().push(id.as_str());
                    }
                    Statement::EdgeDef { from, to, .. } => {
                        node_to_sg.insert(from.as_str(), sg_idx);
                        node_to_sg.insert(to.as_str(), sg_idx);
                        sg_nodes.entry(sg_idx).or_default().push(from.as_str());
                        sg_nodes.entry(sg_idx).or_default().push(to.as_str());
                    }
                    _ => {}
                }
            }
        }

        let mut sg_adj: HashMap<usize, HashSet<usize>> = HashMap::new();
        for (from, to) in edges {
            if !back_edges.contains(&(from.as_str(), to.as_str())) {
                if let (Some(&sg_from), Some(&sg_to)) =
                    (node_to_sg.get(from.as_str()), node_to_sg.get(to.as_str()))
                {
                    if sg_from != sg_to {
                        sg_adj.entry(sg_from).or_default().insert(sg_to);
                    }
                }
            }
        }

        for _ in 0..subgraphs.len() {
            for (&sg_from, to_set) in &sg_adj {
                let max_from = sg_nodes
                    .get(&sg_from)
                    .and_then(|nodes| nodes.iter().map(|n| *depth_map.get(n).unwrap_or(&0)).max())
                    .unwrap_or(0);

                for &sg_to in to_set {
                    let min_to = sg_nodes
                        .get(&sg_to)
                        .and_then(|nodes| {
                            nodes.iter().map(|n| *depth_map.get(n).unwrap_or(&0)).min()
                        })
                        .unwrap_or(0);

                    if min_to <= max_from {
                        let diff = max_from + 1 - min_to;
                        let mut to_shift = HashSet::new();
                        let mut q = std::collections::VecDeque::new();
                        q.push_back(sg_to);
                        to_shift.insert(sg_to);
                        while let Some(curr) = q.pop_front() {
                            if let Some(nexts) = sg_adj.get(&curr) {
                                for &nxt in nexts {
                                    if to_shift.insert(nxt) {
                                        q.push_back(nxt);
                                    }
                                }
                            }
                        }
                        for shifted_sg in to_shift {
                            if let Some(nodes) = sg_nodes.get(&shifted_sg) {
                                for n in nodes {
                                    if let Some(d) = depth_map.get_mut(n) {
                                        *d += diff;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Re-relax downstream forward edges
        for _ in 0..node_ids.len() {
            let mut changed = false;
            for (from, to) in edges {
                if !back_edges.contains(&(from.as_str(), to.as_str())) {
                    if let Some(&d_from) = depth_map.get(from.as_str()) {
                        if let Some(d_to) = depth_map.get_mut(to.as_str()) {
                            if *d_to < d_from + 1 {
                                *d_to = d_from + 1;
                                changed = true;
                            }
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }

    // Group nodes by depth
    let mut max_depth = 0;
    for &d in depth_map.values() {
        max_depth = max_depth.max(d);
    }

    let mut layers: Vec<Vec<String>> = Vec::new();
    for d in 0..=max_depth {
        let layer: Vec<String> = node_ids
            .iter()
            .filter(|id| *depth_map.get(id.as_str()).unwrap_or(&usize::MAX) == d)
            .cloned()
            .collect();
        if !layer.is_empty() {
            layers.push(layer);
        }
    }

    layers
}

/// Kahn's algorithm for topological sort layering (fallback).
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
        let zero_in: Vec<&str> = remaining
            .iter()
            .filter(|id| *in_degree.get(*id).unwrap_or(&0) == 0)
            .cloned()
            .collect();

        if zero_in.is_empty() {
            let layer: Vec<String> = remaining.iter().map(|s| s.to_string()).collect();
            layers.push(layer);
            break;
        }

        let layer: Vec<String> = zero_in.iter().map(|s| s.to_string()).collect();
        layers.push(layer);

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

/// Flatten nested statements from Block variants into a flat list.
fn flatten_seq_stmts(stmts: &[Statement]) -> Vec<Statement> {
    let mut result = Vec::new();
    for stmt in stmts {
        match stmt {
            Statement::Block { statements, .. } => {
                result.extend(flatten_seq_stmts(statements));
            }
            other => result.push(other.clone()),
        }
    }
    result
}

/// Process sequence diagram statements recursively.
#[allow(clippy::too_many_arguments)]
fn process_seq_stmts(
    svg: &mut SvgBuilder,
    center_map: &HashMap<&str, f32>,
    stmts: &[Statement],
    y_pos: &mut f32,
    line_height: f32,
    activations: &mut HashMap<String, f32>,
    box_width: f32,
    start_x: f32,
) {
    for stmt in stmts {
        match stmt {
            Statement::Message {
                from,
                to,
                label,
                arrow_type,
            } => {
                let cx1 = center_map.get(from.as_str()).copied().unwrap_or(0.0);
                let cx2 = center_map.get(to.as_str()).copied().unwrap_or(0.0);

                if from == to {
                    // Self-message loop!
                    let loop_w = 34.0;
                    let loop_h = 22.0;
                    let path = format!(
                        "M {} {} L {} {} L {} {} L {} {}",
                        cx1,
                        *y_pos,
                        cx1 + loop_w,
                        *y_pos,
                        cx1 + loop_w,
                        *y_pos + loop_h,
                        cx1,
                        *y_pos + loop_h
                    );
                    svg.add_path(&path, "stroke:#334155;stroke-width:1.5;fill:none");
                    // Arrow head pointing left towards cx1
                    svg.add_polygon(
                        &[
                            (cx1, *y_pos + loop_h),
                            (cx1 + 6.0, *y_pos + loop_h - 3.5),
                            (cx1 + 6.0, *y_pos + loop_h + 3.5),
                        ],
                        "fill:#334155",
                    );
                    // Label placed to the right of the loop, completely clear of the lifeline
                    svg.add_text(
                        cx1 + loop_w + 6.0,
                        *y_pos + loop_h / 2.0 + 3.0,
                        label,
                        "text-anchor:start;dominant-baseline:middle;font-size:11px;fill:#1e293b;font-family:Arial, sans-serif",
                    );
                    *y_pos += line_height.max(loop_h + 10.0);
                } else {
                    let style = match arrow_type {
                        ArrowType::Dashed | ArrowType::DashedCross | ArrowType::DashedOpen => {
                            "stroke:#334155;stroke-width:1.5;stroke-dasharray:6,4;fill:none"
                        }
                        _ => "stroke:#334155;stroke-width:1.5;fill:none",
                    };

                    svg.add_line(cx1, *y_pos, cx2, *y_pos, style);

                    // Arrow head for solid/dashed/cross types (not open)
                    if !matches!(arrow_type, ArrowType::SolidOpen | ArrowType::DashedOpen) {
                        let arrow_size = 7.0;
                        let dir = if cx2 >= cx1 { 1.0 } else { -1.0 };
                        svg.add_polygon(
                            &[
                                (cx2, *y_pos),
                                (cx2 - dir * arrow_size, *y_pos - 4.0),
                                (cx2 - dir * arrow_size, *y_pos + 4.0),
                            ],
                            "fill:#334155",
                        );
                    }

                    // Message label with dynamic text background
                    let mid_x = (cx1 + cx2) / 2.0;
                    let text_chars = label.chars().count() as f32;
                    let lw = (text_chars * 12.0 + 16.0).max(40.0);
                    let lh = 18.0;
                    svg.add_rect(
                        mid_x - lw / 2.0,
                        *y_pos - lh / 2.0,
                        lw,
                        lh,
                        "fill:#ffffff;stroke:none;",
                    );
                    svg.add_text(
                        mid_x,
                        *y_pos + 3.0,
                        label,
                        "text-anchor:middle;font-size:11px;fill:#1e293b;font-family:Arial, sans-serif",
                    );
                    *y_pos += line_height;
                }
            }
            Statement::Note {
                target,
                text,
                position,
            } => {
                let cx = center_map.get(target.as_str()).copied().unwrap_or(0.0);
                let (nx, note_w) = match position {
                    NotePosition::Right => (cx + 20.0, 160.0),
                    NotePosition::Left => (cx - box_width / 2.0 - 170.0, 160.0),
                    NotePosition::Over => (cx - 80.0, 160.0),
                };
                let ny = *y_pos - 15.0;
                let note_h = 36.0;
                svg.add_rect(
                    nx,
                    ny,
                    note_w,
                    note_h,
                    "fill:#fffde7;stroke:#e6c300;stroke-width:1;rx:4;",
                );
                svg.add_text(
                    nx + note_w / 2.0,
                    ny + note_h / 2.0,
                    text,
                    "text-anchor:middle;dominant-baseline:middle;font-size:11px;font-family:Arial",
                );
                *y_pos += line_height;
            }
            Statement::Block {
                keyword,
                condition,
                statements,
            } => {
                let block_start = *y_pos - 10.0;

                // 计算条件框的左右边界（横跨所有参与者）
                let blk_left = if let Some(min_x) =
                    center_map.values().min_by(|a, b| a.partial_cmp(b).unwrap())
                {
                    min_x - box_width / 2.0 - 20.0
                } else {
                    start_x - 20.0
                };

                let blk_right = if let Some(max_x) =
                    center_map.values().max_by(|a, b| a.partial_cmp(b).unwrap())
                {
                    max_x + box_width / 2.0 + 20.0
                } else {
                    start_x + box_width + 20.0
                };

                process_seq_stmts(
                    svg,
                    center_map,
                    statements,
                    y_pos,
                    line_height,
                    activations,
                    box_width,
                    start_x,
                );
                let block_end = *y_pos;

                // Draw modern condition box
                if keyword == "else" {
                    // Divider line for else block
                    svg.add_line(
                        blk_left,
                        block_start,
                        blk_right,
                        block_start,
                        "stroke:#94a3b8;stroke-width:1;stroke-dasharray:5,5",
                    );
                } else {
                    // Enclosing container box
                    svg.add_rect(
                        blk_left,
                        block_start,
                        blk_right - blk_left,
                        block_end - block_start,
                        "fill:#f8fafc;fill-opacity:0.5;stroke:#94a3b8;stroke-width:1;rx:4;ry:4",
                    );
                }

                let block_label = if let Some(cond) = condition {
                    format!("{} [{}]", keyword, cond)
                } else {
                    keyword.clone()
                };
                let label_chars = block_label.chars().count() as f32;
                let label_width = (label_chars * 7.5 + 10.0).max(30.0);
                svg.add_rect(
                    blk_left,
                    block_start,
                    label_width,
                    16.0,
                    "fill:#f1f5f9;stroke:#cbd5e1;stroke-width:1;rx:3",
                );
                svg.add_text(
                    blk_left + 4.0,
                    block_start + 11.0,
                    &block_label,
                    "text-anchor:start;font-size:10px;fill:#475569;font-weight:bold;font-family:Arial, sans-serif",
                );
                // Extra breathing room after block
                *y_pos += line_height;
            }
            Statement::Activate { participant } => {
                activations.insert(participant.clone(), *y_pos);
                *y_pos += 5.0;
            }
            Statement::Deactivate { participant } => {
                if let Some(&start_y) = activations.get(participant) {
                    if let Some(&cx) = center_map.get(participant.as_str()) {
                        // Activation box showing active lifecycle
                        svg.add_rect(
                            cx - 5.0,
                            start_y,
                            10.0,
                            *y_pos - start_y,
                            "fill:#e2e8f0;stroke:#94a3b8;stroke-width:1;rx:2",
                        );
                    }
                }
                *y_pos += 5.0;
            }
            _ => {}
        }
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
            title: None,
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
            title: None,
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
            title: None,
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
            title: None,
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
            title: None,
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
            title: None,
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
            title: None,
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
            title: None,
            statements: vec![Statement::NodeDef {
                id: "A".to_string(),
                label: Some("Start".to_string()),
                shape: NodeShape::DoubleCircle,
            }],
        };

        let renderer = Renderer::new();
        let svg = renderer.render(&diagram).unwrap();
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
            title: None,
            statements: vec![Statement::NodeDef {
                id: "A".to_string(),
                label: Some("Proc".to_string()),
                shape: NodeShape::Subroutine,
            }],
        };

        let renderer = Renderer::new();
        let svg = renderer.render(&diagram).unwrap();
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
            title: None,
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

    #[test]
    fn test_compute_layers_simple_linear() {
        let nodes = vec!["A".into(), "B".into(), "C".into()];
        let edges = vec![("A".into(), "B".into()), ("B".into(), "C".into())];
        let layers = compute_layers(&nodes, &edges);
        assert_eq!(layers.len(), 3);
    }

    #[test]
    fn test_compute_layers_diamond() {
        let nodes = vec!["A".into(), "B".into(), "C".into(), "D".into()];
        let edges = vec![
            ("A".into(), "B".into()),
            ("A".into(), "C".into()),
            ("B".into(), "D".into()),
            ("C".into(), "D".into()),
        ];
        let layers = compute_layers(&nodes, &edges);
        assert_eq!(layers.len(), 3);
    }

    #[test]
    fn test_compute_layers_with_cycle() {
        let nodes = vec!["A".into(), "B".into(), "C".into()];
        let edges = vec![
            ("A".into(), "B".into()),
            ("B".into(), "C".into()),
            ("C".into(), "A".into()),
        ];
        let layers = compute_layers(&nodes, &edges);
        // Cycle detected: all nodes in one layer
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].len(), 3);
    }

    #[test]
    fn test_compute_layers_disconnected() {
        let nodes = vec!["A".into(), "B".into(), "X".into(), "Y".into()];
        let edges = vec![("A".into(), "B".into())];
        let layers = compute_layers(&nodes, &edges);
        assert!(
            !layers.is_empty(),
            "Should compute layers for disconnected graph"
        );
    }

    #[test]
    fn test_compute_layers_single_node() {
        let nodes = vec!["A".into()];
        let edges: Vec<(String, String)> = vec![];
        let layers = compute_layers(&nodes, &edges);
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0], vec!["A"]);
    }

    #[test]
    fn test_renderer_flowchart_with_edge_labels() {
        use crate::parser::DiagramType;
        let diagram = Diagram {
            diagram_type: DiagramType::Flowchart,
            direction: None,
            title: None,
            subgraphs: vec![],
            statements: vec![Statement::EdgeDef {
                from: "A".into(),
                to: "B".into(),
                label: Some("label1".into()),
            }],
        };
        let renderer = Renderer::new();
        let svg = renderer.render(&diagram).unwrap();
        assert!(svg.contains("label1"), "Should contain edge label");
    }

    #[test]
    fn test_compute_layers_multiple_sources() {
        let nodes = vec!["A".into(), "B".into(), "C".into(), "D".into()];
        let edges = vec![
            ("A".into(), "C".into()),
            ("B".into(), "C".into()),
            ("C".into(), "D".into()),
        ];
        let layers = super::compute_layers(&nodes, &edges);
        assert_eq!(layers.len(), 3);
    }

    #[test]
    fn test_compute_layers_complex_dag() {
        let nodes = vec![
            "A".into(),
            "B".into(),
            "C".into(),
            "D".into(),
            "E".into(),
            "F".into(),
        ];
        let edges = vec![
            ("A".into(), "B".into()),
            ("A".into(), "C".into()),
            ("B".into(), "D".into()),
            ("C".into(), "D".into()),
            ("D".into(), "E".into()),
            ("E".into(), "F".into()),
        ];
        let layers = super::compute_layers(&nodes, &edges);
        assert_eq!(layers.len(), 5);
    }

    #[test]
    fn test_renderer_flowchart_diamond() {
        use crate::parser::DiagramType;
        let diagram = Diagram {
            diagram_type: DiagramType::Flowchart,
            direction: None,
            title: None,
            subgraphs: vec![],
            statements: vec![Statement::NodeDef {
                id: "D".into(),
                label: Some("Decision".into()),
                shape: NodeShape::Diamond,
            }],
        };
        let svg = Renderer::new().render(&diagram).unwrap();
        assert!(svg.contains("Decision"));
    }

    #[test]
    fn test_order_flow_render() {
        let code = include_str!("../../examples/order_flow.mmd");
        let mut parser = crate::parser::Parser::new(code);
        let diagram = parser.parse().unwrap();
        let svg = Renderer::new().render(&diagram).unwrap();
        assert!(svg.contains("用户提交订单"));
        assert!(svg.contains("风控校验与订单创建"));
        assert!(svg.contains("库存预占"));
        assert!(svg.contains("支付处理"));
        assert!(svg.contains("仓储调度与履约"));
        assert!(svg.contains("签收与售后"));
        // Assert class styles are applied
        assert!(svg.contains("#1f77b4"));
        assert!(svg.contains("#fff3cd"));
        assert!(svg.contains("#e7f1ff"));
        assert!(svg.contains("#f8d7da"));
        assert!(svg.contains("#d4edda"));
    }
}
