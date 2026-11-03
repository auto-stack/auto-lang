# Native 有限闭包源码与资源收口（PLAN-924）

> 状态：experimental（真实 native COFF/PE 上的有限闭包源码域；不改 AV
> wire/指令与共享前端合同，不冒全闭包语言、public ABI/FFI 或 native
> loader/HMR）

## 有限源码/typed 支持域（§5.1 冻结）

- **捕获**：i32/bool 按值 Copy（不隐式 clone）；显式 `move` 的 Str 与
  非递归 owned record（字段域 i32/bool/Str）；typed-only 深 clone
  （CaptureKind::Clone，源码无合法形式的 clone capture 明示 typed，不伪称
  新源语法）。**native admission 具名拒绝** Borrow / MoveFromEnv / Shared
  与捕获 List/Variant/Closure/Dyn/宽位型；共享前端与 AV 合同不因 native
  有限域退化。
- **lambda**：i32 参数/i32 结果；普通 factory 接受 move 资源参数并返回
  Closure；程序 entry 是零参 i32 普通函数，闭包 body 不能 entry，直接
  Call lambda body 在 verify 与 native admission 双拒。
- 有限域指令面：Const/Copy/Move/Clone/View/BorrowEnd/Drop/StrLen/Bin/Un/
  Call(Import)/Aggregate/FieldGet + CreateClosure/CallClosure/EnvGet；
  集合/Place/Dynamic 指令与容器类型在本 native 家族具名 unsupported。

## native 家族身份（§5.1）

- recipe `ac/mir-native-closures-resource-v1`；cell ABI
  `mn.closures.cell/1`；layout `mn.closures.layout/1`；codegen
  `mn.closures.codegen/1`；source profile `mn.closures.source/1`——
  全部独立于 collections/typed/scalar 家族，不向既有五 recipe 静默猜测或
  回退。
- 单元身份 = spans-cleared ABC wire bytes（与 AV closures 会话同一 wire
  纪律）；scalar/collections fingerprint 渲染面不放宽。
- runtime 库身份 `mn.collections.runtime/4`（嵌入闭包 env 面，S9 seam）；
  collections cell/3 与 923 seam S1..S8 计数不变。

## 执行合同（§5.2）

- 闭包值是 env 节点 handle；机器 code pointer 来自本次已验证/链接 image
  的重定位绑定（持久工件只存 symbol/descriptor/deps，不存宿主地址）。
- 闭包 body 机器签名 `(env: i64, params: *Cell.., out: *Cell)`；调用方
  `mn_coll_closure_target(bits, sig_id)` 前置 registry+签名准入（foreign/
  stale/错签名拒，被调计数 0）后由 CLIF `call_indirect` 真间接调用——
  工厂常量折叠成 42 或把 runtime 做指令解释器都是红。
- 同签名不同 body/env 的实例给不同真实结果（双目标控制）。

## env 事务、clone、pin 与清理（§5.3）

- CreateClosure 两段式：`mn_coll_env_prepare`（S9 单块 staging）→ 逐
  capture `stage_move`（retain 共担；commit 后由生成代码对 Move 源
  `mn_coll_drop` 杀责——未提交失败源/责任/代际不变）或 `stage_clone`
  （深拷）→ 失败 `mn_coll_env_discard` 逆序无分配回滚 → 成功 commit
  发布。
- EnvGet 是 handle share（env 保有责任）；Closure 深复制 env 与 owned
  backing（`mn_coll_clone` CLOSURE 臂两段式），原/克隆互不共享析构权。
- body prologue pin / 返回 unpin；trap 由 abort 帧扫+parked env 终结，
  清理不变量 live=0/bytes=0/invalid=0 保持；creator frame 退出不杀已返回
  closure 的 env（R05 核心）。

## 函数生产/恢复/资源（§5.5 摘要，详见 mir/native-closures-resource-abi）

- 暖生产复用 912 单函数前端与凭证；native 会话身份/依赖边/恢复按
  collections incremental 模式（详见 auto-mir 规格）。预算沿用 8/32/48/
  16-24-8/128MiB、4MiB 源、1MiB 流、8192 行 MN-EVT、4096/64KiB 栈域。

## 明确不做

任意闭包语言、Shared cell/K06 写回、容器/闭包捕获、借用逃逸、外部
callback、public ABI（797）、native loader/HMR/OSR、Unit/脚本入口、
A2R/A2C 闭包；不自动改旧 M1 K01..K06 NA/其他分母。
