# M2-A 编译期执行：宏、元数据与目标条件（PLAN-966）

> Status: active（2026-11-14，PLAN-966 A 片交付）。
> 实现：`experimental/ace`（`grammars/auto-source.at` 的 `packages-ct` 面、`core::source::{ast,cst,packages}`、`source-query` store、`av` 产品腿）。
> 前置来源：1043（`#in` 最小加载/装配）、936（标量 CT/元数据查询）、958（A2X worker CT 前门）。

## 范围

本文件沉淀 ACE 新前门（`packages-ct`）上的编译期执行面：项位 `#in/#if/#is/#for`、
语句位 `#if/#for/#is`、宏 `#name(args)`，以及它们共享的目标事实域、预算、诊断与身份规则。
它扩展 1043 的"最小 `#in`"描述，不取代 1043 的装配/来源合同。

## 支持面（A 片，已验收）

| 位置 | 形态 | 语义 |
|---|---|---|
| 项位 | `#in sys is current { Windows -> use winio.plat  Linux -> use lnxio.plat }`、`#in sys { is/if/for … }` | 1043 来源；`#in` 建立选择域，内部 `is/if/for` 继承编译期属性（不重复 `#`） |
| 项位 | `#if cond { decl* } else { decl* }` | cond 为 CT bool；选中分支的声明/导入落回本模块 |
| 项位 | `#is expr { CONST -> use/decl … }` | 平台判别值等值选臂；**项位无 `else` 臂** |
| 项位 | `#for i in a..b { decl* }` | i32 计数展开，预算 1024 |
| 语句位 | `#if cond { stmt* } else { stmt* }` | cond 为 CT bool；未选分支不落语句 |
| 语句位 | `#for i in a..b { stmt* }` | 重复展开；循环变量只在 CT 谓词域可见 |
| 语句位 | `#is expr { pat -> { stmt* } … else -> { stmt* } }` | 值相等选臂；`else` 必须末位且至多一个；`->`/`=>` 均接受 |
| 语句位 | `#name(args…)` | 宏调用糖：投影为同名前门函数调用（`let __ace_macro_<源位置> = name(args)`），返回值按语句丢弃 |

未选分支只做 Text 阶段语法检查（语法坏仍报），不做绑定/解析；未选分支的平台库缺失不阻本目标绑定。

## 求值时机

全部在**装配期**折叠（`assemble_unit` 末尾、resolve 之前）：按显式目标事实求值，
不进入 VM、不做运行期判定、不生成运行期分支。折叠后 AST 不再含 CT 语句变体；
bind/resolve 两处 `source.cte.unfolded` 与 semantics 受控 Err 是防御性残留拒（正常路径不可达）。

## CT 谓词域

- 字面量：`i32`（越域 `source.cte.in-domain`）、`bool`（`true`/`false`）。
- 成员：`sys.<member>` 或裸成员名；命名空间仅 `sys`。
- 运算：`== != < <= > >=`（i32）、`&& || !`（bool）；`#if` 条件必须是 bool。
- 字面量/成员以外的形态按 `source.cte.in-expr` 拒。

## 事实边界

| 事实 | 来源 | 是否进装配身份 |
|---|---|---|
| `sys.current`（目标判别值）、`Windows`/`Linux`/`Mac` | 显式装配目标（`PackageAssemblyContext`） | 是 |
| `sys.host`（宿主判别值） | 编译进程 OS（**宿主事实，非目标事实**） | 是（同 target 换宿主会换身份） |
| `sys.arch`/`sys.env`/`sys.abi` | **无显式输入**：未注册成员按 `source.cte.in-name` 具名拒（不静默默认、不读宿主环境） | — |

arch/env/ABI 的正式来源分账：978（target triple/SDK）、967（FFI/ABI）、979（构建配置面）；
显式输入交付后按其事实扩面并更新本表。

## 预算与身份

- `#for` 迭代上限 1024（项位与语句级共用同一常量）；越界 `source.cte.in-budget`，诊断落在语句/范围 span。
- 装配身份 = 公共源 + 所选 provider 实现层 + 目标事实（backend/provider/target/availability/host）；
  未选 provider 层不入身份（1043 ASM-04）；平台臂的**未选模块**是普通编译单元，其源改动属相关失效
  （读取集型身份是具名债务，见 KNOWN-DEBT D-966-T05）。

## 诊断码

| 码 | 触发 |
|---|---|
| `source.cte.in-expr` | 非字面量/成员/受支持运算的 CT 表达式 |
| `source.cte.in-domain` | i32 字面量越域 |
| `source.cte.in-type` | 非 bool 条件；`#is` 匹配域类型不符 |
| `source.cte.in-name` | 未知 namespace 成员（含 arch/env/ABI 未提供事实） |
| `source.cte.in-namespace` | 限定名/臂限定符与 `#in` 命名空间不符 |
| `source.cte.duplicate-arm` | 等值重复臂；重复 `else` |
| `source.cte.else-position` | `else` 不是最后一臂 |
| `source.cte.in-budget` | `#for` 超过 1024 次 |
| `source.cte.macro-args` | 宏命名实参（不支持） |
| `source.cte.unfolded` | 防御性残留拒（bind/resolve） |
| `source.module-symbol` | 未知宏名/未知被调函数 |
| `source.module-missing` | 选中臂的模块缺失 |

## 宏合同

- 语句位 `#name(args)` 展开为同名前门调用，落卫生局部 `__ace_macro_<源位置>`（保留前缀，位置唯一）。
- 名称按普通模块作用域解析：未知→`source.module-symbol`；被调用者必须是函数；命名实参→`source.cte.macro-args`。
- 宏**不**做文本替换、不做 legacy VM `Log.*` 隐式路由；目标专有发射归 967 及其目标 owner。
- `#name()` 不引入新语法面（无 `#name!`、无编译期块参数）。

## 元数据

`FieldCount`/`HasField` 查询（`cte.meta.fields/v1`）由 936 交付、958 worker 前门承载；
本前门通过 worker 消费同一合同（V-02 复用证据）。普通 AV 接受 `ct_field_count(Type)`、`ct_has_field(Type, field)`，使用对应记录在本次绑定中的字段事实。泛型参数作选择子时，先使用各个实例的具体记录，替换结果以函数实例和原源跨度定位。跨模块字段增加、改名或恢复会更新依赖查询。已证明只读取入参和字面量的独立闭合标量调用可维持自己的冻结槽。涉及外部声明的调用保完整源闭包身份。

普通 AV 属性按消费端路由，`#[test]` 进入 971，`#[api]` 进入实际 AV Web 服务，外库继续由 ABI provider 授权。详见 [PLAN-1092](../../plans/archive/1092-av-parity-attributes-and-metadata.md)。任意运行期反射、字段遍历和自动 serde 仍是各自的后继能力。

## 阶段分账

A 片（本文件）已验收；B 片（985 承接）未开工——不得据本文件宣称 B 已支持。

## 证据

- core：`source::packages::binding::plan966_tests`（15 例：bool 谓词、预算、V-09 反例、生成声明/常量、`#in`+宏、namespace 不泄漏）。
- store：`modules::packages_store::tests::plan966_ct`（4 例：目标切换新身份、同输入复用＋选中失效、失败保旧/恢复、namespace 不泄漏）。
- 产品腿：`av::conformance::plan966_ct_product`（2 例，AV/native/A2R/A2C/A2PY 独立 oracle 112/222 与 10/20）。
- 复用：936 `cte_schedule_meta_*`、958 `worker_ct_frontdoor_958`、1043 `stdlib_assembly_1043`/ASM-04。
- 终审并集：`cargo t ace-syntax ace-source ace-semantics ace-comptime ace-artifacts ace-av ace-native ace-a2x`（run-peuyj69x；ace-native 唯一红为 Linux/WSL 裁定排除项）。

## 来源

`docs/plans/archive/966-ace-m2-metaprogramming.md`（归档）；`docs/specs/ace-migration/m2-denominator.md` L-32..L-36；
`docs/specs/auto-lang/comptime/design/hash-syntax.md`。

## 普通 AV 接面（PLAN-1091）

普通 Auto 前门复用本版装配选择，涵盖单文件的声明和语句，以及包闭包里的受检导入。AV worker 以本地 AV 目标作为显式装配输入。包绑定也接受调用方给的 `PackageAssemblyContext`，目标选择读取 `sys.current`，编译宿主事实由 `sys.host` 单独提供。两项事实参与包输入身份。原 `packages-ct` 和其他窄面沿用原多后端合同。

普通 AV 的 `#{expr}` 在装配和实例化后由 source-cte 服务，再交共享类型验证。前门要求 `ac.source.cte.scalar/v1`，元数据仍受原授权控制。冻结的 AV 表面值使用 `int/i32` 与 `bool`。本地 AV 把四个日志宏投影为受检的控制台操作，其他 `#name(args)` 走普通函数绑定。装配有选择跨度，CT 失败带 demand 源跨度。具体行为和验收范围见 [PLAN-1091](../../plans/archive/1091-av-parity-comptime-and-macros.md)。
