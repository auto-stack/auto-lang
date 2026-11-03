# Native 资源场景覆盖与 R02/R03/R05 分层（PLAN-923）

> **Status**: experimental（R03 ac 由真实 native 父证据关闭；R02 非闭包
> 覆盖摘要与 924 唯一最终 owner；不把部分证据勾成 R02 PASS）

## 覆盖分层

| 场景格 | 923 交付 | 最终 owner |
|---|---|---|
| R03（活借用/旧 buffer） | **ac 关闭**：case-ace-m1-native-resource-923（source PE 逐 seam 注入）+ collections_place_tier（受检 ListPlace 机器链 V-04/05） | 923 |
| R02（构造/clone 失败注入） | 非闭包覆盖：V-01/02/03 已交付（source PE 逐点 sweep + 同 runtime probe 快照/重试 + 原子清理） | **924**（K02/env 故障 + 完整闭包） |
| R05（closure 创建者退出） | 不覆盖（原生闭包/924 域） | 924 |

binding.tsv 只回填 R03 的 mn 格（`ok:P923-guards`）；R02 的 mn 格保持
`gap:ac-composite-native` 并在注释记录 923 已验非闭包覆盖，R05 保持原 gap。
原 source/expected/NA/accepted 与 corpus R02/R03 行不改字节。

## 正式父准入

- recipe `ac/mir-native-resource-guards-v1`；父侧请求只带
  操作/seam/index/预算（`mn.resource { fail_seam: [seam, nth] | fail_text_nth |
  fail_ledger_nth | budget_bytes }`），不带期望 N/结果/阶段/事件。
- 合法 N/结果由独立 reference/冻结父表判（ac_mn/resource_guards.rs：
  seam/nth 成对严格解码、trap 映射、zero-live/zero-bytes 清理）。
  坏来源/工具/产物/词表/重复 key/旧 operation/截断/重放全非绿。
- AutoDown 资产：`crates/auto-lang/test/cases/ace-m1/native-resource-923/`
  （suite + case 8 checks + src 副本），source-manifest.json 登记
  `scenario_assets_923`。

## 924 输入清单（单向交接，无循环依赖）

924 消费本包正式 stable SHA（merge 后），补 K02/env 故障，随后重新核对
本包在 924 runtime 版本下的全部非闭包覆盖，由 924 唯一回填完整 R02 与
R05。923 交付不等待 924；本包只关 R03，不存在循环依赖。

- 923 交付接口：`mn.collections.runtime/3`、`mn.collections.cell/3`、
  `mn_coll_fail_nth_seam`、`mn_coll_place_mint/get/set`、
  `restore_collections_session`、`ac/mir-native-resource-guards-v1`。
- R02 欠项（K02/env）：native 闭包环境/K02 源场景、env 维度故障注入。
- 观测：`MN-COLL-INJECT seam=/nth=`、`MN-COLL-TRAP code=`（21..=27/30/31）、
  `MN-COLL live=/bytes=/invalid=/arena_live=` 与 MN-EVT（919）并存。

## PLAN-924：完整 R02 与 R05 的最终闭合

- **闭包域**（924 独有）：suite `case-ace-m1-native-closures-924`
  （`crates/auto-lang/test/cases/ace-m1/native-closures-924/`，source-manifest
  `scenario_assets_924`）；recipe `ac/mir-native-closures-resource-v1`，
  父表 `ac_mn/native_closures.rs`（S1..S9 分类学 + trap/清理 + R05 env
  生命周期事件形状从严判）。K02（S9 单块 1..2/N+1、S1、S4）与 K05（S9
  两块 1..2/3）全腿；runtime 侧另有同进程原语状态探针（staged 事务回滚/
  同进程重试/准入矩阵/深 clone 独立/pin-park-finalize）与
  trap-in-pinned-call 全零基线。
- **非闭包域复验**：923 套件（R03/`native-resource-923`）与 ace-native
  全组在 924 最终库（runtime/4，含 abort 空闲元数据修复）下重跑 PASS——
  完整 R02 = 本节闭包域 + 923 非闭包域在此同一最终库版本下的并集，不拼
  历史版本 PASS。
- **回填与守恒**：binding.tsv 仅 R02/R05 scenario **ac** 格
  `gap:ac-composite-native → ok:P924-closures`；corpus 行/source/expected/
  313 required/45 future gaps/32 ruled-accepted 不变；M1 claim 守卫显式
  引用 cert 轴（G-DEC-5 "with valid evidence" 落码），本轮交付不签新
  M1/M2/v0.6。K06/shared-cell、容器捕获、loader/HMR 等余项仍归后续包。
