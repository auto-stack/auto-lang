# Plan 304: ASH → 生产级 Shell 差距分析

> **类型**: 分析文档（非实施计划）
> **Status:** ✅ 分析完成
> **对标**: Fish 3.x, Nushell 0.100+, Bash 5.x
> **目标**: 识别 ASH 从"技术演示"到"可日常使用的 Shell"的关键差距

---

## 一、总体评估

| 维度 | ASH 现状 | Fish | Nushell | 差距评级 |
|------|---------|------|---------|---------|
| 交互体验 | ★★★☆☆ | ★★★★★ | ★★★★☆ | 中等 |
| 脚本能力 | ★★★☆☆ | ★★★★☆ | ★★★★★ | 较大 |
| 数据管道 | ★★☆☆☆ | ★★☆☆☆ | ★★★★★ | 巨大 |
| 命令覆盖 | ★★★★☆ | ★★★☆☆ | ★★★★☆ | 较小 |
| 可配置性 | ★★★☆☆ | ★★★★★ | ★★★★☆ | 中等 |
| 生态/插件 | ★☆☆☆☆ | ★★★☆☆ | ★★★★☆ | 巨大 |
| 性能 | ★★★☆☆ | ★★★★☆ | ★★★★★ | 中等 |
| 跨平台 | ★★★☆☆ | ★★☆☆☆ | ★★★★★ | 较小 |

**总结**: ASH 有 75 个内建命令、Starship 风格 prompt、reedline 编辑器等良好基础。核心差距集中在 **管道系统、错误处理、脚本完整性、生态** 四个方面。

---

## 二、关键差距（按优先级排序）

### P0: 不修就不能当 Daily Driver

#### 1. 管道没有真正的 OS Pipe

**现状**: 管道通过字符串传递，不是 OS pipe。外部命令之间的管道是"执行完一个，把 stdout 字符串传给下一个的 stdin"。

```rust
// crates/auto-shell/src/cmd/pipeline.rs:70-73
// For external commands, we'll need to pipe input via stdin (TODO)
// For now, just execute without pipeline input
external::execute_external(cmd, current_dir, true)
```

**后果**:
- `ls | grep foo` 对内建命令 OK，但 `git log | head` 这种**外部→外部**管道是串行执行而非流式
- 无法处理大量输出（全部加载到内存再传）
- 无法实现实时流式处理（`tail -f | grep`）

**对标**: Fish/Nushell/Bash 都用 OS pipe（`pipe()` syscall）连接子进程。

**工作量**: 中等（需要重构 pipeline 层，使用 `std::process::Stdio::piped()`）

---

#### 2. 没有登录 Shell 模式

**现状**: ASH 只能从其他 shell 里启动（作为子进程）。不能设为 `/etc/shells` 里的默认 shell。

**缺失**:
- 没有实现 POSIX 登录 shell 行为（`-l`, `--login`）
- 没有 `chsh -s /usr/bin/ash` 支持（需要系统安装）
- 没有 `ash -c "command"` 单命令执行模式
- 没有 `ash -s` 从 stdin 读取模式
- 没有 `/etc/profile.d/` 兼容

**对标**: Fish 通过 `fish -l` 支持登录模式；Nushell 作为默认 shell 使用。

**工作量**: 小（主要是 CLI 参数增强 + RC 文件加载逻辑）

---

#### 3. 错误信息缺乏上下文

**现状**: 错误是 `miette` 格式，但 shell 命令错误只是简单字符串。

```
Error: command not found: foo
```

**应该有的**:
- 命令建议（"did you mean `foo`?" 模糊匹配）
- 退出码统一语义（内建命令的 exit code 不一致）
- 错误类型分类（权限错误 vs 命令不存在 vs 参数错误）
- `$?` 对所有命令都正确设置

**对标**: Fish 的错误信息极其友好，带高亮和建议。Nushell 有 span-based 错误定位。

**工作量**: 小到中等

---

#### 4. History 搜索不完整

**现状**:
- ✅ 文件持久化历史
- ✅ 上下箭头浏览
- ❌ Ctrl+R 反向搜索**没有激活**
- ❌ 历史去重不完善
- ❌ `!!`, `!$`, `!^` 历史展开**未激活**（代码已有，parser 在 ash-core 里）
- ❌ 多 session 历史合并

**对标**: Fish 的历史搜索是杀手级特性。`Ctrl+R` 是每个 shell 用户的基本期望。

**工作量**: 小（reedline 自带 Ctrl+R，可能只需配置；历史展开已有 parser）

---

### P1: 明显的功能缺失

#### 5. 没有 Here Document (`<<EOF`)

```bash
cat <<EOF
Hello $name
EOF
```

**影响**: 很多现有脚本和工具依赖 heredoc（特别是 cloud-init、Dockerfile、Kubernetes YAML 生成）。没有它，ASH 无法直接替换 bash 脚本。

**工作量**: 中等（需要 lexer 和 parser 扩展）

---

#### 6. 没有 Process Substitution (`<(cmd)`)

```bash
diff <(sort file1.txt) <(sort file2.txt)
```

**影响**: 无法使用高级命令组合模式。Git、SSH 等工具的某些功能依赖它。

**工作量**: 中等（需要创建临时文件/FIFO）

---

#### 7. 没有 Shell 函数定义

**现状**: ASH 没有原生的 shell 函数语法。AutoLang 的 `fn` 可以在 `>` 模式下用，但不能直接在 REPL 里定义 shell 函数。

```bash
# Fish:
function ll
    ls -la $argv
end

# Nushell:
def ll [] { ls -la }
```

**ASH 的定位选择**: 要不要支持 shell 层面的函数定义？还是只依赖 AutoLang 的 `fn`？
如果选 AutoLang 路线，需要让 `fn` 在 REPL 里定义后可以直接作为命令调用。

**工作量**: 中等（取决于设计决策）

---

#### 8. 没有命令参数解析框架

**现状**: 每个内建命令手动解析参数（字符串 split），没有统一的参数解析框架。

**对标**:
- Fish: `argparse` 内建命令
- Nushell: 自定义命令有完整签名系统（参数类型、默认值、描述、可变参数）
- Bash: `getopts`

**影响**:
- 命令参数行为不一致（`-n 5` vs `-n5` vs `--number=5`）
- 没有 `--help` 统一生成
- 添加新命令成本高

**工作量**: 中等（设计 `Command` trait 的参数声明系统）

---

#### 9. 环境变量 PATH 管理不完善

**现状**: 有基本的 `export` 和 env var 管理，但缺少：
- `PATH` 变量的列表语义（append/prepend/remove）
- `path` 命令（Fish: `path add`, `path remove`）
- `manpath` 管理
- XDG 规范支持（`XDG_CONFIG_HOME` 等）

**工作量**: 小

---

### P2: 现代 Shell 的差异化特性

#### 10. 结构化数据管道（Nushell 核心优势）

**现状**: ASH 有 Atom 类型系统（ash-core/pipeline/atom.rs），但**实际管道传递的是纯字符串**。

```rust
// pipeline.rs:37
input_data = output.map(|s| ShellValue::String(s));
```

**Nushell 的做法**: 一切都是 `Value`（类型化数据），管道传递的是结构化数据而非文本。`ls` 返回表格，`grep` 过滤行，`select` 取列——全部类型安全。

**ASH 的路径**: Atom 系统已经存在，关键是让内建命令产出到 Atom 而非 String，并让管道层传递 Atom。

**工作量**: 大（核心架构变更，但 Atom 基础设施已有）

---

#### 11. 交互式配置 UI

**现状**: 配置通过编辑 `~/.config/ash.toml` 文本文件。

**对标**:
- Fish: `fish_config` 打开 Web UI，可视化配置颜色/提示符/函数/绑定
- Nushell: `$env.config` 对象，可在 REPL 里直接修改

**建议**: 不做 Web UI，做 REPL 内配置命令：
```
ash> config set shell.edit_mode vi
ash> config set prompt.format "$directory$git$character"
ash> theme list
ash> theme set tokyo-night
```

**工作量**: 中等

---

#### 12. 插件/扩展系统

**现状**: 没有插件机制。新命令只能通过修改源码添加。

**对标**:
- Nushell: 插件是外部进程，通过 JSON/msgpack 通信
- Fish: 函数文件 `~/.config/fish/functions/` 自动加载

**建议**: 先做 Fish 式的"函数文件自动加载"（简单），再做 Nushell 式的插件协议（高级）。

**工作量**: 小（函数自动加载）到 大（插件协议）

---

#### 13. 缩写/Abbreviation 系统

**现状**: 只有 alias，没有 abbreviation。

**区别**: 
- `alias gs="git status"` — 替换命令词，回车后 `gs` 变成 `git status`
- `abbr -a gs "git status"` — **输入时** gs 展开为 `git status`，可看到展开结果

**对标**: Fish 的 `abbr` 是极受欢迎的特性。在输入行就能看到展开结果，比 alias 更安全。

**工作量**: 小（需要 reedline 集成）

---

#### 14. 事件钩子系统

**现状**: 没有事件钩子。

**对标**:
- Fish: `function --on-variable PWD`, `--on-process-exit`, `--on-event fish_prompt`
- Nushell: `hook pre_prompt`, `hook pre_execution`, `hook env_change.PWD`
- Zsh: `chpwd`, `precmd`, `preexec`

**用例**: 
- 进入目录时自动激活 Python venv
- 命令执行前记录时间
- 退出码非零时自动打印栈

**工作量**: 中等

---

### P3: 完善度和打磨

#### 15. Tab 补全的广度和深度

**现状**: 只有 `git` 和 `cargo` 的外部命令补全定义。

**需要补全的外部命令**（优先级排序）:
1. **git** ✅（已有）
2. **cargo** ✅（已有）
3. **npm/yarn/pnpm** — Node 生态
4. **docker** — 容器
5. **kubectl** — Kubernetes
6. **ssh/scp** — 远程
7. **pytest/cargo test** — 测试
8. **make/cmake** — 构建
9. **apt/brew/pacman** — 包管理

**还需要**:
- 补全描述文本（补全菜单里显示命令/参数说明）
- 补全来源的缓存和索引加速
- man page 自动解析生成补全（Fish 有这个）

**工作量**: 每个外部命令补全 ~100-200 行。核心框架改动小。

---

#### 16. 关键绑定可自定义

**现状**: 有 Emacs/Vi 模式切换，但不能自定义绑定。

**对标**: Fish/Nushell 都支持 `bind` 命令自定义键绑定。

```
bind \t complete
bind \cr history-search
bind \ew backward-kill-word
```

**工作量**: 中等（reedline 支持自定义绑定，需要暴露配置接口）

---

#### 17. 多行编辑体验

**现状**: 仅支持 `\` 续行。多行命令输入后无法编辑之前的行。

**对标**: 
- Fish: 自动检测未闭合的结构（引号、`and`/`or`），显示多行提示
- Nushell: 类似
- 最好的体验: 在编辑器中打开（Ctrl+E / Ctrl+X Ctrl+E）

**工作量**: 中等

---

#### 18. 性能优化

**潜在问题**:
- AutoVM 常驻内存（每个命令都可能触发 VM 交互）
- Prompt 渲染用 rayon 并行（启动延迟？）
- 每个命令的 AutoLang 表达式检测（`looks_like_auto_expr`）的开销
- 历史文件在大尺寸时的加载性能

**需要基准测试**: Shell 启动时间、命令执行延迟、内存占用。

**工作量**: 取决于基准测试结果

---

#### 19. 完善特殊变量和语法糖

**缺失的特殊变量**:
- `$0` — 当前脚本名
- `$1`..`$9` — 位置参数
- `$@` — 所有参数
- `$#` — 参数个数
- `$_` — 上一个命令的最后一个参数
- `!!` — 上一条命令
- `!$` — 上一条命令的最后一个参数

**缺失的语法糖**:
- Brace expansion: `{a,b,c}` → `a b c`
- Array: `(item1 item2 item3)`
- Arithmetic: `$((1 + 2))`
- Tilde 用户展开: `~user` → `/home/user`

**工作量**: 小到中等

---

#### 20. 文档和帮助系统

**现状**: 有 `help` 命令，但内容不完整。

**对标**:
- Fish: `help` 打开浏览器看完整文档
- Nushell: `help <command>` 显示详细签名和示例
- 两者都有丰富的在线文档网站

**需要**:
- 每个内建命令的 `--help` 文本标准化
- `man ash` 手册页
- 在线文档站点
- 示例集合（ASH cookbook）

**工作量**: 大（纯写作工作）

---

## 三、ASH 的独特优势（应保持和强化）

不要只看差距，ASH 也有 Fish/Nushell 没有的杀手级特性：

### 1. AutoLang 原生集成
- 在 Shell 里直接用完整的编程语言（类型系统、模式匹配、f-string 等）
- 不需要在 shell 脚本和"真正的语言"之间切换
- `>` 前缀让 shell 命令和 Auto 代码无缝混合

### 2. 模块系统
- `use` 导入模块，共享代码
- 未来可以有 ash package manager

### 3. 内建命令丰富
- 75 个内建命令，远超 Fish（~50）和 Nushell（~60）
- HTTP 客户端、数据格式转换（JSON/CSV/TOML/XML/YAML）、文本处理全覆盖

### 4. 跨平台
- Windows 一等支持（不像 Fish）
- Windows 特有的 job control（SuspendThread/ResumeThread FFI）

### 5. Atom 类型系统基础
- 虽然管道还没用到，但基础架构在 ash-core 里已经就绪
- 一旦激活，可以比 Nushell 更好（因为 AutoLang 有更强的类型系统）

---

## 四、建议的路线图

### Phase 1: Daily Driver 基础（~2-3 周）
1. **真正的 OS Pipe 管道** — 外部命令之间使用 `Stdio::piped()`
2. **Ctrl+R 历史搜索** — 激活 reedline 的历史搜索
3. **`ash -c "cmd"` 单命令模式** — CLI 参数增强
4. **命令建议** — "did you mean?" 模糊匹配
5. **PATH 列表管理** — `path add/remove` 命令

### Phase 2: 脚本完整性（~3-4 周）
6. **Here Document** — `<<EOF` 支持
7. **Shell 函数定义** — 设计并实现（或增强 AutoLang fn 的可调用性）
8. **命令参数解析框架** — 统一的 `Command` trait 签名系统
9. **特殊变量完善** — `$@`, `$#`, `$_`, `!!`, `!$`
10. **事件钩子** — `on_chdir`, `on_preexec`, `on_precmd`

### Phase 3: 差异化特性（~4-6 周）
11. **结构化数据管道激活** — Atom 系统接入管道
12. **Abbreviation 系统** — 输入时展开
13. **补全生态** — 补全常用外部命令（docker, npm, ssh 等）
14. **插件机制** — 函数文件自动加载 + 外部插件协议
15. **REPL 内配置** — `config set/get` 命令

### Phase 4: 打磨和推广（~4-6 周）
16. **文档体系** — `--help` 标准化 + man page + 文档站
17. **性能优化** — 启动时间、命令延迟基准测试和优化
18. **多行编辑** — 编辑器集成（Ctrl+E）
19. **Process Substitution** — `<(cmd)` 支持
20. **自定义键绑定** — `bind` 命令

---

## 五、结论

ASH 的 **架构设计是好的**——ash-core/auto-shell 分离、Atom 类型系统、AutoLang 集成。核心问题不是架构性的，而是 **工程完整性**：

> **最大的单个差距是管道系统**。从字符串管道升级到 OS pipe + Atom 结构化数据，是从"项目"到"产品"的关键跨越。

如果只能做 3 件事，按影响力排序：
1. **真正的 OS Pipe** — 没有 = 不能当 shell 用
2. **Ctrl+R + 历史搜索** — 没有 = 用户不会留下来
3. **`ash -c` + 登录模式** — 没有 = 不能集成到其他工具链
