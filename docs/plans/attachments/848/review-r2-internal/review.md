# PLAN-848 r3 — 修复后的受影响复核

stage: review; review_scope: internal; outcome: pass; 2026-10-25。
提交 `6bc5abc6080b2261a43f55a54f127cd779239edb`，repair base
`cef25b5c41dbbfbec6b7bfd4b13507d8d5f1e34d`，Plan base
`bf22db3465203f3babd649494f4b37aa2a012027`。唯一实施树
`D:/autostack/.wt/lang-848/auto-lang` / `plan-848-dev`，提交及clean再次核实。
固定SDK7b987/三锁/AGP与规范输入未变，身份/hash见
[work readiness](../work-r4/readiness.json.txt)。

**上下文限制**：当前会话包含本次实现历史，这份记录是内部受影响复核，
不冒认独立最终pass，不将Plan设为reviewed。已申请用户授权独立只读子代理；
在授权到达前不创建代理或新chat。最终AC09仍pending。

## F-848-R1-01 closure — 内部复核通过，待独立确认

既有要求是128宽/深控制，双group写、2cast+10getter+6AABB恰32credits，
原100000steps/120resumes/250ms及其余deadline不变。核对实际Session预算函数
和Play pump，修复没有改SDK/字节码/计数/上限，没有减候选或仅留易剪枝分布。

最初完整输入保留在永久segment_counterexamples：与R1一致的box/真实Cylinder、
两次group.x写及线段，诊断Fault断言改成成功；actual Session三分布都成功，
正式Host三分布均Paused/tick1，而非原Faulted/tick0。
另segment_budget独立f32 oracle核hit/node/t/point.x/normal.x与精确32credits，
覆盖60组2D/3D、宽/深、X/Y/Z标量写入和6种分布（包括正/负/非等比例三轴斜向），
加6组真实Host资源/完整配置/最后frame oracle。
不是仅把新方法叫cast却读Rust预设命中；实际Auto执行全部候选数学。

代码检查确认：仍先验证匹配box全部finite，才做任何nearest/miss剪枝；
按方向选offset，实际除法f32与ordinal严格t排序不变。暂存最终node/t，
最终点/法线只物化胜者；normal恢复仍XYZ，inside/boundary零，正t下溢0不丢面。
bounds/quotients显式float，原9/12分类测试未减弱；初次错误分类真实失败已留档，
修复后原断言通过。没有把该失败藏到parse/setup或扩大容差。

## 共享图、布局和生命周期不变量

- 从原shared derive校验提取axis/visibility片段再旋转循环；每个脏轴仍按parent-first
  顺序用local+parent一次f32加法，父缓存不跨轴/derive/Run。原world/effective数组
  初次预填后原位更新，不另建图、Descriptor或world-box cache。
- set_local标指定轴，move保守标全XYZ，visible标visibility；形状不完整全量重建，
  未标记却显式derive保守全量。缺Z/visibility标记变异在另一分量已脏时会被实际
  non-deriving字段抓住，不能靠fallback或ag_world自修复掩盖。混合轴、move、
  可见性、父切换与callback最后发布都核过。
- 受信代码compact改变的是实际SOURCE_LINE指令数量；仍由固定Parser/Codegen/VM
  正常执行和收费，没有清零累计步或改slice边界。恢复冗余调试行的同一合法斜向
  控制明确重现100000steps错误，正常布局成功。用户字节和v1/v2分支不处理；
  实际源码测试保留包含引号、分号、花括号和注释样字面量的用户文件原样。
- 11结果标量44B +4dirty标量16B=15/60B，未超过16/64B。固定SDK实际值/键开销
  单列，未把共享map容量当硬配额。64query heap稳定，新V3 main仍10heap对象，
  与V2一致；旧版没有新增dirty/cast globals。原last-good/Stop/Starting弱owner
  与错误恢复在新代码上重放。其他source/Descriptor/GPU最后owner机制没有变化。

## 验收与证据复用

| AC | 内部结果 | 当前或复用证据 |
|---|---|---|
| AC01 | pass | 当前373源/681SDK/17local/29edge/479metadata、三锁、四反控；独立SDK与target消费源刷新后核验 |
| AC02 | pass | API8/geometry6重放；60组独立oracle；9/12、ULP/endpoint/parallel/XYZ/underflow及nearest变异 |
| AC03 | pass | 新API版本分支与old diff检查、API实际身份测试；原V1/V2源文件/入口组合未修改，CLI/版本事务未变面证据复用 |
| AC04 | pass | 最新world、查询瞬时token/下一query/回调失效、最终非派生字段、序列/预算sticky断言当前通过 |
| AC05 | pass | 原反例及60+6控制，原Host4/feasibility6与新混合dirty/缺标记/布局反控；原CPU仍执行 |
| AC06 | pass | 两真实用户项目4原生正控、2新进程重开/改源；未变Binding/Undo/SaveAs证据按代码/输入复用 |
| AC07 | pass | 新dirty/slot实测、heap稳定、Host故障关闭/Stop；未变owner/GPU负控机制与原始输入复用，不冒称本轮再跑全族 |
| AC08 | pass | 同一runner7ba32…的4原生像素/交互、2重开，独立消费12测试，r3 Spec提案及完整未来分母216 |
| AC09 | pending | 本次是实施上下文内部复核，独立最终角色尚未授权/执行 |
| AC10 | pending | 本轮禁止合入；所有完整统一门、沉淀/归档/guard等未做 |

当前producer33个去重定向检查、consumer12个相关检查有实际断言，0selected初次
storage滤串错误单独留档、不计pass。既有主检出/其他会话WIP未纳入实现。
四原生报告逐项核phase14、源SHA、每part像素子断言及2reopen，binary字节hash
`7ba32aa93a71433e1a1774b58fc704b4d9e5417ed353cc94c7cdfa826cce972a`一致。
完整driver216等仅是merge待核分母，不是本轮全包已过；不运行整档/完整包回归。

SD01..04根据r3更新，仍拟议；规范输入hash相对R1没有漂移。共享derive/dirty60B/
受信生成区布局已纳入SD03，AABB精度/数值/源身份/CPU/未迁移I2/I3边界保持，
没有用规范提案降低验收。canonical/live ledger未发布。

遗漏/延后/workaround检查未见新增开放work finding；F01内部确认已修复。
FMT、类型检查、diff whitespace及源闭包通过；固定SDK146预存warning不变，
最终driver无新增warning。代码提交后仅API行尾空白清理，compact逐行trim保证
生成程序不变；最终消费/原生运行使用最终冻结源，来源身份另已刷新。

next: 获得明确授权后在同一冻结树用独立上下文进行最终复审，或交新的独立会话。
Plan保持execution_done8/10；本记录不授权合入。未修产品、未合入/归档/清树。
