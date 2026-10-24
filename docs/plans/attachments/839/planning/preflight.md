# PLAN-839 r1 — planning preflight

2026-10-23；stage:new；outcome:blocked（G838完整交付pending）；草案静态检查pass。next:838完整交付后，用户启动839 work，先T-01正式冻结再T-02可行性。

- 主检出排他allocation锁+new-plan脚本分配839，active/archive唯一，下一号至少840；锁保留。
- drafting r1、0/10、章节0..10；10任务（8work/1review/1merge）、10AC、11case、4SD唯一，任务依赖与验收owner覆盖。
- 23个已交付837基面不可变Git源码blob/SHA256复核，16个知识/计划输入记录WIP字节指纹；838接口是拟议依赖，不能当已交付，未读改其在途Rust。
- 合同一致性：accepted Editor只读快照，额外1运行VM，全tick准入/last-good，Pause完整tick边界、单步中性输入、Stop关闭并恢复保留编辑呈现。
- 运行保留dirty/history/redo与文件，schema不变；内置真实Auto行为样例，不冒认任意用户脚本绑定/代码打包或完整游戏编辑器。
- 60Hz为逻辑dt，8ms共享pump目标/1inflight/4tick额度、2s整事务/30s启动、4096标量摘要、输入held/parity/焦点清空和SDK同步限制均明示。
- 单步只要求实际Auto tick+1，不与中性输入下物体静止矛盾；过载丢墙钟债务不丢已派发完成tick；错误帧关闭受污染VM，Stop后新Run而非继续旧VM。
- VM Stop释放与Editor合法mesh pin分开，最终Editor/外部pin最后owner释放；资源warm复用不等于实例/组合零写入。
- T-03/04模型/VM，T-05实际Host/文件，T-06输入，T-07 OS/GPU/重开分阶段闭合case；T-02先scratch whole-path，不要求尚未建的正式模块已有。
- 所有本地链接存在，新Plan/附件未被ignore规则屏蔽；scene-assets须838先沉淀，当前不写canonical。
- 未建839树/分支，未准备/实施/build/Cargo/原生图形测试、未创建agent；本轮仅Plan/附件/.next-id，保留其他会话WIP。

Plan SHA256: 2bbbcfc5ddf6e36319b8cd72df761b8a4b9030224d8a6bf23bdedb3c2d250623
