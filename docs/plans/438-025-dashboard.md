# Plan 438: 025-dashboard 系统监视器（App 轨道填洞 ②）

> **状态**: 🟡 M1 + M1-fix 已完成（2026-08-26，分支 plan-438，worktree .worktree/plan-438）；M2 待做（消费 437 Phase 2 组件化，437 复审确认其未做）。M1 执行记录见 §7，两处 vue 生成器缺口根治见 §8。
> **来源**: [Design 21 §5](../design/21-examples-app-track.md) 填洞路线第 2 项。
> **关联**: [Plan 437](437-024-charts.md)（chart 组件上游，弱依赖）、012-stopwatch（.Tick 先例）、Plan 386（golden）、姊妹计划 437/439–441
> **目录**: `examples/ui/025-dashboard/`（编号 025 原为 notes-extended，2026-08-23 已删除、能力并入 015-notes）｜pac `name: "dashboard"`｜端口 4025

## 1. 目标与平台缺口

**系统监视器**形态的 dashboard（AutoOS 默认应用直系）：KPI 行 + CPU/内存/网络实时曲线 + 进程表。钉住：

- **组合布局**：多图表 + 表格 + KPI 卡的响应式排布（侧栏折叠、容器伸缩）。
- **轮询刷新**：`.Tick` 节流轮询 + 数据窗口管理（与 437 的流式模式互补：437 钉单图交互，本计划钉**多数据源并发刷新**）。
- **DataTable 基础用法**（排序点击、行 hover）——深水区（虚拟滚动/列宽拖拽）归 439，本计划只用现有能力。

**demo 边界**（Design 21 §5/§6）：数据源用**前端 mock 生成器**（随机游走 CPU/内存曲线 + 固定进程清单），但消费它的 API 形状按"将来真后端 system API"设计——换真数据源时前端零改动。

## 2. 现状盘点

- vue 端 chart widgets 可用（charts-gallery/registry 已钉）；DataTable 全家桶已注册（registry.rs:2789 起）。
- `.Tick` 机制已有；018 的 storage 内置（Plan 401 018）可持久化面板配置。

## 3. Phase 划分

### M1 — vue 端完整应用（无 crates 改动，可与 437 并行）
- 三区布局：KPI 卡行（4 张：CPU/内存/网络/进程数）、曲线区（3 张 AreaChart，独立开关）、进程表（DataTable：名称/CPU%/内存/状态，可排序）。
- mock 数据服务（widget 内 `.Tick` + 随机游走函数）+ 刷新间隔调节 + 暂停。
- 暗色模式对齐（Design 19 theming）。
- `tests/desktop_mcp.py`：KPI 值随 Tick 变化断言、排序点击断言、曲线开关断言。

### M2 — vm 模式（消费 437 Phase 2 的 vm 图表组件）
- vm 端与 vue 端布局/数据一致（038 双后端先例）；437 Phase 2 未就绪时本阶段挂起，M1 不受影响。
- 面板配置持久化（storage，018 先例）。

## 4. 验收（DoD）

- [x] M1：vue 构建 + vue-tsc 绿 ✓（2026-08-26：`auto build`（strict 再生 + vue-tsc + vite build）全绿；浏览器实机断言 6/6——KPI 随 Tick 变化 / 内存标签 GB·MB 格式 / 排序点击升降翻转 / 三曲线独立开关 / 暂停冻结+恢复 / 图表 path 随数据重算。desktop_mcp 属 VM 轨，随 M2 交付）。
- [ ] M2：vm 实机可跑，三曲线 + 进程表 + 配置持久化可用。
- [ ] mock→真数据源替换演练：接口形状文档化（SPEC.md 内一节）。

## 5. 多 agent 并发边界

- **拥有**：`examples/ui/025-dashboard/**`。**不改 crates**——纯 app 层计划，并发安全度最高。
- **消费**：437 Phase 2 的 vm 图表组件（M2 前置；M1 用 vue 现有映射零等待，437 Phase 1 契约正式化合入后跟进切换）。
- **让渡**：DataTable 能力扩展（虚拟滚动/分页协议/列宽）归 439-database 拥有，本计划只消费现状并提需求单。

## 6. 风险

| 风险 | 缓解 |
|---|---|
| 多 Tick 并发刷新的渲染抖动 | 统一单 Tick 源分发（一个 interval 广播，避免每图一个 timer） |
| 与 437 抢 chart 交互语义 | tooltip/legend 行为以 437 M1 结论为准，本计划不另行定义 |

## 7. M1 执行记录（2026-08-26，分支 plan-438）

### 落地内容

- `examples/ui/025-dashboard/`：pac.at（render vue，端口 4025）+ `src/front/app.at` + SPEC.md（含 mock→真数据源接口形状一节，DoD 第三项）+ gen 树。
- **三区布局**：KPI 卡行 ×4（CPU/内存/网络/进程数，语义 token 暗色对齐）→ 三张面积图卡（CPU 0-100% / 内存 0-30GB / 网络 0-8MB/s，各自 checkbox 独立开关，SVG 直通 + 静态网格 + 左侧刻度列）→ 进程表（table 原语族 + badge 状态列；名称/CPU/内存三列点击排序，列头 ↑↓ 指示）。
- **单 Tick 源分发**（§6 风险缓解落实）：`.Tick`（250ms 基准）+ `subTick >= speedDiv` 分频 → 有效间隔 250ms/1s/2.5s 三档；`running` str 门控（Init 置 "true" 触发 watch 启动——生成的 watch 无 immediate）。
- **mock 随机游走**：确定性算术漂移 + clamp（024 惯例，vue 轨无 rand 映射）；进程值按索引相位漂移；滑窗 30 点 Init 预填（步长恒定）。
- **排序**：选择式重建（扫描剩余取最优，无索引写）；name 列与值漂移无关 → Tick 内跳过重排。

### 实现中发现并绕开的 vue 轨生成器缺口（SPEC.md「已知边界」记账）

1. **f-string 模型引用直插**：`f"${.cpu}"` 发出 `` `${cpu}` `` 而非 `` `${cpu.value}` ``（vue-tsc TS2362）→ 规避：先提升局部变量。
2. **any/int 除法的 Math.trunc 发射不稳定**：record 字段（any）参与 `/`/`%` 不截断（12.109375 直出）；局部 int 的截断行为曾在同源代码不同构建间翻转 → 规避：数值显示一律"十分位整数存储 + 单次 /10"（JS 精确浮点）。
   两项均为 ui_gen/vue 侧潜在改进项，M1 纯 app 层不碰 crates（plan §5 约束），留待立项。

### 解析器踩坑（对后续示例有用）

- `fn` 是关键字——局部变量名撞上会引发级联解析错误（"Expected term, got RBrace" 指向文件尾）。
- view 层 for 循环变量字段访问须用**点访问**（`p.status`）；索引访问（`p["status"]`）出现在 view 条件/属性中会解析失败（015-sidebar 的点访问为既定形态）。

### 验证（全部通过）

- `auto build`（master exe）：strict 再生零 S002 + vue-tsc 零错 + vite build 绿（151.8 kB js）。
- 浏览器实机（IAB + dom_cua；playwright locator click 在高频重渲染页面上不可用，坐标/节点路径点击正常）：KPI 随 Tick 变化（23%→35%、3.4→1.9 MB/s）、内存标签正确（"18.6 GB"/"580 MB"）、排序点击（↓→↑、首行 chrome 37%→mcp-hub 5%、列值升序）、曲线独立开关（CPU 卡隐藏而他卡存活）、暂停冻结（2.6s 值不变）+ 恢复、动态 path 随数据重算。

### 未竟（M2 范畴）

- vm 模式（消费 437 Phase 2 vm 图表组件——437 复审 2026-08-26 确认该组件化未做）；面板配置持久化（storage，018 先例）；`tests/desktop_mcp.py`（013 惯例，VM 轨）。

## 8. M1-fix：vue 轨生成器两缺口根治（2026-08-26，同分支追加）

> M1 落地时绕开的两个 `ui_gen` 缺口（原 SPEC「已知边界」①②），经用户
> 裁定升级为本计划 phase 直接修复（打破"纯 app 层"约束的一次性破例，
> 修复面收敛在 ts_adapter 单文件）。

### 缺陷与根因

| # | 缺陷 | 根因（代码级） |
|---|---|---|
| F-1 | handler 内 `f"${.cpu}"` 发出 `` `${cpu}` `` 而非 `` `${cpu.value}` ``（vue-tsc TS2362；运行时是 Ref 对象） | `ts_adapter::transpile_expr` **没有 `Expr::FStr` 臂**——整个表达式兜底委托 a2ts 打印器（`trans/ts_expr.rs fstr()`），后者不认识 Vue ref，模型引用印成裸名 |
| F-2 | 局部变量整除语义缺失：`var m = .intState` 后 `m / 10` 无 `Math.trunc`（与 VM `DIV`=wrapping_div 整除**跨后端语义分歧**）；用户写 `var g int = …` 标注也不被识别 | `expr_proven_int` 只认 int 字面量与 int 声明的 state/prop（`typed_ints` 表）——**handler 局部变量完全不在类型表里**（澄清：M1 时怀疑的"构建间翻转"系误读，实际是首帧显示了 model 初始值，截断从未发生过） |

### 修复（crates/auto-lang/src/ui_gen/ts_adapter.rs，+4 处）

1. **F-1**：`transpile_expr` 补 `Expr::FStr` 臂——反引号模板字面量，插值
   部分递归走 `transpile_expr`（AURA 感知，`.x` → `x.value`）；字面量
   部分沿用 a2ts 转义规则（`` ` `` 与 `${`）。
2. **F-2**：`AuraTsContext` 增 `int_locals: RefCell<HashSet>`（与既有
   `null_init_locals` 同款扁平 per-handler 模型）；`Stmt::Store` 声明时
   注册（显式 int 族类型标注 **或** `expr_proven_int(initializer)` 双路
   径）；`expr_proven_int` 的 `Ident` 臂并入 `is_local_int` 查询；
   `Op::Asn` 臂对"int 局部 ← 非证整右值"的重赋值做**失效摘除**。
3. 语义边界（有意保守，保持 Plan 014 规则）：record 字段（any）与动态
   调用结果不注册——对实际浮点值截断是静默正确性 bug；复合赋值
   （`+=` 等）不追踪失效（正字法上 int 局部不做混合浮点复合赋值）。

### 回归测试（vue_capabilities.rs，4 例）

- `cap_438_fstr_model_ref_unwraps_value`：`f"cpu=${.cpu} pct"` →
  `` label.value = `cpu=${cpu.value} pct` ``（F-1 主断言）
- `cap_438_fstr_escapes_literal_parts`：字面量 `` ` ``/`${` 转义、局部
  插值保持裸名
- `cap_438_local_int_division_trunc`：`var fromState = .memT` 与
  `var g int = 100` 两条路径的 `/` 均降 `Math.trunc`
- `cap_438_local_int_tracking_invalidation`：int 局部被 record 字段
  （any）重赋值后失去 trunc（失效路径）

### 验证（全部绿）

- `--test vue_capabilities` **76/76**（72 既有 + 4 新增，零回归）；
- **gallery golden 零漂移**（1/1 过）——修复只影响此前坏掉的模式，
  gallery 全用局部变量习语故不变；
- schema_drift 1/1、docs_gen 4/4、`--lib` **3211/0**、auto-man **229+6**；
- 025-dashboard 端到端：worktree exe 重生成后 `auto build`（含 vue-tsc）
  全绿——Tick 标签已改回**直插形态** `f"${.cpu} %"`（正是修复前 TS2362
  报错点，现类型检查通过）；生成物直查 `` cpuLabel.value = `${cpu.value} %` ``
  与 `` memLabel.value = `${mTf / 10} / 32 GB` ``（float 局部量，无 trunc）。
  浏览器实机复核因 IAB webview 会话不可用未做，以 单测+生成物+类型检查
  三层证据替代（M1 首轮实机 6/6 已覆盖交互面）。

### 确立的数值显示正字法（SPEC 同步修订）

int `/` 是整除（VM/vue 同语义）——**小数显示必须走 float 局部量**
（`var x float = …` + `/ 10.0`，437 §0.6.D 纪律的 vue 侧对偶）。
M1 时的"十分位存储 + any 浮除"是修复前的侥幸路径，已在 SPEC 标注
不得再依赖。
