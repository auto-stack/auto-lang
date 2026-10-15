# PLAN-786 independent final review R1 — needs_fix

- reviewer_identity: Codex / 本聊天独立复审上下文；未参与该实现，未委派其他代理。未声称额外模型/会话身份。
- stage: review; review_scope: final; date: 2026-10-15; plan_revision: 1
- worktree: D:/autostack/.wt/lang-786/auto-lang; branch: plan-786-dev
- reviewed_commit: bde9d0523198f1a15bc908c62a5b18b59324e61a
- base_commit: a1d5ed3114b163fa8a10c3d8985f5ae664bdad2c
- dependency_revisions: auto-project-model tree b4a5de79bf608a897944fb794f16201c850e3d29（diff 未改）；组内 auto-down c39ac7f（执行收据）；Cargo.lock SHA256 DAA795C5B9A39810836A5B8770D939B9175D8C506CA595415B57EC22CD96BC00。
- spec_inputs: 两目标当前源版本/hash与 proposed delta 见 spec-delta.md；本轮不发布规范。
- 起止工作树 clean、HEAD 一致。主检出有其他会话的计划簿记变动，仅改本计划及复审/债务附件。

## Findings

| ID | priority/type | AC/tasks | Evidence / required repair |
|---|---|---|---|
| F786-R1-01 | P1 / new production regression | AC02, T02/03/04/05 | reader.rs:76 无条件解析全文。输入 `name: "n"\ntitle: "ok"\nextra: ` + 512 个 `[` + `0` + 512 个 `]` + LF（约1KB）时，旧行 oracle 正常返回 ok，新 read 在 Windows 默认主线程栈上 abort（STATUS_STACK_OVERFLOW，exit -1073741571 / 0xC00000FD）。32/128 层控制成功；512 在外部10秒截止内重复崩溃。非选字段改变导致标题读取进程级失败，不能返回具名兼容。需要保证准入/解析及退出在代表性的结构化/坏输入上安全，避免将递归库未处理失败引入展示入口，并补进程隔离的永久回归。单纯 catch_unwind 捕获不到该 abort；不要顺手加未经合同批准的语法限制。探针只 include 原生产源码并链接实际模型库，无实现改写。 |
| F786-R1-02 | P2 / regression + stale readiness evidence | AC02/04, T04/05 | `cargo test -p auto-man --lib project_model --locked` 实际 10 pass / 1 fail。新增 display-reader/single-quoted-title.pac.at 进入 git ls-files 管理语料，被旧 VM 拒绝而模型归为 unsupported，不在严格精确身份例外表，project_model_corpus_comparison_v02 判红。此前声称 11/11、170 文件没有覆盖提交后新增8文件；本轮实际178。应隔离不同 oracle 的负例集合，或以有依据的精确分类维护语料合同并补负控；不得放宽未知坏输入门或全目录豁免。复跑提交后实际 corpus。详见 adapter.log。 |
| F786-R1-03 | P3 / health closeout | AC04, T04/05 | 触面 fmt 非 clean：vue.rs 新 name 兜底表达式（3314）及 locale 测试新行（10009 起）不符合 rustfmt。两个新增 reader 文件本身通过；vue/lib 中另有大量原有漂移，不要求扩大为全文件重排。修正本次改动行的格式并准确记录检查边界；fmt.log 含具体建议。 |

## Acceptance mapping

| AC | status | task/code/evidence |
|---|---|---|
| AC01 | pass（接线/去重本身） | T02/03：生产 Vue generate 一次 read；两循环删除；legacy_scan 单份；Model(Value) 正常三字段确实可达。原调用者 locale/兜底未改。 |
| AC02 | fail | T02/04：6 reader tests pass，独立260组合 pass；F01 终止/安全回退失败，F02 指定 consumer 回归失败。 |
| AC03 | pass | T03/04：既有 title、locale 两项各1/1；非选字段实现 diff 未改，Cargo manifest/依赖未改、无反向边。F01 是其无关字段解析引入的安全问题，另列AC02。 |
| AC04 | fail | T06 本轮审查完成但不勾 pass；代码/证据/SD尚不能签发通过。 |
| AC05 | pending / merge-owned | 未运行裸 cargo t 等整档，未沉淀/合流/归档/清理。 |

## Reproduced verification

- cargo test -p auto-man --lib display_reader --locked -- --nocapture: 6/6；178 files / 534 field_reads / model=515 / compat=19。与工作旧收据170/510不同，因为提交新增8 fixtures，非随机计数差异。
- cargo test -p auto-man --lib pac_title --locked: 1/1；pac_display_title_locale_chain: 1/1。
- cargo test -p auto-project-model --locked: 30/30；auto-man project_model: 10/11，详见 adapter.log。
- 独立 counterexamples.rs：5种前缀 ×13种值 ×4种行尾=260 组合，三字段对冻结旧算法均一致。数组32/128成功、512 abort；第二次 Start-Process + WaitForExit(10000) 重现，未超时。
- cargo check -p auto-lang -p auto-man --locked: exit0。既有依赖及非触面 warnings 存在；新 reader 文件无新增 warning。不宣称全仓零警告。
- rustfmt --check --edition 2021 --config skip_children=true 两个新增reader文件: exit0；同命令含 vue/lib: exit1（含新旧格式差异）。git diff --check a1d5ed311 HEAD: exit0。
- 首次 cargo 因本机 sccache 端口 WSAEACCES 失败；仅本进程 RUSTC_WRAPPER='' 后重跑成功，无配置写入。
- 补充 consumer 独立 manifest --locked 无 Cargo.lock 因而未执行成功（consumer.log）；不是本次新增回归，未计为 pass，不替代上述明确要求的project_model gate。

## Coverage / knowledge / routing

已查全部V01..05、原调用点、compat完整分支、旧oracle与from_scan归一、SD两目标。未发现未授权VM求值或隐藏新增依赖。正常/重复/空值/藏行/注释/转义/动态的值兼容有证据；当前10k平面输入没有外部截止且不能证明结构化递归安全。独立probe加外部截止。无全量性能承诺，本反例用于已有“坏输入/非选字段兼容安全”要求。

supersedes_spec_components / new_spec_components / touched_goals 保持正确；SD 文本须区分 consumer 的值等价准入与模型静态子集（数值、单引号空值等不能被错误概括为全量静态Str）。迁移附件需明确登记新adapter路径；spec-delta.md 保留本轮提议，修复后重新冻结其hash。

outcome: needs_fix。保留T01；重开T02/03/04/05及依赖验收，current_step=1/7，status=executing。T03正常接线证据可保留，但整体依赖不再算完成。修复在同一工作树进行、提交后重新复审。本轮没有修复实现或合流。

pending_closeout_gates: final review通过后由merge在冻结提交统一裸cargo t；当前未改ui_gen/book/VM/aavm，不追加tu/tb/tv/taa。随后规范/ledger、归档、batch到期判定、guard-clean。不得在needs_fix迭代运行整档。

持久化 replay.ps1 最终复跑（counterexamples.log）：260组合、32/128层仍通过；512层再次输出 stack overflow，但进程在10秒内未完全退出，外部截止将其终止，runner exit1。此前两次直接运行/独立进程均实测STATUS_STACK_OVERFLOW；该Windows崩溃退出时序差异不改变反例，亦说明需要外部截止。
