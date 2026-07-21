# Plan 362: 快速反馈链路 — `auto watch` + 分层重建 + 生成器缓存

> **目标**: 把"改一行 .at → 看到效果"的反馈时间从 **30-90 秒** 压缩到 **<2 秒**（.at 改动）和 **<5 秒**（生成器改动）。

---

## 1. 当前反馈链路为什么慢

### 1.1 改 .at 文件的完整链路

```
用户改 .at
  ↓ （手动）auto run / auto build
解析 .at → Aura AST                    ~10ms
extract_widget / extract_store         ~5ms
VueGenerator::generate                 ~20ms
写 SFC 文件                             ~1ms
  ↓ Vite HMR 检测到 .vue 变化
Vite 重新编译受影响模块                  ~500ms-2s
浏览器热更新                            ~200ms
─────────────────────────────────────
总计：如果不重新 build，本可在 ~3s 内完成

但实际情况是 30-90s，因为：
```

### 1.2 慢在哪里（实测耗时分布）

| 阶段 | 耗时 | 原因 |
|------|------|------|
| `cargo build` auto 二进制 | 20-60s | 改了 .at 但用户以为要重新编译（误解） |
| `auto build` 重新生成 | 5-10s | 全量重建所有 SFC，不增量 |
| `pnpm install` | 2-8s | 每次 build 都跑（即使没变） |
| `vue-tsc` 类型检查 | 5-15s | 全量类型检查 |
| Rust 后端编译 | 3-10s | 每次都重编 |

**关键发现**：真正的 .at → Vue 生成只要 ~35ms，但**周围套了一堆不必要的工作**。

### 1.3 改生成器/Rust 代码的链路

```
改 Rust 代码 → cargo build（60-120s）→ 重新跑 auto run → 全量重建
```

这个确实慢，因为是编译型语言。但可以大幅优化。

---

## 2. 核心设计：`auto watch` — 分层增量重建

### 2.1 命令设计

```
auto watch [--target vue|rust|all] [--no-backend] [--smoke]
```

启动一个长驻进程，监听文件变化并按"变更类型"触发**最小化**的重建：

| 变更类型 | 触发的重建 | 目标耗时 |
|----------|-----------|----------|
| `src/front/**/*.at` 改动 | 仅重新生成受影响的 SFC + Vite HMR | <1s |
| `pac.at` 改动 | 重新生成项目配置（package.json 等） | <1s |
| `src/back/**/*.at` 改动 | 重新生成 Rust API + 重启后端 | <10s |
| `src/front/**/*.css` 改动 | 无（Vite 直接处理） | <500ms |
| `gen/**/*.vue` 手动改 | 警告：会被覆盖，建议改 .at 源文件 | — |

### 2.2 技术架构

```
┌──────────────────────────────────────────────────────────┐
│  auto watch 进程                                          │
│                                                          │
│  ┌─────────────┐    ┌──────────────┐    ┌────────────┐ │
│  │ notify::    │───▶│ 变更分类器    │───▶│ 重建调度器  │ │
│  │ Watcher     │    │ (debounce    │    │ (按层分发)  │ │
│  │             │    │  100ms)      │    │            │ │
│  └─────────────┘    └──────────────┘    └─────┬──────┘ │
│                                                │        │
│              ┌────────────────────────────────┼───┐    │
│              ▼                ▼                ▼   ▼    │
│         .at 重建层      配置重建层        后端重建层    │
│         (增量 SFC)     (package.json)    (cargo)       │
│              │                │                │        │
│              ▼                ▼                ▼        │
│         通知 Vite HMR    提示用户重启      重启 axum    │
└──────────────────────────────────────────────────────────┘
        ↑                                ↑
        │ 写 .vue 文件                    │ 写 .rs + 重启
        │                                │
┌───────┴────────┐              ┌────────┴───────┐
│ Vite dev server│              │ Rust axum      │
│ (独立进程)     │              │ (独立进程)     │
│ :3000          │              │ :8080          │
└────────────────┘              └────────────────┘
```

### 2.3 关键优化：增量 SFC 生成

当前 `auto build` 重新生成**所有** SFC。但 .at 文件和 SFC 是一一对应的（或一对多），**只重新生成改动过的 .at 对应的 SFC**。

```rust
// 伪代码
fn handle_at_change(path: &Path) -> Result<()> {
    // 1. 检查 .at → SFC 的缓存依赖
    let cache_key = compute_cache_key(path);
    if cache_key == cached_keys.get(path) {
        return Ok(());  // 内容没变（可能是格式化）
    }
    
    // 2. 只重新生成这一个文件
    let component = generate_component_from_file(path, opts)?;
    write_sfc(&component.vue_code)?;
    
    // 3. 更新缓存
    cached_keys.insert(path.to_path_buf(), cache_key);
    
    // 4. Vite 的 HMR 会自动检测 .vue 变化并热更新
    Ok(())
}

// 缓存 key = 文件内容 hash + 相关 .at 的 hash + 生成器版本
fn compute_cache_key(path: &Path) -> String {
    let content_hash = sha256(read(path));
    let deps_hash = sha256(read_dependencies(path));  // use store/api 引用的文件
    let gen_version = GENERATOR_VERSION;  // 生成器本身的版本号
    format!("{}-{}-{}", content_hash, deps_hash, gen_version)
}
```

### 2.4 `GENERATOR_VERSION`：生成器改动的处理

**关键问题**：改了 Rust 生成器代码后，所有已生成的 SFC 都可能过期，但当前没有机制感知。

方案：给生成器引入一个**版本号**，嵌入到生成产物的注释里：

```vue
<!-- Auto-generated from app.at (gen v3.2.1, schema v1.7) -->
```

`auto watch` 启动时记录当前生成器版本。如果重新编译后版本变了，**下一次 watch 触发时全量重建**。

```rust
// 在 VueGenerator 里
pub const GENERATOR_VERSION: &str = env!("CARGO_PKG_VERSION");
// 或更细粒度：
pub const SCHEMA_VERSION: &str = "1.7";  // 手动 bump 当生成格式有破坏性变化
```

---

## 3. 改 Rust 代码的反馈链路优化

### 3.1 `auto watch --dev` 模式

针对"改生成器代码"场景，设计一个特殊的开发模式：

```bash
# 终端 1：cargo watch 自动重编译
cargo watch -w crates/auto-lang -w crates/auto-man -x "build --bin auto"

# 终端 2：auto watch 监听 .at 变化
auto watch --dev
```

`--dev` 模式下，`auto watch` 检测到自身二进制文件更新（通过 mtime）时，触发**全量重建 + 校验 + 冒烟测试**，输出对比报告：

```
[auto watch] auto binary updated, regenerating all...
[auto watch] ✓ 12 SFCs regenerated (3 changed)
[auto watch] diff:
  EditorPanel.vue: 2 lines changed (key logic)
  NavTree.vue: no change
[auto watch] ✓ smoke tests passed (5/5)
```

### 3.2 编译时优化

- **sccache**：缓存 Rust 编译产物，重复构建快 2-3x
- **cargo nextest**：并行测试，比 `cargo test` 快 2-5x
- **增量编译**：确保 `incremental = true` 在 dev profile

---

## 4. 生成器测试金字塔

为了在改生成器时快速发现回归，建立三层测试：

```
            ┌──────────────────┐
            │  E2E 冒烟测试     │  ← 5-10 个，慢（10s），覆盖关键路径
            │  (playwright)    │
            ├──────────────────┤
            │  生成快照测试     │  ← 50+ 个，中速（1s），固定 .at → 固定 SFC
            │  (insta snapshot) │
            ├──────────────────┤
            │  单元测试         │  ← 200+ 个，快（10ms），单个函数
            │  (rust #[test])  │
            └──────────────────┘
```

### 4.1 快照测试（最关键）

对每个 .at 示例，固定它的生成输出：

```rust
#[test]
fn snapshot_editor_panel() {
    let at = include_str!("../../../examples/ui/015-notes/src/front/editor.at");
    let sfc = generate_sfc(at).unwrap();
    insta::assert_snapshot!(sfc);
}
```

改生成器时，`cargo test` 会立即显示哪些 .at 的生成结果变了。人工 review diff 后 `cargo insta accept`。

**这是发现"改 A 坏 B"最快的方式**——比 E2E 快 100 倍，比手动浏览器测试可靠。

---

## 5. 交互式调试：`auto ui repl`

### 5.1 一个新的调试 REPL

针对"改了 .at 但生成结果不对"的场景，提供交互式探查：

```
$ auto ui repl
> load examples/ui/015-notes/src/front/editor.at
Parsed widget: EditorPanel (props: 4, state: 5, handlers: 12)

> show ast
WidgetDecl { name: "EditorPanel", ... }

> show aura
AuraWidget { name: "EditorPanel", view_tree: ..., handlers: {...} }

> gen vue
<AutoDownEditor :content="edit_body" ... />
...

> validate
[R001 ERROR] duplicate-component-key: ...
[R004 WARNING] undefined-handler: EditBody is empty

> trace handler ToggleDarkMode
Handler ".ToggleDarkMode" found in:
  - msg Msg { ..., ToggleDarkMode }
  - on { .ToggleDarkMode -> { store.ToggleDarkMode() } }
  - used in template: button "Dark Mode" onclick:.ToggleDarkMode
```

### 5.2 价值

把"改 .at → build → 浏览器 → 猜哪里错了"的盲调试，变成"加载 → 检查 AST → 生成 → 验证"的白盒调试。尤其适合排查"生成器吃掉了我的某些代码"类问题。

---

## 6. 实施计划

### Phase 1: `auto watch` MVP（2-3 天）
- [ ] 新增 `crates/auto/src/cmd_watch.rs`
- [ ] 用 `notify::RecommendedWatcher` 监听 `src/front/**/*.at`
- [ ] debounce 100ms，变更时只重新生成对应 SFC
- [ ] 集成现有 Vite dev server（HMR 自动生效）
- [ ] 启动时打印"watching X files"

### Phase 2: 增量生成 + 缓存（1-2 天）
- [ ] 实现 `compute_cache_key`（内容 + 依赖 hash）
- [ ] `.auto/build/cache.json` 持久化缓存
- [ ] `GENERATOR_VERSION` 嵌入生成产物注释
- [ ] 生成器版本变化时自动全量重建

### Phase 3: 快照测试基础设施（1 天）
- [ ] 引入 `insta` crate
- [ ] 为 015-notes 的 5 个 .at 写快照测试
- [ ] 为 playground 示例写快照测试
- [ ] 文档化 `cargo insta review` 工作流

### Phase 4: 后端热重载（1-2 天）
- [ ] `auto watch` 监听 `src/back/**/*.at`
- [ ] 检测到变化时重新生成 Rust API 代码
- [ ] 用 `cargo watch` 模式重启 axum 进程
- [ ] 前端自动重连

### Phase 5: `auto ui repl`（3-5 天，可选）
- [ ] 用 `rustyline` 实现 REPL
- [ ] `load` / `show ast` / `show aura` / `gen vue` / `validate` / `trace` 命令
- [ ] 集成 Plan 361 的校验规则

---

## 7. 验收标准

### 反馈时间目标

| 场景 | 当前 | 目标 |
|------|------|------|
| 改 .at 的 style 字符串 | 30-90s | **<1s** |
| 改 .at 的 view 结构 | 30-90s | **<1s** |
| 改 .at 的 handler 逻辑 | 30-90s | **<1s** |
| 改 store 定义 | 30-90s | **<2s**（涉及多组件） |
| 改 Rust 生成器 | 60-120s | **<30s**（sccache + 增量） |
| 改后端 .at | 10-30s | **<5s** |

### 工作流验证

- [ ] `auto watch` 启动后，改任意 .at 能在 1s 内在浏览器看到变化
- [ ] 改生成器代码后，`cargo test` 的快照测试立即显示影响范围
- [ ] `auto ui repl` 能加载 .at 并交互式检查生成过程
- [ ] 015-notes 的完整冒烟测试在 <30s 内跑完

---

## 8. 与 Plan 361 的协同

- Plan 361 的校验规则会在 `auto watch` 的每次重新生成后自动运行，警告实时显示
- Plan 361 的 `generate_component_from_file` 是 `auto watch` 增量生成的基础
- Plan 361 的冒烟测试会在 `--smoke` 模式下于每次重建后运行
