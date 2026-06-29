use std::fmt;

#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum DiagramType {
    Flowchart,
    Sequence,
    Pie,
    Class,
    State,
    Er,
    Gantt,
    Mindmap,
    GitGraph,
    Timeline,
    Journey,
    Kanban,
    Venn,
    Packet,
    Radar,
    Ishikawa,
    Quadrant,
    ZenUml,
    Requirement,
    Block,
    C4,
    Architecture,
    XyChart,
    Sankey,
    Treemap,
}

#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone)]
pub struct Diagram {
    pub diagram_type: DiagramType,
    pub statements: Vec<Statement>,
    pub direction: Option<String>,
    pub subgraphs: Vec<Subgraph>,
    /// Optional title for pie charts and other diagram types.
    pub title: Option<String>,
}

#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone)]
pub struct Subgraph {
    pub id: String,
    pub title: Option<String>,
    pub statements: Vec<Statement>,
}

#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone)]
pub enum Statement {
    // Flowchart
    NodeDef {
        id: String,
        label: Option<String>,
        shape: NodeShape,
    },
    EdgeDef {
        from: String,
        to: String,
        label: Option<String>,
    },
    // Sequence diagram
    Participant {
        id: String,
        label: Option<String>,
    },
    Message {
        from: String,
        to: String,
        label: String,
        arrow_type: ArrowType,
    },
    Note {
        target: String,
        text: String,
        position: NotePosition,
    },
    Block {
        keyword: String,
        condition: Option<String>,
        statements: Vec<Statement>,
    },
    Activate {
        participant: String,
    },
    Deactivate {
        participant: String,
    },
    // Pie chart
    PieSlice {
        label: String,
        value: f64,
    },
    // Class diagram
    ClassDef {
        name: String,
        stereotype: Option<String>,
    },
    ClassMember {
        class_name: String,
        member_type: ClassMemberType,
        visibility: ClassVisibility,
        name: String,
        type_annotation: Option<String>,
    },
    ClassRelation {
        from: String,
        to: String,
        relation_type: ClassRelationType,
        from_cardinality: Option<String>,
        to_cardinality: Option<String>,
        label: Option<String>,
    },
    // State diagram
    StateDef {
        id: String,
        label: Option<String>,
    },
    StateTransition {
        from: String,
        to: String,
        label: Option<String>,
    },
    // ER diagram
    ErEntity {
        name: String,
    },
    ErAttribute {
        entity: String,
        name: String,
        type_annotation: Option<String>,
        is_pk: bool,
        is_null: bool,
    },
    ErRelation {
        from: String,
        to: String,
        from_cardinality: ErCardinality,
        to_cardinality: ErCardinality,
        label: Option<String>,
    },
    // Gantt chart
    GanttSection {
        name: String,
    },
    // Mindmap
    MindmapNode {
        id: String,
        label: String,
        children: Vec<Statement>,
    },
    // GitGraph
    GitCommit {
        id: Option<String>,
        tag: Option<String>,
        branch: Option<String>,
    },
    GitBranch {
        name: String,
    },
    GitCheckout {
        name: String,
    },
    GitMerge {
        branch: String,
        tag: Option<String>,
    },
    // Timeline
    TimelineSection {
        name: String,
    },
    TimelineEvent {
        time: String,
        description: String,
    },
    // Journey
    JourneySection {
        name: String,
    },
    JourneyTask {
        name: String,
        score: f64,
        actors: Vec<String>,
    },
    // Kanban
    KanbanColumn {
        name: String,
    },
    KanbanTask {
        name: String,
        description: Option<String>,
    },
    // Venn
    VennSet {
        id: String,
        label: String,
    },
    // Packet
    PacketField {
        start_bit: u32,
        end_bit: u32,
        label: String,
    },
    // Radar
    RadarAxis {
        label: String,
        value: f64,
    },
    // Ishikawa
    IshikawaRoot {
        label: String,
    },
    IshikawaCategory {
        label: String,
    },
    IshikawaCause {
        label: String,
    },
    // Quadrant Chart
    QuadrantTitle(String),
    QuadrantXAxis(String),
    QuadrantYAxis(String),
    QuadrantLabel {
        quadrant: u32,
        label: String,
    },
    QuadrantPoint {
        label: String,
        x: f64,
        y: f64,
    },
    // Block
    BlockNode {
        id: String,
        label: String,
        children: Vec<Statement>,
    },
    // Treemap
    TreemapItem { label: String, value: f64 },
    // Sankey
    SankeyLink { source: String, target: String, value: f64 },
    // XY Chart
    XyTitle(String),
    XyXAxis { label: String, categories: Vec<String> },
    XyYAxis { label: String, min: f64, max: f64 },
    XyBar { data: Vec<f64> },
    XyLine { data: Vec<f64> },
    // Architecture
    ArchService { id: String, label: String },
    ArchDatabase { id: String, label: String },
    ArchQueue { id: String, label: String },
    ArchRelation { from: String, to: String },
    // C4
    C4Person { alias: String, label: String, description: String },
    C4System { alias: String, label: String, description: String },
    C4Container { alias: String, label: String, description: String },
    C4Component { alias: String, label: String, description: String },
    C4Rel { from: String, to: String, label: String },
    // Requirement Diagram
    RequirementDef {
        name: String,
        req_id: String,
        text: String,
        risk: String,
        verify_method: String,
    },
    RequirementElement {
        name: String,
        element_type: String,
    },
    RequirementRelation {
        from: String,
        to: String,
        relation_type: String,
    },
    GanttTask {
        name: String,
        id: Option<String>,
        status: Option<GanttStatus>,
        start: Option<String>,
        duration: Option<String>,
        after: Option<String>,
    },
}

#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ArrowType {
    Solid,       // ->
    Dashed,      // -->
    SolidCross,  // -x
    DashedCross, // --x
    SolidOpen,   // -)
    DashedOpen,  // --)
}

#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum NotePosition {
    Left,
    Right,
    Over,
}

#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ClassMemberType {
    Field,
    Method,
}

#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ClassVisibility {
    Public,    // +
    Private,   // -
    Protected, // #
    Package,   // ~
}

#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ClassRelationType {
    Inheritance,   // <|--
    Composition,   // *--
    Aggregation,   // o--
    Association,   // -->
    Dependency,    // ..>
    Realization,   // ..|>
    Link,          // --
}

#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ErCardinality {
    ZeroOrOne,     // |o or o|
    OneOrMore,     // }| or |{
    ZeroOrMore,    // }o or o{
    ExactlyOne,    // || or ||
}

#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum GanttStatus {
    Done,       // done
    Active,     // active
    Critical,   // crit
    Default,
}

#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum NodeShape {
    Rect,         // [text]
    Circle,       // (text) → 圆角矩形
    Diamond,      // {text} → 菱形
    Rounded,      // ([text]) → 大圆角矩形
    Subroutine,   // [[text]] → 双线矩形
    Cylinder,     // [(text)] → 圆柱
    DoubleCircle, // ((text)) → 双圆
    Flag,         // >text] → 旗帜
}

impl fmt::Display for NodeShape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeShape::Rect => write!(f, "rect"),
            NodeShape::Circle => write!(f, "circle"),
            NodeShape::Diamond => write!(f, "diamond"),
            NodeShape::Rounded => write!(f, "rounded"),
            NodeShape::Subroutine => write!(f, "subroutine"),
            NodeShape::Cylinder => write!(f, "cylinder"),
            NodeShape::DoubleCircle => write!(f, "doubleCircle"),
            NodeShape::Flag => write!(f, "flag"),
        }
    }
}

impl Diagram {
    pub fn get_nodes(&self) -> Vec<String> {
        let mut nodes = std::collections::HashSet::new();
        for stmt in &self.statements {
            match stmt {
                Statement::NodeDef { id, .. } => {
                    nodes.insert(id.clone());
                }
                Statement::EdgeDef { from, to, .. } => {
                    nodes.insert(from.clone());
                    nodes.insert(to.clone());
                }
                Statement::Participant { id, .. } => {
                    nodes.insert(id.clone());
                }
                Statement::Message { from, to, .. } => {
                    nodes.insert(from.clone());
                    nodes.insert(to.clone());
                }
                Statement::Note { target, .. } => {
                    nodes.insert(target.clone());
                }
                Statement::Block { statements, .. } => {
                    // Recursively collect nodes from nested statements
                    let nested = Diagram {
                        diagram_type: DiagramType::Flowchart,
                        statements: statements.clone(),
                        direction: None,
                        subgraphs: vec![],
                        title: None,
                    };
                    nodes.extend(nested.get_nodes());
                }
                Statement::Activate { participant } | Statement::Deactivate { participant } => {
                    nodes.insert(participant.clone());
                }
                Statement::PieSlice { .. } => {} // Pie slices don't have graph nodes
                Statement::ClassDef { name, .. } => {
                    nodes.insert(name.clone());
                }
                Statement::ClassMember { class_name, .. } => {
                    nodes.insert(class_name.clone());
                }
                Statement::ClassRelation { from, to, .. } => {
                    nodes.insert(from.clone());
                    nodes.insert(to.clone());
                }
                Statement::StateDef { id, .. } => {
                    nodes.insert(id.clone());
                }
                Statement::StateTransition { from, to, .. } => {
                    nodes.insert(from.clone());
                    nodes.insert(to.clone());
                }
                Statement::ErEntity { name } => {
                    nodes.insert(name.clone());
                }
                Statement::ErAttribute { entity, .. } => {
                    nodes.insert(entity.clone());
                }
                Statement::ErRelation { from, to, .. } => {
                    nodes.insert(from.clone());
                    nodes.insert(to.clone());
                }
                Statement::GanttSection { name } => {
                    nodes.insert(name.clone());
                }
                Statement::GanttTask { name, .. } => {
                    nodes.insert(name.clone());
                }
                Statement::MindmapNode { id, children, .. } => {
                    nodes.insert(id.clone());
                    let nested = Diagram {
                        diagram_type: DiagramType::Flowchart,
                        statements: children.clone(),
                        direction: None,
                        subgraphs: vec![],
                        title: None,
                    };
                    nodes.extend(nested.get_nodes());
                }
                Statement::GitCommit { id, .. } => {
                    if let Some(id_str) = id {
                        nodes.insert(id_str.clone());
                    }
                }
                Statement::GitBranch { name } => {
                    nodes.insert(name.clone());
                }
                Statement::GitCheckout { name } => {
                    nodes.insert(name.clone());
                }
                Statement::GitMerge { branch, .. } => {
                    nodes.insert(branch.clone());
                }
                Statement::TimelineSection { name } => {
                    nodes.insert(name.clone());
                }
                Statement::TimelineEvent { time, .. } => {
                    nodes.insert(time.clone());
                }
                Statement::JourneySection { name } => {
                    nodes.insert(name.clone());
                }
                Statement::JourneyTask { name, .. } => {
                    nodes.insert(name.clone());
                }
                Statement::KanbanColumn { name } => {
                    nodes.insert(name.clone());
                }
                Statement::KanbanTask { name, .. } => {
                    nodes.insert(name.clone());
                }
                Statement::VennSet { id, .. } => {
                    nodes.insert(id.clone());
                }
                Statement::PacketField { label, .. } => {
                    nodes.insert(label.clone());
                }
                Statement::RadarAxis { label, .. } => {
                    nodes.insert(label.clone());
                }
                Statement::IshikawaRoot { label } | Statement::IshikawaCategory { label } | Statement::IshikawaCause { label } => {
                    nodes.insert(label.clone());
                }
                Statement::QuadrantTitle(t) | Statement::QuadrantXAxis(t) | Statement::QuadrantYAxis(t) => {
                    nodes.insert(t.clone());
                }
                Statement::QuadrantLabel { label, .. } | Statement::QuadrantPoint { label, .. } => {
                    nodes.insert(label.clone());
                }
                Statement::BlockNode { id, children, .. } => {
                    nodes.insert(id.clone());
                    let nested = Diagram { diagram_type: DiagramType::Flowchart, statements: children.clone(), direction: None, subgraphs: vec![], title: None };
                    nodes.extend(nested.get_nodes());
                }
                Statement::TreemapItem { label, .. } => { nodes.insert(label.clone()); }
                Statement::SankeyLink { source, target, .. } => { nodes.insert(source.clone()); nodes.insert(target.clone()); }
                Statement::XyTitle(t) | Statement::XyXAxis { label: t, .. } | Statement::XyYAxis { label: t, .. } => { nodes.insert(t.clone()); }
                Statement::XyBar { .. } | Statement::XyLine { .. } => {}
                Statement::ArchService { id, .. } | Statement::ArchDatabase { id, .. } | Statement::ArchQueue { id, .. } => { nodes.insert(id.clone()); }
                Statement::ArchRelation { from, to } => { nodes.insert(from.clone()); nodes.insert(to.clone()); }
                Statement::C4Person { alias, .. } | Statement::C4System { alias, .. } | Statement::C4Container { alias, .. } | Statement::C4Component { alias, .. } => { nodes.insert(alias.clone()); }
                Statement::C4Rel { from, to, .. } => { nodes.insert(from.clone()); nodes.insert(to.clone()); }
                Statement::RequirementDef { name, .. } => { nodes.insert(name.clone()); }
                Statement::RequirementElement { name, .. } => { nodes.insert(name.clone()); }
                Statement::RequirementRelation { from, to, .. } => { nodes.insert(from.clone()); nodes.insert(to.clone()); }
            }
        }
        nodes.into_iter().collect()
    }

    /// Returns nodes with their display labels.
    /// Uses label if available, falls back to node ID.
    pub fn get_node_labels(&self) -> Vec<(String, String)> {
        let mut node_map = std::collections::HashMap::new();
        for stmt in &self.statements {
            match stmt {
                Statement::NodeDef { id, label, .. } => {
                    let display = label.clone().unwrap_or_else(|| id.clone());
                    node_map.insert(id.clone(), display);
                }
                Statement::EdgeDef { from, to, .. } => {
                    node_map.entry(from.clone()).or_insert_with(|| from.clone());
                    node_map.entry(to.clone()).or_insert_with(|| to.clone());
                }
                Statement::Participant { id, label } => {
                    let display = label.clone().unwrap_or_else(|| id.clone());
                    node_map.insert(id.clone(), display);
                }
                Statement::Message { from, to, .. } => {
                    node_map.entry(from.clone()).or_insert_with(|| from.clone());
                    node_map.entry(to.clone()).or_insert_with(|| to.clone());
                }
                Statement::Note { target, .. } => {
                    node_map.entry(target.clone()).or_insert_with(|| target.clone());
                }
                Statement::Block { .. } => {}
                Statement::Activate { participant } | Statement::Deactivate { participant } => {
                    node_map
                        .entry(participant.clone())
                        .or_insert_with(|| participant.clone());
                }
                Statement::PieSlice { .. } => {} // Pie slices don't have graph nodes
                Statement::ClassDef { name, .. } => {
                    node_map.entry(name.clone()).or_insert_with(|| name.clone());
                }
                Statement::ClassMember { class_name, .. } => {
                    node_map.entry(class_name.clone()).or_insert_with(|| class_name.clone());
                }
                Statement::ClassRelation { from, to, .. } => {
                    node_map.entry(from.clone()).or_insert_with(|| from.clone());
                    node_map.entry(to.clone()).or_insert_with(|| to.clone());
                }
                Statement::StateDef { id, label } => {
                    let display = label.clone().unwrap_or_else(|| id.clone());
                    node_map.insert(id.clone(), display);
                }
                Statement::StateTransition { from, to, .. } => {
                    node_map.entry(from.clone()).or_insert_with(|| from.clone());
                    node_map.entry(to.clone()).or_insert_with(|| to.clone());
                }
                Statement::ErEntity { name } => {
                    node_map.entry(name.clone()).or_insert_with(|| name.clone());
                }
                Statement::ErAttribute { entity, .. } => {
                    node_map.entry(entity.clone()).or_insert_with(|| entity.clone());
                }
                Statement::ErRelation { from, to, .. } => {
                    node_map.entry(from.clone()).or_insert_with(|| from.clone());
                    node_map.entry(to.clone()).or_insert_with(|| to.clone());
                }
                Statement::GanttSection { name } => {
                    node_map.entry(name.clone()).or_insert_with(|| name.clone());
                }
                Statement::GanttTask { name, .. } => {
                    node_map.entry(name.clone()).or_insert_with(|| name.clone());
                }
                Statement::MindmapNode { id, label, children } => {
                    node_map.insert(id.clone(), label.clone());
                    for child in children {
                        if let Statement::MindmapNode { id, .. } = child {
                            node_map.entry(id.clone()).or_insert_with(|| id.clone());
                        }
                    }
                }
                Statement::GitCommit { id, .. } => {
                    if let Some(id_str) = id {
                        node_map.entry(id_str.clone()).or_insert_with(|| id_str.clone());
                    }
                }
                Statement::GitBranch { name } => {
                    node_map.entry(name.clone()).or_insert_with(|| name.clone());
                }
                Statement::GitCheckout { name } => {
                    node_map.entry(name.clone()).or_insert_with(|| name.clone());
                }
                Statement::GitMerge { branch, .. } => {
                    node_map.entry(branch.clone()).or_insert_with(|| branch.clone());
                }
                Statement::TimelineSection { name } => {
                    node_map.entry(name.clone()).or_insert_with(|| name.clone());
                }
                Statement::TimelineEvent { time, description } => {
                    node_map.insert(time.clone(), description.clone());
                }
                Statement::JourneySection { name } => {
                    node_map.entry(name.clone()).or_insert_with(|| name.clone());
                }
                Statement::JourneyTask { name, .. } => {
                    node_map.entry(name.clone()).or_insert_with(|| name.clone());
                }
                Statement::KanbanColumn { name } => {
                    node_map.entry(name.clone()).or_insert_with(|| name.clone());
                }
                Statement::KanbanTask { name, .. } => {
                    node_map.entry(name.clone()).or_insert_with(|| name.clone());
                }
                Statement::VennSet { id, label } => {
                    node_map.insert(id.clone(), label.clone());
                }
                Statement::PacketField { label, .. } => {
                    node_map.entry(label.clone()).or_insert_with(|| label.clone());
                }
                Statement::RadarAxis { label, value } => {
                    node_map.entry(label.clone()).or_insert_with(|| format!("{} ({})", label, value));
                }
                Statement::IshikawaRoot { label } | Statement::IshikawaCategory { label } | Statement::IshikawaCause { label } => {
                    node_map.entry(label.clone()).or_insert_with(|| label.clone());
                }
                Statement::QuadrantTitle(t) | Statement::QuadrantXAxis(t) | Statement::QuadrantYAxis(t) => {
                    node_map.entry(t.clone()).or_insert_with(|| t.clone());
                }
                Statement::QuadrantLabel { label, .. } | Statement::QuadrantPoint { label, .. } => {
                    node_map.entry(label.clone()).or_insert_with(|| label.clone());
                }
                Statement::BlockNode { id, label, children } => {
                    node_map.insert(id.clone(), label.clone());
                    for child in children {
                        if let Statement::BlockNode { id, .. } = child {
                            node_map.entry(id.clone()).or_insert_with(|| id.clone());
                        }
                    }
                }
                Statement::TreemapItem { label, .. } => { node_map.entry(label.clone()).or_insert_with(|| label.clone()); }
                Statement::SankeyLink { source, target, .. } => { node_map.entry(source.clone()).or_insert_with(|| source.clone()); node_map.entry(target.clone()).or_insert_with(|| target.clone()); }
                Statement::XyTitle(t) => { node_map.entry(t.clone()).or_insert_with(|| t.clone()); }
                Statement::XyXAxis { label, categories } => { for c in categories { node_map.entry(c.clone()).or_insert_with(|| c.clone()); } node_map.entry(label.clone()).or_insert_with(|| label.clone()); }
                Statement::XyYAxis { label, .. } => { node_map.entry(label.clone()).or_insert_with(|| label.clone()); }
                Statement::XyBar { .. } | Statement::XyLine { .. } => {}
                Statement::ArchService { id, label } | Statement::ArchDatabase { id, label } | Statement::ArchQueue { id, label } => { node_map.insert(id.clone(), label.clone()); }
                Statement::ArchRelation { from, to } => { node_map.entry(from.clone()).or_insert_with(|| from.clone()); node_map.entry(to.clone()).or_insert_with(|| to.clone()); }
                Statement::C4Person { alias, label, .. } | Statement::C4System { alias, label, .. } | Statement::C4Container { alias, label, .. } | Statement::C4Component { alias, label, .. } => { node_map.insert(alias.clone(), label.clone()); }
                Statement::C4Rel { from, to, .. } => { node_map.entry(from.clone()).or_insert_with(|| from.clone()); node_map.entry(to.clone()).or_insert_with(|| to.clone()); }
                Statement::RequirementDef { name, .. } => { node_map.entry(name.clone()).or_insert_with(|| name.clone()); }
                Statement::RequirementElement { name, .. } => { node_map.entry(name.clone()).or_insert_with(|| name.clone()); }
                Statement::RequirementRelation { from, to, .. } => { node_map.entry(from.clone()).or_insert_with(|| from.clone()); node_map.entry(to.clone()).or_insert_with(|| to.clone()); }
            }
        }
        node_map.into_iter().collect()
    }

    pub fn get_edges(&self) -> Vec<(String, String)> {
        let mut edges = Vec::new();
        for stmt in &self.statements {
            if let Statement::EdgeDef { from, to, .. } = stmt {
                edges.push((from.clone(), to.clone()));
            }
        }
        edges
    }

    /// Returns all edges with their labels (if any).
    pub fn get_edges_with_labels(&self) -> Vec<(String, String, Option<String>)> {
        let mut edges = Vec::new();
        for stmt in &self.statements {
            if let Statement::EdgeDef { from, to, label } = stmt {
                edges.push((from.clone(), to.clone(), label.clone()));
            }
        }
        edges
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagram_creation() {
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
                Statement::EdgeDef {
                    from: "A".to_string(),
                    to: "B".to_string(),
                    label: None,
                },
            ],
            direction: None,
            subgraphs: vec![],
            title: None,
        };

        assert_eq!(diagram.diagram_type, DiagramType::Flowchart);
        assert_eq!(diagram.statements.len(), 3);
    }

    #[test]
    fn test_get_nodes() {
        let diagram = Diagram {
            diagram_type: DiagramType::Flowchart,
            statements: vec![
                Statement::NodeDef {
                    id: "A".to_string(),
                    label: None,
                    shape: NodeShape::Rect,
                },
                Statement::NodeDef {
                    id: "B".to_string(),
                    label: None,
                    shape: NodeShape::Rect,
                },
            ],
            direction: None,
            subgraphs: vec![],
            title: None,
        };

        let mut nodes = diagram.get_nodes();
        nodes.sort();
        assert_eq!(nodes, vec!["A", "B"]);
    }

    #[test]
    fn test_node_shape_display() {
        assert_eq!(NodeShape::Rect.to_string(), "rect");
        assert_eq!(NodeShape::Circle.to_string(), "circle");
        assert_eq!(NodeShape::Diamond.to_string(), "diamond");
        assert_eq!(NodeShape::Rounded.to_string(), "rounded");
        assert_eq!(NodeShape::Subroutine.to_string(), "subroutine");
        assert_eq!(NodeShape::Cylinder.to_string(), "cylinder");
        assert_eq!(NodeShape::DoubleCircle.to_string(), "doubleCircle");
        assert_eq!(NodeShape::Flag.to_string(), "flag");
    }

    #[test]
    fn test_node_shape_equality() {
        assert_eq!(NodeShape::Rect, NodeShape::Rect);
        assert_ne!(NodeShape::Rect, NodeShape::Diamond);
        assert_eq!(NodeShape::Cylinder, NodeShape::Cylinder);
    }
}
