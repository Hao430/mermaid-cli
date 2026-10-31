use mermaid_cli::render;
use mermaid_cli::renderer::routing::GutterChannelAssigner;
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

#[test]
fn test_order_flow_render_latency() {
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
        "\n[PERF REPORT] order_flow.mmd full render: {} iterations took {:?}, avg {:.4} ms / render",
        iterations, total_elapsed, avg_ms
    );

    assert!(
        avg_ms < 5.0,
        "Full render avg time {:.4}ms exceeds 5.0ms budget",
        avg_ms
    );
}

#[test]
fn test_gutter_channel_assigner_direct_latency() {
    // Construct realistic node positions and subgraphs for a 50-node diagram with 20 backward edges
    let mut node_positions = HashMap::new();
    let mut backward_edges = Vec::new();
    let node_size = (130.0, 65.0);

    for i in 0..50 {
        let id = format!("N{}", i);
        let x = 100.0 + (i % 5) as f32 * 160.0;
        let y = 100.0 + (i / 5) as f32 * 100.0;
        node_positions.insert(id, (x, y));
    }

    for k in 1..21 {
        let from = format!("N{}", (k * 2) % 49 + 1);
        let to = format!("N{}", (k * 2) % 10);
        backward_edges.push((from, to));
    }

    let mut subgraph_bounds = HashMap::new();
    subgraph_bounds.insert("sub1".to_string(), (80.0, 80.0, 800.0, 500.0));
    subgraph_bounds.insert("sub2".to_string(), (80.0, 600.0, 800.0, 500.0));

    // Warmup
    for _ in 0..100 {
        let _ = GutterChannelAssigner::assign_channels(
            &backward_edges,
            &node_positions,
            node_size,
            &subgraph_bounds,
        );
    }

    let iterations = 5000;
    let start = Instant::now();
    for _ in 0..iterations {
        let res = GutterChannelAssigner::assign_channels(
            &backward_edges,
            &node_positions,
            node_size,
            &subgraph_bounds,
        );
        assert_eq!(res.len(), 20);
    }
    let total_elapsed = start.elapsed();
    let avg_us = total_elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64;
    let avg_ms = avg_us / 1000.0;

    println!(
        "[PERF REPORT] GutterChannelAssigner::assign_channels (20 loops, 50 nodes): {} calls took {:?}, avg {:.3} µs ({:.4} ms) / call",
        iterations, total_elapsed, avg_us, avg_ms
    );

    assert!(
        avg_ms < 0.1,
        "GutterChannelAssigner direct time {:.4}ms exceeds 0.1ms budget (requirement is < 2ms)",
        avg_ms
    );
}

#[test]
fn test_gutter_channel_incremental_execution_overhead() {
    // Generate a baseline 30-node flowchart without backward edges
    let mut base_code = String::from("flowchart TD\n");
    for i in 0..29 {
        base_code.push_str(&format!("    N{} --> N{}\n", i, i + 1));
    }

    // Generate diagram with 10 backward loops
    let mut loop_code = base_code.clone();
    for i in 1..=10 {
        let src = i * 2 + 5;
        let dst = i;
        loop_code.push_str(&format!("    N{} --> N{}\n", src, dst));
    }

    // Warmup
    for _ in 0..20 {
        let _ = render(&base_code).unwrap();
        let _ = render(&loop_code).unwrap();
    }

    let iterations = 1000;
    let start_base = Instant::now();
    for _ in 0..iterations {
        let _ = render(&base_code).unwrap();
    }
    let base_elapsed = start_base.elapsed();

    let start_loop = Instant::now();
    for _ in 0..iterations {
        let _ = render(&loop_code).unwrap();
    }
    let loop_elapsed = start_loop.elapsed();

    let avg_base_ms = base_elapsed.as_secs_f64() * 1000.0 / iterations as f64;
    let avg_loop_ms = loop_elapsed.as_secs_f64() * 1000.0 / iterations as f64;
    let incremental_ms = (avg_loop_ms - avg_base_ms).max(0.0);

    println!(
        "[PERF REPORT] Incremental Overhead Check:\n  Base DAG (30 nodes, 0 loops) : {:.4} ms/render\n  With Gutters (10 back loops)  : {:.4} ms/render\n  Incremental Gutter Overhead   : {:.4} ms (budget: < 2.0 ms)",
        avg_base_ms, avg_loop_ms, incremental_ms
    );

    assert!(
        incremental_ms < 2.0,
        "Incremental gutter routing overhead {:.4}ms exceeds 2.0ms budget!",
        incremental_ms
    );
}

#[test]
fn test_adversarial_stress_50_nested_loops() {
    let mut code = String::from("flowchart TD\n");
    for i in 0..50 {
        code.push_str(&format!("    A{} --> A{}\n", i, i + 1));
    }
    for i in 1..=50 {
        code.push_str(&format!("    A{} --> A0\n", i));
    }

    let start = Instant::now();
    let res = render(&code).expect("50 nested loops must render without panic");
    let elapsed = start.elapsed();
    assert!(!res.is_empty());
    println!(
        "[PERF REPORT] 50 nested loops single render took {:?}",
        elapsed
    );
}
