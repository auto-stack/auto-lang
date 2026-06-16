# Plan 309: ASH 剩余功能收尾路线图

## Context

近期 ash 共有 8 篇计划（281 / 291 / 295 / 297 / 301 / 302 / 303 / 304）。经代码核实，281、297 已完成，302 / 303 基本完成，其余每篇都残留若干未做项。本计划**把所有未完成项收拢成单一总计划**，每项精确引用老计划的对应章节作为详细设计来源；**本计划全部完成后，所有老 ash 计划即可统一标记为完成**。

- 现状总览见设计文档：[docs/design/ash-design-summary.md](../design/ash-design-summary.md)
- 本计划**不重复**老计划的设计论证，只做「引用 + 任务拆解 + 验收」
- 涉及代码主体：`crates/auto-shell/src/`、`crates/ash-core/src/`

### 范围与排除

| 已完成（本计划不动） | 老计划 |
|---|---|
| 历史自动建议（autosuggestion / Ctrl+F / Ctrl+→） | 281 |
| 外部命令参数补全系统（git/cargo/docker/npm/ssh） | 297 |
| 重定向 / `&&`\|\| / alias / glob / tilde / 命令替换 / 高亮 / Vi / source / pushd-popd / `ash.toml` | 302 |
| 脚本执行 + `>` 语法 / `ash -c` / `-s` | 303 |

> 注：302、303 有极少量残留项（多行续行、统一 lexer、`let x = > cmd`），已纳入本计划 Phase 2。

---

## 老计划关闭映射

| 老计划 | 关闭条件（完成本计划哪些任务后可标记完成） |
|---|---|
| 281 | 已完成，无需本计划动作 |
| 297 | 已完成，无需本计划动作 |
| 301 | 完成 Phase 1 / Task 1.2 |
| 303 | 完成 Phase 2 / Task 2.5 |
| 302 | 完成 Phase 2 / Task 2.3 + Phase 5 / Task 5.4 |
| 295 | 完成 Phase 5（全部） |
| 304 | 完成 Phase 1–4 + Phase 6（附录 A 推迟，见下） |
| 291 | 完成 Phase 5（架构）+ 附录 A（AI / Block UX）——见附录说明 |

---

## 延续计划（独立成篇，本路线图指向其详细设计）

- **[Plan 315：Ash 任意命令补全](315-ash-arbitrary-command-completion.md)** —— 把补全从「手写 5 个命令」
  扩到「任意命令」：三层 spec 目录（user/generated/cache）+ 优先级链 + Help/Man 自动解析 + 运行时
  probe（零配置）+ `completions generate` 离线生成。直接延续 [Plan 297（补全系统）](297-ash-completion-system.md)。
  spec 格式为 Auto/Atom（.at）。

---

## Phase 1: 核心阻塞项（P0 — Daily Driver 硬阻塞）

### Task 1.1 — 真正的 OS Pipe ✅ 已完成（2026-06-15）
- **引用**：[304 §二.1 管道没有真正的 OS Pipe](304-ash-production-gap-analysis.md)；[291 §管线架构](291-autoshell-warp-design.md)
- **目标**：外部命令间用 `Stdio::piped()` 流式连接，`a | b | c` 不再串行字符串中转
- **核查结论**：实际已实现于真实执行路径 `Shell::execute_pipeline_with_auto` 的 Phase 2
  （`shell.rs:622-647`，调用 `spawn_external_chained` → `Stdio::from(prev_stdout)`）。
  external→external 走内核管道；仅 builtin/auto 涉及时才字符串缓冲（shell 的正确行为）。
  本计划原先引用的 `pipeline.rs:70-72 // TODO` 是**死代码**（仅自身测试调用）——已删除整个
  `crates/auto-shell/src/cmd/pipeline.rs` + 对应 `pub mod` / `pub use`。
- **证据**：新增测试 `cmd::external::tests::test_spawn_external_chained_os_pipe` 与
  `test_spawn_external_chained_large_volume_streaming`（20 万行双 sort 全 OS 管道，无死锁/OOM），通过。
- **验收**：✅ OS Pipe 在 external→external 链路上成立。

### Task 1.2 — 环境变量 / PATH 系统（进行中）
- **引用**：[301 整篇](301-ash-environment-variable-system.md)（§一语法 / §二持久化 / §三架构 / §四实现 Phase 1-6）；[304 §二.9 环境变量 PATH 管理不完善](304-ash-production-gap-analysis.md)
- **目标**：落地 301 的全部 6 个 Phase
  - P1 `ShellVars` 升级：`scope_stack` + `push/pop_scope` + `set_env_scoped` + 全套 `path_*` 方法（301 §3.2 / §四 Phase 1）
  - P2 `env` / `env.path` 命令族 + `EnvCommand`/`EnvPathCommand` + 表格渲染（301 §3.3 / §四 Phase 2）
  - P3 `K=V` 内联前缀解析（`NODE_ENV=prod auto build`）（301 §3.4 / §四 Phase 3）
  - P4 `~/.config/ash/env.at` 持久化 + 启动加载（301 §二 / §四 Phase 4）
  - P5 AutoLang FFI（`env()` / `env_try()` / `env_rm()` / `env.path_*`）+ `with env()` 块（301 §四 Phase 5）
  - P6 env 命令补全 + 帮助文本（301 §四 Phase 6）
- **关键改动**：`crates/ash-core/src/shell/vars.rs`（当前无任何 path_*/scope 方法）、新增 `cmd/commands/env_cmd.rs` / `env_path_cmd.rs`、`pipeline.rs` 加 `parse_env_prefixes`
- **验收**：`env.path add ~/.cargo/bin` 持久化生效；`FOO=bar auto build` 进程内 `FOO` 可见、退出后消失；`with env() { ... }` 作用域隔离

#### 进度
- ✅ **P1（ShellVars 升级）已完成**（2026-06-15）：`crates/ash-core/src/shell/vars.rs`
  加入 `scope_stack` + `push_scope/pop_scope/set_env_scoped`（**修正了 301 §3.2 的设计 bug**：原 `pop_scope` 只恢复内存 map 不恢复进程 env，子进程看不到恢复；现两者都恢复）；
  加入 `get_path_list/set_path_list/path_add/path_prepend/path_remove/path_remove_index/path_move/path_dedup/path_clean/get_path_entries`；
  新增 `AshPathEntry`/`AshEnvEntry` 结构。14 个单元测试通过（含嵌套作用域、进程 env 一致性、PATH 增删改/去重/move、跨平台分隔符）。
  > 偏离：`AtomType::EnvVarList/PathList` 枚举扩展推迟——`atom_type_to_u8`（batom.rs:1010）是无 wildcard 穷尽 match，加 variant 需配套 batom 编解码常量+arm，与表格渲染一起做更稳妥（env 表格暂用文本渲染）。
- ✅ **P2（env/env.path 命令）已完成并验证**（2026-06-15）：`crates/auto-shell/src/shell.rs` 加入 `cmd_env`/`cmd_env_path` + `format_env_table`/`format_path_table`，并在 builtin 分发表注册 `"env" | "env.path"`。
  - 子命令采用**空格形式**（`env.path add DIR`）而非 301 的点号形式——契合现有精确匹配 builtin 分发，零改动核心。
  - 验证：集成测试 `tests/env_command.rs`，10 测试通过（set/query/rm、absent→空串、rm PATH 拒绝、列表表格、env.path 表格+add/pre/rm(路径|序号)/dedup/越界报错）。
  - 注：env 表格用文本渲染（未扩展 `AtomType::EnvVarList/PathList`，避免动 batom 编解码）。
- ✅ **P3（K=V 前缀，解析+执行）已完成并验证**（2026-06-15）：
  - P3.1 解析：`ash-core/src/parser/pipeline.rs::parse_env_prefixes`（引号感知），9 单元测试。
  - P3.2 执行：`shell.rs::execute_with_env_prefixes` 包住 `execute_inner` 单一调用点——命中前缀则 `push_scope`→`set_env_scoped`→执行→`pop_scope`（唯一调用点保证所有路径都 pop）。**bash 语义**：纯赋值 `FOO=bar` 持久化，`FOO=bar cmd` 才 scoped。4 集成测试通过（scoped 可见+恢复、多前缀、纯赋值持久化、引号值）。
  - 注：`cargo test -p auto-shell --lib` 整体编不过——`each.rs/insert.rs/math_*.rs/wc.rs` 的 `#[cfg(test)]` 模块有**预先存在的**编译错误（`Obj/Array` 未导入、`ParsedArgs` 缺字段），非本次改动。env 测试改用 `cargo test --test env_command`（不编译 lib 内部 test 模块）绕开。
- ⬜ P4（持久化 `~/.config/ash/env.at`）、P5（AutoLang FFI）、P6（补全/文档）待续。

> ⚠️ **重要发现（影响路线图准确性）**：实施 P2 时发现 shell.rs 的 builtin 分发表（222-239）**已实现** `def`（shell 函数）、`hook`（事件钩子）、`abbr`（缩写）、`config`（配置命令）、`bind`（键绑定）、`path`（PATH 管理）。这与 304 §二 / 本计划 Phase 4 把它们标为「未完成」**不符**——之前的状态分析漏看了这张 builtin 分发表。建议在 Task 1.2 收尾后，重新核实 Phase 4 各项（4.1-4.5）的真实完成度，相应修订路线图。

### Task 1.3 — 错误信息上下文（部分完成）
- **引用**：[304 §二.3 错误信息缺乏上下文](304-ash-production-gap-analysis.md)
- **目标**：did-you-mean 模糊建议、统一 exit code 语义、`$?` 全命令一致
- **进度**：
  - ✅ `$? $@ $# $!` 特殊变量展开已修复（`expand_variables` 现正确路由非字母数字特殊参数；`get_variable` 早已映射）。`$_` 经普通路径可用。
  - ⬜ did-you-mean 模糊建议（输错命令名给最接近项）。
  - ⬜ 统一 exit code 语义、错误分类。
- **关键改动**：命令分发层（`shell.rs` execute 路径）+ 内置命令错误返回
- **验收**：输错命令名给出最接近建议；每条命令退出码正确传播到 `$?`

---

## Phase 2: 脚本完整性（P1）

### Task 2.1 — Here Document
- **引用**：[304 §二.5 没有 Here Document](304-ash-production-gap-analysis.md)
- **目标**：支持 `cmd <<EOF ... EOF` 与 `<<'EOF'`（禁插值）、`<<-EOF`（去前导 tab）
- **关键改动**：pipeline 预处理 / Shell Lexer 层
- **验收**：`cat <<EOF` 多行写入、`<<'EOF'` 不展开 `$var`

### Task 2.2 — Shell 函数定义 ✅ 已完成
- **引用**：[304 §二.7 没有 Shell 函数定义](304-ash-production-gap-analysis.md)
- **目标**：REPL/脚本内 `fn name(args) { ... }`（shell 层）定义后可作命令调用，支持 alias 优先级
- **核查结论**：已实现为 builtin `def`（`shell.rs::cmd_def`，注册于 builtin 分发表）。
  支持 `def name [params] { body }`，转译为 Auto `fn`；`def ll [] { ls -la }` 等。
  此前状态分析漏看了 builtin 分发表，误标未完成。
- **验收**：✅ 函数定义后可作命令调用。

### Task 2.3 — 多行续行（`\` + 引号续行）
- **引用**：[302 §Step 2.3 多行输入](302-ash-daily-driver-roadmap.md)
- **目标**：行尾 `\` 续行；未闭合引号自动续行（区别于已完成的 Ctrl+E 编辑器路径）
- **关键改动**：repl.rs 读取循环 + 续行 prompt
- **验收**：`echo a \↵ b` 输出 `a b`；未闭合 `"` 自动等下一行

### Task 2.4 — 特殊变量与语法糖（部分完成）
- **引用**：[304 §二.19 完善特殊变量和语法糖](304-ash-production-gap-analysis.md)
- **目标**：`$@ $# $_ $! $?`、brace expansion `{a,b,c}`、算术 `$(())`、`~user`
- **进度**：
  - ✅ 特殊变量 `$? $@ $# $! $_` 已可用（见 Task 1.3）。
  - ⬜ brace expansion `{a,b,c}`、算术 `$((1+2))`、`~user`（`~`/`~/path` 已有）。
- **验收**：`echo file.{txt,md}` → 两文件；`echo $((1+2))` → 3

### Task 2.5 — `let x = > cmd` 赋值捕获（可选）
- **引用**：[303 §Step 5 赋值捕获（可选增强）](303-ash-script-execution-shell-syntax.md)
- **目标**：脚本中把 `>` 行 stdout 赋给 Auto VM 变量
- **关键改动**：`shell.rs` `interpolate_auto_vars` 附近 + `>` 行模式解析
- **验收**：`let out = > ls` 后 `print(out)` 输出 ls 结果

---

## Phase 3: 数据流与框架（P2）

### Task 3.1 — 结构化数据管道激活（Atom 接入管道）
- **引用**：[304 §二.10 结构化数据管道](304-ash-production-gap-analysis.md)；[291 §Phase 0 Atom 管线 / §管线架构](291-autoshell-warp-design.md)
- **目标**：管道数据从 `ShellValue::String` 升级为 Atom（`pipeline.rs:37` 当前仍 String），让 `ls | grep | select` 类型安全
- **关键改动**：`ash-core/src/pipeline.rs` + 内置命令的 Atom 输出/消费
- **验收**：`ls | where size > 1k` 结构化过滤可行；旧字符串管道向后兼容

### Task 3.2 — 统一命令参数解析框架
- **引用**：[304 §二.8 没有命令参数解析框架](304-ash-production-gap-analysis.md)
- **目标**：Command trait 统一签名系统，`-n5`/`--num=5` 一致解析、`--help` 自动生成
- **关键改动**：`cmd/` trait 重构（影响全部 74 命令，需渐进迁移）
- **验收**：任意命令 `--help` 输出统一格式；新参数风格一致

### Task 3.3 — 命令品质加固
- **引用**：[291 §风险与缓解](291-autoshell-warp-design.md)（自评「命令实现品质不足为 MVP」）
- **目标**：JSON `\uXXXX` 转义、find `**` 递归、HTTP 原生客户端
- **关键改动**：相应命令实现文件
- **验收**：对应命令的边界用例测试通过

---

## Phase 4: 可定制性与扩展（P3）

| Task | 状态 | 引用 | 目标 |
|---|---|---|---|
| 4.1 REPL 配置命令 | ✅ 已完成 | [304 §二.11](304-ash-production-gap-analysis.md) | `config` builtin：`config list/get/set`，写入 `~/.config/ash.toml` |
| 4.2 `bind` 键绑定 | ✅ 已完成 | [304 §二.16](304-ash-production-gap-analysis.md) | `bind` builtin：`bind list`、`bind <key> <action>` |
| 4.3 Abbreviation | ✅ 已完成 | [304 §二.13](304-ash-production-gap-analysis.md) | `abbr` builtin：`-a/-r/-l`，输入时展开 |
| 4.4 事件钩子 | ✅ 已完成 | [304 §二.14](304-ash-production-gap-analysis.md) | `hook` builtin：chdir / preexec / precmd |
| 4.5 插件系统 | ⬜ 待做 | [304 §二.12 插件/扩展系统](304-ash-production-gap-analysis.md) | Fish 式函数文件自动加载 + 外部插件协议 |

> 4.1–4.4 此前被状态分析误标为「未完成」——它们其实已作为 builtin 实现（注册于 `shell.rs` builtin 分发表，各含 usage 帮助与子命令逻辑）。**Phase 4 仅剩 4.5 插件系统。**

---

## Phase 5: 架构重构（P4）

| Task | 引用 | 目标 |
|---|---|---|
| 5.1 创建 `ash-tui` crate | [295 §六 Task A4](295-ash-layered-architecture-ratatui.md) | TUI 代码（repl.rs / completions/reedline.rs / term/）迁出 auto-shell |
| 5.2 `auto-shell` 瘦身为薄壳 | [295 §六 Task A5](295-ash-layered-architecture-ratatui.md) | 仅依赖 ash-core + ash-tui，移除 crossterm/uucore/regex/unicode-segmentation |
| 5.3 创建 `ash-gui` 占位空壳 | [295 §三 目标架构](295-ash-layered-architecture-ratatui.md) | 空目录 + Cargo.toml |
| 5.4 统一 Shell Lexer | [302 §Step 4.1 统一 Shell Lexer](302-ash-daily-driver-roadmap.md) | `ShellToken` 枚举，消除各解析器重复引号处理 |

> 注：5.1/5.2 是大重构，建议在 Phase 1-4 功能稳定后做，避免迁移与功能开发并行冲突。

---

## Phase 6: 文档与性能（P6）

| Task | 引用 | 目标 |
|---|---|---|
| 6.1 文档体系 | [304 §二.20 文档和帮助系统](304-ash-production-gap-analysis.md) | 每个内建命令 `--help` 标准化、`man ash`、cookbook |
| 6.2 性能基准 | [304 §二.18 性能优化](304-ash-production-gap-analysis.md) | 启动时间 / 命令延迟 / 内存基准 + 优化 |

---

## 附录 A: 战略级大块（推迟，建议独立启动）

这两项体量大、属产品方向级，**不建议并入日常收尾**。完成它们后 291 才能完全关闭；此前 291 标记「P0-P2 完成，P3-P4 推迟」。

| 项 | 引用 | 说明 |
|---|---|---|
| A.1 AI 集成 | [291 §Phase 3 AI 智能集成](291-autoshell-warp-design.md) | LLMProvider trait、自然语言→命令、错误解释、Agent 模式、`stdlib/auto/llm.at` |
| A.2 Block UX 现代化 | [291 §Phase 4 Block UX 现代化](291-autoshell-warp-design.md) | Block 模型、现代输入编辑器、ANSI Block 渲染、Atom 感知渲染 |

---

## 建议执行顺序

1. **Phase 1**（P0，先解硬阻塞）：1.1 OS Pipe → 1.2 环境变量 → 1.3 错误上下文
2. **Phase 2**（脚本完整性）：2.1 heredoc → 2.2 函数 → 2.3 续行 → 2.4 特殊变量 → 2.5（可选）
3. **Phase 3**（数据流/框架）：3.1 → 3.2 → 3.3
4. **Phase 4**（可定制性）：4.1-4.5 独立可并行
5. **Phase 5**（架构重构）：**最后做**，避免与功能开发冲突
6. **Phase 6**（文档/性能）：贯穿，每个 Phase 完成即补 `--help`
7. **附录 A**：Phase 1-6 稳定后另起计划

---

## Verification（每完成一个 Phase）

- `cargo build -p auto-shell -p ash-core` 通过
- 相关单元/集成测试通过；新功能补测试
- 更新 [docs/design/ash-design-summary.md](../design/ash-design-summary.md) 对应状态行
- Phase 全部完成后，按「老计划关闭映射」更新各老计划头部 `Status` 为 ✅
