# Plan 013 交接摘要（用于新会话续开发）

## 一句话状态

将 auto-ai 的 3 个 Rust crate 用 Auto 语言复刻。阶段 1（ai-config）+ 阶段 2（auto-ai-client）**全部完成并通过 cargo check**；阶段 3（auto-ai-agent）已完成 **24/26 文件**，剩余 agent.rs(918行) + workflow.rs(1181行) + roles.rs + skill.rs + workflow_validator.rs + orchestration/*(5文件) + lib.at/mod.at 共 **~4500 行待移植**。

## 仓库与分支

- **工作目录**：`D:\autostack\auto-lang\.worktrees\plan-013-b16`（master 分支，含全部 a2r 修复）
- **Rust 原版参考**：`D:\autostack\auto-ai\crates\`
- **Auto 语法指南**：`D:\autostack\skills\auto-lang-creator\skill.md`
- **计划文档**：`D:\autostack\auto-lang\.worktrees\plan-013-b16\docs\plans\013-auto-ai-port-to-auto.md`

### 构建

```bash
cd D:/autostack/auto-lang/.worktrees/plan-013-b16
cargo build --release --bin auto   # auto.exe 用于 transpile 和 VM 运行
```

### 验证方法

```bash
# transpile 一个 .at → .a2r.rs
./target/release/auto.exe trans --path crates/<crate>/src/<file>.at rust

# AutoVM 运行（纯 Auto 文件才有效；a2r-first 文件会报桥接错误，正常）
./target/release/auto.exe crates/<crate>/src/<file>.at

# cargo check 验证（需在 workspace 内创建临时 crate，含 ai-config/auto-val 等依赖）
```

## 已完成的文件清单

### 阶段 1：ai-config（6 文件，全部 cargo check 0 错误）✅

`crates/ai-config/src/`：tier.at, wire.at, provider.at, loader.at, validate.at, lib.at

### 阶段 2：auto-ai-client（3 文件）✅

`crates/auto-ai-client/src/`：error.at, daemon.at, lib.at

### 阶段 3：auto-ai-agent（24 文件已移植）✅

`crates/auto-ai-agent/src/`：
- **基础层**：error.at, role_def.at, relay.at, tool.at, memory.at, validate.at
- **builtin_roles/**（16 文件）：mod.at + assistant/coder/architect/tester/reviewer/documenter/advisor/planner/gofer/super_advisor/super_coder/super_tester/runner/translator.at
- **config/**（2 文件）：mod.at, role_config.at

## 待移植文件（按优先级）

| 文件 | Rust 行数 | 说明 | 复杂度 |
|---|---|---|---|
| `roles.rs` → `roles.at` | 395 | RoleRegistry（用 Map<str, Role> + names 键表） | 中 |
| `skill.rs` → `skill.at` | 476 | Skill/SkillRegistry/SkillTool | 中 |
| `workflow_validator.rs` → `workflow_validator.at` | 192 | 校验 workflow 步骤 | 低 |
| `agent.rs` → `agent.at` | 918 | **核心 ReAct 循环**（async、tool-calling） | 高 |
| `workflow.rs` → `workflow.at` | 1181 | workflow 引擎（deprecated） | 高 |
| `orchestration/mod.at` | 30 | 模块导出 | 低 |
| `orchestration/budget.at` | 166 | token 预算跟踪 | 中 |
| `orchestration/flow.at` | 162 | 流程定义 | 中 |
| `orchestration/handoff.at` | 225 | 角色交接 | 中 |
| `orchestration/pipeline.at` | 502 | pipeline 引擎 | 高 |
| `orchestration/driver.at` | 432 | pipeline 驱动 | 高 |
| `lib.at` | — | crate 根（re-export） | 低 |

## 关键 Auto 语法规则（移植时必须遵守）

### 必须遵守的（否则 transpile/解析失败）

1. **构造体返回必须 `return`**：`fn foo() Type { return Type(...) }` 不能省 `return`
2. **`use <stdlib>` 会报错**：不要写 `use json`/`use http`/`use fs`，直接全局调用 `json.parse()`/`http.request()`/`fs.exists()`
3. **`||` / `or` 在 if/for 条件里不可用**：用嵌套 `if/else` 替代
4. **`is` 分支不支持多语句块体**：`is x { Some(v) -> { stmt1; stmt2 } }` 会解析失败；用 `??` 提取值 + 函数级逻辑
5. **`is` 分支里的局部赋值失败**：`Some(v) -> limit = v` 不行；`Some(v) -> cfg.x = v`（字段赋值）可以
6. **`pub const` 不支持**：用公开函数返回常量值
7. **`routes`/`route` 是保留关键字**：不能用作字段名
8. **`ext Type has Spec { ... }` 不被解析**：必须用 `type X has Spec { fields + methods }` 内联实现 spec，非 spec 方法放 `ext X { ... }`
9. **VM Map 无 iteration API**：`for k,v in map` 静默产出 0 项；用并行 `List<str>` 键表
10. **跨文件 `use` 在独立 VM 运行不可见**：`use role_def: Role` 在 `auto a.at` 单独运行时报 Module not found；但 transpile 时正确译为 `use crate::role_def::Role`

### 应当遵守的（改善 a2r→Rust 质量）

11. **所有公开类型/枚举用 `pub type`/`pub enum`**：a2r 需要显式 `pub` 才能跨模块
12. **所有公开字段用 `pub`**（a2r 已在 standalone 模式自动加，但源码侧声明也好）
13. **`byte(u8)` 赋给 `int(i32)` 需改类型或手动转换**：order() 类函数直接返回 `int`
14. **桥接类型（auto_val 的 AutoStr）边界加 `.to_string()`**：auto_val 返回 AutoStr，非 String
15. **`HashMap.get(key)` 返回 Option**：用 `is result { Some(x) -> ... }` 解构，或 `?? default`

### async 映射

- `async fn foo()` → `pub fn foo() ~Result<T, E>`
- `.await` 保留 `.await`
- 调用：`client.complete(req).await`

### trait 对象

- `Arc<dyn Tool>` → `Arc<Tool>`（**尖括号**，不是圆括号！）
- `Box<dyn Role>` → `Box<Role>`
- a2r 生成 `Arc<Box<dyn Tool>>`（多一层 Box，功能正确）
- `Map<str, Arc<Tool>>` 作为字段类型可以正常 transpile

### spec 实现

```auto
// 正确：内联 type + has Role
pub type ConfigRole has Role {
    cfg RoleConfig
    base ?Role

    pub fn name() str {
        is self.cfg.name {
            Some(n) -> return n,
            None -> return self.base_name()
        }
    }
    // ... 其他 Role 方法 ...
}

// 正确：单独的 ext 块放非 spec 方法
ext ConfigRole {
    fn base_name() str { ... }
}
```

### 桥接文件（a2r-first）模式

当 .at 文件需要用 Rust crate 的类型（如 auto_atom/auto_val）时：

```auto
dep auto_atom
use.rust auto_atom
dep auto_val
use.rust auto_val
// 这些行让 a2r 生成 use auto_atom::*; use auto_val::*;
// AutoVM 无法运行此类文件（桥接类型未知），但 transpile + cargo check 可用
```

## a2r 已修复的 codegen 问题（本计划修的）

以下 a2r 修复已在 master 上（通过合并 `plan-013/a2r-b1-fixes` 分支 + 后续直接提交）：

1. enum derive 补 Eq/PartialOrd/Ord（安全时）
2. self.field 返回补 .clone()（E0507）
3. 本地类型不误加 crate 前缀（local_struct_types 预扫描）
4. for-loop 迭代器方法调用不加多余的 `&`
5. a2r_std 前导用裸路径（非 `auto_lang::a2r_std`）
6. Err 具体枚举错误不套 Box::new
7. Err(Ident) 重抛不套 Box
8. Some(int) 返回 ?uint 时加 `as u32`
9. Map.get 自动借用（仅 owned-String 参数）
10. 结构体字段 standalone 加 pub
11. 桥接 crate glob import（auto_val::*; auto_atom::*;）
12. Cover(Tag) 桥接绑定记录 + `*(*x).clone()` 双重 deref

## 新会话需要做的事

1. 读本文件了解全部上下文
2. 读计划文档 `013-auto-ai-port-to-auto.md` 了解完整缺陷记录
3. 读 `auto-lang-creator/skill.md` 了解 Auto 语法
4. 从待移植文件表按优先级开始移植
5. 每个文件写完后用 `auto trans --path <file> rust` 验证 transpile 通过
6. 建议先做 roles.rs(中) 和 skill.rs(中)，再做 orchestration/*，最后做 agent.rs(高) + workflow.rs(高)
7. 完成后写 lib.at 收尾
8. 全部完成后提交，更新计划文档状态
