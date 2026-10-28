---
plan_id: PLAN-876
status: execution_done        # drafting → executing → execution_done → reviewed → archived
feature_name: autos-desktop-entry-and-launch-health（桌面入口迁 auto-os + 启动健康面）
author: [DeepSeek Harness session]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 3
current_step: 11
total_steps: 13

# /auto-plan:review 结束时填写：
supersedes_spec_components: [docs/specs/auto-ui-runtime/project.md, docs/specs/auto-ui-compiler/project.md, docs/specs/repository-boundaries/project.md]
new_spec_components: [docs/specs/repository-boundaries/autos-desktop-entry.md]
touched_goals: []             # 有意留空：不改 GOAL，只做归属与健康面
affects: [repository-boundaries, auto-ui-runtime, auto-ui-compiler, auto-ui, auto-os]
depends_on: [PLAN-831, PLAN-867]   # 拆分实况与三仓边界基线；不依赖其未完成项
---

# [PLAN-876] autos-desktop-entry-and-launch-health（桌面入口迁 auto-os + 启动健康面）

> 本计划两件事同一份合同：**①把 iced 桌面入口从 auto-lang 迁成 auto-os 的一等 bin**；
> **②建立"新版桌面"的启动健康面**，并把 2026-10-28 走查暴露的启动类缺陷逐条归位
> （本计划修 / 转派 / 入债）。入口迁移是所有权归位，健康面是让"缺产物、空壳 app、
> daemon 不可达"这类环境/装配问题**一次启动就能看清**，而不是靠双击试错。

## 0. 变更摘要

2026-10-28 实机走查（本会话）在跑通新版桌面时暴露三类问题：
（a）桌面入口仍寄居语言仓——`crates/auto-lang/examples/ui_desktop.rs`（130 行）把
整个编译器层链进桌面进程；
（b）启动失败不可诊断——4 个 app 声明 `desktop_exe` 但产物缺失时宿主直接
`spawn outproc child: 系统找不到指定的路径 (os error 3)`，`auto` CLI 缺失时只有一行
`rqhost daemon spawn failed`；
（c）apps 装配面脆——子模块工作区被清空后注册表静默缩到 9 条、小组件 `faces=0`，
桌面只表现为"图标不对、没小组件"。

本计划把入口迁到 `auto-os/host/desktop`（bin `ui-desktop`，只依赖
`auto-ui-runtime` + `auto-lang-core`，**不依赖 `auto-lang`**），随迁构建配置与三处
OS 政策，新增 boot 期 `[desktop-health]` 自检与"缺产物降级"语义，并给（c）类
环境问题一套确定性判据。auto-term 的 `future_all/race`（既有债 P723-D1）、
widgets-gallery 的 `use { package ... }` 解析（P-15）**不在本计划修**，只做归位登记。

## 1. 目标

- **G1 入口归位**：auto-os 仓成为 iced 桌面入口的唯一 owner，产出 `ui-desktop.exe`；
  auto-lang 不再持有桌面产品入口。依赖闭包**不含** `auto-lang` 编译器层。
- **G2 构建可复现**：auto-os 根 workspace + `Cargo.lock` + `.cargo/config.toml`
  （`/STACK:33554432` 等既有实证参数随迁），干净 clone 后一条命令可构建。
- **G3 启动健康面**：boot 期一次性输出 `[desktop-health]` 结构化行（注册表计数、
  声明缺产物清单、app 空壳清单、shell pack 来源、storage 路径与不可解析 id、
  `auto` CLI 可达性），并可被脚本 grep 断言。
- **G4 缺产物语义定版**：`desktop_exe` 声明但产物缺失时的行为由一条可测规则决定
  （见 Q-1 推荐：非 strict 声明降级 in-proc + 明确提示；`launch: "native"` 严格失败），
  不再是未捕获的 os error。
- **G5 观察项归位**：本会话 5 类"新版桌面"问题各有 owner 与入口，不允许以
  "环境问题"结案而不留判据。

**非目标**：不修 auto-term VM `future_all/race`（走 [KNOWN-DEBT P723-D1](KNOWN-DEBT-AND-RISKS.md)）；
不修 widgets-gallery `use { package ... }` 语法解析（P-15，需独立计划）；不为 4 个 app
重建 a2r exe 产物；不改渲染/投影/MCP 协议语义；不改 vue 轨入口
`examples/desktop-host`（登记为 Q-3）；不 push、不发布、不改远端。

## 2. 架构方案

1. **归属**：桌面入口 = AutoOS 产品装配（apps 目录、storage 政策、rqhost 装配、MCP 端口），
   归 auto-os；渲染/WM/会话/投影 = `auto-ui-runtime`（auto-ui）；语言 = auto-lang-core。
   auto-lang 只保留语言轴（本计划后其 `examples/` 不再承载 OS 桌面）。
2. **依赖方向**：`auto-os/host/desktop` → `auto-ui-runtime`（`ui-iced`、`mpv-widget`）
   + `auto-lang-core`（`set_ui_host_install_hook` 沿既有示例调用）。当前示例经
   `-p auto-lang` 间接链入整个编译器层，迁移后断开。
3. **路径解析**：沿三仓约定 `env（AUTO_UI_ROOT/AUTO_LANG_ROOT）→ 组内兄弟 → 主检出`；
   worktree 组内由兄弟目录解析，禁止 junction/symlink。
4. **健康面是只读自检**：boot 期一次采样、不驻留、不改注册表；输出既可人读也
   可 `Select-String` 断言。它不替代 launch 期错误处理。
5. **迁移与兼容**：入口唯一化（删旧 example），不做 auto-lang → auto-os 的反向依赖
   shim；所有旧调用点在同一计划内扫尾，避免"两个入口"长期并存。

## 3. 技术栈

cargo/rustc（`rust-lld`、`/STACK:33554432`、`RUST_MIN_STACK`）；auto-ui-runtime
（iced 宿主、`DesktopOptions`、`run_dynamic_desktop_with_options`）；
auto-ui-compiler（`app_registry::app_registry` 扫描与 `manifest_launch_lookup`）；
PowerShell/bash 脚本 + grep 断言；AutoUI MCP（`autoui_screenshot`）作视觉证据。

## 4. 需求分析与背景调查

### 4.1 授权与轮次

- 用户 2026-10-28（本会话）明确：同意"按 `/auto-plan:new` 起'桌面入口迁 auto-os'计划"，
  并要求**把本会话其他'新版桌面'问题一并纳入跟踪**（例：apps 路径/产物问题）。
- 本计划只写合同：未改任何实现代码、未建 worktree、未推送。实机走查中的环境修复
  （apps 子模块恢复）已在会话中完成并留证，不属本计划实施。
- 待澄清 Q-1/Q-3 影响 T-06/T-07 的行为面，未获裁定前这两项不进入实施。

### 4.2 已观察事实（2026-10-28）

| # | 事实 | 证据 |
|---|---|---|
| F-1 | 桌面入口仅 130 行，只用 `auto_lang_core` + `auto_ui_runtime`；其余逻辑在库内 | [crates/auto-lang/examples/ui_desktop.rs](../../crates/auto-lang/examples/ui_desktop.rs) 全文 |
| F-2 | auto-lang 对新 auto-ui 已是 path 依赖（`../../../auto-ui/crates/*`），旧 in-tree 五包已删 | [crates/auto-lang/Cargo.toml](../../crates/auto-lang/Cargo.toml)；`887d3d279`（PLAN-831 T-06） |
| F-3 | auto-os **无根 `Cargo.toml`**（仅各 app 的 `rust-workspace/`）；无 `.cargo/` | `Test-Path` 均为 False |
| F-4 | auto-os 脚本 52 处引用 `--example ui_desktop`；docs 4 个文件引用 | `Select-String` 计数 |
| F-5 | auto-lang 内仅 example 自身 + `tests/t5_smoke.rs` 注释引用；auto-man 0 处 | `Select-String` |
| F-6 | 4 个 app 声明 `desktop_exe` 且产物全缺：launcher/tetris/klondike/minesweeper；各仓 README 明写"clone 后不附带 exe" | `pac.at` 扫描 + `Test-Path` |
| F-7 | 注册表对声明**不做产物存在性校验**（PLAN-694 裁定"声明即信，产物可缺"） | `app_registry.rs:1754` 注释与用例 |
| F-8 | `rqhost_auto_binary` 自 `current_exe` 上探 3 级找 `auto`；本次失败实因 `target/debug/auto.exe` 根本未构建 | `rqhost.rs:104-126`；`Test-Path` False |
| F-9 | 子模块工作区被清空 → 注册表 9 条/`faces=0`；恢复后 30 条/`faces=4`（图标与小组件同时恢复） | `tmp/autoos-apps-state-20261028-152823.txt`；[图标](D:/autostack/auto-lang/tmp/ui-desktop-after-apps-fix.png)、[小组件](D:/autostack/auto-os/tmp/autoui-screenshot-1793173001836.png) |
| F-10 | 桌面 storage（`~/.config/autoos/desktop-storage.json`）存 **app id**（非目录）；29 个 id 里 2 个无解析落点（`031-image-viewer`、`041-auto-edit`） | storage 读回 + manifest 比对 |
| F-11 | auto-term 每 tick 报 `future_all/race: empty or invalid future list`（既有债，新版桌面复现） | 会话启动日志；[KNOWN-DEBT P723-D1](KNOWN-DEBT-AND-RISKS.md) |
| F-12 | widgets-gallery 多条 `AUTO-USE-DIAG P-15 use 模块 '{ package' 解析失败` | 会话启动日志 |

### 4.3 现状与边界

- 三仓边界见 [repository-boundaries/project.md](../specs/repository-boundaries/project.md)
  与 [ui-repository-delivery.md](../specs/repository-boundaries/ui-repository-delivery.md)；
  桌面宿主能力面见 [auto-ui-runtime/project.md](../specs/auto-ui-runtime/project.md)。
- Stage B 已裁定"桌面宿主运行时是框架能力"（[auto-os Design 01 §2-§3](D:/autostack/auto-os/docs/design/01-stage-b-desktop-migration.md)）：
  运行时留 auto-ui，**入口委派**当年选了"本仓薄包装（a）"。本计划推进到"入口本体归 auto-os"，
  即该 ADR 的 Stage C 候选（b）的收敛形态，差别是**不新增 `auto desktop` 子命令**，
  而是 auto-os 自己的 bin。
- apps 的 id↔目录映射集中在 `apps.manifest`（id 稳定、目录可改名），因此"目录改名导致
  桌面配置失效"不成立；失效只有 id 无落点与产物缺失两类（F-6/F-10）。

## 5. 详细设计

### 5.1 目标布局与依赖

```
auto-os/
  Cargo.toml                # [workspace] members = ["host/desktop"]
  Cargo.lock                # 入库
  .cargo/config.toml        # [target.x86_64-pc-windows-msvc] rustflags=/STACK:33554432
  host/desktop/
    Cargo.toml              # [[bin]] name = "ui-desktop"
    src/main.rs             # 自 auto-lang example 迁入
```

- 依赖：`auto-ui-runtime = { path = "../../../auto-ui/crates/auto-ui-runtime", default-features = false, features = ["ui-iced","mpv-widget"] }`、
  `auto-lang-core = { path = "../../../auto-lang/crates/auto-lang-core" }`。
- main 迁入内容（逐条对应现有 example，不新增语义）：`set_ui_host_install_hook` 安装、
  `syslog::install_host_logger`、`AUTO_UI_IN_DESKTOP=1`、`AUTO_VM_STORAGE_FILE` 缺省
  落 `~/.config/autoos/desktop-storage.json`（已设不覆盖）、`--fullscreen`/`--apps-dir`
  解析、start/alive/exit 足迹桩与 panic 钩子、`DesktopOptions` 装配。
- `rqhost_endpoint` 由 `AUTO_DESKTOP_RQHOST=1` 派生（沿用现状），失败只降级不阻断。

### 5.2 构建配置随迁

- `rustflags = ["-C","link-arg=/STACK:33554432"]`：来源是 auto-lang 现配置的实证
  （桌面深递归渲染在默认 8MB 栈上起动溢出；[desktop.ps1](D:/autostack/auto-os/scripts/desktop.ps1)
  注释记录了"从 auto-os 直接 cargo run 丢旗标致起动栈溢出"的教训）。
- `RUST_MIN_STACK`（16MB）仅测试需要；bin 不依赖。
- target 目录与 `jobs` 沿用本机既有约定，避免与 auto-lang 构建互相踩缓存。

### 5.3 启动健康面（`[desktop-health]`）

boot 期一次采样，单行 `key=value` 序列 + 一行 `hint=`（脚本可 grep）：

| 键 | 来源 | 语义 |
|---|---|---|
| `apps_dir` / `apps_dir_src` | CLI/env/缺省 | 实际扫描根与来源（`--apps-dir`/`AUTO_OS_ROOT`/缺省探测） |
| `registry` / `desktop_visible` | `app_registry` | 与 `[session] app registry: N entries` 同源，不另算一遍 |
| `skip` | `apps.manifest` repo 条目的 `pac.at` 缺席数 | 形如 `skip=no_pac_at:3`（解析失败等不单独统计，不虚构计数） |
| `declared_exe_missing` | 各 spec `desktop_exe` stat | 逗号分隔 app id（当前应为 4 个） |
| `apps_empty` | `apps/*` 目录存在但无 `pac.at`，且除 `.git` 外无内容（子模块未初始化/工作区被清空） | 空壳目录数（populated=0；未初始化子模块树=24） |
| `shell_pack` / `shell_pack_src` | `ui::shell::resolve_shell_pack_dir` | env/兄弟/内嵌 |
| `storage` / `orphan_icons` | `AUTO_VM_STORAGE_FILE` + `shell.desktop.icons` | 文件路径 + **既无注册表落点也无 manifest 落点**的 id 数（`apps_empty>0` 时不给清理建议） |
| `auto_cli` | PATH → 兄弟 → 主检出 | 路径或 `missing`（缺失即 rqhost 预判失败） |

`hint=` 只在有缺口时出现，内容必须可执行（如 `run: cargo build -p auto`、
`run: cd apps/auto-minesweeper && auto build -r rust`）。自检**不**写 storage、不改注册表。

### 5.4 缺产物语义（Q-1 待裁定，推荐方案）

- 现状：`session.rs::outproc_native_exe` 返回 `spec.exe` 而不 stat（PLAN-694"声明即信"），
  缺失即 `Command::new` 失败 → toast 只有 os error。
- 实现形态（as-built）：降级只打 stderr 一行（宿主当前没有 toast 通道；SD-01 已定版），boot 自检的 `declared_exe_missing=` 是主可见面。\n- 推荐规则（最小、可测、保住显式意图）：
  - `manifest.launch == "native"` → **严格**：产物缺失 = fail-fast，toast 带
    `auto build -r rust` 提示（声明者已表态要原生，不静默换形态）。
  - 其余（仅 `desktop_exe` 声明）→ **降级**：boot 期 warn + launch 落 in-proc，
    toast 一行"原生产物缺失，已按解释态打开"。
- 落点：判定单点放 `auto-ui-runtime`（session 或 renderer 装配期二选一，实施期按
  既有测试锚定最小面），app_registry 保持"声明即信"不背运行时语义。

### 5.5 迁移与扫尾

- 删除 `crates/auto-lang/examples/ui_desktop.rs`；改写 `tests/t5_smoke.rs` 注释指向新入口。
- auto-os：`scripts/desktop.sh`/`desktop.ps1` 改为构建/运行 `ui-desktop`；
  5 个 smoke-*.sh 与 1 处注释里的 `--example ui_desktop` 换为 `-p os-desktop --bin ui-desktop`。
- 文档：auto-os 4 个文档、auto-lang `docs/design/autoui/*` 与 `docs/design/08-ui-systems.md`
  的入口描述改为新命令；跨仓指针留在 SD-04。

### 5.6 规范增量

| delta_id | add/modify/retire | target | before/after 规则 | 理由 | 验收 |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/repository-boundaries/autos-desktop-entry.md` | 新：iced 桌面入口 owner=auto-os，依赖闭包=auto-ui-runtime+auto-lang-core，禁 `auto-lang`；构建参数与三处 OS 政策归属 | 入口所有权与依赖方向必须成文，否则拆分后继续漂移 | AC-02/AC-07 |
| SD-02 | modify | `docs/specs/auto-ui-runtime/project.md` | 增：boot `[desktop-health]` 自检键集与"只读"边界；增：声明缺产物时的 launch 语义（按 Q-1 裁定） | 健康面是宿主行为，必须在 owner 的 spec 里可查 | AC-05/AC-06 |
| SD-03 | modify | `docs/specs/auto-ui-compiler/project.md` | 改：`desktop_exe` "声明即信" → "声明即意图；运行时是否降级由宿主语义决定"（不收回扫描/注册行为） | 保留注册表职责，避免把运行时策略塞回 compiler | AC-06 |
| SD-04 | modify | `docs/specs/repository-boundaries/project.md` | 改：三仓边界表补"桌面入口 = auto-os"一行与迁移实况指针 | 拆分后边界清单必须反映入口归属 | AC-01/AC-09 |
| SD-05 | retire | `docs/specs/auto-lang/project.md`（若含 `examples/ui_desktop` 入口描述） | 删：语言仓桌面入口条目，留指针到 SD-01 | 旧入口退役后不得残留"仍可用"的描述 | AC-08 |
| SD-06 | add | `docs/specs/repository-boundaries/project.md`（计划/债归属节） | 新：自 PLAN-876 起 auto-os 域的计划与债在 auto-os 仓登记（独立 `.next-id`），auto-lang 只留指针；跨仓正文放主改面仓 | 拆分后最后一处治理面（簿记归属）必须成文，否则后续计划仍回流语言仓 | AC-12 |

无 Spec 影响的说明：本计划不改渲染/投影/MCP 协议语义，故不新增协议层条目。

## 6. 测试设计

风险面：①入口迁移（构建闭包、脚本扫尾）；②launch 事务（缺产物时的分支与回落）；
③健康面判据（不能把"空壳 app"与"缺产物"混为一谈）。用下表区分"能证伪的实现"：

| Case | 关联 AC | 输入/夹具 | 入口 | 期望可观察结果 | 断言/证据 | 覆盖边界 |
|---|---|---|---|---|---|---|
| V-01 | AC-01/AC-02 | 干净 auto-os 树（无 target） | `cargo build -p os-desktop --bin ui-desktop` | 构建成功并产出 `target/debug/ui-desktop.exe`；`cargo tree -p os-desktop` 无 `auto-lang` 节点 | 退出码 + `cargo tree \| Select-String auto-lang` 为空 | 仅本机 Windows/x64；不含交叉编译 |
| V-02 | AC-03 | 正常 apps（当前 26 目录） | 新 bin + `--apps-dir` | `registry=` 与 `[session] app registry` 一致；dashboard `faces>=4` | 日志行 + MCP `autoui_screenshot` | in-proc app；不含 outproc 成功腿 |
| V-03 | AC-05 | 夹具：1 个空壳目录 + 1 个声明缺产物 app | 新 bin boot | `apps_empty=1`、`declared_exe_missing=<id>`、`hint=` 含两种建议 | 日志 grep 逐键 | 夹具为新建；实施任务 T-04 使其可跑 |
| V-04 | AC-06 | 声明缺产物 app（非 strict） | 双击/MCP launch | 不出现 `spawn outproc child` 硬错；窗口创建；toast/warn 一行 | 日志 + 窗口存在 | 按 Q-1 推荐；strict 对照见 V-05 |
| V-05 | AC-06 | `launch:"native"` 且产物缺失 | launch | fail-fast，toast 含 `auto build -r rust` | 日志/toast 文本 | 严格臂 |
| V-06 | AC-09 | storage 含 2 个无落点 id | boot | `orphan_icons=2`，且清理后为 0；桌面无 "app not found" | 日志 + 通知中心截图 | 仅 `shell.desktop.icons` 面 |

门禁与角色：

- **实施**：本会话（`work`），实施树 `D:/autostack/.wt/lang-876/{auto-os,auto-ui,auto-lang}`
  （分支 `plan-876-dev`；auto-ui/auto-os 为兄弟树，禁链接）。
- **内部检查**：由实施者在同一树自检（构建 + V-01..V-06 证据），**不替代**最终复审。
- **最终复审**：独立上下文（`review`），按 AC 逐条对代码与日志核验，判 pass/needs_fix。
- **收尾**：`merge`。
- 门禁：auto-os `cargo build -p os-desktop`（新增）；auto-ui 改动 → 该仓 `cargo test -p auto-ui-runtime` 相关模块 + `cargo check`；
  auto-lang 仅删 example → `cargo check -p auto-lang --features ui-iced` + 合并前裸 `cargo t`（AGENTS §2 merge 门禁）。
  证据须含实际命令、退出码、关键日志行与截图路径；未执行的腿标 pending，不用"全绿"代替。

## 7. 验收标准

| ID | 标准（可观察） | owner_stage |
|---|---|---|
| AC-01 | auto-os 有根 workspace + `Cargo.lock`；`cargo build -p os-desktop --bin ui-desktop` 在干净树成功并产出 `ui-desktop.exe` | work |
| AC-02 | `cargo tree -p os-desktop` 不含 `auto-lang`（编译器层）；含 `auto-ui-runtime` 与 `auto-lang-core` | work |
| AC-03 | 新入口启动后：`[desktop-health]` 的 `registry` 与 `[session] app registry` 同值；dashboard `faces>=4`；MCP 截图有图标与小组件 | work |
| AC-04 | 无 `auto.exe` 时 boot 日志给出可执行提示（`cargo build -p auto`），桌面正常进入且不 panic；有 `auto.exe` 时出现 `rqhost daemon spawned` | work |
| AC-05 | 夹具下 `[desktop-health]` 逐键正确：`apps_empty` / `declared_exe_missing` / `orphan_icons` / `auto_cli` 各自命中预期值，且 hint 可执行 | work |
| AC-06 | 缺产物语义按 Q-1 裁定实现：非 strict 降级 in-proc 有 warn/toast；`launch:"native"` 严格 fail-fast 且提示含 `auto build -r rust`；单测覆盖两臂 | work |
| AC-07 | auto-os 仓内**当前面**（非 archive/attachments/reports）`--example ui_desktop` 引用 = 0，唯余子模块内容（转派 `OS876-DOC-SWEEP`）；`desktop.ps1 -Track iced -DryRun` 打印 `-p os-desktop --bin ui-desktop` 与 `target/debug/ui-desktop.exe`；051 launcher 测试全绿 | work |
| AC-08 | auto-lang 仓无 `examples/ui_desktop.rs` 且无残留引用；`cargo check -p auto-lang --features ui-iced` 绿 | work |
| AC-09 | `shell.desktop.icons` 中无落点 id = 0（或每个都有 manifest 落点），桌面不再出 "app not found" 通知 | work |
| AC-10 | 本会话 5 类观察项各有归位：①入口 F-1..F-5（本计划）；②缺产物 F-6/F-7（本计划）；③空壳 apps F-9（健康面 + 文档）；④auto-term F-11（KNOWN-DEBT 指针更新）；⑤use-diag F-12（转派条目）。复审可逐条对账 | review |
| AC-11 | 清理与归档：worktree 组按 wt-guard clean 后移除；spec delta 落地并 `python scripts/spec-index.py` 通过 | merge |
| AC-12 | 计划/债归属成文：auto-os 域计划在 auto-os `docs/plans/` 取号（`.next-id` 052 起）与归档，auto-lang 只留指针；auto-os `AGENTS.md` §1 有生效规则、`docs/plans/autos-desktop-program.md` 有 PLAN-876 指针行、边界 spec 有归属节 | work |

## 8. 执行步骤

| 任务 | owner_stage | 依赖 | 变更点（文件/符号，已对仓核验） | 产出 | 验证 |
|---|---|---|---|---|---|
| T-01 | work | — | 新建 `auto-os/Cargo.toml`（workspace）、`auto-os/.cargo/config.toml`、`auto-os/Cargo.lock` | 可构建骨架（空成员占位） | `cargo metadata --no-deps` 列出成员；`cargo check` 通过 |
| T-02 | work | T-01 | 新建 `auto-os/host/desktop/{Cargo.toml,src/main.rs}`；迁入 `ui_desktop.rs` 全部行为；`[[bin]] name="ui-desktop"` | `ui-desktop.exe` 可启动 | V-01、V-02 |
| T-03 | work | T-02 | `auto-os/host/desktop/src/main.rs` 或 `auto-ui-runtime` 的 `rqhost_auto_binary`（按最小面）加 PATH/SDK 解析 + 可执行提示 | AC-04 | 无 `auto.exe` 与有 `auto.exe` 两态日志 |
| T-04 | work | T-02 | `auto-os/host/desktop/src/health.rs`（新）+ 读取 `app_registry`/`ui::shell`/storage 的只读接口 | `[desktop-health]` 行 | V-03（需建夹具 app 目录：`host/desktop/tests/fixtures/apps/`） |
| T-05 | work | T-02 | `auto-os/scripts/desktop.ps1`/`desktop.sh` + 5 个 `smoke-*.sh` + 1 处注释；auto-os docs 4 处 | 脚本与文档指向新 bin | AC-07 |
| T-06 | work | T-02, Q-1 | `auto-ui-runtime` 缺产物判定单点（session/renderer 之一）+ 单测 | AC-06 | V-04、V-05 |
| T-07 | work | T-05, Q-3 | 删除 `crates/auto-lang/examples/ui_desktop.rs`；改 `crates/auto-lang/tests/t5_smoke.rs` 注释；auto-lang 文档指针 | auto-lang 不再有桌面入口 | AC-08 |
| T-08 | work | T-04 | `auto-os/apps.manifest` 与桌面 storage：处置 `031-image-viewer`/`041-auto-edit`（清理或补落点，按 Q-5） | AC-09 | V-06 |
| T-09 | review | T-01..T-08 | 独立复审：AC 逐条对代码/日志；遗漏与延后扫描；spec delta 终稿 | 复审记录 | AC-10 |
| T-10 | merge | T-09 | 三仓合并、spec 落地、债表更新（P723-D1 指针、use-diag 转派）、worktree 清理归档 | 归档与收据 | AC-11 |
| T-11 | work | — | auto-os `AGENTS.md` §1 生效规则 + `docs/plans/autos-desktop-program.md` 指针行 + auto-lang 边界 spec 归属节（SD-06） | 治理规则成文 | AC-12 |
| T-12 | work | T-05 | 051 三仓 launcher 与 876 语义 reconcile：`desktop.ps1/sh` iced 轨改 `-p os-desktop --bin ui-desktop`、产物 `target/debug/ui-desktop`（含 candidate 模式）、`scripts/tests/test_desktop_launcher.py` 断言更新 | 与 main 无冲突且新入口可用 | AC-07 |
| T-13 | work | 终审 | 终审 findings 闭环：F-01 文档面、F-02 orphan 语义、F-03 `skip` 键、F-04 夹具腿、F-06 假警报/可见面、F-07 `auto.cmd`/3 级/单测、F-08 债指针 | 终审 9 条全部关闭或有据转派 | AC-05/06/07/10/12 |

### 执行证据（2026-10-28 work 轮；worktree 组 `.wt/lang-876/`）

| 任务 | 结果 | 证据（命令 → 实际观察） |
|---|---|---|
| T-01 | [x] | `auto-os/Cargo.toml`（workspace member `host/desktop`）+ `.cargo/config.toml`（`/STACK:33554432`、`jobs=12`）+ `Cargo.lock` 入库；`cargo metadata --no-deps` 列出 `os-desktop` |
| T-02 | [x] | `cargo build -p os-desktop --bin ui-desktop` → **exit 0**，`target/debug/ui-desktop.exe` 120.5MB；启动日志 `[ui-desktop] start pid=… apps_dir=…` |
| T-03 | [x] | `rqhost::resolve_auto_cli`（`AUTO_BIN/AUTO_CLI` → 自身及上两级 → `PATH`）+ 可执行错误文案；无 `auto.exe` 时日志 = `未找到 auto 可执行文件…先 cargo build -p auto`，桌面正常进入 |
| T-04 | [x] | `[desktop-health] apps_dir=… apps_dir_src=cli registry=30 desktop_visible=29 declared_exe_missing=028-launcher,038-minesweeper,037-klondike,036-tetris apps_empty=0 shell_pack=… shell_pack_src=resolved storage=… orphan_icons=2 auto_cli=missing` + 3 条 `hint=`；`cargo test -p os-desktop` → **3 passed**（空壳签名/行形态/全绿无 hint） |
| T-05 | [x] | 10 个脚本（`desktop.ps1/sh` + 7 个 `smoke-*.sh`）改指 `-p os-desktop --bin ui-desktop` 与 `$OS_ROOT/target/debug/ui-desktop.exe`；auto-os 内 `--example ui_desktop` 计数 = **0** |
| T-06 | [x] | `outproc_native_exe` 改为"声明且在场"；新增纯函数 `decide_native_launch`（Outproc/Inproc/StrictMissing）；`cargo test -p auto-ui-runtime --lib -- p876` = **1 passed**、`native_exe_discovery_order` = **1 passed**；实机 MCP `launch 038-minesweeper` → 解释态窗口打开（不再 `os error 3`），截图 `/.wt/lang-876/auto-os/tmp/autoui-screenshot-1793174241066.png` |
| T-07 | [x] | `git rm crates/auto-lang/examples/ui_desktop.rs`；`cargo metadata` 的 auto-lang example 目标无 desktop 项；引用改指 = `t5_smoke.rs`、`acceptance_channel.py`（含 `AUTO_OS_ROOT` 解析）、`host-smithay/render_shell_frame.py`、`028-launcher` 两驱动 + SPEC、`capability-tests/README`、`docs/specs/auto-lang/ui/overview.md` |
| T-08 | [x] | storage 备份 `.bak-p876-20261028-155847` 后清理 `031-image-viewer`/`041-auto-edit`；重启自检 `orphan_icons=0`，桌面无 "app not found"（截图 `autoui-screenshot-1793174401270.png`） |
| SD-01..04 | [x] | 新增 `docs/specs/repository-boundaries/autos-desktop-entry.md`；`auto-ui-runtime`、`auto-ui-compiler`、`repository-boundaries` 三页追加 PLAN-876 节 |
| SD-05 | [x] | `docs/specs/auto-lang/project.md` 检索 `ui_desktop|examples/` = 0 命中 → **无可退役描述**（no-op），非延后 |
| T-11 | [x] | auto-os `AGENTS.md` §1 生效规则 + `docs/plans/autos-desktop-program.md` 指针行（`desktop-entry-ownership（auto-lang PLAN-876）`）+ auto-lang 边界 spec「计划与债的登记归属」节；提交 `b71cf9c` / `5c3b21f9a` |
| SD-06 | [x] | 同 T-11 的边界 spec 节（auto-os 域计划/债在 auto-os 登记，auto-lang 只留指针） |

提交（三仓同分支 `plan-876-dev`）：auto-os `f8860f2`、auto-ui `fa06499`、auto-lang `4628b3a5e` + spec `92db63f53`。
基线：auto-lang `df54eb583`、auto-ui `7095f53`、auto-os `d585fae`。
运行态：新宿主（`ui-desktop.exe`，worktree 构建）当前在跑（PID 56788），registry 30 / faces 4 / orphan 0。
未执行：merge-owned 的裸 `cargo t`、spec 账本索引刷新、三仓合并与 worktree 清理。

## 9. 复审记录

### 2026-10-28 /auto-plan:review 独立终审（review_scope: final）

- `outcome: needs_fix`（复审者=独立子会话，未采信执行者自述；三仓复审期 clean）。
- 基线：auto-lang `5c3b21f9a` / auto-os `b71cf9c` / auto-ui `fa06499`。
- AC 结论：AC-01/02/03/08/09/12 **pass**；AC-04/05/06/10 **partial**；AC-07 **fail**；
  AC-11 merge-owned pending。
- Findings：
  - `F-876-R-01`（P2，AC-07）：auto-os 内 `--example ui_desktop` 命中 10 文件，3 处为
    当前面且给出失效命令（`docs/design/01-stage-b-desktop-migration.md:153/231`、
    `apps/auto-syslog/README.md:21`、`ui-gallery/src/front/registry.at:43`）；T-05 的
    "docs 4 处"实际 0 处。
  - `F-876-R-02`（P2，AC-05/SD-01）：`orphan_icons` 只比对扫描 entries，未初始化子模块
    的检出把 24 个合法 id 判 orphan，并给"清理 storage"破坏性建议。
  - `F-876-R-03`（P2）：§5.3 `skip` 键未实现，SD-01 键集静默缩水；单测只断言子串。
  - `F-876-R-04`（P2）：`supersedes_spec_components: []` 与 SD-02/03/04 的 modify 面不符。
  - `F-876-R-05`（P3）：V-03 夹具腿未执行（无 `apps_empty>=1` 的 hint 证据）。
  - `F-876-R-06`（P3，AC-06）：降级无用户可见 toast（只有 stderr）；`launch:"vm"` +
    产物在场会打假"missing"警报。
  - `F-876-R-07`（P3，AC-04）：正向腿（有 `auto.exe` → `rqhost daemon spawned`）未证；
    `resolve_auto_cli` 无单测、不认 `auto.cmd`、上探 3→2 级。
  - `F-876-R-08`（P2，AC-10）：④P723-D1 无 876 指针、⑤use-diag 转派条目不存在。
  - `F-876-R-09`（P1，merge 阻塞）：auto-os main（051）的 launcher 仍构建已被删除的
    auto-lang example，与本分支在 `desktop.ps1/sh` 冲突。

### 2026-10-28 revision 3（REV2 修复闭环）

| Finding | 修复 | 证据 |
|---|---|---|
| F-01 | `docs/design/01-stage-b-desktop-migration.md` 两处改新命令（旧命令降为历史注记并去字面量）；`ui-gallery/src/front/registry.at` 内嵌 SPEC 命令改新；`apps/auto-syslog/README.md` 属子模块内容不可代提交 → 转派 `OS876-DOC-SWEEP` | 当前面 grep 仅余该子模块 README；`git grep -F -- '--example ui_desktop'`（排除 archive/attachments/reports）= 1 |
| F-02 | `orphan_icons` 判据改为"∉ (entries ∪ apps.manifest repo ids)"；`apps_empty>0` 时不给清理建议；SD-01 同步 | 单测 `orphan_requires_absence_from_registry_and_manifest`；夹具 boot `registry=31/declared=5/apps_empty=1/orphan_icons=0` 且无 "clean them" hint |
| F-03 | 实现 `skip=no_pac_at:N`（manifest repo 条目 `pac.at` 缺席数；不虚构 parse_fail） | 夹具 boot 行含 `skip=no_pac_at:0`；单测改为**全键集**断言 |
| F-04 | frontmatter `supersedes_spec_components` 补三张 modify 页 | 计划 frontmatter（revision 3） |
| F-05 | 新增 `hint_when_apps_empty` 单测 + 真实夹具 boot（`apps_empty=1` → 子模块 hint） | `cargo test -p os-desktop` = 5 passed；夹具 boot 日志 |
| F-06 | warn 只在 `launch!="vm"` 且产物缺失时打；可见面按 SD-01 记录为 stderr（宿主无 toast 通道） | `cargo test --lib -- p876` = 2 passed；SD-01 已注明 |
| F-07 | `resolve_auto_cli` 恢复 3 级上探 + Windows `auto.exe/auto.cmd/auto` 候选 + `find_auto_in` 单测；正向腿用 `AUTO_BIN` 实跑 | 单测 `p876_find_auto_prefers_exe_then_cmd`；boot 日志 `rqhost daemon spawned at autodesk-rqhost-desktop-23528 (AUTO_DESKTOP_ENDPOINT published)`（其后 5s 未就绪=替身非真 daemon，已注明边界） |
| F-08 | auto-os `KNOWN-DEBT-AND-RISKS.md` 增 PLAN-876 节（OS876-APP-EXE/SUBMODULE/TERM-TICK/USE-DIAG/DOC-SWEEP）；auto-lang 同名文件增指针节 | 两文件实测读回 |
| F-09 | merge `main` → 分支，`desktop.ps1/sh` 采纳 051 结构并改 iced 轨为 `-p os-desktop --bin ui-desktop`（产物 `target/debug/ui-desktop`、candidate 同名）；launcher 测试断言同步 | `python scripts/tests/test_desktop_launcher.py` = **13 passed**；`git merge` 无残冲突 |

### 2026-10-28 revision 2（用户裁定：计划/债归属随拆分一同归位）

- 触发：用户 2026-10-28「876 完成之后，auto-os（包括桌面）相关的计划就要在 auto-os
  里独立管理，而不是在 auto-lang 里」。
- 合同变更：新增 **AC-12**（归属成文）与 **SD-06**（边界 spec 归属节）、任务 **T-11**；
  `plan_revision: 1 → 2`；其余目标/验收不变，T-01..T-08 已完成项沿用原证据。
- 实施（work 同轮补齐）：auto-os `AGENTS.md` §1 写入生效规则（本仓域计划与债在本仓
  `docs/plans/` 取号/归档，auto-lang 只留指针；跨仓正文放主改面仓）；auto-os
  `docs/plans/autos-desktop-program.md` 增 PLAN-876 指针行（并注明其为最后一份
  "入口迁移"形态的跨仓正文）；auto-lang `docs/specs/repository-boundaries/project.md`
  增「计划与债的登记归属」节。
- 依据：auto-os 早已有独立计划体系（`docs/plans/` + `.next-id`=052 + `new-plan.sh`
  + `KNOWN-DEBT-AND-RISKS.md`），且 `autos-desktop-program.md` 自 Stage B P-5 起本就
  「桌面域状态只记 auto-os 侧」——本裁定把**计划正文与债**也一并归位，消除最后一处回流。
- 提交：auto-os `b71cf9c`、auto-lang `5c3b21f9a`。
- `next`：final review（AC-10 + AC-12 一并核验）→ merge（AC-11）。

### 2026-10-28 /auto-plan:work 交接

- `stage: work`；`PLAN-876`；`plan_revision: 2`；`outcome: pass`（T-09/T-10 review/merge-owned，未完成）。
- `worktree_path`: `D:/autostack/.wt/lang-876/{auto-lang,auto-ui,auto-os,auto-down}`；`branch: plan-876-dev`（四仓同名）。
- `base_commit`: auto-lang `df54eb583` / auto-ui `7095f53` / auto-os `d585fae`；`code_commit`: `4628b3a5e`+`92db63f53` / `fa06499` / `f8860f2`。
- `task_ids`: T-01..T-08 (+SD-01..05) 完成；T-09/T-10 pending。
- `evidence`: 见上表（构建 exit 0、单元测试 3+1+1 passed、两轮实机日志与两张 MCP 截图）。
- `blockers`: 无（Q-1/Q-3/Q-5 已按用户 2026-10-28 批准执行）。
- `next_responsible_role`: **final review**（独立上下文）；其后 merge。
- 未做门禁（merge-owned，如实 pending）：auto-lang 裸 `cargo t`；spec 账本 `python scripts/spec-index.py`；三仓合并与 `wt-guard` 清理。

### 2026-10-28 /auto-plan:new 起草交接

- `stage: new`；PLAN-876；`plan_revision: 1`。
- `outcome: pass`（T-01..T-05、T-08 在既有授权内可执行；T-06/T-07 受 Q-1/Q-3 约束）。
- `next: work`，起点 **T-01**；受约束项：T-06（Q-1）、T-07（Q-3）、T-08（Q-5 仅影响处置方式）。
- 变更面：无（首版）。
- 遗留说明：本计划不新跑整档测试；auto-ui/auto-os 的门禁按第 6 节，auto-lang 合入前的裸
  `cargo t` 由 merge 阶段执行。

### 待复审确认（review 阶段填写）

- AC-10 的 5 类归位逐条对账结果；AC-06 两臂单测的实际断言文本。

## 10. 待澄清事项

| ID | 问题 | 推荐 | 影响 |
|---|---|---|---|
| Q-1 | `desktop_exe` 声明但产物缺失的行为 | 推荐"非 strict 降级 in-proc + warn；`launch:"native"` 严格 fail-fast" | T-06 / AC-06 语义 |
| Q-2 | auto-lang 旧 example 是否保留兼容副本 | 推荐**直接删除**（保留会形成 auto-lang→auto-os 反向依赖或陈旧双份） | T-07 |
| Q-3 | vue 轨入口 `crates/auto-lang/examples/desktop-host` 是否同轮迁 auto-os | 推荐**本计划不迁**，登记为后续计划（先验证 iced 轨迁移稳定） | 范围 |
| Q-4 | 新 bin 命名 | 推荐 `ui-desktop`（与你口头称的 `ui-desktop.exe` 一致） | T-01/T-02 |
| Q-5 | 两个无落点 id（`031-image-viewer`、`041-auto-edit`）处置 | 推荐：从 `shell.desktop.icons` 清理并保留注释说明；若产品要保留入口，再补 manifest 条目（需各仓 app 就绪） | T-08 / AC-09 |
| Q-6 | 健康面是否需要 MCP 只读工具（`autoui_desktop action=health`） | 推荐**先只做日志行**，MCP 工具待有验收需求再加（避免扩协议） | T-04 范围 |
