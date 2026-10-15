# PLAN-779 R2 规范增量冻结

review_scope=final；plan_revision=1；reviewed_commit=42905b4b596dd36ea5b0a08b1406a34dce1215a1；outcome=needs_fix；delta_status=partial。

Spec源：docs/specs/auto-lang/vm/overview.md SHA256=0a892771a4e9d4902641a78a5911741498264238254e17ea126a866f24e8e144；docs/specs/auto-lang/ui/overview.md SHA256=cde75f5c73817c7c8e68108f9200be852ed600912286c13a08a56e2c65c0caa5。

SD01 modify vm/overview：FrameProvider接口、per-VM快照/覆盖、NI Clone保留provider、ID/canonical/i32上界/缺席0及既有binding revision/proof规则已验证；不能宣称高层入口可保留显式默认，R2-01阻断。

SD02 modify ui/overview：root composition助手已扩展到程序轨、配置、REPL、解释器/back_proxy等，原程序轨及bridge门开验证通过。但助手无条件替换外部默认，需在修复后明确“缺省装配”与“显式替换”的优先关系；不可把覆盖用户provider写成既有合同允许行为来销账。沉淀时更新原Plan仅列三个注册点的旧提案，描述最终实际宿主责任及有界pilot，不宣称整个runtime拆分完成。

supersedes_spec_components=[docs/specs/auto-lang/vm/overview.md,docs/specs/auto-lang/ui/overview.md]；new_spec_components=[]；touched_goals=[GOAL-007,GOAL-003]。

仅冻结审查结论，不发布canonical/ledger；修复后重新绑定提交复审。最终pass后merge-owned裸cargo t+tv及T07沉淀/合流/归档/清理pending。
