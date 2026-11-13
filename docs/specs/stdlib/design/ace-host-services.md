# ACE 标准库宿主服务合同（console/1 与有界资源前探）

> 状态：PLAN-947 交付稿（worktree 预备；merge 时正式沉淀）。AV/native 的
> implemented 状态以 `crates/auto-lang/test/cases/ace-m2/p947-console/` 的
> 双 lane 真实执行为证据；Rust/C 与资源端口缺口按"capability-missing 单列"
> 记录，不冒领、不静默扩权。

## 1. console/1 宿主合同（冻结合同）

- **类别与符号**：`console/1`；`auto.console.print.{i32,bool,str,char}.v1`（stdout）、
  `auto.console.eprint.*`（stderr）、公共拼写 `say(str)` ≡ `auto.console.print.str.v1`
  （stdout；旧 `io.say` 冻结证据链：`io.at:21` 声明 → `io.vm.at:51` `say→print` →
  VM codegen `("io","say")→auto.print_str` → `vm_print` 在字符串后恰追加一个 `\n`）。
- **格式化**（guest 侧，AV 解释器与 mn 机器码逐字节一致）：i32 十进制 ASCII 无正号/
  前导零（MIN 保留负号）；bool `true`/`false`；char 按 Unicode 标量 UTF-8（≤4 字节）；
  str 原字节（内嵌 NUL/换行保留）；每调用恰追加一个 `0a`。
- **预算**：单调用（含追加 LF）≤ `MAX_CALL_BYTES=65536`；每 run 累计 ≤
  `MAX_RUN_BYTES=1048576`；每模块 ≤8 个 console overload 声明；两次取消检查点之间
  原子提交（不半提交）；`ProviderScope` 无 guest 重入。
- **授权**：每 run 注入 `console.stdout`/`console.stderr` caps，先于任何 dispatch
  （`av-core host::authorize` 共享实现）；恢复产物只携带身份事实（owner/entry/
  snapshot/指纹），永不携带授权、provider 地址或凭证；重新授权即可执行。

## 2. 四端支持分账（console 公共输出）

| 端 | 状态 | 证据/边界 | 后继 |
|---|---|---|---|
| AV | implemented | `ac-av/console-run-v1`；语料 p947 双流字节精确 + Freshness | — |
| native (AC/mn) | implemented | `ac/mn/console-run-v1`：共享前门→persist/restore 重接纳→每轮授权→逐函数 Console 发射（guest 格式化+per-call 预算）→shim（累计预算+kernel32 WriteFile 每调用原子）→真实 PE 运行。**Unit-result 入口=命名能力边界**（mn 冻结零参 i32 入口 ABI），AV lane 承接 Unit 通道（corpus O07/O09） | Unit 入口随 mn 入口 ABI 演进单列裁定 |
| A2R (Rust host) | capability-missing | 无 Rust host console provider；host-console profile 不入 Rust 发射面 | PLAN-973（四端补齐） |
| A2C (C host) | capability-missing | 无 C host console provider | PLAN-973 |

机器账：`stdlib/contracts/compatibility.toml` `console.say` 条目
（ace_av/ace_native=supported、hir_rust/hir_c=missing_capability、owner=PLAN-947），
由 `stdlib_assembly::contracts` 审计词汇表与 `contracts/tests.rs` 钉扎测试双锁定。

## 3. 有界资源前探端口（File/Task/Stream；本包仅前探，不交付服务）

依据 939 foreign-resource 契约（`Handle{provider}`/`Callback{ctx}`/`ViewText` 等
形态，`RESOURCE_CAP="foreign.resource"`），本包只为 File/Task/Stream 冻结**有限状态
端口**与分账格式：每项记录 实际 producer/consumer、状态集合（创建/失败/borrow/
move/close/cancel/退出/恢复）、依赖事实（939 是否已交）、缺口归属（967 异步端口/
968 调度/973 系统服务）。未实现状态必须输出 `capability-missing`，绝不冒充 PASS。
详细逐项前探结论见 PLAN-947 §9 T-04 记录（merge 时折叠入本节）。

## 4. 默认自有实现与 FFI 边界（AC-NATIVE 分账基线）

默认 core/alloc/基础系统库由 Auto 自有实现与薄 OS 适配承担；不以 libc 或 Rust std
间接委托为默认；公共 AutoABI 与 OS/外部 ABI 分开建模，Rust/C/provider 依赖显式
选择。缺默认 provider 或自动回落 libc 必须拒绝；合法自有 provider 可装配。
Storage/allocator 接口保持 M2 内存专题待定（本包不冻结签名/ABI 布局）。

## 来源

- 冻结合同与证据：[PLAN-947](../../../plans/947-ace-stdlib-console-and-resource-host-contract.md)（§5 T-01 冻结记录、§9 验证记录）
- console/1 合同规范：[auto-ac/source-console-output.md](../../auto-ac/source-console-output.md)、[auto-av/host-import-and-console.md](../../auto-av/host-import-and-console.md)
- 939 资源形态：`experimental/ace/crates/foreign-contract/src/resource.rs`
