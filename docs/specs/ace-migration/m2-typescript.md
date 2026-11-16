# ace-migration: M2 TypeScript/JavaScript 目标（PLAN-976）

状态：M2-A 已验（2026-11-16，归档收据见 [PLAN-976](../../plans/archive/)）。共享
Checked-HIR TS/JS provider 及其 UI 中立 fragment 接口、worker 执行通道与真实 node/tsc
验证均已落地；产品 UI 适配器替换（Vue/Ark 四处 legacy 委托）归 PLAN-980，旧
`TypeScriptTrans`/`JavaScriptTrans` 退役归 PLAN-987。

## 身份与目标（T-01 冻结）

- 新 crate `experimental/ace/crates/a2x-ts`（package `ace-a2x-ts`）：一个共享 lowering
  服务两个目标，目标差异只存在于文件命名、类型注解槽与身份常量（`target.rs`）。
- 身份：backend `auto-a2x-typescript` / `auto-a2x-javascript`；standard
  `typescript5.9-es2022-erasable`（可擦语法——无 enum/namespace/参数属性，node 原生
  type-strip 可执行）/ `es2022-node-esm`；fragment 层 `AXTS1..3` / `AXJS1..3`（与
  AXR/AXC/AXPY 命名空间互不碰撞，`boundaries.rs` 断言）；fileset schema
  `ace.a2ts-hir.{scalar,record-text,collections}.fileset/1`；producer
  `ace-a2x-ts/modules/1`（semantics-contract 已注册）；worker recipe
  `a2ts-hir-{typescript,javascript}-{checked,record-text,collections}`。
- 唯一输入是 `SemanticView`（a2x-core 只读受检视图）——不新建前端/模块系统/语义分析，
  模块与 `use` 由 store 前端扁平化（packages lane，见下）。
- 工具链：node 原生执行（`.ts` type-strip / `.mjs` ESM）；类型检查腿 `tsc --noEmit
  --target es2022 --module nodenext --strict`（`A2TS_TSC` 可覆盖；缺 tsc = 该腿具名
  infra skip，非语言失败、非 PASS）。观察协议：stdout `value:<i32>` / `trap:<Code>`，
  短路 marks 走 stderr（`mark_a`/`mark_b`）。

## 值表示与运行时合同

- `i32/i8/u8/u32/f32/f64/char→number`，`i64/u64→bigint`（>2^53 保真；混用
  number/bigint 即 TypeError，cast 点显式）；`str→Uint8Array`（UTF-8 字节精确，内嵌
  NUL 保留，`len`/`len_bytes`/`char_at`/slice 按 Auto 语义，不用 JS UTF-16 string）；
  `Str→A2xOwnedStr`（显式 free + ledger，重复 free→`trap:DoubleClose`）；Record=普通
  object、List=`A2xList`、Variant=`A2xVariant`（投影为破坏性移出）、Closure=JS
  function+env 工厂。
- checked 算术 trap（越界 `trap:Overflow`；`/`、`%` C 语义向零截断，不用 JS 浮点除）；
  f32 每步 round、NaN 载荷位保真；预算 `a2x_bill` 超限 `trap:Budget`。
- 资源：拥有 `Str` 释放集合取自验证器计划（`body_plan`/`body_facts` 的 owned，
  非 provider 自推）；函数级 `try/finally` 单一清理点；复合值（record/list/variant/
  closure）由 JS GC 管理（对 a2py 的具名目标差异，不宣称全部 Auto 资源语义等价）。
- 版本化 runtime `a2x_rt.mjs`（+TS 腿 `a2x_rt.d.mts`）：body sha256 参与每个 fragment
  key（runtime 变更只失效对应产物）；stdlib 端口目录 `ports.rs` 镜像
  `stdlib/contracts/operations.toml`（方向/semver/dep 令牌逐调用校验）。
- manifest 绑定 backend/backend_version/target_lang/standard/runtime_version/
  runtime_sha256/print_version/stdlib_port_catalog/input_hash/source_sha256/entry/
  functions[]/files[]（逐文件 sha256）；验证器拒错 producer/schema/tamper/缺文件
  （`cert.rs`，`a2ts.cert`）。

## 具名拒绝（无旧链 fallback）

`a2ts.foreign-unsupported`（foreign 声明/凭证）、`a2ts.sched-unsupported`
（`__sched_*` 内在，共享 HIR 无 await 节点）、`a2ts.platform-unsupported`（平台
IO 内在）、`a2ts.unsupported-*`（视图外构造）、`a2ts.depth-budget`
（`MAX_STRUCTURE_DEPTH=64`，与 a2py 同常量同码位）、`a2ts.closure-owned-capture`
（闭包捕获拥有 `Str`：环境无确定释放点）、`a2ts.seed`/`a2ts.credential`（无 view
且无 seed/凭证不匹配）。sched/depth 有永久负控（`ts_named_rejections.rs`）；
foreign/closure-owned-capture 为复审核读+D-976 债务候选（见 KNOWN-DEBT）。

## 消费接口

1. `TargetProvider` 全链路：单源 cold/warm/restart（`A2xSession`）与多模块
   packages lane（`PackageA2xTarget` + `compile_package_product`：模块/use/init 扁平
   化后由 provider 发射扁平函数集，跨模块调用与 std 端口真实执行——
   `tsready_976.rs` 以两模块+std.math/std.text 用例对齐 a2py 同 oracle）。
2. UI 中立 fragment 接口（`fragment.rs`，PLAN-980 替换 4 处 legacy 委托的冻结面）：
   `emit_stmt_fragment` / `emit_expr_fragment`（输入 view+body+定位符+显式程序上下文
   ——scalar enum 集/struct 集/unit variant 集；输出文本+所需 runtime helper 名+locals
   映射）；宿主函数级清理与响应式/capture/宿主变量替换归 UI 适配层；程序上下文缺
   构造=具名拒 `a2ts.fragment-context`；`innermost_expr_in_span` 提供源 span 定位。
3. worker 通道：`a2x-worker` ops `daily-emit-ts`（spawn-zero）与 `run-ts`（真实
   node：value/trap 行+退出码+引擎身份 `ScriptRuntimeIdentity`）；identity 注册两目
   标与六 recipe；拒绝的 emit 不触碰既有 out_dir（失败保旧）。

## 证据锚（仓内永久测试）

`a2x-ts`：`ts_runtime_and_provider.rs`（runtime 真实 node：UTF-8/NUL/checked/bigint/
f32 位保真/variant 投影/list 越界/端口合同/budget/marks stderr/入口协议含
trap:Internal；manifest tamper/错 producer/错 schema；层身份与 legacy 禁令）、
`ts_ui_fragment.rs`（fragment 接口真实 node 双腿+上下文/越界具名拒）、
`ts_named_rejections.rs`（sched/depth 负控）。`a2x-worker`：`tsready_976.rs`
（packages lane 两模块=模块初始化/dispatch、短路+命名实参源序、>2^53 双腿、
source map 原 span、tsc 严格腿、cold/restart 字节同 manifest、失败保旧）、
`boundaries.rs`（层命名空间/store 无 target 依赖/依赖白名单）。完整职责组
`ace-a2x`（`scripts/test-groups.toml` 已含 `ace-a2x-ts`）。

## 未闭边界（具名）

Vue/Ark 适配器实际切换与响应式映射归 PLAN-980（本接口已由独立 fixture 验证，不反等
980 整包）；独立全语言通用 a2ts/a2js CLI 非自动新增门（生产工程实际依赖项才入
required 闭包）；旧 emitter 退役与旧 `a2ts/a2js` 语料去向归 PLAN-987；Godot 目标按
2026-11-16 用户裁定后置（本包不含，规范已验事实不伪造）。
