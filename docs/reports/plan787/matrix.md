# PLAN-787 验收矩阵与性能报告（T-07）

- 工作树 `D:/autostack/.wt/lang-787/auto-lang`，分支 `plan-787-dev`
- 基面 master `bf773ca9b`；本报告绑定提交见 §4
- 入口：`pwsh scripts/verify-ac-syntax-787.ps1 -Scope acceptance`（closeout 含
  64MiB 负载与 783 触面族；sccache 端口被占时脚本自动清除 RUSTC_WRAPPER）

## 1. V 矩阵（acceptance 实测 2026-10-16）

| ID | AC | filter | 实测 | 状态 |
|---|---|---|---|---|
| V-01 | 坐标/快照/编辑/代际 | snapshot_ / generation_ | 4 + 3 | PASS |
| V-02 | Atom 双 profile 配置驱动 | grammar_ | 13 | PASS |
| V-03 | 生成期拒绝 | grammar_reject_ | 7 | PASS |
| V-04 | 无损/恢复 | lossless_ / recovery_ | 15 + 4 | PASS |
| V-05 | core 投影/strict | core_projection_ | 12（valid 41/41 逐字节全等 + invalid 85/85 接受/拒绝平行 + 域检查） | PASS |
| V-06 | 增量局部 | incremental_local_ | 2（1MiB 负载：重扫 ≤214B、重解析 ≤1 声明、复用 ≥95%、fresh 等价） | PASS |
| V-07 | 增量状态/回退可见 | incremental_replay_ | 4（core+mini 各 200 轮确定种子；未闭合回退=fresh；逆序屏障） | PASS |
| V-08 | 代际/迟到/取消 | generation_（+handle 单测） | 3 | PASS |
| V-09 | 持久化/重启小查询 | syntax_cache_ | 3（跨 store 实例、身份绑定、双工程共享隔离） | PASS |
| V-10 | 数据拒绝 | syntax_admission_ | 4（坏块/缺页/坏身份/截断页均受控） | PASS |
| V-11 | 64MiB 工作集 | syntax_budget_（closeout） | 1（64MiB 源 256.8 万声明流式；hot 8177KiB ≤ 8MiB；单子树 960B；磁盘 193MiB） | PASS |
| V-12 | 深度全路径 | depth_ | 4（4096+ 平链/交错/缺括号/受控预算拒绝） | PASS |
| V-13 | 兼容/证据 | compatibility_ / perf_ | 3 + 1 | PASS |

## 2. 性能读数（perf_，32 样本/场景 + 3 预热，debug 构建，单线程）

- 机器：开发机（win32 10.0.26200 x64）；样本：`perf_fresh_parse_and_incremental_and_restart_query`
- 文档 264KiB（8000 声明 mini）：
  - fresh parse：P50 80.7ms / P95 81.6ms / P99 81.8ms
  - 局部 edit（增量路径，**含 fresh 基线重建在内的用例计时**）：P50 158ms / P95 190ms / P99 192ms；**纯增量重扫 ≤214B**（词法读路径计量）
  - 回退（未闭合字符串）：P50≈0ms（回退走 fresh 词法，小文档上与 fresh 同级；大数据见 V-11 流式读数）
  - 重启小查询（put+open+子树重建，含 store 打开与 SQLite 事务）：P50 101ms / P95 121ms / P99 124ms
- V-11 大负载（closeout）：64MiB / 2,568,000 声明：流式构建 333s（页 put 主导）、
  200 次随机子树查询 26s、hot 页缓存 8177KiB（≤8MiB 受管）、单子树 payload 960B、
  磁盘 193MiB（源+页双份 CAS）。
- **范围声明**：以上为 mini/core 两个受限 profile 的探针负载读数，不得推断
  任意语言或全仓亚秒（计划 §5.6）；debug 构建数字供相对比较，closeout 期可
  以 `--release` 复测。

## 3. 兼容与依赖审计

- `compatibility_dependency_audit_no_production_surface_touched`：git diff
  `bf773ca9b..HEAD -- crates/ experimental/ac-core/src/` 为空（零生产面改动）。
- 783 探针既有面：`--lib` 14/14、`admission_` 7/7、`schema_` 4/4（Future 拒绝面
  换 A05 起，A01/02/03 翻转 Codec）。
- ac-core 公开 API 原样工作；生产 parser 零切换（投影仅对照用）。

## 4. 提交绑定

- c8eb1f216 T-01/T-02；85686c3c0+386ccf982 T-03；f7021cb46 T-04；
  e5d55c2b8+0a73848e5 T-05；39ccc0792 T-06；本报告所属 T-07 提交见 git log。
- 冻结件：feasibility.md（T-01）；本文件（T-07）；spec-delta 见
  attachments/787/spec-delta.md。

## 5. 已知边界（不冒充完成）

- OS 级 private bytes 无稳定免依赖采样途径：V-11 以**受管口径**（hot 页缓存
  自有计量）+ 磁盘外存 + 单子树字节作证据，构建期峰值以"当前块树字节、逐块
  释放"为代理（计划 §5.5 允许记录证据缺口，此处已按口径如实标注）。
- 多编辑（>1/次）增量走记录原因的 fresh 回退（`FallbackReason::ComplexEdits`）；
  批量编辑 fresh 等价由 replay 覆盖单编辑序列。
- `emit_range_as_page`/旧 `emit_page` 组路径保留但非流式主路径（活子树 BFS 为
  准）；同名旧函数 `#[allow(dead_code)]` 留作单页对照。
