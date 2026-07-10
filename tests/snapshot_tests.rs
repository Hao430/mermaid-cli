/// Snapshot tests for all diagram types
/// Verifies that all diagram types render successfully and produce valid SVG

use std::fs;

// Load test fixtures
fn load_fixtures() -> Vec<(String, serde_json::Value)> {
    let fixtures_path = "tests/fixtures/diagrams.json";
    let content = fs::read_to_string(fixtures_path)
        .expect("Failed to read fixtures file");

    let json: serde_json::Value = serde_json::from_str(&content)
        .expect("Failed to parse fixtures JSON");

    json.as_object()
        .expect("Fixtures must be an object")
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

#[test]
fn test_all_diagram_types_render() {
    let fixtures = load_fixtures();
    assert!(!fixtures.is_empty(), "No fixtures found");

    for (name, fixture) in fixtures {
        let input = fixture["input"]
            .as_str()
            .expect(&format!("No input for {}", name));

        let result = mermaid_cli::render(input);
        assert!(result.is_ok(), "Render failed for {}: {:?}", name, result.err());

        let svg = result.unwrap();
        assert!(!svg.is_empty(), "Empty SVG for {}", name);
    }
}

#[test]
fn test_all_diagrams_produce_valid_svg() {
    let fixtures = load_fixtures();

    for (name, fixture) in fixtures {
        let input = fixture["input"].as_str().unwrap();
        let svg = mermaid_cli::render(input).unwrap();

        // Basic SVG validation
        assert!(svg.contains("<svg"), "{}: Missing <svg> tag", name);
        assert!(svg.contains("</svg>"), "{}: Missing </svg> tag", name);
        assert!(svg.contains("xmlns="), "{}: Missing xmlns attribute", name);
    }
}

#[test]
fn test_diagram_count() {
    let fixtures = load_fixtures();
    assert_eq!(fixtures.len(), 30, "Expected 30 test fixtures (15 supported types × 2)");
}

#[test]
fn test_all_diagram_types_covered() {
    let fixtures = load_fixtures();
    let mut types_found = std::collections::HashSet::new();

    for (_, fixture) in fixtures {
        if let Some(diagram_type) = fixture["type"].as_str() {
            types_found.insert(diagram_type.to_string());
        }
    }

    // Verify we have tests for key diagram types
    let required_types = vec![
        "flowchart", "sequence", "pie", "class", "state", "er", "gantt",
        "mindmap", "gitgraph", "timeline", "journey", "kanban", "venn"
    ];

    for required_type in required_types {
        assert!(
            types_found.iter().any(|t| t.contains(required_type)),
            "Missing test for diagram type: {}",
            required_type
        );
    }
}

#[test]
fn test_svg_no_errors() {
    let fixtures = load_fixtures();

    for (name, fixture) in fixtures {
        let input = fixture["input"].as_str().unwrap();
        let svg = mermaid_cli::render(input).unwrap();

        // Check for common error indicators in SVG
        assert!(!svg.to_lowercase().contains("error"),
            "{}: SVG contains 'error' text", name);
        assert!(!svg.contains("undefined"),
            "{}: SVG contains 'undefined'", name);
        assert!(!svg.contains("NaN"),
            "{}: SVG contains 'NaN'", name);
    }
}

#[test]
fn test_svg_has_content() {
    let fixtures = load_fixtures();

    for (name, fixture) in fixtures {
        let input = fixture["input"].as_str().unwrap();
        let svg = mermaid_cli::render(input).unwrap();

        // Verify SVG has actual content (paths, text, etc.)
        let has_content = svg.contains("<path")
            || svg.contains("<circle")
            || svg.contains("<rect")
            || svg.contains("<text")
            || svg.contains("<line");

        assert!(has_content, "{}: SVG has no visible content", name);
    }
}

#[test]
fn test_svg_dimensions() {
    let fixtures = load_fixtures();

    for (name, fixture) in fixtures {
        let input = fixture["input"].as_str().unwrap();
        let svg = mermaid_cli::render(input).unwrap();

        // Verify SVG has width and height
        assert!(svg.contains("width="), "{}: SVG missing width", name);
        assert!(svg.contains("height="), "{}: SVG missing height", name);
    }
}

#[cfg(test)]
mod performance_checks {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_render_completes_quickly() {
        let fixtures = load_fixtures();
        let start = Instant::now();

        for (_, fixture) in &fixtures {
            let input = fixture["input"].as_str().unwrap();
            let _ = mermaid_cli::render(input);
        }

        let elapsed = start.elapsed();
        let avg_time = elapsed.as_millis() as f64 / fixtures.len() as f64;

        println!("Average render time: {:.2}ms per diagram", avg_time);
        assert!(avg_time < 100.0, "Average render time too slow: {:.2}ms", avg_time);
    }
}
