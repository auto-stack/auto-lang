---
plan_id: PLAN-779
status: reviewed
feature_name: UI Frame Native 宿主注入探针
author: [Codex]
created_at: 2026-10-13
updated_at: 2026-10-15
plan_revision: 1
current_step: 6
total_steps: 7
supersedes_spec_components: [docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/ui/overview.md]
new_spec_components: []
touched_goals: [GOAL-007, GOAL-003]
affects: [docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/ui/overview.md]
---

# [PLAN-779] UI Frame Native 宿主注入探针

## 0. 变更摘要

以frame.begin_ms/present_ms两函数验证native定义与实现装配分离。真实external provider供值，保持现有UI生产装配与未捕获行为；之后才能推广clipboard/media/terminal。

## 1. 目标

剪断选定frame shim的core→ui::frame_bench直接实现依赖，保持canonical名、ID9918/9919、i32 clamp、未捕获0。生产UI从host注入；两VM实例可用不同provider；register/merge的contract/proof规则保持。

非目标：全natives迁移、新VM线程模型/NanoValue、async终值/776合同、媒体寿命、全UI独立仓或通用动态插件平台。

## 2. 架构方案

复用NativeInterface的register/typed contract/binding revision/proof；最小FrameProvider或等价能力接口由host factory装配。Core提供合法未捕获0，UI adapter读现有frame_bench；不另造native系统，不用全局单例provider造成实例串线。

## 3. 技术栈

Rust、现有Arc/NativeInterface/SDK宿主装配；provider Send+Sync不代表!Send VM可跨线程。

## 4. 需求分析与背景调查

用户2026-10-13只授权本轮调查与报告/规划；实施需确认本Plan revision 1，不创建新仓、不发布/合入，不改其他在途Plan/WIP。

前置777；建议778后顺序实施。Specs vm/ui、Design37与 reports/v06-modularization-baseline.jsonl hash。实锚vm/native_catalog.rs:63–64；native.rs::production/register_std_shims/shim_frame_begin_ms/shim_frame_present_ms；ui/frame_bench.rs的frame_*与note_*；AutoVM::new/merge_native_interface、UI vm_bridge/handler_codegen/lib装载路径；a2r_std.rs的frame同源语义。

## 5. 详细设计

T01枚举全部选定族注册/调用，冻结无provider行为、ID/contract/值编码。拟新 `src/vm/host_capabilities.rs` 或native等价API，`src/ui/frame_provider.rs`宿主adapter，`test/fixtures/modularity/native-provider-consumer/`独立无UI类型fixture。

UI所有受支持VM创建路径必须注入；core选定实现不再直读frame_bench，不能只多加接口而production继续旁路旧全局。普通Core路径按当前未捕获规则；a2r已有frame调用保持或等价adapter，剩余依赖明确，不冒充已拆runtime。

fake返回非零且两值不同，真实VM stdout证明它被调用；大于i32::MAX仍clamp，binding identity不得以动态分配ID误覆盖9918/9919。T01若发现需要改变已有行为须先修订，不silentfallback为0过门。

### 规范增量

| delta_id | 操作 | docs/specs target | before/after | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/overview.md | frame由core读UI→host供给，ID/编码/降级不变 | 依赖倒置 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-lang/ui/overview.md | 明确frame provider生产注入者与时点 | 不漏真实路径 | AC-03 |

## 6. 测试设计

实现者未指派；work自检后由独立上下文review在同一唯一worktree验收，不修代码；merge仅处理已验收沉淀/归档/清理。本草案没有派遣agent/chat。

T03拟建external gate `cargo test --manifest-path test/fixtures/modularity/native-provider-consumer/Cargo.toml`；worktree `cargo check -p auto-lang`+裸`cargo t`+`cargo tv`。仅改UI codegen才tu，不碰aavm/trans不taa/tt，tf按batch规则。

| V | AC/不变量 | setup/实际入口（T03建） | oracle | 边界 |
|---|---|---|---|---|
| V-01 | AC-01/真注入 | A=(17,31)、B=(43,59)分别装VM执行两native | stdout分别真实17/31、43/59，不是0/0，实例不串线 | 无heap能力 |
| V-02 | AC-02/表示 | 未捕获/无provider、大值i32::MAX+1 | 0、i32::MAX，int lane不变 | 标量族 |
| V-03 | AC-02/identity | 名/ID查找、merge/重注册/proof既有路径 | 9918/9919不撞别族；冲突诊断/proof按revision失效 | 保留现有contract规则 |
| V-04 | AC-03/host | frame_bench note→实际UI host创建VM读取 | 真实观测到值，注入在使用前，core直引被剪断后仍通 | 实际受支持创建路径 |
| V-05 | AC-03/兼容 | 既有corpus+a2r frame约定 | 无新增红/漂移，剩余族与依赖明示 | 不等完整拆仓 |

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 选定frame core无UI类型/直接实现调用，external实值到VM；V01 |
| AC-02 | work | ID/编码/降级/实例/contract/proof正确；V01–03 |
| AC-03 | work | 真UI路径注入，其他装配不损；V04/05；未迁族清单 |
| AC-04 | review | 独立逐AC与旁路扫描，bindcommit/revision |
| AC-05 | merge | SD01/02按pilot实情沉淀、归档、guard-clean清理 |

## 8. 执行步骤

| 状态 | Task | owner_stage | 依赖 | 操作/验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 777+确认r1 | 点检catalog/production/UI创建路径；冻结注入与缺席矩阵 | 01/02/03 |
| [x] | T-02 | work | T01 | provider+frame shims接线，保留ID，剪core frame直引（R1修复：触面rustfmt收口，见§9 R1-repair） | 01/02 |
| [x] | T-03 | work | T02 | 独立consumer，V01–03真VM assertions（R2修复：+高层缺席格；UI 开重放 6/6，见§9 R2-repair） | 01/02 |
| [x] | T-04 | work | T03 | UI adapter与所有生产装配；V04/05；不改media/776（R2修复：兼容默认 if-absent 优先关系，见§9 R2-repair） | 03 |
| [x] | T-05 | work | T04 | check+t+tv+externalgate与宿主实测，后续native族清单（R2修复后双构建覆盖矩阵闭合） | 01/02/03 |
| [x] | T-06 | review | T05 | 独立review，不能外推heap/async已解决 | 04 |
| [ ] | T-07 | merge | reviewpass | Spec/归档/合流/guard-clean清理 | 05 |

T-02–T-05 执行收据（worktree `D:/autostack/.wt/lang-779/auto-lang`，branch `plan-779-dev`，base=master@2253134cb）：

- **T-02（commit 80fade182）**：新 `vm/host_capabilities.rs`（`FrameProvider` trait + 进程默认 set/clear/快照，显式 factory 纪律）；`NativeInterface.frame_provider` per-实例槽（Clone/merge 保留；注入不 bump binding_revision/不撤 proof）；`AutoVM::new` 构造期快照进程默认 + `set_frame_provider`/`frame_provider` 实例 API；shim 9918/9919 改经 `vm.native_interface.frame_provider()` 供值，删除 native.rs 的 cfg(ui) `frame_*_observed()` 直引（**core→ui::frame_bench 剪断达成**）；ID 9918/9919、canonical 名、i32 saturating lane、未捕获 0 全不变。新 `ui/frame_provider.rs`（FrameBenchProvider 读现有 frame_bench，通道零改动）+ 注入点：VmBridge::new_with_children / new_from_decls、run_file_dynamic_ui_inner（-q rqhost 下游）、auto CLI real_main（cfg ui-iced）。
- **T-03（commit 583da2339）**：新 `test/fixtures/modularity/native-provider-consumer/`（独立 workspace、**default-features=false 无 UI 构建**、根 exclude 双保险）。V-01(a) run_with_capture 全管线：默认 A→stdout 17/31、换 B→43/59；V-01(b) 低层双活实例（Parser→Codegen→Linker→VirtualFlash→AutoVM 公开链）：构造期快照换默认不影响已建实例、实例级覆盖生效、互不串线；V-02：缺席=0/0、i32::MAX+1 与宽 i64 saturating 2147483647（int lane 单槽）；V-03：canonical 解析钉 9918/9919、邻段族不撞、provider 注入不 bump revision、register_static/merge 重注册既有 bump/失效规则保持、merge 覆盖 shim 端到端派发 7。**门禁 5/5 绿**。
- **T-04（同 commit）**：in-crate `tests/plan779_frame_provider.rs` 三锚——core 剪断源扫描（native.rs 无 `ui::frame_bench`、host_capabilities 无 `ui::`）、VmBridge 构造即注册进程默认（注入在使用前）、merge 保实例 provider 且 revision 照 bump。plan716_supply_probes 14/14（bridge+门开真值保真=V-04 oracle；a2r 臂 grep 锚不动）。
- **T-05**：`cargo check -p auto-lang`/`-p auto` 零错；定向面 plan755 4/4（production/merge/proof 契约面）、frame 族 64/64；external gate 5/5；**宿主实测**=auto CLI 直跑 .at（main 注入路径）输出 0/0 未捕获语义、干净退出；iced 真窗口循环无法自动化（无显示环境），以 bridge+AUTO_FRAME_BENCH=1 探针（plan716 门开族）为自动化等价证据。**merge-owned 门禁 pending**（2026-10-14 时序裁定，计划文档 10-13 先于裁定）：裸 `cargo t` + `cargo tv`（VM/编译器触面档）由 merge 在冻结提交上执行——V-05 语料面依赖该档。
- **后续 native 族清单（未迁/保留依赖明示，AC-03）**：(1) a2r 臂保留（计划 §5 明示）：`a2r_std.rs` frame 模块 cfg(ui) 直读 frame_bench、handler 体 a2r 发射 `auto_lang::ui::frame_bench::*`、`stdlib/auto/frame.rs.at` 同源声明；(2) core 仍持 cfg(ui)/cfg(code-editor) 直引的族（后续按 Design 37 §4 逐族迁）：code_editor（2910-2924/2932-2933/2939/9906/9908-9913）、diff（9915-9917/9921）、clipboard（2925-2926/2934-2937）、dialog（2927-2928）、console（2916-2918）、shell.add_recent（9914）、dnd（2938）、scroll（9900-9905/9922-9923）、term（2943-3003 段）、dom/ui.focus/storage/sched（2774-2784/9920 等）；(3) 本 pilot 不证明 heap handle/media/async lifetime 已解（plan 非目标）。

T-01 冻结矩阵（2026-10-15，master@2253134cb 实锚；用户本轮指令=rev1 实施确认）：

**注册/调用面（改动前后均不变）**：shim 表 `native_catalog.rs:63-64`（9918/9919 → `shim_frame_begin_ms/present_ms`，canonical `auto.frame.begin_ms/present_ms`）+ bigvm 名表 `native_catalog.rs:884-885`（`("auto.frame.begin_ms", 9918, Int)`）；注册路径 `NativeInterface::production()`(native.rs:141) → `register_std_shims`(native.rs:995) 宏消费 for_each_native；.at 调用面=两段名 `frame.begin_ms()`（codegen.rs:9645-9646 归一 canonical；签名面 stdlib/auto/frame.vm.at `#[vm] fn begin_ms() int`）；值编码=i32 单槽 `.min(i32::MAX)` saturating；未捕获=0。a2r 臂=a2r_std.rs:1219 `frame` 模块 cfg(ui) 直读 frame_bench + handler 体发射 `auto_lang::ui::frame_bench::*`（生成代码，plan716 探针 grep 锚）——按计划 §5 保持，列入未迁清单。

**VM 创建路径（全部经单一收口）**：`AutoVM::new`(engine.rs:809) = `NativeInterface::production()` + CFFI merge；`new_with_capture`(engine.rs:1063) 委托 new。生产调用点：lib.rs run 管线（1995/1998，`auto run` 主路径）、ui/vm_bridge.rs:675（VmBridge::new_with_children→AutoVM::new(4096)，DynamicComponent/UI 组件桥；new_with_vm 遗留入口忽略传入 VM 重建）、autovm_persistent.rs:104（REPL）、back_proxy.rs:1581、vm_interpreter.rs:149、py_ffi.rs 多处。→ 注入点定：`AutoVM::new` 内快照进程默认 provider（每实例独立槽，之后换默认不影响已建实例）+ `set_frame_provider` 实例级覆盖；merge_native_interface(engine.rs:1223) 经 Clone+merge 保留 provider（provider 非 shim/contract，不 bump binding_revision）。

**宿主注册点（ui→vm 单向）**：新 `vm/host_capabilities.rs`（trait FrameProvider + 进程默认，显式 factory 纪律，Design 37 §4）；新 `ui/frame_provider.rs`（FrameBenchProvider 读现有 frame_bench）；注册于 (a) VmBridge::new_with_children（UI 组件路径，含 plan716 探针兼容）、(b) run_file_dynamic_ui_inner(lib.rs:5560，库消费者 UI 入口，-q rqhost 客户端下游)、(c) crates/auto/src/main.rs 启动（cfg auto crate 的 ui-iced feature）。renderer note 调用点不动（renderer.rs:19657/20409）。

**缺席矩阵**：无 provider（core 路径/未注册宿主/无 feature 构建）→ 0（=现状未捕获语义；frame_bench 静态无泵恒 0，行为等价）。AUTO_FRAME_BENCH 门控在 frame_bench 层不动。

拟tree `D:/autostack/.wt/lang-779/auto-lang`，branch `plan-779-dev`；构建/测试在该树，Plan簿记在master，禁止链接。

## 9. 复审记录

2026-10-13 stage=new，r1，outcome=pass（草案preflight）；next=确认后work。0/7任务，external/host probe均未创建运行，不是独立实现复审pass。

2026-10-15 stage=work | PLAN-779 | r1 | outcome=pass（work-owned 全完成，送审交接）| worktree=D:/autostack/.wt/lang-779/auto-lang | branch=plan-779-dev | base=2253134cb | code=451f612bd（80fade182→583da2339→451f612bd）| deps=auto-down 兄弟 detached@c39ac7f（组内路径依赖）| task_ids=T-01..T-05（5/7；T-06 review、T-07 merge 未计）| evidence=check -p auto-lang/-p auto 零错；fixture gate 5/5（V01-03，无 UI feature 构建）；plan779 in-crate 3/3；plan716 14/14；plan755 4/4；frame 族 64/64；CLI 宿主实测 0/0 未捕获干净退出；fmt：触面文件零 drift（仓内预存 28 文件 drift 不在触面）| blockers=无（开放 in-contract 项为空：AC-01/02/03 的 work 面全部闭合，V-05 语料面=merge-owned tv 档）| merge-owned pending=裸 cargo t + cargo tv（2026-10-14 时序裁定，冻结提交上执行，通过前不得合入）| next_responsible_role=review（T-06：独立逐 AC 验证 + 旁路扫描，bind 451f612bd）

2026-10-15 R1：stage=review | review_scope=final | reviewer_identity=本拆分规划/其他探针评审会话，未实施779 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-779/auto-lang | branch=plan-779-dev | reviewed_commit=451f612bdb67c1f6947db878bb37b3105c4eb8c5 | base_commit=2253134cbd8b6e4a6273bf92f2145c3f34ce8934 | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；锁/源码最终核对见指纹 | spec_inputs=vm/overview SHA256 0a892771a4e9d4902641a78a5911741498264238254e17ea126a866f24e8e144；ui/overview cde75f5c73817c7c8e68108f9200be852ed600912286c13a08a56e2c65c0caa5；SD01/02 partial冻结 | acceptance_results=AC01/02功能pass、AC03 partial、AC04审计完成未pass、AC05 pending | findings=F779-R1-01(P2门开程序轨验证未随provider宿主装配迁移：真值2而VM输出0/0)/F779-R1-02(P3新增provider/native触面格式未收口) | evidence=[R1报告](attachments/779/review-r1/REVIEW.md)、[运行命令/env/退出码](attachments/779/review-r1/runs.jsonl)、[门开程序轨](attachments/779/review-r1/gate-program.log)、[门开bridge正控](attachments/779/review-r1/gate-bridge.log)、[触面格式](attachments/779/review-r1/fmt-finding.log)、[冻结SD](attachments/779/review-r1/spec-delta-reviewed.md) | verification=consumer5/5、plan779 3/3、plan755 4/4、默认门态plan716 14/14、check(auto-lang+auto)/diff-check通过；独立AUTO_FRAME_BENCH=1单测程序轨exit101，bridge正控exit0；core选定frame直引已剪、a2r保留臂不变；触面fmt失败，不能归为全仓预存drift | task_state=保留T01/T03，重开T02(格式)/T04/T05及依赖验收，executing 2/7，r1不变 | pending_closeout_gates=最终review通过后merge-owned冻结提交裸cargo t+tv及SD/ledger/合流/归档/批到期判定/guard-clean | next=同树work修复宿主验证接线及格式并提交后复审；保留纯Core缺席0反控，不接回core→UI直引，不删/忽略门开失败或改期望0。本轮不修实现、不跑整档、不发布canonical、不合入或清理。

2026-10-15 R1-repair：stage=work | PLAN-779 | r1 | outcome=pass（F-01/F-02 修复完成，送 R2 复审）| worktree=D:/autostack/.wt/lang-779/auto-lang | branch=plan-779-dev | base=2253134cb | code=42905b4b5（链：80fade182→583da2339→451f612bd→42905b4b5）| task_ids=T-02/T-04/T-05（重开项重勾，5/7）| blockers=无

- **F779-R1-01 修复（invariant 级）**：crate 根声明为内建组合宿主——新 `install_ui_host_capabilities()`（lib.rs，cfg(ui)→装 frame_bench adapter 进程默认，no-ui=no-op，零 core→ui 引用），接线**全部生产执行入口**：`execute_autovm_with_deadline`（run 家族单一漏斗：run/run_with_capture 系/run_autovm 系/staged 变体）、`test_code`、`debug_autovm(+_agent)`、`create_vm_from_source`、`create_vm_from_abt`、`eval_config_with_vm`、REPL `AutovmReplSession::new`、`vm_interpreter::run_ast`、`back_proxy::load_back_session`。py_ffi 全部 14 处 VM 为 `#[cfg(test)]` 值转换助手（非宿主，不接线）。幂等安装；实例快照/覆盖语义与 no-ui 缺席 0 负控（外部 fixture v02）不变。
- **F779-R1-01 反例复放（本提交上）**：`AUTO_FRAME_BENCH=1 cargo test -p auto-lang --lib frame_program_track_gate_on --locked -- --nocapture` → 1 passed（原 exit101/stdout 0/0 vs 真值2）；bridge 正控 `frame_value_flow_gate_on` 同命令 → 1 passed。
- **F779-R1-02 修复**：三触面文件过 `rustfmt --edition 2021`（复审精确命令 --check exit0）；两处方法体缩进修正；其余触面文件新增行零 fmt diff（预存 drift 不动、不归责）。
- **修正 R1 前work收据的过度声明**：原"全部work面已闭合"不实——程序轨宿主装配遗漏；现以本记录修正，T-01 矩阵的"宿主注册点"清单同步扩容（composition-root 助手 + 10 入口，原 3 点为子集）。
- **回归面**：门开 nextest 每测一进程（探针设计态）plan716_supply 14/14；门关 14/14；单进程并行整模块门开有 3 处兄弟探针共享静态泵竞态（探针设计假设每测进程隔离——注释明示"裸 cargo test 下门关自跳"；预存特性非本轮回归，此处记录在案）；plan779 3/3、plan755 4/4、frame 族 64/64、fixture 5/5、check(auto-lang+auto) 0 错、CLI 宿主实测 0/0 干净退出。
- merge-owned pending 不变：最终 review 通过后冻结提交裸 `cargo t` + `cargo tv`；未触 UI 生成器/aavm/trans。
- next_responsible_role=review（R2：独立逐 AC + 旁路扫描，bind 42905b4b5）

2026-10-15 R2：stage=review | review_scope=final | reviewer_identity=本规划/R1评审会话，未实施779 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-779/auto-lang | branch=plan-779-dev | reviewed_commit=42905b4b596dd36ea5b0a08b1406a34dce1215a1 | base_commit=2253134cbd8b6e4a6273bf92f2145c3f34ce8934 | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；锁/源码最终复核见指纹 | spec_inputs=vm/overview SHA256 0a892771a4e9d4902641a78a5911741498264238254e17ea126a866f24e8e144；ui/overview cde75f5c73817c7c8e68108f9200be852ed600912286c13a08a56e2c65c0caa5；SD01/02 partial冻结 | acceptance_results=AC01/02/03 partial、AC04审计完成未pass、AC05 pending | findings=F779-R2-01(P2自动UI装配无条件覆盖显式外部provider) | closure=R1-01原程序轨及bridge门开通过、R1-02精确格式控制通过；两原项closed，新修复回归单列R2-01 | evidence=[R2报告](attachments/779/review-r2/REVIEW.md)、[UI consumer原样输入](attachments/779/review-r2/provider_vm.rs)、[UI构建重放](attachments/779/review-r2/ui-consumer.json)、[失败日志](attachments/779/review-r2/ui-consumer.log)、[冻结SD](attachments/779/review-r2/spec-delta-reviewed.md) | verification=原no-ui consumer5/5、plan779 3/3、plan755 4/4、默认门态plan716 14/14、单独门开程序轨1/1及bridge1/1、check/fmt/diff-check通过；同一consumer源码UI开启重放4pass/1fail，显式A17/31经高层run_with_capture实际0/0，产品exit101，非测试串扰 | task_state=保留T01/T02，重开T03/T04/T05及依赖验收，executing 2/7，r1不变 | pending_closeout_gates=最终pass后merge-owned冻结提交裸cargo t+tv及SD/ledger/合流/归档/批到期/guard-clean | next=同树work先补连续两轮final失败的根因及UI开关×入口×显式/默认/实例装配覆盖矩阵与设计调整，再修复全部finding提交复审；不把core→UI直引接回、不以no-ui单面通过外推。本轮不修实现、不跑整档、不发布canonical、不merge或清理。

2026-10-15 R2-repair：stage=work | PLAN-779 | r1 | outcome=pass（F779-R2-01 修复完成，送 R3 复审）| worktree=D:/autostack/.wt/lang-779/auto-lang | branch=plan-779-dev | base=2253134cb | code=45203b5ca（链：…→42905b4b5→45203b5ca）| task_ids=T-03/T-04/T-05（重开项重勾，5/7）| blockers=无

**装配覆盖根因**（连续两轮 final 失败的共同根）：R1 把"兼容默认装配"实现为**无条件覆盖式 set**（`install_process_default`→`set_process_default_frame_provider`，后注册者胜）——R1-F01 补接线时该覆盖语义把调用方显式默认在 `AutoVM::new` 快照前替换掉（UI 开时显式 A 17/31 → 0/0）。即：装配**覆盖面**（哪些入口装）与装配**优先关系**（谁赢）是两个正交维度，R1 只修了前者。

**设计调整（优先关系契约，复审指示）**：(1) 显式 `set_process_default_frame_provider`（权威 API）恒胜；(2) 隐式/内建兼容安装只补**空槽**——新 `set_process_default_frame_provider_if_absent` 检查+安装同写锁临界区原子完成（无 check-then-install 并发窗口）；(3) 实例级 `AutoVM::set_frame_provider` 在其上独立生效（快照+覆盖）。`ui::frame_provider::install_process_default` 改走 if-absent——全部隐式入口（VmBridge×2、run_file_dynamic_ui_inner、install_ui_host_capabilities 漏斗、auto CLI main）经此单一语义，无 per-site 特判；强制替换仅显式 API。core 零 UI 直引、no-ui 缺席 0 不变。

**覆盖矩阵（UI 开/关 × 高层/原始 VM × 无默认/显式默认/实例覆盖）**：

| 构建 | 入口 | 默认槽 | 期望 | 覆盖锚（45203b5ca 实跑） |
|---|---|---|---|---|
| UI 开 | 高层 run_with_capture | 显式 A(17,31) | stdout 17/31 | **复审 ui-consumer.py harness 原样重放 6/6 exit0**（原 4/5 fail 格）+ in-crate high_level_compat_default_respects_explicit_choice |
| UI 开 | 高层 | 显式 A→B 替换 | 43/59 | in-crate 同测 (b2) |
| UI 开 | 高层 | 空 | 兼容补缺；门关 0 / 门开真值 | in-crate 同测 (a)（0+is_some）+ plan716 frame_program_track_gate_on（门开真值 1/1）|
| UI 开 | 原始 AutoVM | 快照/实例覆盖 | 实例独立 | in-crate merge 测试（实例槽）+ fixture v01(b)（no-ui 形）|
| UI 关 | 高层 | 显式 A/B | 17/31、43/59 | fixture v01(a)（manifest 6/6）|
| UI 关 | 高层 | 空 | 0 | fixture 新增 v02_high_level_absent_default_yields_zero（模式不变断言）|
| UI 关 | 原始 | 空/显式/实例 | 0/显式/实例 | fixture v01(b)/v02×2/v03 |
| （N/A）| 高层 × 实例覆盖 | — | 高层入口不暴露 VM 实例，实例覆盖属 AutoVM 层 | 设计性缺席，非合同缺口 |

**回归面（45203b5ca）**：复审 harness exit0 6/6（head 绑定收据）；fixture no-ui 6/6；in-crate plan779 4/4 + frame_provider 6/6；plan716 门关 14/14 + 门开 nextest 14/14；门开程序轨/bridge 独立复放各 1/1；plan755 4/4；frame 族 66/66；check(auto-lang/auto) 0 错；触面文件 rustfmt --edition 2021 --check 全净。

**注记**：fixture 独立 manifest 的 ui-iced 形态被无关的独立锁解析破裂阻断（fresh lock 解析出双 windows-core 0.57.0/0.62.2 → wgpu-hal 宏边界 E0277；两次钉扎未果）——UI 开的**外部**形态以复审自带 workspace `--locked` harness 覆盖（上表第一行）；in-crate 默认构建（ui-iced on）为同路径常驻回归锚。此依赖解析问题不入本 Plan 范围，记录在案供 fixture 基建后续跟进。

merge-owned pending 不变：最终 review 通过后冻结提交裸 `cargo t` + `cargo tv`。
next_responsible_role=review（R3：bind 45203b5ca；重点复核优先关系矩阵与 ui-consumer 重放）

2026-10-15 R3：stage=review | review_scope=final | reviewer_identity=本规划/R1/R2评审会话，未实施779 | plan_revision=1 | outcome=pass | worktree_path=D:/autostack/.wt/lang-779/auto-lang | branch=plan-779-dev | reviewed_commit=45203b5cad61680097bc9f39590868bac03c816a | base_commit=2253134cbd8b6e4a6273bf92f2145c3f34ce8934 | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；锁/受管源码最终复核见指纹 | spec_inputs=vm/overview SHA256 0a892771a4e9d4902641a78a5911741498264238254e17ea126a866f24e8e144；ui/overview cde75f5c73817c7c8e68108f9200be852ed600912286c13a08a56e2c65c0caa5；SD01/02已reviewed冻结 | readiness=两连败根因及装配覆盖/优先关系设计、永久跨feature矩阵已核验 | acceptance_results=AC01/02/03/04 pass（标量pilot范围）、AC05 merge-owned pending | findings=无新增阻断；F779-R1-01/02、F779-R2-01全部closed | evidence=[R3报告及闭合表](attachments/779/review-r3/REVIEW.md)、[同源UI consumer](attachments/779/review-r3/ui-consumer.json)、[独立优先关系控制](attachments/779/review-r3/priority.json)、[冻结SD](attachments/779/review-r3/spec-delta-reviewed.md)、[最终指纹](attachments/779/review-r3/fingerprints.json) | verification=no-ui consumer6/6、UI consumer6/6、plan779 4/4、plan755 4/4、默认门态plan716 14/14、单独门开程序轨/bridge各1/1、check/fmt/diff-check通过；独立优先关系四组控制全部通过（重复补缺/显式替换/旧VM快照与实例覆盖/并发显式set/空槽稳定）；core零选定UI直引，a2r保留臂不变 | limitations=work报告独立fixture新UI锁解析冲突未在本轮复现；本轮UI外部证明用workspace锁定库公开API，不冒称新UI锁解析成功；列入后续跨仓发行依赖身份验证 | task_state=T01..T06完成，reviewed 6/7，r1不变 | pending_closeout_gates=merge在冻结提交裸cargo t+tv通过后SD01/02 canonical/ledger沉淀、合流/归档、批到期判定/guard-clean清理；未触UI生成器/aavm/trans，不新增tu/taa/tt | next=merge；review pass不等于交付完成，不可跳过合入门禁。本轮未跑整档、不改实现、不发布canonical、不merge或清理。

2026-10-15 merge：stage=merge | PLAN-779 | r1 | outcome=pass（进行中——landed/ledger 已验，archive/cleanup 见后续行）| delivery_commit=0bb1e3709 | reviewed_commit=45203b5ca（两轮 rebase 映射：45203b5ca→1868e522d→bd575b5c7；全链 `git range-diff` 六提交 `=` 补丁等价，第二轮增量=master 纯 docs/781 簿记，t/tv 面字节等价故门禁证据按输入未变复用）

| checkpoint | 证据 |
|---|---|
| prepared | 冻结 SD（review-r3/spec-delta-reviewed.md）落 worktree：vm/overview+ui/overview（73cdf67c3）；ledger 投影 v23→v24（0bb1e3709）；收据报告 docs/reports/779-frame-provider/verification.md |
| gates | 冻结提交 45203b5ca：裸 `cargo t` 5364/5364（111.3s）+ `cargo tv` 163/163；delivery f24d61dcb 复跑：t 5364/5364（104.2s）+ tv 163/163；无 tu/taa/tt 触面 |
| landed | master tip=0bb1e3709=plan-779-dev（ff-only，无 merge commit；首轮拒绝后按规程二次 rebase 于 1b4c0728d 再 ff） |
| ledger_refreshed | `.autoos/specs.json` v23→v24 守卫式离线投影（通道=无 store writer 可达；backup sha256 16a0f22d92043569；首轮 indent 失配被 diff 守卫拦截后还原重投；终态 diff=+75/−1 仅 5 项+version；schema 六节 id 精确、字段完备、读回验证）；P779-1/2(designs)/P779-T(tests)/P779-R3(reviews)/P779-M(reports) |

## 10. 待澄清事项

T01遇到公共Core既有行为改变先提出差异/最小修订；不能缺注入却返回0假过。更大host接口、media handles/async生命周期另立Plan。

（work 轮无新增待澄清——T-01 未发现需改变既有公共行为；缺席=0 与原语义逐值等价。）

### Spec 增量落点提案（merge 按实情沉淀，AC-05）

- SD-01（docs/specs/auto-lang/vm/overview.md）：native 装配节增补——frame 族 9918/9919 为宿主能力 pilot：core shim 经 `vm::host_capabilities::FrameProvider`（NativeInterface per-VM 槽，`AutoVM::new` 快照进程默认 + 实例 `set_frame_provider` 覆盖；注入不 bump binding_revision/不撤 proof）；无 provider=未捕获 0；ID/编码/canonical 不变。未迁族与 a2r 保留臂明示（本计划 §8 清单）。
- SD-02（docs/specs/auto-lang/ui/overview.md）：frame provider 生产注入者与时点——`ui::frame_provider::install_process_default()`（frame_bench adapter）注册于 VmBridge 构造、`run_file_dynamic_ui_inner`（auto run UI 入口）、auto CLI main（cfg ui-iced）；库消费者自驱 iced 循环须自行注册（缺席=0）。
