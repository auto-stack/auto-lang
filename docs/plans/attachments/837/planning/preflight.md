# PLAN-837 r1 — planning preflight

2026-10-23；stage:new；outcome:blocked（G836完整交付前置未完成）；草案静态检查pass。next:836 merge owner完成T-10后，用户启动837 work，先T-01核冻结再T-02可行性。

- 按主检出排他allocation锁+new-plan脚本取837；active/archive唯一，下一号至少838；锁未删除。
- drafting r1、0/10、章节0..10；任务10（8work/1review/1merge）、AC10、case11、SD5均唯一，任务有序依赖和验收owner完整。
- 21个不可变Git源码blob与SHA256复核；19个当前知识/计划输入记录工作区指纹与WIP状态。观察的是836 final R2源码5f3e218，不把并行merge的文档后继52ec4c7当新代码验收或交付。
- 本地链接全部存在；新Plan及3附件未被ignore规则屏蔽。源观察不等于837实施冻结。
- 合同核查：同维单父/无环、共享128节点、group仅平移/显隐、真实Auto world/effective、world/local统一消费、f32不能保持位置时明确拒绝、完整子树事务、v1原字节保护/v2显式保存、owner及overlay预算有界。
- T-03 codec迁移与T-07新进程闭合分开；T-04候选owner与T-06/08全路径故障/资源闭合分开，没有要求任务依赖尚未建立的正式入口。新探针由T-02先scratch whole-path验证，不假称已经有profile4/schema2。
- viewport-interaction是836待沉淀组件，G836之后才作为837的modify基面；本轮未编辑canonical或836合同/实施。
- 进度报告为条件推演，I2/I3/生产AutoUI/RQHost/正式Forge/通用引擎与发行均不冒认；不提供无分母百分比。
- 未建837树/分支，未做准备/实现/build/Cargo/原生图形验收或创建agent；本轮仅Plan/附件/.next-id，其他会话WIP保留。

Plan SHA256: fb59c3b2f73087aa399cd711318a85123d140c1a8484cd46674dc7ab3e583812
