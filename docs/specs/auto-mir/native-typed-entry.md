# native 宽标量 typed 入口与逐位结果通道（PLAN-891）

> **Status**: experimental（PLAN-891 交付；实现位于
> `experimental/ace/crates/native` typed_* 模块、
> `experimental/ace/crates/typed-entry-runtime` 与
> `crates/auto-lang-core` ac_mn/typed）

## ABI 身份（冻结）

- `msvc-x64/priv-typed-data-v1`：独立版本化 mn 私有 typed ABI，非 797
  公开 AutoABI，亦非旧 `msvc-x64/priv-i32-bool-v1`（旧模式行为不变）。
- 六标量以整数位载体过 ABI：I32/Bool/F32→I32（低 32 位，帧/存储宽 4B），
  I64/U64/F64→I64（8B）。逻辑类型在 MIR/verifier/签名/指纹中始终区分，
  不因载体相同混用。浮点全程不透明位型：无算术、无十进制 roundtrip、
  无 NaN 规范化、无 f32↔f64 加宽；sNaN/负零逐位保真。
- 纯值 record（字段闭包=六标量+嵌套纯 record，无 Str/borrow/owning 资源）
  内部按地址传递，callee 复制到自己的值槽；record 结果走尾随隐藏 out
  地址。record 不得出 typed entry（entry 签名仅六标量）。
- typed 符号域 `ac9.<owner-axis>.<unit>` 与旧 `ac8.` 域隔离；跨域/错配在
  link 前按签名逐项拒绝（`mn.admit.callee-sig`），绝不链接错函数。

## 通道（mn891.typed.frames/v1，冻结）

- 子 PE 专用 stdin/stdout 二进制帧，stderr 只承载有界诊断；一次性：
  恰一请求帧、恰一响应帧、无尾随字节。
- 请求帧（LE）：`[0..4)` "MNT1"、`[4..8)` version=1、`[8..12)` 签名 id、
  `[12..16)` 参数数、`[16..24)` nonce、`[24..28)` payload 长、`[28..32)`
  保留 0、参数字节（声明序、按宽对齐：4B 类 4 对齐，8B 类 8 对齐）、
  尾 8B fnv1a64 校验。帧上限 256KiB；参数 payload 预算 64KiB，超限在
  分配前具名拒绝（`mn.resource.typed-args`）。
- 响应帧 44B："MNR1"/version/签名 id/状态（0 value、1 trap、2 resource、
  3 protocol error）/nonce 回显/kind-or-reason/位型 u64/fnv1a64。
  value 与失败互斥；trap 具名原因（1 除零、2 除溢出、3 加溢出、4 减溢出、
  5 乘溢出、6 求负溢出），绝不以退出码 70 猜测。
- 参数值与 nonce 不进入用户 COFF/wrapper 语义键；wrapper 按签名成形、
  运行时从帧装载参数。Bool 参数仅 0/1；参数 kind/宽度/数目/签名不符在
  执行前拒绝。rawbits 永不经 JSON number（canonical hex 字符串）。
- 成功=进程 exit0 且恰一完整合法结果帧；缺帧/截断/尾随/旧 nonce/错
  校验/超限/异常退出均不 PASS。晚失败（成功帧后 hang/异常）整次非 PASS。

## 正式入口与判定

- recipe `ac/mir-native-typed-v1`（合同、dispatcher、worker、stale 四处
  同步注册）；fixture 为 `mn-typed-fixture/1` JSON 文件（语法
  `typed-fixture`/`.mnf`），声明式 units/records/body，经共享 `verify_mir`
  后才准入——builder 不做任何常量折叠或 oracle 推导。
- 期待值只存在 case 的 `expect.typed{kind,bits}`（父侧独立 oracle）；
  worker 请求只带 fixture 字节/入口/参数/预算/nonce。判定用与 AV 相同
  的 canonical hex 规则（i64/u64/f64 16 位、f32 8 位、i32/bool 无填充）。
- W01–W05（binding.tsv）经本通道真实交付：i64-min、u64-max、f64 负零、
  f64 NaN payload、f32 NaN payload 逐位与 AV 侧独立 oracle 一致。

## 缓存/恢复/增量（合同）

- 单元 COFF 键含 TYPED_ABI 轴（CoffMeta typed 元数据，恢复重校验信封/
  符号/长度/blake3/结构 COFF）；typed wrapper 为居民产物，按
  (abi, entry 符号, 签名 id, target, flags, transport, helper 库内容)
  缓存恢复，参数/nonce 永不入键。同签名换参：COFF 与 wrapper 生产为 0
  而 PE 实际再执行。
- typed 指纹：宽类型按逻辑标签（i64/u64/f32/f64）渲染、宽常量按精确
  位型（f32 8 hex、f64 16 hex、整数全宽）——同 bits 不同 kind、NaN
  payload、零符号均不碰撞；import 签名完整展开（callee 签名变更流入
  caller 键）；死表项/span/快照不入键。
- typed 居民会话：暖 body 编辑只重备/重验/重 emit 受影响单元（签名变更
  时含真实依赖者），whole-module fallback 计数恒 0；A→B→A 复用历史受检
  对象而非再 mint；原子多单元编辑承载 callee 签名变更与其 caller 更新。
- 冷恢复从 store 按指纹重准入（emit 0、wrapper 恢复），PE 实际运行。

## 范围与拒绝（非目标保留）

- 宽算术/比较/转换、宽字面量源码、全语言参数模式、Char/Str/record/Unit
  作 entry 值、wide 与 owned/collection 组合、native 闭包/Dyn、公开
  AutoABI/FFI、多平台 ABI、console profile 原生化：均不在本合同，准入
  处具名拒绝（codegen 0）。旧 scalar/owned/collections 通道行为不变。
