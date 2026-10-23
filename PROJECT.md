# Project: Horizontal Parallel Branch Expansion in mermaid-cli

## Architecture
Horizontal Parallel Branch Expansion enhances the pure Rust `mermaid-cli` diagram engine to render branching decision paths and parallel workflows side-by-side in distinct horizontal columns, compressing diagram height by 30%–40% without center collapse.

The architecture comprises three core modules plus an integration layer:
1. **Branch Analysis (`src/renderer/layout/branch.rs`)**: Analyzes the forward DAG (after DFS cycle elimination) to identify fork nodes ($d_{out} \ge 2$), assigns symmetric horizontal lane slot offsets (e.g. $\pm 0.5$ or $-1, 0, +1$), monotonically propagates slots downstream along single-input paths, converges at join nodes ($d_{in} \ge 2$), and reserves virtual through-corridors (`LaneReservation`) for short unbalanced branches across the depth span $D_{max}$.
2. **Column Grid & Node Positioning (`src/renderer/layout/grid.rs`)**: Computes subgraph dynamic widths ($W_{sg} = M \cdot W_{node} + (M-1)S_h + 2P_{sg}$) based on active lane count $M$, computes node X coordinates relative to subgraph centers bound to lane slots, eliminates single-node center collapse, compacts parallel branch layers, and adjusts subgraph bounding boxes.
3. **Orthogonal Bus Routing (`src/renderer/routing.rs`)**: Implements T-bus distribution routing for forks and collection routing for joins at unified bus elevations ($Y_{fork\_bus}$ and $Y_{join\_bus}$), with quadratic Bézier fillet corner curves ($r = 8\text{px}$), pill shield protected text labels, and outer margin gutter (+28px) routing for loop/retry edges.
4. **Renderer Integration (`src/renderer/mod.rs`)**: Binds branch analysis, column grids, and edge bus routing into `render_flowchart` while preserving strict zero-regression for all 24 other diagram types.

```
       [AST / Diagram]
              │
              ▼
    [DFS Cycle Detection] ─── isolates back-edges ───► [Gutter Routing]
              │
              ▼ (Forward DAG)
   [BranchAnalyzer (M1)]
   ├── Fork Detection (out_deg >= 2)
   ├── Lane Slot Assignment (slot_offset = i - (k-1)/2)
   ├── Downstream Propagation & Join Convergence
   └── Lane Reservations (D_max corridors)
              │
              ▼
   [Column Grid & Positioning (M2)]
   ├── Subgraph Dynamic Width (W_sg = M*W + (M-1)*S + 2*P)
   ├── Subgraph-Scoped Node X Positioning (No Center Collapse)
   ├── Parallel Branch Layer Compaction (Height <= 3300px)
   └── Subgraph Bounding Box Auto-Fitting
              │
              ▼
   [Orthogonal Bus Routing (M3)]
   ├── T-Shaped Fork Distribution Bus
   ├── T-Shaped Join Collection Bus
   ├── Quadratic Bézier Fillets (r = 8px)
   └── Pill Shield Protected Text Labels (<rect rx="4">)
              │
              ▼
        [SVG Output]
```

---

## Feature Inventory
Every feature from the Survey phase appears here with its assigned milestone:

| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | DFS Back-Edge Detection | DFS recursion stack detects cycles and isolates back-edges, constructing forward DAG | M1 | Survey (PLAN §2.1) |
| 2 | Fork Node Identification | Identifies nodes where forward out-degree $\ge 2$ | M1 | Survey (PLAN §2.1) |
| 3 | Lane Slot Index Assignment | Symmetric lane slot offset assignment: $\text{slot}_i = i - \frac{k-1}{2}$ | M1 | Survey (PLAN §2.1) |
| 4 | Downstream Slot Propagation | Propagates slot offsets recursively down forward single-input descendant paths | M1 | Survey (PLAN §2.1) |
| 5 | Join Termination & Convergence | Halts propagation at forward in-degree $\ge 2$ and resets join node slot to 0.0 | M1 | Survey (PLAN §2.1) |
| 6 | Virtual Lane Reservation | Reserves column corridors across depth span $D_{max}$ for unbalanced short branches | M1 | Survey (PLAN §2.2) |
| 7 | Subgraph Dynamic Width | Calculates $W_{sg} = M \cdot W_{node} + (M-1)S_h + 2P_{sg}$ to fit internal lanes | M2 | Survey (PLAN §2.3) |
| 8 | Subgraph-Scoped Node X-Positioning | Computes $X_{node} = X_{sg\_center} + \text{slot} \cdot (W+S_h) - W/2$, eliminating center collapse | M2 | Survey (PLAN §2.3) |
| 9 | Branch Layer Compaction | Compacts parallel branch nodes into shared vertical layer rows, compressing height $\le 3300\text{px}$ | M2 | Survey (PLAN §1.2) |
| 10 | Subgraph Bounding Box Auto-Fitting | Auto-fits subgraph outline `<rect>` and title around internal nodes and lane margins | M2 | Survey (PLAN §2.3) |
| 11 | Phase-Level Subgraph Vertical Stacking | Stacks sequential phase subgraphs vertically in topological order without collision | M2 | Survey (PLAN §2.3) |
| 12 | T-Shaped Fork Distribution Bus | Routes fork outputs to parallel columns via intermediate horizontal distribution bus | M3 | Survey (PLAN §3) |
| 13 | T-Shaped Join Collection Bus | Gathers parallel branch columns into horizontal collection bus before entering join trunk | M3 | Survey (PLAN §3) |
| 14 | Quadratic Bézier Corner Fillets | Smooths all 90-degree orthogonal turns with radius $r=8\text{px}$ quadratic curves | M3 | Survey (PLAN §3) |
| 15 | Pill Shield Protected Labels | Positions labels along longest segment with `<rect rx="4">` protective underlay | M3 | Survey (PLAN §3) |
| 16 | Outer Gutter Routing for Loop Edges | Routes backward loop edges via outer margin gutter (+28px) | M3 | Survey (PLAN §3) |
| 17 | Diagram Engine Zero-Regression | Preserves all existing 330+ tests and 24 other diagram types without regression | M4 | Survey (PLAN §5.1) |
| 18 | Sub-2ms Performance Budget | Ensures branch analysis, grid mapping, and bus routing add $< 2\text{ms}$ latency | M4 | Survey (PLAN §5.2) |

---

## Milestones

| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | M1: Forward DAG Analysis & Lane Slot Propagation (R1) | `src/renderer/layout/branch.rs`, `src/renderer/layout/mod.rs`: DFS cycle elimination, fork detection, symmetric lane assignment, downstream propagation, join convergence, virtual lane reservations | none | DONE |
| 2 | M2: Subgraph-Scoped Column Grid & Node Positioning (R2) | `src/renderer/layout/grid.rs`, `src/renderer/mod.rs`: Subgraph dynamic width calculation, subgraph-scoped node X positioning eliminating center collapse, branch layer compaction (height $\le 3300\text{px}$), bounding box auto-fitting | M1 | DONE |
| 3 | M3: Orthogonal Bus Routing for Forks, Joins, Fillets & Shields (R3) | `src/renderer/routing.rs`, `src/renderer/mod.rs`: T-bus distribution for forks, T-bus collection for joins, quadratic Bézier fillets ($r=8\text{px}$), pill shield protected labels, gutter routing | M1, M2 | DONE |
| 4 | M4: Final Verification, 100% E2E Pass & Adversarial Hardening | Phase 1: Pass 100% E2E tests (Tiers 1-4) produced by E2E Testing Track; verify `examples/order_flow.svg` height $\le 3300\text{px}$, G3/G5 branch separation, zero regressions, zero clippy warnings. Phase 2: Tier 5 adversarial coverage hardening. | M1, M2, M3, TEST_READY.md | DONE |

In parallel:
- **E2E Testing Track**: Independent requirement-driven test suite creation (Tiers 1-4) published via `TEST_READY.md`.

---

## Interface Contracts

### 1. `layout/branch` ↔ `layout/grid`
```rust
pub struct BranchLane {
    pub branch_id: usize,
    pub fork_node: String,
    pub slot_offset: f32, // -1.0, -0.5, 0.0, +0.5, +1.0
    pub nodes: HashSet<String>,
    pub depth_span: (usize, usize),
}

pub struct LaneReservation {
    pub branch_id: usize,
    pub slot_offset: f32,
    pub layer: usize,
}

pub struct BranchAnalyzer {
    pub lanes: Vec<BranchLane>,
    pub node_to_lane: HashMap<String, usize>,
    pub node_to_slot: HashMap<String, f32>,
    pub join_nodes: HashSet<String>,
    pub fork_nodes: HashSet<String>,
    pub reservations: Vec<LaneReservation>,
}

impl BranchAnalyzer {
    pub fn analyze(
        node_ids: &[String],
        edges: &[(String, String)],
        layers: &[Vec<String>],
        subgraphs: &[Subgraph],
    ) -> Self;

    pub fn slot_for_node(&self, node_id: &str) -> f32;
    pub fn is_fork(&self, node_id: &str) -> bool;
    pub fn is_join(&self, node_id: &str) -> bool;
}
```

### 2. `layout/grid` ↔ `renderer/mod.rs`
```rust
pub struct SubgraphColumnGrid {
    pub subgraph_id: String,
    pub active_lanes: usize,
    pub width: f32,
    pub center_x: f32,
}

pub struct LayoutGrid {
    pub node_positions: HashMap<String, (f32, f32)>,
    pub subgraph_bounds: HashMap<String, (f32, f32, f32, f32)>, // (x, y, w, h)
    pub effective_width: f32,
    pub effective_height: f32,
}

impl LayoutGrid {
    pub fn compute(
        layers: &[Vec<String>],
        subgraphs: &[Subgraph],
        branch_analyzer: &BranchAnalyzer,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Self;
}
```

### 3. `layout/branch` & `layout/grid` ↔ `renderer/routing.rs`
```rust
#[derive(Debug, Clone, Default)]
pub struct EdgeBusOptions {
    pub fork_bus_y: Option<f32>,
    pub join_bus_y: Option<f32>,
    pub bypass_lane_x: Option<f32>,
}

pub fn route_orthogonal_edge_with_bus(
    from_box: (f32, f32, f32, f32),
    to_box: (f32, f32, f32, f32),
    fillet_radius: f32,
    bus_options: Option<&EdgeBusOptions>,
) -> RoutedEdge;
```

---

## Code Layout

- `src/renderer/layout/mod.rs`: Module declaration and re-exports.
- `src/renderer/layout/branch.rs`: Forward DAG analysis, fork/join detection, symmetric lane slot assignment, monotonic downstream propagation, virtual lane reservations.
- `src/renderer/layout/grid.rs`: Subgraph-scoped column grids, dynamic width calculations, node positioning without center collapse, parallel layer compaction.
- `src/renderer/routing.rs`: Orthogonal routing, T-bus distribution/collection routing, quadratic fillet generation, pill shield labels, outer gutter routing.
- `src/renderer/mod.rs`: Flowchart layout integration, calling `BranchAnalyzer`, `LayoutGrid`, and `route_orthogonal_edge_with_bus`.
- `tests/`: Unit, integration, snapshot, and E2E test suites.
