# PLAN-771 r1 起草 preflight

- stage: new；outcome: pass（草案结构/覆盖检查，不是实施或最终复审通过）。
- observed_main_head: `540dc7d30839eb7df8e68b4d19472358e170fe5b`；plan_revision: 1；status: drafting；current_step: 0/9。
- 9 个唯一Task（全部未勾）、8 个唯一AC、10 个mandatory Case、5 个SD；章节0..10、work/review/merge责任及同树路径完整。
- 每个work-owned AC有Case/Task责任；每个SD指向docs/specs且有AC；review/merge任务未预勾，材料基线变化必须重走work/review。
- Markdown相对链接真实存在；新实现/报告/脚本路径明确为未来T任务产物，不当已执行证据。
- 40个取材/指令/设计/当前Plan内容hash见[planning-inputs.tsv](planning-inputs.tsv)；768行仅context-only，不构成前置或强制等待，执行时调查相关语义漂移。
- 取号中央scripts/new-plan.sh，在`.new-plan.lock`排他期内顺序创建769/770/771，active/archive每次前后检查唯一，末.next-id=772；锁已正常释放。
- 不确定资源/语义/内部ABI由769 D01..07有界探针认领；770/771只有在其决定一致、复审沉淀后才实施；规则不一致先revision，不默默改oracle。
- 未执行cargo/原生测试、未创建实施树、未修改产品源码/canonical Specs/ledger，保留既有用户WIP。
- next: 用户批准对应r1后、767实际修复复审合入；769先执行，770/771在769收口后可独立并行，第二合入方新基线需fresh work/review凭证。
