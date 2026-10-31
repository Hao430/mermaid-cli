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


## Follow-up — 2026-10-23T01:14:23Z

# Teamwork Project Prompt

> Status: Launched
> Goal: Craft prompt → get user approval → delegate to teamwork_preview
> Requested team: Full Team

使用完整团队（Full Team）开展多模块协作开发与测试验收。

针对纯 Rust `mermaid-cli` 图表引擎，全面解决流程图中的连线重合、交叉干扰、横向通道冲突与文本标签遮挡问题，构建工业级的正交通道布线（Orthogonal Channel Routing）、拓扑重心消交与智能标签避让体系。

Working directory: /home/user/projects/mermaid-cli
Integrity mode: development

## Requirements

### R1. 多级回边与跨层旁路通道独立分轨 (Gutter & Bypass Channel Assignment)
分析流程图前向与后向拓扑，为多个并发反馈循环边（Loop / Feedback Edges）以及跨层级直连旁路边（Bypass Edges）动态分配互不重叠的外侧走线通道（Gutter Channels）与列间隙走廊（Corridors），彻底消除硬编码固定偏移量导致的垂直连线共线重叠。

### R2. T 型总线路径去重与多分支清晰分轨 (Bus Trunk Deduplication & Multi-Branch Disambiguation)
优化分叉（Fork）与汇聚（Join）处的总线几何生成机制。对于多分支共享的垂直发车主干、汇集落干与水平总线段，消除在 SVG 中重复绘制相同重叠 `<path>` 的现象，并确保向同侧延展的多个分支在总线转折处具有清晰可辨的分轨间距。

### R3. 层内节点重心交叉最小化与正交跳线拱桥 (Crossing Minimization & Arc Bridges)
结合横向子图网格布局，引入基于父子节点重心或中位数（Barycenter / Median Heuristic）的层内节点次序微调，最大化减少连线几何交叉；对于拓扑上不可避免的正交相交连线，支持生成半圆跳线拱桥（Arc Hop / Bridge）或透明遮罩间隙，彻底消除十字交叉歧义。

### R4. 边文本胶囊碰撞避让与智能锚定 (Label & Shield Collision Avoidance)
升级连线标签与保护胶囊（Pill Shield `<rect rx="4">`）的放置决策算法。在保证放置于连线最平稳开阔段的前提下，引入标签间互斥与拐角避让碰撞检测，防止文字遮罩互相压盖、覆盖分叉贝塞尔圆角或遮挡节点文字。

## Acceptance Criteria

### Correctness & Visual Quality
- [ ] 针对 `examples/order_flow.mmd` 及多循环图表，所有反馈循环边（如 `C10 -> C4` 与 `C7 -> C3`）分配为阶梯状外侧走线通道，垂直线段零重合（$\Delta X \ge 16\text{px}$）。
- [ ] 共享分叉与合流总线处不存在重复叠加绘制的共线 SVG 路径段，同向展开的平行连线拐角清晰无歧义。
- [ ] 正交相交连线呈现清晰的半圆跳线拱桥（Arc Hop）或视差跳跃，图表总连线交叉数显著低于基准。
- [ ] 所有连线文字及其底色胶囊遮罩（Pill Shields）在全局范围内实现两两无碰撞重叠，且不遮挡 90 度圆角弯头。
- [ ] 保持全图画布紧凑，`examples/order_flow.svg` 总高度维持在 $\le 3300\text{px}$ 目标范围内。

### Zero Regression & Performance
- [ ] 现有全部 218 个单元测试、205 个 E2E 测试及 57 个压力/对抗性测试 100% 保持通过或完成断言兼容更新。
- [ ] `cargo clippy --all-targets --all-features` 保持 0 警告、0 错误。
- [ ] 代码格式通过 `cargo fmt --check` 严格校验。
- [ ] 布线通道分配与标签避让增量耗时控制在性能预算（< 2ms）之内。
