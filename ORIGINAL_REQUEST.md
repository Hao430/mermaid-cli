# Original User Request

## Initial Request — 2026-10-22T07:43:29Z

# Teamwork Project Prompt

> Status: Launched
> Goal: Craft prompt → get user approval → delegate to teamwork_preview
> Requested team: Full Team

The user requested: "使用完整团队（Full Team）开展多模块协作开发与测试验收".

Implement horizontal parallel branch expansion (同层分叉节点的横向并行展开能力) in the Rust `mermaid-cli` diagram engine, so that branching decision paths and parallel workflows render side-by-side in distinct columns/swimlanes, compressing diagram height by 30%–40% without center collapse.

Working directory: /home/user/projects/mermaid-cli
Integrity mode: development

Reference plan: docs/PLAN_HORIZONTAL_BRANCH_EXPANSION.md

## Requirements

### R1. Fork Detection and Lane Slot Propagation
Analyze the diagram's forward DAG to identify fork nodes (out-degree >= 2). Assign each outgoing branch a distinct horizontal lane slot offset (e.g. left = -1.0, right = +1.0) and recursively propagate this slot to downstream descendants until reaching a join node (in-degree >= 2), where the offset converges back to 0.

### R2. Subgraph-Scoped Column Grid and Node Positioning
Calculate node horizontal positions based on their assigned lane slots within their containing subgraph rather than collapsing every single-node row back to the global canvas center. Subgraphs must dynamically adjust their width to accommodate their internal parallel lanes, eliminating blank void gaps on unbalanced branches.

### R3. Orthogonal Bus Routing for Forks and Joins
Ensure the rounded orthogonal edge router (`src/renderer/routing.rs`) neatly connects fork outputs to their respective parallel lanes and joins converging parallel lanes back together with clean horizontal distribution/collection bus lines and fillet curves.

## Acceptance Criteria

### Correctness & Layout Quality
- [ ] For `examples/order_flow.mmd`, the parallel branches in G3 (在线支付 vs 货到付款) and G5 (退货 vs 换货) render side-by-side in distinct horizontal columns.
- [ ] Diagram canvas height for `examples/order_flow.svg` is compressed significantly compared to the baseline (target <= 3300px, down from 4785px).
- [ ] Subgraphs dynamically fit their internal parallel columns without overlapping adjacent subgraphs.
- [ ] All 90-degree edge bends maintain smooth fillet corner curves with pill shield text labels.

### Regression & Code Quality
- [ ] All existing 330+ unit, integration, and snapshot tests pass cleanly via `cargo test --all-features`.
- [ ] Zero compiler warnings from `cargo clippy --all-targets --all-features`.
- [ ] Code is properly formatted according to `cargo fmt --check`.
