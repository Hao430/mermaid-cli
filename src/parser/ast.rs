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
    Rect,        // []
    Circle,      // ()
    Diamond,     // {}
    Rounded,     // [()]
    RoundedRect, // [[]]
    Named(String),
}

impl fmt::Display for NodeShape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeShape::Rect => write!(f, "rect"),
            NodeShape::Circle => write!(f, "circle"),
            NodeShape::Diamond => write!(f, "diamond"),
            NodeShape::Rounded => write!(f, "rounded"),
            NodeShape::RoundedRect => write!(f, "roundedRect"),
            NodeShape::Named(s) => write!(f, "{}", s),
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

    pub fn get_edges(&self) -> Vec<(String, String)> {
        let mut edges = Vec::new();
        for stmt in &self.statements {
            if let Statement::EdgeDef { from, to, .. } = stmt {
                edges.push((from.clone(), to.clone()));
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
        };

        let mut nodes = diagram.get_nodes();
        nodes.sort();
        assert_eq!(nodes, vec!["A", "B"]);
    }
}
