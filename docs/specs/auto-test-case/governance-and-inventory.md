# 测试治理：完整盘点、可信裁判与守恒（governance-and-inventory）

> PLAN-896更新：日常入口不再消费逐叶目录/分片/自动影响图；直接执行固定Cargo包和Rust模块组。
> 旧全量台账保迁移/周期审计；实际nextest/Python/TestCase终态和原包装check/oracle仍核对。
> 原模块划分守恒不再要求每次执行重建；详见module-selection.md。

> 状态：delivered（PLAN-887 交付；本规范描述已交付事实，`scripts/test-governance/` 即受管工具）。
> 位置：`scripts/test-governance/{inventory.py,check_inventory.py,measure-first-wave.py,coverage-matrix.json}`、
> `scripts/run-full-regression.py`、`scripts/tier-selection-minimums.json.txt`。

## 盘点闭包与子 case 身份

- 闭包：四仓（language/Web/UI/Man，PLAN-890扩展）当前受管第一方测试 + `crates/auto-lang/test/cases/*`
  全部 suite + ACE 独立 workspace（`experimental/ac-*`，实测 10 仓 202 测试文件/11 bins）实际
  consumer 入口。vendor/第三方、SDK 冻结副本、生成物、缓存、本地运行证据排除或引用。
- 子 case 身份：`crate\tpath::tests::name`（tab 分隔，与 runner 基线文件一致）；聚合包装
  （`vm_file_tests`/`a2r_tests`）不冒充子 case 绿——`check_inventory` 常驻 masquerade 控制。
- 动态枚举 = 权威树 `nextest list`（frozen combo：`-p auto-lang -p auto-lang-core --lib
  --config-file .config/nextest-full.toml --test schema_drift --test docs_gen --test
  component_registry_test`）对账静态基线；不能枚举者记具名 UNRESOLVED 阻塞守恒（本 Plan 关闭时
  unresolved=0）。

## 迁移/退役与守恒

- 退役通道：`full-regression-retirements-{844,869}.json.txt`（证据引用，逐 ID evidence 必填）；
  runner 必须加载双清单（缺一即与冻结基线脱节）。迁移：`full-regression-migrations-841.json.txt`
  （language→language 重命名、language→ui-full 移出）。
- 守恒校验（`check_inventory.py`）：ghost 退役/缺失必需 ID/零选择/抑制 required-selected/退役
  ID 仍被实际列出/跨清单重复/无来源迁移 → 非零退出；合法对账 → conserved。
- floor 是逐 ID 对账后的派生守卫：language 3189 = 3205 − 869 退役 16（理由落档于
  tier-selection-minimums.json.txt）；缺 ID/零选择/伪完整必红，不旁路、不为接受删除再生成 baseline。

## 成本口径与性能验证

- 定向性能入口 `measure-first-wave.py`：warm、单 out 目录、每侧 ≥7 有效全对样本；P50/P90 =
  升序 ceil(0.50N)/ceil(0.90N)。
- 已验证失效轴（PLAN-887 T-05 负结果，机制级证据）：(1) worker `case-serve` 对无 session 请求
  强制 per-request 全新 store（nonce 子目录 + 处理前删除），**跨请求内容缓存不可能命中**；
  (2) worker 会话复用属 `experimental/ac-*`（S/N 域）且 `session-serve` 协议与 avworker/1
  case-serve 不兼容——协议语义禁变。任何新性能计划须先核这两轴，不得重复该负候选。

## 覆盖矩阵

`coverage-matrix.json`（schema test-governance-coverage-matrix/1）：能力×用例类×后端/profile×
入口形态×生命周期轴；首批义务全部强 oracle + 真实起点 + 执行状态；未覆盖格具名 owner/原因
（B17-B19 pending 语义裁定、NA-4、D 生命周期扩展、UI/Man 域）。弱判差额不批改 golden、不复制
裁判；oracle 永不追随旧 VM（B08/B11 差异行保留）。

## 全仓审查对账（PLAN-895）

888 受管闭包是全仓分母的起点而非终点。PLAN-895 交付全仓来源台账（test-purpose-policy.json `sources`）：
受管 recipe、未注册 Rust 单测/集成测试、`.at` 语料与金样的包装输入轴、Python/PowerShell/JS 验证入口、
`test/parity` 与 modularity fixture 门禁、已迁外仓的 AG 线与 UI/Man owner 引用分轴登记；每个排除目录记
依据，不按 experimental/probe 一概排除。未注册声明、包装入口、外仓引用与受管运行义务分开记，遗漏/重复
由审查校验器（audit_test_purposes.py）与逐义务报告守卫；未注册入口的注册属后续独立决定。frozen nextest
combo、退役双清单与 floor 3189 等 887 冻结条款不受本节影响。

## PLAN-888 唯一叶与真实消费

旧九档身份冻结到不相交叶模块，ACE十workspace/正式suite/Python控制纳入本仓目录；外仓owner保留引用。分片内容哈希、实际Cargo及runtime/helper消费者图、原source/oracle/条件/输入守恒共同校验。运行时再次核对选择和实际原子完成记录，聚合包装成功不能替代内部比较；条件不适用和原始失败不冒绿。计划门与周期清单、成本和慢模块遵循[模块选测](module-selection.md)，B/C试点依据[批量合同](batch-execution.md)。887历史收据与原M1分母保持。

## PLAN-890 四仓实际执行守恒

Web库/HTTP/生成各有独立recipe，原Web/媒体迁移ID、断言、条件及17资源由具名清单守恒。
完整tm并集的source/catalog/selection/命令/逐ID实际结果及内部case/child oracle必须相符；原ignore/manual/milestone不算通过。
producer与公开收据consumer共用原baseline/retirement/migration/floor，内部一致的删项也须拒绝；
迁移credit只能来自已验证实际PASS目标。ui-tu仍单进程执行，以同批ui-full来源flags还原libtest省略的原ignore。
缺执行、错命令/输出/hash、重复/零选择/无依据排除、未执行腿或旧SDK认证新来源均失败。
用途分类只作建议。选择/执行/周期和当前组合发行的门分列，详见[四仓合同](../repository-boundaries/web-four-repository-delivery.md)。
