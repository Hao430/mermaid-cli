# Mermaid-CLI Test Infrastructure & Specification

## 1. Test Architecture & Philosophy

### 1.1 Opaque-Box, Requirement-Driven Testing
The `mermaid-cli` end-to-end test suite follows a strict **opaque-box (black-box)** testing architecture. Tests interact exclusively with public library entry points (`mermaid_cli::render`, `mermaid_cli::parse`, `mermaid_cli::check`, `mermaid_cli::Renderer`) and validate the resulting SVG output strings.

Internal layout data structures (`BranchAnalyzer`, `LayoutGrid`, `SubgraphColumnGrid`, `EdgeBusOptions`) are treated as implementation details. By validating observable SVG geometry, DOM structure, CSS styling, and timing characteristics, the test suite guarantees that requirements are verified faithfully against user requirements rather than coupled to private refactorings.

### 1.2 Target Specification Derivation
Expected outputs and assertions are derived authoritatively from:
1. `ORIGINAL_REQUEST.md`: User prompt specifying horizontal parallel branch expansion, side-by-side branch rendering for G3/G5 in `examples/order_flow.mmd`, diagram height reduction from 4785px to $\le 3300\text{px}$, fillet curves ($r=8\text{px}$), and pill shield text labels.
2. `PROJECT.md`: Feature Inventory (18 features), Milestones (M1-M4), and Interface Contracts.
3. `docs/PLAN_HORIZONTAL_BRANCH_EXPANSION.md`: Mathematical models for symmetric lane slots ($\text{slot}_i = i - \frac{k-1}{2}$), subgraph dynamic width ($W_{sg} = M \cdot W_{node} + (M-1)S_h + 2P_{sg}$), node coordinates ($X_{node} = X_{sg\_center} + \text{slot} \cdot (W+S_h) - W/2$), and outer gutter routing (+28px).

### 1.3 Geometric & Semantic Invariants
The test suite inspects SVG strings to verify the following geometric and topological invariants:
- **Horizontal Disjointness**: Nodes on parallel branches must have non-overlapping X-spans ($\Delta X \ge W_{node} + S_h$).
- **Anti-Collapse Alignment**: Descendant nodes on a branch must maintain their branch's X column coordinate rather than collapsing to the canvas geometric center ($X_{center}$).
- **Monotonic Vertical Flow**: In top-down (`TD`) diagrams, child node $Y \ge \text{parent } Y + H_{node} + S_v$.
- **Bounding Box Integrity**: Subgraph boundary rectangles must encompass all member nodes with padding $P_{sg}$ and must not overlap adjacent phase subgraphs.
- **Corner Fillet Curvature**: Orthogonal turns must contain quadratic Bézier segments (`Q` SVG commands) with radius $r=8\text{px}$.
- **Pill Shield Underlay**: Edge labels must be backed by a rounded protective `<rect>` with pill styling (`rx="4"`).
- **Gutter Isolation**: Backward feedback loop edges must route outside the rightmost node boundary ($X \ge X_{max} + 28\text{px}$).
- **Non-Collision Guarantee**: For every pair of nodes $u \ne v$, their bounding boxes $[x, x+w] \times [y, y+h]$ must not overlap.

---

## 2. 4-Tier Test Taxonomy

The test suite is structured into four distinct tiers, providing exhaustive coverage across individual features, edge boundaries, combinatorics, and real-world system flows.

```
┌─────────────────────────────────────────────────────────────┐
│                 Tier 4: Real-World Scenarios                │
│    (order_flow.mmd full workflow, height <= 3300px, G3/G5)   │
├─────────────────────────────────────────────────────────────┤
│             Tier 3: Cross-Feature Combinations              │
│    (Pairwise interactions: Fork+Loop, Bus+Pill, Stacking)   │
├─────────────────────────────────────────────────────────────┤
│             Tier 2: Boundary & Corner Cases                 │
│      (>=5 tests per feature for all 18 features = 90 tests) │
├─────────────────────────────────────────────────────────────┤
│                 Tier 1: Feature Coverage                    │
│      (>=5 tests per feature for all 18 features = 90 tests) │
└─────────────────────────────────────────────────────────────┘
```

### 2.1 Tier 1: Feature Coverage (90 Tests)
Provides $\ge 5$ happy-path test cases for each of the 18 features from `PROJECT.md § Feature Inventory`:

| Feature ID | Feature Name | Test Range | Scope & Primary Assertions |
|------------|--------------|------------|----------------------------|
| F01 | DFS Back-Edge Detection | `tier1_f01_dfs_01` .. `_05` | Cycle detection isolates back-edges; forward DAG renders top-down without infinite loops; self-loops, 2-node cycles, and 3-node cycles isolated. |
| F02 | Fork Node Identification | `tier1_f02_fork_01` .. `_05` | Detects $d_{out} \ge 2$; identifies binary, ternary, and quaternary forks; distributes child edges to distinct endpoints. |
| F03 | Lane Slot Index Assignment | `tier1_f03_slot_01` .. `_05` | Symmetric lane slot calculation $\text{slot}_i = i - (k-1)/2$; validates left ($X < X_{fork}$) and right ($X > X_{fork}$) column placement. |
| F04 | Downstream Slot Propagation | `tier1_f04_prop_01` .. `_05` | Multi-hop branch chains maintain lane slot coordinate; eliminates center collapse on 2-hop, 3-hop, and 4-hop single-node rows. |
| F05 | Join Termination & Convergence | `tier1_f05_join_01` .. `_05` | Nodes with $d_{in} \ge 2$ terminate branch lanes; join node resets slot to 0.0 (aligned with fork/subgraph center). |
| F06 | Virtual Lane Reservation | `tier1_f06_resv_01` .. `_05` | Unbalanced branches (depth 1 vs depth 3) reserve corridor; prevents long branch from drifting into short branch's column. |
| F07 | Subgraph Dynamic Width | `tier1_f07_width_01` .. `_05` | Subgraph rect width matches $W_{sg} = M \cdot W + (M-1)S_h + 2P_{sg}$; expands dynamically with active lane count $M \in \{1, 2, 3, 4\}$. |
| F08 | Subgraph-Scoped Node X-Positioning | `tier1_f08_xpos_01` .. `_05` | Node X coordinates are scoped to containing subgraph center; parallel branches inside separate subgraphs do not interfere. |
| F09 | Branch Layer Compaction | `tier1_f09_compact_01` .. `_05` | Parallel branch nodes at same relative depth share vertical Y layer rows; total height compressed. |
| F10 | Subgraph Bounding Box Auto-Fitting | `tier1_f10_bbox_01` .. `_05` | Subgraph `<rect>` tightly fits member nodes with padding; title text placed cleanly in header area. |
| F11 | Phase-Level Subgraph Stacking | `tier1_f11_stack_01` .. `_05` | Sequential subgraphs stack vertically in topological order ($Y_{sg2} \ge Y_{sg1} + H_{sg1}$); zero vertical overlap. |
| F12 | T-Shaped Fork Distribution Bus | `tier1_f12_forkbus_01` .. `_05` | Fork outgoing edges use unified horizontal distribution bus at elevation $Y_{fork\_bus}$; shared horizontal segment present in SVG path. |
| F13 | T-Shaped Join Collection Bus | `tier1_f13_joinbus_01` .. `_05` | Converging branch edges gather at unified horizontal collection bus at elevation $Y_{join\_bus}$ before entering join node. |
| F14 | Quadratic Bézier Corner Fillets | `tier1_f14_fillet_01` .. `_05` | All 90-degree orthogonal edge turns emit `Q` Bézier curves with radius $r=8\text{px}$; smooth rounded corners verified. |
| F15 | Pill Shield Protected Labels | `tier1_f15_pill_01` .. `_05` | Edge labels are underlaid with `<rect rx="4">` (pill styling) with height 18px and centered text; prevents line clipping. |
| F16 | Outer Gutter Routing for Loop Edges | `tier1_f16_gutter_01` .. `_05` | Backward loop edges route via outer margin gutter ($X \ge X_{nodes\_max} + 28\text{px}$); does not cut through active diagram lanes. |
| F17 | Diagram Engine Zero-Regression | `tier1_f17_regress_01` .. `_05` | Existing non-flowchart diagram types (sequenceDiagram, classDiagram, stateDiagram, erDiagram, gantt) render successfully with valid SVG. |
| F18 | Sub-2ms Performance Budget | `tier1_f18_perf_01` .. `_05` | Layout calculation and rendering add $< 2\text{ms}$ latency overhead; small and medium diagrams render in sub-millisecond to low millisecond range. |

### 2.2 Tier 2: Boundary & Corner Cases (90 Tests)
Provides $\ge 5$ boundary/corner cases for each of the 18 features:
- **Zero Forks & Linear Chains**: Pure single-path DAGs ($d_{out} \le 1$), single-node diagrams, linear 5-node chains (no fork logic triggered, center alignment preserved).
- **Asymmetric & Extreme Branch Depths**: Highly unbalanced forks (1 hop on left, 6 hops on right; 0 intermediate hops on one branch).
- **Deeply Nested Forks**: Fork inside fork (hierarchical branching $A \to (B, C)$, $B \to (B1, B2)$), ensuring slot offsets compose cleanly without overlapping.
- **Multiple Feedback Loops & Nested Cycles**: Multiple backward edges to different upstream ancestors; backward edges crossing subgraph boundaries.
- **External Terminal Nodes**: Terminal sink nodes located outside subgraphs receiving edges from inside subgraphs (e.g. `End1` in `order_flow.mmd`).
- **High Out-Degree Forks**: Wide forks with $k=5, 6, 7$ parallel branches, testing wide canvas expansion without lane collision.
- **Multi-Way Join Convergence**: 3, 4, and 5 parallel branches converging into a single join node simultaneously.
- **Empty / Single-Node Subgraphs**: Subgraphs with 0 nodes or exactly 1 node, verifying dynamic width degenerates cleanly to single-column width.
- **Unicode & Special Character Labels**: Edge labels containing Chinese, Japanese, Korean, XML entities (`&`, `<`, `>`), and long phrases on pill shields.
- **Fillet Edge Distance Thresholds**: Edge segments shorter than $2 \times \text{radius}$, verifying radius clamping ($r \le \text{len}/2$) to prevent path inversion.
- **High Complexity Diagrams**: Flowcharts with 50+ nodes and dense connectivity for performance budget verification.
- **All 24 Diagram Types Coverage**: Comprehensive zero-regression smoke tests across pie, gitgraph, mindmap, timeline, sankey, xychart, etc.

### 2.3 Tier 3: Cross-Feature Combinations (15 Tests)
Validates pairwise and multi-feature interactions where subsystems intersect:
1. `tier3_combo_fork_plus_loop`: Fork distribution with one branch containing a feedback loop back to the fork node.
2. `tier3_combo_fork_plus_join_bus`: Parallel branches with both T-bus distribution and T-bus collection on the same flow.
3. `tier3_combo_unbalanced_branch_dynamic_width`: Unbalanced branches (depth 1 vs 4) inside a dynamically sized subgraph.
4. `tier3_combo_nested_subgraphs_with_fork`: Parallel branches spanning across nested subgraphs.
5. `tier3_combo_pill_shields_on_bus_routing`: Labeled edges routed via distribution bus with pill shield underlays.
6. `tier3_combo_gutter_plus_subgraph_boundary`: Outer gutter routing around multiple nested subgraphs.
7. `tier3_combo_multi_phase_stacking_cross_edges`: Multi-phase subgraphs with cross-phase bypass edges skipping a phase.
8. `tier3_combo_compact_layers_external_terminal`: Parallel branch compaction combined with terminal nodes outside subgraphs.
9. `tier3_combo_diamond_fork_join_fillets`: Diamond decision workflow with 90-degree fillets on all branches and join.
10. `tier3_combo_high_degree_fork_with_labels`: 5-way fork where all 5 branches carry pill-shielded labels.
11. `tier3_combo_cycle_with_branch_convergence`: Feedback loop connecting join node back to an intermediate branch node.
12. `tier3_combo_mixed_subgraph_and_global_nodes`: Graph containing nodes in subgraphs alongside independent root and sink nodes.
13. `tier3_combo_zero_regression_multi_diagram`: Batch sequence of flowchart, sequence, and state diagrams rendered consecutively.
14. `tier3_combo_asymmetric_join_with_bypass`: Direct bypass edge from fork directly to join alongside a 3-node branch.
15. `tier3_combo_stress_bus_fillet_shield`: Stress test combining bus routing, quadratic fillets, pill shields, and dynamic width.

### 2.4 Tier 4: Real-World Scenarios (10 Tests)
End-to-end validation against `examples/order_flow.mmd` (99 lines, 5 subgraphs G1-G5, 30+ nodes, cycles, external terminals, and Chinese labels):
1. `tier4_realworld_order_flow_renders_cleanly`: Full diagram parses and renders to valid SVG with `<svg`, `</svg>`, and non-zero content.
2. `tier4_realworld_order_flow_height_compression`: Canvas height is compressed significantly compared to baseline: **$\text{height} \le 3300\text{px}$** (down from baseline 4785px, a 31%+ compression).
3. `tier4_realworld_order_flow_g3_parallel_branches`: In G3 (支付处理), "在线支付" (C3) and "货到付款" (C9) render side-by-side in distinct horizontal columns ($|X_{C3} - X_{C9}| \ge 130\text{px}$).
4. `tier4_realworld_order_flow_g3_no_center_collapse`: In G3, downstream nodes C4 (回调结果) and C5 (已支付) maintain alignment with C3's column and do NOT collapse to canvas center ($|X_{C4} - X_{C3}| < 20\text{px}$, $|X_{C5} - X_{C3}| < 20\text{px}$).
5. `tier4_realworld_order_flow_g5_parallel_branches`: In G5 (签收与售后), "退货" (E5) and "换货" (E6) render side-by-side in distinct horizontal columns ($|X_{E5} - X_{E6}| \ge 130\text{px}$).
6. `tier4_realworld_order_flow_subgraphs_non_overlapping`: Subgraphs G1, G2, G3, G4, and G5 stack vertically in order without bounding box vertical collisions ($Y_{G_{k+1}} \ge Y_{G_k} + H_{G_k}$).
7. `tier4_realworld_order_flow_fillet_curves_present`: Orthogonal turns across the diagram contain smooth quadratic Bézier corner fillets (`Q` path commands).
8. `tier4_realworld_order_flow_pill_shields_present`: Edge labels (e.g. "高危 ≥ 80", "正常 < 80", "完全充足", "在线支付", "退货", "换货") have protective pill shields (`<rect rx="4">`).
9. `tier4_realworld_order_flow_zero_node_collisions`: Comprehensive pairwise check across all nodes ensuring zero bounding box overlap.
10. `tier4_realworld_order_flow_performance_budget`: End-to-end parsing, layout, and rendering of the entire order flow diagram completes within the performance budget.

---

## 3. Test Runner Commands & Pass/Fail Semantics

### 3.1 Verification Commands

```bash
# 1. Verify clean test compilation (zero errors, zero warnings)
cargo check --test e2e_branch_expansion_tests

# 2. Run the entire E2E test suite
cargo test --test e2e_branch_expansion_tests

# 3. Run specific test tiers selectively
cargo test --test e2e_branch_expansion_tests tier1_       # Tier 1: Feature Coverage (90 tests)
cargo test --test e2e_branch_expansion_tests tier2_       # Tier 2: Boundary & Corner Cases (90 tests)
cargo test --test e2e_branch_expansion_tests tier3_       # Tier 3: Cross-Feature Combinations (15 tests)
cargo test --test e2e_branch_expansion_tests tier4_       # Tier 4: Real-World Scenarios (10 tests)

# 4. Run tests for a specific feature (e.g. F03 Lane Slots, F14 Fillets)
cargo test --test e2e_branch_expansion_tests f03
cargo test --test e2e_branch_expansion_tests f14

# 5. Run with verbose output
cargo test --test e2e_branch_expansion_tests -- --nocapture
```

### 3.2 Pass/Fail Semantics
- **Exit Code 0 (PASS)**: All executed test cases satisfied all invariant assertions.
- **Exit Code Non-Zero (FAIL)**: An assertion failed. Failure diagnostics output:
  - Exact invariant violated (e.g., node horizontal collision, center collapse detected, height exceeded 3300px).
  - Observed coordinates vs expected bounds.
  - The offending Mermaid input code snippet.

### 3.3 Progressive Testability & Milestone Alignment
- Tests verify contracts against public APIs.
- Features implemented in existing code (fillet curves, pill shields, gutter routing, zero regression, baseline rendering) pass immediately.
- Features under active development in M1 (Forward DAG, Lane Slots), M2 (Column Grid, Dynamic Width, Compaction), and M3 (T-Bus Routing) contain precise target assertions that turn green as their corresponding milestones are integrated into `src/renderer/mod.rs`.
- Milestone 4 acceptance gate requires **100% pass across all 205 tests**.
