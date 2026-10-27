# PLAN-842 冻结门红项诊断（未全部修复）

三仓冻结输入见 [frozen-inputs](frozen-inputs.json.txt)。裸 `cargo t` 3192/3192、`cargo tv` 163/163、`cargo tt` 3563/3563 通过；早期两轮冷构建超时与 tt 一次进程清理红、定向及完整重跑均如计划记录保留。Man 常规 281/281、Core 23/23，UI 生成 861 pass/2 ignored、UI 集成 28 pass/15 skipped；每条记录绑定原冻结提交，不能概括为完整跨仓门通过。

## 计划引入的回归：D-842-G1

UI 常规档 `delivery_redelivery_failure_preserves_old` 原有断言要求坏 producer 错误包含“预检失败”，旧交付不变。新增非受管依赖图预检提前返回 `producer 路径依赖缺清单 ...`，缺少原诊断前缀；旧产物实际仍保留。已在 `delivery_preflight_producer` 对此结构错误统一加既有具名预检前缀，保留原错误内容，不改变拒绝顺序或旧产物语义；[原回归](ui-redelivery-diagnostic-fix.log) 1/1、[plan842 定向](ui-plan842-diagnostic-fix.log) 7/7、[真实交付生成](ui-delivery-e2e-diagnostic-fix.log) 1/1 通过。UI 修复提交 `e4aa3daf0454edc9bd70ab654d2ba410207e8287`，Man 来源 pin 更新提交 `010ef7e`。此项属于 T-03/AC-02/04/08 原范围，修复后须重新独立复审、重绑来源和受影响的安装与冻结门证据；历史 R7 pass 不覆盖新提交。

## 已在当前基面独立重现的四项未决阻断

1. **桌面双进程 smoke**：计划三仓版本及 [固定基面](man-ti-autodesk-baseline.log) 同样在点击后的第一个 count 断言失败，nextest 报 leak。历史 816 同类在案；[仪表诊断](man-ti-autodesk-layout-probe.log) 证明旧点击点被转换为窗口本地 `(44,24)`，而真实首帧按钮矩形为 `(180,127.2,120,36)`。在独立基面副本按真实首帧中 `+` 标签所在按钮 quad 的中心与真实窗口偏移计算点击，[候选探针](man-ti-autodesk-candidate.log) 仍执行两次点击、两个帧递增、L2Detach 与宿主回收，全 1/1 通过；源码副本已恢复。若接手可把此定位改为永久测试，不更改断言或产品行为。
2. **Vue scaffold golden**：计划版本与 [固定基面](ui-css-baseline.log) 同一失败。现有 golden 保留 55 个旧键值，当前生成器额外发射亮/暗各四个 status token，分别为 success `142 71% 45%`、warning `45 93% 47%`、info `217 91% 60%`、error `0 84% 60%`。现行 compiler `scaffold_extended_tokens_dual_face` 明确要求四键；未重写 golden 的 [候选探针](ui-css-candidate.log) 精确保留原 55 键/顺序并额外锁定 8 个指定值，1/1 通过。若接手可明确记录是新增预期并保留原 golden 检查。
3. **merged API CRUD fallback**：计划版本与 [固定基面](ui-crud-baseline.log) 同一失败。原测试无 db、标量 endpoints 现已按 PLAN-681 正式 route A 走 api_impl 而非旧 CRUD；加入无关主类型以排除 route A 后，现行 PLAN-710 GET 生成器仍输出 `Vec<String>` 签名和通用占位体，而原测试要求 `Vec<Value>` 全量列表；[候选探针](ui-crud-candidate.log) 证明仅改 fixture 不能维持旧断言。PLAN-648 的旧 CRUD 实际行为与 PLAN-681/710 的类型和真实 body 优先合同需做有界设计，不能只删断言或改 golden 制造通过；须通过真实生成、编译与 GET/POST 行为正反控决定适用条件。
4. **Vue 增量解析失败语义**：此前 UI 常规门首红即停，75 项未运行。按 nextest 实际选择集减去已执行项，列出 [75 项](ui-remaining-75-selection.json.txt) 并定向补测，结果 [74 pass/1 fail](ui-remaining-75-diagnostic-retry.log)；首次表达式因误用 binary-id 而解析失败，未运行任何测试，纠正为 binary-name 后重试。`vue::tests::test_incremental_parse_failure_semantics` 在 [固定基面](ui-incremental-baseline.log) 同样失败：前置整树解析拒绝坏 store，而原测试要求非严格模式继续生成正常 store、跳过坏 store，严格模式失败。前置校验的目的与严格模式合同需要一起厘清，不可简单删除验证或测试。

这些门禁中的既有失败不可自动豁免。原计划 r2 §5.3 将额外接手范围限定为安装投影缺口，且 [PLAN-841 交接](../../841/v07-v08-closeout.md) 将第 2/3 项列为独立基线债，不重开上游。[修订提案](proposed-scope-extension.md) 列明四项修复和正反控；用户已授权纳入计划 r3，但明确暂不修复。恢复后仍须按 r3 对旧/新行为作有界决策，不运行改动后的整档直到修复冻结并复审。
