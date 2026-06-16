# Plan 315: Ash 任意命令补全（Three-Tier Spec + Help/Man 自动解析）

> **Status**: Draft
> **关系**: 直接延续 [Plan 297（补全系统）](297-ash-completion-system.md)。297 实现了声明式 `CompletionSpec` + `CompletionProvider` 引擎 + 内置 5 命令定义；本计划把补全**从「手写 5 个命令」扩到「任意命令」**。
> **路线图位置**: 在 [Plan 309](309-ash-remaining-features-roadmap.md) 里作为补全系统的延伸项（加一行指针）。

---

## 1. 背景与问题

### 1.1 现状（Plan 297 成果）

- 声明式 spec：`CompletionSpec` 树（`command → subcommands → flags/args`），定义在
  [`ash-core/src/completions/spec.rs`](../../crates/ash-core/src/completions/spec.rs)（`CompletionSpec`/`SubcommandSpec`/`FlagSpec`/`ArgSpec`/`WhenCondition`/`CompletionSource`/`ParseMode`）。
- 纯函数引擎：`CompletionProvider::resolve()`（`provider.rs`），含子命令导航、flag 收集、位置计算、5 秒 TTL 缓存。
- 依赖注入：`command_executor`（执行外部命令取候选）+ `current_dir`。
- 内置定义：`auto-shell/src/completions/definitions/{git,cargo,docker,npm,ssh}.rs`，由 `definitions/mod.rs::register_all` 硬编码注册。
- reedline 桥：`ShellCompleter`（`completions_reedline.rs`）。

### 1.2 问题

- **每个命令都要手写一份 spec**。要支持新命令（`rg`/`fd`/`bat`/`kubectl`/…）必须改 Rust 代码、重新编译。
- `CompletionSource::Command` 只是「定义里写死要执行哪条命令取候选」，不是「自动发现命令结构」。
- 用户无法自定义/覆盖补全。

### 1.3 目标

让 ash 对**任意命令**开箱即有 flag/子命令补全，零配置；同时支持手写精修与离线生成。

---

## 2. 参考：Fish / Nushell 的做法

| Shell | 做法 | 时机 |
|---|---|---|
| **Fish** | `fish_update_completions` **离线解析 man page**（roff 源，正则）→ 生成 `~/.config/fish/generated_completions/*.fish` 静态脚本；运行时纯查表 + 动态 completer 函数 | 离线生成（手动/安装时） |
| **Nushell** | 不啃 man。靠 `def` 签名注解 + 社区补全模块 + 外部 completer | 运行时 |

**Fish 的核心 = 离线把 man/help 解析成静态配置**。本计划采用其思路，但额外加**运行时 help-probe**实现「零配置任意命令」。

---

## 3. 总体设计：三层 Spec 目录 + 优先级链

**核心思想**：三层机制产出**同一格式（.at）的 spec**，存于**不同目录**，读时按优先级查、写时各写各的。

### 3.1 三层目录

| 层 | 目录 | 谁写 | 优先级 |
|---|---|---|---|
| **用户**（user） | `~/.config/ash/completions/<cmd>.at` | 用户手写 | **最高** |
| **生成**（generated） | `~/.config/ash/completions/generated/<cmd>.at` | `completions generate <cmd>` 离线生成 | 中 |
| **缓存**（cache） | `~/.config/ash/completions/cache/<cmd>.at` | 运行时 help-probe 自动写 | 低 |

> 内置 5 命令（git/cargo/docker/npm/ssh）的 Rust 手写 spec 视同 **generated 级**（精度高、但用户 `user/` 可覆盖）。

### 3.2 读时优先级链

```
resolve(cmd):
  1. user/<cmd>.at      命中 → 用之（跳过其余）
  2. generated/<cmd>.at 命中 → 用之
  3. 内置 Rust spec     命中 → 用之（git/cargo/...）
  4. cache/<cmd>.at     命中 → 用之
  5. 都没有            → 运行时 probe（见 §5）→ 写 cache → 用之
```

任一层命中即**不再向下**。

### 3.3 写时规则

- **运行时 probe 只在 user + generated + 内置都无**时触发（有更可信的配置就不浪费 probe）。
- probe 结果写 `cache/`，**默认只写一次**（持久；重启后直接读 cache，零延迟）。
- 想刷新：`completions generate --refresh <cmd>` 重跑 `--help`/man，写到 `generated/`（覆盖 cache）。
- 引擎**永不写 `user/`**（避免污染用户手写配置）。

### 3.4 效果

- 首次对新命令 Tab → probe + 写 cache（一次性延迟）。
- 之后（含重启）→ 直接读 cache，零延迟。
- 想更准 → `completions generate`（→ generated/）或手写 `user/`（最高优先，可覆盖自动结果）。

---

## 4. Spec 格式：Auto/Atom（.at）

### 4.1 设计约束

`CompletionSpec` 深度嵌套（command → subcommands → 每层 flags/args/嵌套 subcommand；arg 还有 `source`/`when` 枚举）。现有 `auto_config` 解析器是扁平的（`block { key : "string" }`），**不够**。需要支持**嵌套对象 + 数组**的 Auto 对象字面量。

### 4.2 .at Schema（示例）

```auto
// ~/.config/ash/completions/git.at
spec {
    command : "git"
    desc    : "Git version control"
    flags   : [
        flag { long : "git-dir", takes_arg : "path", desc : "Repository path" }
    ]
    subcommands : [
        sub {
            name : "checkout"
            desc : "Switch branches"
            flags : [
                flag { short : "b", long : "branch", takes_arg : "name" }
            ]
            args : [
                arg {
                    position : 0
                    desc     : "Branch to switch to"
                    when     : "flags_absent:b,B"          // 字符串编码的 WhenCondition
                    source   : "cmd:git branch --list|line" // 字符串编码的 CompletionSource
                }
            ]
            subcommands : []                                // 可递归嵌套
        }
    ]
}
```

### 4.3 枚举的字符串编码（人工可编辑）

为保持扁平可读，`source` / `when` 用前缀字符串：

| 字段 | 编码 | 例子 |
|---|---|---|
| `source` | `static:a,b,c` / `cmd:<cmd>\|<parse>` / `files[:glob]` / `dirs` / `vars` | `cmd:git branch --list\|line` |
| `when` | `flags_present:a,b` / `flags_absent:a` / `prev:<token>` | `flags_absent:b,B` |
| `parse`（cmd 内）| `line` / `field:N` | `field:0` |

> 字符串编码的好处：人能直接读写；坏处：`source`/`when` 的值含 `:`/`|` 时需转义（实现时处理，或限定 cmd 不含这些——`CompletionSource::Command` 的 cmd 一般是简单命令，可接受）。

### 4.4 解析/序列化路径

- **解析（.at → CompletionSpec）**：用 `auto_lang::parse(text)` 把文件解析成 Auto AST，**走对象字面量 AST 节点**（不跑 VM）→ 递归构建 `CompletionSpec`。
  - 备选：若 auto-lang AST 的 object/array 字面量节点不便提取，写一个**聚焦的递归解析器**（在 `auto_config` 基础上扩展支持 `{}` 嵌套与 `[]` 数组），自包含、不耦合 auto-lang 内部。
  - **推荐**：先试 `auto_lang::parse` 走 AST；若耦合过重，退到聚焦解析器。实现期定。
- **序列化（CompletionSpec → .at）**：递归把 spec 树写成上面的对象字面量文本（probe/generate 写 cache/generated 时用）。

### 4.5 实现位置

- 新模块 `ash-core/src/completions/spec_format.rs`：`CompletionSpec <-> .at text`（序列化 + 反序列化）。
- 与 `spec.rs` 的结构体定义配合（不改动结构体，仅做转换）。

---

## 5. Help / Man 自动解析器

### 5.1 输入

- 优先 `<cmd> --help`（跨平台，Windows 也有）。
- 备选 `<cmd> help`（子命令式，如 `git help`、`docker help`）。
- Unix 可选 `man <cmd>`（roff，更结构化，Phase 3）。

### 5.2 解析启发式（`--help` 文本）

覆盖 clap / argparse / getopt / 常见手写 help 的 ~80% 格式：

1. **flag 提取**：正则匹配 `-([a-zA-Z]), --([a-zA-Z][\w-]*)`，捕获短/长 flag；同行/下一行的描述作为 `desc`；若描述含 `<...>` 或 `FILE`/`PATH`/`DIR` 等 → `takes_arg`。
2. **子命令提取**：识别 `Commands:`/`Subcommands:` 段标题，其后缩进的单词 → 子命令；递归对子命令跑 `<cmd> <sub> --help`（深度限 1-2 层，避免爆炸）。
3. **去噪**：跳过 `Usage:`/`Options:`/`Examples:` 等非命令行；过滤明显非 flag 的行。
4. **降级**：解析不出任何 flag/子命令 → 返回空 spec 并在 cache 写**空标记**（避免反复 probe）。

### 5.3 输出

`parse_help(cmd, help_text) -> CompletionSpec`（带 flags + subcommands，args 一般留空——位置参数难从 help 推断）。

### 5.4 实现位置

- 新模块 `ash-core/src/completions/help_parser.rs`：`parse_help()` + 内部正则/启发式。
- 不依赖外部 crate（用 std + 简单字符串扫描；正则用 `regex` crate 若已在 workspace，否则手写扫描）。

---

## 6. 运行时 Probe 流程（零配置任意命令）

在 `CompletionProvider`（或其上层）加一层「无 spec 时自动 probe」：

```
ShellCompleter::complete("rg --ig<Tab>"):
  provider.resolve("rg", ...) 
    → 无 user/generated/内置/cache spec
    → probe("rg"):
        run "rg --help"（via command_executor，cwd=当前目录）
        parse_help("rg", output) → spec
        spec_format::serialize(spec) → 写 cache/rg.at
    → 用新 spec resolve
```

- **同步执行**（首次 Tab 有延迟，命令大的话几百 ms）；probe 写 cache 后**永不再 probe**（持久）。
- 失败（命令不存在/`--help` 非零/解析为空）→ 写 cache 空标记 → 回退文件补全（现有行为）。
- **probe 在后台线程**可作为后续优化（Phase 1 先同步）。

### 6.1 内置 5 命令的处理

内置 Rust spec（git/cargo/docker/npm/ssh）注册时视为 **generated 级**：用户 `user/git.at` 可覆盖。实现：加载优先级链时，内置 spec 插在「generated」与「cache」之间。

---

## 7. `completions` 命令（离线生成 / 管理）

新增 builtin `completions`（注册进 `shell.rs` builtin 分发表）：

```
completions generate <cmd> [--refresh]   # 跑 <cmd> --help/man → 写 generated/<cmd>.at
completions generate --man <cmd>         # 用 man page 解析（Unix）
completions list                         # 列出 user/generated/cache 里已有的 spec
completions clear <cmd> | --cache        # 删除某命令的 cache/所有 cache
completions path                         # 显示三层目录路径
```

- `generate` = probe 的「手动 + 写 generated/」版本（覆盖 cache）。
- 写入用 `spec_format::serialize`。

---

## 8. 实现阶段

### Phase 1：运行时 help-probe（核心，零配置任意命令）

**交付**：任意命令首次 Tab 即得到 flag/子命令补全，持久缓存。

1. `help_parser.rs`：`parse_help(cmd, text) -> CompletionSpec`（`--help` 启发式 + 子命令递归 1 层）。
2. `spec_format.rs`：`CompletionSpec <-> .at`（序列化 + 反序列化）。
3. 三目录加载 + 优先级链：改 `CompletionProvider`，加 `load_user/generated/cache`、`resolve_tiered(cmd)`；内置 spec 插 generated 级。
4. 运行时 probe：无 spec 时跑 `cmd --help` → parse → 写 cache → resolve。失败写空标记。
5. 测试：parse_help 对几种 help 格式（clap/argparse）、spec 往返序列化、probe→cache→命中。

### Phase 2：离线生成 + 用户配置 + 管理命令

1. `completions` builtin（generate/list/clear/path）。
2. `completions generate` 写 generated/。
3. 用户手写 `user/<cmd>.at` 加载（覆盖一切）。
4. 启动时扫三层目录加载（惰性或全量，性能权衡）。
5. 测试 + 文档（.at spec 编写指南）。

### Phase 3：增强（可选）

- man page（roff）解析（Unix，更结构化）。
- probe 后台线程化（消除首次延迟）。
- 配置热加载（文件变化重新加载 spec）。
- subcommand 递归 probe 加深（`git checkout --help` → checkout 的 flag）。
- arg `source` 的智能推断（如 `git checkout` 的分支来自 `git branch`）——难，长期。

---

## 9. 边界与风险

| 风险 | 应对 |
|---|---|
| `--help` 格式千差万别 | 启发式覆盖 ~80%；解析为空→空标记+回退；用户可手写 user/ 覆盖 |
| 首次 probe 延迟 | 写 cache 后持久；Phase 3 可后台线程化 |
| Windows 无 man | 以 `--help` 为主，man 仅 Unix Phase 3 |
| source/when 字符串编码的转义 | 限定 cmd 不含 `\|`/`:`；实现时校验/报错 |
| 命令不存在/`--help` 卡住 | command_executor 加超时；失败写空标记 |
| 内置 spec vs user 覆盖 | 优先级链明确：user > generated > 内置 > cache |

---

## 10. 验证（每 Phase）

- `cargo build -p auto-shell -p ash-core` 通过。
- `parse_help` 单测：对 clap/argparse/getopt 三种 help 样本，正确抽出 flag/子命令。
- `spec_format` 往返单测：`CompletionSpec → .at → CompletionSpec` 等价。
- 集成测：无任何 spec 时 `rg --<Tab>` → probe + cache 写入 + 返回 flag；第二次同命令 → cache 命中（无 probe）。
- `completions generate rg` → `generated/rg.at` 生成且可加载。
- user/git.at 覆盖内置 git spec。

---

## 11. 关键文件（预估改动）

| 文件 | 改动 |
|---|---|
| `ash-core/src/completions/help_parser.rs` | **新**：`parse_help()` |
| `ash-core/src/completions/spec_format.rs` | **新**：spec ↔ .at 序列化 |
| `ash-core/src/completions/provider.rs` | 三目录加载 + 优先级链 + probe 触发 |
| `ash-core/src/completions/mod.rs` | 导出新模块 |
| `auto-shell/src/completions/definitions/mod.rs` | 内置 spec 标记为 generated 级 |
| `auto-shell/src/completions/reedline.rs` | ShellCompleter 接 probe（command_executor 已有） |
| `auto-shell/src/shell.rs` | 新 `completions` builtin 注册 |
| `auto-shell/src/completions/spec_io.rs`（或复用 auto_config） | 三目录路径解析（~/.config + APPDATA 回退，同 env.at） |
