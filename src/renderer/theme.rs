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

/// Resolve CSS custom properties in inline styles to concrete values.
///
/// resvg/usvg (used for PNG output) does not support CSS custom properties,
/// so `fill:var(--mermaid-node-bg, #ffffff)` is dropped and shapes fall back
/// to black. This rewrites every `var(--name[, fallback])` expression using
/// the values declared in the document's first `:root` block.
///
/// SVG output keeps the variables (browsers support them and can switch
/// themes via `prefers-color-scheme`); only the raster pipeline flattens.
pub fn inline_css_variables(svg: &str) -> String {
    let vars = root_variables(svg);
    if vars.is_empty() {
        return svg.to_string();
    }

    let mut out = String::with_capacity(svg.len());
    let mut rest = svg;

    while let Some(pos) = rest.find("var(") {
        out.push_str(&rest[..pos]);
        let after = &rest[pos + 4..];

        // Find the matching ')' so nested calls (e.g. rgb(...)) survive.
        let mut depth = 1usize;
        let mut end = None;
        for (i, ch) in after.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(end) = end else {
            // Unterminated expression: leave the remainder untouched.
            out.push_str(&rest[pos..]);
            return out;
        };

        let inner = &after[..end];
        let (name, fallback) = match inner.split_once(',') {
            Some((n, f)) => (n.trim(), Some(f.trim())),
            None => (inner.trim(), None),
        };
        let value = vars
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
            .or(fallback);

        match value {
            Some(v) => out.push_str(v),
            None => out.push_str(&rest[pos..pos + 4 + end + 1]),
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

/// Collect `--name: value` declarations from the document's first `:root` block.
fn root_variables(svg: &str) -> Vec<(String, String)> {
    let Some(start) = svg.find(":root") else {
        return Vec::new();
    };
    let Some(open_rel) = svg[start..].find('{') else {
        return Vec::new();
    };
    let open = start + open_rel;
    let Some(close_rel) = svg[open..].find('}') else {
        return Vec::new();
    };
    svg[open + 1..open + close_rel]
        .split(';')
        .filter_map(|decl| {
            let (name, value) = decl.split_once(':')?;
            let name = name.trim();
            let value = value.trim();
            if name.starts_with("--") && !value.is_empty() {
                Some((name.to_string(), value.to_string()))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_resolves_variable_from_root() {
        let svg = r#"<svg><style>:root { --mermaid-node-bg: #ffffff; }</style>
            <rect style="fill:var(--mermaid-node-bg, #000000)"/></svg>"#;
        let out = inline_css_variables(svg);
        assert!(out.contains("fill:#ffffff"));
        assert!(!out.contains("var("));
    }

    #[test]
    fn inline_uses_fallback_for_unknown_variable() {
        let svg = r#"<svg><style>:root { --known: #111111; }</style>
            <rect style="stroke:var(--missing, #abcdef)"/></svg>"#;
        let out = inline_css_variables(svg);
        assert!(out.contains("stroke:#abcdef"));
    }

    #[test]
    fn inline_leaves_unknown_variable_without_fallback() {
        let svg = r#"<svg><style>:root { --known: #111111; }</style>
            <rect style="stroke:var(--missing)"/></svg>"#;
        let out = inline_css_variables(svg);
        assert!(out.contains("var(--missing)"));
    }

    #[test]
    fn inline_ignores_dark_media_query_overrides() {
        let svg = r#"<svg><style>:root { --mermaid-node-bg: #ffffff; }
            @media (prefers-color-scheme: dark) { :root { --mermaid-node-bg: #1e293b; } }</style>
            <rect style="fill:var(--mermaid-node-bg, #000000)"/></svg>"#;
        let out = inline_css_variables(svg);
        assert!(out.contains("fill:#ffffff"));
    }
}
