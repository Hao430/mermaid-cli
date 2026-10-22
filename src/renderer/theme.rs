//! CSS Custom Properties & Dynamic Theming Engine
//! Inspired by Ondine (ondine.sh)
//! Supports zero-recompile instant dark/light theming with full resvg fallback

pub const DEFAULT_CSS_VARIABLES: &str = r#"
    :root {
      --mermaid-bg: transparent;
      --mermaid-font-family: 'Inter', system-ui, -apple-system, sans-serif;
      --mermaid-node-bg: #ffffff;
      --mermaid-node-border: #334155;
      --mermaid-node-border-width: 1.5px;
      --mermaid-text-primary: #0f172a;
      --mermaid-text-muted: #64748b;
      --mermaid-edge-stroke: #475569;
      --mermaid-edge-width: 1.5px;
      --mermaid-arrowhead-fill: #475569;
      --mermaid-pill-bg: #ffffff;
      --mermaid-pill-border: #cbd5e1;
      --mermaid-subgraph-bg: #f8fafc;
      --mermaid-subgraph-border: #cbd5e1;
      --mermaid-subgraph-title: #475569;
    }
    @media (prefers-color-scheme: dark) {
      :root {
        --mermaid-node-bg: #1e293b;
        --mermaid-node-border: #475569;
        --mermaid-text-primary: #f8fafc;
        --mermaid-text-muted: #94a3b8;
        --mermaid-edge-stroke: #94a3b8;
        --mermaid-arrowhead-fill: #94a3b8;
        --mermaid-pill-bg: #0f172a;
        --mermaid-pill-border: #334155;
        --mermaid-subgraph-bg: rgba(15, 23, 42, 0.6);
        --mermaid-subgraph-border: #334155;
        --mermaid-subgraph-title: #94a3b8;
      }
    }
"#;

/// Return default node style with CSS variable and fallback
pub fn default_node_style() -> String {
    "fill:var(--mermaid-node-bg, #ffffff);stroke:var(--mermaid-node-border, #334155);stroke-width:var(--mermaid-node-border-width, 1.5)".to_string()
}

/// Return default node text fill color with fallback
pub fn default_text_fill() -> String {
    "var(--mermaid-text-primary, #0f172a)".to_string()
}

/// Return default edge path style with fallback
pub fn default_edge_style() -> String {
    "fill:none;stroke:var(--mermaid-edge-stroke, #475569);stroke-width:var(--mermaid-edge-width, 1.5)".to_string()
}

/// Return default arrowhead fill with fallback
pub fn default_arrowhead_fill() -> String {
    "var(--mermaid-arrowhead-fill, #475569)".to_string()
}

/// Return default edge label pill shield style with fallback
pub fn default_pill_style() -> String {
    "fill:var(--mermaid-pill-bg, #ffffff);stroke:var(--mermaid-pill-border, #cbd5e1);stroke-width:1;rx:4;ry:4".to_string()
}

/// Return default edge label text fill
pub fn default_pill_text_fill() -> String {
    "var(--mermaid-text-primary, #475569)".to_string()
}

/// Return default subgraph container style
pub fn default_subgraph_style() -> String {
    "fill:var(--mermaid-subgraph-bg, #f8fafc);stroke:var(--mermaid-subgraph-border, #cbd5e1);stroke-width:1.5;stroke-dasharray:4;rx:8;ry:8".to_string()
}

/// Return default subgraph title text style
pub fn default_subgraph_title_style() -> String {
    "text-anchor:start;dominant-baseline:middle;font-size:13px;font-weight:bold;fill:var(--mermaid-subgraph-title, #475569);font-family:Arial, sans-serif".to_string()
}
