/// Performance benchmarks for mermaid-cli
/// Compares render times for various diagram types and batch sizes
use std::fs;
use std::time::{Duration, Instant};

fn load_test_diagrams() -> Vec<String> {
    let fixtures_path = "tests/fixtures/diagrams.json";
    let content = fs::read_to_string(fixtures_path).expect("Failed to read fixtures");

    let json: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse fixtures");

    json.as_object()
        .expect("Fixtures must be an object")
        .iter()
        .map(|(_, v)| v["input"].as_str().unwrap_or("").to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[test]
#[ignore] // Run with: cargo test bench_ -- --ignored --nocapture
fn bench_single_diagram() {
    let code = "graph TD; A[Start] --> B[Process] --> C[End]";

    let mut times = Vec::new();
    for _ in 0..100 {
        let start = Instant::now();
        let _ = mermaid_cli::render(code);
        times.push(start.elapsed());
    }

    let avg = times.iter().sum::<Duration>() / times.len() as u32;
    let min = times.iter().min().unwrap();
    let max = times.iter().max().unwrap();

    println!("\n=== Single Diagram Benchmark ===");
    println!("Average: {:.2}ms", avg.as_secs_f64() * 1000.0);
    println!("Min: {:.2}ms", min.as_secs_f64() * 1000.0);
    println!("Max: {:.2}ms", max.as_secs_f64() * 1000.0);
}

#[test]
#[ignore] // Run with: cargo test bench_ -- --ignored --nocapture
fn bench_batch_10_diagrams() {
    let diagrams = load_test_diagrams();
    let batch: Vec<&str> = diagrams.iter().take(10).map(|s| s.as_str()).collect();

    let start = Instant::now();
    for diagram in &batch {
        let _ = mermaid_cli::render(diagram);
    }
    let total = start.elapsed();
    let avg_per_diagram = total.as_secs_f64() / batch.len() as f64;

    println!("\n=== 10 Diagrams Batch Benchmark ===");
    println!("Total time: {:.2}ms", total.as_secs_f64() * 1000.0);
    println!("Per diagram: {:.2}ms", avg_per_diagram * 1000.0);
}

#[test]
#[ignore] // Run with: cargo test bench_ -- --ignored --nocapture
fn bench_batch_100_diagrams() {
    let diagrams = load_test_diagrams();
    let batch: Vec<&str> = diagrams
        .iter()
        .cycle()
        .take(100)
        .map(|s| s.as_str())
        .collect();

    let start = Instant::now();
    for diagram in &batch {
        let _ = mermaid_cli::render(diagram);
    }
    let total = start.elapsed();
    let avg_per_diagram = total.as_secs_f64() / batch.len() as f64;

    println!("\n=== 100 Diagrams Batch Benchmark ===");
    println!("Total time: {:.2}ms", total.as_secs_f64() * 1000.0);
    println!("Per diagram: {:.2}ms", avg_per_diagram * 1000.0);
}

#[test]
#[ignore] // Run with: cargo test bench_ -- --ignored --nocapture
fn bench_complex_diagram() {
    let complex_diagram = r#"
        graph TD
            A[User Login]
            B{Authentication}
            C[Database Query]
            D[Session Created]
            E[Error Logged]
            F[Email Sent]

            A --> B
            B -->|Valid| C
            B -->|Invalid| E
            C -->|Found| D
            C -->|Not Found| E
            E --> F
            D --> F
            F --> G[Complete]
    "#;

    let mut times = Vec::new();
    for _ in 0..50 {
        let start = Instant::now();
        let _ = mermaid_cli::render(complex_diagram);
        times.push(start.elapsed());
    }

    let avg = times.iter().sum::<Duration>() / times.len() as u32;

    println!("\n=== Complex Diagram Benchmark ===");
    println!("Average: {:.2}ms", avg.as_secs_f64() * 1000.0);
}

#[test]
#[ignore] // Run with: cargo test bench_ -- --ignored --nocapture
fn bench_all_diagram_types() {
    let diagrams = load_test_diagrams();

    println!("\n=== All Diagram Types Benchmark ===");
    println!("Testing {} diagrams...", diagrams.len());

    let start = Instant::now();
    let mut count = 0;
    for diagram in diagrams {
        if mermaid_cli::render(&diagram).is_ok() {
            count += 1;
        }
    }
    let total = start.elapsed();

    println!("Successfully rendered: {}", count);
    println!("Total time: {:.2}ms", total.as_secs_f64() * 1000.0);
    println!(
        "Average per diagram: {:.2}ms",
        (total.as_secs_f64() * 1000.0) / count as f64
    );
}

#[test]
#[ignore] // Run with: cargo test bench_ -- --ignored --nocapture
fn compare_render_times() {
    println!("\n=== Performance Comparison ===");
    println!("Single render vs batch processing\n");

    let simple_code = "graph TD; A-->B";

    // Single render
    let start = Instant::now();
    mermaid_cli::render(simple_code).unwrap();
    let single = start.elapsed();

    // Batch render
    let diagrams: Vec<&str> = vec![simple_code; 100];
    let start = Instant::now();
    for code in diagrams {
        mermaid_cli::render(code).unwrap();
    }
    let batch = start.elapsed();

    println!("Single render: {:.2}ms", single.as_secs_f64() * 1000.0);
    println!("100 renders: {:.2}ms total", batch.as_secs_f64() * 1000.0);
    println!(
        "100 renders: {:.2}ms average",
        (batch.as_secs_f64() * 1000.0) / 100.0
    );

    // Expected comparison with mmdc
    println!("\n--- Expected vs mmdc ---");
    println!("mmdc single: ~2400ms");
    println!("mermaid-cli single: {:.2}ms", single.as_secs_f64() * 1000.0);
    println!("Speedup: ~{:.0}x", 2400.0 / (single.as_secs_f64() * 1000.0));
}
