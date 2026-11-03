---
plan_id: PLAN-907
status: archived              # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: secure-random
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-01

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: ["docs/specs/stdlib/a2r-std.md"]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 4
total_steps: 4
---

# [PLAN-907] secure-random

## 变更摘要

a2r-std 新增基于 OS CSPRNG 的安全随机原语 `rand::secure_hex(n)`（n 字节 → 2n 位小写 hex），并在 stdlib `.at` 面暴露后端轨声明（`use auto.rand` → `rand.secure_hex(n)` → `a2r_std::rand::secure_hex`），补齐后端轨"受支持的 ≥128-bit 安全随机"缺口。消费者：auto-chat CHAT-002（P-REQ-1，2026-11-01 用户目标指令授权本计划按仓规约实施并继续下游 T-02）。

## 目标

- G1 后端轨 `.at` 代码可调用 `rand.secure_hex(n)`，生成物调用 `a2r_std::rand::secure_hex`。
- G2 实现基于 OS 熵（getrandom crate，Windows ProcessPrng/CryptGenRandom 路径），非时间种子、非自创密码学。
- G3 与 a2r-std 既有哨兵错误约定一致：绝不 panic，失败返回空串并记录 `rand::last_error()`。
- G4 双路径分发齐备（`use auto.rand` 后裸 `rand.secure_hex(...)` Dot 臂 + `auto.rand.secure_hex(...)` Bina 臂）。
- G5 非目标：VM 轨随机（已有 weak natives，不属本计划）、KDF/密码方案、digest。

## 架构方案

- `crates/a2r-std/src/rand.rs`：`secure_hex(n: i32) -> String` + `last_error() -> String`，`getrandom = "0.2"`（镜像缓存 0.2.15/0.2.17 在位）；手工 hex 编码不加新 dep。
- `stdlib/auto/rand.at`：track-neutral 声明（env.at 先例；无属性 = 后端轨映射）。
- `stdlib/auto/rand.rs.at`：`#[rs]` 镜像签名（KNOWN-DEBT 396 manual-copy 模式）。
- 分发臂：`call_dot_methods.rs`（Dot 路径，sqlite 先例同款）+ `call_module_methods.rs`（Bina 路径对称）。
- parity：`a2r_std_signature_parity.rs` 登记 rand 对。

## 需求分析与背景调查

- CHAT-002 T-00 实证（docs/plans/907 证据指针见 auto-chat 仓 plan §9）：a2r-std 模块面无 rand/digest/KDF；`uuid()`=epoch 纳秒 hex；bundled SQLite（libsqlite3-sys 0.27）Windows 种子仅 SYSTEMTIME+PID+tick+QPC（SQLITE_WIN32_USE_UUID 未定义）；VM 轨 `Random` 实为 SystemTime 纳秒 Xorshift64。
- 后端轨映射机制：`use auto.X` + `X.fn()` 走 `call_dot_methods.rs` 硬编码臂（sqlite.open/last_error、env.get 同款）；`auto.X.fn()` Bina 臂在 `call_module_methods.rs`。
- getrandom 0.2 API：`getrandom::getrandom(&mut [u8]) -> Result<(), getrandom::Error>`。

## 详细设计

1. `rand.rs`：
   - `static LAST_ERROR: Mutex<String>`；`fn record_error(msg: String)`。
   - `pub fn secure_hex(n: i32) -> String`：n<=0 → ""；buf=n bytes；`getrandom::getrandom(&mut buf)` 失败 → 记录错误返回 ""；小写 hex 输出（`format!("{:02x}")` 循环）。
   - `pub fn last_error() -> String`。
2. `rand.at`：`pub fn secure_hex(n int) str;`、`pub fn last_error() str;` + 文档注释（含"仅后端轨；VM 轨另有 random.at natives"边界说明）。
3. `rand.rs.at`：`#[rs]` 两签名镜像。
4. 分发臂两处：`("rand", "secure_hex")` → `a2r_std::rand::secure_hex(...)`（int 直传）；`("rand", "last_error")` → `a2r_std::rand::last_error()`。
5. parity 登记：`("rand.rs.at", PairOpt { rs_mod: "rand", implicit_self: false, qualified_methods: false, renames: &[], allow_only_at: &[], allow_only_rs: &[] })`。

## 测试设计

| Case | oracle | 入口 |
|---|---|---|
| R-1 单元：长度/字符集/唯一性/非空/last_error 初始空 | n=32 → 64 位 [0-9a-f]；连续调用不重复；n=0/负 → "" | `cargo t a2r-std`（rand.rs mod tests） |
| R-2 金样：`rand.secure_hex(32)` 调用发射 `a2r_std::rand::secure_hex(32)` 且 `use a2r_std` | expected.rs 逐字节 | `cargo t a2r`（新增 `test/a2r/17_rust_std/018_rand`） |
| R-3 parity：rand.rs.at ↔ rand.rs 名/元数对齐 | harness 零漂移 | `cargo t a2r`（a2r_std_signature_parity） |
| R-4 消费者探针（落地后）：auto-chat 生成后端经 `.at` 调用 secure_hex 返回高熵 hex | 两 token 前缀随机、长度 64 | auto-chat 仓重建 + HTTP 探针（属 CHAT-002 T-02 证据） |

## 验收标准

- AC-1（owner=work）：R-1/R-2/R-3 全绿于冻结提交；零新 warning（health check）。
- AC-2（owner=work）：分发双臂齐备；金样覆盖 Dot 路径。
- AC-3（owner=review）：独立复审核对 AC-1/2 与 rand.rs.at/分发臂实际 diff。
- AC-4（owner=merge）：落地 master ff；R-4 消费者探针在 auto-chat 通过（或记名遗留）；Spec 沉淀 docs/specs/stdlib/a2r-std.md；归档+清理。

## 执行步骤

- [x] T-1（work）：worktree `D:/autostack/.wt/lang-907/auto-lang`（branch plan-907-dev）；实现 rand.rs/rand.at/rand.rs.at/lib.rs/Cargo.toml + 分发双臂 + parity 登记 + 金样 018_rand。 [✅ 2026-11-01] 提交 `0fd8ac5fb`（11 文件：新 4 + 改 7）；组内补 detached auto-web 兄弟（auto-lang-core 相对路径依赖 auto-web-contract 实测必需）。
- [x] T-2（work）：`cargo t a2r-std a2r` 于冻结提交全绿；零新增 warning。 [✅ 2026-11-01] a2r-std PASS（含 rand 4 单测）；a2r PASS 387/387（含 018_rand 金样与 a2r_std_signature_parity）；rustfmt rand.rs clean；fs.rs:145 格式漂移为既有（stash 复证），不在本计划范围。
- [x] T-3（review）：独立复审（/auto-plan:review 范式）AC-1/2。 [✅ 2026-11-01] 独立上下文复审 `0fd8ac5fb`：AC-1/AC-2/security/一致性/遗漏扫描/健康 六区全 pass；2 条非阻塞观察（rand.rs vec 分配上限属计划设计内；金样测试断言消息内嵌样本仅测试路径）。全 diff 逐行核对：纯增量（235+/2-，2 行为白名单追加 rand）。
- [x] T-4（merge）：落 master（feat(stdlib): a2r-std secure random via OS CSPRNG (Plan 907)）；guard+清理 worktree；Spec 沉淀；归档。 [✅ 2026-11-01] merge `9381e0746`（feat `0fd8ac5fb` 并入 master，plan 记账分叉以常规 merge 收敛）；wt-guard clean → worktree/plan-907-dev/lang-907 组（含 detached auto-web、auto-down 兄弟）全清；Spec 沉淀 docs/specs/a2r-std/project.md（rand 模块契约 + 清单行）+ plans.md 907 行；R-4 消费者探针：auto-chat workspace 重建依赖闭包通过、`.at` 功能探针由 CHAT-002 T-02 首个消费者交付承载。

## 复审记录

- stage: review | plan_id: PLAN-907 | outcome: pass | commit: 0fd8ac5fb（worktree plan-907-dev） | 复审范围：AC-1/2 逐项、rand.rs 安全性（getrandom/小写 hex/哨兵失败/无日志）、三方签名一致、遗漏与 workaround 扫描、健康检查 | 结论：pass，无阻塞发现。
- stage: merge | plan_id: PLAN-907 | outcome: pass | master merge 9381e0746 | worktree/branch/group 全清（guard clean） | Spec：docs/specs/a2r-std/project.md + plans.md | 消费者：auto-chat P-REQ-1 解阻（workspace 闭包重建通过） | 归档 docs/plans/archive/907-secure-random.md。

## 待澄清事项

无产品方向待确认。授权来源：2026-11-01 用户目标指令"在 auto-lang 仓按其 Plan/worktree 规约实现 P-REQ-1（a2r-std 增加基于 OS CSPRNG 的 secure_hex(n) 及 stdlib 后端轨声明），落地后继续执行 CHAT-002 T-02 直至复审与归档"。
