# PLAN-827 SEM-01..08 决策材料（T-02，2026-10-24）

权威来源：`docs/design/strategy/av-language-capability-matrix.md` §3、
[scalar 合同](../../../specs/auto-av/abc-core-i32.md)（798 已批准语义）、
catalog 报告。规则（计划 §5.1）：**不能用旧 expected 覆盖 798 已批准语义**；本计划
不实现 legacy 兼容 profile、不批准语言破坏变更；有冲突的能力停在其所属后续 G 门。

| ID | 议题 | 证据/权威 | 保留或改变选项 | owner | 阻塞门 | 本计划裁定 |
|---|---|---|---|---|---|---|
| SEM-01 | i32 wrapping vs trap | 798 scalar 合同已裁定：算术溢出/除零/MIN/-1 全部 trap（`TrapKind` 四码，abc-core-i32.md） | 改 wrapping=语言破坏变更 | 后续 G2 数据/算术 Plan | G2 | **不重开**。G1 沿已批准 trap 合同：AP-23=`av.trap.div-by-zero`；旧 VM corpus 中 wrapping 行为仅登记差异，不改 oracle |
| SEM-02 | 负索引 | List/容器未在 AV 源域交付（C-15 typed-only） | bounds trap vs wrap vs UB | 822 | G2·R2-L | 登记缺口，本计划零实现 |
| SEM-03 | str 字段/拥有 | C-13 限（821 在途 drafting） | 拷贝语义 vs 借用 | 821 | G2·R2-D | 登记缺口 |
| SEM-04 | 共享可变捕获 | C-19 明确排除于 821–824 之外 | 允许 vs 拒绝 | 无（后续 R5-O） | G5 | 登记缺口 |
| SEM-05 | 入口/Unit/void main | C-06 未交付；当前唯一合法入口=`fn main() int`+显式 return | 尾表达式/隐式 0/Unit | 798 后继 | G2·R2-E | G1 固定 `d_main`+i32 返回；AP-01/22/23 均为显式 return 形态，无冲突 |
| SEM-06 | 错误 ABI（?/try/panic） | C-26 未交付；trap 码=当前唯一运行期错误通道 | guest trap 码族扩展 | 821/822 | G2/G6 | 沿用四 trap 码；不新增错误通道 |
| SEM-07 | object/null 语义 | C-20/C-24 未交付（AP-19 匿名对象=旧 VM） | null/Option 化 | 824 后继 | G3 | 登记缺口 |
| SEM-08 | 旧误报见证 | AP-18（obj_call_auto_catch.as，py 门控，catalog 已记 conservative false-positive）；其余 `.expected.*` 旧 witness | 逐条修复 vs 保留见证 | 对应 G2..G6 各 Plan | 各 G 门 | **AP-18 不机械当新规范**；18 个旧 VM AP 的 `.expected.*` 一律按"待核输入"登记 gap+owner，本计划不给通过结论、不复制为 AV oracle |

V-12 断言面：上表逐行存在 + 差异归类正确（profile/未决门）+ AP-18 行保留见证状态；
超出本表范围的语义不擅自实现。
