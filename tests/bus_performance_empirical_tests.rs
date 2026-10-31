use mermaid_cli::render;
use std::fs;
use std::time::Instant;

#[test]
fn test_order_flow_render_latency_budget() {
    let mmd_content = fs::read_to_string("examples/order_flow.mmd")
        .expect("Failed to read examples/order_flow.mmd");

    // Warmup
    for _ in 0..10 {
        let _ = render(&mmd_content).unwrap();
    }

    let iterations = 200;
    let start = Instant::now();
    for _ in 0..iterations {
        let svg = render(&mmd_content).unwrap();
        assert!(!svg.is_empty());
    }
    let total_elapsed = start.elapsed();
    let avg_ms = total_elapsed.as_secs_f64() * 1000.0 / iterations as f64;

    println!(
        "\n[M3 PERF REPORT] order_flow.mmd full render: {} iterations took {:?}, avg {:.4} ms / render",
        iterations, total_elapsed, avg_ms
    );

    assert!(
        avg_ms < 5.0,
        "Full render avg time {:.4}ms exceeds 5.0ms budget",
        avg_ms
    );
}

#[test]
fn test_bus_deduplication_incremental_overhead() {
    // Baseline: linear chain of 30 nodes (no forks, no joins)
    let mut base_code = String::from("flowchart TD\n");
    for i in 0..29 {
        base_code.push_str(&format!("    N{} --> N{}\n", i, i + 1));
    }

    // Graph with 10 forks and 10 joins (dense bus routing)
    let mut fork_join_code = String::from("flowchart TD\n");
    for i in (0..20).step_by(2) {
        // Fork node i into branch a and b, joining at i+2
        fork_join_code.push_str(&format!("    F{} --> B{}_1\n", i, i));
        fork_join_code.push_str(&format!("    F{} --> B{}_2\n", i, i));
        fork_join_code.push_str(&format!("    B{}_1 --> J{}\n", i, i + 2));
        fork_join_code.push_str(&format!("    B{}_2 --> J{}\n", i, i + 2));
        fork_join_code.push_str(&format!("    J{} --> F{}\n", i + 2, i + 2));
    }

    // Warmup
    for _ in 0..20 {
        let _ = render(&base_code).unwrap();
        let _ = render(&fork_join_code).unwrap();
    }

    let iterations = 1000;
    let start_base = Instant::now();
    for _ in 0..iterations {
        let _ = render(&base_code).unwrap();
    }
    let base_elapsed = start_base.elapsed();

    let start_bus = Instant::now();
    for _ in 0..iterations {
        let _ = render(&fork_join_code).unwrap();
    }
    let bus_elapsed = start_bus.elapsed();

    let avg_base_ms = base_elapsed.as_secs_f64() * 1000.0 / iterations as f64;
    let avg_bus_ms = bus_elapsed.as_secs_f64() * 1000.0 / iterations as f64;
    let incremental_ms = (avg_bus_ms - avg_base_ms).max(0.0);

    println!(
        "[M3 PERF REPORT] Bus Deduplication Incremental Overhead:\n  Base Linear DAG (30 nodes, 0 forks) : {:.4} ms/render\n  With Forks & Joins (30 nodes, 10 buses) : {:.4} ms/render\n  Incremental Bus Overhead                : {:.4} ms (budget: < 2.0 ms)",
        avg_base_ms, avg_bus_ms, incremental_ms
    );

    assert!(
        incremental_ms < 2.0,
        "Incremental bus deduplication overhead {:.4}ms exceeds 2.0ms budget!",
        incremental_ms
    );
}

#[test]
fn test_high_degree_fork_join_stress_latency() {
    // 8-way fork and 8-way join
    let mut code = String::from("flowchart TD\n");
    for i in 1..=8 {
        code.push_str(&format!("    Start --> Branch{}\n", i));
    }
    for i in 1..=8 {
        code.push_str(&format!("    Branch{} --> EndNode\n", i));
    }

    // Warmup
    for _ in 0..20 {
        let _ = render(&code).unwrap();
    }

    let iterations = 500;
    let start = Instant::now();
    for _ in 0..iterations {
        let svg = render(&code).unwrap();
        assert!(!svg.is_empty());
    }
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_secs_f64() * 1000.0 / iterations as f64;

    println!(
        "[M3 PERF REPORT] 8-way fork + 8-way join stress ({} iterations): avg {:.4} ms/render",
        iterations, avg_ms
    );

    assert!(
        avg_ms < 2.0,
        "8-way fork-join stress avg time {:.4}ms exceeds 2.0ms budget",
        avg_ms
    );
}
