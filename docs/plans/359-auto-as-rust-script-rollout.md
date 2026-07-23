# Plan 359 — Auto as Rust's Script Layer — Rollout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为下一版本宣传点"Auto 是 Rust 的脚本层"补齐从地基到门面的全部交付物——VM↔a2R 一致性验证体系、Rust 生态用例库、"From Script to Ship"互动教程、英雄演示与核心叙事——使宣传点具备可审计的证据、可上手的教育路径、可传播的演示。

**Architecture:** 按 C(地基)→D(弹药)→B(旗舰)→A(门面)四阶段推进。C 修复名不副实的 `conformance_tests.rs`、清理 `parity/` 的 DBG355 残留、建立 CI 门禁与公开 parity 仪表盘；D 在 `parity/libs/` 下扩展真实生态用例（每例三重身份：parity 用例 / 教程素材 / 宣传弹药）；B 新建 `docs/script-to-ship/` 主题化 tour（保留现有 `docs/tour/` 不动）并扩展 `<CodeView>` 接入真实 a2r 转译；A 用 D 的最佳用例做 hero demo 与落地页。每阶段有独立出口条件，可阶段性验收。

**Tech Stack:** Rust (cargo workspace), Auto 语言 (.at), clap, TAP (Test Anything Protocol), axum (playground 后端), Vue 3 + VitePress (website/playground 前端), CodeMirror 6, GitHub Actions (CI), a2r-std 运行时

**Design spec:** `docs/design/auto-as-rust-script-strategy.md`

---

## 阶段总览

| 阶段 | 子项目 | 核心交付 | 出口条件 | 解锁 |
|------|--------|---------|---------|------|
| **C1** | 地基-诚实化 | 修复 `conformance_tests.rs` 名实不符 | 注释与实现一致；对外文案不再误引 | 宣传可信度底线 |
| **C2** | 地基-parity 加固 | 清理 DBG355、补全 p1/p2、CI 门禁 | `auto-parity phase p1/p2` 全绿；CI 跑 parity | D 可大规模产用例 |
| **C3** | 地基-仪表盘 | 公开 parity 仪表盘 + L1/L2/L3 成熟度目录 | 仪表盘网页可访问；成熟度目录可被 A/B 引用 | A/B 的"现状声明"有链接目标 |
| **D1** | 弹药-核心用例 | serde_json/regex/CLI 三类用例 L1 验证 | 三类用例 parity 三向通过 | B 有素材；A 有 hero 候选 |
| **D2** | 弹药-薄弱区补强 | trait 高级 + generators 用例 | parity 通过；补 a2r 测试薄弱区 | 教程可覆盖高级 Rust 模式 |
| **D3** | 弹药-async 用例 | tokio 子集 + HTTP 客户端（同步）用例 | async parity（sorted TAP）通过 | 教程 async 章节有素材 |
| **B1** | 旗舰-CodeView 扩展 | 新建 `<ScriptShipView>` 组件接入 `/api/trans` | 组件可并排展示 VM 与 a2r 转译输出 | tour 可用新组件 |
| **B2** | 旗舰-tour 内容 | `docs/script-to-ship/` 6-8 章 + 双语 | 章节在 website 可运行、有导航 | 完整教育路径 |
| **A1** | 门面-最小版 | 用现有 demo 做 hero + 落地页叙事 v1（仅 L2 声明） | 落地页上线，叙事打动人验证 | （可与 C 并行探路） |
| **A2** | 门面-正式版 | hero 用 L1 用例 + 对照页 + 现状声明 + 双语 | 宣传物料就绪 | 发布 |

**关键路径**：C1→C2→C3 → D1→(D2‖D3) → B1→B2 → A2。A1 与 C 并行。

**与现有 Plan 的关系**：本计划复用 Plan 347（`parity/` 框架与 8 库骨架）的产出。C2 在 Plan 347 基础上加 CI 与清理；D 在 Plan 347 的 `libs/` 下扩展新用例。如某子项目需更深专项设计，可单独立项并以本计划为依据。

---

## 文件结构（本计划触及的关键路径）

```
auto-lang/
├── crates/auto-lang/src/tests/
│   └── conformance_tests.rs          # C1: 修复名实不符
├── parity/
│   ├── crates/auto-parity/src/
│   │   ├── main.rs                   # C2: 清理 DBG355
│   │   ├── runner.rs                 # C2: 清理 DBG355
│   │   └── report.rs                 # C3: 仪表盘数据生成
│   ├── libs/                         # D1/D2/D3: 扩展用例
│   │   ├── serde_json/  regex/        # Plan 347 已有，D1 补 L1 验证
│   │   ├── cli_app/  trait_advanced/  # D2 新增
│   │   ├── generators/                # D2 新增
│   │   └── http_client_sync/          # D3 新增（tokio 见 Plan 347）
│   └── docs/
│       ├── known-divergences.md       # C2/C3: 维护
│       └── parity-dashboard.html      # C3: 新建（CI artifact）
├── crates/a2r-std/src/
│   └── lib.rs                         # D1/D3: 按需扩展模块
├── website/
│   ├── .vitepress/theme/components/
│   │   ├── CodeView.vue               # 保留不动（语言 tour 仍用）
│   │   └── ScriptShipView.vue         # B1: 新建
│   ├── scripts/prepare-content.js     # B2: 可能扩展白名单前缀
│   ├── .vitepress/config/sidebar-docs-en.ts   # B2: 加 Script-to-Ship 条目
│   └── docs/script-to-ship/           # B2: 新建（chXX-script-to-ship/*.md + .at）
├── examples/script-to-ship-demos/     # D/A: 面向用户的 demo 副本
└── .github/workflows/
    └── parity-ci.yml                  # C2: 新建
```

---

# Phase C1: 地基-诚实化（修复 conformance 名实不符）

**目标：** 让 `conformance_tests.rs` 的注释与实现一致，消除"虚假宣传"风险。

**背景事实：**
- 文件 `crates/auto-lang/src/tests/conformance_tests.rs:1-9` 头注释写 "Tests that AutoVM and a2r produce identical output"，但 `run_conformance_test`（行 22-45）只调 `run_autovm_capture`（行 28），从未调 a2r。
- 该测试在 `crates/auto-lang/src/tests.rs:63-64` 通过 `#[cfg(feature = "test-vm-files")]` 声明。
- 真正的三向对比在 `parity/`（Plan 347），不在此文件。

**出口条件：**
- `conformance_tests.rs` 注释明确声明当前只验证 AutoVM-vs-golden，a2r 一致性由 `parity/` 框架负责。
- 文件内不再出现暗示"VM↔a2r 一致性"的措辞。

---

### Task C1.1: 改写 conformance_tests.rs 头注释

**Files:**
- Modify: `crates/auto-lang/src/tests/conformance_tests.rs:1-9`

- [ ] **Step 1: 读取当前头注释**

Run: `sed -n '1,9p' crates/auto-lang/src/tests/conformance_tests.rs`
Expected: 看到 "Dual-Execution Conformance Tests" 与 "Tests that AutoVM and a2r produce identical output" 等措辞。

- [ ] **Step 2: 改写头注释为诚实描述**

将 `crates/auto-lang/src/tests/conformance_tests.rs:1-9` 替换为：

```rust
// AutoVM Output Regression Tests
//
// What this file verifies: AutoVM execution output is stable against golden
// files (expected_output.txt). Each case runs input.at through AutoVM and
// compares captured stdout to the golden. This catches VM regressions only.
//
// What this file does NOT verify: AutoVM-vs-a2r behavioral parity. Three-way
// parity (AutoVM vs a2r-transpiled Rust vs native Rust) is handled by the
// separate `parity/` workspace (see parity/docs/parity-guide.md and Plan 347).
//
// Strategy:
// 1. Run input.at through AutoVM → capture stdout
// 2. Compare AutoVM output against expected_output.txt
// 3. On mismatch, write .wrong.out for debugging
```

- [ ] **Step 3: 全文扫描，替换残留的"VM ↔ a2r"/"Dual-Execution"措辞**

Run: `grep -n -i "dual.execution\|a2r\|identical output\|VM.*and.*a2r" crates/auto-lang/src/tests/conformance_tests.rs`
Expected: 列出残留行号。逐一改为 "AutoVM output stability" 类措辞。例如行 8 原 "(Future) Transpile via a2r..." 整行删除（已并入新头注释的"does NOT verify"段）。

- [ ] **Step 4: 同步修复 tests.rs 的声明注释**

Modify `crates/auto-lang/src/tests.rs:63-64`，把注释从 "Plan 266: AutoVM ↔ a2r semantic conformance tests" 改为 "AutoVM output regression tests (golden-file based); VM↔a2r parity lives in parity/":

```rust
#[cfg(feature = "test-vm-files")]
mod conformance_tests; // AutoVM output regression tests (golden-file); VM↔a2r parity is in parity/
```

- [ ] **Step 5: 验证编译与测试仍可运行**

Run: `cargo test -p auto-lang --features test-vm-files --test '*' -- --list 2>&1 | grep conformance | head`
Expected: 列出 conformance 测试名（编译通过，未破坏测试本身）。

- [ ] **Step 6: Commit**

```bash
git add crates/auto-lang/src/tests/conformance_tests.rs crates/auto-lang/src/tests.rs
git commit -m "fix(tests): make conformance_tests.rs header honest about scope (Plan 359 C1)

The header claimed 'AutoVM and a2r produce identical output' but the
implementation only runs AutoVM vs golden files. Rewrite the header to
state what it actually verifies (AutoVM output stability) and point to
the parity/ workspace for real VM↔a2r comparison. This removes a
credibility landmine for the 'Auto as Rust script' messaging."
```

---

### Task C1.2: 核对 design spec 与对外文案不再误引

**Files:**
- Read-only check: `docs/design/auto-as-rust-script-strategy.md`

- [ ] **Step 1: 确认母纲第 4.3 节"漏洞 1 的对外处理"措辞与 C1 修复一致**

Run: `grep -n "conformance_tests.rs" docs/design/auto-as-rust-script-strategy.md`
Expected: 母纲第 4.3 节已写明"在 C 子项目堵上之前不在对外文案里引用 conformance_tests.rs 作为一致性证据"——C1 完成后，conformance 已被诚实降级为"VM 输出回归测试"，措辞仍然成立（对外仍不应作为"VM↔a2r 一致性"证据）。无需修改。

- [ ] **Step 2: 记录 C1 完成事实到母纲（可选，轻量）**

在 `docs/design/auto-as-rust-script-strategy.md` 第 1.2 节"漏洞 1"段落末尾追加一行：

```
（已于 Plan 359 C1 修复：conformance_tests.rs 注释已诚实降级为"AutoVM 输出回归测试"。）
```

- [ ] **Step 3: Commit**

```bash
git add docs/design/auto-as-rust-script-strategy.md
git commit -m "docs(strategy): note C1 conformance header fix completed (Plan 359)"
```

---

# Phase C2: 地基-parity 加固（清理 + CI 门禁）

**目标：** 清理 `parity/` 的 DBG355 诊断残留，确认 p1/p2 库三向通过，建立 parity CI 门禁。

**背景事实：**
- DBG355 残留：`parity/crates/auto-parity/src/main.rs:143,147,148,149,152`（5 处 `eprintln!("DBG355 ...")`），`runner.rs:83-96`（4 处，会写 `DBG355_*.txt` 到 cwd）。`parity/.gitignore:19-20` 已忽略产物。
- CI 现状：`.github/workflows/` 下无任何 parity/conformance job；唯一 Rust workflow `auto-lsp-ci.yml` 只覆盖 auto-lsp 且 `workflow_dispatch` 手动触发。
- `auto-parity` CLI：`parity/crates/auto-parity/src/main.rs:27-43`，子命令 `run <lib>` / `phase <p0-p4>` / `all` / `list`。
- `parity/libs/` 现有 8 库：`_dummy, base64, regex, rusqlite, serde_json, sha2, tokio, url`。

**出口条件：**
- `parity/` 下无 DBG355 残留，stderr 干净。
- `cargo run -p auto-parity -- phase p1` 与 `phase p2` 在干净环境全绿（或已知 diverge 在 `known-divergences.md` 记录）。
- `.github/workflows/parity-ci.yml` 存在，在 PR 上跑 parity p1/p2。

---

### Task C2.1: 清理 DBG355 诊断打印

**Files:**
- Modify: `parity/crates/auto-parity/src/main.rs:143-152`
- Modify: `parity/crates/auto-parity/src/runner.rs:83-96`
- Modify: `parity/.gitignore:19-20`

- [ ] **Step 1: 定位所有 DBG355 残留**

Run: `cd parity && grep -rn "DBG355" crates/ src/ 2>/dev/null; grep -n "DBG355" .gitignore`
Expected: 列出 main.rs 5 行、runner.rs 相关行、.gitignore 2 行。

- [ ] **Step 2: 删除 main.rs 的 DBG355 eprintln**

删除 `parity/crates/auto-parity/src/main.rs` 中所有 `eprintln!("DBG355 ...")` 行（约 143、147、148、149、152 行）。若这些打印是为调试 main 中 results 比较逻辑，删除前先确认逻辑无误；保留实际业务逻辑，只删调试打印。

- [ ] **Step 3: 删除 runner.rs 的 DBG355 调试块**

删除 `parity/crates/auto-parity/src/runner.rs:83-96` 的 DBG355 注释与 `eprintln!`/文件写入（`DBG355_*.txt` 创建）。这是 `run_vm` 结果缺失诊断，删除后若 run_vm 缺失结果应改为正常的 `Result` 错误传播（若现有代码已用 Result 返回错误，则直接删打印即可；否则补一个 `return Err(format!("..."))`）。

- [ ] **Step 4: 清理 .gitignore 的 DBG355 规则**

删除 `parity/.gitignore:19-20` 的 DBG355 注释与 `DBG355_*.txt` 忽略行（不再产生该文件）。

- [ ] **Step 5: 验证 parity 仍可编译运行**

Run: `cd parity && cargo build -p auto-parity 2>&1 | tail -5 && cargo run -p auto-parity -- list 2>&1 | head -10`
Expected: 编译无 warning（无 unused），`list` 子命令正常列出 8 个库，stderr 无 DBG355 输出。

- [ ] **Step 6: Commit**

```bash
git add parity/crates/auto-parity/src/main.rs parity/crates/auto-parity/src/runner.rs parity/.gitignore
git commit -m "chore(parity): remove DBG355 debug residue from runner/main (Plan 359 C2)

Leftover diagnostic prints (9 eprintln! sites + DBG355_*.txt file writes)
from active debugging polluted stderr and cwd. Removed; the underlying
result-missing diagnostics now flow through normal Result error paths."
```

---

### Task C2.2: 确认 p1/p2 库三向通过，更新 known-divergences

**Files:**
- Read/Update: `parity/docs/known-divergences.md`
- Possibly fix: `parity/libs/base64/`, `parity/libs/url/`, `parity/libs/serde_json/`, `parity/libs/regex/` 下源码

- [ ] **Step 1: 跑 p1 阶段（base64, url）**

Run: `cd parity && cargo run -p auto-parity -- phase p1 2>&1 | tail -30`
Expected: 看到 base64/url 三向（VM/a2r/rust）对比结果。记录任何 `not ok` / diverge。

- [ ] **Step 2: 跑 p2 阶段（serde_json, regex）**

Run: `cd parity && cargo run -p auto-parity -- phase p2 2>&1 | tail -30`
Expected: 同上。

- [ ] **Step 3: 对每个失败用例分类**

参考 `parity/docs/parity-guide.md` 的"Bug classification"真值表，将每个失败归类为：consistent / replication bug / a2r bug / VM bug。对 a2r bug 和 VM bug，在 `parity/docs/known-divergences.md` 按现有格式（`- **DIV-<LIB>-<SIDE>-N**: ...`，含 status: fixed/open/accepted）记录；对 accepted 类（如数值精度、panic 信息差异）标注 rationale。

- [ ] **Step 4: 修复可快速修复的 diverge（若有）**

对 status=open 且属 a2r 复刻或测试用例 bug 的，修复 `parity/libs/<lib>/auto/<lib>.at` 或 `tests/auto/*.at`，重跑确认通过，在 known-divergences 标 status=fixed。

- [ ] **Step 5: 验证 p1/p2 干净（已知 diverge 外全绿）**

Run: `cd parity && cargo run -p auto-parity -- phase p1 2>&1 | grep -c "not ok" && cargo run -p auto-parity -- phase p2 2>&1 | grep -c "not ok"`
Expected: `not ok` 计数等于 known-divergences.md 中 p1/p2 的 open 项数（无新增未记录 diverge）。

- [ ] **Step 6: Commit**

```bash
git add parity/docs/known-divergences.md parity/libs/
git commit -m "test(parity): confirm p1/p2 green, record divergences (Plan 359 C2)"
```

---

### Task C2.3: 建立 parity CI 门禁

**Files:**
- Create: `.github/workflows/parity-ci.yml`

- [ ] **Step 1: 确认 auto 二进制构建方式与 feature**

Run: `cargo build -p auto --release 2>&1 | tail -3 && ls target/release/auto* 2>/dev/null | head -3`
Expected: `auto` 二进制可构建（parity runner 通过 `--auto-binary` 调用它，默认 "auto"）。

- [ ] **Step 2: 写 parity-ci.yml（p1/p2 硬门禁）**

Create `.github/workflows/parity-ci.yml`:

```yaml
name: parity

on:
  pull_request:
    paths:
      - 'crates/auto-lang/**'
      - 'crates/a2r-std/**'
      - 'crates/auto/**'
      - 'parity/**'
      - '.github/workflows/parity-ci.yml'
  push:
    branches: [master]

jobs:
  parity-p1-p2:
    runs-on: ubuntu-latest
    timeout-minutes: 40
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: ". -> target\nparity -> parity/target"
      - name: Build auto binary
        run: cargo build -p auto --release
      - name: Run parity p1 (base64, url)
        working-directory: parity
        run: cargo run -p auto-parity -- --auto-binary ../target/release/auto phase p1
      - name: Run parity p2 (serde_json, regex)
        working-directory: parity
        run: cargo run -p auto-parity -- --auto-binary ../target/release/auto phase p2
      - name: Upload parity artifacts on failure
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: parity-failure
          path: parity/
```

- [ ] **Step 3: 本地模拟 CI 验证（用 release auto）**

Run: `cargo build -p auto --release && cd parity && cargo run -p auto-parity -- --auto-binary ../target/release/auto phase p1 2>&1 | tail -5`
Expected: p1 通过（与 C2.2 一致）。这确认 CI 命令行正确。

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/parity-ci.yml
git commit -m "ci: add parity p1/p2 gate on PRs touching a2r/auto-lang/parity (Plan 359 C2)"
```

---

# Phase C3: 地基-parity 仪表盘

**目标：** 产出一份可公开访问的 parity 仪表盘，展示通过率、覆盖面、已知 diverge，供 A/B 子项目做"现状声明"的链接目标。

**出口条件：**
- `parity/docs/parity-dashboard.html` 由 `auto-parity` 生成，含：总用例数、通过数、按库/阶段汇总、已知 diverge 列表（链 known-divergences.md）、L1/L2/L3 成熟度目录。
- 仪表盘可作为 CI artifact 上传，或嵌入 website 静态页。

---

### Task C3.1: 给 auto-parity 加 report 子命令生成仪表盘

**Files:**
- Modify: `parity/crates/auto-parity/src/main.rs`（加 `Report` 子命令）
- Create/Modify: `parity/crates/auto-parity/src/report.rs`

- [ ] **Step 1: 确认 report.rs 现有内容**

Run: `ls parity/crates/auto-parity/src/report.rs 2>/dev/null && head -30 parity/crates/auto-parity/src/report.rs || echo "no report.rs"`
Expected: 确认是否存在及现有职责（Plan 347 可能有占位）。

- [ ] **Step 2: 在 Command 枚举加 Report 变体**

Modify `parity/crates/auto-parity/src/main.rs:27-43`，在 enum Command 末尾加：

```rust
    /// Generate a static HTML dashboard of parity results
    Report {
        #[arg(short, long, default_value = "docs/parity-dashboard.html")]
        output: String,
    },
```

并在 main 的 match 分发里加：

```rust
        Command::Report { output } => report::generate_dashboard(&output)
            .map_err(|e| { eprintln!("report error: {e}"); 1 }),
```

- [ ] **Step 3: 实现 generate_dashboard（汇总所有 phase 结果）**

在 `parity/crates/auto-parity/src/report.rs` 实现 `generate_dashboard(output_path: &str) -> Result<(), String>`：
- 调 `runner::run_phase("p1")`、`run_phase("p2")`（或 `all`）收集每库 TapResult。
- 汇总：总用例数、通过数、按库统计、失败列表。
- 读 `docs/known-divergences.md`，解析 DIV 条目（按现有格式 `- **DIV-<LIB>-<SIDE>-N**`）。
- 渲染为自包含 HTML（内联 CSS，无外部依赖），含四节：
  1. Summary（总通过率，大字）。
  2. Coverage matrix（库 × 阶段 × VM/a2r/rust 三列）。
  3. Known divergences（表格，含 status）。
  4. **Maturity directory**（L1/L2/L3 分组列表，依据见 Step 4 规则）。
- 写到 `output_path`。

- [ ] **Step 4: 定义 L1/L2/L3 成熟度归类规则**

在 report.rs 顶部以常量/注释固化归类逻辑（与母纲 §4.1 对齐）：
- **L1（已验证）**：该用例在仪表盘所代表的 phase 中三向通过 → 列入"已验证一致性"。
- **L2（VM 稳定）**：有 VM-vs-golden（conformance）但无 a2r 三向 → 列入"VM 行为已回归测试"。
- **L3（路线图）**：计划支持但当前 phase 未覆盖（如 p3/p4 的 sha2/rusqlite/reqwest/tokio）→ 列入"路线图"。

L3 列表硬编码为 p3/p4 库名（来自 Plan 347 阶段表）。

- [ ] **Step 5: 验证仪表盘生成**

Run: `cd parity && cargo run -p auto-parity -- report --output docs/parity-dashboard.html && ls -la docs/parity-dashboard.html`
Expected: 生成 HTML 文件，非空。用浏览器打开确认渲染正常。

- [ ] **Step 6: Commit**

```bash
git add parity/crates/auto-parity/src/main.rs parity/crates/auto-parity/src/report.rs parity/docs/parity-dashboard.html
git commit -m "feat(parity): add 'report' subcommand generating HTML dashboard (Plan 359 C3)

Dashboard summarizes pass rate, coverage matrix, known divergences, and
L1/L2/L3 maturity directory. Serves as the evidence target for the
'Auto as Rust script' messaging's 'status declaration' sections."
```

---

### Task C3.2: 仪表盘接入 CI 自动产出

**Files:**
- Modify: `.github/workflows/parity-ci.yml`

- [ ] **Step 1: 在 parity-ci.yml 加 report 步骤与 artifact 上传**

在 `parity-ci.yml` 的 `parity-p1-p2` job 末尾（p2 步骤之后、artifact 步骤之前）加：

```yaml
      - name: Generate parity dashboard
        working-directory: parity
        run: cargo run -p auto-parity -- --auto-binary ../target/release/auto report --output docs/parity-dashboard.html
      - name: Upload parity dashboard
        uses: actions/upload-artifact@v4
        with:
          name: parity-dashboard
          path: parity/docs/parity-dashboard.html
```

- [ ] **Step 2: 验证 workflow YAML 语法**

Run: `python -c "import yaml; yaml.safe_load(open('.github/workflows/parity-ci.yml'))" && echo "YAML OK"`
Expected: `YAML OK`。

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/parity-ci.yml
git commit -m "ci(parity): auto-generate and upload dashboard artifact (Plan 359 C3)"
```

---

# Phase D1: 弹药-核心用例（serde_json / regex / CLI）

**目标：** 让 serde_json、regex 两类核心生态用例达到 L1（三向一致），并新增一个 CLI 用例（对标 std::fs + clap 场景，补 a2r 测试薄弱区"纯 Rust 输出"）。

**背景事实：**
- `parity/libs/serde_json/` 与 `parity/libs/regex/` 已存在（Plan 347），D1 聚焦"补到 L1 全绿 + 做成可展示用例"。
- a2r-std 无 regex 模块（`crates/a2r-std/src/lib.rs:8-17` 仅 10 模块，无 regex）。regex 复刻走纯 Auto 实现（parity/libs/regex/auto/regex.at），不依赖 a2r-std.regex。
- a2r 测试薄弱区"纯 Rust 输出（无 a2r-std 依赖）"仅 1 例——CLI 用例对标此。

**出口条件：**
- serde_json、regex 在 parity 三向通过（D1 阶段，仪表盘归 L1）。
- 新增 `parity/libs/cli_app/` 用例，三向通过。
- 三个用例在 `examples/script-to-ship-demos/` 有面向用户的副本（带 README 说明 Dev/Ship 两侧）。

---

### Task D1.1: serde_json 用例达 L1

**Files:**
- Fix as needed: `parity/libs/serde_json/auto/serde_json.at`, `parity/libs/serde_json/tests/auto/*.at`

- [ ] **Step 1: 跑 serde_json parity 看当前状态**

Run: `cd parity && cargo run -p auto-parity -- run serde_json 2>&1 | tail -30`
Expected: 看到三向结果与失败用例（若 C2.2 已修则应接近全绿）。

- [ ] **Step 2: 修复剩余 diverge（分类 + 修复 + 记录）**

对每个 `not ok`：按 parity-guide 真值表分类。a2r/复刻 bug 修源码；accepted 差异记入 known-divergences.md（status=accepted + rationale）。

- [ ] **Step 3: 验证全绿**

Run: `cd parity && cargo run -p auto-parity -- run serde_json 2>&1 | grep -E "^(ok|not ok)" | grep -c "not ok"`
Expected: `0`（或等于 known-divergences 中 serde_json 的 accepted 项数）。

- [ ] **Step 4: Commit**

```bash
git add parity/libs/serde_json/ parity/docs/known-divergences.md
git commit -m "test(parity): serde_json reaches L1 three-way green (Plan 359 D1)"
```

---

### Task D1.2: regex 用例达 L1

**Files:**
- Fix as needed: `parity/libs/regex/auto/regex.at`, `parity/libs/regex/tests/auto/*.at`

- [ ] **Step 1-4: 同 D1.1 流程，对象换为 regex**

Run: `cd parity && cargo run -p auto-parity -- run regex 2>&1 | tail -30`

修复 diverge → 验证 `grep -c "not ok"` 归零 → 记录 known-divergences → Commit:

```bash
git add parity/libs/regex/ parity/docs/known-divergences.md
git commit -m "test(parity): regex reaches L1 three-way green (Plan 359 D1)"
```

---

### Task D1.3: 新增 CLI 用例（补"纯 Rust 输出"薄弱区）

**Files:**
- Create: `parity/libs/cli_app/README.md`
- Create: `parity/libs/cli_app/auto/cli_app.at`
- Create: `parity/libs/cli_app/tests/auto/*.at`
- Create: `parity/libs/cli_app/tests/rust/Cargo.toml`, `tests/cli_app.rs`

- [ ] **Step 1: 设计 CLI 用例（对标 Rust 的 std::fs + 文件处理）**

用例功能：读取一个文本文件，统计行数/单词数/字符数（类 `wc` 的最小子集），打印结果。刻意**不依赖 a2r-std 之外的库**，以验证"纯 Rust 输出"能力。Auto 侧用 `auto.fs`（a2r-std 已有 `fs` 模块，`crates/a2r-std/src/lib.rs:13`）。

- [ ] **Step 2: 写 Auto 复刻版**

Create `parity/libs/cli_app/auto/cli_app.at`：实现 `count_file(path str) { lines int, words int, chars int }`，读取文件内容、按行/空格分割统计。公共 API 与下面 Rust oracle 一致。

- [ ] **Step 3: 写 Auto 测试用例**

Create `parity/libs/cli_app/tests/auto/basic.at`、`edge_cases.at`（空文件、单行无换行、多字节字符等），每个用例打印 `ok N - <name>` 或 `not ok` 的 TAP 行。

- [ ] **Step 4: 写 Rust oracle 测试**

Create `parity/libs/cli_app/tests/rust/Cargo.toml`（独立 crate）与 `tests/cli_app.rs`，用 std::fs 实现等价逻辑，对相同输入产出相同 TAP 输出。

- [ ] **Step 5: 跑 parity 三向**

Run: `cd parity && cargo run -p auto-parity -- run cli_app 2>&1 | tail -20`
Expected: 三向通过（VM/a2r/rust 输出一致）。若 a2r 路径失败（如 `auto.fs` 在 a2r 下映射问题），修 a2r-std.fs 或记 diverge。

- [ ] **Step 6: Commit**

```bash
git add parity/libs/cli_app/
git commit -m "feat(parity): add cli_app use case (wc-style, no-dep pure Rust output) (Plan 359 D1)"
```

---

### Task D1.4: 三个用例做面向用户副本

**Files:**
- Create: `examples/script-to-ship-demos/serde_json-demo/{main.at, README.md}`
- Create: `examples/script-to-ship-demos/regex-demo/{main.at, README.md}`
- Create: `examples/script-to-ship-demos/cli-demo/{main.at, README.md}`

- [ ] **Step 1: 为每个用例做精简可读的单文件 demo**

从 parity/libs/<lib>/auto/<lib>.at 提取核心逻辑，做成 `main.at`（带 `fn main()`，可 `auto main.at` 直接跑），附 README 说明：Dev 模式（`auto main.at`）、Ship 模式（`auto trans --path main.at rust`）、一致性（链接仪表盘）。

- [ ] **Step 2: 验证 demo 两模式可跑**

Run: `for d in serde_json-demo regex-demo cli-demo; do echo "== $d =="; auto examples/script-to-ship-demos/$d/main.at 2>&1 | tail -3; auto trans --path examples/script-to-ship-demos/$d/main.at rust 2>&1 | tail -3; done`
Expected: 每个 demo 的 VM 跑与 a2r 转译都成功。

- [ ] **Step 3: Commit**

```bash
git add examples/script-to-ship-demos/
git commit -m "docs(examples): add user-facing Script-to-Ship demos for D1 use cases (Plan 359 D1)"
```

---

# Phase D2: 弹药-薄弱区补强（trait 高级 + generators）

**目标：** 补 a2r 测试最薄弱的两个 Rust 模式——trait 高级特性（默认方法、关联类型、带 bound 的泛型 trait impl）与 generators（`yield` → `~Iter<T>`，当前仅 1 例）。

**出口条件：**
- `parity/libs/trait_advanced/` 用例三向通过，覆盖：trait 默认方法、关联类型、带 trait bound 的泛型 impl。
- `parity/libs/generators/` 用例三向通过，覆盖 `yield` 迭代器、惰性序列。
- 两用例有面向用户副本。

---

### Task D2.1: trait_advanced 用例

**Files:**
- Create: `parity/libs/trait_advanced/{README.md, auto/trait_advanced.at, tests/auto/*.at, tests/rust/Cargo.toml, tests/trait_advanced.rs}`

- [ ] **Step 1: 设计覆盖三类 trait 高级特性的用例**

三个子场景：
1. **默认方法**：`spec Greeter { fn greet() { print(name()) } fn name() str }`，实现者只提供 `name`，`greet` 用默认。
2. **关联类型**：`spec Container { type Item; fn get() Item }`，实现者指定关联类型。
3. **带 bound 的泛型 impl**：`spec Ord<T> { fn cmp(other T) int }`，对实现了 Ord<T> 的类型做泛型排序。

- [ ] **Step 2: 写 Auto 复刻版与测试**

Create `parity/libs/trait_advanced/auto/trait_advanced.at`（三个 spec + 三个实现 + 测试用例），`tests/auto/*.at` 打印 TAP。

- [ ] **Step 3: 写 Rust oracle**

Create `parity/libs/trait_advanced/tests/rust/`（trait + impl + tests），产出相同 TAP。

- [ ] **Step 4: 跑 parity 三向，修复 diverge**

Run: `cd parity && cargo run -p auto-parity -- run trait_advanced 2>&1 | tail -20`

**预期会暴露 a2r 对 trait 高级特性的转译缺口**。若 a2r 输出无法编译（如不支持关联类型转译），记录到 known-divergences.md（status=open，标注 "a2r gap: associated types"），并在母纲 L3 目录体现。这是**诚实暴露边界**，不是失败。

- [ ] **Step 5: 验证与记录**

对能通过的子场景标 L1；a2r 暂不支持的子场景标 L3（路线图），在 known-divergences 详记。

- [ ] **Step 6: Commit**

```bash
git add parity/libs/trait_advanced/ parity/docs/known-divergences.md
git commit -m "feat(parity): add trait_advanced use case (default methods, assoc types, bounded generics) (Plan 359 D2)

Surfaces a2r gaps in advanced trait features; pass-through cases reach L1,
unsupported cases documented as L3 roadmap per honesty strategy."
```

---

### Task D2.2: generators 用例

**Files:**
- Create: `parity/libs/generators/{README.md, auto/generators.at, tests/auto/*.at, tests/rust/Cargo.toml, tests/generators.rs}`

- [ ] **Step 1: 设计 generators 用例**

两个子场景：
1. **基本 yield 迭代器**：`fn naturals() ~Iter<int> { var i = 0; loop { yield i; i = i + 1 } }`，取前 N 项。
2. **惰性序列组合**：range → map(×2) → filter(>5) → take(3)，验证惰性求值顺序。

- [ ] **Step 2: 写 Auto 复刻版与测试**

Create `parity/libs/generators/auto/generators.at` 与 `tests/auto/*.at`（TAP 输出）。

- [ ] **Step 3: 写 Rust oracle**

Create `parity/libs/generators/tests/rust/`（用 std::iter::from_fn 或自定义 Iterator 实现），产出相同 TAP。

- [ ] **Step 4: 跑 parity 三向，修复/记录 diverge**

Run: `cd parity && cargo run -p auto-parity -- run generators 2>&1 | tail -20`

`yield` → `~Iter<T>` 的 a2r 转译当前仅 1 例覆盖，可能暴露 diverge。按 D2.1 Step 5 同样处理（通过标 L1，不支持的标 L3）。

- [ ] **Step 5: Commit**

```bash
git add parity/libs/generators/ parity/docs/known-divergences.md
git commit -m "feat(parity): add generators use case (yield iterators, lazy chains) (Plan 359 D2)"
```

---

### Task D2.3: D2 用例面向用户副本

**Files:**
- Create: `examples/script-to-ship-demos/trait-demo/{main.at, README.md}`
- Create: `examples/script-to-ship-demos/generators-demo/{main.at, README.md}`

- [ ] **Step 1-3: 同 D1.4 流程**

为 trait_advanced、generators 各做精简单文件 demo + README，验证 VM/a2r 双模式可跑，Commit。

```bash
git add examples/script-to-ship-demos/
git commit -m "docs(examples): add trait-demo and generators-demo Script-to-Ship demos (Plan 359 D2)"
```

---

# Phase D3: 弹药-async 用例

**目标：** 补 a2r 最大短板——async/await（当前仅 3 个 trivial 案例）。聚焦"同步 HTTP 客户端"（用 a2r-std 已有的 ureq/http 模块）与 tokio 子集（Plan 347 已有 `parity/libs/tokio/`）。

**背景事实：**
- a2r-std HTTP 走 `ureq`（同步，`crates/a2r-std/Cargo.toml`），非 reqwest。故"HTTP 客户端"用例做同步版本（D3），更易达 L1。
- tokio 异步完成顺序非确定，parity 用 sorted TAP 比较（parity-guide.md 已说明）。
- Auto async 语法：`fn foo() ~RetType`（`~T` 返回类型 = async）。

**出口条件：**
- `parity/libs/http_client_sync/` 用例三向通过（HTTP GET/POST + JSON 解析，同步）。
- `parity/libs/tokio/` 在 sorted TAP 模式下三向通过（spawn/join、channel 子集）。
- 两用例有面向用户副本。

---

### Task D3.1: http_client_sync 用例

**Files:**
- Create: `parity/libs/http_client_sync/{README.md, auto/http_client_sync.at, tests/auto/*.at, tests/rust/Cargo.toml, tests/http_client_sync.rs}`
- Maybe extend: `crates/a2r-std/src/http.rs`

- [ ] **Step 1: 设计同步 HTTP 客户端用例**

功能：对一个本地 mock HTTP 端点（parity 测试自带，避免外网依赖），做 GET 与 POST JSON，解析响应。Auto 侧用 `auto.http`（a2r-std.http，ureq）。Rust oracle 直接用 ureq。

**避免外网依赖**：测试用例启动一个最小的 in-process HTTP server（parity/rust 测试侧用 std::net::TcpListener 起假 server），或用预录的响应 fixture。

- [ ] **Step 2: 写 Auto 复刻版与测试**

Create `parity/libs/http_client_sync/auto/http_client_sync.at`（GET/POST 函数 + JSON 解析）与 `tests/auto/*.at`。

- [ ] **Step 3: 写 Rust oracle**

Create `parity/libs/http_client_sync/tests/rust/`（ureq + 假 server + tests）。

- [ ] **Step 4: 跑 parity 三向，修复 diverge**

Run: `cd parity && cargo run -p auto-parity -- run http_client_sync 2>&1 | tail -20`

若 a2r-std.http 在 a2r 转译路径下缺失绑定，扩展 `crates/a2r-std/src/http.rs`。

- [ ] **Step 5: Commit**

```bash
git add parity/libs/http_client_sync/ crates/a2r-std/src/http.rs parity/docs/known-divergences.md
git commit -m "feat(parity): add http_client_sync use case (synchronous ureq-based) (Plan 359 D3)"
```

---

### Task D3.2: tokio 用例达 sorted-TAP 一致

**Files:**
- Fix as needed: `parity/libs/tokio/auto/tokio.at`, `parity/libs/tokio/tests/auto/*.at`

- [ ] **Step 1: 跑 tokio parity 现状**

Run: `cd parity && cargo run -p auto-parity -- run tokio 2>&1 | tail -30`
Expected: tokio 是 p4，可能未完成。记录状态。

- [ ] **Step 2: 确认/启用 sorted TAP 比较**

确认 `parity/crates/auto-parity/src/compare.rs` 对 async 库用 sorted TAP（完成顺序非确定）。若未启用，按 parity-guide.md §async 说明启用。

- [ ] **Step 3: 补全 tokio 子集用例（spawn/join、channel）**

聚焦两个子场景：
1. **spawn + join**：并发 N 个任务，join 收集结果，sorted 输出。
2. **channel**：mpsc channel 生产-消费，sorted 输出消费结果。

修复 `parity/libs/tokio/auto/tokio.at` 与 tests 至 sorted TAP 一致。

- [ ] **Step 4: 验证与记录 diverge**

Run: `cd parity && cargo run -p auto-parity -- run tokio 2>&1 | grep -c "not ok"`
Expected: 0 或等于已知 accepted diverge 数。a2r 对 `~T` async 转译若有缺口，记 L3。

- [ ] **Step 5: Commit**

```bash
git add parity/libs/tokio/ parity/docs/known-divergences.md
git commit -m "test(parity): tokio reaches sorted-TAP parity for spawn/join + channel (Plan 359 D3)"
```

---

### Task D3.3: D3 用例面向用户副本

**Files:**
- Create: `examples/script-to-ship-demos/http-client-demo/{main.at, README.md}`
- Create: `examples/script-to-ship-demos/async-demo/{main.at, README.md}`

- [ ] **Step 1-3: 同 D1.4 流程**

为 http_client_sync、tokio 各做精简 demo + README（async demo 注明"sorted 输出对比"），验证双模式，Commit。

```bash
git add examples/script-to-ship-demos/
git commit -m "docs(examples): add http-client-demo and async-demo Script-to-Ship demos (Plan 359 D3)"
```

---

# Phase B1: 旗舰-CodeView 扩展（新建 ScriptShipView）

**目标：** 新建 `<ScriptShipView>` 组件，接入真实 `/api/trans`，支持"VM 跑 + a2r 转译"并排展示与运行对比，供 Script-to-Ship tour 使用。**保留现有 `<CodeView>` 不动**（语言 tour 仍用）。

**背景事实：**
- 现有 `CodeView.vue`（`website/.vitepress/theme/components/CodeView.vue`）只 POST `/api/run`，props 为 `auto/rust/c/typescript/python/caption/runnable/apiUrl`，不支持多文件、不支持 trans。
- 真正接 `/api/trans` + 多文件的是 `packages/auto-playground-vue/src/AutoPlaygroundFull.vue` + `composables/usePlaygroundFull.ts`（已有 `projectDir/projectFiles/transpile(target)`）。
- 后端 `POST /api/trans`（`crates/auto-playground/src/routes/trans.rs`）请求体 `TransRequest { source, target, project_dir?, files? }`，响应 `TransResponse { target, files: Vec<TransFile{path,code}>, source_map }`。Rust target 走 `transpile_rust_project` 产出 Cargo.toml + 多 .rs。
- `<Listing>` 靠 `chXX-` 前缀自动识别 tour 目录（`prepare-content.js:91-92`）。

**出口条件：**
- `website/.vitepress/theme/components/ScriptShipView.vue` 存在，接受 Auto 源码，可：① VM 运行（调 `/api/run`）显示 stdout；② a2r 转译（调 `/api/trans` target=rust）显示转译后 Rust 代码；③ 可选"编译运行 Rust"对比输出。
- 组件在 website theme 注册（`theme/index.ts`）。

---

### Task B1.1: 设计 ScriptShipView 组件 API

**Files:**
- Create: `website/.vitepress/theme/components/ScriptShipView.vue`（骨架）

- [ ] **Step 1: 定义组件 props**

Props（与 CodeView 风格一致，全 optional）：

```ts
const props = defineProps<{
  auto: string          // Auto 源码（必给）
  caption?: string
  apiUrl?: string       // 默认 'http://localhost:3030'
  showRust?: boolean    // 是否显示 a2r 转译的 Rust，默认 true
  compareRun?: boolean  // 是否显示"并排运行对比"（VM vs Rust 编译运行），默认 false
}>()
```

- [ ] **Step 2: 设计 UI 布局**

三栏（compareRun=true 时）或两栏：
- 左：Auto 源码（CodeMirror，autoLanguage，可编辑）+ "Run in VM" 按钮 + VM stdout 面板。
- 中（showRust）：转译后 Rust 代码（只读 CodeMirror，lang-rust）+ "Transpile" 按钮。
- 右（compareRun）：Rust 编译运行 stdout 面板 + 一致性指示（绿勾✓ 一致 / 红叉✗ 不一致）。

- [ ] **Step 3: 创建组件骨架（template + 空逻辑）**

Create `ScriptShipView.vue`，搭出三栏布局（用现有 CodeView 的 CodeMirror 初始化代码作参考），逻辑留空占位（下一步实现）。

- [ ] **Step 4: Commit**

```bash
git add website/.vitepress/theme/components/ScriptShipView.vue
git commit -m "feat(website): scaffold ScriptShipView component (Plan 359 B1)"
```

---

### Task B1.2: 实现 VM 运行与 a2r 转译逻辑

**Files:**
- Modify: `website/.vitepress/theme/components/ScriptShipView.vue`

- [ ] **Step 1: 实现 runInVm()**

复用 CodeView.vue:197-220 的 fetch 模式：

```ts
async function runInVm() {
  const res = await fetch(`${props.apiUrl || 'http://localhost:3030'}/api/run`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ source: autoCode.value }),
  })
  const data = await res.json()
  vmOutput.value = data.stdout || data.stderr || '(no output)'
}
```

- [ ] **Step 2: 实现 transpileToRust()**

调 `/api/trans`，target=rust，取首个 .rs 文件代码：

```ts
async function transpileToRust() {
  const res = await fetch(`${props.apiUrl || 'http://localhost:3030'}/api/trans`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ source: autoCode.value, target: 'rust' }),
  })
  const data = await res.json()
  // TransResponse: { target, files: [{path, code}], source_map }
  const mainRs = data.files.find((f: any) => f.path.endsWith('main.rs')) || data.files[0]
  rustCode.value = mainRs ? mainRs.code : '// transpile produced no files'
}
```

- [ ] **Step 3: 绑定按钮与状态**

`runInVm` 绑定左栏 Run 按钮，`transpileToRust` 绑定中栏 Transpile 按钮。状态：`autoCode`（ref，初值 props.auto，可编辑）、`vmOutput`、`rustCode`。

- [ ] **Step 4: 本地验证**

启动 playground 后端（`cargo run -p auto-playground`）与 website dev（`cd website && npm run dev`），在一个临时 md 页面放 `<ScriptShipView auto="fn main() { print(\"hi\") }" />`，验证 Run 显示 "hi"、Transpile 显示转译后 Rust。

- [ ] **Step 5: Commit**

```bash
git add website/.vitepress/theme/components/ScriptShipView.vue
git commit -m "feat(website): ScriptShipView VM-run + a2r-transpile logic (Plan 359 B1)"
```

---

### Task B1.3: 实现"并排运行对比"（compareRun）

**Files:**
- Modify: `website/.vitepress/theme/components/ScriptShipView.vue`
- Maybe extend: `crates/auto-playground/src/routes/run.rs`（新增 `/api/run_rust` 端点，见 Step 1 决策）

- [ ] **Step 1: 决策 Rust 编译运行的实现方式**

后端已有 `/api/run_code`（`crates/auto-playground/src/routes/run_code.rs` + `code_runner.rs`，执行转译后 Python/Rust/C）。确认它是否接受 Rust 源码并编译运行。

Run: `grep -n "rust\|rs" crates/auto-playground/src/code_runner.rs | head`
Expected: 看 run_code 是否支持 rust target。

**选项 a（优先）**：复用 `/api/run_code`——前端转译拿到 Rust 代码后，POST `/api/run_code` { target: 'rust', code: rustCode } 编译运行。
**选项 b**：新增 `/api/compare` 一次性端点（后端串起 transpile → cargo build → run → 返回 stdout）。更省往返但后端改动大。

倾向选项 a（复用现有）。若 run_code 不支持 rust，先补 code_runner.rs 的 rust 分支。

- [ ] **Step 2: 实现 runRust()（选项 a）**

```ts
async function runRust() {
  await transpileToRust()  // 确保 rustCode 最新
  const res = await fetch(`${props.apiUrl || 'http://localhost:3030'}/api/run_code`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ target: 'rust', code: rustCode.value }),
  })
  const data = await res.json()
  rustOutput.value = data.stdout || data.stderr || '(no output)'
  // 一致性比较（trim 后字符串相等）
  consistent.value = vmOutput.value.trim() === rustOutput.value.trim()
}
```

- [ ] **Step 3: 实现"一键对比"按钮**

compareRun=true 时显示 "Run Both & Compare" 按钮，依次调 runInVm() + runRust()，右栏显示 rustOutput + 一致性指示（绿勾/红叉）。

- [ ] **Step 4: 本地验证对比功能**

用 D1 的 serde_json-demo 源码放进去，验证 VM 与 Rust 输出一致（绿勾）。

- [ ] **Step 5: Commit**

```bash
git add website/.vitepress/theme/components/ScriptShipView.vue crates/auto-playground/src/code_runner.rs
git commit -m "feat(website): ScriptShipView side-by-side compare mode (Plan 359 B1)"
```

---

### Task B1.4: 注册组件并加类型导出

**Files:**
- Modify: `website/.vitepress/theme/index.ts`

- [ ] **Step 1: 在 theme/index.ts 注册 ScriptShipView**

Modify `website/.vitepress/theme/index.ts:54-63`（现有全局组件注册区），加：

```ts
import ScriptShipView from './components/ScriptShipView.vue'
// ...在 enhanceApp 内：
app.component('ScriptShipView', ScriptShipView)
```

- [ ] **Step 2: 验证组件在 md 页面可用**

在 website 任一 md 页面写 `<ScriptShipView auto="fn main() { print(1+1) }" compare-run />`，dev 起来确认渲染。

- [ ] **Step 3: Commit**

```bash
git add website/.vitepress/theme/index.ts
git commit -m "feat(website): register ScriptShipView global component (Plan 359 B1)"
```

---

# Phase B2: 旗舰-Script-to-Ship tour 内容

**目标：** 新建 `docs/script-to-ship/` 主题化 tour（6-8 章），用 `<ScriptShipView>` 展示 Dev/Ship/Bridge 三段式，接入 website sidebar。

**背景事实：**
- 现有 `docs/tour/`（12 章，hello/types/.../interop）保留不动，是语言入门。
- `<Listing>` 靠 `chXX-` 前缀自动识别（`prepare-content.js:91-92`）。但 `<ScriptShipView>` 是新组件，prepare-content.js 现有 `listingToCodeView` 只生成 `<CodeView>`——需扩展或直接在 md 里手写 `<ScriptShipView>`。

**出口条件：**
- `docs/script-to-ship/` 含 6-8 章 md + 每章可运行 `.at`。
- website sidebar 有 "Script to Ship" 分组。
- 中英双语（EN 先行，CN 跟进）。

---

### Task B2.1: 扩展 prepare-content.js 支持新组件（或决策手写）

**Files:**
- Maybe modify: `website/scripts/prepare-content.js`

- [ ] **Step 1: 决策 Listing→ScriptShipView 的转换方式**

两个选项：
- **选项 a**：扩展 prepare-content.js 的 `listingToCodeView`，根据 Listing 的 `view="scriptship"` 属性生成 `<ScriptShipView>` 而非 `<CodeView>`。改动小、保持作者用 Listing 的统一体验。
- **选项 b**：在 md 里直接手写 `<ScriptShipView auto="..." compare-run />`（代码内联），不走 Listing。简单但代码内联在 md 里不优雅。

倾向选项 a（保持 Listing 统一）。

- [ ] **Step 2: 实现 Listing 的 view 属性分支（选项 a）**

Modify `website/scripts/prepare-content.js:164-188`（`listingToCodeView`），读取 `attrs.view`（默认 "codeview"）：

```js
const view = attrs.view || 'codeview'
if (view === 'scriptship') {
  // 生成 <ScriptShipView auto="..." :compare-run="true" caption="..." />
  // 只读 .at 文件（auto prop），不读 expected 文件（Rust 实时转译）
  return `<ScriptShipView auto="${escapeAttr(autoCode)}" :compare-run="true" caption="${escapeAttr(caption)}" />`
}
// 原有 CodeView 逻辑...
```

- [ ] **Step 3: 扩展白名单前缀，让 script-to-ship 目录被识别**

`resolveListingDir`（`prepare-content.js:85-102`）当前对 `chXX-` 前缀自动加 `tour/`。script-to-ship 目录若命名 `script-to-ship/ch01-...`，需让它解析到 `docs/script-to-ship/`。

Modify `prepare-content.js:314-325`（included doc dirs 白名单），把 `'script-to-ship'` 加入白名单；并让 `resolveListingDir` 识别 `script-to-ship/` 前缀（类似 tour/ 的处理）。

- [ ] **Step 4: 验证转换**

写一个测试 `<Listing file="script-to-ship/ch01-hello/01_hello.at" view="scriptship" caption="test" />`，跑 `node website/scripts/prepare-content.js`，确认生成 `<ScriptShipView>`。

- [ ] **Step 5: Commit**

```bash
git add website/scripts/prepare-content.js
git commit -m "feat(website): support <Listing view='scriptship'> -> ScriptShipView (Plan 359 B2)"
```

---

### Task B2.2: 撰写 Script-to-Ship tour 章节（初版 6 章）

**Files:**
- Create: `docs/script-to-ship/README.md`
- Create: `docs/script-to-ship/ch01-hello-script-ship.md` + `ch01-hello-script-ship/01_*.at`
- Create: `docs/script-to-ship/ch02-ai-in-the-loop.md` + 用例
- Create: `docs/script-to-ship/ch03-types-ownership.md` + 用例
- Create: `docs/script-to-ship/ch04-errors.md` + 用例
- Create: `docs/script-to-ship/ch05-traits-generics.md` + 用例
- Create: `docs/script-to-ship/ch06-ship-release.md` + 用例

- [ ] **Step 1: 写 README（tour 总览）**

`docs/script-to-ship/README.md`：说明这是"工作流教程"（区别于语言 tour），三段式 Dev/Ship/Bridge 概述，前置要求（会基本 Auto，否则先看 docs/tour），链接 parity 仪表盘（现状声明）。

- [ ] **Step 2: 写 ch01 Hello, Script & Ship**

最小闭环：一个 `fn main() { print("hello, script-ship") }`，用 `<Listing view="scriptship" compare-run />` 展示 VM 跑 + a2r 转 + 一致。讲解"同一份代码两种执行模式"。

- [ ] **Step 3: 写 ch02 AI in the Loop**

讲解 AI 闭环（生成→验证→迭代→冻结），强调脚本模式为何适合 AI 试错（无编译、可丢弃）。用 `<ScriptShipView>` 展示一个 AI 会"反复改"的小例子（如逐步给一个函数加错误处理）。

- [ ] **Step 4: 写 ch03 Types & Ownership**

用 D1 的 serde_json-demo 素材，讲 struct/enum 与所有权（`.view`/`.mut`/`.take` → Rust `&`/`&mut`/move）。ScriptShipView 展示 a2r 如何转译所有权。

- [ ] **Step 5: 写 ch04 Errors**

讲 `!` 函数与 `.?` 传播 → Rust `Result`/`?`。用 D1 的 cli-demo（文件不存在错误）作素材。

- [ ] **Step 6: 写 ch05 Traits & Generics**

用 D2 的 trait-demo 素材，讲 spec → trait/impl/Box<dyn>。**诚实标注**：高级 trait（关联类型等）若 D2 标 L3，在此章注明"路线图"。

- [ ] **Step 7: 写 ch06 Ship: Release**

讲发布工作流：`auto trans --path main.at rust` → 链 a2r-std → cargo build → 性能对比。引用 D 的用例，展示 Ship 后的 Rust 二进制与脚本模式的性能差（若可测）。

- [ ] **Step 8: Commit**

```bash
git add docs/script-to-ship/
git commit -m "docs(script-to-ship): 6-chapter interactive tour (EN) (Plan 359 B2)"
```

---

### Task B2.3: 接入 website sidebar

**Files:**
- Modify: `website/.vitepress/config/sidebar-docs-en.ts`

- [ ] **Step 1: 在 sidebar 加 Script to Ship 分组**

Modify `website/.vitepress/config/sidebar-docs-en.ts`，在 Tour 分组（L642-699）之后加一个并列顶层对象：

```ts
{
  text: 'Script to Ship',
  collapsed: true,
  items: [
    { text: 'Overview', link: 'script-to-ship/README' },
    { text: 'Hello, Script & Ship', link: 'script-to-ship/ch01-hello-script-ship' },
    { text: 'AI in the Loop', link: 'script-to-ship/ch02-ai-in-the-loop' },
    { text: 'Types & Ownership', link: 'script-to-ship/ch03-types-ownership' },
    { text: 'Errors', link: 'script-to-ship/ch04-errors' },
    { text: 'Traits & Generics', link: 'script-to-ship/ch05-traits-generics' },
    { text: 'Ship: Release', link: 'script-to-ship/ch06-ship-release' },
  ],
},
```

- [ ] **Step 2: 跑 prepare-content 并验证构建**

Run: `cd website && node scripts/prepare-content.js && npm run build 2>&1 | tail -10`
Expected: 构建成功，script-to-ship 页面生成，sidebar 显示新分组。

- [ ] **Step 3: 浏览器验证交互**

dev 起来，逐章点开，确认每个 `<ScriptShipView>` 能 Run、Transpile、Compare。

- [ ] **Step 4: Commit**

```bash
git add website/.vitepress/config/sidebar-docs-en.ts website/docs/script-to-ship/
git commit -m "docs(website): wire Script-to-Ship tour into sidebar (Plan 359 B2)"
```

---

### Task B2.4: 中文版（CN 跟进）

**Files:**
- Create: `website/zh/docs/script-to-ship/`（镜像 EN）
- Modify: `website/.vitepress/config/sidebar-docs-zh.ts`（若有对应中文 sidebar 配置）

- [ ] **Step 1: 翻译 6 章为中文**

逐章翻译（EN 为准，CN 跟进，术语对齐现有 `docs/syntax.cn.md` 风格）。代码块与 Listing 不变。

- [ ] **Step 2: 中文 sidebar 接入**

在中文 sidebar 配置加对应的 "脚本到发布" 分组。

- [ ] **Step 3: Commit**

```bash
git add website/zh/docs/script-to-ship/ website/.vitepress/config/
git commit -m "docs(script-to-ship): Chinese translation of 6-chapter tour (Plan 359 B2)"
```

---

# Phase A1: 门面-最小版（探路，可与 C 并行）

**目标：** 用现有 playground-demo 快速做一个 hero + 落地页叙事 v1，验证宣传点是否打动人。**只声明 L2 证据**（不超前）。

**出口条件：**
- website 首页或新落地页有 hero demo（可点 Run 看脚本跑、点 Transpile 看转 Rust）。
- 三段式叙事（Dev/Ship/Bridge）+ 不可能三角可视化 + 与 Python+C/C++ 对照表。
- 现状声明模块（L1/L2/L3，C3 仪表盘就绪后升级为可链接）。

---

### Task A1.1: hero demo（基于现有 demo）

**Files:**
- Modify/Create: `website/index.md`（或新建 `website/script-as-rust.md`）
- Maybe reuse: `<AutoPlayground>` 或新建 hero 专用组件

- [ ] **Step 1: 选 hero 题材**

从 `examples/playground-demo/`（8 个 VM/a2r/a2c 全 ✅ 的 demo）选最直观的——建议 `04-fibonacci.at`（递归，易理解）或 `07-types.at`（展示 struct/方法）。

- [ ] **Step 2: 在落地页嵌入可交互 hero**

用现有 `<AutoPlayground>` 组件（已注册，支持 run+trans），或写一个更聚焦的 hero 区（左 Auto、右转译 Rust、下方 Run 输出）。A1 阶段可用 `<AutoPlaygroundFull>` 的简化版。

- [ ] **Step 3: 写 hero 文案**

一句话 punchline（母纲候选）："Python taught the world that fast iteration wins. Rust taught the world that safety wins. Auto refuses to choose." 配合 hero 区。

- [ ] **Step 4: Commit**

```bash
git add website/index.md website/.vitepress/theme/
git commit -m "docs(website): v1 hero demo + narrative (L2 claims only) (Plan 359 A1)"
```

---

### Task A1.2: 三段式叙事 + 对照表 + 现状声明

**Files:**
- Modify: `website/index.md` 或 `website/script-as-rust.md`

- [ ] **Step 1: 写 Dev/Ship/Bridge 三段叙事**

每段一段文字 + 一个可交互代码块（用 `<ScriptShipView>` 若 B1 已完成，否则用 `<CodeView>` 配静态 expected.rs）。

- [ ] **Step 2: 加"不可能三角"可视化**

用 SVG 或简单 CSS 画三角图（开发效率/运行效率/安全），Auto 在中心。参考母纲 §2.2 的 ASCII 图。

- [ ] **Step 3: 加与 Python+C/C++ 对照表**

直接搬母纲 §1.3 的对照表（生态关系/能力对等/迁移成本/一致性保证/AI 辅助五维度）。

- [ ] **Step 4: 加现状声明模块（L2 基线）**

显眼模块列出：当前 L2 覆盖（VM 稳定的 34 conformance 例 + 8 playground-demo）、L1 路线图（C3 仪表盘就绪后链接）、L3 路线图（async 高级等）。**A1 阶段诚实声明"VM↔a2r 三向一致性验证体系建设中（parity/），见路线图"**。

- [ ] **Step 5: Commit**

```bash
git add website/index.md
git commit -m "docs(website): three-act narrative + comparison table + status declaration (Plan 359 A1)"
```

---

# Phase A2: 门面-正式版

**目标：** hero 升级为用 L1 用例（D 产出），现状声明链接真实仪表盘（C3），双语，宣传物料就绪。

**出口条件：**
- hero 用 D1/D3 的最佳 L1 用例（建议 http_client_sync 或 serde_json，真实场景比 fibonacci 有说服力）。
- 现状声明模块的 L1 项可点击跳转 parity 仪表盘。
- EN + CN 双语落地页。
- 与 B 的 tour 互链（落地页"动手试" → script-to-ship tour）。

---

### Task A2.1: hero 升级为 L1 用例

**Files:**
- Modify: `website/index.md` 或 hero 组件

- [ ] **Step 1: 选 D 中最佳 L1 用例作 hero**

评估 D1（serde_json/regex/cli）、D3（http_client_sync）哪个最直观。建议 http_client_sync（HTTP 是 Rust 开发者熟悉的真实场景，且能展示 async 能力）或 serde_json（生态核心）。

- [ ] **Step 2: 替换 hero 区代码与交互**

把 A1 的 fibonacci demo 换成选定用例，确保 ScriptShipView 的 compare-run 显示绿勾（一致性已 L1 验证）。

- [ ] **Step 3: Commit**

```bash
git add website/index.md
git commit -m "docs(website): upgrade hero to L1-verified use case (Plan 359 A2)"
```

---

### Task A2.2: 现状声明链接真实仪表盘

**Files:**
- Modify: `website/index.md` 现状声明模块
- Maybe: 嵌入 parity-dashboard.html 或链接 CI artifact

- [ ] **Step 1: 把现状声明的 L1 项链接到 parity 仪表盘**

A1 的"L1 路线图"改为"L1 已验证"，链接 `parity/docs/parity-dashboard.html`（或 CI 部署的 URL）。每个 L1 项对应仪表盘里一个通过用例。

- [ ] **Step 2: 嵌入仪表盘摘要**

在现状声明模块旁嵌入仪表盘的 summary 区（总通过率大字），或直接 iframe/链接。

- [ ] **Step 3: Commit**

```bash
git add website/index.md
git commit -m "docs(website): link status declaration to live parity dashboard (Plan 359 A2)"
```

---

### Task A2.3: 双语 + tour 互链

**Files:**
- Modify: `website/zh/index.md`（中文落地页）
- Modify: 落地页加"动手试"链接到 script-to-ship tour

- [ ] **Step 1: 翻译落地页为中文**

EN 为准，CN 跟进。punchline 与叙事对齐。

- [ ] **Step 2: 加 tour 互链**

落地页 hero 下方加 "Try it yourself →" 链接到 `script-to-ship/ch01-hello-script-ship`。tour 首页加"← Back to overview"链回落地页。

- [ ] **Step 3: Commit**

```bash
git add website/zh/index.md website/index.md website/docs/script-to-ship/README.md
git commit -m "docs(website): bilingual landing + tour cross-link (Plan 359 A2)"
```

---

## Phase E: 剩余 trait / VM / 语言缺口（补强）

> **⚠️ 执行前置：本 Phase 全部任务必须等 `plan-348/phase7-bug-completion` 那个 agent 合并后再开始。** 该 agent 正在修 VM bug（Task 20-25：E1/A1/G1 补全 + G3 channel 语法 / Result[T,E] 泛型 / struct variant is 解构），与本 Phase 的 VM/语言改动有 master 稳定性关联。等它的 1 个未合并提交（`4f43d0ab` 及后续）进 master、HEAD 自洽后再开 worktree。

**目标：** 把 trait_advanced 暴露的剩余 a2r/VM/语言层缺口逐个修掉，让 trait_advanced parity 库的 L1 子集从 10/10 扩到接近全覆盖，并解锁被 DIV-HTTP-LANG-1 阻塞的 http_client_sync。

**已完成（本 Phase 前置）：**
- DIV-TRAIT-A2R-1（返回值默认方法）：fixed（`9042085b`）
- DIV-TRAIT-A2R-2（泛型 impl 丢参数）：fixed（`15445355`）
- DIV-TRAIT-A2R-3（self. 前缀）：retracted（非 bug，前导点约定）

**剩余缺口（按依赖与风险排序）：**

| 缺口 | 归属 | 难度 | 阻塞项 |
|---|---|---|---|
| E1: DIV-TRAIT-VM-2（VM trait 检查器不跳过默认方法） | VM | 低 | — |
| E2: DIV-TRAIT-VM-1（有界泛型函数 `<T has Spec>`） | VM+解析 | 中 | 依赖 E1 先让默认方法工作 |
| E3: DIV-TRAIT-LANG-1（关联类型） | 语言/解析 | 中 | — |
| E4: DIV-HTTP-LANG-1（stdlib http.at 解析） | 解析 | 中 | 解锁 http_client_sync |
| E5: DIV-A2R-CHAR-AT-1（a2r char_at 类型推断） | a2r | 低 | — |
| E6: trait_advanced parity 扩大 L1（回归性收益） | parity | 低 | 依赖 E1-E3 |

---

### Task E1: DIV-TRAIT-VM-2 — VM trait 检查器跳过默认方法

**问题：** `crates/auto-lang/src/trait_checker.rs:108-114` 的 `check_conformance` 对 spec 每个方法在 type_decl 找实现，找不到就报错。但 spec 方法若有默认 body（`SpecMethod.body` 非空），实现者应能继承，不该报错。

**Files:**
- Modify: `crates/auto-lang/src/trait_checker.rs:95-115`

- [ ] **Step 1: 读 check_conformance 的循环结构**

Run: `sed -n '28,115p' crates/auto-lang/src/trait_checker.rs`
确认遍历 `spec_decl.methods`，对每个 `spec_method` 查 `type_decl.methods` 匹配（None → 报错）。

- [ ] **Step 2: 修 None 分支：若 spec_method 有 body 则跳过**

在行 ~111 的 `None =>` 分支前，加判断：若 `spec_method.body.is_some()`（默认方法），`continue`（视为由默认提供，不报错）。需确认 `SpecMethod` 结构有 `body: Option<...>` 字段（参考 `crates/auto-lang/src/ast/spec.rs`）。

```rust
None => {
    // Plan 359 E1 (DIV-TRAIT-VM-2): a spec method with a default body
    // is satisfied by the default; don't require the implementer to re-declare.
    if spec_method.body.is_some() {
        continue;
    }
    errors.push(/* 原报错 */);
}
```

- [ ] **Step 3: 验证 — 移除 trait_advanced.at 里 Book 的 full_title 重声明**

`parity/libs/trait_advanced/auto/trait_advanced.at` 里实现者目前重声明了默认方法（VM-2 workaround）。删掉重声明，VM 应仍能跑。

Run: `./target/release/auto.exe parity/libs/trait_advanced/auto/trait_advanced.at`
Expected: 不再报 "does not implement required method"，正常输出。

- [ ] **Step 4: 回归 — trait_advanced parity 仍 10/10**

Run: `cd parity && cargo run -p auto-parity -- --auto-binary ../target/release/auto.exe run trait_advanced`
Expected: `Consistency: 10/10 (100.0%)`。

- [ ] **Step 5: Commit**

```bash
git commit -m "fix(vm): trait checker skips spec methods with default body (Plan 359 E1 / DIV-TRAIT-VM-2)"
```

---

### Task E2: DIV-TRAIT-VM-1 — 有界泛型函数 `<T has Spec>`

**问题：** Auto 无法写 `fn max<T has Comparable>(a T, b T) T`——`<T has Spec>` bound 语法被解析器拒绝，且即使解析通过，AutoVM 无法在泛型类型参数 `T` 上分发 spec 方法（"Undefined symbol: T.compare"）。

**Files:**
- Modify: `crates/auto-lang/src/parser.rs`（GenericParam 的 bound 解析）
- Modify: `crates/auto-lang/src/vm/codegen.rs`（泛型函数的 spec 方法分发）
- Modify: `crates/auto-lang/src/vm/monomorphize.rs`（单态化时注入具体类型）

**这是本 Phase 最复杂的任务**（涉及解析 + 单态化 + codegen 三层）。建议拆成子任务，且先做解析（让 `<T has Spec>` 能解析），再做 VM 分发。

- [ ] **Step 1: 调研现有 GenericParam 与 bound 处理**

Run: `grep -n "GenericParam\|has \|bound\|trait_bound" crates/auto-lang/src/parser.rs | head -10`
确认 `GenericParam::Type` 当前是否带 bound 字段（`ast/types.rs` 的 `GenericParam` 定义）。

- [ ] **Step 2: 解析器接受 `<T has Spec>` / `<T as Spec>`**

在 `GenericParam::Type` 加 `bound: Option<Name>` 字段；parser 的泛型参数解析处识别 `has`/`as` 后跟 spec 名。

- [ ] **Step 3: VM 单态化时解析 bound → 具体分发**

`monomorphize.rs` 单态化泛型函数时，若 `T` 被 `has Spec` 约束且实参类型实现了该 spec，将 `T.method()` 解析为该类型的 spec 方法。

- [ ] **Step 4: 测试 — bounded generic 函数三向一致**

在 `parity/libs/trait_advanced/tests/auto/` 加 `bounded_generic.at`，验证 `max<T has Comparable>` 在 VM/a2r/rust 三向一致。

- [ ] **Step 5: Commit**

```bash
git commit -m "feat(vm): bounded generic functions <T has Spec> (Plan 359 E2 / DIV-TRAIT-VM-1)"
```

**风险：高**（三层改动，单态化逻辑复杂）。若 Step 3 卡住，可标 E2 保持 L3 路线图，先做 E1/E3/E4/E5。

---

### Task E3: DIV-TRAIT-LANG-1 — spec 关联类型

**问题：** Auto 的 spec 语法无关联类型构造：`spec Container { type Item; fn get(i int) Item }` 是解析错误（"Expected term, got RBrace"）。

**Files:**
- Modify: `crates/auto-lang/src/ast/spec.rs`（SpecDecl 加 `associated_types: Vec<...>`）
- Modify: `crates/auto-lang/src/parser.rs`（spec 解析识别 `type Item;`）
- Modify: `crates/auto-lang/src/trans/rust.rs`（spec_decl 转译出 `trait C { type Item; ... }`）

- [ ] **Step 1: 调研 spec 解析与 SpecDecl 结构**

Run: `grep -n "parse_spec\|SpecDecl::new\|pub struct SpecDecl" crates/auto-lang/src/parser.rs crates/auto-lang/src/ast/spec.rs | head`

- [ ] **Step 2: SpecDecl 加 associated_types 字段**

`ast/spec.rs` 的 `SpecDecl` 加 `pub associated_types: Vec<Name>`（或带 kind）。`SpecDecl::new` 默认空。

- [ ] **Step 3: parser 在 spec 体里识别 `type Name;`**

spec 体的解析循环里，遇到 `type` 关键字 → 解析为关联类型声明，push 进 `associated_types`。

- [ ] **Step 4: trans/rust.rs 的 spec_decl 输出 `type Item;`**

在 trait 体里，关联类型输出为 `type Item;`（Rust 关联类型声明）。

- [ ] **Step 5: 测试 — 关联类型 spec 转译**

加 `crates/auto-lang/test/a2r/12_specs/006_assoc_types/`，验证 `spec Container { type Item; fn get(i int) Item }` 转译为 `trait Container { type Item; fn get(&self, i: i32) -> Self::Item; }`。

- [ ] **Step 6: Commit**

```bash
git commit -m "feat(lang): spec associated types (Plan 359 E3 / DIV-TRAIT-LANG-1)"
```

**风险：中**（解析+AST+转译三层，但每层改动小）。

---

### Task E4: DIV-HTTP-LANG-1 — stdlib http.at 解析（解锁 http_client_sync）

**问题：** `stdlib/auto/http.at:51` 用 `pub fn Request.method(self Request) str;`（`Type.method` 声明 + 分号），解析器报 "Expected term, got Newline"（在后面的 `///` 文档注释处）。任何 `use auto.http: ...` 都触发此失败。

**这是最高价值的阻塞项**——修它直接解锁 http_client_sync parity 库。

**Files:**
- Modify: `crates/auto-lang/src/parser.rs`（`Type.method` 声明语法 + `;` 结尾的外部方法声明）
- Verify: `stdlib/auto/http.at` 能被解析

- [ ] **Step 1: 精确复现解析失败**

Run: `cat > /tmp/t.at <<'EOF'
type Request
pub fn Request.method(self Request) str;
pub fn Request.path(self Request) str;
EOF
./target/release/auto.exe /tmp/t.at`
Expected: 复现 "Expected term, got Newline" 或类似错误。确认是 `Type.method` 语法还是 `;` 结尾的问题。

- [ ] **Step 2: 调研 parser 对 `Type.method` 的支持**

Run: `grep -n "Type\.method\|method_name\|qualified.*method\|\\." crates/auto-lang/src/parser.rs | grep -i "method\|fn " | head`
看解析器是否识别 `TypeName.method` 作为方法声明（typedecl 外部的方法声明形式）。

- [ ] **Step 3: 实现/修复 `Type.method` 外部声明解析**

在 fn 声明解析处，若 fn 名含 `.`（如 `Request.method`），解析为类型 `Request` 的方法声明（self 类型推断为 Request）。加分号结尾支持（外部声明，无 body）。

- [ ] **Step 4: 验证 stdlib http.at 能解析**

Run: `./target/release/auto.exe -e 'use auto.http: post_sync'`（或最小测试 `use auto.http: post_sync; fn main() {}`）
Expected: 无解析错误。

- [ ] **Step 5: 激活 http_client_sync parity 库**

去掉 `parity/libs/http_client_sync/README.md` 的 blocker 标注；起 mock-server，跑 `cargo run -p auto-parity -- run http_client_sync`，确认三向一致。

- [ ] **Step 6: 把 http_client_sync 加入 phase 表 + 仪表盘**

`parity/crates/auto-parity/src/main.rs` phase_map 加 `("d3", &["http_client_sync"])`；report phases 加 `d3`；重生成仪表盘。

- [ ] **Step 7: Commit**

```bash
git commit -m "fix(parser): support Type.method external declarations (Plan 359 E4 / DIV-HTTP-LANG-1)"
```

**风险：中**（解析器改动，但 `Type.method` 形式可能其他地方也有用）。

---

### Task E5: DIV-A2R-CHAR-AT-1 — a2r char_at 类型推断

**问题：** `var c = s.char_at(i)` 不带显式 `int` 标注时，a2r 把 c 推断为 string，导致 `c = c + 32` 被转成 `format!("{}{}", c, 32)`（E0308）。当前 workaround 是 `var c int = s.char_at(i)`。

**Files:**
- Modify: `crates/auto-lang/src/trans/rust.rs`（char_at 返回类型推断）

- [ ] **Step 1: 定位 char_at 的类型推断**

Run: `grep -n "char_at\|chars().nth" crates/auto-lang/src/trans/rust.rs | head`
看 a2r 怎么处理 `s.char_at(i)` 的返回类型——应该是固定推断为 i32（因为 VM 里 char_at 返回 codepoint int）。

- [ ] **Step 2: 修：char_at 结果默认推断为 i32**

在 `self.expr` 处理 `MethodCall` 时，若方法名是 `char_at`，返回类型标注为 i32（而非沿用 string）。

- [ ] **Step 3: 验证 — string_utils.at 去掉 `int` 标注仍工作**

把 `parity/libs/string_utils/auto/string_utils.at` 里 `var c int = s.char_at(i)` 改回 `var c = s.char_at(i)`，重跑 parity 仍 22/22。

- [ ] **Step 4: Commit**

```bash
git commit -m "fix(a2r): infer char_at result as i32 (Plan 359 E5 / DIV-A2R-CHAR-AT-1)"
```

**风险：低**（单点改动，有 parity 库作回归）。

---

### Task E6: trait_advanced parity 扩大 L1

**前置：E1（默认方法继承）、E2（bounded generic）、E3（关联类型）完成后。**

**目标：** 把 trait_advanced.at 里之前因 VM-1/VM-2/LANG-1 标 L3（未跑）的子场景激活，扩到 L1，重生成仪表盘。

- [ ] **Step 1: 激活 trait_advanced.at 里被 L3 注释掉的子场景**

去掉 VM-1/VM-2/LANG-1 相关注释（如 bounded generic 函数、关联类型 spec），加测试用例。

- [ ] **Step 2: 跑 trait_advanced parity，确认新用例三向一致**

Run: `cd parity && cargo run -p auto-parity -- --auto-binary ../target/release/auto.exe run trait_advanced`
Expected: 用例数 > 10，仍 100%。

- [ ] **Step 3: 重生成仪表盘 + 更新 known-divergences**

`cargo run -p auto-parity -- report`；known-divergences 里 VM-1/VM-2/LANG-1 标 fixed。

- [ ] **Step 4: Commit**

```bash
git commit -m "feat(parity): trait_advanced expanded L1 (Plan 359 E6)"
```

---

### Phase E 出口条件

- E1 完成 → trait_advanced 实现者不再需重声明默认方法
- E4 完成 → http_client_sync 解锁，加入 L1
- E5 完成 → a2r char_at 不再需 workaround
- E2/E3 完成（可选，难度高）→ trait_advanced 接近全覆盖
- **总验收：L1 用例数从 254 显著增长，known-divergences 的 trait 段落大部分标 fixed**



- [ ] **V1: parity 仪表盘公开可访问**，L1 用例数 ≥ D1+D3 产出（≥5 个库三向绿）。
- [ ] **V2: Script-to-Ship tour 6 章全部可交互**（Run/Transpile/Compare 工作），中英双语。
- [ ] **V3: 落地页 hero 用 L1 用例**，现状声明链接真实仪表盘，无 L3 内容用 L1 措辞。
- [ ] **V4: CI parity 门禁在 master 与 PR 上运行**，p1/p2 全绿（已知 diverge 外）。
- [ ] **V5: known-divergences.md 完整**，所有未通过用例有记录与分类。
- [ ] **V6: conformance_tests.rs 注释诚实**，对外文案无误引。
- [ ] **V7: 母纲 §4 证据策略落地**——所有对外措辞可追溯到 L1/L2/L3 目录。

---

## 风险与对策（沿用母纲 §6）

- **R1 Rust 社区质疑玩具**：parity 仪表盘 + 主动公开 a2r 输出（C3）。
- **R2 async 覆盖不足**：D3 重点补，L3 诚实标注。
- **R3 parity 暴露大量 diverge**：known-divergences 公开 + 定义"可接受差异"边界。
- **R4 a2r-std 能力空洞**（rusqlite/regex/sha2/tokio 无绑定）：D 用例评估需求，必要时扩展 a2r-std；同步 HTTP 用已有 ureq 规避。
- **R5 ScriptShipView 前端复杂度**：新建组件而非改 CodeView（B1 决策），隔离风险。
- **R6 双语工作量**：EN 先行 CN 跟进，不阻塞 C/D/B 核心。
- **R7 trait/generators a2r 缺口暴露**：D2 诚实记录为 L3，是特性而非 bug——证明"主动公开边界"。

---

## 执行说明

本计划按 C1→C2→C3→D1→(D2‖D3)→B1→B2→(A1 并行)→A2 顺序推进，每个 Task 内 Steps 顺序执行。建议用 `superpowers:subagent-driven-development`：每个 Task 派一个 fresh subagent，两阶段 review。

各 Phase 出口条件独立可验收，可在任一 Phase 暂停并对外报告进展（如 C3 完成即可发布仪表盘作为"地基就位"信号，D 完成即可宣布"核心生态已验证"，不必等 A2）。
