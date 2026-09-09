---
plan_id: PLAN-596
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: dep-rust-v2-trait-generic-callback
author: [ZCode]
created_at: 2026-09-09
updated_at: 2026-09-09
plan_revision: 1
current_step: 0
total_steps: 11

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

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

- [ ] **T-01** fixture `autolang_traits`（§5 语料节全集）：
  `test/ffi_dual/020_dep_traits_generics/fixture/autolang_traits/{Cargo.toml(空ws),src/lib.rs}`。
  验证：`cargo check` 通过；方法签名均满足 marshaller ABI≤3（含接收者）。
  → AC-01..03
- [ ] **T-02**（bounded investigation）T5 回调重入机制选型：spike 两个候选
  （§2 架构①②）各出最小可编译验证（宿主符号可达性 / panic 隔离可行性），
  决策记录落本节"设计决策"小节。产出=决策工件，不实现产品代码。
  → AC-03
- [ ] **T-03** shim-metadata T3：rustdoc 保留 trait impl 归属 + classify
  opt-in 白名单（Display/ToString/Clone/Engine）+ emit trait 转发 wrapper +
  GENERATOR v1.3。验证：shim-metadata 单测 + 对 fixture 生成的 wrapper 手工
  `cargo check`（沙箱 builds 目录）。→ AC-01
- [ ] **T-04** T4 泛型：调用点 mono 提示通道（codegen → dep_scanner/D2 →
  FunctionShim）+ emit `fn::<T>` 实例 shim + 指纹含提示集。验证：`pick` 双
  实例符号出现在 manifest；同 fixture 改实例重跑指纹变化。→ AC-02
- [ ] **T-05** T5 adapter：manifest callbacks 元数据 + wrapper adapter 生成
  （按 T-02 选型）+ VM 侧令牌注册/重入执行/panic 隔离。验证：V2-6 最小
  snippet `run_with_capture` == 11。→ AC-03
- [ ] **T-06** dep_methods marshaller 挂接三类新 shim（trait/泛型/回调），
  错误路径（未白名单 trait 调用 → 明确报错非 Unknown 模糊）。
  验证：`cargo t ffi_dual`（016/017 零回归）+ 新臂单测。→ AC-01..03
- [ ] **T-07** a2r D8 半边：rust-typed 值 print/`.to(str)` Display 对齐
  （判定=接收者 use.rs 导入类型；存量 Debug 语料隔离）。验证：`cargo tt`
  全绿（唯 charts 预存）+ D8 翻绿语料。→ AC-04
- [ ] **T-08** 语料 020 三件套 + ffi_dual/dep_parity 注册（CASES += 020）+
  V2-1..V2-6 断言（V2-6 experimental 标注）+ 回避惯例（let 绑定/String 形态）。
  验证：`cargo t ffi_dual_020` + `AUTO_LANG_DEP_PARITY_A2R=1 cargo t dep_parity`。
  → AC-01..04
- [ ] **T-09** base64 复测：T3/T4 落地后重跑 `parity/libs/dep/base64_real`
  （`AUTO_LANG_PARITY_NET` 门控），encode/decode 面绿则回填 594 报告实测行；
  a2r 腿按 #1 裁定。验证：parity run 输出 + 报告 diff。→ AC-05
- [ ] **T-10** 登记/文档：DIV-DEP-18+ 与翻绿条目、KNOWN-DEBT（T5 后续指针、
  P59x 增量）、guides ffi 节增 V2、SD-01..04 spec 回写。
  验证：人工核对四处 diff。→ AC-07
- [ ] **T-11** 收口门禁：`cargo check -p auto-lang`（零新警告）→
  `cargo t ffi_dual dep_parity` → `cargo tv` → `cargo tt`；（review 阶段
  `cargo tf`）。验证：输出留痕本节。→ AC-06

### 设计决策（T-02 spike 产出落此）

## 9. 复审记录

**draft handoff（2026-09-09）**：`stage: new`，PLAN-596 r1。`outcome: pass`
（授权范围内可执行）；`next: work`。待用户确认三项裁定（§10 #1/#2/#3，均附
建议默认值，不阻塞起草、阻塞执行前的最终范围）。

## 10. 待澄清事项

1. **DIV-DEP-13（a2r 常量接收者 `STANDARD::encode`）是否并入 T-07**：并入则
   base64 三轨全绿（工作量 +1 个 a2r 发射小修）；不并入则 T-09 a2r 腿登记
   延后。**建议并入**（最小修正：发射时对 use.rs 导入项查"常量 vs 类型"）。
2. **P591-D2（自由函数 Result/Option Err 通道）是否顺路并入**：与 T3/T4 正交
   （auto-cache FunctionShim 侧小改）；不并入则维持登记。**建议不并入**
   （保 V2 聚焦行为等价面），除非执行中 T-04 通道顺路触及。
3. **T5 重入机制选型确认方式**：T-02 spike 产出决策工件后——执行会话自决
   并记录（复审把关），还是回到用户裁定？**建议自决+复审把关**（两候选均
   不越安全边界，属实现选型非范围变更）。
