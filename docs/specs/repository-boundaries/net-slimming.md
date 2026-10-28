# net-slimming（净瘦身口径与收据）

> **Status**: active（PLAN-844 SD-05）
> 范围：三仓（auto-lang / auto-ui / auto-man）净瘦身的统一度量口径、冻结基线与收据

## 目的

R12 减法半部（PLAN-844）要求"用统一口径证明生产净删除和重复清偿，而非把移仓/格式/
生成物删除算瘦身"。本页登记该口径的规范性定义；逐次度量收据见对应 plan 的
attachments 与 §9 记录。

## 统一口径（PLAN-844 T-01 冻结）

度量脚本 `docs/plans/attachments/844/probe/measure.py`（同脚本前后自比）：

| 桶 | 定义 |
|---|---|
| P | `crates/*/src/**.rs` 手写生产源（扣测试文件白名单与生成标记） |
| T2 | 文件化 cfg(test) 测试模块（白名单锚定声明处，如 `auto-lang/src/tests/`） |
| G | 首 12 行含 DO-NOT-EDIT 标记的生成 .rs |
| XT | 根 `test|tests/` 与 `crates/*/test|tests/`（语料/集成测试） |
| VEND | `vendor/` 与 `crates/patches/`（第三方及其补丁） |
| XO | 其余 .rs（examples / experimental / website） |
| AT-* | .at 文件：P=生产（`stdlib/`、aavm lib）/ L=lib-legacy 活跃载荷 / T=测试 / O=其余 |

- 行分类：prod / comment / blank / **inline test**（cfg(test) 属性起括号深度追踪；已知
  局限：`#[tokio::test]` 型不计 inline test——文件级前后自比不受影响）。
- **主指标 NET-PROD = P total − P inline test**。生产净删 = 前后 NET-PROD 差额。
- 分列规则：跨仓搬同一文件 = 0 净删；新增 helper 计入 P（净额自抵）；测试/生成/第三方/
  资产各自分列不得计入主指标；把生产改标 generated = 挪分母，禁止。
- 重复清偿独立申报：真重复族（同语义双实现收敛单源）逐族记 owner 与证据，不与
  行数混同（小体量族也可能是必要清偿）。

## 冻结基线（2026-10-28）

| 仓 | commit | NET-PROD | 备注 |
|---|---|---|---|
| auto-lang | `bb85bf304` | 308,909 | P 555 文件/360,282 行（含 inline test 51,373） |
| auto-ui | `7095f536` | 296,190 | P 369 文件/410,675 行（inline test 114,485——runtime 内联测试占比高） |
| auto-man | `4cc912e9` | 24,530 | P 83 文件/30,664 行 |

完整桶表与 top-20 文件：`attachments/844/probe/baseline-stats.txt`（本地工件）。
依赖面（777）：三仓互依全为同级 path、lock 零 git/registry source；反向边仅
auto-lang→auto-ui 4 条（PLAN-814 组合根合同内）。

## PLAN-844 收据（2026-10-28，终值见 plan §9/handoff）

- 生产净删：auto-lang NET-PROD 308,909 → 307,676（**−1,233**，R1 修复后终值；首轮记录 −1,225 系 fmt 归一化前读数，R1 复审更正）；auto-man 资产类 −5 文件
  （AT-O）；auto-ui 零触面。
- 真重复清偿 5 族：应答 JSON 头三份→单源、迭代器注册样板 14 处→`register_iterator`、
  Generator 构造三胞胎→`new_generator_state`、死纯 shim 3 函数删除、autoui-skill 目录
  逐字双份→auto-man 单源。
- 无消费者退役：auto-lang-macros/auto-xml/ac-examples 整目录、infer/registry、legacy
  Scope+Database scope API、StackedScope、Evaluator 变体、elems_mut、set_scope、
  auto-man 拆仓残留。
- 保留有证（非废码）：lib-legacy（playground PLAN-752 活跃）、str.len/math 双轨（语义
  边界不同）、type_registry REPL 面（API pin）、aavm 全家（语料角色）。
