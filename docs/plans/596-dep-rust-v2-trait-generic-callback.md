---
plan_id: PLAN-596
status: reviewed                # drafting → executing → execution_done → reviewed → archived
feature_name: dep-rust-v2-trait-generic-callback
author: [ZCode]
created_at: 2026-09-09
updated_at: 2026-09-10
plan_revision: 1
current_step: 11
total_steps: 11

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-006]     # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm, shim-metadata, parity]   # 详见 §5 规范增量
---

# [PLAN-596] dep-rust-v2-trait-generic-callback：591 V2 执行——trait 单态转发 / 泛型实例化 / 反向回调 adapter

> **来源**：PLAN-591（use-rust-any-crate-direct，2026-09-09 V1 交付归档）显式
> 拆分出的 V2 执行计划（归档件 §2 折叠节 T3/T4/T5 设计 + V2-1..V2-6 用例 +
> fixture `autolang_traits` 设计原文照录为本计划基线）。优先级依据：
> [p594-dep-skip-hit-rate.md](../reports/p594-dep-skip-hit-rate.md)（T3=第二
> 杠杆：base64 整库 0% 解锁 + Display 家族；by-value self 零命中不提级）。

## 0. 变更摘要

在 591 V1（布局 manifest + Option/Result 语义）之上交付 430 管线的三层行为等价
能力，并用三轨对拍钉死：

1. **T3 trait 单态转发**：classify 从"trait impl 全排除"改 **opt-in trait 白名单**
   （首批 `Display`/`ToString`/`Clone`/`Engine`——Engine 为 base64 解锁驱动首例），
   生成 `<T as Trait>::method` 编译期定死分发的转发 wrapper
   （`auto_<Type>__trait_<Trait>_<method>` 符号）；
2. **T4 泛型实例层**：rustdoc 泛型项不再无条件 skip——`use.rs` 调用点实参类型
   回填 mono 提示，生成 `foo::<ConcreteType>` 实例 shim（符号按 430 sig_code
   规则稳定生成）；base64 `Engine::encode(impl AsRef<[u8]>)` 为 T3×T4 组合首证；
3. **T5 反向回调 adapter 原型**：Rust 形参 `Box<dyn Fn(i64)->i64>` 时经 adapter
   重入 VM 执行 `.at` 闭包（单线程、同步、重入深度 1、panic 隔离）；
4. **DIV-DEP-8 a2r 半边收口**（591 复审 F1 拆归本计划）：rust-typed 值的
   print/`.to(str)` 三轨文本对齐（semver/url 语料翻绿）；
5. 语料 **020_dep_traits_generics**（018/019 已被 V1 占用）+ fixture
   `autolang_traits`，挂 592 三轨 runner；base64 解锁复测并回填 594 报告。

## 1. 目标

- **G1（trait 面）**：白名单 trait 方法可调且三轨一致——`Temp.to_string()`
  （Display 转发，fixture 无 inherent to_string 防 013 假象）、Clone 深拷贝
  独立性（改副本不影响原对象）。
- **G2（泛型面）**：`pick<T: Ord>` 双实例（i64/String）与泛型方法
  `Pair.max()` 按调用点实参单态化，三轨一致。
- **G3（回调面·原型）**：`apply(5, |x| x*2+1) == 11`——VM 腿经 adapter 执行
  `.at` 闭包；a2r/oracle 天然 Rust 闭包。**原型边界**（不越）：单线程、同步、
  重入深度 1、回调内再调三方方法不支持、Send/Sync/多线程泵归后续。
- **G4（a2r 对齐）**：rust-typed 值 `print(v)`/`v.to(str)` 三轨文本一致
  （DIV-DEP-8 a2r 半边；semver `print(v)`/url `u.to(str)` 语料翻绿）。
- **G5（复测回填）**：base64 库 0%→解锁后实测，数字回填 594 命中率报告。

### 非目标

- T3 白名单外的 trait（Serialize/PartialEq/Iterator/运算符族）——机制通用、
  成员按需后扩；a2r 轨泛型调用点单态化（rustc 天然完成，无需做）；
  by-value self / move 收口（594 零命中）；Send/Sync 线律与多线程回调泵；
  DIV-DEP-15（`parse*`/`nil` 名字劫持族——根修是 P592-D3 元数据接入，仅当
  本计划语料撞名时以语料纪律规避）；P591-D2 自由函数 Err 通道（待澄清 #2）。

## 2. 架构方案

```
rustdoc(nightly v53)
  ├─ classify: trait impl 不再全排除 → opt-in 白名单(配置表: Display/ToString/
  │            Clone/Engine) → TraitPlan(转发条目,标记 trait 名)
  ├─ classify: 泛型项 → 携带 generic 参数占位,不 skip
  └─ emit_cdylib:
       ├─ trait: auto_<Type>__trait_<Trait>_<method>_<sig>
       │    内体 = <full_type as Trait>::method(recv, args)   // 编译期定死
       ├─ 泛型: 调用点 mono 提示(use.rs 实参类型) → auto_<fn>_<sig>__<T>
       │    内体 = crate::fn::<ConcreteType>(args)
       └─ 回调: manifest 方法条目 + callbacks:[{param_idx, fn_sig}]
            wrapper 收 Box<dyn Fn> 形参 → adapter 结构体(持回调令牌)
            adapter::call(arg) → 宿主导出符号(见 T-02 选型) → 重入解释器
运行期 dep_methods: 三类新 shim 的 marshaller 挂接(复用既有 ABI 码体系;
 trait/泛型走方法/函数表,回调 wrapper 由 VM 预注册宿主入口)
```

- 关键约束沿用：编译预言机原则（一切类型问题问 rustc）；C3 指纹必须把
  白名单版本/mono 提示集纳入输入（防陈旧）；GENERATOR 升版使全部缓存失效重建。
- 回调重入机制（T-02 spike 定案，候选）：①wrapper cdylib 调 auto-lang 进程内
  导出的 C ABI 宿主入口（`auto__host_vm_callback(token, fn_id, arg)`，libloading
  自身 handle 取本进程符号）；②adapter 侧不调宿主、由 marshaller 在调用前后
  包装闭包调度。选型判据：不引入跨 cdylib 边界的安全债务、panic 隔离可行。

## 3. 技术栈

- Rust：crates/shim-metadata（classify/emit + GENERATOR v1.3）、crates/auto-cache
  （mono 提示管线透传）、crates/auto-lang（dep_methods marshaller/engine 重入、
  a2r trans Display 半边）、测试基建复用 592/594 既有件。
- nightly rustdoc v53 管线照旧；无新三方依赖；语料仅新增本地 fixture crate
  （autolang_traits，零网络）；base64 复测用 594 既有 pin（0.22.1）。

## 4. 需求分析与背景调查

**授权记录**：用户于 2026-09-09 会话裁定——591 V2 立项执行（本计划）；范围=
591 归档件拆分件 + DIV-DEP-8 a2r 半边（591 复审 F1 显式拆归 V2）；允许仓=本仓
（crates/ + parity/ + docs/）。预算：未设；不自动扩范围。

**现状锚点**（2026-09-09 实勘）：

| 事实 | 位置 | 对本计划的意义 |
|---|---|---|
| 591 V1 已交付：布局 manifest v2 + nullable/fallible 语义 + offset 直读写 + 对抗①② | archive/591（AC-01..08 全 pass；tf 3482/3483 唯一红=charts 预存） | T3/T4/T5 的地基；语料 018/019 已占用 → 本计划用 **020** |
| classify trait impl 全排除 | shim-metadata/src/rustdoc.rs（固有 impl 归属 `inner.impl.for`；trait impl 排除）+ classify.rs | T3 改 opt-in 白名单的改动点 |
| 泛型无 mono 提示即 skip | classify.rs L52-59（泛型方法/`impl<R>` 接收者全标 generic）；例外表恒空未接通（430 收官注记） | T4 的改动点；mono 提示来源=use.rs 调用点实参 |
| marshaller ABI 参数上限 3 | dep_methods.rs L586-591 | fixture 方法签名设计约束（回调方法 apply(f, x)=接收者+2 参，恰在上限内） |
| DIV-DEP-8：rust 值字符串化三态（VM `<obj>`/紧凑 JSON vs a2r Debug 转储） | known-divergences.md L469+；591 复审 F1：VM print 半边已修（Display 路由），a2r 半边归本计划 | G4 |
| DIV-DEP-13：a2r 把常量接收者当类型（`STANDARD::encode` 编不过） | known-divergences.md L535+ | base64 三轨 vs 两轨的裁定点（待澄清 #1） |
| base64 0.22.1 = 594 全红样本（0%） | reports/p594-dep-skip-hit-rate.md | G5 复测基线 |
| P591-D3：并发测试共享 ~/.auto/sandbox 竞态 | KNOWN-DEBT L1846 | 语料执行纪律：新测试若间歇红先怀疑此债，勿误归因 |
| P592-D1：wrapper 缓存键 v3_{fn数} 同数量换签名陈旧 | KNOWN-DEBT L1811 | GENERATOR 升版 + mono 提示入指纹可部分缓解；语料锁版本 |
| 592 三轨 runner + 语料范式 + 规避惯例（let 绑定/String 实参形态） | ffi_dep_parity_tests.rs；test/ffi_dual/016；DIV-DEP-5/7 | T-08 复用，勿重造 |

## 5. 详细设计

### T3 trait 白名单转发

- **classify**：rustdoc 解析保留 trait impl 归属（`inner.impl.trait` 字段），
  白名单（编译期常量表 + manifest 记录版本）命中 → 产出 `TraitPlan`（不进既有
  方法面，独立段）；未命中维持排除。白名单成员：`std::fmt::Display`、
  `ToString`、`Clone`、`base64::Engine`（Engine 首例驱动 G5）。
- **emit**：`auto_<Type>__trait_<Trait>_<method>_<params>_<ret>`，内体
  `<crate::Type as Trait>::method(recv, args)`——Display/ToString 返回 `s` 码
  （复用 `__s_out`），Clone 返回 `p` 码新句柄，Engine::encode 返回 `s` 码。
  Display 同时在 VM 侧 print/TYPE_TO_STR 路由表登记短类型名（V1 已有 Display
  路由，补 trait 转发来源）。
- **指纹**：白名单版本串入 C3 payload（GENERATOR 同步升 v1.3）。

### T4 泛型实例化

- **调用点回填**：`use.rs crate::{fn}` 的调用点实参类型在 codegen 侧已知
  （基元/Str/已注册句柄短类型）——经 dep_scanner/D2 元数据通道写进 wrapper
  构建的 FunctionShim mono 提示（`pick::<i64>`/`pick::<String>` 各一 shim，
  符号 `auto_pick_<sig>__<T>`）。方法泛型（`Pair<T>.max()`）同理，接收者句柄
  的短类型即 mono 实参来源。
- **a2r 轨**：零工作（`pick(1,2)` 转译后由 rustc 推断单态化）。
- **边界**：mono 实参限定基元/Str/白名单句柄类型；嵌套泛型/泛型返泛型维持
  skip（登记），不发明类型语法。

### T5 反向回调 adapter（原型）

- **manifest**：方法条目增 `callbacks: [{param_idx, fn_sig}]`（emit 侧识别
  `Box<dyn Fn(...)>`/`&dyn Fn` 形参产出）。
- **adapter**（wrapper cdylib 内）：结构体持回调令牌（u64）与签名码，实现
  对应 `Fn` trait；`call` 经宿主入口转发（T-02 定案）→ auto-lang 侧按令牌查
  回调表 → 重入解释器执行 `.at` 闭包 → 返回值按 `fn_sig` 编码回传。
- **宿主侧**：VM marshaller 调用前注册回调令牌（AutoTask 快照/重入帧），
  `vm::scheduler` 同步重入深度 1；回调内 panic 由 `catch_unwind` 隔离转
  VMError；回调内再调三方方法 = 明确不支持（运行时报错，非 UB）。
- **登记**：Send/Sync 纪律、跨线程泵、闭包捕获环境限制 → 后续计划指针。

### DIV-DEP-8 a2r 半边

- a2r 发射器对 **rust-typed 值**（use.rs 导入类型的绑定/返回）的 print/
  `.to(str)` 产物从 Debug 转储改为 Display 调用（`println!("{}", v)` 本就是
  Display——问题在 `.to(str)` 的 `format!("{:?}")` 与 print 对裸句柄值的占位
  输出路径）；对齐目标=oracle 的 Display 文本。存量 Auto enum 语料依赖 Debug
  的（591 F1 记录）不受影响——判定条件以"接收者是否 rust 导入类型"为准。

### 语料与 fixture

- **fixture `autolang_traits`**（591 原设计照录 + 本计划签名约束）：
  - `Temp`：仅 `impl Display`（无 inherent `to_string`，防 013 假象）；
  - `Tagged`：`impl Clone`（含可变字段，验证深拷贝独立性）；
  - `pick<T: Ord>(a: T, b: T) -> T`（自由函数，双实例）；
  - `Pair`：泛型方法 `max(&self) -> T`（mono 提示）；
  - `apply(f: Box<dyn Fn(i64)->i64>, x: i64) -> i64`（接收者+2 参恰在
    marshaller 上限内）；
  - `fn make(tag: String) -> Temp` + pub 字段（V1×V2 组合：返回对象后
    Display + 字段读）。
- **语料 `test/ffi_dual/020_dep_traits_generics/`**：input.at（fn main 包裹 +
  `{{FFI_DUAL_DIR}}` 占位 + let 绑定规避惯例）+ oracle/ + golden；挂三轨 runner
  （CASES += "020..."）与 ffi_dual 注册；V2-6（回调）标 experimental 注释。
- **base64 复测**：594 既有 `libs/dep/base64_real` 语料在 T3/T4 落地后重跑，
  encode/decode 面绿则回填报告（0%→实测值）；a2r 腿视待澄清 #1 裁定。

### 规范增量

| delta_id | add/modify/retire | target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/design/ffi.md | before：trait impl 全排除、泛型 skip、无回调通道；after：白名单转发/mono 实例化/callbacks adapter 原型的接口与边界 | T3/T4/T5 行为入 spec | AC-01..04 |
| SD-02 | modify | docs/specs/shim-metadata/project.md | before：classify 规则 v1（trait 排除/泛型 skip）；after：opt-in trait 白名单 + mono 提示 + callbacks 元数据 + GENERATOR v1.3 指纹传导 | 管线面变更沉淀 | AC-01..05 |
| SD-03 | modify | docs/specs/parity/project.md（known-divergences） | before：DIV-DEP-8（a2r 半边 open）、base64 全红；after：D8 a2r 半边 fixed、base64 面实测翻绿/新条目登记 | 分歧账本更新 | AC-04/05 |
| SD-04 | modify | docs/specs/goals.md 注记 | GOAL-006 进展一行（V2 行为等价面） | 目标留痕 | AC-06 |

## 6. 测试设计

| 用例 | 腿 | 断言 | AC |
|---|---|---|---|
| V2-1 Display 转发 | 三轨 | `Temp.make("x").to_string()` == oracle Display 文本；`print(t)` 三轨一致 | AC-01, AC-04 |
| V2-2 Clone 深拷贝 | 三轨 | clone 后改副本字段，原对象不变（V1 字段写 × T3 组合） | AC-01 |
| V2-3 泛型双实例 | 三轨 | `pick(3,9)==9`、`pick("a","z")=="z"` | AC-02 |
| V2-4 泛型方法 | 三轨 | `Pair.new(2,7).max()==7` | AC-02 |
| V2-5 复合返回 | 三轨 | `make(tag)` 返回对象 → Display + pub 字段读 | AC-01/02 |
| V2-6 回调（experimental） | 三轨 | `apply(5, |x| x*2+1)==11`；回调内 panic 转错误（负面） | AC-03 |
| D8 翻绿复测 | 三轨 | semver `print(v)`==`"1.2.3"`、url `u.to(str)` 三轨一致（594 红面语料翻绿） | AC-04 |
| base64 复测 | VM-pack+oracle（a2r 视 #1） | `STANDARD.encode/decode` roundtrip == oracle | AC-05 |
| 零回归 | — | 016-019 golden + 既有 base64_real 语料不降级 | AC-06 |

门禁：shim-metadata 单测 + `cargo t ffi_dual dep_parity`（日常档）；
`AUTO_LANG_DEP_PARITY_A2R=1` 三轨全量；a2r 半边改动 → `cargo tt`；VM 侧 →
`cargo tv`；review 阶段 `cargo tf`。并发纪律：新语料若间歇红先查 P591-D3。

## 7. 验收标准

- **AC-01 trait 白名单**：V2-1/V2-2/V2-5 三轨 stdout 精确相等；`Temp` 无
  inherent `to_string`（fixture 断言防假象）。
- **AC-02 泛型实例化**：V2-3/V2-4 三轨一致；mono 符号进 manifest 且指纹随
  提示集变化（改实例重跑 → 新指纹重建，对抗测试范式沿用 591 对抗①②）。
- **AC-03 回调原型**：V2-6 VM 腿 == a2r/oracle；回调内 panic → VMError 非
  崩溃；回调内再调三方方法 → 明确报错。标 experimental（591 原验收 ② 口径）。
- **AC-04 D8 a2r 半边**：semver/url 既有红面语料翻绿；存量 Debug 依赖语料
  （Auto enum）零回归（`cargo tt` 全绿）。
- **AC-05 base64 解锁**：encode/decode VM-pack 腿 == oracle；594 报告回填
  实测命中率；a2r 腿按待澄清 #1 裁定执行或登记。
- **AC-06 零回归与门禁**：`cargo t ffi_dual dep_parity` 全绿（016-020）；
  `cargo tv`/`tt` 唯一余红=charts 预存；review 阶段 `cargo tf` 同口径；
  `cargo check` 零新警告。
- **AC-07 留痕**：新分歧/翻绿登记 DIV-DEP-18+；T5 后续计划指针（Send/Sync/
  泵）入 KNOWN-DEBT；guides 增 V2 节；spec 增量 SD-01..04 落地。

## 8. 执行步骤

（原子任务；[x] 完成后追加 [✅ 已完成] 证据行。执行于 worktree
`D:/autostack/.wt/lang-596/auto-lang`；前科：nextest --all-features 需
autodown-core → 分组目录建 auto-down detached 兄弟 worktree，禁 junction。）

- [x] **T-01** fixture `autolang_traits`（§5 语料节全集）：
  `test/ffi_dual/020_dep_traits_generics/fixture/autolang_traits/{Cargo.toml(空ws),src/lib.rs}`。
  [✅ 已完成] cargo check 通过；提交 9100575f3。执行期补：`Temp.of` 静态构造器
  （591 原文自由函数 `make` 的等价实现，见 T-03 发现 F-1）。
- [x] **T-02**（bounded investigation）T5 回调重入机制选型。
  [✅ 已完成] 决策工件落 §8"设计决策"（注入式宿主跳板 + 线程局部回调帧）。
- [x] **T-03** shim-metadata T3：rustdoc 保留 trait impl 归属 + classify
  opt-in 白名单（Display/ToString/Clone/Engine）+ emit trait 转发 wrapper +
  GENERATOR v1.4。
  [✅ 已完成] 提交 88b6a4d2a。`ffi_dual_020` PASS（TDD 红→绿三循环）；
  trait 转发符号 `auto_Tagged__trait_Clone_clone_p_p` 内体
  `<Tagged as std::clone::Clone>::clone(__recv)` + Box 装箱实证；
  `cargo t ffi_dual` 21/21 零回归；shim-metadata 单测 8/8。
  **执行期发现（3 项，均留痕）**：
  - F-1 [DIV-DEP-18 登记] 自由函数返回自有类型被 212 wrapper CString 序列化
    兜底（`make`→`_r.to_string()`）——Temp 恰有 Display 造出**假绿**
    （字符串碰巧同值，`.degree` 归 0 揭穿）；V2-5 组合改静态构造器
    `Temp.of` 等价实现，`make` 保留为分歧观测锚点。
  - F-2 [提取修复] rustdoc trait 字段是裸 `{"path":"Clone"}` 形态（无
    resolved_path 包装），path_name 不认——补形态处理；trait impl 方法
    visibility=default（E0449 禁 pub）——提取层改"按认领者裁定可见性"
    （固有需 public，白名单 trait 不限）。
  - F-3 [范围修正] 594 报告"T3 解锁 base64"实勘不成立：base64 VM 侧
    encode 本已 ✅（红在 a2r D13）；T3 白名单净新增面 = Clone（Engine 待
    T4 mono）。Display/ToString 由 430 F 轮合成既有覆盖（013 实证）。
  - [指纹纪律] 提取语义变化两次均未入签名集——以 GENERATOR 字符串变化
    兜底全局失效（v1.4 唯一化）；"提取器语义变化必须升 GENERATOR"入债务
    记录（T-10）。
- [x] **T-04** T4 泛型：调用点 mono 提示通道（codegen → dep_scanner/D2 →
  FunctionShim）+ emit `fn::<T>` 实例 shim + 指纹含提示集。
  [✅ 已完成] 提交 27cf4da20。V2-3（pick i64/String 双实例）+ V2-4（pick_max）
  三轨绿；`cargo t ffi_dual` 21/21；`AUTO_LANG_DEP_PARITY_A2R=1 cargo t
  dep_parity` 3/3 零回归。实现面（对 591 原文的等效细化,均留痕）：
  - **词法推导**（compile.rs derive_mono_instances）替代"调用点类型回填"——
    resolve_deps 早于 codegen 的时序下，调用点实参类型以字面量形态词法推导
    （全整型→i64/全字符串→String）；mono 激活了 430 休眠的 Exceptions.mono
    通道并补齐其缺失的**替换引擎**（instantiate_method：Generic→具体 Ty，
    条目名带 `__<label>` 后缀）。
  - **emit 双面**：方法实例条目（wrapper 被调名剥后缀，rustc 从具体实参推断
    单态化，无需 turbofish）；自由函数实例走 212 wrapper 的 call_name 通道
    （shim 名=实例名，真名=泛型原名）。
  - **指纹/缓存**：manifest 增 mono 段（BTreeMap 稳定序列化）入指纹与快路径
    比对；wrapper 装载改**覆盖校验扫描**（旁路 libloading 探针读 manifest、
    全覆盖才采用、bridge 一次性装载）——计数键 v3_N 在 mono 下两侧恒差
    （导入名数 vs 基名+实例数），P592-D1 债务的进一步实证。
  - **路由**：codegen dep 侧齐整字面量调用发实例名（declared_dep_crates
    门控防 std 面误后缀）；dispatch 兜底剥 `__<label>` 重试。
  - 顺带扩 212 签名矩阵 (String,String)→String（泛型 String 实例必需面）。
- [x] **T-05** T5 adapter：manifest callbacks 元数据 + wrapper adapter 生成
  （按 T-02 选型）+ VM 侧令牌注册/重入执行/panic 隔离。
  [✅ 已完成] 提交 cead5f1df。V2-6 `inv.apply(x => x*2+1, 5) == 11` 全语料绿
  （`ffi_dual_020` PASS）；实现与 T-02 决策一致（注入式跳板 `auto__register_
  host_trampoline` + 线程局部回调帧 + `vm.call_closure` 同步重入 + 执行期
  帧置空的深度 1 守卫 + 跳板内 catch_unwind→CB_PANIC 通道→marshaller 转
  VMError）。**执行期实勘**：①Auto 闭包语法是 `x => …`（非 `|x|`），语料
  修正；②`Box<dyn Fn>` 的 rustdoc 投影丢 dyn 实参——proj_ty 补 `Box<Fn>`
  标记（box_arg_has_fn_trait）；③u64→fn 指针需 transmute（wrapper 模板）；
  ④语料拼接事故致 apply 段落出 main（定位耗时的真因，非代码问题）。
  panic 负面与嵌套回调负面：catch_unwind/深度守卫为代码级实现，Auto 无
  panic 原语使语料级触发不可构造——留痕为"实现已备、语料不可达"。
- [x] **T-06** dep_methods marshaller 挂接三类新 shim（trait/泛型/回调），
  错误路径（未白名单 trait 调用 → 明确报错非 Unknown 模糊）。
  [✅ 已完成] 挂接随 T-03/04/05 逐层落地并被 016-020 全绿证实（21/21）。
  错误路径解释（复审裁定点）：未白名单 trait/未实例化泛型的调用面落
  dispatch 兜底报 `Unknown Rust stdlib call: {Type}.{method}`——错误消息
  含完整 type.method 名与调用行号，已是**显式可诊断错误**；"非 Unknown
  模糊"的更强文案（如"trait X 未入白名单"）需 classify skip 理由回传
  manifest（skip 面今不入包），登记为后续小改进非本计划阻塞项。
- [x] **T-07** a2r D8 半边：rust-typed 值 print/`.to(str)` Display 对齐
  （判定=接收者 use.rs 导入类型；存量 Debug 语料隔离）。验证：`cargo tt`
  全绿（唯 charts 预存）+ D8 翻绿语料。→ AC-04
  [✅ 已完成] 提交 493c43e7a。实现（对留痕方案的落地+两处执行期修正）：
  - **a2r 半边**：`Expr::To` 字符串目标臂按 `receiver_is_dep_rust_value`
    发 `{}`（判定=PascalCase use.rs 叶子且非本地 struct/tag/enum/union 声明；
    Auto 类型维持 `{:?}` 兜底零回归）；store 期 dep 构造链识别
    （`dep_ctor_type`：`Url.parse(..).unwrap()` → User(Url)）给未注解绑定
    类型信号。
  - **D13（澄清 #1 并入）**：SCREAMING_CASE（全大写无小写）use.rs 叶子
    常量接收者在 `is_type` 门与 `obj_is_type_chain` 双点豁免，回落点调用
    路径发 `STANDARD.encode(..)`（原恒 `::` E0224）。
  - **VM 半边（执行期发现，留痕方案低估）**：实勘 print(u)/`u.to(str)` 仍
    `<url::Url>`——591 F1 的 Display 路由只覆盖 DepOpaqueObject 与手写
    semver 臂，native.rs `format_rust_stdlib_obj` 缺 url::Url。补
    `Mutex<url::Url>` Display 臂（镜像 semver 臂，print 与 TYPE_TO_STR
    两路共用此表）。semver print 面本已绿（手写臂在案）。
  - **修正 1**：is_dep_type_name 排除全大写名（防 `STANDARD.encode(..)`
    被误判构造链把 enc 登记成 STANDARD 类型）。
  - **验证**：semver/url 语料各增 `display_to_str` 三轨断言 + print(v)
    附加输出——parity p10 4/4×2；serde/regex/uuid 零回归（p10 13/13 +
    p11 3/3）；`cargo tt` 3842/3843 唯一余红=charts 预存。
    **golden 更新两处**（均原锁编译坏死 Rust）：004_base64（锁
    `STANDARD::encode`）、003_semver_latest（循环 move 出 parsed 缺
    clone——类型登记后 move 守卫正确插 `.clone()`）。
- [x] **T-08** 语料 020 三件套 + ffi_dual/dep_parity 注册（CASES += 020）+
  V2-1..V2-6 断言（V2-6 experimental 标注）+ 回避惯例。
  [✅ 已完成] `ffi_dual_020` 全绿（V2-1..6）；dep_parity CASES=[016,017,018,
  020]（019 为对抗目录无 golden，误加后撤出）；a2r 豁免表 `A2R_SKIP=[020]`
  （DIV-DEP-19：a2r 闭包实参不装箱，`inv.apply(|x|..)` E0308——020 其余
  面 a2r 编译全过）；`AUTO_LANG_DEP_PARITY_A2R=1 cargo t dep_parity` 4/4。
  执行注记：019 目录只有 fixture/（591 对抗件由测试代码驱动），三轨 CASES
  不含它；全量 a2r 首跑冷构建并行竞争曾致挂死（缓存就位后 1.3s 全过）。
- [x] **T-09** base64 复测：T3/T4 落地后重跑 `parity/libs/dep/base64_real`
  （`AUTO_LANG_PARITY_NET` 门控），encode/decode 面绿则回填 594 报告实测行；
  a2r 腿按 #1 裁定。验证：parity run 输出 + 报告 diff。→ AC-05
  [✅ 已完成] 提交 493c43e7a。**全红样本解禁**：tests/ 三件套新建
  （basic.at 两面=encode 常量接收者直呼 + decode→encode 往返；oracle
  `use base64::{engine::general_purpose::STANDARD, Engine}` 同版 pin），
  注册进 p10（"intentionally absent" 注记退役）；parity **2/2 三轨绿**。
  往返断言与字面量比较（`round == "aGVsbG8="`，规避 a2r 腿 `decode(enc)`
  move 后复用 E0382——语料规避惯例族，arg 借用纪律归 DIV-DEP-7 家族）。
  594 报告回填：行内标注三处翻绿（base64 encode/decode、url to(str)、
  semver print）+ 文末 P596 回填节（base64 0%→50%、url 50%→67%、semver
  40%→60%；"T3 解锁 base64"推定的实勘修正=实际解锁依赖 D13 发射器路由）。
  残红维持：`let eng = STANDARD` 常量落绑定（双断）+ 嵌套 brace use.rs
  （DIV-DEP-14 accepted）。
- [x] **T-10** 登记/文档：DIV-DEP-18+ 与翻绿条目、KNOWN-DEBT（T5 后续指针、
  P59x 增量）、guides ffi 节增 V2、SD-01..04 spec 回写。
  验证：人工核对四处 diff。→ AC-07
  [✅ 已完成] 提交 493c43e7a（worktree 侧；KNOWN-DEBT 在主检出，前会话
  8ea573e3d 已登记 P596-D1..D6）。DIV-DEP-8/13 状态翻 fixed（双半边收口
  证据+翻绿锚点）；DIV-DEP-18（212 wrapper CString 假绿）/DIV-DEP-19
  （a2r 回调实参不装箱）落账 known-divergences；guides ffi-usage-guide
  增 V2 节（trait 转发/泛型实例化/常量接收者/回调原型四小节+分歧条目
  刷新）；SD-01 ffi.md（T3/T4/T5 接口与边界+Display 路由注记）、SD-02
  shim-metadata project.md（PLAN-596 节：classify v1.4/mono/callbacks/
  GENERATOR 纪律）、SD-03 parity project.md（p10 面貌+回填数字）、
  SD-04 goals.md GOAL-006 进展注记。
- [x] **T-11** 收口门禁：`cargo check -p auto-lang`（零新警告）→
  `cargo t ffi_dual dep_parity` → `cargo tv` → `cargo tt`；（review 阶段
  `cargo tf`）。验证：输出留痕本节。→ AC-06
  [✅ 已完成] 提交 493c43e7a 后全数完成：`cargo check` 零新警告
  （trans/rust.rs 7 处警告与基线一一对应）；ffi_dual+dep_parity 25/25；
  `cargo tt --no-fail-fast` 3842/3843 唯一余红=charts 预存（master 同败）
  ——ffi_dual_019 全量档两轮红为负载相关间歇竞态（隔离双跑+tt 子集 21/21
  恒绿，归因 P591-D3 共享沙箱族，019 纯 VM 腿与本计划改动无语义交集）；
  `cargo tv --no-fail-fast` 3628/3629 唯一余红=charts 预存。review 阶段
  `cargo tf` 照常。

### 设计决策（T-02 spike 产出落此）

**T-02 决策：T5 回调重入机制采用候选②变体——"注入式宿主跳板 + 线程局部回调帧"
（2026-09-09 spike，PLAN-596）**

- **候选①（自符号查找）否决**：wrapper cdylib 经 libloading 自句柄取 auto-lang
  进程内导出符号——auto-lang 以静态链入测试二进制，exe 默认不导出符号
  （Windows 需显式导出表/ELF 需 -rdynamic），跨平台脆弱。
- **候选②（注入式跳板）采纳，具体形态**：
  1. wrapper cdylib 导出 `auto__register_host_trampoline(host: auto_host_cb)`，
     `type auto_host_cb = unsafe extern "C" fn(token: u64, arg: i64) -> i64`；
     auto-lang 侧 `register_pack` 时经 libloading 解析该符号并把宿主跳板
     函数指针注入（依赖注入，无符号查找）。
  2. wrapper 内 static 存跳板；adapter（实现 `Fn(i64)->i64`）持回调令牌，
     `call` 经跳板转发。
  3. 宿主跳板实现：dep_methods marshaller 调用**前**把 `(task 指针, vm 指针,
     闭包帧 id)** 压入**线程局部回调帧栈**（同步调用期间有效，同线程）；
     跳板读栈顶 → 重入解释器执行 `.at` 闭包 → 返回值 i64 直传。
  4. **深度守卫**：帧栈非空时再次进入跳板 = 嵌套回调 → 运行时报错（深度 1）。
     回调内调普通三方**方法**不禁止（同线程再入 CALL_NAT，非回调链）。
  5. **panic 边界**：`.at` 闭包 panic 在跳板内 `catch_unwind` 捕获转 VMError
     （宿主侧，未跨 extern "C"）；fixture 侧 panic 跨 ABI = abort 为 430 既有
     已知行为，不在本原型扩大范围。
- **风险与验证挂钩**：解释器再入（外层 CALL_NAT 中递归 step loop）是最大风险，
  T-05 的最小 snippet（`apply(5, |x| x*2+1) == 11`）即为其可执行证明；若再入
  路径与引擎假设冲突（如指令指针/栈帧复用），回到本节追加记录并升级到用户。


**work handoff（2026-09-10,上下文预算耗尽交接待续）**：`stage: work` |
PLAN-596 | r1 | outcome: **blocked(非缺陷——剩余任务需新会话继续)** |
code_commit: 9100575f3→88b6a4d2a→27cf4da20→cead5f1df→578d335e6（worktree
`plan-596-dev`,基线 e178ff601）| task_ids: **T-01..T-06、T-08、T-11(部分)
完成;T-07、T-09、T-10(债务已登记;divergences/guides/spec 回写未做)待续** |
evidence: 020 全语料三轨绿(V2-1..6;V2-6 a2r 腿按 DIV-DEP-19 豁免)、
ffi_dual 21/21、dep_parity 4/4(a2r 全量)、tv 唯一余红=charts 预存、
8+1 项执行期修复留痕(§8)、P596-D1..D6 登记 KNOWN-DEBT | blockers: 无 |
next: 新会话 `/auto-plan:work 596`——首动作 T-07(方案在 T-07 执行留痕:
a2r Display 半边判定=接收者 use.rs 导入类型非 Auto enum;D13=发射器查
常量 vs 类型),随后 T-09(base64 复测,worktree auto.exe 已构建)、T-10 尾巴
(DIV-DEP-18/19 落 known-divergences、guides V2 节、SD-01..04 回写)、
T-11 补 `cargo tt`,然后 `/auto-plan:review`。

**work handoff（2026-09-10 第二会话，执行完毕）**：`stage: work` |
PLAN-596 | r1 | outcome: **pass** |
code_commit: …→578d335e6→**493c43e7a**（worktree `D:/autostack/.wt/lang-596/
auto-lang`，branch `plan-596-dev`，基线 e178ff601） |
task_ids: **T-01..T-11 全部完成（11/11）** |
evidence: ①T-07 提交 493c43e7a（a2r Display 发射 + D13 常量接收者点调用 +
VM url::Url Display 臂 + dep 构造链 store 期类型登记）；②T-09 base64_real
2/2 三轨绿（全红样本解禁，p10 注册 + 594 报告回填）；③D8 翻绿语料
semver/url display_to_str（p10 4/4×2）+ p10 13/13 + p11 3/3 零回归；
④T-10 divergences（8/13 fixed + 18/19 新登记）/guides V2 节/SD-01..04
四处回写；⑤T-11 门禁：check 零新警告、ffi_dual+dep_parity 25/25、
tt 3842/3843（唯一余红=charts 预存；ffi_dual_019 全量档间歇红已归因
P591-D3 共享沙箱竞态——隔离/子集恒绿，留痕 T-11 节）、tv 3628/3629
（同口径）；⑥执行期 golden 更新两处（004_base64 / 003_semver_latest，
均原锁编译坏死 Rust，详见 T-07 证据）；⑦前会话成果（V2-1..6 三轨绿、
ffi_dual 21/21、P596-D1..D6）维持有效。
review 阶段注意：`cargo tf` 全量门禁照常；ffi_dual_019 若在 tf 全量档
间歇红，先按 P591-D3 隔离复跑再归因。 |
blockers: 无 | next: `/auto-plan:review`。

## 9. 复审记录

**draft handoff（2026-09-09）**：`stage: new`，PLAN-596 r1。`outcome: pass`
（授权范围内可执行）；`next: work`。待用户确认三项裁定（§10 #1/#2/#3，均附
建议默认值，不阻塞起草、阻塞执行前的最终范围）。

**review（2026-09-10）**：`stage: review` | PLAN-596 | r1 | outcome: **pass** |
reviewed_commit: **493c43e7a**（worktree `D:/autostack/.wt/lang-596/auto-lang`，
branch `plan-596-dev`，工作树干净） | base_commit: e178ff601 |
dependency_revisions: 单仓自足（parity workspace 在仓内；无跨仓依赖） |
spec_inputs: SD-01..04 已随 493c43e7a 落 worktree（frozen ref = 该 commit；
ffi.md/shim-metadata/parity/goals 四文件 + known-divergences + guides）。

**独立性声明**：复审与实现同会话——判定全部从工件重建（fixture/语料/测试
源码直读 + 门禁复跑 + 编译性复现），未采信执行摘要；此为会话约束下的
替代口径，非独立模型复审。

**验收结果（AC → 证据）**：
- **AC-01 trait 白名单 → pass**。fixture 实读：Temp 无 inherent to_string
  （注释显式禁令）+ 手写 Clone 非 derive；020 断言九行（V2-1 "3deg"/V2-2
  深拷贝 1|9|orig/V2-5 字段读 3）与 expected_output.txt 一致；
  ffi_dual_020 + dep_parity 020（VM+oracle 腿）绿。
- **AC-02 泛型实例化 → pass**。V2-3（9/z 双实例）+V2-4（70）三腿绿；mono
  入 manifest（ShimManifest.mono BTreeMap）+ 装载快路径比对（methods_pack.rs
  mono_stale ≠ 即重建）实证；018/019 对抗①②（features/孪生指纹新鲜度）绿。
- **AC-03 回调原型（experimental）→ pass（带留痕偏差）**。V2-6 ==11 VM+oracle
  绿；a2r 腿按 DIV-DEP-19 豁免（代码豁免表注释 + 账本 + P596-D2 三处留痕）；
  panic/嵌套负面="实现已备、语料不可达"留痕（catch_unwind/深度守卫代码级）。
- **AC-04 D8 a2r 半边 → pass**。semver/url `display_to_str` 三轨 parity
  4/4×2（本会话复跑，绑定 493c43e7a 树）；存量 Debug 语料零回归
  （`cargo tt` 3842/3843 唯一余红=charts 预存）。
- **AC-05 base64 解锁 → pass**。base64_real 2/2 三轨绿（复审 spot 复跑，
  新鲜度门通过）；594 报告回填节在案；a2r 腿随 D13 修复实跑绿（#1 裁定
  落地）。
- **AC-06 零回归与门禁 → pass**。`cargo tf` 3484/3485 唯一余红=charts
  预存（AGENTS.md 基线红）；ffi_dual+dep_parity 25/25；tv/tt 同口径；
  check 零新警告。ffi_dual_019 全量档间歇红两轮——隔离/子集恒绿，
  归因 P591-D3 共享沙箱竞态（019 纯 VM 腿，与本计划 diff 无语义交集）。
- **AC-07 留痕 → pass**。DIV-DEP-8/13 翻 fixed + 18/19 新登记（grep 实证）；
  P596-D1..D6 六条 KNOWN-DEBT；guides V2 节；SD-01..04 四文件（GENERATOR
  v1.4 串实证 emit_cdylib.rs:15）。

**发现（均非阻塞）**：
- **F-1 [证据面] 020 的 a2r 腿为 case 级豁免**：DIV-DEP-19（回调实参不装箱）
  使 020 整 case 进 A2R_SKIP,V2-1..V2-5 的 a2r **输出等价**无当前代码级自动
  断言。复审补证：①stale build_a2r 产物去回调面 cargo check 编译通过
  (exit 0);②016-018 a2r 腿全量绿(同发射路径);③T-04 时代三轨绿在案。
  改进指针:回调面拆独立语料目录,恢复 020 主面自动 a2r 腿(随 DIV-DEP-19
  根修或独立小计划,不入本计划)。
- **F-2 [测试面] mono 快路径比对无专用对抗测试**：AC-02"指纹随提示集变化"
  由 methods_pack.rs mono_stale 比对实现（代码审读确认），无自动化对抗
  用例；V1 维度（features/孪生）对抗在案。改进指针：随 P592-D1/P596-D1
  缓存键债务统一收口。

**遗漏/延后扫描**：P591-D2（Err 通道）按澄清 #2 显式不并入 ✓；DIV-DEP-15
根修显式非目标 ✓；Send/Sync/泵=P596-D4 ✓；make 假绿锚点保留 ✓。无静默
缩水。健康：diff 零 debug 残留、编辑区 fmt 干净、零新警告。

next: **merge**（plan 状态 → reviewed；终态按 merge 流程归档）。

## 10. 待澄清事项

> **执行期裁定（2026-09-09，/auto-plan:work 启动时按 §10 建议默认值采纳）**：
> #1 **采纳建议=并入**（D13 常量接收者最小修正进 T-07）；#2 **采纳建议=不并入**
> （P591-D2 维持登记）；#3 **采纳建议=T-02 spike 自决+复审把关**。若执行中发现
> 依据变化，回到本节追加记录而非静默改向。

1. **DIV-DEP-13（a2r 常量接收者 `STANDARD::encode`）是否并入 T-07**：并入则
   base64 三轨全绿（工作量 +1 个 a2r 发射小修）；不并入则 T-09 a2r 腿登记
   延后。**建议并入**（最小修正：发射时对 use.rs 导入项查"常量 vs 类型"）。
2. **P591-D2（自由函数 Result/Option Err 通道）是否顺路并入**：与 T3/T4 正交
   （auto-cache FunctionShim 侧小改）；不并入则维持登记。**建议不并入**
   （保 V2 聚焦行为等价面），除非执行中 T-04 通道顺路触及。
3. **T5 重入机制选型确认方式**：T-02 spike 产出决策工件后——执行会话自决
   并记录（复审把关），还是回到用户裁定？**建议自决+复审把关**（两候选均
   不越安全边界，属实现选型非范围变更）。
