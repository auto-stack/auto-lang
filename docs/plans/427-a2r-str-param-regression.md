# Plan 427: a2r 字符串形参借用回归修复（DIV-A2R-STRPARAM-1）

> **状态**: ✅ 全部任务完成（2026-08-23）——修复 3f6aa1be(396 §2.4) 引入的 is_str_slice_var 误判;三库恢复 100%,L1 回到 260 例/10 库
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

### Task 1: bisect 定位引入提交（半天）✅ 2026-08-23

- [x] 定位完成（源码考古 + 复现实证，比逐点建 worktree 更快）：
  - **引入提交：`3f6aa1be`**（Plan 396 §2.4，2026-08-21 13:08，
    "fix(396): §2.2/§2.3/§2.4 + 裸循环变量克隆根治"）。
  - **失效机制一句话**：§2.4 给 `is_str_slice_var` 补查了
    `local_var_types` 的 StrSlice 登记，而该 map 把**所有** str 型局部
    （含显式 `let x str`——产物 Rust 中是 owned String）都记为 StrSlice，
    于是 `needs_borrow` 被抑制、调用点 `.as_str()` 不再发射 →
    `check_str(n, name, actual, expected)` 直传 String → E0308。
    （这正是 rust.rs ~:8108 Plan 376 Pass 7 注释警告过的同一陷阱被
    重新引入；§2.4 的真实目标——&str 返回 scrutinee 的 `Some(x)`
    is-arm 绑定——本应走专属集合。）
  - **实证链**：master 二进制转译 serde_json 产物在 main.rs:463 报
    E0308 `expected &str, found String`（与 §0 症状逐字吻合）；
    `git log -L :is_str_slice_var` 显示该臂由 3f6aa1be 唯一加入。
  - 备注：`3f6aa1be^`（61712dac）的 parity 实测被环境问题污染
    （aliyun 镜像索引瞬时缺 `async_stream` 包，a2r 列 missing），
    该数据点弃用；结论以源码考古 + master 复现 + 修复后恢复为准。

### Task 2: 根因修复（1 天内）✅ 2026-08-23

- [x] 定位 `trans/rust.rs` 中 `&str` 形参调用点自动借用的判定链
  （`fn_str_param_indices` 注册于 fn_decl 发射 ~:11188，消费于调用编译
  ~:7129/7269+；另有 417-D2 的 `register_import_signatures` 注册路径
  ~:12294/:18137）。判定为何对本例失效——嫌疑方向：
  ①局部 fn 的注册时机 vs 调用点发射顺序（同文件先定义后调用本应覆盖）；
  ②`local_var_types` 推断 `actual` 为 String 的路径变了；
  ③as_str 插入条件被某批收紧。
- [x] 修复 + 单测：`crates/auto-lang/test/a2r/` 新增 golden（`&str` 形参 +
  String 实参 + `.as_str()` 断言）；golden 流程照旧（wrong.rs 独立
  cargo build 验证后提升）。
  - 修复：`str_slice_pattern_bindings` 专属集合（is-arm 两处注册点同步
    填充，local_var_types 登记保留供其他消费者）；`is_str_slice_var`
    第二臂从查 local_var_types 改为查该集合；fn_decl /
    transpile_body_stmts 入口随 local_var_types 同步清空（fn 作用域）。
  - golden `04_strings/008_str_param_borrow`：显式标注/推断两种
    `let x str` 局部（断言 `.as_str()` 发射）+ 字面量/形参直传（断言
    不发射）+ `strip_prefix is Some(rest)`（断言 §2.4 保护不发射）。
    产物独立 crate cargo build + 运行 5/5 ok 后提升。
- [x] **编译级回归防线**：评估后采用轻量形态——`a2r_compile_smoke_str_
  param_borrow`（a2r_tests.rs，`#[ignore]` 按需跑）：对零依赖的 008
  产物跑裸 `rustc --crate-type=lib --emit=metadata` 类型检查（~1s，
  无 cargo/网络）。全量 golden 每例 cargo build 因成本与镜像抖动
  （本次 aliyun 索引缺包实卡两次）不做。

### Task 3: parity 三库恢复（半天）✅ 2026-08-23

- [x] `auto-parity run serde_json / url / base64` 三库三向恢复全绿
  （56/56、30/30、33/33 或当前用例数）；全量 `report` 重生成仪表盘
  （**必须 `AUTO_BINARY=<新构建>`**，默认 PATH 旧 binary 会全 0——
  417-final 踩坑两次）。
  - 实测：serde_json **56/56**、url **30/30**、base64 **33/33**。
- [x] http_client_sync 的 mock-server 首跑竞态注意复跑确认。
  - 首跑仪表盘 3/5（a2r 3/Rust 4）→ 单库复跑 **5/5 (100%)** 确认竞态；
    仪表盘重生成一次取干净快照。

### Task 4: 声明回填（半天）✅ 2026-08-23

- [x] `website/script-as-rust.md`（EN+zh）：L1 表恢复三库行、141→**260**、
  诚实边界删除 STRPARAM 条目。
- [x] `parity/docs/known-divergences.md`：头部警示改为修复记录（机制描述 +
  gaps 表加 ✅ 行）；`docs/plans/KNOWN-DEBT-AND-RISKS.md` 行翻转。
- [x] 仪表盘 L1 目录核对（report 的 maturity section）。

## 2. 验证矩阵

- a2r golden 全绿 + 新增 golden ≥1；
- parity 全量 report：三库恢复 + 核心七库不回归（141 基线不减）；
- lib 全量（`RUST_MIN_STACK=33554432 cargo test -p auto-lang --lib`，
  唯一允许败 = route::discovery test_exists 环境项）；
- 改 VM 侧才跑实机 038/013/015（预计只动 trans/rust.rs，可豁免）。

## 3. 明确不做

- 不修 `c_*` 系 consumer 库的既有部分失败——独立事项。
  （复审更新 2026-08-23：c_env_app 当时的 6/7 其实同为 STRPARAM 形态，
  修复后顺带恢复 7/7；c_crawler/c_http_get/c_json_app/c_wget/tokio_stream
  的 0% 合并前后不变，维持另行立项口径。）
- tokio_stream 0/4 不在本计划（sorted TAP 之外的形态问题，另行登记）。

## 5. 复审记录（2026-08-23，合并后）

- 仪表盘逐库 diff（e03a0d91 → 合并后）：变化恰为目标三库 + http_client_sync
  （竞态恢复）+ c_env_app（6/7→7/7 顺带）；其余 13 库零变化——无回归。
- 修复完整性：RustTrans 仅 2 个构造器均已初始化 str_slice_pattern_bindings；
  Some(x) 注册仅 is_stmt 发射器两处（Expr::Some / OptionPattern），均已接入；
  match 无独立发射器（Auto 模式匹配走 is 块），无遗漏路径。
- 合并树补测：分支点验证不等价于合并后验证（master 并行并入 421/422/428/
  420），合并后 master 全量 test-trans **3440/0**（route::discovery 环境项
  本次亦过，佐明其为环境抖动）。golden 008 含在其中。
- workaround 检查：无。修复为主流写法（fn 作用域专属集合，仿
  by_value_iter_bindings/spec_bound_idents 家族）；is-arm 处保留
  local_var_types 双登记系有意为之（Plan 376E `.to_string()` 路径仍消费），
  已在代码注释与本节说明。

## 4. 工作流备注（沿用既有模式）

- worktree：`git worktree add .worktrees/plan-fix-427 -b plan-fix/427-strparam master`；
  分阶段全绿 → `--no-ff` 合并 → 删 worktree → 推 origin+gitee。
- bisect 的临时 worktree 用完即删（D 盘易满）。
- 提交前 `git status` 检查：`examples/rust-workspace/`（并行 WIP）与
  `examples/capability-tests/`（未跟踪）绝不能 add。
