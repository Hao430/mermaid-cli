# TEST_READY: Advanced Orthogonal Routing & Visual Quality Test Suite (R1–R4)

## Executive Summary
The independent, opaque-box, requirement-driven E2E test suite for **Advanced Orthogonal Channel Routing, Bus Deduplication, Arc Hop Bridges & Label Collision Avoidance (Requirements R1–R4)** has been implemented, validated, and published in `tests/e2e_advanced_routing_tests.rs`.

This test suite covers all 4 architectural pillars and provides 4 explicit mathematical verification oracles as quality gates for Milestones M1, M2, M3, M4, and M5.

Combined with the existing 205 baseline E2E tests in `tests/e2e_branch_expansion_tests.rs` and 275 unit/stress tests, the diagram engine now possesses a total of **520+ tests** enforcing structural, visual, and performance contracts.

---

## 1. Test Suite Architecture: `tests/e2e_advanced_routing_tests.rs`

The suite contains exactly **40 requirement-driven tests** (10 per requirement) and **4 mathematical verification oracles**:

| Group | Focus / Architectural Scope | Target Milestone | Test Count | Baseline Pass | Baseline Fail | Defect Escalate Target |
|-------|-----------------------------|------------------|------------|---------------|---------------|------------------------|
| **Group R1** | Multi-level stepped gutter channels ($\Delta X \ge 16\text{px}$), nested stepped loop routing (`C10 -> C4` vs `C7 -> C3`), obstacle-aware base clearance, bypass corridor isolation | M2 | 10 | 7 | 3 | M2 Gutter Track Assigner |
| **Group R2** | T-bus trunk deduplication (vertical stems & horizontal buses drawn at most once in SVG), multi-branch fillet turning disambiguation ($\Delta X \ge 30\text{px}$) | M3 | 10 | 5 | 5 | M3 Bus Deduplicator |
| **Group R3** | Sugiyama in-layer barycenter crossing minimization, orthogonal transverse crossing detection, semi-circular Arc Hop bridges (`A rx ry 0 0 sweep ...`, span 8–12px) | M1, M4 | 10 | 8 | 2 | M1 & M4 Crossing & Hop Generator |
| **Group R4** | Global pill shield pairwise mutual non-collision, zero corner fillet obscuration ($[cx-8, cx+8] \times [cy-8, cy+8]$), zero node bounding box intersection, axial sliding & normal offset | M4 | 10 | 8 | 2 | M4 Label Collision AABB Placer |
| **Total** | **Advanced Routing Test Suite** | **M1–M5** | **40** | **28 (70%)** | **12 (30%)** | **Ready as Quality Gate** |

---

## 2. The 4 Mathematical Verification Oracles

All 4 verification oracles are exposed as public helper functions in `tests/e2e_advanced_routing_tests.rs::helpers`:

### Oracle 1: Gutter Channel Separation (`verify_gutter_separation_oracle`)
- **Mathematical Specification**: For any two concurrent vertical segments in outer gutters (feedback loops where $Y_{end} < Y_{start}$), if their vertical spans overlap ($\min(Y_{max1}, Y_{max2}) - \max(Y_{min1}, Y_{min2}) > 4.0\text{px}$), their horizontal coordinates must satisfy:
  $$\Delta X = |X_1 - X_2| \ge 16.0\text{px}$$
- **Stepped Nesting Invariant**: Shorter feedback loops route on inner tracks; longer enclosing feedback loops route on outer tracks ($X_{outer} > X_{inner}$).

### Oracle 2: Bus Collinear Segment Deduplication (`verify_bus_deduplication_oracle`)
- **Mathematical Specification**: Across all distinct `<path>` elements in the SVG, no two rectilinear segments $s_1$ and $s_2$ ($path\_idx_1 \ne path\_idx_2$) may be collinear with an overlap length $\ge 1.0\text{px}$:
  $$\text{Overlap}(s_1, s_2) < 1.0\text{px}$$
- Eliminates duplicate vertical stems below fork nodes and duplicate drops into join nodes.

### Oracle 3: Arc Hop Presence on Orthogonal Crossings (`verify_arc_hop_presence_oracle`)
- **Mathematical Specification**: Every transverse intersection between a horizontal segment and a vertical segment across different paths must feature an SVG Arc Hop (`A` command) bridge with radius $r \in [4.0, 6.0]\text{px}$ (span $8\text{px} \le 2r \le 12\text{px}$):
  $$\text{dist}((X_{cross}, Y_{cross}), (X_{hop}, Y_{hop})) \le 12.0\text{px}$$

### Oracle 4: Pill Shield Mutual Non-Collision & Fillet Protection (`verify_pill_shield_oracle`)
- **Mathematical Specification**:
  1. **Pairwise Disjointness**: $\forall i \ne j, \text{AABB}(S_i) \cap \text{AABB}(S_j) = \emptyset$
  2. **Corner Fillet Protection**: $\forall \text{fillet at } (cx, cy), \text{AABB}(S) \cap [cx - 8, cx + 8] \times [cy - 8, cy + 8] = \emptyset$
  3. **Node Bounding Box Protection**: $\forall \text{node } N, \text{AABB}(S) \cap \text{AABB}(N) = \emptyset$

---

## 3. Concrete Test Inventory Breakdown (40 Tests)

### Group R1: Multi-Level Gutter & Bypass Channels
1. `test_r1_gutter_01_two_loops_channel_separation_16px`: Verifies $\Delta X \ge 16\text{px}$ between two concurrent loops.
2. `test_r1_gutter_02_three_nested_loops_stepped_nesting`: Verifies 3 stepped non-overlapping outer tracks ($X_3 > X_2 > X_1$).
3. `test_r1_gutter_03_subgraph_perimeter_cleared`: Verifies gutter channel clears subgraph right boundary plus padding ($X \ge X_{sg} + W_{sg} + 16\text{px}$).
4. `test_r1_gutter_04_cross_layer_bypass_corridor_isolation`: Verifies forward bypass edge does not intersect intermediate nodes.
5. `test_r1_gutter_05_feedback_loop_fillet_curves_preserved`: Verifies smooth quadratic Bézier corner fillets on gutter edges.
6. `test_r1_gutter_06_wide_canvas_gutter_spacing`: Verifies canvas width automatically expands to encompass outer gutters.
7. `test_r1_gutter_07_asymmetric_depth_gutter_routing`: Verifies distinct channels for loops originating from different depths to the same node.
8. `test_r1_gutter_08_order_flow_c10_c7_stepped_gutter`: Verifies $|X(C10 \to C4) - X(C7 \to C3)| \ge 16\text{px}$ in `order_flow.svg`.
9. `test_r1_gutter_09_direct_loop_to_same_layer`: Verifies outer gutter track for 2-node cycle clearing node right bounds.
10. `test_r1_gutter_10_multiple_subgraphs_gutter_avoidance`: Verifies feedback loop clears the rightmost perimeter of all traversed subgraphs.

### Group R2: Bus Trunk Deduplication & Multi-Branch Disambiguation
1. `test_r2_bus_01_fork_vertical_stem_drawn_exactly_once`: Verifies vertical stem from fork node to bus is emitted at most once.
2. `test_r2_bus_02_join_vertical_stem_drawn_exactly_once`: Verifies vertical drop into join node is emitted at most once.
3. `test_r2_bus_03_horizontal_bus_segment_uniqueness`: Verifies horizontal bus segments have zero duplicate collinear overlaps.
4. `test_r2_bus_04_ternary_fork_zero_duplicate_segments`: Verifies zero duplicate segments across 3 fork branch paths.
5. `test_r2_bus_05_multi_branch_fillet_centers_disambiguated`: Verifies turning centers have distinct coordinates with spacing $\ge 20\text{px}$.
6. `test_r2_bus_06_diamond_fork_join_bus_deduplication`: Verifies zero duplicate collinear stems on diamond fork & join.
7. `test_r2_bus_07_quaternary_fork_clean_fillet_spacing`: Verifies 4-way fork branches maintain clean turning fillets.
8. `test_r2_bus_08_order_flow_fork_trunks_deduplicated`: Verifies fork `B1` vertical stem in `order_flow.svg` is drawn at most once.
9. `test_r2_bus_09_asymmetric_join_collection_bus`: Verifies asymmetric join collection bus has zero duplicate segments.
10. `test_r2_bus_10_high_degree_fork_bus_clarity`: Verifies 5-way fork deduplication and branch clarity.

### Group R3: Barycenter Crossing Minimization & Arc Bridges
1. `test_r3_cross_01_barycenter_reordering_reduces_crossings`: Verifies in-layer barycenter ordering eliminates topological crossing.
2. `test_r3_cross_02_orthogonal_intersection_arc_hop_present`: Verifies transverse orthogonal intersections feature Arc Hop bridges.
3. `test_r3_cross_03_arc_hop_geometry_span_8_to_12px`: Verifies Arc Hop radius $r \in [4, 6]\text{px}$ (span 8–12px).
4. `test_r3_cross_04_arc_hop_command_syntax_svg_a`: Verifies SVG path contains standard `A rx ry 0 0 sweep x y` command syntax.
5. `test_r3_cross_05_horizontal_edge_hops_over_vertical`: Verifies horizontal edge leaps cleanly over vertical track.
6. `test_r3_cross_06_bypass_line_arc_hops_over_bus`: Verifies bypass line leaping over horizontal bus features Arc Hop.
7. `test_r3_cross_07_multiple_crossings_multiple_arc_hops`: Verifies multiple crossings each generate an Arc Hop bridge.
8. `test_r3_cross_08_crossing_near_corner_fillet_clearance`: Verifies Arc Hop maintains $\ge 8\text{px}$ clearance from corner fillets.
9. `test_r3_cross_09_order_flow_arc_hops_present`: Verifies transverse intersections in `order_flow.svg` feature Arc Hop bridges.
10. `test_r3_cross_10_planar_graph_zero_arc_hops_needed`: Verifies planar graph without crossings generates 0 Arc Hops.

### Group R4: Label & Pill Shield Collision Avoidance
1. `test_r4_shield_01_pairwise_pill_shield_mutual_disjointness`: Verifies all pill shields are pairwise mutually disjoint.
2. `test_r4_shield_02_corner_fillet_clearance_no_obscuration`: Verifies pill shields do not intersect corner fillet envelopes.
3. `test_r4_shield_03_node_bounding_box_clearance`: Verifies pill shields do not intersect node bounding boxes.
4. `test_r4_shield_04_long_label_bus_segment_sliding`: Verifies long label slides or offsets to avoid corner fillets.
5. `test_r4_shield_05_adjacent_fork_labels_staggered`: Verifies adjacent fork branch pill shields have staggered non-colliding positions.
6. `test_r4_shield_06_chinese_labels_pill_shield_disjointness`: Verifies multi-character Chinese edge labels do not collide.
7. `test_r4_shield_07_vertical_segment_label_clearance`: Verifies labels on vertical segments maintain clearance from nodes.
8. `test_r4_shield_08_order_flow_all_pill_shields_collision_free`: Verifies all 17+ pill shields in `order_flow.svg` are pairwise disjoint.
9. `test_r4_shield_09_order_flow_fillets_unobstructed_by_shields`: Verifies "高危 ≥ 80" and other pill shields do not obstruct 90-degree corner fillets.
10. `test_r4_shield_10_shield_underlay_rx4_preserved`: Verifies pill shields maintain rounded pill styling (`rx:4`) and height 18px.

---

## 4. Test Execution Commands

```bash
# 1. Compilation verification (zero warnings, zero errors)
cargo check --test e2e_advanced_routing_tests

# 2. Strict linter & formatting check
cargo clippy --test e2e_advanced_routing_tests
cargo fmt --check

# 3. Run all 40 advanced routing tests
cargo test --test e2e_advanced_routing_tests

# 4. Run individual requirement groups
cargo test --test e2e_advanced_routing_tests group_r1_multi_level_gutter_and_bypass
cargo test --test e2e_advanced_routing_tests group_r2_bus_trunk_deduplication
cargo test --test e2e_advanced_routing_tests group_r3_crossing_minimization_and_arc_hops
cargo test --test e2e_advanced_routing_tests group_r4_label_pill_shield_collision_avoidance

# 5. Run full test suite across entire engine (480 baseline + 40 new = 520 tests)
cargo test
```

---

## 5. Milestone Transition Quality Gates

- **M1 (Barycenter Crossing Minimization)**:
  - Turn `test_r3_cross_01_barycenter_reordering_reduces_crossings` green.
  - Zero regression on existing 480 tests.
- **M2 (Multi-Level Gutter & Bypass Assignment)**:
  - Turn `test_r1_gutter_01..10` 100% green (resolving the 3 baseline gutter failures).
  - Assert $|X(C10 \to C4) - X(C7 \to C3)| \ge 16\text{px}$ in `order_flow.svg`.
- **M3 (T-Bus Trunk Deduplication & Disambiguation)**:
  - Turn `test_r2_bus_01..10` 100% green (resolving the 5 baseline bus deduplication failures).
  - Zero duplicate vertical stems or collinear bus paths in SVG.
- **M4 (Arc Hop Bridges & Label Collision Avoidance)**:
  - Turn `test_r3_cross_02..09` and `test_r4_shield_01..10` 100% green (resolving arc hop & pill shield collisions).
  - Zero corner fillet obscuration by pill shields in `order_flow.svg`.
- **M5 (Final Acceptance)**:
  - **40/40 tests in `e2e_advanced_routing_tests.rs` PASSING (100%)**.
  - **205/205 tests in `e2e_branch_expansion_tests.rs` PASSING (100%)**.
  - Canvas height for `order_flow.svg` $\le 3300\text{px}$.
  - Zero clippy warnings under `--all-targets --all-features`.
  - Clean `cargo fmt --check`.
  - Render latency budget $< 2\text{ms}$.
