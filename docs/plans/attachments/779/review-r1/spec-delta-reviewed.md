# PLAN-779 R1 规范增量冻结

review_scope=final；plan_revision=1；reviewed_commit=451f612bdb67c1f6947db878bb37b3105c4eb8c5；outcome=needs_fix；delta_status=partial。

| 源目标 | SHA256 |
|---|---|
| docs/specs/auto-lang/vm/overview.md | 0a892771a4e9d4902641a78a5911741498264238254e17ea126a866f24e8e144 |
| docs/specs/auto-lang/ui/overview.md | cde75f5c73817c7c8e68108f9200be852ed600912286c13a08a56e2c65c0caa5 |

SD01 modify vm/overview：FrameProvider能力接口、AutoVM构造期快照进程默认、实例覆盖、NI Clone保留provider；provider替换不改变shim绑定身份，register/merge仍保持既有revision/proof失效规则；ID9918/9919、canonical、i32上界收窄及缺席0保持。外部无UI consumer及控制已验证。只证明frame标量pilot，不证明其他UI natives/heap/async已拆。

SD02 modify ui/overview：UI adapter从现有frame_bench读值；当前注册在VmBridge两构造、run_file_dynamic_ui_inner及CLI ui-iced启动。门开bridge已验证；程序轨旧门禁仍失败，F779-R1-01阻断“所有原验证面保持”的完整声明。不能仅凭库消费者自行注册的文字新增要求就销账，须明确真实宿主责任并完成迁移/验证，或先批准合同调整。

supersedes_spec_components=[docs/specs/auto-lang/vm/overview.md,docs/specs/auto-lang/ui/overview.md]；new_spec_components=[]；touched_goals=[GOAL-007,GOAL-003]（保留原计划影响元数据）。

不发布canonical Spec/ledger。修复后重新绑定提交复审。最终pass后merge-owned冻结提交裸cargo t+tv及T07沉淀/合流/归档/批到期/guard-clean pending。
