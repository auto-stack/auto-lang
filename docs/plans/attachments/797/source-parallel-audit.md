# PLAN-797 来源 / 并行边界审计（source-parallel-audit）

> 状态：T-07 可独立准备草稿。T-06 完成后复核依赖实际合入 SHA 与
> semref 退役收据。

## 1. 规范 / 设计来源

| 来源 | 角色 | 本计划用法 |
|---|---|---|
| docs/specs/overview.md | 项目导航 | 背景 |
| docs/specs/auto-ac/project.md | AC 模块 | SD-04 导航补丁 |
| docs/specs/auto-ac/data-resource-contract.md | 774 数据/资源 | SD-05 分域 |
| docs/specs/auto-ac/artifact-contract.md | 产物合同 | link/obj 语境 |
| docs/specs/auto-cache/project.md | 783 cache | ABI-L0/O1 层复用 |
| docs/specs/auto-test-case/project.md | TestCase/AutoDown | .case.ad 声明 |
| docs/design/strategy/auto-native-abi-rfc.md | ABI RFC | **目标**，非已发布 ABI |
| docs/design/41-interface-metadata-and-foreign-abi.md | Design41 | **目标** |
| docs/plans/797-… §5 SD 意向表 | 契约 | 本 spec-delta 草稿展开 |
| attachments/797/contract-decision.md | T-01 冻结件 | profile/工具/预算 |

## 2. 实现来源 / 复用边界

| 组件 | 来源 | 是否改写 |
|---|---|---|
| `find_rust_lld` / `find_sdk_um_dir` / staged link / `run_deadlined_split` | 已合入 ac-core | **否**（path 消费） |
| 783 ManagedStore | `experimental/ac-artifact-probe` | **否**（probe 自有层） |
| Cranelift 0.126 栈 | 与 ac-core 同版本约束 + Cargo.lock | 不改 ac-core |
| 774 ABI-A–E | `probes/data-resources` | **否** |
| 783/787 层注册 / GC | store 底层 | **否** |
| runtime/text、旧 774 ABI、crates 生产 FFI | 产品面 | **否** |

## 3. 并行合同（与 796 / 788）

| 依赖门 | 含义 | 当前 |
|---|---|---|
| D-796 | PLAN-796 独立 review/门禁/合入/Spec 齐全后，才接真实 semantic contract | **未满足**（796 executing） |
| D-788 | PLAN-788 合入后才消费 v2 worker/观察协议 | **未满足**（788 executing） |

并行允许：T-01–05 用有版本/hash 的只读语义 ref 快照
（`autoabi-semref-snapshot/0`，blake3 `15f81fd0…`）独立开展。

禁止：

- 在 796/788 合入前实现依赖其未合入 API 的 T-06；
- 复制其在途实现 / 第二套长期语义规范；
- 双方改同一共享协议还自称隔离完成；
- 用临时快照冒充正式 AutoDown 执行通过。

## 4. 临时模型退役义务

| 项 | 动作 | 时机 |
|---|---|---|
| `src/semref.rs` 快照 | 删除/替换为 796 真实 contract 解码校验 | T-06 |
| 退役收据 | 写入 spec-delta / freeze-proposal / Plan §9 | T-06 |
| 全部三腿 + 正反控 | 在真实通路重跑 | T-06 |
| 性能六场景 | 正式 adapter 复测 | T-06 |
| `.case.ad` | 正式 AutoDown 执行非 pending | T-06 |

## 5. 范围红线（复核）

- 允许：`experimental/auto-abi-probe/**`、`cside` 参考源、`scripts/verify-autoabi-797.ps1`、
  `docs/reports/plan797/**`、`docs/plans/attachments/797/**`、
  `test/cases/autoabi/**`、必要的 plan 簿记。
- 禁止：改 ac-core/native/source、runtime/text、774 ABI、783/787 store、
  crates 生产 FFI/诊断/包管理/生成器；安装工具；自动下载；操作旧仓。

## 6. 锁文件 / 依赖冻结（F-797-R1-03）

- `experimental/auto-abi-probe/Cargo.lock` 必须**在提交内**（force-add
  对抗根 `.gitignore:8`），不能只存在本机。
- 校验：`git ls-files` 可见 + blob == worktree `git hash-object` +
  `cargo metadata/check/test --locked --offline` 通过。
- 历史测试通过数在锁一致性核对前不得自动充作最终验收（R1 裁定）。

## 7. 候选标签声明

freeze proposal 只覆盖 **Windows x64 有限 profile**。v0.1-candidate
在 T-08/T-09 前不是现有产品支持声明，也不扩大 Auto 源码编译器能力。
