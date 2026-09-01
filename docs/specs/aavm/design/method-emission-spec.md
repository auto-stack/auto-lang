# 方法发射规格（Plan 514 W0 考古，2026-09-01）

> 宿主（主 a2r = `crates/auto-lang/src/trans/rust.rs`）方法族语法发射规格与已知洞清单。
> 本文件是 W2（AA2R 方法发射）的**对齐基准**：AA2R 产物必须与主 a2r live 输出逐字符一致。
> 全部样例为 2026-09-01 master `77393d70b`（target/debug/auto.exe）实测。

## 1. 语法形态 → Rust 发射（live 对齐表）

参照样例：`examples/playground-demo/13-methods.at`（转译产物已实测）。

| Auto 形态 | Rust 发射（主 a2r live） | 备注 |
|---|---|---|
| `type T { 字段...; fn m() ... }` | `#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)] struct T { pub 字段... }` + `impl T { ... }` | 体内方法与 ext 方法**合并进同一个 impl 块**，按源码顺序 |
| 实例方法 `fn get() int { return .value }` | `fn get(&self) -> i64 { return self.value; }` | `.field` 简写 → `self.field` |
| 显式 `self.x` | `self.x` 直发 | 双形态等价（arch2 实测 `fn get() { return self.v }`） |
| 字段写 `.value = ...` | `fn m(&mut self)`，体内 `self.value = ...` | &mut 经 scan_mutated_bindings 按字段写检测（arch1：`inc`/`inc_mut` 均 `&mut self`） |
| `static fn new(v int) Counter` | `fn new(v: i64) -> Counter`（无 self） | static 免接收者 |
| 构造 `Counter(v)`（位置构造） | `Counter { value: v }`（字段名展开） | 见 13-methods `new` 体内 |
| static 调用 `Counter.new(10)` | `Counter::new(10)` | **live 怪癖**：`let c = Counter::new(10);;` 双分号（语句已带 `;`，发射再补一个） |
| 实例调用 `c.get()` | `c.get()` | 接收者直发 |
| 方法链 `a.add(b).get()` | `a.add(b).get()` | arch2 实测 |
| 方法内调方法 `.helper(3)` | `self.helper(3)` | arch1 实测 |
| `ext Counter { fn double() ... }` | 并入 `impl Counter { ... }` 同一块 | 13-methods 实测：double 与体内方法同 impl |
| List 元素方法 `xs[1].get()` | `xs[1].clone().get()` | 索引访问带 `.clone()`（既有 a2r 索引克隆惯例，arch11） |
| `for c in cs { c.get() }`（`cs List(Counter)`） | `for c in &cs { c.get(); }` | 泛型参数列表 for-in 按 `&` 引用迭代（arch11） |
| 方法值传参 `getter(a)`（闭包捕获） | `getter(a)`；闭包 `fn(x Box) int` → `\|x: Box\| { ... }` | arch2 实测 |
| `let` 后接突变调用 | `let mut a = ...` | let mut 自动推断（arch3 实测） |

### f-string 注意（探针书写规范）

插值语法是 `${...}`（带 `$`），裸 `{...}` 是字面量。VM 与 a2r 行为一致。
99_idiom2 探针必须用 `${}` 形态。

## 2. VM 侧执行语义（实测）

- `type` 体内方法 + `ext` + static 构造 + 方法链：VM 全部正确执行
  （arch6 嵌套 fn 有返回值形态 `42` ✅；arch11 `sum=6/direct=2` ✅；
  13-methods 为 playground 常跑样例）。
- `Stmt::Ext`（`vm/codegen.rs:2402`）+ type 体内方法同路径编译。
- 嵌套 fn **void 形态静默失效**（见 §4 债②）。

## 3. 已知洞清单（W1 修复输入）

### 洞 A（P511-5）：独立二进制转译 `File::read_text` 缺映射

- **现象**：五方矩阵②腿（aavm2_bin）编译红，`error[E0433]: cannot find type File`×3
  （merged.rs:5874/5921/5946，源自 `auto/lib/codegen.at:2487/2534/2561`）。
- **根因**：`trans/rust.rs` 的 `(Ident(obj), Ident(method))` stdlib 模块分派
  （`"env"`/`"fs"`/`"json"`... 臂，~:5300）没有 `"File"` 模块臂；
  `fs.read_text` 有 `a2r_std::fs::read_text` 映射而 `File.read_text` 直落。
- **定案（W0 裁定，步骤 3 实施方案）**：**std 直映 inline shim**——
  `File.read_text(x)` → `std::fs::read_to_string(x).unwrap_or_default()`。
  理由：(a) 三方语义一致：VM shim（`ffi/stdlib.rs:296` `read_to_string(&p).unwrap_or_default()`）
  与 `a2r_std::fs::read_text` 同为"错误→空串"；(b) merge 模式（②腿）不注入
  a2r_std import 且其 Cargo deps 为空，`a2r_std::` 路径无法链接，std 直映零依赖；
  (c) 与 429-B1 shim 体系同构。同族 `File.exists`/`File.write_text` 顺带补齐
  （VM shim 同语义：`metadata().is_ok()` / `fs::write`）。
- **预置先例**：`("File", "delete")` 元组臂已存在（rust.rs:5187/:7263）。

### 洞 B：内建方法名遮蔽用户类型方法（探针实测新发现）

- **现象**：`type Box { fn set(n int) {...} }` + `a.set(10)`（a: Box）被误发射为
  `a.insert(10 as usize)`——List/Map 内建重映射命中了非数组/Map 接收者。
- **根因**：`rust.rs:7543` `"set" => Some("insert")` 无接收者类型检查；
  `"append"` 臂已有 Plan 393 E1 的"已知 struct 直通"先例（:7525），`"set"` 未套用。
- **修复口径（W1 步骤 4）**：镜像 Plan 393 E1——接收者是已知用户类型
  （`Type::User/Tag/Enum/GenericInstance`）时直通方法名；同理审视
  `"get"`/`"push"`/`"len"` 等高频遮蔽名。

### 债①（447-旁支）：嵌套 fn void 形态静默失效

- **复现**（arch7）：
  ```auto
  fn main() { fn inner() { print("hi from inner") } inner() print("after") }
  ```
  VM 输出只有 `after`——`inner()` 调用无输出无报错。
- **边界**：有返回值且在表达式位使用的嵌套 fn（arch6 `helper(21)`→42）正常；
  失效集中在 void 嵌套 fn 语句调用路径。修复定位 `parser.rs`/`vm/codegen.rs` 嵌套路径。

### 债②（447-旁支）：`struct` 误用报 E0201 名字解析错

- **复现**（arch10）：fn 体内 `struct Val { x int }` 后 `return Val { x: 2 }` →
  `auto_name_E0201: Variable 'Val' is not defined`。
- **误导性**：真实问题是 `struct` 非 Auto 关键字（具名结构声明实为 `type`）；
  应在 `struct` 词法位报语法错。修复定位 `parser.rs check_symbol`（:11225）。
- **顺带发现**：顶层 `struct Frame {...}` 语句当前被**静默丢弃**
  （VM 与 a2r 均无报错，arch8/arch9）——改报语法错后此形态一并覆盖。

## 4. AA2R（a2r.at）对照现状（W2 输入）

- `ar_prescan_type`（a2r.at:655）：遇 `Fn`/`Static`/`Hash` 的 type 体内成员直接
  `v2 unsupported`——方法表零起点。
- `ar_run`（:3014）顶层分派仅 Fn/Type/Enum，无 Ext 分支。
- 可复用杠杆：`ar_emit_fn2`（:2851 方法体复用）、`ar_scan_mutations`
  （:784 &mut 判定）、`ar_vpush`（:347 self 入作用域）、`ar_method_call`。
- 对齐目标：主 a2r `ext_decl`（rust.rs:16052）、`fn_decl`（:12288，
  `fn.parent.is_some()` 判方法身份 :64/:74、接收者合成、static 免）、
  `.field`→`self.field`（:11500）、type 体内方法与 ext 共用发射族（:12330）。
