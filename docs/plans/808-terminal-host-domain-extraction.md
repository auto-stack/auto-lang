---
plan_id: PLAN-808
status: drafting
feature_name: 终端宿主域整体解耦与会话生命周期收口
author: [Codex]
created_at: 2026-10-18
updated_at: 2026-10-18
plan_revision: 1
current_step: 0
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

| 状态 | ID | owner_stage | 依赖 | 文件/动作、交付与验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 既有provider已交付 | 穷举native/term_engine/UI/生成器及余edge；新增`attachments/808/contract-baseline.md`冻结34项、所有者/锁序/线程/feature矩阵、配置stub、实际DLL来源/能力、旧oracle和driver；有界probe确认FFI寿命及provider替换路由，未决先阻塞依赖实现 | AC-01..04 |
| [ ] | T-02 | work | T-01 | 新`crates/auto-terminal-core`owned类型/端口、句柄owner与轻consumer骨架；check/签名及无反向闭包控制 | AC-01,03 |
| [ ] | T-03 | work | T-02 | DLL加载/符号/采样/会话表/背压迁服务，原term_engine删业务；保旧缺件/配置空值；core定向V-04/05 | AC-01,03 |
| [ ] | T-04 | work | T-03 | UI terminal owned转换/input/view/palette/窗口adapter及Mux真源，保set_app_window_px门面；V-02/03与锁序反例 | AC-02,03 |
| [ ] | T-05 | work | T-04 | VM34项/实例存储/默认装配完整接线，核对已支持生成Rust组件消费；V-01与替换/两VM控制 | AC-01..03 |
| [ ] | T-06 | work | T-05 | 建plan808测试/独立consumer和实际DLL进程driver，V-01..06全矩阵含真实资源退出；记录未支持面，不能skip当绿 | AC-01..04 |
| [ ] | T-07 | work | T-06 | 777/792并集和受影响fresh/locked；删重复glue，终端/媒体剩余owner表、净迁移/SD草案及`attachments/808/handoff.md`；定向check后提交 | AC-04 |
| [ ] | T-08 | review | T-07 | 同树独立AC/SD/34项全链/真实DLL及UI生命周期审查，终端外边清单完整；记录merge门pending | AC-05 |
| [ ] | T-09 | merge | T-08 pass | 合流共享装配/manifest并集，必要复审后最终冻结门；失败归因修复并复审重冻 | AC-06 |
| [ ] | T-10 | merge | T-09 | 规范/ledger沉淀、归档/发行输入/owner交接、guard clean后工作树清理及计数 | AC-06 |

handoff：`D:/autostack/.wt/lang-808/auto-lang`，分支`codex/plan-808-dev`；唯一树由work/review/merge顺序使用。actual base/DLL路径/组内兄弟依赖由work记录；不在worktree内建symlink/junction，不为本计划新建auto-term实现树。

## 9. 复审记录

2026-10-18 revision 1 drafting；仅合同预检，尚未实施或独立review。阶段结果须逐次绑定实际SHA/平台/DLL身份及原始失败/退出证据，不能将“函数返回”替代资源退出。

## 10. 待澄清事项

无现在需用户决定的阻塞项。T-01有界决定FFI/会话owner及支持面，实际DLL缺失则可继续不依赖它的任务但最终验收pending。若必须改auto-term ABI/相邻仓、取消旧共享句柄/原生支持或扩张a2r引擎能力，应具体重规划，不擅自将本Plan变成整套终端重写。媒体及i18n原残边不在本Plan，交接清单保留。
