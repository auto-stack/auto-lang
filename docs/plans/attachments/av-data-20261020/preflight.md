# AV数据架构与820–824起草预检（历史r1）

本附件保留r1起草时的范围、数量与状态；当前r2/r3及825/826以[ABC采纳记录](../av-abc-adoption-20261020/decisions.md)为准。

日期：2026-10-20，Asia/Shanghai。范围：Design44、Design43衔接、设计索引、五份drafting计划及本附件。取材版本见[planning-inputs.json.txt](planning-inputs.json.txt)。这是起草者的合同预检，不是独立实现review，不执行代码或推进Plan状态。

## 1. 完成范围

- 正式目标态设计覆盖表示/布局/Place/初始化/权限/生命周期、类型化集合/ADT、闭包、独立Dyn、portable、cache/代际和实施顺序。
- 通过仓库脚本在互斥及独占文件锁内领取820–824；发现归档819已占号而.next-id仍819，核对active/archive最大值后修正到820再取号；最终.next-id=825，无覆盖/复用旧Plan。原有空锁文件保留。
- 五Plan保持drafting、current_step=0、total_steps=8，各有11个编号节、8任务、8 AC、10必需V案例；820/821各6 SD、822–824各5 SD（含正式recipe登记，前两份另含HIR消费阶段）。所有实现/测试/review/merge未完成。
- D-798与各前序D均要求实际review/冻结门禁/合入/Spec收据；不以在途分支替代。817/818只有标量兼容与共享owner交接责任，不被扩成新数据消费者。
- 不改canonical Spec、.autoos/specs.json、旧VM、共享实验源码、798/817/818合同或其他会话的变更。不创建实施worktree、代理或新chat，不提交/合入。

## 2. 交叉审查及已消歧事项

| 风险 | 起草时处理 | 验证/实际边界 |
|---|---|---|
| Rust enum作为载体被误认为动态语言，或为8B槽提前unsafe | Design44 §4区分安全参考实现、逻辑布局和物理存储；NaN-boxing推迟至实测专项 | 820先安全实现；此处无实现证明 |
| 821重复定义已交付text/record语义 | 引用774/782的Str/str/char、默认view、整根move/clone与拒绝面 | source支持的record/text-record必须经真实HIR；额外类型只typed fixture |
| LHS求值后RHS使内部指针失效 | 明确reservation、根级保守冲突、只读RHS、结构epoch/代际重验；禁跨挂起/未知callback | 821/822 V反例分别验证，非通用两阶段借用承诺 |
| move/replace先销毁旧值导致失败丢数据 | prepare/validate/commit/drop，源转移在提交点；失败原值保留 | 821故障注入、822扩容/分支失败、824失败take/freeze均有owner |
| 合法参数view因callee退帧被误拒 | token追踪真实owner；允许沿774参数/静态来源，拒局部/move-in/活mut来源 | 821 V-06的合法与非法来源对照 |
| 798标量Copy derive扩面后复制owned责任 | Design44 §7和821要求重审所有槽/参数/返回/临时；定位handle与owned token分离 | 不是仅添加枚举variant即可完成 |
| 预算耗尽后不清理，或cleanup再次耗尽正常fuel | 内建无回调/无guest分配drop、显式遍历、预留记账、真实事件和峰值 | 用户析构/强杀清理/循环shared不在首批 |
| 新返回类型偷偷放松旧core-i32入口 | 新profile typed入口/结果和宿主owner独立，旧entry保持零参/i32 | 820/821明文，817/818旧消费兼容必须实测 |
| Dyn自动容纳所有可描述类型，freeze保存进程身份 | 824只白名单；借用/闭包/资源/共享图/环拒绝，portable树恢复重建handle | 823闭包支持不等于824可冻结闭包 |
| 补充案例无owner/把未来门禁算已完成 | V-01..10逐行AC/work任务/入口/oracle；review、merge各单独任务及AC | 未来脚本和测试前缀明确不存在、未运行 |
| 公共核心与在途817/818/SDK拆分并发改写 | 五Plan实施串行；共享API/注册须owner交接/实际合入；只读准备可先做 | 不创建/消息其他会话，不修改其Plan |

## 3. 静态校验

执行内容：检查五Plan frontmatter、编号节、任务/AC/V/SD唯一性及数量；检查本批Markdown本地链接、fence配对、行尾空白、取号active/archive唯一性、.next-id、Design43/索引双向导航；检查本批tracked diff。

实际结果：PASS。8份正文（Design43/44、五Plan、本预检）的96处本地Markdown链接全部可解析；fence配对及行尾空白检查通过；五Plan编号/状态/任务/AC/V/SD数量与唯一性通过；820–824在active/archive均唯一、.next-id=825；Design43与索引均有Design44入口；本批tracked diff whitespace检查通过。检查中将820布局溢出反例的oracle明确为“执行0且非法layout不发布”，不错误要求用于识别溢出的布局计算本身为0。

Rust/cargo/docs_gen/新脚本/AutoDown均未执行；不能由文档检查授信任何实现或资源数字。仓库其他会话并发簿记变化不纳入本批diff。

## 4. 后续能力与责任

原生复合MIR lowering由818后续扩面Plan负责，复合CTE producer/consumer由817后续版本化Plan负责；完整宽数值算术、shared/weak循环图、async/Send/Sync、在线布局迁移/OSR、生产SDK/默认入口、完整AAC/AAV自举仍未认领为本批交付。不得在820–824通过时顺便宣布这些能力完成。

用户已授权本轮设计/起草，无待答架构问题；实施因明确“先不写代码”保持不启动。各Plan的stage:new outcome:blocked只描述实施handoff的授权/依赖门，不表示本轮文档请求未完成。
