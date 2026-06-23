# Plan 330：ash 项目独立化 + ui-iced feature 隔离

> **For Claude:** 本计划做两件事：(1) 把 `ui-iced`/`python` 从 auto-lang 的 default feature 拿掉，改为按需 opt-in；(2) 把 `auto-shell`(+`ash-core`) 从 auto-lang workspace 拆成独立 repo，内部用"一个 repo 两个 workspace"结构，为将来的 `Ash`(CLI) / `AshGUI`(AutoUI) 双版本做隔离。**前置**：`cargo build` 当前被 `crates/auto-lang/src/vm/ffi/http_server.rs`（另一 Agent 未提交的 VM HTTP 改造）阻断，Phase 1 起需要等它收尾才能编译验证。改 Cargo.toml 后 `cargo build -p auto` + `cargo build -p auto-shell`（或新 repo 内对应命令）。计划文件本身是 doc，可直接在 master 写；代码实现走专用 worktree。

## 背景（为什么做）

- `auto-lang` 当前 `default = ["with-file-history", "ui-iced", "python"]`，导致所有依赖方（含 `auto-shell`/ash）默认拖上 iced + PyO3。开发 ash 时也被迫编译 iced，拖慢构建。
- ash 未来要分两版：**Ash**（CLI/TUI，ratatui，不需要 iced）和 **AshGUI**（基于 AutoUI 组件，需要 iced）。
- Cargo 在**同一 workspace** 内对共享依赖取 feature 并集（unification）。所以无论 `default-features = false` 与否，只要同 workspace 有成员要 iced，其它成员也编 iced。
- **隔离单位是 workspace，不是 repo**。要"Ash 不编 iced、AshGUI 编 iced"，两者必须在不同 workspace。

## 关键事实（已调研）

- auto-lang workspace 中**没有任何 crate 反向依赖 auto-shell**（`grep` 确认）→ 拆出无回流，干净。
- `auto-shell` 依赖：`auto-lang`、`auto-val`（本 workspace）、`ash-core`（本 workspace）、`auto-ai-client`（独立 repo `../auto-ai`）。
- workspace 用 `resolver = "2"`。

## 目标架构

新 repo `auto-shell`（根目录**不放** `[workspace]`），内部两个独立 workspace + 一个共享 lib：

```
auto-shell/                       (repo 根，无 [workspace])
├── ash-core/                     共享 shell 逻辑（lib，path-dep，不归属任一 workspace 成员）
│   └── Cargo.toml                auto-lang = { path = "../../auto-lang/crates/auto-lang" }（无 ui-iced）
├── ash/                          ← workspace A（CLI/TUI）
│   ├── Cargo.toml                [workspace] members = ["ash-bin"]
│   ├── ash-bin/Cargo.toml        依赖 ash-core + auto-lang(无 iced) + ratatui
│   └── target/
└── ash-gui/                      ← workspace B（GUI，Phase 3 再建）
    ├── Cargo.toml                [workspace] members = ["ash-gui-bin"]
    ├── ash-gui-bin/Cargo.toml    依赖 ash-core + auto-lang(ui-iced) + auto-ui
    └── target/
```

- `cd ash && cargo build` → auto-lang 仅按 ash-bin 的 feature 解析 → **不编 iced**。
- `cd ash-gui && cargo build` → auto-lang 带 ui-iced → 编 iced。
- 两 workspace 独立 `target/`，互不统一。

## Phase 0 — 前置与调研（等 HTTP Agent）

1. **等 `http_server.rs` 收尾**，`cargo build -p auto` 恢复，否则后续都无法编译验证。
2. 确认 `auto-shell` / `ash-core` 当前的二进制/库产物与对外 API 面（`ash` bin；`ash-core` lib 导出哪些），供 Phase 2 搬迁不丢符号。

## Phase 1 — auto-lang feature 正确化（层 1，本 repo 内）

**文件**：`crates/auto-lang/Cargo.toml`、`crates/auto/Cargo.toml`、`crates/auto-shell/Cargo.toml`（临时，Phase 2 拆走前）、`crates/ash-core/Cargo.toml`。

1. `auto-lang/Cargo.toml`：`default` 去掉 `"ui-iced"` 与 `"python"`：
   ```toml
   [features]
   default = ["with-file-history"]
   ```
2. `crates/auto/Cargo.toml`：让 `auto`（需要 iced 的 UI CLI）显式 opt-in。任选其一：
   - 依赖行直接带：`auto-lang = { path = "../auto-lang", features = ["ui-iced", "python"] }`；或
   - auto 的 `default` feature 带透传：`default = ["ui-iced", "python", "with-file-history"]` + `[features] ui-iced = ["auto-lang/ui-iced"]`（已存在）等。
   - 推荐：auto 的 `default` 含这三者，保证 `cargo build -p auto` 开箱即用、行为不变。
3. `auto-shell` / `ash-core`：保持 `auto-lang = { path = "..." }` **不带** ui-iced/python（去掉 default 即自动不带入）。若它们需要 `with-file-history` 之外的默认能力，按需显式加。
4. **验证**：
   - `cargo build -p auto` 成功（iced 在）。
   - **注意**：同 workspace 内 `cargo build -p auto-shell` 仍会因统一而编 iced——这是预期的，Phase 2 拆 repo 后才真正隔离。本阶段只保证"声明正确 + auto 不回归"。
   - `cargo test -p auto-lang --lib` 不新增失败。
5. **提醒**：Phase 1 改 Cargo.toml feature 会触发 auto-lang 全量重编（feature 集变了），首次构建变慢，正常。

## Phase 2 — 拆 auto-shell 独立 repo（CLI 版）

**新建 repo** `auto-shell`（与 `auto-lang`、`auto-ai` 同级，如 `D:\autostack\auto-shell`）。

1. **建 repo 骨架**（无根 workspace）：
   - `ash-core/`（lib）：从 `auto-lang` workspace 的 `crates/ash-core/` 整体搬来；`Cargo.toml` 去掉 `[workspace]` 归属，`auto-lang`/`auto-val` 改成 path 指回 `../auto-lang/crates/...`（或 git）。
   - `ash/`（workspace A）：`Cargo.toml` = `[workspace] members = ["ash-bin"]`；`ash-bin/` 从 `crates/auto-shell/` 搬来（`src/main.rs` 等），依赖改 `ash-core = { path = "../../ash-core" }` + `auto-lang`/`auto-val` path 指回 + `auto-ai-client` path。
2. **从 auto-lang workspace 移除** `crates/auto-shell` 与 `crates/ash-core` 成员：删 `Cargo.toml` 里对应 `members` 两行。
3. **确认无反向依赖**（已调研为空）：auto-lang workspace 内不应再有谁引用 auto-shell/ash-core；若有遗漏，改掉。
4. **路径依赖方向**：auto-shell repo → 依赖 auto-lang repo（`../../auto-lang/crates/auto-lang`），单向。auto-lang repo 不依赖 auto-shell。
5. **验证**：
   - `cd D:\autostack\auto-shell\ash && cargo build` → **不编 iced**（确认：构建日志无 iced crate，或 `cargo tree -p ash-bin` 不含 iced）。
   - `cd D:\autostack\auto-lang && cargo build -p auto` 仍成功（移除 auto-shell 成员后）。
   - `ash` 二进制能跑（基本命令）。
6. **回归**：auto-lang workspace 的 `cargo test` 不新增失败。

## Phase 3 — AshGUI workspace（未来，按需）

等 AutoUI 的 GUI 版需求明确再做，本计划只占位：
1. 在 auto-shell repo 内建 `ash-gui/` workspace + `ash-gui-bin` crate。
2. 依赖 `ash-core` + `auto-lang`（**带** `ui-iced`）+ auto-ui 组件。
3. `cd ash-gui && cargo build` → 编 iced；与 `ash/` workspace 互不污染。
4. GUI 版用 AutoUI 组件直接展示 ash-core 的结果（非 ratatui 文字模拟）。

## Verification（总）

1. **Phase 1**：`cargo build -p auto` 绿；auto-lang `default` 不含 ui-iced/python。
2. **Phase 2**：`cd auto-shell/ash && cargo build` **不触发 iced 编译**（这是本计划核心验收）；`auto-lang` repo 移除两成员后 `cargo build -p auto` 仍绿；`ash` 可运行。
3. **Phase 3**（未来）：`ash-gui` 编 iced，`ash` 不编。

## 备注

- **Phase 1 前置**：等 `http_server.rs`（另一 Agent）收尾，否则无法编译验证 feature 改动。
- 隔离单位是 workspace。同 workspace 内 `default-features=false` 无效于"不编译"——必须物理分 workspace。
- 拆 repo 后，auto-shell repo 的 auto-lang/auto-val 依赖用 path（开发期）或 git（CI/发布）。`auto-ai` 已是先例。
- 每个实施阶段用专用 worktree（Phase 1 在 auto-lang repo 内，注意 sibling worktree 撞 `../auto-ai` 路径依赖的老问题——可能需在主 worktree 直接做；Phase 2 涉及新 repo，直接在新 repo 操作）。
- 与 [[autoui-vm-option-b-reroute]] 无直接耦合；本计划是工程结构治理，独立于 VM 渲染主线。
