---
plan_id: PLAN-580
status: execution_done          # drafting → executing → execution_done → reviewed → archived
feature_name: automan-n2-fallback
author: [zhaopuming]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-man]           # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 7
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

1. [x] PATH 有 ninja 时用 ninja；无 ninja 有 n2 时用 n2（集成形态 A/B 实测日志为证）。
   证据：A `[ninja] build runner: D:\soft\bin\ninja.exe`；B（剥 D:\soft\bin）`[ninja] build runner: C:\Users\zhaop\.cargo\bin\n2.exe`（见 T6 证据节）。
2. [x] 两者皆无且 cargo 在：自动 `cargo install --rev b1fead52` 后用 n2（或装后仍不可达时给出含两条手动安装路径的 Err）。
   证据：install-arm 探针——真实执行 `cargo install --locked --git ... --rev b1fead52`（`Replaced package n2 ... executable n2.exe`）→ 复检 `~/.cargo/bin` 命中 `resolved ninja host: C:\Users\zhaop\.cargo\bin\n2.exe`。
3. [x] 全无形态返回 `AutoResult::Err`，进程不 panic。
   证据：all-none 探针——`resolve error: no build runner found: install ninja (https://ninja-build.org), or cargo install ... --rev b1fead52, or point AUTO_NINJA at ...`，探针进程正常退出。
4. [x] 构建命令非零退出 → `auto build` 层面返回 Err（形态 C）。
   证据：util.c 语法错误 → `build runner D:\soft\bin\ninja.exe failed: exit code: 2` → auto exit 1（fail 语料双宿主 exit 1 为辅证）。
5. [x] `setup()`/`target()` 写出代码 diff 为零；unified-demo build.ninja 重新生成 `git diff` 为零（形态 D）。
   证据（带注）：写出语句/format 串零改动（diff 核对）；golden 副本重生成唯一差异 = 第 22 行 link 命令加引号（= 缺陷#3 修复本体），其余逐字节一致——"零变化"字面口径与 T5 修复互斥，登待澄清①。
6. [x] MSVC Linker/Archiver 同目录优先单测绿；`quote_if_spaced` 单测绿。
   证据：msvc_linker_archiver_prefer_compiler_dir / msvc_sibling_tool_requires_located_compiler / quote_if_spaced_wraps_only_spaced 全绿（nextest 262/262 内）。
7. [x] `cargo check -p auto-lang` 零新警告；`cargo t auto_man` 绿。
   证据（等价口径，见待澄清②）：`cargo check -p auto-man` 零错误、触及文件零警告（crate 级 17 警告=api_gen.rs 预存基线）；`cargo nextest run -p auto-man` 262/262 绿（8 skip=手动探针等）。
8. [x] 相邻缺陷 1/2/3 的修复各自有对应单测或集成证据。
   证据：#1 状态吞→形态 C（Err 上抛）；#2 spawn panic→finish() map_err（编译验证+全链路无 panic）；#3→单测 4/4b + 形态 A 生成物四工具 MSVC 同目录加引号 + 形态 D 修复行。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [x] **T1 流程基建**：master 上提交 `docs/plans/.next-id`（581）与本 plan 骨架；创建 worktree `git worktree add D:/autostack/.wt/lang-580/auto-lang -b plan-580-dev`。
  [✅ 已完成] master 1af1ebb4e 提交 plan 骨架（.next-id 已因 581/582 立项推进至 583，无需再动）；worktree 建成、status clean、分支 plan-580-dev。
- [x] **T2 runner.rs 探测核心**：新建 `crates/auto-man/src/builder/ninja/runner.rs`——`NinjaHost`/`find_in`/`exe_names`/`cargo_bin_dir`/`quote_if_spaced` + `resolve_ninja_host()` 的 1-3 级（env → ninja → n2，`OnceLock` 缓存）+ 设计节单测 1/2/3；`mod.rs` 注册 `pub mod runner;`。
  [✅ 已完成] worktree 4dfc073e9；`cargo check -p auto-man` 干净（17 警告全为 api_gen.rs 预存）；`cargo nextest run -p auto-man runner` 3/3 绿（find_in_prefers_ninja_and_falls_back_to_n2 / exe_names_windows_variants / quote_if_spaced_wraps_only_spaced）。注：计划门禁字面 `cargo t auto_man runner` 因 `cargo t` 别名硬编码 `-p auto-lang`（不会编译 auto-man）改跑等价的 `-p auto-man` 定向 nextest；另因 workspace 成员 a2r-actor-tests 的跨仓 path 依赖，组内按 lang-541/566 先例补建 auto-down 兄弟 worktree（detached da3070e）。
- [x] **T3 runner.rs 安装兜底臂**：实现 `try_install_n2()`（cargo 探测 → `cargo install --locked --git https://github.com/evmar/n2 --rev b1fead52` → PATH 与 `~/.cargo/bin` 复检 → 全失败 Err 指引文案），接入 `resolve_ninja_host()` 第 4-6 级；错误文案按详细设计节。
  [✅ 已完成] 兜底臂与 T2 同文件一次成文（worktree 4dfc073e9）；`cargo check -p auto-man` 过；本机取证（--ignored 探针 probe_resolve_logs_host，worktree 6c70edf5c）：正常 PATH → `resolved ninja host: D:\soft\bin\ninja.exe`（第 2 级）；PowerShell 剥 `D:\soft\bin` 后 → `resolved ninja host: C:\Users\zhaop\.cargo\bin\n2.exe`（第 3 级命中，Git Bash 的 PATH 覆盖对本 harness 的原生子进程不可靠，故用 PS 脚本；探针脚本在组目录 _probe/ 不入仓）。
- [x] **T4 builder.rs 接线**：`finish()` 按详细设计节替换硬编码调用——`resolve_ninja_host()?` + spawn `map_err`（灭 panic）+ `status.success()` 检查（灭状态吞）。
  [✅ 已完成] worktree 37e90c2da；`cargo check -p auto-man` 零错误 + `cargo nextest run -p auto-man` 260/260 绿（8 skip=手动探针等）。终态错误文案 `build runner {} failed: {}`（取详细设计节字样）。
- [x] **T5 resolver.rs MSVC 修复**：`resolve_executable` 增 MSVC Linker/Archiver 同目录优先分支 + `setup()` 四工具路径套 `quote_if_spaced` + 单测 4（tempdir fixture）。
  [✅ 已完成] worktree 4f0a086b1；`resolve_executable` 前置 MSVC 分支（`msvc_sibling_tool` 纯函数：编译器裸名守门 + 同目录存在检查）+ `setup()` 四处渲染参数套 `quote_if_spaced`（写出语句/format 串零改动，G4 语义保持）。单测 4 调整为 Env 位置+绝对 cl.exe 钉定形态（Dir 位置旧逻辑本就无条件 join 同目录、区分不出优先级，红→绿实证），另补 4b 裸名守门测试。`cargo check -p auto-man` 零错误；`cargo nextest run -p auto-man resolver` 42/42 绿；全套 262/262 绿。
- [x] **T6 集成矩阵**：在 worktree 跑测试设计节形态 A/B/C/D（B 用临时 PATH 子 shell；C 用 `scratch/n2_probe/fail` 拷进临时目录；D 重新生成 unified-demo 后 `git diff examples/unified-demo/build/build.ninja`）。记录四形态日志到本文件复审记录前。
  [✅ 已完成] 四形态全中 + 加测验收 2/3（详见 T6 证据节）。载体调整：B/C 用 PowerShell 子进程控 PATH（Git Bash 的 PATH 覆盖对本 harness 原生子进程不可靠）；C 主证=真编译失败（cl exit 2→Err），fail 语料双宿主直跑 exit 1 为辅证；D 因 build/ 未跟踪改为 golden 副本字节对比（唯一差异=缺陷#3 修复行，见待澄清①）。`cargo nextest run -p auto-man` 全套 262/262 绿。
- [x] **T7 收尾健康检查**：`cargo check -p auto-lang` 零新警告、无新增 `println!`、`cargo fmt` 涉及文件过闸；对照验收标准 1-8 逐条打勾并留证据行。
  [✅ 已完成] worktree cc96f6932：check 零错误+触及文件（runner/builder/resolver/mod）零警告（crate 级 17=预存基线）；新增生产代码零 `println!`（runner.rs 全 log:: 宏，探针测试内 println 为取证本体）；触及文件新增行全过 rustfmt（预存格式漂移不动，避免范围膨胀）；验收 1-8 全勾（见验收标准节证据行）。附注：auto-man 测试套件每次运行会副作用弄脏 examples/rust-workspace/{Cargo.toml,015-notes}（vue 测试再生成+剪缺席成员），非本计划改动，已 checkout 还原、不入分支。

## T6 集成矩阵证据（2026-09-07，worktree auto.exe + PowerShell 环控；完整日志 `D:/autostack/.wt/lang-580/_probe/form-{A,B,C,D}.log`）

- **形态 A（PATH 有 ninja）**：t6app（scene c，MSVC env）`auto build` → `[ninja] build runner: D:\soft\bin\ninja.exe`（resolver 第 2 级）；cl/link 真编译链接 3 任务 `[3/3] Linking main.exe`，exit 0。生成 build.ninja 四工具全部解析到 MSVC 同目录 `"C:/Program Files/Microsoft Visual Studio/.../bin/Hostx64/x64/{cl,ml64,link,lib}.exe"` 且加引号——**缺陷#3 两半（同目录优先+加引号）实战生效**。
- **形态 B（剥 D:\soft\bin，n2 在）**：`build runner: C:\Users\zhaop\.cargo\bin\n2.exe`（第 3 级命中），`n2: ran 3 tasks, now up to date`，exit 0；产物 obj/exe 尺寸与 A 逐一相同（967/619/9728 字节）。跨宿主字节差异归因：cl 内嵌时间戳（ninja-vs-ninja 同秒对照亦差 1-2 字节）+ automan 既有 HashSet 扫描序（link 输入序 `main.obj util.obj` vs `util.obj main.obj` 放大 exe 字节差，与宿主无关）；echo 语料（scratch/n2_probe/probe 拷贝）同 build.ninja 背靠背直跑 ninja/n2 → main.o/util.o **逐字节 IDENTICAL**（确定性内容下宿主字节级一致，与调研期结论吻合）。
- **形态 C（失败传播）**：util.c 注入语法错误 → `error C2059` → ninja exit 2 → `Error: build runner D:\soft\bin\ninja.exe failed: exit code: 2` → auto exit 1（旧代码此处吞错报成功）；字面 fail 语料（`cmd /c exit 1` 规则）双宿主均 exit 1。
- **形态 D（生成稳定性）**：unified-demo golden 副本重生成（native 路径触发，环境类同 golden：cl 不在 PATH、Git link 在）→ 与 golden **唯一差异 = 第 22 行 link 命令加引号**（`C:/Program Files/Git/usr/bin/link.exe` → `"..."`），即缺陷#3 修复本体；其余逐字节一致。字面 `git diff examples/unified-demo/build/build.ninja` 因 build/ 整体未跟踪而恒空，已改用 golden 副本字节对比（更强口径）。
- **验收 2 加测（安装兜底臂）**：PATH 仅含 cargo shim（无 ninja/n2）→ 真实执行 `cargo install --locked --git https://github.com/evmar/n2 --rev b1fead52`（日志 `Replaced package n2 ... executable n2.exe`）→ 复检经 `~/.cargo/bin` 显式探测命中 → `resolved ninja host: C:\Users\zhaop\.cargo\bin\n2.exe`。
- **验收 3 加测（全无形态）**：PATH 剥 ninja/n2/cargo → `resolve error: no build runner found: install ninja (https://ninja-build.org), or `cargo install --locked --git https://github.com/evmar/n2 --rev b1fead52`, or point AUTO_NINJA at a ninja-compatible executable`——含两条手动安装路径，无 panic，探针进程正常退出。

## 复审记录

## 待澄清事项

- **形态 D"零变化"与 T5 修复互斥（执行实证）**：golden 第 22 行（Git 含空格 link 路径未加引号）本身即缺陷#3 的实证样本，T5 加引号修复后重生成必改该行——实测唯一差异恰为此行（其余逐字节一致）。建议复审时把验收 5 的"build.ninja 重新生成 git diff 为零"口径修正为"除缺陷#3 修复行外零变化"，并将主检出 `examples/unified-demo/build/build.ninja`（未跟踪研究残留）决定去留。
- **验证命令的字面/等价口径**：计划各步写的 `cargo check -p auto-lang` 与 `cargo t auto_man <filter>` 因 `cargo t` 别名硬编码 `-p auto-lang`（不编译 auto-man）已等价替换为 `cargo check -p auto-man` + `cargo nextest run -p auto-man <filter>`；另因 workspace 成员 a2r-actor-tests 的跨仓 path 依赖，组内按 lang-541/566 先例补建 auto-down 兄弟 worktree（detached da3070e，merge 时随组清理）。
- **跨宿主真工具链产物字节级一致不可达（登 KNOWN-DEBT 候选）**：cl 产物内嵌时间戳、automan srcs 扫描为 HashSet 无序（link 输入序不稳定，预存行为）；宿主一致性以"同任务集+同尺寸+echo 语料字节级一致"为准。srcs 扫描序稳定化如需做，另案立项。
