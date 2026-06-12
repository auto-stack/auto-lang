# Plan 301: ASH 环境变量系统设计

## Context

ASH shell 目前有基础的 `ShellVars`（`crates/ash-core/src/shell/vars.rs`），支持 `set_local`/`set_env`/`unset`，但缺少：

1. **用户可用的 env 命令**：REPL 里没有 `env`、`export`、`set` 等内置命令
2. **PATH 一等公民**：PATH 是普通字符串，无列表操作、无去重、无表格展示
3. **临时环境变量**：没有 `VAR=val command` 内联语法
4. **脚本集成**：AutoLang 脚本里没有 env 模块 API
5. **持久化**：没有跨会话保存 env 的机制
6. **作用域隔离**：env 修改全局泄漏，没有块级作用域

### 设计目标

**shell 和脚本统一风格**：在 ash REPL 里打命令和在 .at 脚本里写代码，使用相同的语义模型，只是语法形态略有不同（命令式 vs 函数式）。

### 研究基础

对比分析了 6 种 shell（Bash、PowerShell、Fish、Nushell、Zsh、Xonsh）的环境变量处理方式，提取精华：

| 设计灵感 | 来源 | 应用 |
|---|---|---|
| 词法块作用域（env 修改不泄漏） | Nushell | `with env() {}` 作用域隔离 |
| 专用 PATH 命令 | Fish `fish_add_path` | `env.path.add/pre/rm` |
| 跨会话持久化 | Fish universal variables | `env -save` + `~/.config/ash/env.at` |
| 内联临时语法 | Bash `VAR=val cmd` | `K=V cmd args` 解析 |
| `$env` 命名空间 | Nushell | `env` 作为统一入口 |
| 表格展示 | Nushell table | Atom PathList 表格渲染 |
| PATH 自动去重 | Fish + Zsh `typeset -U` | `env.path.clean` / `env.path.dedup` |

---

## 一、核心语法设计

### 1.1 env 命名空间

REPL 中 `env` 是内置命令，脚本中 `env()` 是函数调用。语义完全一致。

**REPL 命令：**

```
env                  → 列出所有环境变量（表格展示）
env NAME             → 查询单个变量值
env NAME=val         → 设置环境变量
env -rm NAME         → 删除环境变量
env -save NAME val   → 持久化保存到 ~/.config/ash/env.at
env -load            → 手动加载持久化配置
```

**脚本 API：**

```auto
// 查询
let home = env("HOME")                    // → str（不存在返回空串）
let maybe = env.try("FOO")                // → Option<str>

// 设置
env("EDITOR", "vim")                      // 设置单个
env({                                      // 批量设置
    FOO: "bar",
    BAZ: "qux",
    LANG: "zh_CN.UTF-8"
})

// 删除
env_rm("OLD_VAR")

// 持久化
env.save("EDITOR", "vim")                 // 保存到文件
env.load()                                 // 加载
```

### 1.2 PATH 一等公民

PATH 不是普通字符串，而是一等 `List<str>`，支持列表操作和 Atom 表格展示。

**REPL 命令：**

```
env.path              → 以表格列出所有 PATH 条目（序号、路径、是否存在）
env.path.add DIR      → 追加到末尾
env.path.pre DIR      → 前插到开头（更高优先级）
env.path.rm DIR       → 按路径移除
env.path.rm #3        → 按序号移除
env.path.dedup        → 去重
env.path.clean        → 去重 + 移除不存在的目录
env.path.move #5 to #1 → 按序号移动条目位置
```

**REPL 展示效果：**

```
> env.path

 #  Path                      Exists
──  ────────────────────────  ──────
 0  /usr/local/bin            ✓
 1  /usr/bin                  ✓
 2  /home/user/.cargo/bin     ✓
 3  /opt/bad/path             ✗
 4  /usr/bin                  ✓  (duplicate of #1)
```

**脚本 API：**

```auto
// PATH 作为 List<str>
let p = env.path()                        // → List<str>
p.append("/new/dir")
p.prepend("/important/dir")
p.remove("/bad/dir")
env.set_path(p)                            // 写回

// 快捷方法
env.path_add("~/bin")                      // 追加（自动展开 ~）
env.path_pre("/usr/local/bin")             // 前插
env.path_rm("/old/path")                   // 移除
env.path_clean()                           // 去重 + 清理不存在路径
```

### 1.3 临时环境变量

**内联前缀语法（REPL + 脚本通用）：**

```bash
NODE_ENV=production auto build
FOO=bar BAZ=qux some_command arg1 arg2
```

解析规则：
- 出现在命令行开头
- `=` 前是合法标识符（`[A-Za-z_][A-Za-z0-9_]*`）
- `=` 后紧跟值（可引号包裹）
- 连续多个 `K=V` 对后跟实际命令

**with 块作用域（脚本专用）：**

```auto
// 单变量
with env("FOO", "bar") {
    run_command()       // FOO=bar 生效
}
// FOO 自动恢复

// 多变量
with env({FOO: "a", BAR: "b"}) {
    do_stuff()
}

// 单行简写
with env("FOO", "bar") run_command()
```

### 1.4 作用域模型

借鉴 Nushell 的词法块作用域：

```
全局 env ──────────────────────────────────────
  │
  └─ with env("FOO", "bar") ──────────────────
        │   FOO=bar 生效，不影响外部
        │   退出块自动恢复
        └─ with env("BAZ", "qux") ────────────
              │   FOO=bar + BAZ=qux 生效
              │   退出内层：BAZ 恢复
              └─ 嵌套结束
```

规则：
- **默认不泄漏**：`with` 块内的 env 修改在块外自动消失
- **全局设置**：`env NAME=val` 或 `env("NAME", "val")` 直接修改全局
- **子进程继承**：所有 env（全局 + 当前作用栈合并）自动传给子进程
- **内联前缀**：`K=V cmd` 等价于单命令的隐式作用域

### 1.5 完整命令速查表

| 命令 | 作用 | 示例 |
|---|---|---|
| `env` | 列出所有 env（表格） | `env` |
| `env NAME` | 查询单个 | `env HOME` |
| `env NAME=val` | 设置 env | `env EDITOR=vim` |
| `env -rm NAME` | 删除 env | `env -rm FOO` |
| `env -save NAME val` | 持久化保存 | `env -save EDITOR vim` |
| `env -load` | 加载持久化配置 | `env -load` |
| `env.path` | PATH 表格展示 | `env.path` |
| `env.path.add DIR` | 追加 PATH | `env.path.add ~/bin` |
| `env.path.pre DIR` | 前插 PATH | `env.path.pre /usr/local/bin` |
| `env.path.rm DIR` | 移除 PATH 条目 | `env.path.rm /bad/path` |
| `env.path.rm #N` | 按序号移除 | `env.path.rm #3` |
| `env.path.dedup` | 去重 PATH | `env.path.dedup` |
| `env.path.clean` | 去重 + 清理不存在 | `env.path.clean` |
| `env.path.move #N to #M` | 移动条目 | `env.path.move #5 to #1` |
| `K=V cmd args` | 临时 env 执行 | `NODE_ENV=prod auto build` |

### 1.6 错误处理

```bash
env PATH                   # → "/usr/bin:/bin:..." （正常输出）
env NONEXISTENT            # → "" （空串，不报错）
env -rm PATH               # → 错误：不能删除 PATH
env.path.rm #99            # → 错误：序号超出范围，当前共 8 条
env -save PATH /new        # → 错误：PATH 请使用 env.path 命令操作
```

---

## 二、持久化设计

### 2.1 配置文件 `~/.config/ash/env.at`

本质是标准 AutoLang 脚本，由 ash 自动管理，也可手动编辑：

```auto
// AutoShell 持久化环境变量
// 此文件由 ash 自动管理，手动编辑后运行 env -load 生效

env("EDITOR", "vim")
env("LANG", "zh_CN.UTF-8")
env("AUTO_THEME", "dark")
env.path_pre("/usr/local/bin")
env.path_pre(env("HOME") + "/.cargo/bin")
```

### 2.2 持久化流程

```
env -save EDITOR vim
    ↓
1. 在当前会话设置 env("EDITOR", "vim")
2. 追加一行 env("EDITOR", "vim") 到 ~/.config/ash/env.at
3. 去重：如果文件中已有 env("EDITOR", ...)，替换该行

env -load（或 ash 启动时自动执行）
    ↓
1. 检查 ~/.config/ash/env.at 是否存在
2. 用 AutoVM 执行该脚本（设置所有持久化变量）
```

### 2.3 目录结构

```
~/.config/ash/
├── env.at              # 持久化环境变量
├── history             # 命令历史（已有）
└── bookmarks.json      # 目录书签（已有）
```

---

## 三、架构设计

### 3.1 Atom 类型扩展

**文件**: `crates/ash-core/src/pipeline/atom.rs`

```rust
pub enum AtomType {
    // 已有类型...
    EnvVarList,       // env 命令返回的完整环境变量列表
    PathList,         // env.path 返回的 PATH 条目列表
}

/// 单个环境变量条目
pub struct AshEnvEntry {
    pub name: String,
    pub value: String,
    pub exported: bool,
}

/// 单个 PATH 条目
pub struct AshPathEntry {
    pub index: usize,
    pub path: String,
    pub exists: bool,
    pub duplicate: bool,
}
```

### 3.2 ShellVars 升级

**文件**: `crates/ash-core/src/shell/vars.rs`

```rust
pub struct ShellVars {
    locals: HashMap<String, String>,
    env: HashMap<String, String>,
    /// 作用域栈：每个 with 块 push 一层
    /// HashMap 值为 Some(new_val) 表示覆盖，None 表示临时删除
    scope_stack: Vec<HashMap<String, Option<String>>>,
}

impl ShellVars {
    // === 已有方法保持不变 ===

    // === 新增：作用域管理 ===

    /// 进入一个新作用域
    pub fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    /// 退出当前作用域，恢复被覆盖的变量
    pub fn pop_scope(&mut self) {
        if let Some(overrides) = self.scope_stack.pop() {
            // 恢复被覆盖的值（不需要额外操作，因为 set_env_scoped
            // 记录了旧值在 overrides 中）
            // 遍历 overrides，将旧值写回 env
            for (key, old_val) in overrides {
                match old_val {
                    Some(val) => { self.env.insert(key, val); },
                    None => { self.env.remove(&key); },
                }
            }
        }
    }

    /// 在当前作用域设置 env（记录旧值用于恢复）
    pub fn set_env_scoped(&mut self, name: String, value: String) {
        if let Some(scope) = self.scope_stack.last_mut() {
            // 只记录第一次覆盖的旧值
            if !scope.contains_key(&name) {
                let old = self.env.get(&name).cloned();
                scope.insert(name.clone(), old);
            }
        }
        self.env.insert(name.clone(), value.clone());
        std::env::set_var(name, value);
    }

    // === 新增：PATH 操作 ===

    /// 获取 PATH 作为列表（跨平台分隔符处理）
    pub fn get_path_list(&self) -> Vec<String> {
        let path_str = self.get_env("PATH").unwrap_or_default();
        let sep = if cfg!(windows) { ";" } else { ":" };
        path_str.split(sep)
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// 从列表写回 PATH
    pub fn set_path_list(&mut self, paths: Vec<String>) {
        let sep = if cfg!(windows) { ";" } else { ":" };
        let path_str = paths.join(sep);
        self.set_env("PATH".to_string(), path_str);
    }

    /// PATH 追加
    pub fn path_add(&mut self, dir: &str) {
        let mut paths = self.get_path_list();
        paths.push(dir.to_string());
        self.set_path_list(paths);
    }

    /// PATH 前插
    pub fn path_prepend(&mut self, dir: &str) {
        let mut paths = self.get_path_list();
        paths.insert(0, dir.to_string());
        self.set_path_list(paths);
    }

    /// PATH 移除
    pub fn path_remove(&mut self, dir: &str) {
        let mut paths = self.get_path_list();
        paths.retain(|p| p != dir);
        self.set_path_list(paths);
    }

    /// PATH 按序号移除
    pub fn path_remove_index(&mut self, index: usize) -> Result<(), String> {
        let mut paths = self.get_path_list();
        if index >= paths.len() {
            return Err(format!("序号超出范围，当前共 {} 条", paths.len()));
        }
        paths.remove(index);
        self.set_path_list(paths);
        Ok(())
    }

    /// PATH 去重
    pub fn path_dedup(&mut self) {
        let mut seen = std::collections::HashSet::new();
        let paths = self.get_path_list();
        let deduped: Vec<String> = paths.into_iter()
            .filter(|p| seen.insert(p.to_lowercase())) // 大小写不敏感去重
            .collect();
        self.set_path_list(deduped);
    }

    /// PATH 清理（去重 + 移除不存在的目录）
    pub fn path_clean(&mut self) {
        let mut seen = std::collections::HashSet::new();
        let paths = self.get_path_list();
        let cleaned: Vec<String> = paths.into_iter()
            .filter(|p| {
                let canonical = p.to_lowercase();
                seen.insert(canonical) && std::path::Path::new(p).exists()
            })
            .collect();
        self.set_path_list(cleaned);
    }

    /// 获取 PATH 条目详情（含 exists 和 duplicate 信息）
    pub fn get_path_entries(&self) -> Vec<AshPathEntry> {
        let paths = self.get_path_list();
        let mut seen = std::collections::HashSet::new();
        paths.iter().enumerate().map(|(i, p)| {
            let canonical = p.to_lowercase();
            let dup = !seen.insert(canonical);
            AshPathEntry {
                index: i,
                path: p.clone(),
                exists: std::path::Path::new(p).exists(),
                duplicate: dup,
            }
        }).collect()
    }
}
```

### 3.3 新增内置命令

**文件**: `crates/auto-shell/src/cmd/commands/env_cmd.rs`（新建）

```rust
/// env 命令：统一环境变量管理
pub struct EnvCommand;

/// env.path 子命令：PATH 操作
pub struct EnvPathCommand;
```

### 3.4 管道解析扩展

**文件**: `crates/ash-core/src/parser/pipeline.rs`

在 pipeline 解析器中添加 `K=V` 前缀识别：

```rust
/// 尝试解析命令行开头的 K=V 对
fn parse_env_prefixes(tokens: &[&str]) -> (Vec<(String, String)>, &[&str]) {
    let mut env_pairs = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if let Some((key, val)) = try_parse_kv(tokens[i]) {
            env_pairs.push((key, val));
            i += 1;
        } else {
            break;
        }
    }
    (env_pairs, &tokens[i..])
}

fn try_parse_kv(token: &str) -> Option<(String, String)> {
    let eq_pos = token.find('=')?;
    let key = &token[..eq_pos];
    let val = &token[eq_pos + 1..];
    // key 必须是合法标识符
    if key.is_empty() || !key.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_') {
        return None;
    }
    if !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    // 去掉 val 的引号
    let val = val.trim_matches('"').trim_matches('\'');
    Some((key.to_string(), val.to_string()))
}
```

---

## 四、实现计划

### Phase 1：ShellVars 升级（核心基础设施）

**目标**：升级 `ShellVars` 支持 PATH 操作和作用域栈。

**步骤**：

1. **P1.1** 升级 `ShellVars` 结构体
   - 文件：`crates/ash-core/src/shell/vars.rs`
   - 添加 `scope_stack: Vec<HashMap<String, Option<String>>>`
   - 添加 `push_scope()` / `pop_scope()` / `set_env_scoped()`

2. **P1.2** 添加 PATH 操作方法
   - 文件：`crates/ash-core/src/shell/vars.rs`
   - 添加 `get_path_list()` / `set_path_list()` / `path_add()` / `path_prepend()`
   - 添加 `path_remove()` / `path_remove_index()` / `path_dedup()` / `path_clean()`
   - 添加 `get_path_entries()` 返回 `Vec<AshPathEntry>`

3. **P1.3** 添加 PATH 数据类型
   - 文件：`crates/ash-core/src/pipeline/atom.rs`
   - 在 `AtomType` 枚举中添加 `EnvVarList` / `PathList`
   - 添加 `AshEnvEntry` / `AshPathEntry` 结构体

4. **P1.4** 编写单元测试
   - 测试作用域 push/pop 恢复
   - 测试 PATH 增删改查
   - 测试 PATH 去重和清理
   - 测试跨平台分隔符

### Phase 2：env 内置命令

**目标**：在 REPL 中实现 `env` 和 `env.path` 命令。

**步骤**：

1. **P2.1** 实现 `EnvCommand`
   - 新文件：`crates/auto-shell/src/cmd/commands/env_cmd.rs`
   - 无参数 → 调用 `vars.list_env()` + `std::env::vars()`，返回 `AtomType::EnvVarList`
   - 单参数 `NAME` → 调用 `vars.get_env(NAME)`，返回文本
   - `NAME=val` 形式 → 调用 `vars.set_env()`
   - `-rm NAME` → 调用 `vars.unset_env()`
   - `-save NAME val` → 写入 `~/.config/ash/env.at`
   - `-load` → 读取并执行 `~/.config/ash/env.at`

2. **P2.2** 实现 `EnvPathCommand`
   - 新文件：`crates/auto-shell/src/cmd/commands/env_path_cmd.rs`
   - 无参数 → 调用 `vars.get_path_entries()`，返回 `AtomType::PathList`
   - `add DIR` → `vars.path_add()`
   - `pre DIR` → `vars.path_prepend()`
   - `rm DIR` / `rm #N` → `vars.path_remove()` / `vars.path_remove_index()`
   - `dedup` → `vars.path_dedup()`
   - `clean` → `vars.path_clean()`
   - `move #N to #M` → 按序号移动

3. **P2.3** 注册命令到 CommandRegistry
   - 文件：`crates/auto-shell/src/cmd/registry.rs`
   - 注册 `env` 和 `env.path` 命令

4. **P2.4** 表格渲染
   - `EnvVarList` → 两列表格（Name | Value），按名称排序
   - `PathList` → 三列表格（# | Path | Exists），标记重复和不存在

5. **P2.5** 集成测试
   - 文件：`crates/auto-shell/tests/env_test.rs`
   - 测试 `env`/`env NAME`/`env NAME=val`/`env -rm`
   - 测试 `env.path`/`env.path.add`/`env.path.rm`/`env.path.dedup`

### Phase 3：K=V 内联前缀解析

**目标**：支持 `VAR=val command args` 临时环境变量语法。

**步骤**：

1. **P3.1** 扩展 pipeline 解析器
   - 文件：`crates/ash-core/src/parser/pipeline.rs`
   - 在 pipeline 解析前提取 `K=V` 前缀对
   - 将剩余 tokens 作为实际命令

2. **P3.2** 执行时应用临时 env
   - 文件：`crates/auto-shell/src/cmd/pipeline.rs`
   - 如果有 env 前缀：push_scope → 设置临时变量 → 执行命令 → pop_scope
   - 确保 pop_scope 在任何情况下都执行（包括错误）

3. **P3.3** 测试
   - 测试 `FOO=bar echo $FOO` → 输出 `bar`
   - 测试多变量 `A=1 B=2 cmd`
   - 测试作用域清理（临时变量不影响全局）

### Phase 4：持久化

**目标**：实现 `env -save` / `env -load` 和启动时自动加载。

**步骤**：

1. **P4.1** 配置目录管理
   - 新模块：`crates/ash-core/src/persistence.rs`（或扩展已有模块）
   - `ensure_config_dir()` → 创建 `~/.config/ash/`
   - `env_file_path()` → 返回 `~/.config/ash/env.at` 路径

2. **P4.2** env.at 文件读写
   - `save_env_var(name, val)` → 追加或替换行
   - `load_env_file()` → 读取文件内容
   - `remove_env_var(name)` → 从文件删除行

3. **P4.3** 启动时自动加载
   - 文件：`crates/auto-shell/src/shell.rs`
   - Shell 初始化时检查 `~/.config/ash/env.at`，存在则执行

4. **P4.4** 测试
   - 测试 save/load 循环
   - 测试文件不存在时的行为
   - 测试重复 save 的去重

### Phase 5：脚本 API（AutoLang 集成）

**目标**：在 .at 脚本中通过 FFI 调用 env 功能。

**步骤**：

1. **P5.1** VM FFI 注册
   - 文件：`crates/auto-lang/src/vm/ffi/stdlib.rs`
   - 注册 `env()` 函数：0 参数返回全部，1 参数查询，2 参数设置
   - 注册 `env_try()` 函数：返回 Option
   - 注册 `env_rm()` 函数
   - 注册 `env_save()` / `env_load()` 函数

2. **P5.2** PATH 脚本 API
   - 注册 `env_path()` → 返回 `List` (AutoLang Value)
   - 注册 `env_set_path()` → 从 `List` 写回
   - 注册 `env_path_add()` / `env_path_pre()` / `env_path_rm()`
   - 注册 `env_path_clean()` / `env_path_dedup()`

3. **P5.3** `with env()` 块语法
   - 需要编译器支持 `with` 语句（如果已有则复用）
   - 编译为：push_scope → 设置变量 → 执行块 → pop_scope
   - **注意**：如果当前 `with` 语法不支持自定义类型，可以考虑先实现为函数：

   ```auto
   // 如果 with 语法暂不支持，可以先用函数式 API
   env.scoped({FOO: "bar"}, fn() {
       run_command()
   })
   ```

4. **P5.4** 测试
   - 创建测试文件 `crates/auto-lang/test/a2c/` 下的 env 相关测试
   - 测试脚本中的 env 读写
   - 测试 PATH 列表操作

### Phase 6：补全 + 文档

**目标**：为 env 命令添加 tab 补全和使用文档。

**步骤**：

1. **P6.1** 补全支持
   - `env` 后 tab → 列出所有 env 变量名
   - `env -` 后 tab → 列出 flag（`-rm`, `-save`, `-load`）
   - `env.path.` 后 tab → 列出子命令（`add`, `pre`, `rm`, `dedup`, `clean`）

2. **P6.2** 帮助文本
   - `env --help` → 显示完整用法说明
   - `env.path --help` → 显示 PATH 操作说明

3. **P6.3** 更新设计文档
   - 更新 `docs/design/11-shell-tools.md` 添加环境变量章节

---

## 五、实现优先级与里程碑

| Phase | 内容 | 预估工作量 | 依赖 |
|---|---|---|---|
| **Phase 1** | ShellVars 升级 | 1-2 天 | 无 |
| **Phase 2** | env 内置命令 | 2-3 天 | Phase 1 |
| **Phase 3** | K=V 内联前缀 | 1 天 | Phase 1 |
| **Phase 4** | 持久化 | 1-2 天 | Phase 2 |
| **Phase 5** | 脚本 API | 2-3 天 | Phase 1, 2 |
| **Phase 6** | 补全 + 文档 | 1 天 | Phase 2 |

**推荐实施顺序**：Phase 1 → Phase 2 → Phase 3 → Phase 5 → Phase 4 → Phase 6

理由：先让 REPL 中的 env 命令跑起来（Phase 1-2-3），然后接入脚本（Phase 5），最后做持久化和文档。

---

## 六、跨平台注意事项

1. **PATH 分隔符**：Windows 用 `;`，Unix 用 `:`。`get_path_list()` / `set_path_list()` 使用 `cfg!(windows)` 自动适配。
2. **路径大小写**：Windows 路径不区分大小写，PATH 去重时需要做 `to_lowercase()` 比较。
3. **配置目录**：Windows 上 `%APPDATA%\ash\`，Unix 上 `~/.config/ash/`。使用 `dirs::config_dir()` crate 获取。
4. **环境变量大小写**：Windows 上环境变量不区分大小写，`get_env()` 需要做 case-insensitive 查找。
