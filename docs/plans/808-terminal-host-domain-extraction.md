---
plan_id: PLAN-808
status: execution_done
feature_name: 终端宿主域整体解耦与会话生命周期收口
author: [Codex]
created_at: 2026-10-18
updated_at: 2026-10-18
plan_revision: 1
current_step: 7
total_steps: 10
supersedes_spec_components: [docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/ui/overview.md, docs/specs/terminal-mux-model.md]
new_spec_components: [docs/specs/auto-terminal-core/project.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-terminal-core, auto-lang/vm/ffi, auto-lang/ui/terminal]
depends_on: [PLAN-779, PLAN-790, PLAN-794, PLAN-802]
---

# [PLAN-808] 终端宿主域整体解耦与会话生命周期收口

## 0. 变更摘要

交付R03中完整、独立有状态的终端域：34项既有term native、DLL引擎服务、会话句柄/快照/背压、输入/几何/滚动/样式投影及默认装配一起迁移。新`auto-terminal-core`持有轻合同与非UI引擎实现，VM只编解码，UI保留视图adapter；清除term_engine对UI的生产直连并验证真实多会话/释放。不以34项接口壳或fake provider视为完成。

本计划只完成R03终端子域；媒体asset/session/http、i18n等其他真实剩余edge另有owner清单，后续按完整生命周期包规划。它们未迁不阻止本计划终端验收，也不得把R03整包标完成。

## 1. 目标

- 原终端域完整生产链落到清晰的合同、非UI DLL服务、UI投影/输入adapter、VM编解码与装配层。
- 多会话不串线、一次性读取不重复消费、句柄释放归原owner、DLL/线程/子进程寿命清楚；保留VM默认缺件与无UI行为。
- 保留现有native ID/类型/堆栈顺序/缺席值；配置六项当前空集/空串保持，不新增autoterm-config依赖或配置功能。
- 不改auto-term ABI/代码，不移整个terminal renderer，不新增此前不支持的term a2r引擎绑定；已支持的Rust UI组件与VM共享投影语义须保持。

## 2. 架构方案

`crates/auto-terminal-core`（新增）定义owned行/cell/color/几何/事件/会话合同并承接libloading引擎服务；normal/build/dev不依赖auto-lang/UI/auto-man/auto-term。DLL指针和Library只留服务私有，UI cell转换由adapter完成。轻view/input端口可由UI实现；不得让VM provider返回UI核心、裸指针、VMValue或任意闭包。

VM使用终端能力端口、进程默认快照及实例覆盖；旧默认行为通过显式factory/补缺装配保留。T-01冻结进程级引擎服务与每VM/每句柄owner边界，新增provider切换不能把旧句柄误路由给新服务；不能借“per-VM”口号擅自取消旧跨调用/共享服务行为。UI仅投影引擎display_offset，不生成第二真源。

## 3. 技术栈

Rust现有libloading、Arc/Mutex/OnceLock与原DLL ABI；类型/纯转换按需迁移，不建立对相邻auto-term Cargo路径依赖。测试既用受控能力/DLL负控，也必须消费实际autoterm_core.dll与真实进程。

## 4. 需求分析与背景调查

授权与范围：用户批准模块粒度后要求本批计划；当前只规划auto-lang仓内重构，不执行，不修改相邻仓、不切分支/远端。依据[总路线](reports/v06-extraction-execution-roadmap.md)、[批次并行合同](reports/module-migration-batch-807-809.md)、[来源清单](attachments/module-batch-20261018/inputs.json)。

Specs：[VM宿主](../specs/auto-lang/vm/overview.md)、[UI](../specs/auto-lang/ui/overview.md)、[终端Mux](../specs/terminal-mux-model.md)、[绘制合同](../specs/widgets/terminal-iced-draw.md)。实际锚点：`vm/ffi/term_engine.rs`、`vm/native_catalog.rs`、`vm/host_capabilities.rs`、`ui/terminal/mod.rs`及其子模块、`ui_gen/rust.rs`（均相对`crates/auto-lang/src/`）。

现状：term_engine同时持LIB/NEXT_HANDLE/HANDLES/SNAPSHOTS/CURSORS/VIEWPORTS，加载可选palette符号、直接取UI输入/菜单/resize并喂cells/scroll/window。DLL搜索为AUTOTERM_ENGINE_DLL→宿主exe邻接→祖先target，缺件spawn=0/无效handle no-op。窗口无UI为1024×768；配置六项是有意空值stub。现有全量查询/once缓存/数值收窄先按旧臂冻结，不可在迁移中悄改。

前置provider基础已交付；不依赖806/807/809。804只在公共装配合流/全闭包时核对，不能以其未归档要求全线停工。T-01重新穷举目录而非仅搜`crate::ui`，分类已迁、终端归本Plan、媒体/i18n等未来owner。

## 5. 详细设计

### 5.1 冻结终端全集

| IDs | 既有操作 |
|---|---|
| 2943–2949 | spawn/write_line/rows/resize/interrupt/is_exited/free |
| 2957–2959 | pump_input/cursor_row/cursor_col |
| 2976–2982 | apply_resize/viewport_cols/viewport_rows/backlog_pending_mb/paused/take_alerts/dropped |
| 2983–2987 | spawn_ex/rows_for/pump_for/apply_resize_for/menu_take |
| 2988–2993 | config_profiles/default_profile/profiles_full/spawn_program/spawn_argv/spawn_cwd（VM旧空值） |
| 2998–3003 | window_width/window_height/write_raw/scroll_pending_for/pend_resize_geom/history |

T-01按catalog核实34项canonical名/签名和实现映射；新增差额不得静默遗漏。生成Rust组件已支持的onmenu/onkeydown/scroll/scheme等继续消费同份view合同；未有term引擎a2r入口如实记“不支持/不在原支持面”，不拿新fake程序声称原生parity。

### 5.2 服务、值域与生命期

- DLL加载/符号查找/引擎调用、会话句柄表、行采样、光标/viewport/backlog缓存在同一非UI服务owner；类型转换/颜色位域算法只一份。缺DLL、缺必须/可选符号、无效句柄的旧返回值逐项冻结；OnceLock环境失败缓存用进程隔离测试，不在同进程假设可重载。
- 会话free/失败spawn/interrupt/exited有明确可复跑状态转移；重复free不误释放新会话。provider替换前后句柄须能追溯原服务；两VM可隔离注入，同时默认兼容服务不无故失去原会话。禁止把Library卸载早于FFI句柄或让raw指针跨安全接口。
- 输入、resize、menu与alert等take类严格一次消费；pump_all和pump_for差异、广播/定向rows差异保留。pending scroll/resize几何钳制按旧域，不调整MIN/MAX；不持服务/registry锁回调UI或DLL导致重入锁死。
- display_offset唯一真源、history窗口anchor、程序化回声抑制和用户滚轮代数均遵守Mux；UI cells/history只是投影。线程亲和和无UI端口的空行为逐项写矩阵；不借迁移改变同步调用为异步排队。
- DLL可能持PTY/子进程/读线程：按现有free/interrupt语义做真实清理观测，证明本计划未新增泄漏。不能承诺VM drop自动杀死进程级共享会话；若现有引擎本身不保证某退出形式，具名记录其基线/归属，不能把调用返回当已reap。无法满足必要清理验收则阻塞或有界重规划，不能忽略。

### 5.3 装配、旧路径与发行

`term_engine.rs`最终只保shim/兼容转发，不保DLL加载、采样、UI取值业务。`set_app_window_px`既有生产调用保兼容门面转发到具名view adapter；宿主在既有启动入口安装，显式选择先于自动补缺。ID/catalog绑定身份不因注入改变；无必要不bump binding_revision。

迁入包的DLL是运行资源，不得Cargo反依赖auto-term；来源/ABI能力/搜索顺序/缺件语义进777/792受影响清单。独立consumer能在无auto-lang/UI checkout条件下消费core，并以受控输入或实际DLL测试。当前分发缺件fallback不被“自动搜索开发仓”掩盖。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-terminal-core/project.md | 混合glue→owned合同、非UI引擎服务与owner/缺席/资源支持矩阵 | 独立有状态域 | AC-01..04 |
| SD-02 | modify | docs/specs/auto-lang/vm/overview.md | term34项改provider编解码，默认/实例/句柄绑定及配置空值保持 | VM去UI边 | AC-01,03 |
| SD-03 | modify | docs/specs/auto-lang/ui/overview.md | terminal视图adapter/启动装配，剩余media/i18n等明确未迁 | 不虚报整域完成 | AC-02,04 |
| SD-04 | modify | docs/specs/terminal-mux-model.md | 保持display_offset/回声合同，登记实现owner新路径 | 保护交互语义 | AC-02 |

## 6. 测试设计

新增`crates/auto-lang/src/tests/plan808_terminal_domain.rs`与`test/fixtures/modularity/terminal-core-consumer/`，由T-06建立；T-01先生成旧臂矩阵与实际入口命令。以下均owner_stage=work，当前尚未运行。

| Case | AC / invariant | 输入与实际入口 | 区分性断言／覆盖边界 |
|---|---|---|---|
| V-01 | AC-01 34项/无UI | 真实VM catalog逐项、code-editor/ui组合开关及独立core consumer | 参数/返回/pop与旧臂相同；closed模式window=1024/768/config为空；无UI/语言反依赖 |
| V-02 | AC-02 定向与一次性 | 两pane不同输入/rows/resize/menu/scroll，pump_all与pump_for，take两次；真VM→UI adapter→真实终端 | pane不串、take第二次为空、几何/颜色/光标与旧迹线一致；含真实组件view投影，不只有mock |
| V-03 | AC-02 滚动真源 | 真实history scroll_offset、输入贴底、bind回声后立刻滚轮、resize无效/边界 | 对照Mux既有三段回归和生成Rust组件消费；不新增独立offset或吞用户滚轮 |
| V-04 | AC-03 生命周期 | 实际DLL spawn两会话→输出→resize→interrupt/exited/free；失败spawn/重复free/释放后读、provider替换与两VM | 句柄绑定正确、一个free不影响另一个；私有表归零/残留可解释，拥有的PTY/子进程/线程清理有观测。测试自带截止并在超时后reap；缺实际DLL环境记pending，不用fake取代 |
| V-05 | AC-03 缺能力 | 进程隔离测试DLL缺失/加载失败/缺必须符号/缺可选palette、无provider及只给core服务无UI | 旧缺件0/no-op/内置palette等逐项oracle，区分无UI≠引擎不存在；不得靠宿主开发目录意外补件 |
| V-06 | AC-04 闭包/非遗漏 | 独立fresh/locked、777 strict/受影响792，完整native→实现→adapter清单 | 非UI服务可消费、清单含runtime DLL；仅终端34项清零；媒体/其他残边有实路径和R03后续归属 |

work命令：`cargo check -p auto-terminal-core -p auto-lang`、`cargo test -p auto-terminal-core`、`cargo test -p auto-lang --lib plan808`、独立consumer manifest测试；旧term/Mux定向反例和实际DLL进程driver由T-01冻结、T-06建立可重放入口。UI交互验证若需运行应用，使用仓库autoui-verifier现有脚本并明确实际覆盖端；不因Vue无原生PTY而伪称完全等价。777使用`python scripts/modularity-audit.py --mode strict --no-cargo-check`。

review独立上下文在同一工作树检查所有AC/SD及资源失败出口。merge最终冻结提交跑裸`cargo t`+`cargo tv`；若实际改ui_gen发射算法则加`cargo tu`，纯终端adapter不机械触发tu。真实DLL生命周期是work验收必需，不能仅丢给merge。tf按主检出到期单实例批量；禁止开发/复审阶段跑整档。当前文档规划不跑Cargo/docs_gen。未指定agent/model、不创建其他会话。

## 7. 验收标准

| ID | owner_stage | 可观察验收与证据 |
|---|---|---|
| AC-01 | work | 终端34项真实VM调用改轻合同，非UI服务独立构建/消费，默认/关闭feature/配置stub保持；V-01 |
| AC-02 | work | 真UI输入/多pane投影/几何/样式/滚动/一次性消费和已有Rust生成组件保持；V-02/03 |
| AC-03 | work | 实际DLL成功/缺件/失败和句柄owner/drop/free/进程线程退出受控；V-04/05。未测环境为pending，不能凭shim或fake判完成 |
| AC-04 | work | 原glue算法与UI直连退役，777/792和独立fresh/locked通过，剩余edge逐项归属；V-06；不宣称整个R03结束 |
| AC-05 | review | 同树独立审查绑定修订、AC/SD/真实消费/失败清理通过；零新增未处理warning/格式/debug问题，merge门pending明确 |
| AC-06 | merge | 冻结合入门、Spec/ledger/归档/资源交接及wt-guard clean后清理有真实收据 |

## 8. 执行步骤

共10项，按冻结→完整终端服务/adapter→生产/生命周期验收→审查合流四阶段组织，不逐项再开Plan。

**当前状态（2026-10-18 final review R1 修复轮）**：五项 findings 已在原树修复（commit `69961e046`），T-01/02/03/05/06/07 重新勾选为 work-owned pass，T-04 原实现保留并随 adapter 生命周期辅助复验；送 T-08 二次独立复审。下方原“✅ 已完成”叙述保留为送审历史；R1 闭合证据见 [R1 报告](attachments/808/review-r1/review.md) 与 [handoff.md](attachments/808/handoff.md) §4.1。merge 门仍 pending，不得合入。

| 状态 | ID | owner_stage | 依赖 | 文件/动作、交付与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 既有provider已交付 | 穷举native/term_engine/UI/生成器及余edge；新增`attachments/808/contract-baseline.md`冻结34项、所有者/锁序/线程/feature矩阵、配置stub、实际DLL来源/能力、旧oracle和driver；有界probe确认FFI寿命及provider替换路由，未决先阻塞依赖实现 | AC-01..04 |
[✅ 已完成] 2026-10-18：34项矩阵/catalog 34条/shim 34一一对应；term_engine 49处UI直引登记；DLL=auto-term main@7a0d0e8 构建 SHA256 c17ecca1…（22导出在位、双会话隔离、kill/exit/free 探针 pass）；provider 端口/默认共享/每service句柄表/锁序冻结。收据 [contract-baseline.md](attachments/808/contract-baseline.md) + [probe](attachments/808/t01-dll-probe.log.txt)。 | AC-01..04 |
[✅ R1 补强] 2026-10-18：owner 路由（进程全局句柄命名空间 + per-VM provider 历史）、会话 lease/退役位、service Drop 释放规则写入 [contract-baseline.md](attachments/808/contract-baseline.md) §3.1/3.2/4.2/5/8；F808-R1-01/03/04 的实现合同以该更新为准。 | AC-01..04 |
| [x] | T-02 | work | T-01 | 新`crates/auto-terminal-core`owned类型/端口、句柄owner与轻consumer骨架；check/签名及无反向闭包控制 | AC-01,03 |
[✅ 已完成] 2026-10-18：`crates/auto-terminal-core`（edition 2021，闭包仅 libloading）——owned `TerminalColor/TerminalCell/Geometry/Snapshot` + `decode_style_color` 单份；`EngineService`（LIB/符号/句柄/快照/光标/视口/背压/palette）+ 34 方法 `TerminalHostProvider`；`shared()` 进程单例与 `new()/with_candidates()/without_dll()` 隔离实例。`cargo check -p auto-terminal-core` 0；`cargo test -p auto-terminal-core` 4/4（absent/load-failure/类型解码）。外部无反向闭包：terminal-core-consumer `verify-deps.sh` PASS（仅 libloading/windows-link）。 | AC-01,03 |
[✅ R1 闭合] 2026-10-18 `69961e046`：`SessionTable`（每会话门+退役位，free 恰好一次）、全局 handle 计数器、`Drop for EngineService`；`owns_handle`/`live_handle_count` 生命周期辅助（非 native）。F808-R1-01/03/04。 | AC-01,03 |
| [x] | T-03 | work | T-02 | DLL加载/符号/采样/会话表/背压迁服务，原term_engine删业务；保旧缺件/配置空值；core定向V-04/05 | AC-01,03 |
[✅ 已完成] 2026-10-18：DLL 加载/符号/采样/会话表/背压/palette 查询全部落 core；`term_engine.rs` 1394→355 行，仅 34 shim + 2 处窗口门面委托，`libloading/Library/autoterm_engine_/HANDLES/SNAPSHOTS/terminal_*` 业务零残留（plan808 core-cut 测试永久钉扎）。缺件/缺符号/无效句柄/config 空值逐项冻结；core 4/4 + plan808 absent 2/2。 | AC-01,03 |
[✅ R1 闭合] 2026-10-18 `69961e046`：DLL free 走 lease/退役路径（表锁不跨门/FFI），Drop 在 Library 析构前释放活会话；core `session_leases_serialize_free_and_never_double_free`/`different_sessions_do_not_block_each_other` 永久控制。F808-R1-03/04。 | AC-01,03 |
| [x] | T-04 | work | T-03 | UI terminal owned转换/input/view/palette/窗口adapter及Mux真源，保set_app_window_px门面；V-02/03与锁序反例 | AC-02,03 |
[✅ 已完成] 2026-10-18：`ui::terminal_provider::TerminalUiProvider`（共享 service + UI 注册表）承接输入/几何/menu/scroll/window/palette/样式旁路/window_store；`TermCell/TermColor/几何常量/palette slots` 由 core 单份 re-export（路径不变）；`set_app_window_px` 门面保留。定向 UI 测试 41/41（含 per-key 定向、once 消费、p025 滚动回归、窗口注入），plan808 real-UI 投影 1/1（真 DLL）。 | AC-02,03 |
[✅ R1 复验] 2026-10-18 `69961e046`：adapter 增加 `owns_handle`/`live_handle_count` 委托；UI 定向 41/41 复跑绿，真实 DLL UI 投影仍绿。F808-R1-01/02 的 owner/装配路径经此 adapter。 | AC-02,03 |
| [x] | T-05 | work | T-04 | VM34项/实例存储/默认装配完整接线，核对已支持生成Rust组件消费；V-01与替换/两VM控制 | AC-01..03 |
[✅ 已完成] 2026-10-18：NI `terminal_provider` 槽（Clone/new）+ AutoVM 构造快照/实例 set/get + `host_capabilities` 进程默认四级 API；34 shim 全部 provider 化（ID/canonical/弹序/返回 shape 不变）；`install_ui_host_capabilities`/`run_file_dynamic_ui_inner` 同点 if-absent 装配；无 provider 回落共享 engine（window 1024×768/config 空保留）。plan808 assembly 4/4（34 形状/快照/实例/Clone+merge/binding revision/无 provider 回落）+ real-DLL 替换路由 1/1；生成 Rust terminal 金样 3 项待合并跑（见 T-06/收据）。 | AC-01..03 |
[✅ R1 闭合] 2026-10-18 `69961e046`：NI `terminal_owners` 历史 + `terminal_owner_for`/剪枝；带 handle 的 shim 全部走 owner 解析；两 VmBridge 构造补装 adapter（F808-R1-01/02）。plan808 `provider_replacement_keeps_old_handle_bound_to_owner` + `both_bridge_constructors_install_terminal_adapter` 永久控制。 | AC-01..03 |
| [x] | T-06 | work | T-05 | 建plan808测试/独立consumer和实际DLL进程driver，V-01..06全矩阵含真实资源退出；记录未支持面，不能skip当绿 | AC-01..04 |
[✅ 已完成] 2026-10-18：`tests/plan808_terminal_domain.rs` 10/10（真实 DLL 环境：34 形状/装配/缺席矩阵/双会话/自然+交互退出/重复 free/provider 替换/真 UI 定向输入+几何+menu+scroll 单源+window 投影）；独立 consumer fixture 4/4 + verify-deps PASS；真实 DLL=auto-term@7a0d0e8 SHA256 c17ecca1…；`--no-default-features` 0（关闭 feature 缺席臂），ui-only 组合红为 F-785-P01 预存缺陷（非本计划文件）。无 DLL 运行显式 PENDING（不记绿）。 | AC-01..04 |
[✅ R1 闭合] 2026-10-18 `69961e046`：真实 DLL plan808 **13/13**（新增 F01 双活 owner 路由/重复 free/两 VM/共享回落；F04 service drop 与 provider 末引用的 PowerShell 子进程 reap；F02 两 VmBridge 生产构造；F03 core lease 控制）。F808-R1-01..05。 | AC-01..04 |
| [x] | T-07 | work | T-06 | 777/792并集和受影响fresh/locked；删重复glue，终端/媒体剩余owner表、净迁移/SD草案及`attachments/808/handoff.md`；定向check后提交 | AC-04 |
[✅ 已完成] 2026-10-18：提交 `3c02068ca`（实现/测试/fixture）+ `c346f1077`（777 基线重绑）。777：observe core_to_ui 905→860，strict **pass**（0 violations/2 grandfathered）。792：`DELIVERY_CRATES` 12→13（auto-terminal-core），driver **25/25**（fresh 141s/relocated locked 120s/负控/dev 路由；[收据](attachments/808/t07-792-driver-receipt.json)）。净迁移：term_engine 1394→355 行、UI 直引 49→2、core 1182 行新、adapter 540 行；剩余 edge owner 表 + SD 草案 + 送审收据 [handoff.md](attachments/808/handoff.md)/[spec-delta.md](attachments/808/spec-delta.md)。 | AC-04 |
[✅ R1 闭合] 2026-10-18 `69961e046`：修复后重跑 777 strict **pass**（0 violations/2 grandfathered，`r1-777-strict.json.txt`）与 792 driver **25/25**（`r1-792-driver-receipt.json.txt` / `r1-792-driver.log.txt`）；contract-baseline/spec-delta/handoff 已同步 owner 路由、lease/退役、Drop 与 VmBridge 装配；收据改为 `.json.txt` 持久入库。 | AC-04 |
| [ ] | T-08 | review | T-07 | 同树独立AC/SD/34项全链/真实DLL及UI生命周期审查，终端外边清单完整；记录merge门pending | AC-05 |
| [ ] | T-09 | merge | T-08 pass | 合流共享装配/manifest并集，必要复审后最终冻结门；失败归因修复并复审重冻 | AC-06 |
| [ ] | T-10 | merge | T-09 | 规范/ledger沉淀、归档/发行输入/owner交接、guard clean后工作树清理及计数 | AC-06 |

handoff：`D:/autostack/.wt/lang-808/auto-lang`，分支`codex/plan-808-dev`；唯一树由work/review/merge顺序使用。actual base/DLL路径/组内兄弟依赖由work记录；不在worktree内建symlink/junction，不为本计划新建auto-term实现树。

## 9. 复审记录

2026-10-18 revision 1 drafting；仅合同预检，尚未实施或独立review。阶段结果须逐次绑定实际SHA/平台/DLL身份及原始失败/退出证据，不能将“函数返回”替代资源退出。

2026-10-18 / stage: work / PLAN-808 revision: 1 / outcome: **pass（work-owned T-01..T-07 全完成，送 T-08 同树独立终审）** / worktree `D:/autostack/.wt/lang-808/auto-lang` / branch `codex/plan-808-dev` / base `2046a025b9a445d35a02da2c72a1613259a0cb60` / code `3c02068ca`（实现/测试/fixture）+ `c346f1077`（777 基线重绑），tree clean / dependency_revisions: auto-down sibling `c39ac7f` detached 只读；真实 DLL = auto-term `main`@`7a0d0e8` 构建 SHA256 `c17ecca1…`；root Cargo.lock（ignored）SHA256 `9a431eb9…` / task_ids: T-01..T-07 / acceptance: AC-01..04 work pass（34 形状/真实 DLL 生命周期/UI 投影/Mux 单源/缺席矩阵/独立 consumer），AC-05 review pending、AC-06 merge pending / evidence: [contract-baseline.md](attachments/808/contract-baseline.md)、[handoff.md](attachments/808/handoff.md)、[spec-delta.md](attachments/808/spec-delta.md)、[T-01 probe](attachments/808/t01-dll-probe.log.txt)、[777 observe](attachments/808/t07-777-observe.json)、[792 receipt](attachments/808/t07-792-driver-receipt.json)/[log](attachments/808/t07-792-driver.log.txt)；core 4/4、UI 41/41、plan808 10/10（真实 DLL）、consumer 4/4 + verify-deps PASS、792 25/25、fmt 0 diff / blockers: 无；如实登记（1）无 DLL 运行时 plan808 真实测试显式 PENDING（不记绿），review/merge 环境须以 `AUTOTERM_ENGINE_DLL` 复跑；（2）`terminal_` 过滤面唯一红 `plan705_client_total_deadline_terminal_error` 为并发负载时序 flake，solo 1/1 绿、不属 terminal 面；（3）媒体/i18n R03 残边不在本计划 / pending_closeout_gates: merge 冻结提交裸 `cargo t`+`cargo tv`、777 strict 在最终组合树重绑、792 受影响 fresh/locked 重放（若闭包变化）、Spec/ledger/索引/归档/guard 清理 / next_responsible_role: **review（T-08）**。本记录不宣称 merge 门已过或 R03 整包完成。

2026-10-18 / stage: review / review_scope: final / reviewer_identity: Codex，当前会话独立于实施上下文，未参与实现、未创建其他 agent / PLAN-808 revision: 1 / outcome: **needs_fix** / worktree_path: `D:/autostack/.wt/lang-808/auto-lang` / branch: `codex/plan-808-dev` / reviewed_commit: `c346f107790f86b7d1d045331243b3ad7f6e2c7a` / base_commit: `2046a025b9a445d35a02da2c72a1613259a0cb60` / dependency_revisions: auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，auto-term `7a0d0e8b718fb260641f0d09ada95a7346771cd3`，DLL SHA256 `c17ecca15a84f86cb26ad1f391490afe2e8b7420a369047c4bac3f6b8f0e8f58`，Cargo.lock `9a431eb9…` / spec_inputs: [完整版本及 hash](attachments/808/review-r1/baseline.json.txt)，[冻结 SD](attachments/808/review-r1/spec-delta-frozen.md) SHA256 `c8a68c441decba577c8b58f5e39df175bcf6a69d612ce7e9c16b6c1f26cdd948` / acceptance_results: AC01/02/04 partial、AC03 fail、AC05 fail、AC06 merge pending / findings: **F808-R1-01 P1** 双活服务同号句柄经真实 VM shim 误释放新服务；**F808-R1-02 P1** 两 VmBridge 构造漏装 terminal adapter；**F808-R1-03 P1** 并发 free 同柄两次进入 DLL，裸指针生命周期未保护；**F808-R1-04 P1** 独立 service drop 无释放，实际 DLL 子进程仍活；**F808-R1-05 P3** 新测试两条 warning。均为合同内待修，不作延期豁免。背压单位核查与旧实现相同，不列 finding。证据：[完整报告/AC/SD/反例](attachments/808/review-r1/review.md)、[可重放输出](attachments/808/review-r1/probes.txt)；core4/4、plan808真实DLL10/10、UI41/41、生成terminal3/3、consumer4/4、仅复制core+consumer的fresh/locked4/4、双包及no-ui check、fmt/diff、777 strict均过，但不覆盖上述反例。T01/02/03/05/06/07 重开，T04保留，executing1/10；pending_closeout_gates: 修复后最终冻结 `cargo t`+`cargo tv`、最终组合777/受影响792、Spec/ledger/index/归档/guard清理；next: 原树 work 一次修完、提交并给出 finding 闭合正负控制表，再送独立复审。本轮未修实施代码、未跑整档、未合入。


2026-10-18 / stage: work（R1 needs_fix 修复轮） / PLAN-808 revision: 1 / outcome: **pass（五项 findings 全闭合，重新送 T-08 独立复审）** / worktree `D:/autostack/.wt/lang-808/auto-lang` / branch `codex/plan-808-dev` / base `2046a025b9a445d35a02da2c72a1613259a0cb60` / code `69961e046`（R1 修复，tree clean）/ dependency_revisions: auto-down `c39ac7f` detached；DLL auto-term `7a0d0e8` SHA256 `c17ecca1…`；Cargo.lock `9a431eb9…` / finding closure: **F808-R1-01 CLOSED**（全局句柄命名空间 + per-VM owner 历史 + 所有带 handle shim 反查原 owner；双活 A/B 读/写/free、重复 free、两 VM、共享回落控制）；**F808-R1-02 CLOSED**（两 VmBridge 构造先装 adapter 再建 VM；真实构造 + bridge 自身 VM 泵输入 + 显式默认优先）；**F808-R1-03 CLOSED**（每会话门/退役位；free 摘除→等待→一次 DLL free；core 并发 lease/free 与跨会话不互堵控制）；**F808-R1-04 CLOSED**（`Drop for EngineService` 在 Library 析构前释放活会话；直接 drop 与 provider 替换末引用的真实 PowerShell 子进程 OS reap）；**F808-R1-05 CLOSED**（未用导入/多余 mut 清除，新增文件零 warning）/ evidence: [handoff.md](attachments/808/handoff.md) §4.1 闭合表、[R1 报告](attachments/808/review-r1/review.md)、core **6/6**、plan808 真实 DLL **13/13**（含 F01/F02/F04 新永久控制）、UI 41/41、no-default 0、consumer 4/4 + verify-deps PASS、777 strict pass、792 25/25、fmt 0 diff；复审原反例 replay 全部期望值命中（`r1-replay.log.txt`：ha=1/hb=2、A=0/B=1、in-flight free=0、并发 free=1、drop free=1、VmBridge true、泵 1/0、真实 child 均回收） / blockers: 无；真实 DLL 测试仍以 `AUTOTERM_ENGINE_DLL` 开启为有效证据（缺省显式 PENDING）；`plan705_client_total_deadline_terminal_error` 在并发负载下为预存时序 flake（solo 1/1 绿） / pending_closeout_gates: 二次复审；merge 冻结裸 `cargo t`+`cargo tv`、最终组合 777 重绑/受影响 792、Spec/ledger/索引/归档/guard / next_responsible_role: **review（T-08 二次独立复审）**。本记录不宣称 merge 门已过、未合入。

## 10. 待澄清事项

无现在需用户决定的阻塞项。T-01有界决定FFI/会话owner及支持面，实际DLL缺失则可继续不依赖它的任务但最终验收pending。若必须改auto-term ABI/相邻仓、取消旧共享句柄/原生支持或扩张a2r引擎能力，应具体重规划，不擅自将本Plan变成整套终端重写。媒体及i18n原残边不在本Plan，交接清单保留。
