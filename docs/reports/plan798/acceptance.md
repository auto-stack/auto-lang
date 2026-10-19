# PLAN-798 验收/覆盖/并行审计（T-08，work-owned 收口）

2026-10-19。代码基线：`plan-798-dev`（本 worktree，T-06..T-08 修复+续作后）；
依赖：master `13367d127`（796 已归档合入，交付 b76eca8d2；788 已归档合入）。
本文件是 work 自检产物，最终验收归 review；canonical Spec 沉淀归 merge。

## V-01..V-13 覆盖矩阵（实际入口）

| V | 断言族 | 实际入口（测试/脚本） | 状态 |
|---|---|---|---|
| V-01 | 私有阶段信任 | mir_tier 7 + abc_semantics（only-verify 句柄/篡改内层必拒） | 绿 |
| V-02 | 类型/初始化/CFG | mir_admission 变异矩阵+哈希证明+半初始化合流+循环不外溢 | 绿 |
| V-03 | 数值/短路 trap | av_semantics trap 全矩阵+未走 RHS 潜伏+native 对拍 | 绿 |
| V-04 | 参数/控制/递归 | p-binop-args/p-gcd-euclid 降级+fib(15)+帧预算两侧 | 绿 |
| V-05 | wire/恢复 | abc_codec 截断/尾随/校验和/未知版本-opcode+envelope 拒绝 | 绿 |
| V-06 | 真缓存 | cache_tier 命中跳产仍执行+跨进程 worker 恢复 | 绿 |
| V-07 | 依赖/配置 | body-only 单元重键/签名变更全 caller 重键/跨工程共享 | 绿 |
| V-08 | 错身份/并发/回收 | envelope 错 producer/schema/stage+Conflict 保旧+lease/GC | 绿 |
| V-09 | 规模/内存 | 4096 链+33.5MiB 冷产物+512 真实 churn+分区反控三件 | 绿 |
| V-10 | 有界执行 | fuel/取消/1024 帧两侧/trace 预算 | 绿 |
| V-11 | 亚秒 | 六场景 30 样本（Rust 定向）+ **正式 AutoDown warm-edit 门**（T-07） | 绿 |
| V-12 | AutoDown 真实执行 | corpus 6 checks+负 oracle+decode 负例，统一 runner | 绿 |
| V-13 | 绑定/边界 | binding_tier 4（同名异 owner/重启稳定/schema/tamper） | 绿 |

## 正式 AutoDown（T-07 交付面）

- recipe `ac-av/core-run-v1` 注册（REGISTERED_PROBE_RECIPES；配方专属步骤词表
  run/edit-source、专属 `av` 绑定、probe 通道 exit 观察、run 完成态）。
- 共享侧 `crates/auto-lang/src/test_cases/ac_av.rs`：Job 收容+resume、
  envelope/nonce/request_hash/recipe/双版本镜像准入、stale 按配方族路由
  （av-worker 四轴）、父侧判定、probe-receipt.json。
- worker：`case-serve`（会话步骤+持续 store+逐步墙钟）与 `load-revision`
  （冻结负载单一来源）；从不读 expected。
- 用例：`crates/auto-lang/test/cases/ac-av/`（suite+5 case 文件 6 checks，
  成功/trap/诊断/能力门/资源负例全 oracle 形态）。
- 兼容：probe 接缝 17 腿绿（decode 负例/v1 pilot 原样/atom 准入/stale 分族/
  收容/类型负例）；contract_v2 20 测绿。**sema corpus 腿阻断在案**：其
  bindings oracle JSON 未被 master 跟踪（根 `.gitignore` `*.json`），Plan 813
  分支 `6643ceaeb` 修复在途未合入——非本计划 diff 引入，登记待 813 合入后重放。

## SD 提案（merge 沉淀输入；review 复核后定稿）

| SD | 目标 | 要点 |
|---|---|---|
| SD-01 | auto-mir/project+core-i32 | 块/槽/CFG/初始化合同+CheckedMir 私有凭证+工件族预算（r2） |
| SD-02 | auto-av/project+abc-core-i32 | wire v1 布局/验证次序/LoadedModule/unsupported 面 |
| SD-03 | auto-av/cache-and-conformance | A08/A09 producer 身份、单元/manifest/sidecar 分轨、恢复重验、计数语义 |
| SD-04 | auto-ac/artifact-contract | A08/A09 的 798 实现范围追加（carrier=LayoutData） |
| SD-05 | auto-hir/stage-contract | HIR→新 MIR 消费前门+128 前门递归风险表 |
| SD-06 | auto-test-case/architecture | ac-av/core-run-v1 recipe 支持表（步骤词表/绑定/exit 通道/run 完成态/stale 分族） |
| SD-07 | auto-av/cache-and-conformance | AC/AV/AJ 合同边界与 native 产物引用准入（T-08 有限代际模型） |

touched_goals=[] 说明：798 全程为 experimental/ 探针+测试接缝扩展，不改产品
目标行为；目标级 goal 映射由 review/merge 判定。

## 并行审计

- 写面：`experimental/ac-av-probe/**`、`crates/auto-lang/test/cases/ac-av/**`、
  `scripts/verify-ac-av-798.ps1`、`docs/reports/plan798/**`、
  `test/cases/ac-av→crates/auto-lang/test/cases/ac-av` 迁移（git mv）。
- 共享接缝最小扩展（按 796 接缝预留的追加点）：contract.rs（注册表+ProbeStep::Run
  +ProbeAvSpec+av 已知字段+exit/run-stage probe 放开）、suite.rs（probe_av 字段
  透传）、mod.rs（按配方分派）、sema.rs（containment pub(crate)+防御 arm）、
  stale.rs（配方族路由）。未改 decoder 协议/共享 IPC。
- 依赖只读：ac-core/ac-artifact-probe/ac-sema-contract 均未修改。
- 组内 auto-down 兄弟树：`D:/autostack/.wt/lang-798/auto-down`（detached @
  master c39ac7f，根 workspace 路径依赖所需，无本计划改动）。
- 交叉：Plan 813（未合入）拥有 oracle JSON 跟踪修复；本计划不改其文件。

## merge-owned pending

冻结提交统一门禁（根档按最终 diff）、native/796/788 实际触面兼容整档、tf 到期
批量、Spec/ledger 沉淀与归档清理——最终 review pass 后由 merge 执行。
