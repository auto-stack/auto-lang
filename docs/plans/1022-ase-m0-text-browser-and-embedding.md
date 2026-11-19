---
plan_id: PLAN-1022
status: execution_done
feature_name: ASE M0两页文字浏览器与公共嵌入
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-19
plan_revision: 2
status: reviewed
current_step: 8
total_steps: 9
depends_on: [PLAN-1021, PLAN-967, PLAN-969, PLAN-978, PLAN-979, PLAN-980]
supersedes_spec_components: []
new_spec_components: [docs/specs/ase/m0-text-browser.md, auto-ui:docs/specs/runtime/ase-m0-frame-consumption.md]
touched_goals: [GOAL-002, GOAL-007, GOAL-010]
---

# [PLAN-1022] ASE M0两页文字浏览器与公共嵌入

## 0. 变更摘要

### 2026-11-19 · 执行轮（work pass）

T-01..T-07 已交付（execution_done）：同宿主 frame 先证 **GO**（机制=全局 HOST session 轮换 + 单窗口渲染 current_session + 适配器独占桥轮换强制先关旧 owner，不装 980 全局桥槽）；两仓提交 lang `02adfe896` / ui `248b2bf4`（组 `D:/autostack/.wt/lang-1022/`，auto-web/auto-down detached 兄弟）；`cargo t ase-m0` 本树 PASS 7/7（V-06）；`ase_m0_text` 目标 7/7（CPU 6 + 排窗 e2e 1，全部 ≤10s）；resize/焦点 OS 级探针 PASS。冻结合同/证据见 §9。下一阶段 review（T-08）。

### 2026-11-19 · 980/981 后继接口

本包从 980 已交 DIALECT/NATIVE 取实际使用的嵌入接口；如需新增完整 AV 组件行，则按使用项消费 [1071](1071-autoui-w1-av.md) 正式来源。嵌入和浏览器用例沿本包合同。

r2，drafting，0/9；用户只授权按M2-A已交来源更新计划，不启动实现。1020/1021已归档，F0/H0/E0/B0/HELLO及MIN-A-UI已有正式交付。本包的下一入口是T-01重核正式来源、1021当前兼容性与同宿主公共frame先证，不再等待上述基础接口实施。保留9任务与全部AC/20轮原验收，尚未运行。路线：[ASE M0](../design/strategy/ase-m0-delivery-roadmap.md)。

## 1. 目标

一个原生验证窗口，提供打开A/B、重新打开、关闭、错误/当前地址；A/B内容由独立普通AutoUI源经ACE AV产生。验证导航代次、改源、失败重试与真正frame嵌入。

不复制980 Hello的FFI/Schema执行器，不使用1021低层路径代替E0，不建设完整旧引擎浏览器；不包含业务按钮/保留状态、多窗口、3D、后台服务、全量Gallery或默认切换。

## 2. 架构方案

ASE目录/导航状态→UI owner的规范Node受检入口与ACE公共执行/授权接口→MIN-A-UI文本提交/关闭→同一宿主frame。优先评估`ui/ace_node_desktop.rs::NodeDesktopBridge`的`load/open/close_session`等公共面，不复制其FFI/解析/窗口。通用frame adapter仍待T-01先证，不宣称现已支持ASE嵌入。Rust/Iced只提供测试控制与现有投影；子页面文字来自普通Node源的AV真值。旧HELLO的F0字节事实与实际UI-FFI效果分开记录，`text_byte_len`成功不充UI dispatch。

## 3. 技术栈

复用已交`experimental/ase-m0/`目录/源身份等可共享部分，不改1021历史冷执行路径为产品前门。已核UI owner为`ui/ace_node_desktop.rs`、`ui/ace_hello.rs`与`ui/iced/renderer/{host,projection}.rs`；旧VmBridge只作参照，不伪造旧桥。按现有`ace-bridge` feature装配，关闭时具名拒绝。新薄适配位置/API在T-01冻结，无Core→UI/ASE反依赖。

## 4. 需求分析与背景调查

2026-11-17用户授权更新本计划、1023及ASE路线：明确已交接口、剩余依赖与真实会话接线，保留1020/1021归档，**先不启动实现**。本轮不建树、不改源码/fixture/Spec、不构建/复验/GUI/探针、提交/合入/发布，不接管上游状态；未指定模型/工期/token预算。

取材[Specs overview](../specs/overview.md)、[ASE已验冷加载](../specs/ase/m0-headless-loading.md)、[M2-A认证](../specs/ace-migration/m2-certification.md)、[UI已交接口§7/12](../specs/ace-migration/m2-autoui.md)、[执行前门](../specs/ace-migration/m2-facade.md)及[源码重放规范](../specs/auto-test-case/ace-source-only-replay.md)。部分Spec头部/计划勾选滞后，交付按已验正文＋正式收据/落地身份判断，不因旧“未交”无限等待，也不因归档推断任意profile支持。本包新Spec仍待真实交付。

本轮读取auto-lang HEAD=`f60266a99f53ca154d8caa717519aecbae2baef3`、auto-ui HEAD=`69e64d584f5eb62116604dba8d2da708acec9ab3`及并发文档WIP，开工重核。Spec SHA256前16：m2-certification=`045340642830e6b0`、m2-autoui=`bf01874c6fa2e8ba`、m0-headless-loading=`b92dc30ee43c202c`。具体来源锚：1021交付`af35aa318`；980 HELLO fold #4为lang `1991021a5`/UI `e529dde6`，MIN-A-UI fold #7为lang `7f431bd5f`/UI `403c936d`。T-01核全SHA/祖先/Schema/ABI，不将这些来源当本包测试结果。

已核`NodeDesktopBridge`生命周期及`ui_open/ui_apply_*/ui_poll_event/ui_close`真实UI-FFI。980的单窗/全局bridge不证明一个ASE宿主能轮换独立应用，该消费接缝仍待本包。原r1“上游尚为草案”的判断保留在§9历史摘要，不作当前门。

## 5. 详细设计

### 接口门与有界先证

frontmatter来源owner并集保持，不是等待全部B阶段的AND门。当前来源与剩余条件如下：

| 来源 | 已交可消费面 | T-01仍须关闭 |
|---|---|---|
| 1021、960 | 注册身份/有界源闭包/公开AV冷执行 | 当前ACE包链已变，专属树以现有完整`cargo t ase-m0`定向复验一次，不重开1021 |
| 967/F0、969/E0、978/B0及归档扩面 | 受检FFI、执行/授权/关闭、宿主闭包 | 核所用profile/ABI/SDK与正式来源；有效证据可复用 |
| 979/H0、980/HELLO | Node文本前门、字面量/函数文本与真窗 | 核正式fold，旧T-HELLO-D勾选滞后不充缺交付；不代上游改簿记 |
| 980/MIN-A-UI | 公共Node宿主候选及实际文本提交/关闭 | 单窗≠公共嵌入，**同宿主frame先证仍未执行** |

不新增等待M2-B全完/AGE整包的门；M2A-08作为当前背景与身份记录，不代本包验收。实际来源缺失/不兼容须具名阻塞consumer与原owner，不能切回1021低层冷链或旧VM。

T-01在重核/复验后最多比较两种公共视图消费机制，在**同一个真实宿主窗口**的frame先显示A、关闭、显示B，冻结app→源→AV值→UI session→frame所有权。每页重启完整窗口/第二Iced事件环不能算嵌入；不能覆盖全局bridge后遗留旧owner。若只能合并源码、伪造旧桥或重写renderer则needs_replan并停T-02以后。UI薄消费适配归本包，语言/FFI核心缺口交原owner合同承接，不重开归档计划。

两份页面以**规范node根**表达中文/ASCII不同文本与普通函数返回文本，换合法根/函数名及改文字仍工作；widget只按979显式兼容配置，不另建传统UI线。T-02创建拟`experimental/ase-m0/fixtures/text-{a,b}/`应用输入，UI显式路径消费，不复制权威。T-03创建拟`auto-ui/crates/auto-ui-runtime/tests/ase_m0_text.rs`与原生验证入口，T-01先冻结adapter/API/recipe；这些目标当前尚未创建。

导航世代属于ASE控制面，每次打开/重开/关闭推进；受理当前代次结果才挂载，旧完成被拒绝并释放owner。无guest异步时可在宿主控制面延迟一次真实装载结果验证陈旧拒绝，不以此冒通用异步执行。关闭后空frame，重开重执行；不承诺保留AppSession或缓存零编译。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ase/m0-text-browser.md | 已验headless→当前Node/公共执行/UI-FFI消费、导航/关闭与兼容边界 | M0-T可复核，不冒单窗即嵌入 | AC-01..05 |
| SD-02 | add | auto-ui: docs/specs/runtime/ase-m0-frame-consumption.md | 无ASE公共消费约定→实际薄适配/资源与输入owner，仅记录已验范围 | 公共frame接缝归UI | AC-02/04/05 |

### 实施树与材料

唯一组`D:/autostack/.wt/lang-1022/{auto-lang,auto-ui}`，主分支`codex/plan-1022-dev`，每仓唯一树与显式锁/SDK身份；先读UI当地规约。正式源码不落master，Plan簿记留主检出。scope `D:/autostack/plan-artifacts/plan-1022/<repo>/`建立即登记owner/用途，截图/日志/构建副本不散落。

## 6. 测试设计

测试与oracle以下均为拟议、未执行；新增测试/CLI入口在对应实施任务创建，不能在创建前报告PASS。实现与内部检查由执行者承担，最终独立review采用独立上下文、同一权威树；本轮未指定模型/人员，不创建agent/chat。

语言仓只按真实触面选择完整固定组及必要消费者，参考现存`ace-av`、`ace-source`、`ace-artifacts`，不为单一外部消费者机械跑`cargo t ace`全并集或`cargo t all`。若改ACE的源码/共享语义须补相应完整组与下游理由；纯ASE消费者不修改ACE时以新增ASE独立目标为主。UI只跑新增/受影响命名目标与公共接缝必要集成，按本仓入口，不沿用语言仓退役别名；原生/GPU/焦点单实例，自动注入与真实用户输入分账。工作迭代按反例/完整职责面，最终review后冻结提交验证受影响并集，身份未变证据可复用。

r2验证选择：现有`cargo t ase-m0`由T-01在专属树先复验，失败先定位自身/上游/环境，不重跑整个1020或重开1021。`ace-facade`为当前公共执行owner固定组，涉及其改面才补完整组；其余按实际触面。正式性能只显式`cargo t ace-perf`，不附加本包；Windows不跑Linux/WSL腿，具名排除归专用Linux环境，不充PASS。以下新case由T-06创建，本轮未运行。

| Case ID | AC / invariant | 输入/入口 | oracle与边界 |
|---|---|---|---|
| V-01 | AC-01 正式来源 | T-01核1021及F0/H0/E0/B0/HELLO、实际采用的MIN-A-UI身份 | 已落地收据/祖先与Schema/ABI匹配，旧proof按MISS/拒绝；不因历史未勾或整包executing误判缺接口 |
| V-02 | AC-02 独立真实UI | 同宿主frame打开两份node源、函数文本/换名 | AV值→实际UI-FFI提交→画面/当前frame对应；Rust硬画、仅字节事实调用或每页重开窗口均不算成功 |
| V-03 | AC-03 独立改源/恢复 | 改A不改B，非法A→修复，未知app | A变化/B不变；非法不半挂载，修复后真实重开；未知身份零执行 |
| V-04 | AC-04 导航代次 | A迟到→B、关窗后迟到、A→B→A | 旧结果拒绝不覆盖新frame且owner释放；受控宿主延迟不冒guest async |
| V-05 | AC-05 生命周期/呈现 | 有效resize/焦点、打开关闭20轮 | 当前内容按新frame呈现、关闭空frame；存活owner/目标不随轮次无界增长，所计资源实源，不假称驱动显存归零 |
| V-06 | AC-01/AC-03 当前兼容 | 在本包树跑1021既有完整ase-m0目标，再接当前Node源 | 11/22/12、拒绝/修复/资源原oracle不减；与1021历史PASS分账，不用旧编译产物或历史日志冒当前复验 |

后续T-01在语言唯一树跑现有`cargo t ase-m0`。UI已有参考回归为`cargo test -p auto-ui-runtime --features ace-bridge --test ace_node_desktop -- --test-threads=1`与`ace_ui_hello`目标，仅适用改面需跑时选择。拟新增完整入口`cargo test -p auto-ui-runtime --features ace-bridge --test ase_m0_text -- --test-threads=1`由T-06创建；排窗flag/预构建recipe由T-01按实际owner冻结，T-05记录可重放命令，不能把已有CLI引擎选择自动当ASE入口。零图形改面不跑完整AGE；真实原生交互缺环境保持pending。

## 7. 验收标准

| ID | owner_stage | 验收与方法 |
|---|---|---|
| AC-01 | work | 当前正式来源/1021兼容核齐，T-01同宿主公共frame先证通过，V-01/06 |
| AC-02 | work | 两独立普通Node源/函数文本经真实AV与UI提交至同宿主frame，无legacy/硬编码，V-02 |
| AC-03 | work | 单源修改与非法/修复/未知app处理，V-03 |
| AC-04 | work | 打开/重开/关闭代次及迟到拒绝不覆盖新页，V-04 |
| AC-05 | work | 有效resize/焦点、20轮关闭释放与真实窗口结果，V-05 |
| AC-R | review | 独审源→受检执行→UI owner→实际画面与清理；不拿Hello代嵌入 |
| AC-D | merge | 两仓正式来源/SD-01/02及冻结定向并集齐，归档与路径对账清理 |

## 8. 执行步骤

- [x] T-01｜work｜已交1021/F0/H0/E0/B0/HELLO/MIN-A-UI；后续须实施指令｜核正式来源/跨仓树/scope，跑当前`cargo t ase-m0`（V-06）；最多两机制同宿主frame先证并冻结adapter/API/原生命令；AC-01。no-go停T-02以后，本轮不执行。[✅ 2026-11-19：7锚点提交全为两树祖先；Spec哈希2/3同r2、m2-autoui=857ceb6a（PLAN-1072 §15已解释增量）；`cargo t ase-m0`本树PASS executed=7 run-dlqhk5zd；同宿主frame先证GO（CPU+真窗双面）；机制/adapter/API/原生命令冻结，见§9合同]
- [x] T-02｜work｜T-01先证通过｜在ASE实验工程加两独立普通Node文字源/描述，换根名与函数文本；AC-02、AC-03，核源码闭包。[✅ lang 02adfe896：fixtures/text-{a,b}/main.at（TextPageA中文/SecondTextRoot ASCII，字面量+函数双叶）+README消费描述；单文件自包含闭包；换名/改字V-02反硬编码用例绿]
- [x] T-03｜work｜T-02｜UI owner薄消费适配与同一原生宿主，控制按钮只发导航；复用公共执行及MIN-A-UI文本/关闭，防全局bridge覆盖旧owner；AC-02，V-02。[✅ ui 248b2bf4：ase_m0_frame.rs薄适配+AseM0TextWindow（地址/错误/导航按钮只发导航/frame区渲染current_session）+ase_m0_text_window.exe；纯980公共面，不装全局BRIDGE槽；V-02 CPU+排窗e2e绿]
- [x] T-04｜work｜T-03｜导航代次/取消/错误/修复与关闭owner；AC-03、AC-04，V-03/04。[✅ begin_open/mount票据代次协议；非法具名拒不半挂载、修复同入口、未知key零执行代次不动；迟到整票拒绝零UI效果；reopen先关旧代（V-05首轮抓到的真实泄漏已修）；V-03/V-04绿]
- [x] T-05｜work｜T-04｜实际窗口有效resize/焦点/20轮生命周期，核文本与dispatch来源；AC-05，V-05。人工原生与自动注入分列。[✅ 自动OS注入档：resize-focus-probe.ps1两次真实SetWindowPos+SetForegroundWindow，41帧持续渲染、内容=AV真值、0错误；20轮开/关CPU档全closed零存活；人工原生档待人工走查（排窗单实例）]
- [x] T-06｜work｜T-05｜创建完整ase_m0_text目标并验V-01..06，结合T-01兼容复验及必要公共接缝集成；身份未变证据复用，未跑格不PASS。[✅ tests/ase_m0_text.rs 7/7（CPU 6+排窗1，0.92s热/2.16s窗档）；V-06另跑ase-m0组PASS；980既有ace_node_desktop/ace_ui_hello为非改面参考回归（本包未改其触面，证据复用）]
- [x] T-07｜work｜T-06｜告警/格式/调试输出检查，折§9摘要、Spec delta与1023交接；AC-01..05。[✅ 新文件rustfmt clean、零编译警告（crate既有150+告警为存量非本包触面）；无调试输出（eprintln仅CLI用法/计时行）；§9收据+SD表+1023交接见§9]
- [x] T-08｜review｜T-07｜独立核原源/AV/公共嵌入、陈旧释放及上下游边界；AC-R，不重复接管980。[✅ 2026-11-19 终审 pass（独立子代理上下文；AC-01..05 全 pass、无 blocker、F-1..3 全 non-blocking；收据见§9）]
- [ ] T-09｜merge｜独审pass｜冻结相关完整目标/必要组，两仓SD正式落地、归档；全输出目录对账折摘要清scope，以`pwsh -NoProfile -File D:/autostack/wt-guard.ps1 <每仓工作树>`确认clean后清树；AC-D。

## 9. 复审记录

stage: new/revision；PLAN-1022 r2（2026-11-17）；outcome: pass（来源/消费合同可交接，非兼容/嵌入测试PASS）；next: 未来取得实施指令后T-01当前兼容复验与frame先证。基础接口已交，不再沿用r1等待；公共frame可行性仍待实证。0/9，运行/测试/review/merge全部未执行。

历史r1（2026-11-12）：当时接口未齐，stage:new/outcome:blocked，0/9；当时的静态结构检查通过，不是实现证据。旧读取基面lang `7b2f2e959`/UI `ad9c05da`及“无ASE规范、上游尚草案”仅作当时观察，已被本r2 §4–8取代；未删除原目标/AC。r2保留T-01..09/9项全未勾，不重开或修改1020/1021归档；本轮仅文档检查，未生成测试/探针或运行产品。

r2静态预检（2026-11-17）：两Plan原19任务/15个AC标识与全未勾保持，新增V-06兼容复验及1023的V-07..10映射到原AC；三个文档25处本地链接、章节/围栏/计数/阶段DAG/前置身份与空白检查通过。1020/1021归档、已验ASE Specs和所读M2 Specs及取号计数器内容hash与读前相同。只证明文档结构与交接一致，不是当前兼容性、嵌入或运行review PASS。

stage: work | PLAN-1022 | r2 | 2026-11-19 | outcome: pass（execution_done）| worktree: D:/autostack/.wt/lang-1022/{auto-lang@02adfe896(base b2d830a2c), auto-ui@248b2bf4(base 67893861), auto-web@26d085f(detached), auto-down@72b0970(detached)} | branch: codex/plan-1022-dev | task_ids: T-01..T-07 | blockers: 无 | next: review（T-08，独立上下文）

**T-01 来源核验（V-01）**：7 锚点提交（1021 `af35aa318`、HELLO fold lang `1991021a5`/UI `e529dde6`、MIN-A-UI fold lang `7f431bd5f`/UI `403c936d`、r2 读取基面 lang `f60266a99`/UI `69e64d584`）经 `git merge-base --is-ancestor` 全为两树 HEAD 祖先。Spec SHA256 前16：m2-certification `045340642830e6b0`、m0-headless-loading `b92dc30ee43c202c` 与 r2 一致；m2-autoui `857ceb6a33ec4dae`＝PLAN-1072 merge 登记的 §15 哈希（已解释增量）。ABI/Schema：`NodeDesktopBridge` 公共面（load/load_package/open/step/close_session/reopen、ui980 七函数、Ui980Hooks 表、instance*1000+slot 打包、负值具名状态码）逐一读源核实在 HEAD；H0 函数文本源形态=examples/ui/ace-hello-minimal/src/front/app-body.at。

**T-01 同宿主 frame 先证（GO）与冻结合同**：
- 机制裁定（两候选中取一）：**M1 = 全局 HOST session 轮换 + 单窗口组件渲染 `current_session` + 适配器独占持桥轮换（切换前强制 `close_session` 释放旧 owner），不安装 980 全局 BRIDGE 槽**。M2（每页独立窗口/第二 Iced 事件环）被计划红线排除（不算嵌入），未实施。`load_package` 面存在但单模块文字页不需要（裁定记录，未消费）。
- 冻结 API：`AsePage{key,path}`（显式路径消费）/`AseFrameHost::{new,begin_open,mount,open,close,reopen,step,install_script,evidence,nav_generation,current_key,pages()}`/`AseLoadTicket`（代次票据，陈旧整票拒绝零 UI 效果）/`AseM0TextWindow`+`install_ase_host/with_ase_host`。
- 冻结所有权链：app key → 源路径（显式）→ `NodeDesktopBridge::load`（AV 真值文本）→ `open()`（ui980 真实 FFI 建 session、HOST current 切换）→ 单窗 frame 渲染 current_session；导航代次只在受理指令（open/reopen/close）推进，未知 key 拒绝不动代次；reopen 先关旧代再开新代（修复首轮 `bridge.reopen` 不关旧代的存活泄漏——V-05 oracle 抓获）。
- 冻结原生命令：预备档 `cargo build -p auto-ui-runtime --features ace-bridge --example ase_m0_text_window` + `ase_m0_text_window.exe --prep-only`；运行档 `--page a=<path> --page b=<path> --evidence <json> [--script "t:open:k|close|reopen"] [--hold-ms] [--min-frames]`；测试档 `cargo test -p auto-ui-runtime --features ace-bridge --test ase_m0_text -- --test-threads=1`（排窗 `AUTO_UI_ASE_M0_E2E=1` 单实例）。fixture 根可用 `ASE_M0_TEXT_ROOT` 覆盖（默认组内兄弟树 `../../../auto-lang/experimental/ase-m0/fixtures`）。

**本会话实际测试**（2026-11-19；机器占用检查 5 次，其中 2 次让行他计划构建/回归后择窗启动）：

| 测试模块/实际范围 | 结果与执行数 | 准备/构建 | 运行 | 总墙钟 |
|---|---|---|---|---|
| auto-ui `ase_m0_text`（CPU 6 案，串行） | 7/7 PASS（含排窗位返回；热缓存 0.92s） | 首建 6m10s（全仓冷）+重编译 1m23s | 0.92s | 见左 |
| auto-ui `ase_m0_text` 排窗 e2e（单窗轮换） | 1/1 PASS | 示例重建 55s+42s | 2.16s | 2.16s |
| auto-lang 固定组 `cargo t ase-m0`（V-06 复验） | PASS executed=7 run-dlqhk5zd，config_sha `718592f480c3d…` | 冷编译含于总时 | — | 1m22.6s |
| resize/焦点探针（OS 级 SetWindowPos×2+SetForeground） | PASS：41 帧、内容=AV真值、0 错误 | 示例复用（0 增量） | ~9s hold | ~11s |

新增 case 计时（全部 ≤10s 门）：

| Case ID | 行为/oracle | 实测 |
|---|---|---|
| p1022_v01 | 真实fixture显式路径装载，中文/ASCII AV真值字节级一致 | 232.7ms ✅ |
| p1022_v02 | 同宿主轮换A→空→B，文本经真实ui980 FFI | 47.9ms ✅ |
| p1022_v02r | 换合法根/函数名/改文字仍工作（反硬编码） | 19.7ms ✅ |
| p1022_v03 | 改A不改B、非法具名拒不半挂载、修复重开、未知零执行 | 68.2ms ✅ |
| p1022_v04 | 代次：A迟到→B拒收零UI效果、关后迟到拒、A→B→A | 132.7ms ✅ |
| p1022_v05 | 20轮开关全closed零存活owner、重开重提交 | 537.9ms ✅ |
| p1022_window_e2e | 同一真窗≥12帧跨轮换重渲染、nav代次1..5、终态空frame | 2156.8ms ✅ |

证据复用：980 既有 `ace_node_desktop`/`ace_ui_hello` 目标本包零触面改动，按"身份未变证据复用"未重跑（review 可按需点名重放）。散落输出对账：本计划全部输出在 `D:/autostack/plan-artifacts/plan-1022/`（resize探针脚本+各运行日志），两 worktree target 为正常构建产物随树删除；未创建第二证据目录。

**主检出观察（上报不接管）**：master 两仓各带一批 2026-11-17 未提交 docs/blueprints WIP（含本计划 r2 修订）；与 1022 触面无交集，本计划簿记按文件选择性提交，未触碰他改动。

**1023 交接**：ASE 控制面/代次/关闭语义与窗口排窗 recipe 已冻结（上述 API+命令）；页内交互（页面自身事件/输入）仍属静态面外（本包页面无 model/msg），1023 交互会话消费同一 `AseFrameHost` 面并需自行先证页内事件经 ui980 回 AV 的会话接线；`ase_m0_frame.rs` 未占用全局 BRIDGE 槽，1023 可并行扩展。review F-1 顺手项转 1023：`begin_open` 源文件读失败路径补 nav_log record 簿记（现仅 Err 上抛，非阻塞）。

stage: goal | PLAN-1022 | r2 | round 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1022 | commit: lang 02adfe896 + ui 248b2bf4 | next: review

stage: review | review_scope: final | reviewer: 独立子代理上下文（general-purpose agent_fed51c57，与执行者非同会话）| plan_id: PLAN-1022 | plan_revision: r2 | outcome: **pass**（reviewed）| worktree: D:/autostack/.wt/lang-1022/{auto-lang,auto-ui} | branch: codex/plan-1022-dev | reviewed_commit: lang 02adfe896 / ui 248b2bf4 | base: b2d830a2c / 67893861 | dependency_revisions: auto-web 26d085f / auto-down 72b0970（detached 兄弟）| spec_inputs: m2-autoui 857ceb6a(§7/§12)、m2-certification 045340642830e6b0、m0-headless-loading b92dc30ee43c202c | acceptance_results: AC-01..05 全 pass（独立复跑 7 锚点祖先核验；AC-02 oracle 判别力逐条推演：硬编码/byte_len伪车道/每页新窗各有断言组合必红）| findings: F-1 begin_open fs 读失败无 nav_log record（non-blocking，路由 1023）；F-2 HOST 已关闭 session 记录随轮次累积=980 既有 late 靶设计，不违"存活 owner"措辞，裁剪义务归 980 owner；F-3 CLI --evidence 缺值 panic 非 exit(2)、view 实例交错渲染（无 oracle 依赖）。**无 blocker** | evidence: diff 逐行审阅（lang 3 文件 31 行、ui 5 文件 1169 行全新增零既有改动）；install_bridge 零引用核实（未装全局桥）；V-06 results.json（run-dlqhk5zd, executed=7, gate_passed, config_sha 718592f480c3d…）身份未变复用；rustfmt 三新文件本次实跑 clean | pending_closeout_gates: 冻结受影响并集归 merge（T-09）：ui ase_m0_text 全档 + lang ase-m0 组；人工原生 resize/焦点走查档显式 pending | 局限: 机器被 PLAN-1098 作业占用（3→6 进程，等待 60s×2 后按纪律跳过实跑），判定基于代码审阅+§9 收据+V-06 身份核验；计时数值以 work 收据为准 | next: merge（T-09）

## 10. 待澄清事项

剩余问题为当前ACE对1021兼容、公共frame/单例bridge所有权及关闭可行性；T-01用已有接口最多两机制先证，明确go/no-go与责任后才进入后续实现。不再将已交F0/H0/E0/B0/HELLO列为待实施；不等待980完整B波/AGE增强。若确需新增语义由原owner先定修订，不另写旧引擎版。
