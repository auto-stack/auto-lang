---
plan_id: PLAN-580
status: executing              # drafting → executing → execution_done → reviewed → archived
feature_name: automan-n2-fallback
author: [zhaopuming]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-man]           # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 7
---

# [PLAN-580] automan ninja 后端备用构建器：ninja.exe 优先、n2 兜底探测链

## 变更摘要

NinjaBuilder 对 ninja 的调用从硬编码裸名 `Command::new("ninja")`（`crates/auto-man/src/builder/ninja/builder.rs:442`）升级为**三级探测回退链**：PATH 上的 `ninja(.exe)` → PATH 上的 `n2(.exe)` → 一次性 `cargo install --locked --git`（钉 rev `b1fead52`）安装 n2。n2 是 ninja 原作者 Evan Martin 的 Rust 重实现（Apache-2.0，未上 crates.io），2026-09-07 已在本机用 automan 实际生成的 build.ninja 特性子集与真 ninja 对拍验证**行为完全一致**（探针证据 `scratch/n2_probe/`）。

同时修复同一路径上的三个现存缺陷（调研时实证，ninja/n2 下同样存在）：

1. **构建失败被吞**：`finish()` 里 `child.wait()` 后只 `println!` 状态就返回 `Ok(())`（builder.rs:447-451）——编译失败时 `auto build` 仍报成功。
2. **spawn 失败 panic**：`.expect("Failed to spawn ninja process")`（builder.rs:445）应转 `AutoResult` 错误。
3. **MSVC 工具误解析**：`examples/unified-demo/build/build.ninja:22` 实证 link.exe 被解析到 Git 的 `C:/Program Files/Git/usr/bin/link.exe`（PATH 上 Git coreutils 抢先），且含空格路径未加引号、命令行直接断裂。

**生成端零改动**：build.ninja 的写出逻辑（`setup()`/`target()`）一行不动，n2 兼容性以现有生成子集为准。

## 目标

- **G1 探测回退链**：`AUTO_NINJA` env 覆盖 → PATH `ninja(.exe)` → PATH `n2(.exe)` → `cargo install` 兜底；同进程决策缓存（`OnceLock`，pkg.rs `auto_detect` 先例）。
- **G2 失败传播**：构建命令非零退出 → `AutoResult::Err`；探测/spawn 失败 → 带安装指引的错误信息，不 panic。
- **G3 MSVC 工具误解析修复**：MSVC 的 Linker/Archiver 优先从编译器定位目录（cl.exe 同目录）解析；解析出的工具路径含空格时命令模板处加引号。
- **G4 生成端不变**：`setup()`/`target()` 写出代码零改动（复审 diff 核对），现有 build.ninja 样例 golden 不变。

## 架构方案

**spec 锚点**：`docs/specs/auto-man/project.md` builder 模块（cargo/ninja/tool/vue 后端）；`docs/architecture/bpbe.md`——Ninja 是 c 后端的**内部 Builder**（CMake/IAR/GHS 仅为导出器，不 spawn 进程），因此调用链收口在 NinjaBuilder 一处。

**新增 `crates/auto-man/src/builder/ninja/runner.rs`**（本 plan 唯一新文件）：

```
NinjaHost { program: String }              // 最终用于 Command::new 的程序名/绝对路径
resolve_ninja_host() -> AutoResult<NinjaHost>   // OnceLock 进程级缓存
  1. env AUTO_NINJA 非空 → 直接采用（用户显式覆盖，不探测）
  2. find_on_path("ninja")    → 命中即返回（cfg!(windows) 时探测带 .exe，下同）
  3. find_on_path("n2")       → 命中即返回
  4. find_on_path("cargo") 缺席 → Err(指引文案)
 5. cargo install --locked --git https://github.com/evmar/n2 --rev b1fead52
 6. 复检：find_on_path("n2") 或显式查 ~/.cargo/bin/n2(.exe) → 命中返回；否则 Err(指引文案)
```

- 探测函数设计为**可注入路径列表**（`find_in(paths: impl IntoIterator<Item = PathBuf>, exe: &str)`），生产包装从 `env::var("PATH")` 取——单测零全局状态污染（同进程串行测试下 `set_var` 不可用）。
- 引号工具：`pub fn quote_if_spaced(s: &str) -> String`——非空且含空格 → `"s"` 包裹；裸名/无空格原样返回（`"cl.exe" /c ...` 与 `cmd /c` 语义兼容，ninja/n2 均把 command 行交给 shell）。
- `builder.rs::finish()` 消费：`Command::new(host.program).args(["-C", dir])`；`status.success()` 为假 → `Err(format!("build runner ({}) failed: {}", host.program, status))`。
- `resolver.rs`：`CompilerKind::MSVC` 的 Linker/Archiver 解析顺序改为——编译器已定位到具体目录（`CompilerLocation::Dir` 或 Env 探测到的绝对路径）时，**先查同目录 `link.exe`/`lib.exe`**（VS 工具链布局：`VC\Tools\MSVC\<ver>\bin\Hostx64\<arch>` 下四件套同目录），命中即用；未命中再退回现行 PATH 探测。`setup()` 渲染四处工具路径（cc/as/link/ar）套 `quote_if_spaced`。

**n2 选型依据（2026-09-07 本机调研实测）**：

| 项 | 结论 |
|---|---|
| 作者/许可 | Evan Martin（ninja 原作者）/ Apache-2.0 |
| 分发 | **未上 crates.io**，只能 `cargo install --locked --git`；钉 rev `b1fead52`（本机验证过的 commit） |
| 缺失面（doc/comparison.md） | dyndep、console pool 完整语义、subninja 部分、`-n`/`-l`/`-d`/`-t`/`-w`、Windows 反斜杠特殊处理（issue #42） |
| 与生成子集交集 | **为空**：生成端仅用 4 rule + `build out: rule inputs` + 顶层变量 + `$out/$in` + `-C` 调用；样例全为正斜杠路径/裸编译器名/LF 行尾，不触 #42 |
| 对拍矩阵（`scratch/n2_probe/{probe_ninja,probe_n2,fail}`） | 冷构建拓扑序一致 / 未定义 `$ldflags $libs` 空展开产物逐字节一致 / no-op / touch 单源选择性重建 / `-C` / 失败退出码 1——**全绿** |

**边界与已知限制**（登入 KNOWN-DEBT 候选，复审时定稿）：

- n2 自报版本 0.1.0、无正式 release——钉 rev 保证复现性；上游演进需人工跟进。
- `~/.cargo/bin` 不在 PATH 的环境装完仍探测不到 → 走 Err 指引文案（提示把 cargo bin 加入 PATH 或设 `AUTO_NINJA`）。
- 无网络/无 cargo 环境自动安装失败 → 同上，错误信息含 ninja 与 n2 两条手动安装路径。
- 未来生成端加 `deps`/`depfile`（头依赖追踪）时需复验 n2 支持（comparison.md 缺失清单不含它，预期支持但未实测）。

## 技术栈

Rust（`crates/auto-man`），无新增第三方依赖；`std::process`/`std::env`/`OnceLock`（仓内既有用法）。

## 需求分析与背景调查

（取材 docs/specs/overview 与 module spec `docs/specs/auto-man/project.md`；INDEX:115 auto-man active，11 条 spec）

- **调用点唯一性**：全仓 grep 确认 spawn ninja 的位置仅 `builder/ninja/builder.rs:442` 一处；exporter（cmake/iar/ghs）只生成工程文件不执行构建，cargo/vue builder 不经 ninja。改一处即全覆盖。
- **默认后端权重**：`pac.rs:369-374` `default_builder = "ninja"`（rust 顶层语言才默认 cargo）——ninja 链路是 native C 工程的主路径，其可用性直接影响 `auto build` 开箱体验；这正是"引入 ninja 二进制依赖"痛点与本 plan 的动机。
- **仓内探测先例**（直接套用模式，不新造轮子）：
  - `builder/ninja/resolver.rs:45` `CompilerResolver::find_in_path`——`env::var("PATH")` + `env::split_paths` + `.exists()`，未命中回退裸名；
  - `pkg.rs:49` `command_exists`（where/which 子进程）与 `:36-84` `auto_detect` 的 `OnceLock` 缓存先例；
  - `crates/auto-playground/src/code_runner.rs:28` `find_exe`——`cfg!(windows)` 补 `.exe` 后缀先例。
- **相邻缺陷实证**：见变更摘要三缺陷；均在本 plan 改动函数半径内（finish()/resolver/渲染路径），一并修复成本最低。

## 详细设计

### runner.rs（新文件，约 150 行 + 单测）

```rust
pub struct NinjaHost { pub program: String }

static HOST: OnceLock<NinjaHost> = OnceLock::new();

pub fn resolve_ninja_host() -> AutoResult<NinjaHost> { /* 首次决策后缓存 */ }

// 探测核心（可注入，单测用）：
fn find_in(paths: impl IntoIterator<Item = PathBuf>, exe: &str) -> Option<PathBuf>
fn exe_names(base: &str) -> Vec<String>          // windows: [base.exe, base], unix: [base]
fn cargo_bin_dir() -> Option<PathBuf>            // home_dir()/.cargo/bin
fn try_install_n2() -> AutoResult<()>            // 探测 cargo → install --rev b1fead52 → 容忍 "already exists" 退出码
```

- `AUTO_NINJA` 语义：值视为程序名/路径直接采用，不做存在性校验（与 `CC` 惯例一致，让 spawn 错误自然暴露）。
- 安装命令固定：`cargo install --locked --git https://github.com/evmar/n2 --rev b1fead52`；已安装时 cargo 以非零退出并提示 already exists——**以安装后的复检探测结果为准**判断成败，不解析 cargo 输出。
- 错误文案（全无形态）：
  `no build runner found: install ninja (https://ninja-build.org), or `cargo install --locked --git https://github.com/evmar/n2 --rev b1fead52`, or point AUTO_NINJA at a ninja-compatible executable`

### builder.rs 改动（仅 finish()，约 -8/+10 行）

```rust
let host = super::runner::resolve_ninja_host()?;          // 替换硬编码 "ninja"
let mut child = std::process::Command::new(&host.program)
    .args(["-C", build_path.to_astr().as_str()])
    .spawn()
    .map_err(|e| format!("failed to spawn {}: {}", host.program, e))?;   // 消灭 .expect #2
let status = child.wait().map_err(...)?;
if !status.success() {                                    // 消灭状态吞 #1
    return Err(format!("build runner {} failed: {}", host.program, status).into());
}
```

### resolver.rs 改动（MSVC 同目录优先）

- `resolve_executable` 增加 MSVC 分支前置：`ExecutableType::Linker/Archiver` 时，若 compiler 自身已解析为绝对路径，取其父目录 join `link.exe`/`lib.exe`，`is_file()` 命中即返回；未命中走原 PATH 逻辑。
- `builder.rs::setup()` 渲染前对 `cc/as/link/ar` 四个 `to_astr()` 结果套 `runner::quote_if_spaced`（含空格路径加引号，消灭 #3 的断裂半边；误命中半边由同目录优先消灭）。

### 不做什么

- 不动 `setup()`/`target()` 的 build.ninja 写出逻辑（G4）。
- 不动既有 `println!` 调试输出（既有风格，避免范围膨胀）；新增代码一律 `log::` 宏（文件已 `use log::*`）。
- 不引入 n2 的 symlink-ninja 兼容形态（我们控制调用点，直接 `n2 -C` 即可）。
- 不给生成端加 `deps`/`depfile`（另案）。

## 测试设计

**单测（随代码落 `runner.rs`/`resolver.rs` `#[cfg(test)]`，零外部依赖、零全局状态）**：

1. `find_in` 注入 fixture 路径列表：含 `ninja.exe` 的目录优先命中；仅含 `n2.exe` 时命中 n2；全空 None。
2. `exe_names`：windows 形态含 `.exe` 变体。
3. `quote_if_spaced`：`cl.exe` 原样、`C:/Program Files/x/link.exe` 加引号、空串原样。
4. MSVC 同目录解析：tempdir 造 `cl.exe`+`link.exe`+`lib.exe` 假文件（探测只查存在不执行），断言 Linker/Archiver 返回同目录路径；同目录无 link.exe 时回退注入 PATH。
5. 缓存语义不单测（OnceLock 行为，靠集成验证）。

**集成验证（T6 手工矩阵，不进 cargo test——CI 不保证 PATH 有 ninja/n2）**：

| 形态 | 构造 | 期望 |
|---|---|---|
| A 现状 | PATH 有 ninja（`/d/soft/bin`） | 用 ninja，行为与现在一致 |
| B 回退 | 临时 PATH 去掉 ninja 目录（n2 在） | 日志显示用 n2，构建产物一致 |
| C 失败传播 | `scratch/n2_probe/fail`（exit 1 规则） | runner 报 Err（退出码 1 传播） |
| D 生成不变 | 重新生成 unified-demo build.ninja 后 `git diff` | 零变化 |

**门禁**（Category B 局部 Rust 改动）：`cargo check -p auto-lang` → `cargo t auto_man`（模块档）；不触 VM/编译器/transpiler，不跑 tv/tt/taa。

## 验收标准

1. [ ] PATH 有 ninja 时用 ninja；无 ninja 有 n2 时用 n2（集成形态 A/B 实测日志为证）。
2. [ ] 两者皆无且 cargo 在：自动 `cargo install --rev b1fead52` 后用 n2（或装后仍不可达时给出含两条手动安装路径的 Err）。
3. [ ] 全无形态返回 `AutoResult::Err`，进程不 panic。
4. [ ] 构建命令非零退出 → `auto build` 层面返回 Err（形态 C）。
5. [ ] `setup()`/`target()` 写出代码 diff 为零；unified-demo build.ninja 重新生成 `git diff` 为零（形态 D）。
6. [ ] MSVC Linker/Archiver 同目录优先单测绿；`quote_if_spaced` 单测绿。
7. [ ] `cargo check -p auto-lang` 零新警告；`cargo t auto_man` 绿。
8. [ ] 相邻缺陷 1/2/3 的修复各自有对应单测或集成证据。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] **T1 流程基建**：master 上提交 `docs/plans/.next-id`（581）与本 plan 骨架；创建 worktree `git worktree add D:/autostack/.wt/lang-580/auto-lang -b plan-580-dev`。
  验证：`git -C D:/autostack/.wt/lang-580/auto-lang status` clean 且分支 plan-580-dev。
- [ ] **T2 runner.rs 探测核心**：新建 `crates/auto-man/src/builder/ninja/runner.rs`——`NinjaHost`/`find_in`/`exe_names`/`cargo_bin_dir`/`quote_if_spaced` + `resolve_ninja_host()` 的 1-3 级（env → ninja → n2，`OnceLock` 缓存）+ 设计节单测 1/2/3；`mod.rs` 注册 `pub mod runner;`。
  验证：`cargo check -p auto-lang && cargo t auto_man runner`。
- [ ] **T3 runner.rs 安装兜底臂**：实现 `try_install_n2()`（cargo 探测 → `cargo install --locked --git https://github.com/evmar/n2 --rev b1fead52` → PATH 与 `~/.cargo/bin` 复检 → 全失败 Err 指引文案），接入 `resolve_ninja_host()` 第 4-6 级；错误文案按详细设计节。
  验证：`cargo check -p auto-lang`；本机已装形态下 `resolve_ninja_host()` 走第 3 级返回 n2（日志证据）。
- [ ] **T4 builder.rs 接线**：`finish()` 按详细设计节替换硬编码调用——`resolve_ninja_host()?` + spawn `map_err`（灭 panic）+ `status.success()` 检查（灭状态吞）。
  验证：`cargo check -p auto-lang && cargo t auto_man`。
- [ ] **T5 resolver.rs MSVC 修复**：`resolve_executable` 增 MSVC Linker/Archiver 同目录优先分支 + `setup()` 四工具路径套 `quote_if_spaced` + 单测 4（tempdir fixture）。
  验证：`cargo check -p auto-lang && cargo t auto_man resolver`。
- [ ] **T6 集成矩阵**：在 worktree 跑测试设计节形态 A/B/C/D（B 用临时 PATH 子 shell；C 用 `scratch/n2_probe/fail` 拷进临时目录；D 重新生成 unified-demo 后 `git diff examples/unified-demo/build/build.ninja`）。记录四形态日志到本文件复审记录前。
  验证：四形态期望全中 + `cargo t auto_man` 全绿。
- [ ] **T7 收尾健康检查**：`cargo check -p auto-lang` 零新警告、无新增 `println!`、`cargo fmt` 涉及文件过闸；对照验收标准 1-8 逐条打勾并留证据行。
  验证：验收标准全勾。

## 复审记录

## 待澄清事项

（无——探测链顺序、env 名 `AUTO_NINJA`、rev 钉版 `b1fead52`、三缺陷一并修复均已在上游对话裁定；如需调整 env 名或砍掉 T5 范围，在执行前提出。）
