# Plan 291: AutoShell → 高速版 Warp 全栈升级设计

> **日期**: 2026-06-10
> **状态**: 设计已批准
> **策略**: 自底向上，数据管线优先
> **总预估**: 15-22 周

## 愿景

将 AutoShell 从 18 命令的传统 REPL 升级为**三位一体**的现代 Shell：
1. **结构化数据管线** (对标 NuShell 400+ 命令)
2. **AI 智能集成** (对标 Warp 2.0 Agent 模式)
3. **Block UX 现代化** (对标 Warp 终端体验)

核心原则：**NuShell 底层库优先 → uutils 兜底 → 自定义实现**。

## 当前状态

### 已有 (18 命令)
- 文件系统: `ls`, `cd`, `pwd`, `mkdir`, `rm`, `mv`, `cp`
- 数据处理: `grep`, `wc`, `select`, `where`, `get`
- 系统: `ps`, `sys`
- Auto 专属: `build`, `run`
- 工具: `echo`, `help`

### 管线架构
- `PipelineData(Value|Text)` 二元模型
- 零拷贝 Value 传递
- 159 个测试通过

### Atom/Batom 状态
- Atom 格式已设计 (JSON 超集 + 树结构)
- Batom (二进制 Atom) 仅提及，未实现
- 管道未集成 Atom

---

## Phase 0: Atom 管线基础 (2-3 周)

### 目标
将 Atom 格式集成为 AutoShell 管道的一等公民数据载体。

### 核心变更

#### 1. 新的管线数据模型

```rust
// 当前
pub enum PipelineData {
    Value(Value),
    Text(String),
}

// 目标
pub enum AtomPipeline {
    Atom(Atom),           // 结构化 Atom 数据（零拷贝）
    AtomStream(AtomIter), // 流式 Atom（大数据集）
    Text(String),         // 兼容外部命令的文本回退
    Empty,                // 无输出
}
```

#### 2. Atom 作为管线通用语言

```
ls ──→ Atom ──→ where ──→ Atom ──→ select ──→ Atom ──→ table渲染
                        │
                        ├─→ to json ──→ Text(输出)
                        └─→ to yaml ──→ Text(输出)
```

每个内置命令的输入输出都是 Atom，不再通过 `Value` 中间层。

#### 3. 现有 18 个命令迁移

| 命令 | 输出 Atom 类型 |
|------|---------------|
| `ls` | `Atom::List<FileEntry>` |
| `ps` | `Atom::List<ProcessEntry>` |
| `sys` | `Atom::Object<SystemInfo>` |
| `grep` | `Atom::List<MatchResult>` |
| `wc` | `Atom::Object<CountResult>` |
| `select` | 透传 Atom，投影字段 |
| `where` | 透传 Atom，过滤记录 |
| `get` | 提取单个字段为 Atom |
| `echo` | `Atom::Text` |
| `cd`/`pwd` | `Atom::Text` (路径) |
| `mkdir`/`rm`/`mv`/`cp` | `Atom::Empty` (状态命令) |
| `build`/`run` | `Atom::Object<BuildResult>` |
| `help` | `Atom::Object<HelpInfo>` |

#### 4. NuShell crate 集成点预留

```rust
impl From<nu_protocol::Value> for Atom { ... }
impl From<Atom> for nu_protocol::Value { ... }
```

Phase 2 集成 NuShell 库时，命令输出可无缝转换为 Atom。

### 交付物
- `crates/auto-shell/src/pipeline/atom_pipeline.rs`
- 18 个命令迁移到 Atom 输出
- Atom ↔ nu_protocol::Value 转换层骨架
- 更新测试套件

---

## Phase 1: Batom 二进制格式 (2-3 周)

### 目标
在 Atom 文本格式基础上，增加 Batom (Binary Atom) 二进制编码，实现高性能跨进程管道传输。

### 编码格式

```
┌──────────┬────────┬──────────────────────┐
│ Magic    │ Header │ Payload              │
│ 4 bytes  │ N bytes│ Variable             │
│ "BATM"   │        │                      │
└──────────┴────────┴──────────────────────┘

Header:
  - version: u8
  - flags: u8 (compression, endianness)
  - type_count: u16 (number of distinct types)
  - string_table_offset: u32
  - string_table_length: u32

Payload:
  - Type definitions (schema-like)
  - String table (deduplicated strings)
  - Data records (using type IDs + string refs)
```

### 关键特性

- **Magic Number** `"BATM"` — 便于识别和校验
- **字符串去重表** — 重复的字段名/值只存一次
- **类型定义** — 类似 FlatBuffers 的 schema-inline 设计
- **可选压缩** — `flags` 位控制，大数据集可启用 LZ4
- **小端序** — 与现代 CPU 一致

### Atom ↔ Batom 双向转换

```rust
fn encode_atom(atom: &Atom) -> Vec<u8>;
fn encode_atom_stream(atoms: impl Iterator<Item=Atom>) -> Vec<u8>;
fn decode_batom(data: &[u8]) -> Result<Atom>;
fn decode_batom_stream(data: &[u8]) -> Result<AtomIter>;
```

### 跨进程管道

```bash
# 环境变量协商
export AUTOSHELL_PIPE_FORMAT=batom

# 管道中使用
ls | batom-encode | ./external-tool | batom-decode | where size > 1024
```

### 性能目标

| 场景 | Atom (文本) | Batom (二进制) | 提升 |
|------|------------|----------------|------|
| 1000 条文件信息 | ~45KB | ~18KB | 2.5x 体积 |
| 解析 `ls -R /usr` | ~120ms | ~25ms | 5x 速度 |
| 跨进程管道 | 序列化+解析 | 零拷贝映射 | ~10x |

### 交付物
- `crates/auto-shell/src/pipeline/batom.rs` (编码/解码)
- `crates/auto-shell/src/pipeline/batom_stream.rs` (流式处理)
- 跨进程管道协议实现
- 性能基准测试

---

## Phase 2: NuShell 库集成 + 命令扩展 (4-6 周)

### 目标
通过集成 NuShell 底层 crate，将命令从 18 扩展到 100+，全部输出 Atom。

### 集成架构

```
┌─────────────────────────────────────────────────┐
│                  AutoShell                       │
├─────────────────────────────────────────────────┤
│  Command Registry (Atom-based)                   │
├────────┬──────────────┬──────────┬───────────────┤
│ NuShell│   uutils     │  Custom  │  External     │
│ Crates │   Fallback   │  Auto    │  Wrapper      │
│        │              │          │               │
│ nu-* → │ coreutils → │ Auto专有 │ 系统命令+     │
│ Atom   │ Atom         │ Atom     │ 文本解析→Atom │
└────────┴──────────────┴──────────┴───────────────┘
```

| 层级 | 来源 | 输出 | 适用场景 |
|------|------|------|---------|
| **优先** | NuShell crate (`nu-command`) | `nu::Value` → Atom | 已有结构化输出 |
| **备选** | uutils-coreutils | 文本 → 解析 → Atom | NuShell 未覆盖 |
| **兜底** | AutoLang 自定义实现 | 原生 Atom | Auto 特有功能 |
| **兼容** | 外部系统命令 | 文本 → Atom | 无法集成 |

### NuShell crate 依赖

- `nu-protocol` — 核心类型系统 (Value, ShellError, Span)
- `nu-command` — 具体命令实现
- `nu-engine` — 命令执行引擎
- `nu-parser` — 命令参数解析

### 适配层

```rust
pub struct NuAdapter;

impl NuAdapter {
    pub fn value_to_atom(nu_val: nu_protocol::Value) -> Atom { ... }
    pub fn atom_to_value(atom: Atom) -> nu_protocol::Value { ... }
}
```

### 命令扩展优先级

**第一批：高频文件操作 (15 个)**
`cat`, `head`, `tail`, `touch`, `ln`, `chmod`, `chown`, `file`, `find`, `glob`, `open`, `save`, `du`, `stat`, `tee`

**第二批：文本处理 (10 个)**
`sort`, `uniq`, `cut`, `paste`, `tr`, `split`, `rev`, `column`, `fmt`, `diff`

**第三批：数据格式 (10 个)**
`from json/yaml/csv/toml/xml`, `to json/yaml/csv/toml/xml`

**第四批：字符串操作 (10 个)**
`str replace`, `str contains`, `str split`, `str join`, `str trim`, `str upcase`, `str downcase`, `str length`, `str capitalize`, `str index-of`

**第五批：数学 + 聚合 (10 个)**
`math sum/avg/min/max/round/floor/ceil/abs/sqrt`, `reduce`

**第六批：数据变换 (10 个)**
`update`, `insert`, `upsert`, `merge`, `flatten`, `group-by`, `sort-by`, `rename`, `each`, `enumerate`

**第七批：HTTP + 网络 (6 个)**
`http get/post/put/delete/patch/head`

**第八批：日期时间 + 工具 (8 个)**
`date now/format/list-timezone`, `sleep`, `timeit`, `which`, `type`, `version`

### 交付物
- NuShell crate 集成 + 适配层
- 80+ 新命令实现（覆盖第一批到第六批）
- HTTP + 日期时间命令
- 每批命令对应的测试套件

---

## Phase 3: AI 智能集成 (3-4 周)

### 目标
在成熟的 Atom 数据管线和丰富命令集基础上，集成 AI 能力。

### AI 三层架构

```
┌─────────────────────────────────────────────┐
│           Layer 3: Agent Mode               │
│   自然语言工作流 / 多步任务自动执行          │
├─────────────────────────────────────────────┤
│           Layer 2: Command Intelligence      │
│   命令建议 / 错误解释 / 参数补全            │
├─────────────────────────────────────────────┤
│           Layer 1: LLM API Foundation        │
│   统一 LLM 接口 / 流式响应 / 上下文管理     │
└─────────────────────────────────────────────┘
```

### Layer 1: LLM API 基础

```rust
pub trait LLMProvider {
    async fn chat(&self, messages: Vec<Message>) -> Result<LLMResponse>;
    async fn chat_stream(&self, messages: Vec<Message>) -> Result<LLMStream>;
}

// Provider 实现
- AnthropicProvider (Claude)
- OpenAIProvider (GPT)
- OllamaProvider (本地模型)
```

AutoLang 绑定 (`stdlib/auto/llm.at`):
```auto
use ai: chat, stream

fn ask_ai(prompt str) str {
    let response = chat(prompt, provider: "claude")
    return response.text
}
```

### Layer 2: 命令智能

**自然语言 → 命令**:
```
> # 列出所有大于1MB的文件
→ ls -lh | where size > 1048576 | select name size
```

**错误解释**:
```
> rm /protected/file
Error: Permission denied
💡 AI 解释: 文件受系统保护，需要管理员权限。
   建议尝试: sudo rm /protected/file
```

### Layer 3: Agent 模式

```
> agent: "清理项目中所有未使用的依赖"

Agent 执行:
  1. 分析项目结构... ✓
  2. 扫描依赖引用... ✓
  3. 发现 3 个未使用依赖: lodash, moment, axios
  4. 执行: rm node_modules/lodash node_modules/moment node_modules/axios
  5. 更新 package.json
  6. 完成 ✓
```

### Agent 安全模型

- 每个操作需用户确认（可配置自动批准白名单）
- 沙箱执行模式：危险命令在隔离环境中运行
- 操作日志：所有 Agent 动作可审计

### Atom 管线上下文

Agent 能理解 AutoShell 的 Atom 管线上下文:
```bash
ls | where type == dir        # Agent 知道上一步的输出是目录列表
> agent: "这些目录中哪个占用空间最大？"
→ du -sh * | sort -rh | first
```

### 交付物
- `crates/auto-shell/src/ai/` (LLM provider, 消息类型)
- `stdlib/auto/llm.at` (AutoLang 绑定)
- 自然语言 → 命令生成
- 错误解释集成
- Agent 模式基础实现

---

## Phase 4: Block UX 现代化 (4-6 周)

### 目标
将 REPL 从传统终端升级为 Block 级交互的现代 Shell 体验。

### Block 模型

```
┌─────────────────────────────────────────────────┐
│  📂 Block #1  ls -lh src/              0.02s ✓  │  ← 粘性标题
│  ────────────────────────────────────────────── │
│  名称        类型    大小      修改时间           │  ← 可折叠
│  main.rs    file   12.3 KB   2026-06-10 14:30  │
│  mod.rs     file    2.1 KB   2026-06-09 09:15  │
│  utils/     dir      --      2026-06-08 16:22  │
│  ────────────────────────────────────────────── │
│  [复制] [搜索] [展开/折叠] [分享]                │  ← 操作栏
├─────────────────────────────────────────────────┤
│  > ┃ ls -lh | where size > 1MB                 │  ← 现代编辑器
│    ┃                                           │
│  ┌─────────────────────────────────────────────┐│
│  │ 建议命令:  ls -lh | where size > 1048576    ││  ← AI 建议
│  │ 最近使用:  ls -lh src/                      ││  ← 历史推荐
│  └─────────────────────────────────────────────┘│
└─────────────────────────────────────────────────┘
```

### Block 数据结构

```rust
pub struct Block {
    pub id: BlockId,
    pub command: String,
    pub started_at: DateTime,
    pub duration: Duration,
    pub exit_code: i32,
    pub output: AtomPipeline,
    pub display_state: BlockDisplayState,
}

pub enum BlockDisplayState {
    Expanded,
    Collapsed,
    Searching(String),
    Selected,
}
```

### 现代输入编辑器

- 多行编辑 (Shift+Enter)
- 语法高亮 (命令/参数/字符串/运算符)
- 光标点击定位
- 多光标 (Ctrl+点击)
- 内联补全
- 富粘贴

### Atom 感知渲染

| Atom 类型 | 渲染视图 |
|-----------|---------|
| `Atom::List<FileEntry>` | 表格视图 |
| `Atom::List<Process>` | 表格 + 进度条 |
| `Atom::Text` | 代码视图 + 语法高亮 |
| `Atom::Object` | 键值对视图 |
| `Atom::Empty` | 隐藏 Block |

用户可切换: `table` / `json` / `atom` / `compact`

### 交互快捷键

| 操作 | 快捷键 | 描述 |
|------|--------|------|
| 上一个 Block | `Ctrl+↑` | 导航到上一个命令输出 |
| 下一个 Block | `Ctrl+↓` | 导航到下一个命令输出 |
| 折叠/展开 | `Ctrl+Space` | 切换 Block 折叠状态 |
| Block 内搜索 | `Ctrl+F` | 搜索当前输出 |
| 复制 Block | `Ctrl+Shift+C` | 复制整个 Block 输出 |
| AI 解释 | `右键 → Ask AI` | AI 解释当前 Block |

### 渲染策略

**终端内 Block 模拟**，不重造终端模拟器：
- ANSI 转义序列实现 Block 边框和标题
- alternate screen buffer 实现全屏模式
- 依赖终端 (Windows Terminal, iTerm2) 的 GPU 能力
- 专注 Shell 层的 Block 体验

### 交付物
- `crates/auto-shell/src/block/` (Block 模型, 渲染, 导航)
- `crates/auto-shell/src/editor/` (现代输入编辑器)
- ANSI Block 渲染引擎
- 交互快捷键绑定
- Atom 感知渲染器

---

## 里程碑时间线

| Phase | 内容 | 预估时间 | 累计 |
|-------|------|---------|------|
| 0 | Atom 管线基础 | 2-3 周 | 2-3 周 |
| 1 | Batom 二进制格式 | 2-3 周 | 4-6 周 |
| 2 | NuShell 集成 + 命令扩展 | 4-6 周 | 8-12 周 |
| 3 | AI 智能集成 | 3-4 周 | 11-16 周 |
| 4 | Block UX 现代化 | 4-6 周 | 15-22 周 |

## 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| NuShell crate 版本兼容 | 高 | 锁定版本 + 适配层隔离 |
| Batom 性能不达预期 | 中 | 保留 Atom 文本模式作为回退 |
| AI Provider API 变更 | 低 | 抽象层隔离，多 Provider 支持 |
| Block UX 终端兼容性 | 中 | 检测终端能力，降级为传统模式 |

## 参考文档

- [docs/design/11-shell-tools.md](../design/11-shell-tools.md) — AutoShell 设计
- [docs/design/07-data-structures.md](../design/07-data-structures.md) — Atom/Batom 数据模型
- [docs/plans/old/017-auto-shell-design.md](old/017-auto-shell-design.md) — AutoShell 实现计划 (已完成)
- [docs/plans/old/047-auto-value-pipelines.md](old/047-auto-value-pipelines.md) — 结构化管道 (已完成)
- [docs/plans/old/153-autoshell-ai-agent-design.md](old/153-autoshell-ai-agent-design.md) — AI Agent 设计 (草案)
