# Plan 427: a2r 字符串形参借用回归修复（DIV-A2R-STRPARAM-1）

> **状态**: 🟢 立项待执行（2026-08-23）
> **来源**: 417-final 收官批（2026-08-22）全量仪表盘重跑时发现；known-divergences.md 头部警示 + KNOWN-DEBT 已登记。
> **仓库**: **auto-lang**（trans/rust.rs 为主）。
> **影响**: base64 / url / serde_json 三库 a2r 编译级失败（三向 0-64%），网站 L1 声明被迫从 241 例收缩到 141 例。

---

## 0. 背景与现状

**症状**（serde_json `parse` 测试为例，生成代码 E0308）：

```rust
fn check_str(n: i64, name: &str, actual: &str, expected: &str) { ... }
fn check_parse_ok(n: i64, name: &str, input: &str, expected: &str) {
    let actual: String = ok_value(parse(input));
    check_str(n, name, actual, expected);   // ← String 直传 &str 形参 → E0308
}
```

交付时点（359 D1，2026-08-20 前）三库 a2r 全绿：`check_str(..., actual, ...)`
会自动发射 `actual.as_str()`。该自动借用于 8-20 ~ 8-22 某个并行批次中失效。

**关键事实**：

- 回归**早于** 417-E3 批次：已在 `2ac2298f`（2026-08-22 前置点）复现
  （tmp worktree 构建 + `auto-parity run serde_json` = 0/58）。
- 交付时点回归**不存在**（359 D1 验收 56/56）。引入区间：
  359-D1 验收提交 … `2ac2298f` 之间的并行批次（候选：E2 关联类型批、
  417-D2 导入签名批、u32 字符串池批 8482021e、auto-shell/418 批）。
- a2r golden（348 例）**全绿**——golden 只做转译文本对比不编译，编译级回归
  对 golden 不可见。这是本回归能潜伏数日的原因。
- 当前三库状态：serde_json 0/58、url 0/32、base64 22/34。

## 1. 任务

### Task 1: bisect 定位引入提交（半天）

- [ ] 在 `git log` 区间内二分：每个候选点建临时 worktree，构建 auto +
  `auto-parity run serde_json`（以 0/58 vs 56/58 为判据）。
- [ ] 候选优先级：`8482021e`（字符串池 u32 化）、417-D2 的
  `register_import_signatures` 批、E2 的 rust.rs 批（c7001f83/48b49be0）。
- [ ] 输出：引入提交 hash + 失效机制一句话结论（登记到本节）。

### Task 2: 根因修复（1 天内）

- [ ] 定位 `trans/rust.rs` 中 `&str` 形参调用点自动借用的判定链
  （`fn_str_param_indices` 注册于 fn_decl 发射 ~:11188，消费于调用编译
  ~:7129/7269+；另有 417-D2 的 `register_import_signatures` 注册路径
  ~:12294/:18137）。判定为何对本例失效——嫌疑方向：
  ①局部 fn 的注册时机 vs 调用点发射顺序（同文件先定义后调用本应覆盖）；
  ②`local_var_types` 推断 `actual` 为 String 的路径变了；
  ③as_str 插入条件被某批收紧。
- [ ] 修复 + 单测：`crates/auto-lang/test/a2r/` 新增 golden（`&str` 形参 +
  String 实参 + `.as_str()` 断言）；golden 流程照旧（wrong.rs 独立
  cargo build 验证后提升）。
- [ ] **编译级回归防线**：a2r golden 增加一个"产物必须 cargo build 通过"
  的冒烟（最小 crate，链 a2r-std；或复用 parity harness 的 build 路径），
  防止同类回归再潜伏（可选，评估成本后决定）。

### Task 3: parity 三库恢复（半天）

- [ ] `auto-parity run serde_json / url / base64` 三库三向恢复全绿
  （56/56、30/30、33/33 或当前用例数）；全量 `report` 重生成仪表盘
  （**必须 `AUTO_BINARY=<新构建>`**，默认 PATH 旧 binary 会全 0——
  417-final 踩坑两次）。
- [ ] http_client_sync 的 mock-server 首跑竞态注意复跑确认。

### Task 4: 声明回填（半天）

- [ ] `website/script-as-rust.md`（EN+zh）：L1 表恢复三库行、141→恢复后总数、
  诚实边界删除 STRPARAM 条目。
- [ ] `parity/docs/known-divergences.md`：头部警示改为修复记录（详条翻转
  fixed + 机制描述）；`docs/plans/KNOWN-DEBT-AND-RISKS.md` 行翻转。
- [ ] 仪表盘 L1 目录核对（report 的 maturity section）。

## 2. 验证矩阵

- a2r golden 全绿 + 新增 golden ≥1；
- parity 全量 report：三库恢复 + 核心七库不回归（141 基线不减）；
- lib 全量（`RUST_MIN_STACK=33554432 cargo test -p auto-lang --lib`，
  唯一允许败 = route::discovery test_exists 环境项）；
- 改 VM 侧才跑实机 038/013/015（预计只动 trans/rust.rs，可豁免）。

## 3. 明确不做

- 不修 `c_*` 系 consumer 库的既有部分失败（c_env_app 6/7 等）——独立事项，
  与本回归无关（其失败形态不同，跑 `run c_env_app` 看诊断再决定是否立项）。
- tokio_stream 0/4 不在本计划（sorted TAP 之外的形态问题，另行登记）。

## 4. 工作流备注（沿用既有模式）

- worktree：`git worktree add .worktrees/plan-fix-427 -b plan-fix/427-strparam master`；
  分阶段全绿 → `--no-ff` 合并 → 删 worktree → 推 origin+gitee。
- bisect 的临时 worktree 用完即删（D 盘易满）。
- 提交前 `git status` 检查：`examples/rust-workspace/`（并行 WIP）与
  `examples/capability-tests/`（未跟踪）绝不能 add。
