# AutoGraphics 跨端边界与依赖审计（XP-0 固定版本事实）

PLAN-878 交付的静态审计沉淀。对象为固定版本：AG `8b12632d7b5b1b0df72fbb4eef84297a054de31d`
（861 冻结）、旧 SDK 内容锁 pin `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`
（681 文件/17 本地包，plan-860 保留副本）、auto-ui `7095f536747e5180338a470a7d73452c6b550ed1`。
完整报告：[docs/reports/autographics-platform-audit-878/](../../reports/autographics-platform-audit-878/README.md)。
本页只记审计事实与边界，**不新增任何已实现平台能力**；不改 canonical auto-ui/AG
接口或现行 ACE/迁移合同。

## 固定版本静态分类（可共享 / 平台特有）

- 第一方 222 个 .rs 全文件分类：shared-candidate 22、adapter-needed 16、host-only 73、
  evidence-tool 106、其余 vm-harness/frozen-oracle/build-tool/structure 各 1-2。
- `autographics-prototype/src/core/**`（身份/场景/事件/时间/composition 语义）为
  shared-candidate：模块明文禁止依赖 wgpu/winit/Iced（`core/mod.rs:5`、
  `core/composition/mod.rs:21`）；core 只用 `std::time::Duration`，无墙钟读取。
- 渲染族（`render2d/render3d/renderer/assets` 16 文件）adapter-needed：wgpu 27
  耦合，`get_current_texture` 表面呈现与 `map_async` readback 假设未在非桌面验证。
- 宿主/UI/存储（73 文件）host-only：winit/iced/kira/线程/进程/文件系统；
  driver 作者存储非 Windows 为代码内声明的硬错误（`authoring/storage.rs:613`）。
- 依赖闭包（cargo metadata --locked --offline，10/10 target 解析 complete）：
  AGP 锁 478 包、driver 锁 787 包；wasm 可达 192/449。
- **SDK 宿主栈无 target 门控**：auto-lang-core 的 axum/tokio/reqwest/notify/
  libloading/stacker/sysinfo/rustyline 依赖边在包括 wasm32-unknown-unknown 的
  全部 5 个 target 解析图中存在；跨端可编译性未测（XP-1 前置）。auto-val 仅
  7 个轻量直接依赖，shared-candidate。SDK 17 本地包中 15 个在 driver 闭包内，
  auto-test-case 与 vendor/autodown-test-parser 仅测试用。
- 旧 SDK pin 对象在 auto-lang 当前仓**不可达**，仅 plan-860 保留副本（681 文件
  SHA-256 锁）可验证消费；任何迁仓/换副本须先保全该副本或恢复对象可达性。

## 已知限制与非支持声明

- Windows x64 单窗是唯一已验证运行面；本审计不构成任何浏览器/移动/其他桌面
  的可编译或可运行证据。
- 未测假设显式清单（U-01..U-08 / H-01..H-05）：SDK 栈跨端编译性、wasm 的
  `Instant`/64 位原子可用性、wgpu Web 后端对本管线兼容性、winit 移动生命周期、
  kira 移动音频会话、driver 测试面非桌面不可用等——全部见
  [dependency-map §7](../../reports/autographics-platform-audit-878/dependency-map.md)
  与 [platform-seams §3](../../reports/autographics-platform-audit-878/platform-seams.md)。
- u64 身份（代次/会话/序列）与 f32 数值是跨运行时桥接的精确编码面（`core/id.rs`、
  `output.rs` Packet）；转 JS Number 不得无检查截断（>2^53）。
- device 失效"保逻辑状态/资源身份、不承诺像素"与挂起/恢复墙钟债务规则在代码中
  无实现，属后继 adapter 合同范围；原生故障保帧合同不跨端外推。

## XP 前置与后继门

- XP-1 最小图形双端的入口/oracle/规模建议与阻塞清单（readiness=limited）见
  [next-probes](../../reports/autographics-platform-audit-878/next-probes.md)；
  XP-2/3/4 仅研究接口。XP 后继各需独立 Plan，不因本审计自动启动。
- 纯源码归仓（迁入 auto-ui `experimental/`，保持嵌套独立 workspace/锁/旧 SDK）
  的静态可行性成立：auto-ui 默认 `crates/*` 不受影响（AG 嵌套不进 members、
  7095f53 处 crates 零 autographics 引用、SDK 消费路径活/冻不相交，无重环）。
  迁移触面清单与门分离见
  [migration-readiness](../../reports/autographics-platform-audit-878/migration-readiness.md)；
  纯归仓/声明融合/ACE 替换/切默认退役四门相互独立，均不因本审计放行。

Provenance: [PLAN-878](../../plans/878-autographics-platform-boundary-and-dependency-audit.md)，
审计合同 `docs/plans/attachments/878/work/audit-contract.md`。
