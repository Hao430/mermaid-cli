use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum DiagramType {
    Flowchart,
    Sequence,
}

#[derive(Debug, Clone)]
pub struct Diagram {
    pub diagram_type: DiagramType,
    pub statements: Vec<Statement>,
    pub direction: Option<String>,
    pub subgraphs: Vec<Subgraph>,
}

#[derive(Debug, Clone)]
pub struct Subgraph {
    pub id: String,
    pub title: Option<String>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
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
}

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
