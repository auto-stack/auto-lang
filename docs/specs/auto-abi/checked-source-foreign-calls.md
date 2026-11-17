# auto-abi 受检有限 foreign 标量调用（PLAN-938，CA-D2）

> Status: reviewed（PLAN-938 r1，冻结提交链至 75330f456；T-12 复审 AC-01..08 pass，
> 冻结固定职责并集门 13 组 gate_passed=True/2328 项 0 not-ok）。

普通 Auto 源（`auto.source.scalar-i32.draft` profile，仅 i32/bool 标量）经**显式受检
typed 声明**调用独立 C provider 的有限合同；不是任意 C 头 importer，也不扩公共 ABI。

## 冻结身份

- manifest schema `ace.foreign.manual/1`（独立 typed 输入，非 Auto 语法）；声明环境页
  `core-decl-env/2`（含 foreign；无 foreign 仍 `/1`）；source facts 会话
  `ac-source-facts-session/1`；target 快照 `abi/source-foreign-session/1`。
- 共享 MIR 指令 `FOREIGN_CALL`（opcode 0x36，traps，profile `foreign-native`）；
  候选 wire AVFI v1/profile 9、schema `auto.abc.foreign-scalar/1`；ABC 页
  `MAX_CODEC_BYTES` 有界编解码。Foreign 声明在 `MirInstr` 内 boxed（≤64B inline 门）。
- C 侧 fileset `auto.a2x.c.foreign-library.fileset/1`、recipe
  `ac-a2c/hir-foreign-scalar-import-v1`；正式认证 recipe
  `abi/source-foreign-certify-v1`（deck `abi.source-import-deck/1`、report
  `abi.source-import-report/2`；`/1` 与旧 ABI/C/Rust 报告互拒）。
- 人工审定契约/头/物理表按 BLAKE3 身份 pin（SDK 侧与父侧独立 pin 精确 ADD/Scalar
  值）；普通 source SHA256 沿原算法。

## 合同要点（与实现同审）

- 声明准入：当前宿主 HostAuthority（review/contract/header/table/provider/target/
  caps/symbol）逐项资格化；unknown ownership/effects、缺显式 `foreign.scalar` cap、
  伪 CheckedSource、陈旧 semref 具名拒；持久 safe 标志不授信。
- native 经唯一共享 HIR→MIR 直接机器调用（不经 C 转译）；A2C 从受检 HIR 发薄
  call；AV 准入 `foreign.unsupported`、Rust `a2r.foreign-unsupported` 具名拒，无
  fallback。每次实际 call 记 callee counter/trace（`source938_events.c` 事件契约）。
- required 页：logical/physical/declarations/binding 四内容页 + source/provider/
  runtime 页；恢复只做有界深验（不重投影/重编码），缺页/坏页/错 owner/伪 safe 拒；
  发布为 pointer 锁内 leased put→原子替换→旧根退休，故障/取消/迟到保旧根。
- 资源先准入（T-09 逐 owner）：codec/ABC/MIR/Verifier/emit/solver/接口工作集/
  会话元数据在分配前 reserve；输出账户单产物 64MiB/请求累计 128MiB；store 整
  namespace 128MiB 于排他锁内自读数横跨 metadata commit。
- SDK/MSVC 闭包按进程解析一次并复用（override 键控；identity/fingerprint 材料不
  变）；正式暖门 body-edit P90≤2000ms 为常驻门。

## 拒绝面（节选）

分配后才准入/漏计页/绕过页账户/late-JSON admission 等变异均为永久反控捕获
（各单元负控日志见 PLAN-938 §9）；本文件只记合同，不复制测试清单。

## M2-A/M2-B 分阶段 AC 合同与 B 移交边界（PLAN-1064 SD-AC-STAGED，2026-11-17）

- **阶段入口定位**：M2-A 阶段要求同平台（Windows）下 `AV`、`A2R` 具备完整公共标准库能力，`A2PY` 保既有标准库/AV 三 provider 义务，`A2C` 完整对齐列为增强项并保留既有代码/用例/支持事实；`AC`（`ace-native`）在 M2-A 交付真实可用的基础能力，不得以 stub 冒支持、不得用 `AV`/`A2R` 代签 `AC`。
- **M2-A 已交付 AC 基础（`PLAN-1065` + `PLAN-1064`）**：`packages-aggregates` 纯 Auto `std.bytes` 编解码在 `MnDriver::build_collections_checked` 真编译运行；Windows `LowerMode::Console` 下 `50` 项 `sys973` OS 原语（`36` Win32 kernel32/bcrypt 真身含对齐后的 `canonical_w16` 缺失 `-8` / `env_get_w16` `SetLastError(0)` 空串 `0` vs 未设 `-8`，及 `14` `-10` 字节形拒绝）经 PE 导入审计闭合。
- **具名 B 后延 AC 差额**：① `packages-aggregates` 下 `std.{file,path,env,process,storage,zip}`（`D-01..D-09`）与 `mn_pio_*` `PlatformCall` 合流（`native/src/lower.rs` 目前对非 `Console` 路径拒绝 `PlatformCall`）；② `ResKind::OutBytes` 在 `AC` native 机器侧调用与帧守卫——均具名移交 [PLAN-1068](../../plans/1068-ace-m2b-three-backend-parity-and-aac-proof.md) (`T-03`)，异步调度交 [PLAN-1067](../../plans/1067-ace-ac-native-async-and-task-parity.md)，Linux 目标实跑统一交 [PLAN-1063](../../plans/1063-ace-stdlib-linux-deferred-validation.md) (`NOT_RUN`，不得以 Windows 证据升 Linux verified)。M2 最终出口（M2-B）时 `AV`/`AC`/`A2R` 差额须收口为 `0`。
