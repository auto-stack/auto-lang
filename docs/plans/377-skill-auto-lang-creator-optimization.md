# Plan 377: auto-lang-creator 技能优化（基于 plan 013/373/376 的移植经验）

> **状态**：✅ 可启动（2026-07-31：所有前置条件已满足，re-transpile 达到 0 错误）
> **仓库**：auto-lang（技能位于 `D:/autostack/skills/auto-lang-creator/`）
> **前置**：plan 013（Auto 移植 G1-G4 全部达成）、plan 373（B1 细节）、plan 376S/T/U/V/W（re-transpile 136→0）
> **目标**：把 plan 013→373→376 过程中积累的 Auto 代码编写经验回灌到
> `auto-lang-creator` 技能，使将来生成的 Auto 代码从一开始就避免这些问题。

## 背景

plan 013 把 auto-ai 的 3 个 Rust crate 用 Auto 语言复刻（`.at` 文件），
plan 373 修了 343 个 a2r codegen 错误（手修 + post_process），plan 376S/T/U/V/W
把 a2r 生成器 + .at 源码持续改进到 **re-transpile 0 错误**（从 136 降到 0），
并实现了 lib.rs 自动生成。Auto 版 ReAct 端到端跑通（GLM-5.2 真实回答 + echo 工具调用）。

这些工作暴露了大量"Auto 代码编写时应该注意的模式"——它们目前散落在
计划文档、commit messages 和 a2r post_process 函数里。本计划的目标是
把这些经验系统化地整理进技能，使将来的代码生成直接产出正确模式。

## 三方对比方法

### 数据来源

| 对比项 | 来源 | 说明 |
|---|---|---|
| **Auto 源码（原始版）** | `git show 0c2c630b:crates/auto-ai-agent/src/*.at` | plan 013 首次移植的手写版本（未经修复） |
| **Auto 源码（修复后）** | master HEAD 的 `crates/auto-ai-agent/src/*.at` | 经 plan 373/376 修复后的版本 |
| **Rust 原版** | `auto-ai/crates/auto-ai-agent/src/*.rs` | 手写 Rust 参照基准 |
| **a2r 差异** | `git diff 0c2c630b HEAD -- crates/auto-ai-agent/src/*.at` | 原始版 → 修复版的所有 .at 源码变更 |

### 分析流程

```
1. 提取所有 .at 源码变更（git diff）
2. 分类每处变更：
   A. "Auto 代码编写规范"（应在技能中教）
   B. "a2r 生成器缺陷"（应修生成器，非技能问题）
   C. "桥接类型 API 差异"（应在技能中记录）
3. 把 A 类和 C 类经验整理进 auto-lang-creator 技能
```

## 已知的经验分类（待系统化整理）

### A. Auto 代码编写规范（应在技能中教）

从 plan 373/376 的修复中，以下模式**必须在 Auto 代码中显式写**：

| 模式 | 错误症状 | 正确写法 | 为什么 |
|---|---|---|---|
| `.len()` 返回 `usize` | E0308 usize/uint | `s.len() as uint` | Rust 的 `.len()` 返回 usize，Auto 的 `uint` 是 u32 |
| `for m in self.field`（&self 方法中） | E0507 move | `for m in self.field.clone()` | &self 不能 move 字段 |
| `names.push(str_param)` | E0308 String/&str | `names.push(name.to_string())` | `str` 是 &str，Vec 需要 String |
| `HashMap.get(key)` 用作值 | E0308 Option | `self.map.get(key) ?? default` 或 `.clone()` | `.get()` 返回 Option<&T> |
| `is self.cfg.field`（两层 self） | E0507 move | `self.cfg.field ?? default` 或 bind 局部 var | a2r 的 auto-clone 只覆盖 `self.X` 一层 |
| `Some(map.get(key))` | E0308 Option 嵌套 | `Some(map.get(key).clone())` | `.get()` 返回引用，需 clone |
| `pair[0]` tuple 索引 | E0608 | `pair.0` | Auto tuple 用 `.N` 不用 `[N]` |
| `step.id` 多次使用 | E0382 use after move | `step.id.clone()`（除最后一次使用） | String 是 owned，move 后不可再用 |
| `e.message()` on 外部类型 | E0599 | `e.to_string()` 或 `format!("{}", e)` | 外部 crate 类型没有 Auto 的 `.message()` 方法 |
| `is last_handoff { None -> "" }` | E0308 match arms 不一致 | `None -> Str.new()` | `""` 是 &str，`Some(h) -> h.from` 是 String |
| `c.load()` + `v != 0` | E0277 bool/int 比较 | `c.load()` + `v` 直接用 | AtomicBool.load() 返回 bool |
| `soft_limit * 5`（uint * int 字面量） | E0308 uint/int 混用 | `soft_limit * 5u`（用 uint 字面量） | 默认字面量是 int，需显式 `5u` |
| `s.len() as i32` 残留 | E0308 i32/u32 | 避免 `as i32`，用 `as uint` 或不 cast | a2r 的 `as i32` 习惯导致类型不匹配 |
| **局部变量与模块导入同名** | E0433 scope 错误 | 避免用 `agent`/`error`/`handoff` 等模块名做局部变量名 | a2r 误判为模块路径，渲染 `agent::method()` 而非 `agent.method()` |
| **函数体不支持 `::` 路径表达式** | 解析失败 | 不写 `std::time::SystemTime::now()`，改用 a2r 已知方法 `time.now_sec()` | Auto 函数体不解析 `::`，需走 a2r 的 stdlib 方法分发 |
| **`use.rust` 不能导入 const** | 解析失败 | 不写 `use.rust std::time::UNIX_EPOCH`（const），只导入 type | const 导入后仍报 `undefined variable` |
| **spec 类型进 Arc 需先装箱** | E0308 | `Arc(role)` 的 role 是 `has Role` 类型时，写 `Arc(Box(role))` | 需先 `Box` 成 `Box<dyn Trait>` 才能进 `Arc` |
| **Arc 内的值需解引用** | E0308 | `build_x(registry)` 的 registry 是 `Arc<T>` 时，写 `build_x((*registry).clone())` | `Arc<T>` 不是 `T`，需解引用再 clone |
| **返回 &str 切片但签名是 str** | E0308 | `fn f() str { return s.trim() }` → 补 `.to_string()` | a2r 把 `str` 返回类型渲染为 Rust `String`，但 `trim()` 返回 `&str` |
| **`pair.0.as_str()` 链式点丢失** | E0308 | 用 `pair[0].as_str()`（经 fix_tuple_index 转换保留 .as_str） | a2r 渲染 `pair.0.as_str()` 时丢失 `.as_str()`（链式点解析 bug） |

### B. a2r 生成器缺陷（不教技能，应修生成器）

这些是 a2r 生成器的 bug，不应在技能中教用户绕开：

| 缺陷 | 已有修复 | 状态 |
|---|---|---|
| `has Spec` 生成空 `ToolTrait` | plan 373 G2: 生成真正的 `impl Trait for Type` | ✅ |
| `~Result` 方法不标 async | plan 373 G2: `Type::Result` → `async fn` | ✅ |
| 多余 `.as_str()` on &str | plan 376B P7: 检查 `local_var_types` | ✅ |
| `#[async_trait]` 未加到 impl | plan 373 G2: `has_async` 检测 | ✅ |
| `&mut self` 未检测 | plan 373: `method_mutates_self` 扫描 | ✅ |
| `.await` 未自动插入 | plan 373: `fn_ret_types` + `call_needs_await` | ✅ |
| ContentBlock struct-variant | plan 373: `seed_known_struct_enum_variants` | ✅ |
| `(Uint, Int)` unify 返回 Int | plan 376G: 改为返回 Uint | ✅ |
| `dyn Trait` derive Clone/Debug 失败 | plan 376V: `fix_dyn_trait_derives` 降级为 `#[allow(dead_code)]` | ✅ |
| `Box<dyn Trait>` 未装箱 | plan 376V: `fix_spec_trait_boxing`（`Some(X{})` → `Some(Box::new(X{}))`） | ✅ |
| PathBuf 无 `.as_str()` | plan 376V: `fix_pathbuf_as_str`（→ `.to_str().unwrap()`） | ✅ |
| fn 字段调用 `self.on_event(x)` | plan 376V: `fix_fn_field_calls`（→ `(self.on_event)(x)`） | ✅ |
| tuple 索引 `pair[0]` | plan 376V: `fix_tuple_index`（→ `pair.0`） | ✅ |
| fs Result 嵌套括号 | plan 376W: `fix_a2r_std_fs_result_patterns`（`[^)]*`→`.*?`） | ✅ |
| enum `#[derive]` 不生效 | plan 376S: `EnumDecl.attrs` + parser 分支 | ✅ |
| lib.rs 不自动生成 | plan 376U: `A2R_CRATE_ROOT=1` → `pub use` + shims 注入 | ✅ |

### C. 桥接类型 API 差异（应在技能中记录）

| 差异 | 说明 | 正确写法 |
|---|---|---|
| `auto_val::Node` 不可 deref | a2r 生成 `*(*node).clone()` | `node.clone()` |
| `ClientError` 无 `.message()` | 用 `thiserror` 的 Display | `format!("{}", e)` |
| `PathBuf` 无 `.as_str()` | 用 `to_str().unwrap()` 或 `to_string_lossy()` | `path.to_str().unwrap()` |
| `a2r_std::fs::read_to_string` 返回 String 非 Result | 桥接函数签名差异 | 用 `Ok()` 包装或不 match |
| `HashMap.get()` 返回 `Option<&T>` | Rust 标准库 API | `?? default` 或 `.clone()` |

## 实施步骤

### 步骤 1：提取完整 diff（plan 376 完成后）

```bash
cd D:/autostack/auto-lang
git diff 0c2c630b HEAD -- crates/auto-ai-agent/src/*.at > /tmp/at_changes.diff
# 分类每处变更（A/B/C）
```

### 步骤 2：更新 auto-lang-creator 技能

在 `D:/autostack/skills/auto-lang-creator/skill.md` 中：

1. **新增"Rust→Auto 移植 gotchas"节**：把 A 类经验（21 条，含 376V/W 新增 8 条）写成明确的规则
2. **更新"桥接类型"节**：把 C 类经验（5 条）加入已知桥接 API 差异列表
3. **更新"常见错误"节**：把最容易犯的错误排到前面

### 步骤 3：验证

用一个简单的 Rust 文件（如 `auto-ai/crates/auto-ai-agent/src/error.rs`），
让技能（或手工）重新生成 `.at`，检查是否避免了已知问题。

### 步骤 4：更新 a2r 生成器（B 类）

确认 plan 373/376 的 B 类修复都在 post_process 链或生成阶段中，
不需要在技能里教用户绕开。

## 依赖

- ~~plan 376 re-transpile 错误降到 ~100 以下（当前 180）后再启动~~
  **✅ 已满足（2026-07-31）：re-transpile 达到 0 错误，经验数据已沉淀完毕。**
- ~~每次 .at 修复都在产生新的经验，提前做会漏掉后续~~
  **经验积累已趋于稳定（376V/W 是最后一批大批量修复）。**

## 预期效果

- 技能生成的 Auto 代码减少 ~50% 的 a2r 编译错误（A 类 + C 类经验）
- 移植新 crate 时不需要重走 plan 373 的 343→0 手修过程
- a2r 生成器的 post_process 链进一步简化（因为源码侧更规范了）
