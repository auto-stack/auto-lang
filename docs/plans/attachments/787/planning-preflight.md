# PLAN-787 r1 起草预检

2026-10-15；stage:new；outcome:pass（规划契约）；next:work，T-06接入前必须通过D-783。当前只交付计划，未创建实施树、未实施或运行测试。

## 定界与来源

- 用户要求在783实施、review期间继续规划；使用auto-plan-new及verification-contract。新计划号787，784–786属仓治理/host/provider/reader路线，与本实验无生产热点写入重叠。
- 主检出master调查HEAD f56765c2f30f06f1ecfe816cfb007b9537fd5673。查看783专用树62d93dae8的候选接口/工作报告，未修改其实现或review附件。
- 起草期间783 R1报告出现needs_fix。已明确依赖门：783实际修复、独立review与merge收据、Spec沉淀后才进行T-06。不能把工作性能报告/绿色套件视为最终验收，也不将其findings转交本计划。
- 全层cache合同已由783负责；本计划只接真实grammar/token/CST。A03合成造数不再作为真实语法证据，GC/生产迁移不重复实施。
- 语法支持面以core Spec为准，保留/*排除和换行规则；第二个interface-mini是明确新小语言，非C/Rust。语法候选不授予FFI/ABI信任。
- 当前只规划，新增experimental/ac-syntax、有限783实验schema/codec扩展；不修改crates/ac-core生产源码，不切生产parser，不搬仓。

## 合同一致性

- 0–10编号、frontmatter、9任务/0完成、AC-01–08、V-01–13、SD-01–06对应；work/review/merge归属分明。
- T-01负责最小全路径及有限grammar能表达core的技术可行性，不能通过删语法、硬编码Auto parser或增加预算过关。
- 两语言共用runtime，配置关键字/precedence反控实际执行；左递归/nullable repeat/非前进规则生成期受控拒绝。
- lossless重建与strict投影分开，错误当前修订仍可outline；投影不得再调旧parser；不宣称新CST已经参与native。
- 增量双oracle：fresh等价和真实局部复用计量。困难编辑允许显式回退；迟到任务不得覆盖当前发布。
- 深度检验包括构建/查询/编解码/释放，无固定语法嵌套上限；资源拒绝不冒充语法错误。
- 小查询64MiB实际source负载与1MiB局部编辑负载分开计量；16MiB受管总预算包含索引/热点/暂存，OS峰值另测。数值为实验提出，不冒充已批准产品默认值。
- 缓存读入口必须验证请求ChunkId与实际frame身份，包括有效frame替换反例，不以checksum helper取代真实入口；坏offset受控拒绝。
- D-783只阻止接入任务，独立T-01–05可先行；没有跳过AC-05进入execution_done的路径。
- 最终review独立上下文同树、绑定提交；完整独立套件merge-owned，但功能矩阵不能后置。无根cargo/docs_gen测试。
- SD拟沉淀新auto-syntax模块与实际产物接入边界，不提前编辑canonical Specs或宣称通用生成器完成。

来源逐文件SHA-256见planning-inputs.tsv。未来入口均明确由实施任务创建，当前不存在不算证据缺失或已执行通过。
