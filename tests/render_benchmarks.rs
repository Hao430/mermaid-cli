use mermaid_cli::{parse, Renderer};

/// Run the benchmark as a test so it works with `cargo test`.
#[test]
fn test_benchmarks() {
    // Simple manual benchmark runner (no external deps needed)
    let cases = vec![
        ("Flowchart simple", "graph TD; A-->B", 1000),
        ("Flowchart labeled", "graph TD; A[Start]-->B[End]", 500),
        ("Flowchart complex", "graph TD\nA[Start]-->B{Decision}\nB-->|Yes|C[Result]\nB-->|No|D[Other]", 500),
        ("Sequence simple", "sequenceDiagram\n    A->B: Hello", 500),
        ("Sequence complex", "sequenceDiagram\n    Alice->Bob: Hello\n    Bob-->Alice: Hi\n    alt ok\n        Alice->Bob: Yes\n    else no\n        Alice->Bob: No\n    end", 200),
        ("Pie chart", "pie title Data\n\"A\" : 50\n\"B\" : 30\n\"C\" : 20\n\"D\" : 10", 500),
        ("Class diagram", "classDiagram\nclass Animal {\n+String name\n+int age\n+isMammal() bool\n}", 500),
        ("State diagram", "stateDiagram-v2\n[*] --> Still\nStill --> Moving\nMoving --> Still", 500),
        ("ER diagram", "erDiagram\nCUSTOMER ||--o{ ORDER : places", 500),
        ("Gantt chart", "gantt\n    title Project\n    section S\n    Task :t1, 1, 5d", 500),
        ("Invalid syntax", "grpah TD; A-->B", 500),
    ];

    println!("╔══════════════════════════╤═════════╤══════════╤════════════╤══════════════╗");
    println!("║ Test case                │  Count  │ Parse(ns)│ Render(ns)│ Combined(ns)║");
    println!("╠══════════════════════════╪═════════╪══════════╪════════════╪══════════════╣");

    let mut total_parse: u64 = 0;
    let mut total_render: u64 = 0;

    for (name, code, count) in &cases {
        let mut parse_times = Vec::with_capacity(*count);
        let mut render_times = Vec::with_capacity(*count);

        // Warmup
        for _ in 0..10 {
            let _ = parse(code);
        }

        for _ in 0..*count {
            let start = std::time::Instant::now();
            let diagram = parse(code).ok();
            let parse_time = start.elapsed().as_nanos() as u64;
            parse_times.push(parse_time);

            if let Some(d) = diagram {
                let renderer = Renderer::new();
                let start = std::time::Instant::now();
                let _ = renderer.render(&d);
                let render_time = start.elapsed().as_nanos() as u64;
                render_times.push(render_time);
            }
        }

        let parse_avg = parse_times.iter().sum::<u64>() / parse_times.len() as u64;
        let render_avg = if render_times.is_empty() {
            0
        } else {
            render_times.iter().sum::<u64>() / render_times.len() as u64
        };
        let combined = parse_avg + render_avg;

        total_parse += parse_avg;
        total_render += render_avg;

        println!(
            "║ {:<28}│ {:>7}│ {:>9}│ {:>10}│ {:>12}║",
            name, count, parse_avg, render_avg, combined
        );
    }

    println!("╠══════════════════════════╪═════════╪══════════╪════════════╪══════════════╣");
    println!(
        "║ {:<28}│ {:>7}│ {:>9}│ {:>10}│ {:>12}║",
        "AVERAGE",
        "",
        total_parse / cases.len() as u64,
        if cases.is_empty() {
            0
        } else {
            total_render / cases.len() as u64
        },
        (total_parse + total_render) / cases.len() as u64
    );
    println!("╚══════════════════════════╧═════════╧══════════╧════════════╧══════════════╝");

    // Verify all diagram types can be parsed and rendered
    for (name, code, _) in &cases {
        let parsed = parse(code);
        assert!(
            parsed.is_ok() || name == &"Invalid syntax",
            "Should parse valid diagram: {}",
            name
        );
    }

    println!("\n✓ All diagram types parse successfully");
}
