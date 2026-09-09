---
plan_id: PLAN-591
status: executing              # drafting → executing → execution_done → reviewed → archived（r1 修订经用户 /auto-plan:work 确认，2026-09-09；D2 裁定按提案并入案生效）
feature_name: use-rust-any-crate-direct
author: [zhaopuming, ZCode]
created_at: 2026-09-07
updated_at: 2026-09-09（r1 修订：V1 先行拆分——T2 语义层先行 + T1 布局层；V2（T3/T4/T5）拆后续独立计划；对抗测试①②与非白名单勘测入验收；测试编号 018/019）
plan_revision: 1
current_step: 0
total_steps: 11

# /auto-plan:review 结束时填写（以下为 r1 起草期暂填，review 定稿）：
supersedes_spec_components: []
new_spec_components:
  - "shim-metadata: 拟修改 —— classify 增 Option nullable 返回规则（'?' 前缀）+ Result 错误通道传导收口 + emit_cdylib 探针 crate 与 manifest v2 layouts 段（format: 2）+ 指纹输入扩布局行"
  - "auto-cache: 拟修改 —— C3 指纹输入增 features 组合（布局行经 emit_cdylib 传导）；自由函数 wrapper 版本键传导 manifest format（P592-D1 链）"
  - "auto-lang/vm: 拟修改 —— DepOpaqueObject 布局挂载 + CALL_SPEC offset 直读直写 + Option nullable/Result 错误通道解码 + manifest format: 2 装载校验拒载旧版"
touched_goals:
  - "GOAL-006: Consumer-mode parity——本计划是 GOAL-006 的首个实体化主线（V1）：dep 显式声明 + use.rs 消费三方 crate 的字段透明（布局 manifest）与 Option/Result 语义收口，在 592/594 三轨对拍网上翻绿"
  - "GOAL-003: Auto 作 Rust 脚本层——use.rs 直用三方库是'脚本层体验'的消费面入口，V1（字段+语义层）先行；V2 行为等价归后续计划"

affects: [shim-metadata, auto-cache, auto-lang/vm]  # V1 实际触面；DIV-DEP-8 若裁定并入则追加 auto-lang/trans
---

# [PLAN-591] use-rust-any-crate-direct

## 0. 变更摘要

愿景：**`dep a(version: "1.2")` 声明之后，`use.rs a::b::c::D`，D 直接可用**——不管 D
是自由函数、结构体还是泛型方法，不需要手写 shim、不需要 JSON 桥、不需要了解
D 的内部形态。

> **r1 修订（2026-09-09，plan_revision: 1）**：本计划由双层愿景修订为 **V1 先行
> 可执行计划**。修订依据：前序 PLAN-592（P0 特征化网，已归档）与 PLAN-594
> （R 层真三方库，已归档）交付了三轨对拍基建与五库 25 面命中率数据
> （32%，docs/plans/reports/p594-dep-skip-hit-rate.md），裁定 T2（Option/Result）
> 最高杠杆。要点：
>
> 1. **拆分**：本计划只执行 **V1 = T2 语义层（先行）+ T1 布局层**；V2（T3 trait
>    单态化 / T4 泛型 / T5 反向 adapter）**拆为后续独立计划**（编号未取），设计
>    文字全部保留（§2 分层设计 T3-T5 + §6 V2 用例表），不得静默丢弃。
> 2. **V1 内部排序改为 T2 先行**（594 命中率裁定：Option/Result ~5 面直接解锁，
>    且 DIV-DEP-8 格式化/字符串化是所有 rust-typed 值断言表达的隐性天花板）。
> 3. **对抗测试①② 入验收**（592 设计，原稿未列）：孪生 fixture 布局指纹防
>    stale + features 入 C3 指纹。
> 4. **非白名单库勘测并入**（594 方法论注记）：uuid（Cargo.lock 既有 1.24.0，
>    零网络、不在 BUILTIN_OPAQUE_CRATES）pack 面用例；执行中若判定体量超界，
>    允许显式登记为后续勘测计划，不可无痕缩水。
> 5. 测试编号改用 **018/019**（待澄清 #3 已消化）；014/015/016/017 均已占用。

现状（430 管线）已把后端机器造好：dep 声明 → nightly rustdoc 提签名 → 分类/生成
shim 包 → 编译 cdylib → C3 指纹全局缓存 → 运行时装载注册 dispatch（`ffi_dual_013_dep_method`
实测可用；592 又修复 8 个管线 bug 并钉死 marshalling golden）。剩余缺口集中在
**覆盖端**：类型字母表停在 `v/i/l/f/b/s/p + 不透明句柄`——字段不可读（592 已桥接
合成 getter 面 `p.x`，但真布局直读直写仍缺）、trait 方法不可调、泛型需手工 mono
提示、Option/Result 语义未收口（`types.rs`/`classify.rs` 的 skip 清单）。

理论依据（2026-09-07 musk 会话推导定案，canonical 于
[docs/design/28-rust-interop-architecture.md](../design/28-rust-interop-architecture.md)）：
**编译预言机原则**——签名与布局只存在于编译期，但管线本就用同一工具链编译目标
crate（同编译保证已由 C3 指纹落地），因此任何类型问题都可以"再编一段探针代码去问
rustc"（`offset_of!` 支持 repr(Rust)、单态化实例可按需生成、trait 分发可在 shim
编译期定死）。430 字母表是这栋楼的一层，不是天花板。

## 1. 目标

1. **G1（V1-T2 语义层，先行）**：`Option<T>` 返回不再 skip——`Some(x)` 压 x /
   `None` 压 null（nullable 标记，返回码 `?` 前缀）；`Result<T,E>` Err 通道传导
   收口（复用 430-F unwrap_ok/auto__last_error 通道，Err → VMError 且 message
   含 Err 串）；参数位 `Option<T>` null→None / 值→Some。DIV-DEP-8
   格式化/字符串化对齐的**范围裁定**落地（裁定提案见 D2，修订确认门定案）。
2. **G2（V1-T1 布局层）**：三方结构体字段按真实布局可读可写（含乱序 repr(Rust)
   字段、嵌套两级句柄链、enum 判别）；布局 manifest（v2 `layouts` 段）与 cdylib
   同源；布局/features 变化指纹失配自动重建、缓存命中零重建（对抗测试①②钉死）。
3. **G3（工程面）**：全部用例落在 592 三轨 runner 上扩展（018 三轨精确相等、
   019 布局不变量、016/017 零回归）；uuid 非白名单勘测显式处置；文档与债务
   收口（Option/Result 语义进 syntax/guides；字段写失效语义登记 KNOWN-DEBT）。

**Non-goals（V1 不做，均显式去向）**：V2 全量（T3 白名单 trait 家族转发/Clone、
T4 泛型 mono、T5 反向 adapter——后续独立计划，设计文字保留）；by-value self /
move 收口（594 数据零命中，不提级）；多线程回调泵 / Send-Sync 纪律 / 借用记账
（28 号设计 §6 在途项）；BUILTIN_OPAQUE_CRATES 库的 VM native 面行为
（DIV-DEP-11 native 直出等——T2 生效面是 430 pack 管线，native 面登记不改）。

## 2. 架构方案

### 总原则：构建期制备、运行时装载

"动态调用任意 Rust 库"在 Rust 无稳定 ABI / 无运行时反射约束下的最大可行形态 =
**构建期由编译器本尊制备绑定（签名+布局+marshalling wrapper），运行期按指纹装载**。
V1 两层只是把"制备"的信息面从符号扩到语义、再到布局：

```
dep a(version/path) 显式声明（既有 Plan 092 通道，本计划不动）
   │ ①既有 430 管线（不动）：nightly rustdoc → classify → emit cdylib → C3 指纹缓存
   │ ②【T2·先行】语义段：classify Option/Result 返回规则 → wrapper nullable/fallible 发射
   │   → VM 错误通道/NULL 解码
   │ ③【T1·随后】布局段：探针 crate（offset_of!/size_of/enum 判别值）→ manifest v2 layouts
   │   → 指纹传导（布局行 + features 入 C3）→ DepOpaqueObject 挂 layout → CALL_SPEC offset 直读直写
   ▼
装载（dep_methods）：manifest format: 2 校验拒载旧版 + FUNCTION_SIGS + layouts
   ▼
use.rs a::b::c::D —— 字段直读写、Option/Result 语义可用
```

### V1 分层设计

- **T2 语义层（先行）**：`classify` 增 `Option<T>`/`Result<T,E>` 返回规则：
  `Option` → T 的返回码 + nullable 标记（`?` 前缀，`Some(x)` 压 x / `None` 压
  null）；`Result` → T 的返回码 + fallible 标记（Err 进 `auto__last_error` 通道
  转 VMError——430-F unwrap_ok 通道复用扩展，含自由函数 wrapper 路径的同一
  传导，DIV-DEP-12 收口面）。参数位 `Option<T>`：null → None / 值 → Some
  （构造探针 shim 由 wrapper 内联完成）。
- **T1 布局层（随后）**：emit 阶段追加生成一个**探针 crate**——对 manifest 中
  每个导出类型编 `offset_of!`/`size_of`/`align_of`（repr(Rust) 合法）与 enum
  判别值探针（match 计数法），结果写入 manifest v2 新增 `layouts` 段。VM 侧
  `DepOpaqueObject` 挂 layout，CALL_SPEC 属性访问按 offset 直读直写。
- **V2 行为等价（后续独立计划，本计划不执行）**：T3 trait 单态转发（白名单
  `Display`/`ToString`/`Clone`，`<T as Trait>::method` 编译期静态分发）、T4 泛型
  按调用点实例化、T5 反向 adapter 原型——设计与用例文字原样保留（见下方
  折叠节与 §6 V2 表），拆分去向在验收 AC-06 显式留痕。

<details>
<summary>V2 设计文字（原样保留，后续独立计划执行——不得静默丢弃）</summary>

- **T3 trait 转发层**：分类器从"trait impl 全排除"改为**opt-in trait 白名单**
  （首批：`std::fmt::Display`、`ToString`、`Clone`）。对白名单 trait 生成
  `<T as Trait>::method` 单态转发 shim——分发在 shim 编译期定死，无 vtable 跨界。
  594 数据：直接解锁 base64 整库（当前全红样本）+ Display/ToString 家族。
- **T4 泛型实例层**：rustdoc 泛型项不再直接 skip：`use.rs` 调用点携带的具体
  实参类型（基元/字符串/已注册句柄类型）回填 mono 提示 → 管线生成
  `foo::<ConcreteType>` 实例 shim（符号名按 430 sig_code 规则稳定生成）。
- **T5 反向 adapter 原型**：fixture 场景——Rust 方法形参为 `Box<dyn Fn(i64) -> i64>`
  时，生成 adapter 结构体持有 VM 任务句柄，trait/fn 调用时重入解释器执行 .at
  闭包并取回返回值。原型范围限定进程内单线程重入；Send/Sync 纪律与多线程泵
  归后续计划。
- **V2 用例（`019` 后续编号另定）**：V2-1 Display 转发 / V2-2 Clone 深拷贝 /
  V2-3 泛型自由函数 `pick<T: Ord>` 双实例 / V2-4 泛型方法 `Pair.max()` /
  V2-5 复合返回可读 / V2-6 反向 adapter `apply(5, |x| x*2+1)` == 11
  （fixture `autolang_traits`：Display/Clone impl + 泛型自由函数/方法 + 回调形参）。

</details>

## 3. 技术栈

- Rust（crates/shim-metadata + crates/auto-cache + auto-lang vm/tests 改动）；
  nightly rustdoc 管线照旧（v53 JSON，指纹已钉工具链版本串）；无新构建依赖。
- 勘测语料三方依赖：`uuid 1.24.0`（Cargo.lock 既有，零网络；**不在**
  BUILTIN_OPAQUE_CRATES——非白名单勘测的先决条件成立）。
- 复用 592 三轨 runner（`ffi_dep_parity_tests.rs`）与 594 parity dep 透传基建
  （`auto-parity/src/deps.rs` + `libs/dep/*_real` 范式 + `AUTO_LANG_PARITY_NET`
  门控）；测试不引新 crate。

## 4. 需求分析与背景调查

（2026-09-07 实勘 + 2026-09-09 r1 复勘，来源：musk 会话全链分析、28 号设计、
592/594 归档计划与执行证据、仓内代码锚点复验）

**授权记录（r1）**：用户 2026-09-09 指令授权——以 plan revision 模式修订本计划
（保持编号 591，不新建计划号），修订为 V1 先行可执行形态并准备执行；修订稿
完成后停下经用户确认，再按 /auto-plan:work 591 执行。两条硬约束：V2 拆分不得
静默丢弃；非白名单勘测可显式延后但不可无痕缩水。允许仓：本仓（crates/ +
test/ + parity/ + docs/）；执行于 worktree `D:/autostack/.wt/lang-591/auto-lang`
（Plan 529 分组布局；nextest `--all-features` 需解析跨仓 path 依赖 autodown-core
时，在分组目录建 auto-down detached 兄弟 worktree——592 已验证，禁 junction）。
无用户指定预算上限；全量门禁 `cargo tf` 归 review 阶段。

**已有机器（全部可复用，本计划不重造）**：

| 环节 | 载体 | 状态 |
|:---|:---|:---|
| dep 声明（path/git/version + features） | `compile.rs` `scan_dep_statements`（Plan 092）、`dep_sources`/`dep_features` | ✅（不动） |
| 沙箱取料/构建 | `auto-cache/src/sandbox.rs` | ✅ |
| rustdoc 签名提取 | `shim-metadata/src/rustdoc.rs`（nightly JSON v53） | ✅ |
| 分类/跳过规则 | `shim-metadata/src/classify.rs`（generic/closure/Option/exception 四类 skip） | T2 改动点 |
| shim 包生成 | `shim-metadata/src/emit_cdylib.rs`（`MANIFEST_FORMAT=1`@L16、`GENERATOR v1.2`@L15） | T1/T2 改动点 |
| C3 指纹 | `emit_cdylib.rs` `fingerprint_parts`@L111-138：payload = toolchain × crate × version × GENERATOR × CLASSIFIER_VERSION × 签名行（方法：参数 Ty+ABI 码+ret+fallible+chain+field；自由函数）；**布局与 features 均不在 payload**（r1 实勘——对抗①②的根因锚点；28 号设计 §4「指纹钉全配置」补强项即此） | T1 扩输入 |
| 编排/缓存 | `auto-cache/src/methods_pack.rs` + `lib.rs`（Plan 082 内容寻址）；**无 features 输入**（r1 grep 实证） | T1 扩输入 |
| 自由函数 wrapper | auto-cache 生成器（592 修复集：CStringOwned 折叠/syn 扫描/header 无条件）+ 装载键 `v3_{自由函数数}`（**P592-D1 弱失效：同数量换签名陈旧**，KNOWN-DEBT-AND-RISKS.md:1816） | T1 传导 |
| 运行时装载 | `vm/ffi/dep_methods.rs`（`DepOpaqueObject`、METHODS 表 dispatch 3000、`manifest.fingerprint` 校验）；592 已桥接 GET_FIELD→合成 getter（`p.x` 语法面） | T1/T2 改动点 |
| 三轨对拍 runner | `src/tests/ffi_dep_parity_tests.rs`（CASES 白名单 016/017；VM 腿 nightly 门控 + oracle 腿内容 hash 缓存 + a2r 腿 `AUTO_LANG_DEP_PARITY_A2R=1`） | ✅ 018/019 直接登记 |
| 语料范式 | `test/ffi_dual/016_dep_abi_matrix/`（`{{FFI_DUAL_DIR}}` 占位 + fn main 包裹 + `oracle/` 空 workspace 相对路径依赖 + 负面断言走测试体内 err-assert） | ✅ 照抄 |
| parity dep 透传 | 594：`parity/crates/auto-parity/src/deps.rs`（version 解析）+ `run_a2r` Cargo.toml 渲染 + `libs/dep/*_real` + phase p10 | ✅ uuid 勘测直接用 |
| 红面登记 | `parity/docs/known-divergences.md` DIV-DEP-1..14 | 续号 15+ |

**现状能力边界（013/016/017 实测）**：`Counter.new()` 句柄构造、方法调用、
基元/串返回、builder 链、static 方法、合成 getter 字段读（`p.x` 语法桥接）可用；
**真布局直读直写不可用**（getter 仅 pub + 标量/Str/Opaque 白名单字段，且无写）、
trait 方法（013 的 `to_string` 是 fixture 自带 inherent 的假象）、泛型、Option 返回
（classify L70-79 三方直接 skip）、Result 谓词（DIV-DEP-12 静默 None）均不可用。

**classify 现状精确定位（r1 实勘）**：`classify.rs` L32-33/L70-79——三方路径
`Option` 返回 v1 跳过；L142-145——`Result<T,E>` 已由 rustdoc 投影解包为 T +
`m.fallible=true`（wrapper 解 Ok + 错误通道），**但自由函数 wrapper 路径的
Err 传导不完整**（DIV-DEP-12：`from_str(bad).is_err()` VM 静默 None）。故 T2 的
Option 面 =「解除 skip + nullable 语义」，Result 面 =「错误通道在方法/自由函数
两条 wrapper 路径的传导收口」，均为增量而非从零造。

**墙的本质（定案，不再重议）**：Rust 类型信息只在编译期存在；布局无跨版本承诺；
`#[repr(Rust)]` 字段顺序未指定。解法是**同编译保证下的探针制备**——管线内
`offset_of!`（1.77+ 稳定，支持 repr(Rust)）拿到的就是真实偏移，指纹保证 manifest
与 cdylib 同源。风险：rustdoc JSON v53 无稳定性承诺 → 指纹已钉 nightly 版本串，
装载侧校验拒载；探针与 wrapper 同管线编译，布局一致性由同一 rustc 实例保证。

**594 命中率裁定（已回填原待澄清 #4，本修订落正文）**：五库 25 面绿 8（32%）。
T2（Option/Result）最高杠杆（~5 面直接解锁 + DIV-DEP-8 格式化/字符串化 ~7 面
建议并入范围裁定）；T3（trait 单态化）第二（base64 整库 + Display 家族）；
**by-value self 五库零命中，V2 move 收口不提级**。方法论注记：五库全在
BUILTIN_OPAQUE_CRATES——类型面实为 VM native 实现对拍，自由函数面才走真编译
pack → 非白名单库勘测单独立项（本修订并入 uuid，见 D5）。

## 5. 详细设计

### D1：T2 语义层——Option/Result 返回规则（先行）

- `classify.rs`：解除三方 `Option` 返回 skip（L70-79）→ 返回码 T + nullable
  标记（`types.rs` 返回码扩展 `?` 前缀，不改字母表本体）；`Result` fallible
  面补齐自由函数 wrapper 路径的 Err 传导（复用 430-F `unwrap_ok`/
  `auto__last_error` 通道，Err → VMError 且 message 含 Err 串）。
- `emit_cdylib.rs`：nullable/fallible wrapper 发射——`Option`：`Some(x)` 压 x /
  `None` 压 null；`Result`：Ok 压值 / Err 写错误通道；参数位 `Option<T>`：
  null → None / 值 → Some（wrapper 内联构造）。
- `dep_methods.rs`：nullable 返回解码（null 判定语义）+ 错误通道转 VMError。
- 注意：**VM native 面（BUILTIN_OPAQUE_CRATES 的类型构造器/方法）不经
  classify**，本设计不触 native 面（DIV-DEP-11 native 直出等维持登记）。

### D2：DIV-DEP-8 格式化/字符串化——范围裁定【裁定提案，修订确认门定案】

**提案：并入 V1/T2，最小切片**。594 报告：DIV-DEP-8 影响所有 rust-typed 值的
断言表达力（~7 面），是隐性天花板；且三态分歧的成因就是「a2r 发射 Debug /
VM pack 面占位 / VM native 面恰好 Display」。

切片范围（仅此，不提前 T3 全量）：

1. a2r 发射器：`print`/`.to(str)` 对 rust-typed 值发射 Display 形态
   （`format!("{}", x)`）而非 Debug——`trans/rust.rs` 一处口径对齐；
2. pack 面：wrapper 对 impl `Display` 的导出类型补 `__to_string` 转发 shim
   （`format!("{}", ...)`，编译期静态分发——**T3 机制的先行切片**，仅 Display
   一个 trait；T3 白名单其余成员 ToString/Clone 仍归后续计划）；
3. VM 侧：print/`.to(str)` 对带 `__to_string` 的 dep 对象路由该 shim；
   native 面（serde_json Value 等）已有各库自身 Display 语义，不动。

若裁定并入：`affects` 追加 `auto-lang/trans`，门禁加 `cargo tt`。
若裁定延后：DIV-DEP-8 相关红面以 DIV-DEP-15+ 显式登记「归 T3 后续计划」，
V1 语料以基元/串断言规避（018 断言面本就不依赖 Display，不阻塞）。
两案均满足"显式留痕、无痕缩水禁止"约束；确认门裁决。

### D3：T1 布局层——探针 crate 与 manifest v2

emit 阶段追加生成 `probe.rs`（与 shim 包同 cdylib 编译）：

- 每个 manifest 导出 struct：`offset_of!(T, field)` / `size_of::<T>()` /
  `align_of::<T>()` 逐字段导出为 `pub const`；
- enum：判别值用 match 计数探针（每 variant 一臂返回常量）；
- 嵌套类型字段记 `nested: "crate::Inner"` 引用，VM 侧按句柄链读。

manifest v2 新增段（`MANIFEST_FORMAT: 1 → 2`，装载侧 format 校验拒载旧版）：
`layouts: { type_name: { size, align, fields: [{name, offset, ty}], enum_discriminants?: [...] } }`。

**指纹传导（对抗①②的机制链，本设计的硬性要求）**：

- `fingerprint_parts` payload 增**布局行**（每导出类型的 size/align/字段偏移
  列表）——同签名集异布局（孪生 fixture、features 切字段型）必然异指纹；
- features 组合入指纹：`dep_features` 自 `compile.rs` 管线传导进 `PackMeta`
  并入 payload（28 号设计 §4「指纹钉全配置」的 features 半边落地）；
- `MANIFEST_FORMAT` bump 与 wrapper 发射改动（nullable/fallible/probe）→
  `GENERATOR` 升版（v1.3），确保全量缓存失效重建；自由函数 wrapper 版本键
  （现 `v3_{自由函数数}`，P592-D1 弱失效）至少传导 format/GENERATOR 变更，
  使 format bump 必然穿透到自由函数缓存键（对抗①依赖此链）。

VM 侧：`DepOpaqueObject` 增 `layout: Arc<LayoutInfo>`；CALL_SPEC 属性访问命中
DepOpaqueObject 时按 offset 直读（基元/串/嵌套句柄三分派）；**字段写**仅对
`&mut self` 上下文句柄开放（V1 范围：方法返回的可变句柄 + 顶层 `var` 绑定的
句柄字段赋值语句），写后不做 Rust 侧失效通知（单线程纪律，已知限制 #1）。
592 已落合成 getter 桥接（GET_FIELD → dispatch `Type.field`）——offset 直读
在其同一入口点分派（合成 getter 面保留兼容，layouts 命中优先）。

### D4：对抗测试设计（入验收，592 设计补列）

- **对抗①（同签名集异字段序孪生对）**：fixture 对 `autolang_shapes` /
  `autolang_shapes_b`——方法签名集完全相同、字段声明序不同（offset 不同）、
  各自带真值函数与可区分的默认值。连续装载两 crate，断言第二次装载**不复用**
  第一次的 pack（指纹不同 → 重建），字段读回**各自**真值。机制依赖链：
  布局行入 `fingerprint_parts` + 自由函数 wrapper 版本键传导 format bump
  （P592-D1 至少最小传导；全集单一化顺带收口或显式维持债务登记）。
  r1 实勘注记：`fingerprint_parts` payload 现含 `crate_name`，孪生异名 crate
  可能天然异指纹——执行期 T-08 先实证指纹差异来源，若 crate 名已区分则对抗①
  改为**同 crate 名变体重跑形态**（字段集变更不改签名）加链断言，layouts 入
  指纹的要求不放宽。
- **对抗②（features 组合改变字段类型）**：fixture 增 `#[cfg(feature = "wide")]`
  字段型变体（如 `count: i32` / `i64`）；语料 `dep fixture(path, features:
  ["wide"])` 两形态分别装载，断言指纹互异（重建发生）且两形态字段宽窄各自
  读回正确。机制链：features 入 `PackMeta`/payload（现无——r1 实勘确认）。

### D5：非白名单库勘测（uuid）

- 载体：`parity/libs/dep/uuid_real/`（594 `*_real` 范式：README + tests/auto/*.at
  TAP 语料 + tests/rust oracle 同版 pin `uuid = "=1.24.0"`）；门控
  `AUTO_LANG_PARITY_NET=1`（uuid 已在 Cargo.lock/~/.cargo，离线可跑）。
- 用例（green-first，2-4 条，全部确定性输出）：`Uuid.parse_str(s).unwrap()`
  （**Option 返回 unwrap 面 + T2 联动**）→ `.to(str)`（Display 面，DIV-DEP-8
  联动）/ `.get_version_num()` / `.is_nil()`；红面逐条 DIV-DEP-15+ 登记不进 TAP。
  uuid 不在 BUILTIN_OPAQUE_CRATES → 类型面也走真编译 pack——正是 594 方法论
  注记要补的「非白名单 pack 面命中率」首个样本。
- **处置约束（硬约束）**：执行中若判定体量超 V1（如 a2r 腿发射面意外红且
  修复越界），允许显式登记为后续勘测计划（编号另取 + KNOWN-DEBT/计划注记
  留痕 + 已钉语料资产保留），**不可无痕缩水**；默认路径 = 并入 V1 完成。

### D6：V2 拆分去向（显式留痕，不得静默丢弃）

V2（T3/T4/T5 + V2-1..V2-6 用例 + fixture `autolang_traits` 设计）拆为后续
独立计划：编号在 V1 收口时经 `scripts/new-plan.sh` 另取（或用户指定）；本计划
§2 折叠节保留全部设计文字与用例表作为该计划的起草底稿；V1 收口时在
KNOWN-DEBT-AND-RISKS.md 与本计划复审记录各留一行去向指针。D2 若裁定延后案，
DIV-DEP-8 对齐面同样挂接该计划（与 T3 Display 白名单同源）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... 目标 | before/after 规则 | rationale | acceptance IDs |
|:---|:---|:---|:---|:---|:---|
| SD-01 | modify | shim-metadata/project.md | before: 三方 Option 返回 skip；Result 谓词自由函数路径静默。after: Option→nullable `?` 返回码（None→null）；Result Err→auto__last_error→VMError，方法/自由函数双路径传导 | 594 五库最高杠杆裁定 | AC-01, AC-03 |
| SD-02 | modify | shim-metadata/project.md | before: MANIFEST_FORMAT=1、无布局信息、指纹=签名集。after: manifest v2 `layouts` 段（探针 crate 制备，format: 2 拒载旧版）+ 指纹 payload 增布局行与 features | 编译预言机原则（28 号设计 §4）| AC-01, AC-02, AC-04 |
| SD-03 | modify | auto-cache/project.md | before: C3=工具链×crate×版本×生成器×签名集，无 features；自由函数 wrapper 键 v3_{数量}。after: 指纹增 features 组合与布局行传导；自由函数 wrapper 版本键传导 format/GENERATOR（P592-D1 最小收口） | 对抗①②机制链；28 号设计「指纹钉全配置」 | AC-02, AC-04 |
| SD-04 | modify | auto-lang/vm/design/ffi.md | before: DepOpaqueObject 无布局、字段仅合成 getter 读、Option/Result 未收口。after: layout 挂载 + CALL_SPEC offset 直读直写（写限 &mut 上下文句柄）+ nullable 解码 + 错误通道 VMError | V1 能力面；写失效语义=已知限制 #1 | AC-01, AC-05 |
| SD-05 | add | parity/project.md | 新增 uuid_real 非白名单勘测语料条目（pin 1.24.0、网络门控、pack 面首样本）+ 处置结论（完成或显式延后） | 594 方法论注记闭环 | AC-06, AC-07 |
| SD-06 | modify | auto-lang/trans/（仅 D2 并入案） | before: print/.to(str) 对 rust-typed 值发射 Debug。after: 发射 Display 形态 | DIV-DEP-8 三态对齐 | AC-01 |

（若 D2 裁定延后案：SD-06 撤销，DIV-DEP-8 登记条目替代；SD-05 按处置结论
改写。review 阶段定稿 frontmatter 三字段。）

## 6. 测试设计

沿用 592 三轨 runner 范式：checked-in 语料 + fixture crate + 手写 oracle，
nightly 缺席自动 skip（全程离线，fixture 均为本地 path 源）；负面断言按 592
惯例走测试体内 err-assert（错误输出不进 stdout golden）；语料规避惯例：自由
函数先 let 绑定再 print（DIV-DEP-5）、String 实参用 `let x str` 注解或方法
结果传参（DIV-DEP-7）。

fixture 新增（放 `test/ffi_dual/018_dep_fields/fixture/`）：

- `autolang_shapes`（V1 主 fixture）：乱序字段 struct `Messy`（`u8/String/u8/
  i64/bool` 交错声明）、嵌套 struct、`Option`/`Result` 返回方法、enum + 判别、
  `&mut self` 变异方法、字段校验真值函数。
- `autolang_shapes_b`（孪生）：方法签名集与 `autolang_shapes` 完全相同、字段
  声明序不同、真值可区分（对抗①）。
- features 变体：`#[cfg(feature = "wide")]` 字段型切换（对抗②）。

### V1 用例（`018_dep_fields` 三轨 + `019_dep_layout_invariants`）

| # | 用例 | 层 | `.at` 语料核心 | 断言 |
|:--|:---|:---|:---|:---|
| V1-4 | Option 返回 | T2·018 | `find(key) -> Option<&str>`：存在键 / 缺失键 | 前者返回串；后者 VM 侧 null（is_none 判定通过） |
| V1-5 | Result 返回 | T2·018 | `parse(s) -> Result<Point, String>`：合法/非法 | 合法 → 字段可读；非法 → VMError 且 message 含 Err 串 |
| V1-1 | 乱序字段读 | T1·018 | `use.rs autolang_shapes::Messy` → `m.a / m.tag / m.count / m.flag` | 与 fixture 真值函数 `messy_truth()` 逐一相等（探针偏移正确性由乱序声明放大） |
| V1-2 | 字段写 | T1·018 | `m.count = 9` 后再读 + 调用 fixture `m.count_snapshot()` | VM 侧读 = 9；**Rust 侧方法读回 = 9**（写真达 cdylib 堆） |
| V1-3 | 嵌套结构 | T1·018 | `o.inner.tag / o.inner.n` 两级句柄链 | 真值相等 |
| V1-7 | enum 判别 | T1·018 | `shape_kind() -> Kind` + `is_circle()` 判别探针 | 判别值与 match 语义一致 |
| V1-6 | 布局指纹防 stale | T1·019 | fixture `Messy` 加字段后重跑 V1-1 | 旧缓存拒载/重建，新字段读通；同指纹连跑第二次零重建（构建计数器/mtime 断言） |
| 对抗① | 孪生布局防陈旧 | T1·019 | `autolang_shapes` → `autolang_shapes_b` 连续装载 | 第二次装载靠 layouts 差异重建，字段读回各自真值（同签名集指纹相同场景拒 stale，见 D4 实勘注记） |
| 对抗② | features 入指纹 | T1·019 | `dep fixture(features: ["wide"])` 两形态 | C3 指纹必变（重建发生），宽窄字段各自读回正确 |

### uuid 勘测（`parity/libs/dep/uuid_real/`，网络门控）

见 D5。TAP 三轨（VM/a2r/oracle）；红面 DIV-DEP-15+ 登记不进 TAP。

### 门禁与回归（按改动面分级，AGENTS.md Category B）

| 改动 | 门禁 |
|:---|:---|
| classify/types/emit（shim-metadata） | shim-metadata 相关单测 + 生成器指纹校验（GENERATOR/FORMAT bump 后全量缓存重建一次性成本） |
| dep_methods/engine（VM） | `cargo tv`（纯 .at 语料 golden 兜底；aavm 零触发——本计划不触 `auto/lib`、aavm2 语料） |
| trans/rust.rs（仅 D2 并入案） | `cargo tt` |
| 日常档 | `cargo t ffi_dual`（013/016/017/018/019）+ `cargo t dep_parity`（VM+oracle 腿）|
| a2r 腿全量 | `AUTO_LANG_DEP_PARITY_A2R=1 cargo t dep_parity`（本地/CI） |
| uuid 勘测 | `cargo test -p auto-parity` + `AUTO_LANG_PARITY_NET=1 phase p10`（或新 phase） |
| review 阶段 | `cargo tf`（full 档兜底） |

## 7. 验收标准

1. **AC-01**：V1-1..V1-7 全绿（乱序字段读/字段写断言 Rust 侧读回/嵌套两级
   句柄链/Option 存在缺失/Result 合法非法/布局指纹防 stale/enum 判别）。
   验证：`cargo t ffi_dual_018` + `cargo t ffi_dual_019`。
2. **AC-02**：两个对抗测试绿——同签名异布局拒 stale（孪生对连续装载）、
   features 组合入指纹（重建发生 + 宽窄各自正确）。验证：019 语料断言。
3. **AC-03**：新语料 018 在三轨 runner 下 VM/a2r/oracle stdout 精确相等
   （`AUTO_LANG_DEP_PARITY_A2R=1 cargo t dep_parity`）；016/017 零回归
   （`cargo t ffi_dual` 全绿）。
4. **AC-04**：manifest v2 `layouts` 段装载校验生效（旧 format 拒载）；
   GENERATOR/format bump 后全量缓存重建且二次命中零重建。
5. **AC-05**：Option/Result 语义（None→null、Err→VMError、nullable 判定）
   进 syntax 文档或 guides；use.rs 能力矩阵补 V1 面；GOAL-006 状态推进留痕。
6. **AC-06**：V2 拆分去向 + uuid 勘测处置在计划与 KNOWN-DEBT 显式留痕
   （后续计划指针 / 勘测完成或延后登记），无静默丢弃、无无痕缩水。
7. **AC-07**：新增分歧全部登记 DIV-DEP-15+（如有），格式对齐既有条目
   （签名面/三面表现/归类/解锁条件）。
8. **AC-08**：门禁按面零回归——`cargo tv`（VM 改动兜底）、`cargo tt`（若
   D2 并入）、`cargo check -p auto-lang` 零新警告、探针残留扫描为零；
   review 阶段 `cargo tf` 通过。

## 8. 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成]
一行证据。执行于 worktree `D:/autostack/.wt/lang-591/auto-lang`——worktree 内
禁 junction/symlink，移除前必跑 wt-guard；跨仓 path 依赖 autodown-core 需解析时
建 `D:/autostack/.wt/lang-591/auto-down` detached 兄弟 worktree（592 先例）。）

- [ ] **T-00** 收口修订：用户确认本修订稿（含 D2 裁定）→ master 提交修订；
  建 worktree `git worktree add D:/autostack/.wt/lang-591/auto-lang -b
  plan-591-dev`。验证：`git worktree list` 含 lang-591。
- [ ] **T-01**（T2-classify）`shim-metadata/src/classify.rs` 解除三方 Option
  返回 skip → nullable 计划 + `types.rs` 返回码 `?` 前缀与 fallible 传导补全
  （自由函数路径同覆盖）；`#[cfg(test)]` 单测覆盖 Option/Option&str/Result
  返回分类。验证：`cargo test -p shim-metadata` 分类单测绿 + `cargo check -p auto-lang`。
- [ ] **T-02**（T2-wrapper/VM）`emit_cdylib.rs` nullable/fallible wrapper 发射
  （Some 压值/None 压 null；Ok 压值/Err 写 `auto__last_error`；参数位 Option
  内联构造）+ `dep_methods.rs` nullable 解码与错误通道转 VMError。验证：
  shim 发射单测 + 手工小 corpus 冒烟（Option/Err 面）。
- [ ] **T-03**（T2-corpus）fixture `autolang_shapes` 建档（含 Option/Result
  方法面）+ `test/ffi_dual/018_dep_fields/` 语料（先注册 V1-4/V1-5，占位
  `{{FFI_DUAL_DIR}}` + fn main 包裹 + `oracle/` 空 workspace 相对路径依赖）+
  `ffi_dep_parity_tests.rs` CASES 登记 018。验证：
  `cargo t ffi_dual_018` + `cargo t dep_parity_018`（VM+oracle 腿绿）。
- [ ] **T-04**（D2 裁定执行案）若裁定并入：`trans/rust.rs` print/`.to(str)`
  Display 口径 + wrapper `__to_string` 最小转发 + VM print 路由；语料补
  Display 断言面。验证：`cargo tt` 零回归 + 018 新增面三轨绿。
  若裁定延后：DIV-DEP-15+ 登记落盘（归后续 T3），本步转为登记任务。
- [ ] **T-05**（T1-probe）`emit_cdylib.rs` 探针 crate 生成（offset_of/size_of/
  align_of/enum 判别）+ `types.rs` LayoutInfo + `MANIFEST_FORMAT: 2` + 装载侧
  （`dep_methods.rs`）format 校验拒载旧版；`fingerprint_parts` payload 增布局行；
  `GENERATOR` 升 v1.3；features 自 `compile.rs` dep_features 传导进
  `methods_pack.rs` `PackMeta` 入 payload；自由函数 wrapper 版本键传导
  format/GENERATOR（P592-D1 最小收口）。验证：`cargo test -p shim-metadata`
  + `cargo test -p auto-cache` + 对 fixture 产出的 manifest 含 layouts 且偏移
  单测对拍。
- [ ] **T-06**（T1-VM）`dep_methods.rs` `DepOpaqueObject` 挂 `layout:
  Arc<LayoutInfo>` + CALL_SPEC 属性访问 offset 直读直写（基元/串/嵌套句柄
  三分派；写限 &mut 上下文句柄；合成 getter 面保留兼容）。验证：
  `cargo tv` 零回归 + 013/016/017 零回归。
- [ ] **T-07**（T1-corpus）018 扩展 V1-1/V1-2/V1-3/V1-7 用例（乱序读/写/
  嵌套/enum 判别）。验证：`cargo t ffi_dual_018` +
  `AUTO_LANG_DEP_PARITY_A2R=1 cargo t dep_parity` 三腿精确相等。
- [ ] **T-08**（对抗语料）`autolang_shapes_b` 孪生 fixture + features cfg 变体
  + `test/ffi_dual/019_dep_layout_invariants/` 语料（V1-6 + 对抗① + 对抗②；
  先实证 D4 注记的指纹差异来源，按实勘形态落断言）。验证：
  `cargo t ffi_dual_019`（重建/零重建与真值断言全绿）。
- [ ] **T-09**（非白名单勘测）`parity/libs/dep/uuid_real/`（README + TAP 语料
  2-4 条 + oracle 同版 pin 1.24.0）+ phase 注册；红面 DIV-DEP-15+ 登记。
  若判定体量超界 → 显式登记后续勘测计划并留痕（D5 硬约束）。验证：
  `cargo test -p auto-parity` + `AUTO_LANG_PARITY_NET=1` 跑 uuid_real 三轨。
- [ ] **T-10**（文档/债务收口）Option/Result 语义进 syntax/guides + use.rs
  能力矩阵 V1 面 + GOAL-006 留痕；KNOWN-DEBT 登记：字段写失效语义（已知
  限制 #1）+ V2 拆分去向指针（D6）+ uuid 勘测处置 + DIV-DEP-15+ 引用。
  验证：文档 diff 人工核对。
- [ ] **T-11**（收口健康检查）门禁按面全跑：`cargo t ffi_dual`、
  `dep_parity`（+a2r 腿）、`cargo tv`、（若并入）`cargo tt`、
  `cargo check -p auto-lang` 零新警告、探针残留扫描（TMP/eprintln 六模式）。
  验证：输出留痕进复审记录；`cargo tf` 留给 review 阶段。

## 9. 复审记录

**r1 修订交接（2026-09-09，/auto-plan:new revision 模式）**

- `stage: new`（revision），PLAN-591，`plan_revision: 1`（基线 = 2026-09-07
  裁定修订稿；本修订未执行任何代码，进度无失效项）。
- `outcome: blocked`——等待用户两项输入：①修订稿确认（含 V2 拆分与
  uuid 勘测并入两项硬约束落位）；②D2 裁定提案（并入最小切片 vs 延后归 T3）。
- `next: work`——确认后按 `/auto-plan:work 591` 执行（T-00 起）。
- 变更任务/验收 ID：全部重排——执行步骤 T-00..T-11（total_steps 6→11），
  验收 AC-01..AC-08；V2 用例与 T3/T4/T5 设计文字保留于 §2 折叠节（归后续
  独立计划）。

## 10. 待澄清事项

1. **字段写的失效语义**（r1 裁定，承原稿建议）：VM 侧写字段后，若 Rust 侧
   缓存了旧值（fixture 内 `OnceLock` 类）不刷新——单线程纪律下**接受为 V1
   已知限制**，T-10 登记 KNOWN-DEBT；多线程/失效通知语义归后续计划。
2. ~~**T5 原型边界**~~（随 V2 拆分移交后续计划，原裁定保留：原型先行，
   单线程重入深度 1，musk 消费评估后另立计划）。
3. ~~**测试编号回改**~~（已消化：正文改用 018/019；014/015/016/017 已占用）。
4. ~~**五库 skip 面命中率**~~（已消化：594 报告结论落 §4 背景调查与 r1 修订
   要点，T2 先行）。
5. **DIV-DEP-8 范围裁定**（r1 新增，确认门裁决）：并入最小切片（SD-06 +
   affects 追加 trans + `cargo tt` 门禁）vs 延后归 T3 后续计划（DIV-DEP-15+
   登记替代）。提案=并入，理由见 D2。
6. **对抗①指纹差异来源**（r1 新增，执行期 T-08 实证）：`fingerprint_parts`
   payload 现含 `crate_name`——孪生异名 crate 可能天然异指纹。若实勘确认，
   对抗①改为同 crate 名变体重跑形态（不改签名改布局），layouts 入指纹的
   要求不放宽（防 P592-D1 同型陈旧）。
7. **features 传导路径**（r1 新增，执行期 T-05 实勘）：`dep_features` 现存于
   `compile.rs`，未入 `methods_pack.rs` 管线——传导的最小切口（PackMeta 增
   字段 vs 指纹字符串拼接）执行期定，验收以对抗②行为断言为准。

> 已裁定（2026-09-07）：~~V1 触发自动化（use.rs 自动解析 + auto.lock + 首引用
> 阻塞回填）~~ **取消**——`dep` 声明的 version/git/path 源配置有价值，保留
> 两步形态；原 V2/V3 顺位上移为 V1/V2（r1 起 V2 归后续独立计划）。
