use crate::parser::{ArrowType, ClassVisibility, DiagramType, NotePosition};
use crate::parser::{Diagram, NodeShape, Statement};
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
        let h_spacing = 60.0;
        let v_spacing = 100.0;
        let padding = 50.0;

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
                    let x = padding + layer_idx as f32 * (node_width + v_spacing);
                    let y = start_x + pos_idx as f32 * (node_height + h_spacing);
                    (x, y)
                } else {
                    let x = start_x + pos_idx as f32 * (node_width + h_spacing);
                    let y = padding + layer_idx as f32 * (node_height + v_spacing);
                    (x, y)
                };
                node_positions.insert(node_id.clone(), (x, y));
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

                svg.add_line(cx1, cy1, cx2, cy2, "stroke:black;stroke-width:2");
                svg.add_arrow(cx2, cy2);

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
                cx_val - box_width / 2.0,
                lifeline_top,
                cx_val + box_width / 2.0,
                top_margin + box_height,
                "fill:none;stroke:black;stroke-width:1",
            );
            svg.add_line(
                *cx_val,
                top_margin + box_height + 10.0,
                *cx_val,
                y_pos + 10.0,
                "stroke:black;stroke-width:1;stroke-dasharray:4,4",
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
                "fill:white;stroke:black;stroke-width:1;rx:4",
            );
            svg.add_text(
                *cx,
                top_margin + box_height / 2.0,
                display_name,
                "text-anchor:middle;dominant-baseline:middle;font-size:13px;font-family:Arial",
            );
            if *display_name != *id {
                svg.add_text(
                    *cx,
                    top_margin + box_height + 4.0,
                    id,
                    "text-anchor:middle;font-size:9px;fill:gray;font-family:Arial",
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
        let header_h = 30.0;
        let member_h = 22.0;
        let spacing_x = 60.0;
        let spacing_y = 80.0;
        let start_x = 50.0;
        let start_y = 50.0;

        let class_list: Vec<&String> = classes.keys().collect();
        let cols = ((class_list.len() as f32).sqrt().ceil() as usize).max(1);

        for (i, class_name) in class_list.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            let x = start_x + col as f32 * (class_w + spacing_x);
            let y = start_y + row as f32 * (header_h + spacing_y);

            let (_stereotype, members) = &classes[class_name.to_owned()];
            let total_h = header_h + members.len() as f32 * member_h;

            svg.add_rect(
                x,
                y,
                class_w,
                total_h,
                "fill:white;stroke:black;stroke-width:1",
            );
            svg.add_line(
                x,
                y + header_h,
                x + class_w,
                y + header_h,
                "stroke:black;stroke-width:1",
            );

            svg.add_text(
                x + class_w / 2.0, y + header_h / 2.0, class_name,
                "text-anchor:middle;dominant-baseline:middle;font-size:13px;font-weight:bold;font-family:Arial",
            );

            for (j, (_, member_text, _)) in members.iter().enumerate() {
                svg.add_text(
                    x + 8.0,
                    y + header_h + j as f32 * member_h + member_h / 2.0,
                    member_text,
                    "dominant-baseline:middle;font-size:11px;font-family:monospace",
                );
            }
        }

        for stmt in &diagram.statements {
            if let Statement::ClassRelation {
                from, to, label, ..
            } = stmt
            {
                let pos_from = class_list.iter().position(|n| *n == from);
                let pos_to = class_list.iter().position(|n| *n == to);
                if let (Some(ifrom), Some(ito)) = (pos_from, pos_to) {
                    let col_from = ifrom % cols;
                    let row_from = ifrom / cols;
                    let col_to = ito % cols;
                    let row_to = ito / cols;
                    let x1 = start_x + col_from as f32 * (class_w + spacing_x) + class_w / 2.0;
                    let y1 = start_y + row_from as f32 * (header_h + spacing_y);
                    let x2 = start_x + col_to as f32 * (class_w + spacing_x) + class_w / 2.0;
                    let y2 = start_y + row_to as f32 * (header_h + spacing_y);
                    svg.add_line(x1, y1, x2, y2, "stroke:black;stroke-width:1;fill:none");
                    if let Some(lbl) = label {
                        svg.add_text(
                            (x1 + x2) / 2.0,
                            (y1 + y2) / 2.0 - 6.0,
                            lbl,
                            "text-anchor:middle;font-size:10px;font-family:Arial",
                        );
                    }
                    svg.add_polygon(
                        &[(x2, y2 - 6.0), (x2 - 6.0, y2), (x2, y2 + 6.0)],
                        "fill:white;stroke:black;stroke-width:1",
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

        for (name, _label) in &state_labels {
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

        fn render_mm_node(svg: &mut SvgBuilder, label: &str, x: f32, y: f32) {
            svg.add_rect(
                x - 60.0,
                y - 15.0,
                120.0,
                30.0,
                "fill:#e8f4f8;stroke:#45b7d1;stroke-width:1;rx:5",
            );
            svg.add_text(
                x,
                y + 4.0,
                label,
                "text-anchor:middle;font-size:13px;font-family:Arial",
            );
        }

        let cx = self.width as f32 / 2.0;
        let mut root_y = 50.0;
        let mut found = false;

        for stmt in &diagram.statements {
            if let Statement::MindmapNode {
                id: _,
                label,
                children,
            } = stmt
            {
                found = true;
                render_mm_node(&mut svg, label, cx, root_y);
                root_y += 60.0;
                let mut child_x = 100.0;
                let child_spacing = 80.0;
                for child in children {
                    if let Statement::MindmapNode { label: cl, .. } = child {
                        svg.add_line(
                            cx,
                            root_y - 30.0,
                            child_x + 60.0,
                            root_y - 15.0,
                            "stroke:#999;stroke-width:1",
                        );
                        render_mm_node(&mut svg, cl, child_x + 60.0, root_y - 15.0);
                        child_x += child_spacing;
                    }
                }
            }
        }

        if !found {
            svg.add_text(
                cx,
                self.height as f32 / 2.0,
                "mindmap",
                "text-anchor:middle;font-size:14px;font-family:Arial",
            );
        }

        svg.build()
    }

    fn render_gitgraph(&self, diagram: &Diagram) -> String {
        let mut svg = self.setup_svg_builder();

        let start_x = 100.0;
        let start_y = 50.0;
        let spacing_y = 40.0;
        let branch_colors = ["#45b7d1", "#ff6b6b", "#96ceb4", "#a29bfe", "#fdcb6e"];

        let mut branches: HashMap<String, usize> = HashMap::new();
        branches.insert("main".to_string(), 0);
        let mut current_branch = "main".to_string();
        let mut commit_count: usize = 0;

        for stmt in &diagram.statements {
            match stmt {
                Statement::GitCommit { id, tag, .. } => {
                    let col = *branches.get(&current_branch).unwrap_or(&0);
                    let x = start_x + col as f32 * 80.0;
                    let y = start_y + commit_count as f32 * spacing_y;
                    let color = branch_colors[col % branch_colors.len()];

                    svg.add_circle(
                        x,
                        y,
                        12.0,
                        &format!("fill:{};stroke:#333;stroke-width:1", color),
                    );

                    if commit_count > 0 {
                        let prev_y = start_y + (commit_count - 1) as f32 * spacing_y;
                        svg.add_line(x, prev_y, x, y, &format!("stroke:{};stroke-width:2", color));
                    }
                    if let Some(tag_str) = tag {
                        svg.add_text(
                            x + 20.0,
                            y + 4.0,
                            tag_str,
                            "font-size:9px;font-family:Arial;fill:#666",
                        );
                    }
                    if let Some(id_str) = id {
                        svg.add_text(
                            x,
                            y - 18.0,
                            id_str,
                            "text-anchor:middle;font-size:9px;font-family:monospace;fill:#999",
                        );
                    }
                    commit_count += 1;
                }
                Statement::GitBranch { name } => {
                    if !branches.contains_key(name) {
                        let col = branches.len();
                        branches.insert(name.clone(), col);
                    }
                    let col = *branches.get(name).unwrap_or(&0);
                    let x = start_x + col as f32 * 80.0;
                    let y = start_y + commit_count as f32 * spacing_y;
                    svg.add_text(
                        x,
                        y + 4.0,
                        name,
                        "text-anchor:middle;font-size:10px;font-family:monospace;fill:#666",
                    );
                }
                Statement::GitCheckout { name } => {
                    current_branch = name.clone();
                }
                Statement::GitMerge { branch, .. } => {
                    let col_from = *branches.get(branch).unwrap_or(&0);
                    let col_to = *branches.get(&current_branch).unwrap_or(&0);
                    let x_from = start_x + col_from as f32 * 80.0;
                    let y = start_y + commit_count as f32 * spacing_y;
                    let x_to = start_x + col_to as f32 * 80.0;
                    svg.add_line(
                        x_from,
                        y - spacing_y,
                        x_to,
                        y,
                        "stroke:#666;stroke-width:2;stroke-dasharray:4,2",
                    );
                    commit_count += 1;
                }
                _ => {}
            }
        }

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

        for i in 0..n {
            let angle =
                2.0 * std::f64::consts::PI * i as f64 / n as f64 - std::f64::consts::PI / 2.0;
            let ex = cx + max_r * angle.cos() as f32;
            let ey = cy + max_r * angle.sin() as f32;
            svg.add_line(cx, cy, ex, ey, "stroke:#ccc;stroke-width:1");

            let lr = max_r + 25.0;
            let lx = cx + lr * angle.cos() as f32;
            let ly = cy + lr * angle.sin() as f32;
            svg.add_text(lx, ly, &axes[i].0,
                         "text-anchor:middle;dominant-baseline:middle;font-size:12px;font-weight:bold;font-family:Arial");
        }

        let mut data_points = Vec::new();
        for i in 0..n {
            let angle =
                2.0 * std::f64::consts::PI * i as f64 / n as f64 - std::f64::consts::PI / 2.0;
            let value = axes[i].1.max(0.0).min(100.0);
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
        let cols = 3usize.max(1);

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
        let _range = (y_max - y_max).max(1.0);

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
        let opt = usvg::Options::default();
        let rtree =
            usvg::Tree::from_str(&svg_data, &opt).map_err(|e| format!("SVG parse error: {}", e))?;

        let w = self.width.max(1);
        let h = self.height.max(1);
        let mut pixmap = tiny_skia::Pixmap::new(w, h).ok_or("Failed to create Pixmap")?;

        resvg::render(
            &rtree,
            tiny_skia::Transform::from_scale(self.scale as f32, self.scale as f32),
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

/// Kahn's algorithm for topological sort layering.
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

                let style = match arrow_type {
                    ArrowType::Dashed | ArrowType::DashedCross | ArrowType::DashedOpen => {
                        "stroke:black;stroke-width:2;stroke-dasharray:8,4;fill:none"
                    }
                    _ => "stroke:black;stroke-width:2;fill:none",
                };

                svg.add_line(cx1, *y_pos, cx2, *y_pos, style);

                // Arrow head for solid/dashed/cross types (not open)
                if !matches!(arrow_type, ArrowType::SolidOpen | ArrowType::DashedOpen) {
                    let arrow_size = 8.0;
                    svg.add_polygon(
                        &[
                            (cx2, *y_pos),
                            (cx2 - arrow_size, *y_pos - 4.0),
                            (cx2 - arrow_size, *y_pos + 4.0),
                        ],
                        "fill:black",
                    );
                }

                // Message label
                let mid_x = (cx1 + cx2) / 2.0;
                let lw = 70.0;
                let lh = 18.0;
                svg.add_rect(
                    mid_x - lw / 2.0,
                    *y_pos - lh / 2.0,
                    lw,
                    lh,
                    "fill:white;stroke:none;",
                );
                svg.add_text(
                    mid_x,
                    *y_pos + 4.0,
                    label,
                    "text-anchor:middle;font-size:11px;font-family:Arial",
                );
                *y_pos += line_height;
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
                let blk_x = start_x - 20.0;
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
                svg.add_line(
                    blk_x,
                    block_start,
                    blk_x,
                    block_end,
                    "stroke:gray;stroke-width:1",
                );
                let block_label = if let Some(cond) = condition {
                    format!("{} [{}]", keyword, cond)
                } else {
                    keyword.clone()
                };
                svg.add_text(
                    start_x - 16.0,
                    block_start + 4.0,
                    &block_label,
                    "text-anchor:end;font-size:10px;font-family:Arial",
                );
                *y_pos += 10.0;
            }
            Statement::Activate { participant } => {
                activations.insert(participant.clone(), *y_pos);
                *y_pos += 5.0;
            }
            Statement::Deactivate { participant } => {
                if let Some(&start_y) = activations.get(participant) {
                    if let Some(&cx) = center_map.get(participant.as_str()) {
                        svg.add_rect(
                            cx - 5.0,
                            start_y,
                            10.0,
                            *y_pos - start_y,
                            "fill:#e8e8e8;stroke:#666;stroke-width:1",
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
}
