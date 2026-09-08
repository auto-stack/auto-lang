---
plan_id: PLAN-591
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: use-rust-any-crate-direct
author: [zhaopuming, ZCode]
created_at: 2026-09-07
updated_at: 2026-09-07（裁定修订：取消 dep+use 合并/auto 解析，保留 dep 显式声明；V1/V2 两层重划分）

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals:
  - "GOAL-006: Consumer-mode parity——本计划是 GOAL-006 的首个实体化主线：dep 显式声明（version/path 源，既有 Plan 092 通道不动）+ use.rs 消费任意三方 crate，类型覆盖面扩容（字段直读写/trait 单态转发/泛型实例化/反向 adapter 原型），V1/V2 两层推进"
  - "GOAL-003: Auto 作 Rust 脚本层——use.rs 直用三方库是'脚本层体验'的消费面入口，与 442/533 的 musk 后端线互补"

affects: [auto-lang/vm, auto-lang/ffi]  # 受影响的 specs 路径
current_step: 0
total_steps: 6
---

# [PLAN-591] use-rust-any-crate-direct

## 变更摘要

愿景：**`dep a(version: "1.2")` 声明之后，`use.rs a::b::c::D`，D 直接可用**——不管 D
是自由函数、结构体还是泛型方法，不需要手写 shim、不需要 JSON 桥、不需要了解
D 的内部形态。

> **2026-09-07 裁定**：原草案的"V1 触发自动化"（use.rs 未知 crate 自动解析
> workspace→AUTO_DEP_PATH→crates.io + auto.lock 锁版 + 首引用阻塞回填）**取消**。
> 理由：现有 `dep` 声明支持 version/git/path 源配置，显式依赖面有价值，保留
> 两步形态（dep 声明 + use.rs 引用）。本计划聚焦**消费端覆盖面**。

现状（430 管线）已把后端机器造好：dep 声明 → nightly rustdoc 提签名 → 分类/生成
shim 包 → 编译 cdylib → C3 指纹全局缓存 → 运行时装载注册 dispatch（`ffi_dual_013_dep_method`
实测可用）。剩余缺口集中在**覆盖端**：类型字母表停在 `v/i/l/f/b/s/p + 不透明句柄`
——字段不可读、trait 方法不可调、泛型需手工 mono 提示、Option/Result 语义未定
（`types.rs`/`classify.rs` 的 skip 清单）。

本计划按两个能力层推进：

- **V1（结构体透明）**：offset 探针 → 布局 manifest → VM 字段直读写；
  Option/Result marshalling 语义收口（None→null、Err→错误通道）。
- **V2（行为等价）**：trait 单态转发 shim、泛型按需实例化、反向 adapter 原型
  （Rust 回调 .at 闭包）。

理论依据（2026-09-07 musk 会话推导定案）：**编译预言机原则**——签名与布局只存在于
编译期，但管线本就用同一工具链编译目标 crate（同编译保证已由 C3 指纹落地），因此
任何类型问题都可以"再编一段探针代码去问 rustc"（`offset_of!` 支持 repr(Rust)、
单态化实例可按需生成、trait 分发可在 shim 编译期定死）。430 字母表是这栋楼的一层，
不是天花板。

## 目标

1. **G1（V1 结构体透明）**：三方结构体字段可读可写（含乱序 repr(Rust) 字段、
   嵌套结构）；`Option` 返回值 None→null / Some→值；`Result` Ok→值 / Err→VMError；
   字段布局变化指纹失配自动重建；缓存命中零重建。
2. **G2（V2 行为等价）**：trait 方法可调（首个面：`Display`/`ToString`/`Clone`）；
   泛型函数/方法按调用点具体类型自动实例化；反向 adapter 原型（fixture 以
   `Box<dyn Fn>` 回调 .at 闭包并取回返回值）。
3. **G3（工程面）**：每层配套 checked-in 测试（nightly 缺失自动跳过，沿用
   `ffi_dual_013` 模式），fixture crate 进 `test/ffi_dual/`；全量门禁
   `cargo test -p auto-lang ffi_dual` 绿（013 既有用例零回归）。

## 架构方案

### 总原则：构建期制备、运行时装载

"动态调用任意 Rust 库"在 Rust 无稳定 ABI / 无运行时反射约束下的最大可行形态 =
**构建期由编译器本尊制备绑定（签名+布局+marshalling wrapper），运行期按指纹装载**。
V1/V2 各层只是把"制备"的信息面从符号逐步扩到布局、再到行为：

```
dep a(version/path) 显式声明（既有 Plan 092 通道，本计划不动）
   │ ①既有 430 管线（不动）：nightly rustdoc → classify → emit cdylib → C3 指纹缓存
   │ ②新增生成段（本计划）：探针 crate（offset_of!/size_of/enum 判别值）→ manifest v2 布局段
   │ ③新增转发段（本计划）：trait 单态 shim、泛型实例 shim、Option/Result wrapper
   ▼
装载（dep_methods）：manifest 注册 dispatch 3000 兜底段 + FUNCTION_SIGS
   ▼
use.rs a::b::c::D —— D 直接可用（字段/trait 方法/泛型实例）
```

### 分层设计

- **T1 布局层（V1）**：emit 阶段追加生成一个**探针 crate**——对 manifest 中每个
  导出类型编 `offset_of!`/`size_of`/`align_of`（repr(Rust) 合法）与 enum 判别值
  探针（match 计数法），结果写入 manifest v2 新增 `layouts` 段。VM 侧
  `DepOpaqueObject` 字段访问（CALL_SPEC 属性路径）按 layouts 直读直写。
- **T2 语义层（V1）**：`classify` 增 `Option<T>`/`Result<T,E>` 返回规则：
  `Option` → `Some(x)` 压 x / `None` 压 null；`Result` → Ok 压值 / Err 进
  `auto__last_error` 通道转 VMError（430-F unwrap_ok 通道复用扩展）。
- **T3 trait 转发层（V2）**：分类器从"trait impl 全排除"改为**opt-in trait 白名单**
  （首批：`std::fmt::Display`、`ToString`、`Clone`）。对白名单 trait 生成
  `<T as Trait>::method` 单态转发 shim——分发在 shim 编译期定死，无 vtable 跨界。
- **T4 泛型实例层（V2）**：rustdoc 泛型项不再直接 skip：`use.rs` 调用点携带的具体
  实参类型（基元/字符串/已注册句柄类型）回填 mono 提示 → 管线生成
  `foo::<ConcreteType>` 实例 shim（符号名按 430 sig_code 规则稳定生成）。
- **T5 反向 adapter 原型（V2 远端，本计划只做原型）**：fixture 场景——Rust 方法
  形参为 `Box<dyn Fn(i64) -> i64>` 时，生成 adapter 结构体持有 VM 任务句柄，
  trait/fn 调用时重入解释器执行 .at 闭包并取回返回值。原型范围限定进程内单线程
  重入；Send/Sync 纪律与多线程泵归后续计划。

## 需求分析与背景调查

（2026-09-07 实勘，来源：musk 会话全链分析 + 仓内代码/报告）

**已有机器（全部可复用，本计划不重造）**：

| 环节 | 载体 | 状态 |
|:---|:---|:---|
| dep 声明（path/git/version + features） | `compile.rs` `scan_dep_statements`（Plan 092）、`dep_sources`/`dep_features` | ✅（本计划不动，作为显式依赖面保留） |
| 沙箱取料/构建 | `auto-cache/src/sandbox.rs`（`CrateSource`/`DepSource`/`Sandbox`） | ✅ |
| rustdoc 签名提取 | `shim-metadata/src/rustdoc.rs`（nightly JSON v53） | ✅ |
| 分类/跳过规则 | `shim-metadata/src/classify.rs`（generic/closure/Option/exception 四类 skip） | ✅ |
| shim 包生成（C ABI wrapper + manifest） | `shim-metadata/src/emit_cdylib.rs`（shimpack v1） | ✅ |
| 管线编排 + C3 指纹缓存 | `auto-cache/src/methods_pack.rs`（指纹输入含 nightly rustc 版本串） | ✅ |
| 全局构建缓存 | `auto-cache/src/lib.rs`（Plan 082，`~/.auto/cache/` 内容寻址） | ✅ |
| 运行时装载/校验/分发 | `vm/ffi/dep_methods.rs`（`DepOpaqueObject`、METHODS 表挂 dispatch 3000 兜底段、`manifest.fingerprint` 装载校验） | ✅ |
| use.rs 名字解析/懒注册 | `vm/codegen.rs:4922`（Plan 212b）、`native_registry.rs`（Plan 250 懒注册） | ✅ |
| 现有测试范式 | `tests/ffi_dual_tests.rs` `ffi_dual_013_dep_method` + fixture `test/ffi_dual/013_dep_method/fixture/autolang_counter` | ✅ |

**现状能力边界（013 实测 + 类型表实证）**：`Counter.new()` 句柄构造、方法调用、
基元/串返回、builder 链（chain 标记）、static 方法可用；**字段不可读**、trait 方法
（Display 类，013 的 `to_string` 是 fixture 自带 inherent 方法走通的假象）、泛型、
Option 返回均不可用。

**墙的本质（定案，不再重议）**：Rust 类型信息只在编译期存在；布局无跨版本承诺；
`#[repr(Rust)]` 字段顺序未指定。解法不是运行时反射（不存在），而是**同编译保证下
的探针制备**——管线内 `offset_of!`（1.77+ 稳定，支持 repr(Rust)）拿到的就是真实
偏移，指纹保证 manifest 与 cdylib 同源。风险：rustdoc JSON v53 无稳定性承诺 →
指纹已钉 nightly 版本串，装载侧校验拒载（现状）；探针与 wrapper 同管线编译，
布局一致性由同一 rustc 实例保证。

## 详细设计

### D1：探针 crate 与 manifest v2 layouts 段（T1）

emit 阶段追加生成 `probe.rs`（与 shim 包同 cdylib 编译）：

- 每个 manifest 导出 struct：`offset_of!(T, field)` / `size_of::<T>()` /
  `align_of::<T>()` 逐字段导出为 `pub const`；
- enum：判别值用 match 计数探针（每 variant 一臂返回常量）；
- 嵌套类型字段记 `nested: "crate::Inner"` 引用，VM 侧按句柄链读。

manifest v2 新增段（`format: 2`，装载侧 format 校验拒载旧版）：
`layouts: { type_name: { size, align, fields: [{name, offset, ty}], enum_discriminants?: [...] } }`。

VM 侧：`DepOpaqueObject` 增 `layout: Arc<LayoutInfo>`；CALL_SPEC 属性访问命中
DepOpaqueObject 时按 offset 直读（基元/串/嵌套句柄三分派）；**字段写**仅对
`&mut self` 上下文句柄开放（V1 范围：方法返回的可变句柄 + 顶层 `var` 绑定的
句柄字段赋值语句），写后不做 Rust 侧失效通知（单线程纪律，见待澄清 #1）。

### D2：Option/Result 语义（T2）

classify 新增返回类型规则（不改字母表本体，返回码扩展 `?` 前缀）：
`Option<T>` → T 的返回码 + nullable 标记；`Result<T,E>` → T 的返回码 +
fallible 标记（430-F 通道已有 `fallible` 字段雏形）。参数位 `Option<T>`：
null → None / 值 → Some（构造探针 shim 由 wrapper 内联完成）。

### D3：trait 单态转发与泛型实例（T3/T4）

- 分类器 exceptions 增 `trait_whitelist`（首批 `Display`/`ToString`/`Clone`），
  命中即生成 `auto_<Type>__trait_<Trait>_<method>` wrapper（编译期静态分发）；
- 泛型项：调用点 mono 提示从"exceptions 手工表"升级为"调用点实参类型自动回填"
  ——codegen 已知实参的具体类型（基元/串/manifest 类型），缺省规则：基元→对应
  实例；句柄类型→该类型实例；无法推断→保留手工提示路径并给出诊断。

### D4：反向 adapter 原型（T5，原型范围）

manifest 方法条目新增 `callbacks: [{param_idx, fn_sig}]`（形参为 `Box<dyn Fn...>`/
`&mut dyn FnMut...`）。生成 adapter：持有 `AutoTask` 快照 id，call 时经
`vm::scheduler` 重入解释器执行 .at 闭包。**本计划仅原型**：单线程同步重入、
不支持回调内再调三方方法（重入深度 1）、panic 隔离 catch_unwind。多线程泵、
Send/Sync 纪律、生命周期敏感返回（保守 clone 策略已定，激进借用记账）均立
后续计划。

## 测试设计

沿用 `ffi_dual_013` 范式：checked-in Rust 测试 + fixture crate + `.at` 语料，
nightly 缺席自动 skip（全程离线，fixture 均为本地 path 源）。fixture 新增
两个 crate（放 `test/ffi_dual/` 同级 fixture 目录）：

- `autolang_shapes`（V1）：乱序字段 struct（`u8/String/u8/i64/bool` 交错声明）、
  嵌套 struct、`Option`/`Result` 返回方法、字段校验真值函数。
- `autolang_traits`（V2）：`Display`/`Clone` trait impl、泛型自由函数
  `pick<T: Ord>`、泛型方法、`Box<dyn Fn(i64)->i64>` 回调形参方法。

### V1 用例（`014_dep_fields`，目标层 G1）

| # | 用例 | `.at` 语料核心 | 断言 |
|:--|:---|:---|:---|
| V1-1 | 乱序字段读 | `use.rs autolang_shapes::Messy` → `m.a / m.tag / m.count / m.flag` | 与 fixture 真值函数 `messy_truth()` 逐一相等（探针偏移正确性由乱序声明放大） |
| V1-2 | 字段写 | `m.count = 9` 后再读 + 调用 fixture `m.count_snapshot()` | VM 侧读 = 9；**Rust 侧方法读回 = 9**（写真达 cdylib 堆） |
| V1-3 | 嵌套结构 | `o.inner.tag / o.inner.n` 两级句柄链 | 真值相等 |
| V1-4 | Option 返回 | `find(key) -> Option<&str>`：存在键 / 缺失键 | 前者返回串；后者 VM 侧为 null（`is_none` 判定通过） |
| V1-5 | Result 返回 | `parse(s) -> Result<Point, String>`：合法/非法输入 | 合法 → 字段可读；非法 → VMError 且 message 含 Err 串 |
| V1-6 | 布局指纹防 stale-read | fixture `Messy` 加字段后重跑 V1-1 | 旧缓存拒载/重建，新字段读通；同指纹连跑第二次零重建（构建计数器/mtime 断言） |
| V1-7 | enum 判别 | `shape_kind() -> Kind` + `is_circle()` 判别探针 | 判别值与 match 语义一致 |

### V2 用例（`015_dep_traits_generics`，目标层 G2）

| # | 用例 | `.at` 语料核心 | 断言 |
|:--|:---|:---|:---|
| V2-1 | trait 转发（Display） | `t.fmt()`（fixture `Temp` 仅 impl Display，无 inherent to_string） | 输出与 `format!("{}", temp_truth)` 相等 |
| V2-2 | Clone 转发 | `t2 = t.clone__trait()` 后改 t | t2 不受影响（深拷贝语义） |
| V2-3 | 泛型自由函数实例化 | `pick(3, 7)` / `pick("a", "b")`（`fn pick<T: Ord>`） | i64 与 str 两实例均正确（mono 自动回填） |
| V2-4 | 泛型方法实例化 | `Pair<i64>.max()` / `Pair<str>.max()` | 两实例正确 |
| V2-5 | 复合返回可读 | `make_point() -> Point`（按值复合类型返回） | 返回句柄字段直读（V1×V2 组合） |
| V2-6 | 反向 adapter 原型 | `apply(5, |x| x * 2 + 1)`（形参 `Box<dyn Fn(i64)->i64>`） | 返回 11（.at 闭包被 Rust 回调且返回值流回） |

### 门禁与回归

- 全量：`cargo test -p auto-lang ffi_dual`（含 013 既有用例零回归）；
- 离线 CI 可全跑（fixture 全部本地 path 源，无网络依赖）；
- 手册同步：`docs/guides`/`syntax.md` 的 use.rs 节在 V1 收口时补字段访问语法与
  Option/Result 语义说明。

## 验收标准

1. **V1**：V1-1..V1-7 全绿；manifest v2 `layouts` 段装载校验生效（旧 format 拒载）；
   013 既有用例零回归。
2. **V2**：V2-1..V2-6 全绿（V2-6 允许标记 experimental，单线程重入限制写入文档）。
3. **文档**：use.rs 能力矩阵（V1 字段透明 / V2 行为等价）+ Option/Result 语义进
   syntax.md 或 guides；GOAL-006 状态由"规划中"推进留痕。
4. **债务登记**：反向 adapter 多线程泵 / 借用记账 / Send-Sync 纪律 / 字段写失效
   语义（若裁定为已知限制）进 KNOWN-DEBT-AND-RISKS.md。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **T1-探针生成**：`shim-metadata/src/emit_cdylib.rs` 增 probe 段生成 + manifest
   v2 `layouts`；`shim-metadata/src/types.rs` 布局类型。验证：`shim-metadata
   shim-emit-pack` 对 fixture 产出的 manifest 含 layouts 且偏移单测对拍。
2. **T1-VM 字段读写**：`vm/ffi/dep_methods.rs` `DepOpaqueObject` 布局挂载 +
   CALL_SPEC 属性读写三分派。验证：V1-1/V1-2/V1-3。
3. **T2-Option/Result**：`shim-metadata/src/classify.rs` 返回规则 + wrapper 生成 +
   VM 错误通道接 `fallible`。验证：V1-4/V1-5/V1-7。
4. **V1 测试落位**：fixture `autolang_shapes` + `test/ffi_dual/014_dep_fields/` +
   `src/tests/ffi_dual_tests.rs` 增 014 组（含 V1-6 缓存/指纹不变量）。验证：
   `cargo test -p auto-lang ffi_dual_014`。
5. **T3/T4-trait 与泛型**：classify trait 白名单 + mono 自动回填 + wrapper。
   验证：V2-1..V2-5。
6. **T5-反向 adapter 原型 + V2 测试落位**：fixture `autolang_traits` +
   `test/ffi_dual/015_dep_traits_generics` + adapter 生成段。验证：
   `cargo test -p auto-lang ffi_dual_015`；全量门禁 + 文档/债务登记收口。

## 复审记录

## 待澄清事项

1. **字段写的失效语义**：VM 侧写字段后，若 Rust 侧缓存了旧值（fixture 内
   `OnceLock` 类）不刷新——单线程纪律下是否可接受为 V1 已知限制（建议：是，
   登记 KNOWN-DEBT）。
2. **T5 原型边界**：单线程重入深度 1 的限制是否满足 musk 场景前置评估需要，
   还是需要直接上多线程泵（建议：原型先行，musk 消费评估后另立计划）。
3. **测试编号回改（2026-09-09，PLAN-592 执行期注记）**：本计划 §测试设计中的
   `014_dep_fields`/`015_dep_traits_generics` 与现存
   `014_std_generated_segment`/`015_musk_backend_wave1` **撞号**；且 016/017
   已被 PLAN-592（dep-rust-parity-matrix）占用。落地时改用 **018/019**。
   PLAN-592 已交付可复用资产：三轨 runner `ffi_dep_parity_tests.rs`
   （VM/a2r/oracle 对拍，`{{FFI_DUAL_DIR}}` 占位语料）、fixture 扩展件
   （autolang_counter drop 计数）、P0 修复集（i8/i16 cast、返回符号扩展、
   堆感知弹参、自由函数 wrapper 装载链、GET_FIELD 字段桥接、未覆盖签名
   报错）——V1 用例应在同一 runner 上扩展而非另起炉灶。

> 已裁定（2026-09-07）：~~V1 触发自动化（use.rs 自动解析 + auto.lock + 首引用
> 阻塞回填）~~ **取消**——`dep` 声明的 version/git/path 源配置有价值，保留
> 两步形态；原 V2/V3 顺位上移为 V1/V2。
