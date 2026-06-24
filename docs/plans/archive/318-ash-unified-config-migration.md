# Plan 318: 统一 Ash 配置系统——迁移到 Auto/Atom 格式 + 统一目录

> **Status**: ✅ Implemented (2026-06-17). Phases 1–3 done; Phase 4 (cleanup/docs) done via this update + design doc §5.
> **关系**: 收拢 [Plan 309](309-ash-remaining-features-roadmap.md) / [315](315-ash-arbitrary-command-completion.md) /
> [317](317-ash-24bit-truecolor.md) 引入的各配置文件,统一为 **Auto/Atom(.at) 格式**,全部放在 `~/.config/ash/` 目录下。
> 设计文档同步更新到 [ash-design-summary.md §5](../design/ash-design-summary.md)。

---

## 1. 现状

| 文件 | 格式 | 位置 | 用途 | 已转 .at? |
|---|---|---|---|---|
| `ash.toml` | TOML | `~/.config/` | Shell 行为(history/autosuggestion/edit_mode/aliases/completion) | ❌ |
| `ash.at` | Auto/Atom | `~/.config/` | ls 图标 + 未来设置 | ✅ |
| `ash-prompt.toml` | TOML | `~/.config/` | Prompt 模块/format/各模块 disabled/style/symbol | ❌ |
| `~/.ashrc` | Shell 脚本 | `~/` | 启动脚本(可执行 alias/env/abbr) | N/A(脚本) |
| `env.at` | .at(shell 行) | `~/.config/ash/` | env 持久化 | ✅ |
| `completions/*.at` | Auto/Atom | `~/.config/ash/completions/` | 补全 spec 三层 | ✅ |

**问题**:两个 TOML 文件未迁移;配置散落在 `~/.config/` 根目录而非统一子目录。

---

## 2. 目标目录结构

```
~/.config/ash/
├── config.at           ← 统一主配置(替代 ash.toml + ash.at)
├── prompt.at           ← Prompt 配置(替代 ash-prompt.toml)
├── env.at              ← env 持久化(已存在)
└── completions/
    ├── *.at            ← 用户手写补全 spec
    ├── generated/
    │   └── *.at        ← completions generate 生成的 spec
    └── cache/
        └── *.at        ← 运行时 help-probe 缓存的 spec
```

- **一个目录、一个格式**:所有声明式配置在 `~/.config/ash/`,统一 Auto/Atom。
- **按关注点分文件**:`config.at`(shell 行为) / `prompt.at`(prompt) / `env.at`(env 持久化)。
- **`~/.ashrc` 保留**:它是**可执行脚本**(可写逻辑、条件、循环),不是声明式配置,不合并。
- **`completions/`** 已是这个结构(Plan 315),不动。

---

## 3. 目标 `config.at` 格式

```auto
// ~/.config/ash/config.at — 统一 Ash 配置（Auto/Atom 格式）

shell {
    history_size             : 10000
    autosuggestion           : true
    autosuggestion_min_chars : 1
    edit_mode                : emacs     // emacs | vi
    syntax_highlighting      : true
}

aliases {
    ll : "ls -la"
    gs : "git status"
}

completion {
    case_sensitive : false
}

ls {
    icons : nerdfont          // plain | nerdfont | emoji | off
}
```

---

## 4. 目标 `prompt.at` 格式

```auto
// ~/.config/ash/prompt.at — Prompt 配置

prompt {
    format              : "$directory$git_branch$git_status$character"
    right_format        : "$time"
    add_newline         : true
    cmd_duration_threshold : 5000

    directory {
        style : "cyan bold"
        truncation_length : 3
    }
    git_branch {
        disabled : false
        symbol : "⎇ "
        style : "green bold"
    }
}
```

> prompt 的嵌套比 config 更深(模块 → 属性),需要 auto_config 支持二级嵌套(§5 Phase 1)。

---

## 5. 实现阶段

### Phase 1:扩展 auto_config 解析器 + 路径统一

1. `auto_config.rs`:支持**二级嵌套** block(`prompt { git_branch { symbol : ... } }`)——当前只支持一级。
2. 路径统一:`auto_config.rs` / `spec_tiers.rs` / `env.at` 加载器统一从 `~/.config/ash/` 目录读(目前有些读 `~/.config/` 根目录)。
3. 值类型:支持 bool/int 解析(auto_config 返回 string,caller 按 key 转换;或加 typed getter)。
4. 单测:二级嵌套解析、bool/int 转换、路径解析。

### Phase 2:迁移 ash.toml → config.at

1. `config.rs::load()`:优先读 `~/.config/ash/config.at`(auto_config);若不存在 → 向后兼容读 `~/.config/ash.toml`(TOML)。
2. `config.rs` 的 `AshShellConfig` 字段从 auto_config 的 block→string map 填充(parse bool/int from string)。
3. `config set/get` builtin:写回 `config.at`(而非 `ash.toml`)。
4. 启动时检测旧 `ash.toml` 存在 → 打印迁移提示(`ash.toml will be deprecated; run 'config migrate'`)。
5. `config migrate` 子命令:读 `ash.toml` → 生成 `config.at` → 打印完成。
6. 测试 + 验证。

### Phase 3:迁移 ash-prompt.toml → prompt.at

1. `prompt/config.rs`:优先读 `~/.config/ash/prompt.at`(auto_config 二级嵌套);向后兼容读 `ash-prompt.toml`。
2. `AshConfig`(prompt)的字段从 auto_config 填充。
3. 测试 + 验证。

### Phase 4:清理 + 文档

1. 删除 `~/.config/ash.at`(旧,已合入 `config.at` 的 `ls` 块)。
2. 更新 [ash-design-summary.md §5](../design/ash-design-summary.md) 配置系统设计(目录结构 + .at schema)。
3. 更新 CLAUDE.md 提及统一配置路径。
4. 保留向后兼容读(旧 TOML 文件存在时读取 + 提示),直至确认无用户依赖。

---

## 6. 向后兼容

- 旧 `ash.toml` / `ash-prompt.toml` / `ash.at` 存在时**仍读取**(降级路径),但启动时打印一次性提示。
- `config migrate` 命令一键迁移。
- 旧的 `~/.config/ash.at`(独立文件)的内容(ls.icons)合入 `config.at` 的 `ls { icons : ... }` 块;旧的独立 `ash.at` 删除。

---

## 7. 关键文件

| 文件 | 改动 |
|---|---|
| `auto-shell/src/auto_config.rs` | 二级嵌套 + bool/int 值 + 统一路径 `~/.config/ash/` |
| `auto-shell/src/config.rs` | 从 `config.at` 读(shell/aliases/completion/ls) + `config migrate` |
| `auto-shell/src/prompt/config.rs` | 从 `prompt.at` 读(二级嵌套) |
| `auto-shell/src/completions/spec_tiers.rs` | 路径统一(已在 `~/.config/ash/completions/`,确认) |
| `auto-shell/src/shell.rs` | env.at 路径统一(已在 `~/.config/ash/`,确认) |
| `docs/design/ash-design-summary.md` | §5 配置系统设计 |
