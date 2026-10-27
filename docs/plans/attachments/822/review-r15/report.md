
### 2026-10-27 独立最终复审 R15

stage: review | review_scope: final | reviewer_identity: 当前聊天Codex，未实施修复，独立核合同/源码/反例，无代理 | plan_id: PLAN-822 | plan_revision: 5 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-822/auto-lang | branch: plan-822-dev | reviewed_commit: 14b6a9df2e52a46c72f49789e1dcd91818d11da5 | bookkeeping_commit: 3e4b68235 | base_commit: de93bacc0ad16622140b0adb4d36eee86fe7fc83 | repair_base: fab5f492a74377d97c508e02dcea1916504bf8d2

F-822-R14-01 CLOSED：实际decoded类型表指纹在execute前比较，原不同类型表恢复拒绝；正式cache_compat六控本轮独立运行6/6绿（含原反例及诚实store正控）。但其要求的相邻身份审计未完成：提交称 envelope_unwrap 已比较 embedded entry/snapshot 与请求，实际 collections_envelope 的 _entry 被忽略，snapshot 仅在外层metadata核对，restore 未比对 decoded function身份。

F-822-R15-01(P1，R14相邻身份残余，AC-06/V-08，T-05/T-06，SD-03)：restore_collections 在verify_abc_bytes及types_fp比较后直接execute，未绑定payload实际入口函数名及source_snapshot。独立真实store两反例均使用first-case原类型表（types_fp相等46d81ee5fdbb84a2），原owner/entry/snapshot key及matching envelope；①只把实际入口函数name改为foreign_entry，②只把实际函数source_snapshot改为请求值+1。两者均经MIR/ABC验证合法，restore均返回Ok(Success I32(0),steps121)，拒绝oracle各红，进程exit101。违反§5.3缓存身份/恢复不可信及V-08，不是要求拒绝合法改名/新修订，而是不能在旧请求身份下执行不同payload。已建立原因=外层身份检查与实际解码内容未绑定；未把其他未测试轴冒领失败。

最小重放：将 attachments/822/review-r15/replay.rs 放在既有plan树 experimental/ac-av-probe/probe/tests/review_822_r15.rs，运行 cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test review_822_r15 -- --nocapture；本轮已有该临时文件，证据保存后移除。关闭oracle：两错配在dispatch前受控拒绝，原诚实恢复/依赖clean miss保持；核查其余实际payload身份轴，避免再用封套元数据替代实际内容核对。无全档要求。

acceptance_results: AC-06 fail、AC-07 fail；AC-01..05/09保留原证据边界，原R14类型指纹子面关闭；AC-08 pending。SD-03恢复身份完整主张仍不批准，其余SD维持既有边界；冻结SD hash=4FB74793E02982B904317A13D2FF0DFB10088A0D902D69249C6482CFCE1EE739。依赖/配置未变，Cargo.lock沿原SHA256；最终HEAD/cleanliness复核见input-hashes。未实施修复/发布canonical或ledger。

next: 同树work修F-822-R15-01并补真实入口名/快照正反控，T-05/T-06重开，executing5/9，其他任务保留；提交后复审受影响内容。merge-owned冻结裸cargo t+tv+实验workspace有限合同档及实际触面档仍pending，仅最终pass后执行；tf主检出批量。本轮无整档、合入、归档、清树。
