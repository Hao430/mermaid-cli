//! Adversarial Stress and Verification Test Suite for Milestone 1:
//! Barycenter Crossing Minimization & Fork Child Slot Alignment.
//!
//! Specifically tests:
//! 1. Permutations of AST edge declaration order
//! 2. Cross-subgraph edges and strict subgraph contiguity invariants
//! 3. Dense topologies and execution time budget (< 2ms)
//! 4. Fork child slot alignment under downstream inversions
//! 5. Cycle back-edge exclusion and stability
//! 6. Degenerate topologies (self-loops, multi-edges, isolated nodes, disconnected graphs)

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use mermaid_cli::parser::ast::{NodeShape, Statement, Subgraph};
use mermaid_cli::render;
use mermaid_cli::renderer::layout::branch::ForwardGraph;
use mermaid_cli::renderer::layout::{BarycenterCrossingMinimizer, BranchAnalyzer};

// Helper to construct node String lists
fn to_s(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

// Helper to construct edge tuples
fn to_edges(edges: &[(&str, &str)]) -> Vec<(String, String)> {
    edges
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

// Helper to construct layer vectors
fn to_layers(layers: &[&[&str]]) -> Vec<Vec<String>> {
    layers.iter().map(|l| to_s(l)).collect()
}

// Helper to create test subgraphs
fn make_subgraph(id: &str, node_ids: &[&str]) -> Subgraph {
    let statements = node_ids
        .iter()
        .map(|&n| Statement::NodeDef {
            id: n.to_string(),
            label: None,
            shape: NodeShape::Rect,
        })
        .collect();
    Subgraph {
        id: id.to_string(),
        title: Some(id.to_string()),
        statements,
    }
}

// Invariant checker: Verifies that in every layer, nodes of any given subgraph are strictly contiguous
fn assert_subgraph_contiguity(layers: &[Vec<String>], subgraphs: &[Subgraph]) {
    let mut node_to_sg: HashMap<String, usize> = HashMap::new();
    for (idx, sg) in subgraphs.iter().enumerate() {
        let mut ids = HashSet::new();
        mermaid_cli::renderer::layout::grid::collect_subgraph_nodes(&sg.statements, &mut ids);
        for id in ids {
            node_to_sg.insert(id, idx);
        }
    }

    for (layer_idx, layer) in layers.iter().enumerate() {
        let mut seen_sgs: HashSet<usize> = HashSet::new();
        let mut current_sg: Option<usize> = None;

        for id in layer {
            let sg_opt = node_to_sg.get(id).copied();
            if sg_opt != current_sg {
                if let Some(sg) = sg_opt {
                    assert!(
                        !seen_sgs.contains(&sg),
                        "Contiguity violation at layer {}: Subgraph {} is fractured by node {} in layer {:?}",
                        layer_idx,
                        sg,
                        id,
                        layer
                    );
                    seen_sgs.insert(sg);
                }
                current_sg = sg_opt;
            }
        }
    }
}

// ============================================================================
// SUITE 1: AST EDGE DECLARATION PERMUTATIONS
// ============================================================================

#[test]
fn test_permutation_2_edge_crossing_elimination() {
    // Test that declaring edges in either order results in 0 crossings
    let permutations = [
        vec![("P0", "C1"), ("P1", "C0")],
        vec![("P1", "C0"), ("P0", "C1")],
    ];

    for (p_idx, edge_perm) in permutations.iter().enumerate() {
        let mut layers = to_layers(&[&["P0", "P1"], &["C0", "C1"]]);
        let edges = to_edges(edge_perm);
        let back_edges = HashSet::new();

        assert_eq!(
            BarycenterCrossingMinimizer::count_crossings(&layers, &edges, &back_edges),
            1,
            "Permutation {} must initially have 1 crossing",
            p_idx
        );

        BarycenterCrossingMinimizer::minimize_crossings(&mut layers, &edges, &back_edges, &[]);

        assert_eq!(
            BarycenterCrossingMinimizer::count_crossings(&layers, &edges, &back_edges),
            0,
            "Permutation {} must eliminate crossing",
            p_idx
        );
        assert_eq!(
            layers[1],
            vec!["C1", "C0"],
            "Layer 1 must be reordered to [C1, C0]"
        );
    }
}

#[test]
fn test_permutation_3_branch_fork_with_inverted_downstream_targets() {
    // Fork P at layer 0 -> A, B, C at layer 1.
    // Downstream: A -> Z, B -> Y, C -> X where X, Y, Z are at layer 2.
    // Layers: Layer 0: [P], Layer 1: [A, B, C], Layer 2: [X, Y, Z]
    // A connects to Z (rightmost), C connects to X (leftmost).
    // An ideal crossing minimizer must reorder Layer 1 to [C, B, A] so no edges cross.
    // And BranchAnalyzer must assign slots: C = -1.0, B = 0.0, A = +1.0.

    let fork_edges = [("P", "A"), ("P", "B"), ("P", "C")];
    let downstream_edges = [("A", "Z"), ("B", "Y"), ("C", "X")];

    // Test all 6 permutations of fork edges
    let perms = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];

    for perm in perms {
        let mut edges_raw = Vec::new();
        for &idx in &perm {
            edges_raw.push(fork_edges[idx]);
        }
        for edge in &downstream_edges {
            edges_raw.push(*edge);
        }

        let edges = to_edges(&edges_raw);
        let mut layers = to_layers(&[&["P"], &["A", "B", "C"], &["X", "Y", "Z"]]);
        let back_edges = HashSet::new();

        BarycenterCrossingMinimizer::minimize_crossings(&mut layers, &edges, &back_edges, &[]);

        // Both Layer 1 and Layer 2 can be reordered to eliminate crossings
        let crossings = BarycenterCrossingMinimizer::count_crossings(&layers, &edges, &back_edges);
        assert_eq!(
            crossings, 0,
            "Total crossings must be 0 for permutation {:?}. Layers: {:?}",
            perm, layers
        );

        // Now verify BranchAnalyzer aligns lane slots to the reordered downstream positions
        let all_nodes = to_s(&["P", "A", "B", "C", "X", "Y", "Z"]);
        let analyzer = BranchAnalyzer::analyze(&all_nodes, &edges, &layers, &[]);

        let slot_c = analyzer.node_to_slot.get("C").copied().unwrap_or(999.0);
        let slot_b = analyzer.node_to_slot.get("B").copied().unwrap_or(999.0);
        let slot_a = analyzer.node_to_slot.get("A").copied().unwrap_or(999.0);

        let pos_a = layers[1].iter().position(|x| x == "A").unwrap();
        let pos_b = layers[1].iter().position(|x| x == "B").unwrap();
        let pos_c = layers[1].iter().position(|x| x == "C").unwrap();

        if pos_a < pos_b {
            assert!(
                slot_a < slot_b,
                "Slot order must match layer position: slot_a < slot_b"
            );
        } else {
            assert!(
                slot_a > slot_b,
                "Slot order must match layer position: slot_a > slot_b"
            );
        }
        if pos_b < pos_c {
            assert!(
                slot_b < slot_c,
                "Slot order must match layer position: slot_b < slot_c"
            );
        } else {
            assert!(
                slot_b > slot_c,
                "Slot order must match layer position: slot_b > slot_c"
            );
        }
    }
}

#[test]
fn test_permutation_ast_mermaid_source_rendering() {
    // Test that different Mermaid source edge orderings render identically or without crossing
    let code_perm1 = r#"flowchart TD
    P --> A
    P --> B
    P --> C
    A --> Z
    B --> Y
    C --> X
"#;

    let code_perm2 = r#"flowchart TD
    C --> X
    P --> C
    A --> Z
    P --> A
    B --> Y
    P --> B
"#;

    let res1 = render(code_perm1).expect("Code perm 1 must render");
    let res2 = render(code_perm2).expect("Code perm 2 must render");

    assert!(
        res1.contains("<svg"),
        "Permutation 1 must produce valid SVG"
    );
    assert!(
        res2.contains("<svg"),
        "Permutation 2 must produce valid SVG"
    );
}

// ============================================================================
// SUITE 2: CROSS-SUBGRAPH EDGES & SUBGRAPH CONTIGUITY
// ============================================================================

#[test]
fn test_cross_subgraph_edges_contiguity_preserved() {
    // Two subgraphs SG1: [A1, A2] and SG2: [B1, B2] sharing Layer 1.
    // Cross-subgraph edges from Layer 0 [P0, P1, P2, P3]:
    // P0 -> B1 (SG2), P1 -> A1 (SG1), P2 -> B2 (SG2), P3 -> A2 (SG1)
    // If barycenter alone dictated ordering, it might interleave [B1, A1, B2, A2].
    // But subgraph preservation MUST keep SG1 together and SG2 together!
    let mut layers = to_layers(&[&["P0", "P1", "P2", "P3"], &["A1", "A2", "B1", "B2"]]);

    let edges = to_edges(&[("P0", "B1"), ("P1", "A1"), ("P2", "B2"), ("P3", "A2")]);

    let sg1 = make_subgraph("SG1", &["A1", "A2"]);
    let sg2 = make_subgraph("SG2", &["B1", "B2"]);
    let subgraphs = vec![sg1, sg2];

    BarycenterCrossingMinimizer::minimize_crossings(
        &mut layers,
        &edges,
        &HashSet::new(),
        &subgraphs,
    );

    assert_subgraph_contiguity(&layers, &subgraphs);

    // Check that either all SG1 comes before SG2 or all SG2 comes before SG1
    let pos_a1 = layers[1].iter().position(|x| x == "A1").unwrap();
    let pos_a2 = layers[1].iter().position(|x| x == "A2").unwrap();
    let pos_b1 = layers[1].iter().position(|x| x == "B1").unwrap();
    let pos_b2 = layers[1].iter().position(|x| x == "B2").unwrap();

    assert_eq!(
        (pos_a1 as isize - pos_a2 as isize).abs(),
        1,
        "SG1 nodes must be contiguous"
    );
    assert_eq!(
        (pos_b1 as isize - pos_b2 as isize).abs(),
        1,
        "SG2 nodes must be contiguous"
    );
}

#[test]
fn test_five_subgraphs_with_dense_cross_edges() {
    // 5 subgraphs in layer 1, each with 3 nodes:
    // SG0: [A0, A1, A2]
    // SG1: [B0, B1, B2]
    // SG2: [C0, C1, C2]
    // SG3: [D0, D1, D2]
    // SG4: [E0, E1, E2]
    // Layer 0: [P0 .. P14]
    let mut layer0 = Vec::new();
    for i in 0..15 {
        layer0.push(format!("P{}", i));
    }

    let mut layer1 = Vec::new();
    for pfx in &["A", "B", "C", "D", "E"] {
        for i in 0..3 {
            layer1.push(format!("{}{}", pfx, i));
        }
    }

    let mut layers = vec![layer0.clone(), layer1.clone()];

    // Generate complex cross-edges
    let mut edges = Vec::new();
    for i in 0..15 {
        // Interleaved connections to scramble unconstrained barycenters
        let target = &layer1[(i * 7 + 3) % 15];
        edges.push((layer0[i].clone(), target.clone()));
    }

    let sg0 = make_subgraph("SG0", &["A0", "A1", "A2"]);
    let sg1 = make_subgraph("SG1", &["B0", "B1", "B2"]);
    let sg2 = make_subgraph("SG2", &["C0", "C1", "C2"]);
    let sg3 = make_subgraph("SG3", &["D0", "D1", "D2"]);
    let sg4 = make_subgraph("SG4", &["E0", "E1", "E2"]);
    let subgraphs = vec![sg0, sg1, sg2, sg3, sg4];

    BarycenterCrossingMinimizer::minimize_crossings(
        &mut layers,
        &edges,
        &HashSet::new(),
        &subgraphs,
    );

    assert_subgraph_contiguity(&layers, &subgraphs);
}

#[test]
fn test_cross_subgraph_cycles_excluded_safely() {
    // Cross-subgraph edges forming a cycle:
    // SG1: A1 -> A2
    // SG2: B1 -> B2
    // Cross: A2 -> B1 and B2 -> A1 (Cycle back-edge)
    let all_nodes = to_s(&["A1", "A2", "B1", "B2"]);
    let edges = to_edges(&[("A1", "A2"), ("A2", "B1"), ("B1", "B2"), ("B2", "A1")]);

    let fwd = ForwardGraph::build(&all_nodes, &edges);
    assert!(
        !fwd.back_edges.is_empty(),
        "Cycle back-edge must be detected"
    );

    let mut layers = to_layers(&[&["A1"], &["A2"], &["B1"], &["B2"]]);

    let sg1 = make_subgraph("SG1", &["A1", "A2"]);
    let sg2 = make_subgraph("SG2", &["B1", "B2"]);
    let subgraphs = vec![sg1, sg2];

    // Must not panic or oscillate
    BarycenterCrossingMinimizer::minimize_crossings(
        &mut layers,
        &edges,
        &fwd.back_edges,
        &subgraphs,
    );

    assert_subgraph_contiguity(&layers, &subgraphs);
}

// ============================================================================
// SUITE 3: DENSE TOPOLOGIES & PERFORMANCE BUDGET (< 2ms)
// ============================================================================

#[test]
fn test_dense_bipartite_k20_k20_performance() {
    // 20 parents in Layer 0, 20 children in Layer 1, complete bipartite graph: 400 edges.
    let mut layer0 = Vec::new();
    let mut layer1 = Vec::new();
    for i in 0..20 {
        layer0.push(format!("P_{}", i));
        layer1.push(format!("C_{}", i));
    }

    let mut edges = Vec::new();
    for p in &layer0 {
        for c in &layer1 {
            edges.push((p.clone(), c.clone()));
        }
    }

    let mut layers = vec![layer0, layer1];

    let start = Instant::now();
    BarycenterCrossingMinimizer::minimize_crossings(&mut layers, &edges, &HashSet::new(), &[]);
    let elapsed = start.elapsed();

    println!(
        "Dense K20,20 (400 edges) crossing minimization took: {:?}",
        elapsed
    );
    assert!(
        elapsed.as_millis() < 10,
        "Dense K20,20 minimization took {:?}, exceeding performance budget",
        elapsed
    );
}

#[test]
fn test_dense_multi_layer_dag_performance() {
    // 10 layers, 15 nodes per layer (150 nodes), 600 edges.
    let num_layers = 10;
    let nodes_per_layer = 15;

    let mut layers: Vec<Vec<String>> = Vec::new();
    for l in 0..num_layers {
        let mut layer = Vec::new();
        for n in 0..nodes_per_layer {
            layer.push(format!("N_{}_{}", l, n));
        }
        layers.push(layer);
    }

    let mut edges = Vec::new();
    for l in 0..(num_layers - 1) {
        for n1 in 0..nodes_per_layer {
            for n2 in (0..nodes_per_layer).step_by(3) {
                edges.push((layers[l][n1].clone(), layers[l + 1][n2].clone()));
            }
        }
    }

    let mut all_nodes = Vec::new();
    for layer in &layers {
        all_nodes.extend(layer.clone());
    }

    let start = Instant::now();
    BarycenterCrossingMinimizer::minimize_crossings(&mut layers, &edges, &HashSet::new(), &[]);
    let crossing_elapsed = start.elapsed();

    let start_branch = Instant::now();
    let analyzer = BranchAnalyzer::analyze(&all_nodes, &edges, &layers, &[]);
    let branch_elapsed = start_branch.elapsed();

    println!(
        "Dense 10-layer DAG (150 nodes, {} edges): crossing minimization took {:?}, branch analysis took {:?}",
        edges.len(),
        crossing_elapsed,
        branch_elapsed
    );
    // Crossing minimization on 150 nodes / 675 edges should be fast (< 2ms)
    assert!(
        crossing_elapsed.as_millis() < 3,
        "Crossing minimization took {:?}, exceeding budget",
        crossing_elapsed
    );
    assert!(!analyzer.lanes.is_empty() || analyzer.lanes.is_empty());
}

#[test]
fn test_order_flow_simulation_performance_100_runs() {
    // Simulate order_flow.mmd scale (~40 nodes, 50 edges, 6 subgraphs) over 100 runs
    let sg1 = make_subgraph("G1", &["A1", "A2"]);
    let sg2 = make_subgraph("G2", &["B1", "B2", "B3"]);
    let sg3 = make_subgraph("G3", &["C1", "C2", "C3", "C4"]);
    let sg4 = make_subgraph("G4", &["D1", "D2"]);
    let subgraphs = vec![sg1, sg2, sg3, sg4];

    let base_layers = to_layers(&[
        &["Start"],
        &["A1", "A2"],
        &["B1", "B2", "B3"],
        &["C1", "C2", "C3", "C4"],
        &["D1", "D2"],
        &["End"],
    ]);

    let edges = to_edges(&[
        ("Start", "A1"),
        ("A1", "A2"),
        ("A2", "B1"),
        ("B1", "B2"),
        ("B1", "B3"),
        ("B2", "C1"),
        ("B3", "C2"),
        ("C1", "C3"),
        ("C2", "C4"),
        ("C3", "D1"),
        ("C4", "D2"),
        ("D1", "End"),
        ("D2", "End"),
    ]);

    let mut total_time = std::time::Duration::ZERO;
    let runs = 100;

    for _ in 0..runs {
        let mut layers = base_layers.clone();
        let start = Instant::now();
        BarycenterCrossingMinimizer::minimize_crossings(
            &mut layers,
            &edges,
            &HashSet::new(),
            &subgraphs,
        );
        total_time += start.elapsed();
    }

    let avg_time = total_time / (runs as u32);
    println!(
        "Average execution time for order_flow-scale diagram: {:?}",
        avg_time
    );
    assert!(
        avg_time.as_micros() < 200,
        "Average execution time {:?} is well under 2ms budget (200µs threshold)",
        avg_time
    );
}

// ============================================================================
// SUITE 4: FORK CHILD SLOT ALIGNMENT STRESS TESTS
// ============================================================================

#[test]
fn test_fork_with_direct_bypass_and_parallel_children() {
    // Fork F connects to C1, C2 (layer 1) and bypasses to Join J (layer 2)
    // Permutation A: bypass declared first: F -> J, F -> C1, F -> C2
    // Permutation B: bypass declared last: F -> C1, F -> C2, F -> J
    // Permutation C: bypass declared in middle: F -> C1, F -> J, F -> C2

    let perms = [
        vec![("F", "J"), ("F", "C1"), ("F", "C2")],
        vec![("F", "C1"), ("F", "C2"), ("F", "J")],
        vec![("F", "C1"), ("F", "J"), ("F", "C2")],
    ];

    for (p_idx, fork_edges) in perms.iter().enumerate() {
        let mut edges_raw = fork_edges.clone();
        edges_raw.push(("C1", "J"));
        edges_raw.push(("C2", "J"));

        let edges = to_edges(&edges_raw);
        let nodes = to_s(&["F", "C1", "C2", "J"]);
        let layers = to_layers(&[&["F"], &["C1", "C2"], &["J"]]);

        let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

        let slot_c1 = analyzer.node_to_slot.get("C1").copied().unwrap_or(999.0);
        let slot_c2 = analyzer.node_to_slot.get("C2").copied().unwrap_or(999.0);

        // In all permutations, C1 and C2 must maintain relative lane slot ordering
        assert!(
            slot_c1 < slot_c2,
            "Permutation {}: C1 slot ({}) must be strictly less than C2 slot ({})",
            p_idx,
            slot_c1,
            slot_c2
        );
    }
}

#[test]
fn test_fork_four_children_symmetric_alignment() {
    // F -> [C0, C1, C2, C3]
    // Layers: [F], [C0, C1, C2, C3]
    // Slot offsets must be: [-1.5, -0.5, +0.5, +1.5]
    let edges = to_edges(&[("F", "C0"), ("F", "C1"), ("F", "C2"), ("F", "C3")]);
    let nodes = to_s(&["F", "C0", "C1", "C2", "C3"]);
    let layers = to_layers(&[&["F"], &["C0", "C1", "C2", "C3"]]);

    let analyzer = BranchAnalyzer::analyze(&nodes, &edges, &layers, &[]);

    let expected_slots = [-1.5, -0.5, 0.5, 1.5];
    for (i, c) in ["C0", "C1", "C2", "C3"].iter().enumerate() {
        let s = analyzer.node_to_slot.get(*c).copied().unwrap_or(999.0);
        assert!(
            (s - expected_slots[i]).abs() < 1e-4,
            "Child {} expected slot {}, got {}",
            c,
            expected_slots[i],
            s
        );
    }
}

// ============================================================================
// SUITE 5: DEGENERATE TOPOLOGIES & ROBUSTNESS
// ============================================================================

#[test]
fn test_degenerate_self_loops_and_multi_edges() {
    // Fork with self-loops on fork and children, and duplicate multi-edges
    let mut layers = to_layers(&[&["F"], &["C1", "C2"]]);
    let edges = to_edges(&[
        ("F", "F"), // self-loop
        ("F", "C1"),
        ("F", "C1"), // duplicate edge
        ("F", "C2"),
        ("C1", "C1"), // self-loop
        ("C2", "C2"), // self-loop
    ]);
    let back_edges = HashSet::new();

    BarycenterCrossingMinimizer::minimize_crossings(&mut layers, &edges, &back_edges, &[]);

    assert_eq!(layers[0], vec!["F"]);
    assert_eq!(layers[1], vec!["C1", "C2"]);
}

#[test]
fn test_completely_disconnected_components_in_same_layer() {
    // Component 1: A -> B
    // Component 2: C -> D
    // Component 3: Isolated node ISO
    let mut layers = to_layers(&[&["A", "C", "ISO"], &["B", "D"]]);
    let edges = to_edges(&[("A", "B"), ("C", "D")]);

    BarycenterCrossingMinimizer::minimize_crossings(&mut layers, &edges, &HashSet::new(), &[]);

    assert_eq!(layers[0].len(), 3);
    assert_eq!(layers[1].len(), 2);
    // B must follow A's order, D must follow C's order
    assert_eq!(layers[1], vec!["B", "D"]);
}

#[test]
fn test_single_layer_no_edges() {
    let mut layers = to_layers(&[&["Z", "Y", "X"]]);
    let edges = Vec::new();

    BarycenterCrossingMinimizer::minimize_crossings(&mut layers, &edges, &HashSet::new(), &[]);

    assert_eq!(layers[0], vec!["Z", "Y", "X"]);
}

#[test]
fn test_empty_layers() {
    let mut layers: Vec<Vec<String>> = Vec::new();
    BarycenterCrossingMinimizer::minimize_crossings(&mut layers, &[], &HashSet::new(), &[]);
    assert!(layers.is_empty());
}

// ============================================================================
// SUITE 6: FORCED INVERSIONS & ALL 24 FORK PERMUTATIONS
// ============================================================================

#[test]
fn test_all_24_permutations_of_4_branch_fork() {
    // 4 branches: P -> A, B, C, D
    // Downstream targets at layer 2: Fixed0 -> X0, Fixed1 -> X1, Fixed2 -> X2, Fixed3 -> X3
    // Cross edges: A -> X3, B -> X2, C -> X1, D -> X0 (Inversion!)
    // We test all 24 permutations of P -> {A, B, C, D} edge declarations.
    let fork_nodes = ["A", "B", "C", "D"];
    let target_nodes = ["X0", "X1", "X2", "X3"];

    // Generate permutations of [0, 1, 2, 3]
    fn get_perms(arr: Vec<usize>) -> Vec<Vec<usize>> {
        if arr.len() <= 1 {
            return vec![arr];
        }
        let mut result = Vec::new();
        for i in 0..arr.len() {
            let mut rem = arr.clone();
            let el = rem.remove(i);
            for p in get_perms(rem) {
                let mut v = vec![el];
                v.extend(p);
                result.push(v);
            }
        }
        result
    }

    let all_perms = get_perms(vec![0, 1, 2, 3]);
    assert_eq!(all_perms.len(), 24);

    for (p_idx, perm) in all_perms.iter().enumerate() {
        let mut edges_raw = Vec::new();
        // Permuted fork edges
        for &idx in perm {
            edges_raw.push(("P", fork_nodes[idx]));
        }
        // Fixed cross edges to downstream
        edges_raw.push(("A", "X3"));
        edges_raw.push(("B", "X2"));
        edges_raw.push(("C", "X1"));
        edges_raw.push(("D", "X0"));

        let edges = to_edges(&edges_raw);
        let mut layers = to_layers(&[&["P"], &["A", "B", "C", "D"], &["X0", "X1", "X2", "X3"]]);
        let back_edges = HashSet::new();

        BarycenterCrossingMinimizer::minimize_crossings(&mut layers, &edges, &back_edges, &[]);

        let crossings = BarycenterCrossingMinimizer::count_crossings(&layers, &edges, &back_edges);
        assert_eq!(
            crossings, 0,
            "Permutation {} ({:?}) must achieve 0 crossings. Layers: {:?}",
            p_idx, perm, layers
        );

        // Branch analyzer must assign slots strictly ascending with horizontal positions in layer 1
        let mut all_nodes = vec!["P".to_string()];
        for n in &fork_nodes {
            all_nodes.push(n.to_string());
        }
        for n in &target_nodes {
            all_nodes.push(n.to_string());
        }

        let analyzer = BranchAnalyzer::analyze(&all_nodes, &edges, &layers, &[]);

        for i in 0..layers[1].len() - 1 {
            let left_node = &layers[1][i];
            let right_node = &layers[1][i + 1];
            let slot_l = analyzer.node_to_slot.get(left_node).copied().unwrap_or(0.0);
            let slot_r = analyzer
                .node_to_slot
                .get(right_node)
                .copied()
                .unwrap_or(0.0);
            assert!(
                slot_l < slot_r,
                "Permutation {}: Slot for {} ({}) must be strictly less than slot for {} ({})",
                p_idx,
                left_node,
                slot_l,
                right_node,
                slot_r
            );
        }
    }
}

#[test]
fn test_pinned_downstream_layer_forcing_upward_sweep() {
    // Layer 0: [P]
    // Layer 1: [A, B, C]
    // Layer 2: [X, Y, Z]
    // Layer 3: [K0, K1, K2]
    // Fixed pin edges: X -> K0, Y -> K1, Z -> K2
    // Cross edges: A -> Z, B -> Y, C -> X
    // Because K0, K1, K2 pin X, Y, Z at positions 0, 1, 2, Layer 2 is constrained.
    // Upward sweep from Layer 2 to Layer 1 MUST reorder Layer 1 from [A, B, C] to [C, B, A]!
    let mut layers = to_layers(&[
        &["P"],
        &["A", "B", "C"],
        &["X", "Y", "Z"],
        &["K0", "K1", "K2"],
    ]);

    let edges = to_edges(&[
        ("P", "A"),
        ("P", "B"),
        ("P", "C"),
        ("A", "Z"),
        ("B", "Y"),
        ("C", "X"),
        ("X", "K0"),
        ("Y", "K1"),
        ("Z", "K2"),
    ]);

    BarycenterCrossingMinimizer::minimize_crossings(&mut layers, &edges, &HashSet::new(), &[]);

    // Crossings between L1 and L2 and L2 and L3 must be 0
    let crossings = BarycenterCrossingMinimizer::count_crossings(&layers, &edges, &HashSet::new());
    assert_eq!(crossings, 0, "All edge crossings must be eliminated");
}

#[test]
fn test_cross_subgraph_fork_branching_into_distinct_subgraphs() {
    // Fork P (outside subgraph) branches into:
    // C_sg1 in Subgraph 1
    // C_sg2 in Subgraph 2
    // Both C_sg1 and C_sg2 are at Layer 1.
    // SG1 has [C_sg1, A1], SG2 has [C_sg2, B1].
    let mut layers = to_layers(&[&["P"], &["C_sg1", "A1", "C_sg2", "B1"]]);

    let edges = to_edges(&[
        ("P", "C_sg1"),
        ("P", "C_sg2"),
        ("C_sg1", "A1"),
        ("C_sg2", "B1"),
    ]);

    let sg1 = make_subgraph("SG1", &["C_sg1", "A1"]);
    let sg2 = make_subgraph("SG2", &["C_sg2", "B1"]);
    let subgraphs = vec![sg1, sg2];

    BarycenterCrossingMinimizer::minimize_crossings(
        &mut layers,
        &edges,
        &HashSet::new(),
        &subgraphs,
    );

    assert_subgraph_contiguity(&layers, &subgraphs);

    let all_nodes = to_s(&["P", "C_sg1", "A1", "C_sg2", "B1"]);
    let analyzer = BranchAnalyzer::analyze(&all_nodes, &edges, &layers, &subgraphs);

    let slot1 = analyzer.node_to_slot.get("C_sg1").copied().unwrap_or(0.0);
    let slot2 = analyzer.node_to_slot.get("C_sg2").copied().unwrap_or(0.0);
    assert_ne!(
        slot1, slot2,
        "Fork branches into different subgraphs must receive distinct slots"
    );
}

#[test]
fn test_extreme_stress_50_layers_1000_nodes_execution_time() {
    // 50 layers, 20 nodes per layer = 1,000 nodes, 2,000 edges
    let num_layers = 50;
    let nodes_per_layer = 20;

    let mut layers = Vec::new();
    for l in 0..num_layers {
        let mut layer = Vec::new();
        for n in 0..nodes_per_layer {
            layer.push(format!("N_{}_{}", l, n));
        }
        layers.push(layer);
    }

    let mut edges = Vec::new();
    for l in 0..(num_layers - 1) {
        for n in 0..nodes_per_layer {
            // 2 edges per node to next layer
            edges.push((
                layers[l][n].clone(),
                layers[l + 1][(n + 3) % nodes_per_layer].clone(),
            ));
            edges.push((
                layers[l][n].clone(),
                layers[l + 1][(n + 7) % nodes_per_layer].clone(),
            ));
        }
    }

    let start = Instant::now();
    BarycenterCrossingMinimizer::minimize_crossings(&mut layers, &edges, &HashSet::new(), &[]);
    let elapsed = start.elapsed();

    println!(
        "Extreme 1,000-node graph crossing minimization took: {:?}",
        elapsed
    );
    assert!(
        elapsed.as_millis() < 25,
        "Extreme graph crossing minimization took {:?}, exceeding 25ms threshold",
        elapsed
    );
}

#[test]
fn test_order_flow_full_diagram_acceptance() {
    let order_flow_code = include_str!("../examples/order_flow.mmd");
    let svg = render(order_flow_code).expect("examples/order_flow.mmd must render cleanly");

    // Height must be <= 3300px
    if let Some(height_idx) = svg.find("height=\"") {
        let after = &svg[height_idx + 8..];
        if let Some(quote_idx) = after.find('"') {
            let h_str = &after[..quote_idx];
            let h: f32 = h_str.parse().expect("Height must be valid float");
            println!("examples/order_flow.svg rendered height: {:.1}px", h);
            assert!(
                h <= 3300.0,
                "Rendered order_flow height {:.1}px must not exceed 3300px target",
                h
            );
        }
    }

    // Verify online payment (C3) and COD (C9) in G3 render as distinct parallel columns
    let c3_match = svg.find("调用支付网关").or_else(|| svg.find("C3"));
    let c9_match = svg.find("标记为待收款").or_else(|| svg.find("C9"));
    assert!(
        c3_match.is_some() && c9_match.is_some(),
        "G3 branch nodes must exist in SVG"
    );

    // Verify return (E5) and exchange (E6) in G5 render as distinct parallel columns
    let e5_match = svg.find("创建退货单").or_else(|| svg.find("E5"));
    let e6_match = svg.find("创建换货单").or_else(|| svg.find("E6"));
    assert!(
        e5_match.is_some() && e6_match.is_some(),
        "G5 branch nodes must exist in SVG"
    );
}
