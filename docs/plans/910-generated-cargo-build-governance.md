---
plan_id: PLAN-910
status: executing
feature_name: 生成Cargo工程的有限构建与产物治理
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-02
plan_revision: 1
current_step: 4
total_steps: 9
supersedes_spec_components: [docs/specs/repository-boundaries/generated-ui-distribution.md, docs/specs/auto-test-case/build-storage-lifecycle.md]
new_spec_components: [docs/specs/repository-boundaries/generated-cargo-build-lifecycle.md]
touched_goals: []
affects: [auto-generation-core, auto-ui-tools, auto-web-generation, auto-man-runtime, generated-cargo-storage]
depends_on: [PLAN-905, PLAN-906]
---

# [PLAN-910] 生成Cargo工程：有限构建与产物治理

## 0. 变更摘要

承接[905研究](archive/905-generated-cargo-artifact-governance-research.md)、[906交付](archive/906-standalone-language-web-boundary.md)和[Design53](../design/53-independent-build-and-generated-cargo-governance.md)。一个跨仓实施包交付：生成消费者构建配置/target归属单源化、稳定工程与成员身份、内容驱动的生成失效、并发/取消/恢复、可预览且仅针对自有闲置缓存的回收；用真实UI与Web消费者验证正确性和有限空间收益，再向UI007交接。

当前drafting r1、0/9，仅用户授权制定实施合同，暂不实施。未建worktree/运行metadata、构建、测试或容量实验，未改生产源码、用户环境、共享缓存或现行Spec。905仅研究交付；本计划才是有限治理实施载体，归档前不得称Cargo/M1已关闭。

## 1. 目标

- G1：UI Rust生成和Web Rust服务在同一受管生成构建上下文内复用兼容依赖；target与语言仓工具构建、其他worktree可写状态分离。profile/features/triple/工具链实际差异由Cargo正常区分，不强拼成同一编译单位。
- G2：不同路径的同名工程不互覆盖；源码/导入/sidecar/生成器/SDK来源等真实变化不被mtime快路径漏掉；未变输入不无故重写或重编，返回的exe/服务必须对应当前源码。
- G3：生成/编译/运行/清理各有明确owner、来源和退出路径；停止、失败、坏件后可有据恢复，不清全局缓存或误删其他会话产物。
- G4：取得同配置完整工作负载的稳态空间和周转数据，达到§5.5有限出口；交付UI007所需的路径/身份/生成接口、独立消费与迁移规则，不将新主题实现并入本包。

非目标：Kache/CoW/改卷、调整sccache、统一全机或跨worktree可写target、修改debug/优化/incremental/features来凑收益、整套Design40缓存重写、全部历史target清仓/全局自动GC、全ACE/AG/所有产品验收、远端发布。013-todo已有Rust生成错误不自动纳入；但本计划造成的回归与本合同必需验收失败不得延期冒绿。

## 2. 架构方案

主责语言仓，唯一合同在本计划；共享规则放`crates/auto-generation-core`。UI `rust_ui.rs`、Web `api_gen.rs`和Man `http_service.rs`消费同一规则，生产生成算法及owned进程继续归各原owner。不在四仓各写一套路径/flags推断器。

分开三个对象：工程的生成源码workspace；该工程的来源/成员/失效身份；同一owner内兼容消费者共用的Cargo缓存根。多个项目可以保自己的workspace/锁并复用一个受管target，不以强制合并所有源码workspace实现复用。缓存只含可重建产物，源/状态记录不混入待清根。

默认开发模式由**实际选中的producer来源/工作树**拥有生成缓存，候选落点为该来源树`target/generated-consumers/<context-id>/`（新布局，T-01核所有写入者/工具解析后冻结）；不再把生成应用直接混进语言根target。一个worktree只能使用自己的生成上下文；显式覆盖须解析真实绝对路径、标明owner/隔离边界，不能指向另一活动worktree或未认领共享根。只读安装SDK/发行独立模式不向producer写入：使用显式受管的消费侧根或原项目本地隔离根，记录模式和不能获得跨项目收益的边界，不能静默借主检出。具体设置名/序列化版本由T-01冻结为唯一接线表，当前不虚构已有CLI。

生成消费者的flags由一个生成构建配置定义并被UI/Web真实Cargo调用一致使用；保现有生成Windows深视图所需256MiB栈支持，不通过降到语言仓32MiB取得复用。语言仓自身配置不强行统一；lld及用户显式RUSTFLAGS/ENCODED_RUSTFLAGS/target/linker/config覆盖按实际优先级保留、具名分隔或冲突拒绝，不能悄悄抹除。实施不承诺所有不同feature的UI/Web单位都相同，只保证本计划控制的配置不再受cwd偶然分裂。

## 3. 技术栈

现有Rust/Cargo、`auto-generation-core::rust_workspace`/`source_owner`、UI生成指纹/正式SDK发行、Web generation.json/assembly身份、Man `auto_tool_execution` owned树、Python清理器。新增构建上下文/生成身份辅助模块如需建立须在现包内（拟`src/generated_build.rs`，新路径），不先细拆crate或新造通用调度器。按本机实际Cargo版本和有效配置验证，不把在线最新机制当本机实测。

## 4. 需求分析与背景调查

### 4.1 授权、当前规范和版本

2026-11-01用户要求根据905结论制定正式实施计划，核906后的实际情况、与UI007的边界和验收，暂不实施。沿auto-plan-new及verification-contract：本轮只计划/必要设计指引，不创建/联系agent/chat，不取用其他会话WIP或改其合同进度，不提交、发布或清理任何现存产物。

**实施授权（2026-11-02）**：用户 `/goal 完成计划910` 经 auto-plan-goal 授权整链（work 含建树、review、有界修复、merge）；§10.1 的实施授权缺口由此关闭，后续阶段继承不再追问。不授权改目标/AC/阈值/兼容承诺/仓库范围/预算。

已读`docs/specs/overview.md`（部分仓地图是旧快照）、[语言独立边界](../specs/repository-boundaries/standalone-language-build.md)、[生成发行](../specs/repository-boundaries/generated-ui-distribution.md)、[测试存储生命周期](../specs/auto-test-case/build-storage-lifecycle.md)、[auto-cache](../specs/auto-cache/project.md)，以及UI[结构](../../../auto-ui/docs/specs/ui/structure-closure.md)/[测试映射](../../../auto-ui/docs/specs/ui/structure-test-map.md)/[编译配置](../../../auto-ui/docs/specs/build/ui-compilation-configuration.md)/[选择](../../../auto-ui/docs/specs/build/ui-test-selection.md)。当前无共享生成生命周期专篇，SD-01补齐；不改auto-cache通用存储合同来冒Cargo治理。

只读快照：lang `367258e7ac3b90aa51fd90cc2d4b4e148522e011`；UI `98cbcd2e79ebc61b0683886f24f0747b20d72617`；Web `847527e388a9bc6a3bcb0f9cf7e5acac979d36bb`；Man `7f30eb88d66c511d60a2f3415ac2b254278f9a0d`；down `7c4d5cbb28d2093cc2e5dfb76d3104984a8dfe9c`。开工重绑实际SHA/锁/源码hash和子目录指令；主检出已有大量docs/号源WIP保全。本次不递归统计磁盘，905旧空间/工具版本均须重核。

### 4.2 接受905和906，不能沿用的旧假设

| 来源 | 接受事实与限制 | 本包后果 |
|---|---|---|
| 905 archived7/7；研究交付65f3b8c92、归档7b48c56e9 | 同配置共享target复用已证；UI第二工程22s/+1.11GiB，Web温6.8s/+0GiB是冻结旧工作集观察；659单位配置漂移、basename劫持/mtime漏失效有实证 | 用于选方向，不作为新组合收益数字/硬阈值；原始scope已删，仅使用计划永久摘要，旧t05_analysis等链接不视为尚存证据 |
| 906 archived8/8；lang4571f55e9、Web847527e、UI98cbcd2、Man7f30eb8 | 合同包单源改归语言；facade/api-gen零Web依赖；L1/L2隔离两层通过；真HTTP组合在lang-web-e2e | 接新来源/receipt owner/锁，不能恢复语言→Web/UI边或用旧路径修旧测试；显式Web消费者仍直连Web |
| 906已具名限制 | D-906-1 Web发行/vendor布局门既有红；D-906-2三UI布局用例既有红；D-906-3模块审计既有红，另有环境flake | 不是全四仓已全绿。T-01核是否阻塞本包必需oracle；所触发行路径若仍红，须精确修复/获批独立前置后闭门，不把预存归因当AC通过或再换假工作负载 |
| UI014/018/901 | 结构/编译测试选择已交付，SDK仍按完整实际来源闭包交付 | 不重拆结构或重开test-feature；28个发行源码crate只是历史快照，当前以受管清单和906来源为准 |

### 4.3 本次代码证据（只读，无运行结果）

- `crates/auto-generation-core/src/rust_workspace.rs`：`resolve_rust_workspace_dir`区分仓内共享/仓外项目workspace；`shared_cargo_target_dir`读取CARGO_TARGET_DIR或寻找语言根target；ensure写workspace/config，生成256MiB栈；`front_member_name`=basename、`back_member_name`=basename-back；`workspace_lock_identity`区分absent/empty/IO错误，必须保持。
- `.cargo/config.toml`：语言MSVC lld+32MiB；与生成配置不同是当前事实，不推断所有差异均应消除。ensure的“Always rewrite”存在，不变内容应比较后写，但905间歇4单位重建根因未定，不能只凭移除fs::write称已修。
- UI `crates/auto-ui-tools/src/rust_ui.rs`：`needs_regeneration`以.at/sidecar mtime与main.rs比较；`build_rust_ui`/运行链约6213起调用成员名并显式CARGO_TARGET_DIR；`generate_rust_ui_with_request`及delivery/fingerprint路径必须共同适配。
- Web `crates/auto-web/src/api_gen.rs`：`backend_generation_is_fresh`/`generate_api`/generation.json已有内容、assembly、consumer、锁等身份保护；不退化成仅输入mtime，不另写不兼容收据。
- Man `crates/auto-man-runtime/src/http_service.rs::serve_rust`：显式Web生成、cwd=生成workspace、cargo run --release的ExecutionSpec，目前只注入服务JSON而未用UI相同target策略；READY来自真实子进程，进程树/reader收尾语义保持。
- `scripts/test-governance/build_storage.py`：默认预览、确切本树Cargo根、lease/活动进程/reparse/源码/证据拒绝。复用其守卫并有限扩展生成缓存子根准入，不直接全盘clean，也不重新实现宽松清理器。

## 5. 详细设计

### 5.1 T-01冻结入口与有界决策

实施授权后唯一组`D:/autostack/.wt/lang-910/{auto-lang,auto-ui,auto-web,auto-man,auto-down}`；语言`plan-910-dev`，实际改动仓用`codex/plan-910-<repo>`各一树，down固定detached；实现/内部核查/独立终审顺序使用同组权威树，无链接/第二审查checkout。计划簿记在语言主检出。

T-01核上表所有调用者及exe发现/desktop_exe/registry/build-cache读取者，冻结source→workspace/member→manifest/receipt→Cargo argv/cwd/env/config→exe/READY→回收图；列现有公开命名与内部缓存布局兼容差额。最多比较两个布局接线，选择单一方案，不能延伸成又一轮无期限研究。对已有公开包名/CLI输出/路径支持的改变，先给具体迁移/回滚差额，再决定是否需用户确认；默认保用户可见app名、启动/desktop_exe发现结果及显式覆盖语义。

代表工作集：UI使用真实002-counter源副本，A/B不同可见计数行为；Web使用真实可独立启动的服务项目源副本（优先905 auto-chat同义样本，或已正式支持的生成服务fixture），A/B响应有不同业务标记。T-01明确现存真实源、请求/返回oracle和生成命令，不能用纯std小包替代。013-todo只记录其原红边界；如该路径成为必需消费，另按§4.2真实闭门，不记skip为PASS。

资源不是无限：T-01列工具/冷构建和受管实验的完整峰值、实时卷余量、jobs/进程数/截止，提出具体预算后，在大构建前确认。建议以905的scope≤45GiB、≤2个实验target、并发≤2 cargo为起始估算而非现成批准；包含组内工具target后不足须重算，不照旧阈值硬跑。正式计量默认单实例，短并发窗口只用于V-07，停止新工作不清他人缓存。新门/支持策略不能为已测结果降低。

#### 5.1.1 T-01 冻结记录（2026-11-02，work）

**组与基面**：`D:/autostack/.wt/lang-910/{auto-lang@367258e7a plan-910-dev, auto-ui@30c9027 plan-910-ui-dev, auto-web@847527e plan-910-web-dev, auto-man@7f30eb8 plan-910-man-dev, auto-down@7c4d5cb detached}`；wt-guard clean。分支名沿906既定 `plan-NNN-<repo>-dev`（计划文本的 `codex/` 前缀改为既定约定，记录调整）。工具链 cargo/rustc 1.98.0（`File::lock` 稳定可用）。scope `D:/autostack/plan-artifacts/plan-910/` 已建。主检出预检：`test/fixtures/reader-import-budget/` 为他会话未跟踪夹具（含 10MiB 文件），不触碰、已呈报；auto-ui 主检出仅未跟踪 docs。

**调用图（只读四仓扫描，全部生产点）**：
- cargo 派生 5 点：①UI `build_rust_ui_inner`（cwd 继承，`CARGO_TARGET_DIR=<lang根>/target`，debug）②UI `run_rust_ui_inner`（cwd=front_dir，同 env，debug）③UI `start_api_server` split 模式（cwd 继承、**无任何 target 钉**，debug）④Man `serve_rust`（cwd=ws_dir 故 ws config 生效，无 env 钉，release）⑤Man tauri 后端（`gen/back/rust` 自带 config，**不在本包范围**）。rustflags 实际取决于 cwd 发现的 config 链：树内 cwd 吃语言仓 `.cargo/config.toml`（lld+/STACK:32MiB），ws cwd 吃 ws config（/STACK:256MiB，无 lld）→ 同 target 同依赖双指纹（905 "659单位漂移"根因，本次代码实证）。
- exe 发现 3 点：`auto-ui-compiler::app_registry::convention_native_exe`（三段约定路径×[snake(pac name), pac name, dir name]×[release,debug]）、`auto-ui-runtime::session::launcher_compiled_exe`（`<app>/rust-workspace/target/<build>/<exe>`）、`host.rs` pac `desktop_exe` 声明路径。生成侧从不算 exe 路径（全走 cargo run）。
- 收据：Web 后端 `generation.json` 写1读1（内容/assembly/lock 三身份，严格准入保留）；**UI 前端成员无收据**（仅 mtime vs main.rs + sidecar mtime）；`backend_generation_is_fresh` 的 `try_ensure_workspace_for` 调用在只读新鲜度检查中无条件重写 ws 清单/config。
- 重复推断器：Man `resolve_workspace_anchored`（无兄弟/无 env 档）、`app_registry::framework_repo_root`（找 `crates/`）与 `find_lang_repo_root`（找 `crates/auto-lang`+兄弟）三套"仓根"概念。
- 其他：auto-man 仓**已跟踪**陈旧 `crates/auto-man/examples/rust-workspace/{Cargo.toml,.cargo/config.toml}`；`scripts/test-governance/external_workloads.py` 以 `-back` 后缀识别生成后端。

**布局选择（两方案比较后单一冻结）**：A=保 `<lang根>/target` 混用+仅钉 flags（否：不分离工具/生成可写状态，清理无法按根）；**B=采用**：生成缓存根 `<producer_root>/target/generated-consumers/<ctx8>/`，producer_root=实际选中 `crates/auto-lang` 的上两级（`compute_auto_lang_rel_path_for` 同一解析序：env→安装SDK→发现→编译期）。ctx8=sha256("generated-build-context/v1"‖canonical小写 producer_root‖有效 rustflags 策略‖用户覆盖标记)[..8]；工具链/实际 argv/cwd/发现的 config 链 hash 记入 `context.json` 与成员收据（可复验，不作键）。支持模式：Development（源检出）→上式；InstalledSdk（exe 旁 `install-manifest.json`）→不写 producer，用 `<ws>/target` 项目本地隔离根；Explicit（`AUTO_GENERATED_TARGET_ROOT`，兼容既有 `CARGO_TARGET_DIR`）→须绝对、规范化、不得为 producer 工具 target 或另一含 `.git`/`crates/auto-lang` 的检出（非 producer/非项目）→否则具名拒绝。
**flags 单源**：受管派生统一注入 `CARGO_TARGET_DIR=<根>`、`CARGO_ENCODED_RUSTFLAGS=-C\x1flink-arg=/STACK:268435456`（仅 windows-msvc；env 档高于一切 config，消除 cwd 漂移，保 256MiB）；用户已设 `RUSTFLAGS`/`CARGO_ENCODED_RUSTFLAGS` 时**不覆盖**，ctx8 并入其值 hash（具名隔离）并打印说明。linker 不钉（不影响依赖 rlib 指纹，仅终端链接；记为边界）。cwd：UI build/split-back/Man serve 统一 cwd=ws_dir；UI run 改为受管 build+发布+直接派生发布 exe（cwd=front_dir 供资源解析，cargo cwd 仍=ws_dir）。
**成员/bin 身份**：id8=sha256("member/v1"‖canonical小写规范化工程路径)[..8]；成员目录 front=`<basename>-<id8>`、back=`<basename>-<id8>-back`（保 `-back` 后缀供脚本识别）；包名 front=`<snake(pac name)>-<id8>`（数字开头沿用 `app-` 前缀）、back=`app-`?`<成员目录名>`；**bin 名保 pac `name`/`exe_name` 语义不变**（用户可见 exe 名不变）。成员目录内 `member.json`{schema, role, project_path canonical, created}二次一致性校验，不符拒绝。构建成功后 exe(+pdb) 硬链接/回退复制发布到 `<ws>/<member>/bin/<profile>/`，前端新收据 `generation.json`{schema, inputs 内容 hash（.at 导入闭包/sidecar/pac/资源清单）, generator/SDK/source-owner 身份, lock 三态, context ctx8+target_root, artifact.exe 绝对路径+hash}；发现者（三点）先读收据再退约定路径。旧 basename 成员：不认领、不删除、报告保留。
**公开面差额**：pac name/exe_name/窗口标题/`auto run|build|service` 命令与参数**零变化**；变化仅为内部缓存目录与成员目录名、`auto run` 打印的生成路径。迁移=新身份一次重建；回滚=逆向四仓接线后旧成员/旧根原位可用。判定为常规等价接线，不需用户确认。
**两真实样本**：UI=`examples/ui/002-counter`（pac name counter，无 back）源副本 A/B，B 改 `Counter:` 文本标签为 `Count B:` 并初值 100（可见行为差异，oracle=生成 main.rs 含该字面量+exe 字符串表）；Web=`D:/autostack/auto-chat@db76383`（真实仓外服务，`src/back/api.at`+db.at，入口 `auto service --server rust`=serve_rust）源副本 A/B，B 在 `/api/health` 等只读端点响应加 `"variant":"B"` 标记（oracle=真实 HTTP 响应体）。副本只在 scope 内，原仓只读。
**既有红阻点判定**：D-906-1（Web 发行/vendor 布局门）触 V-08 外部 delivery 路径→T-05 必须真闭门或精确修复，不以静态检查替代；D-906-2（三 UI 布局用例）与 D-906-3（模块审计）不触本包 oracle，仅记录；013-todo 不在本包工作集。
**预算（按实际容量重算）**：D: 681G 总/340G 余；组内四仓工具 target 冷估 ≤30G；实验上限沿 905 最保守档 scope≤45GiB、单 target≤25G、≤2 并存实验根、并发≤2 cargo、jobs≤12、冷 45min/热 10min 截止；触发线 scope>36G 或 D: 余<100G 则停新实验并报告。340G 余量为该档 7.5×，在既定预算内执行不需追问；每次大构建前 `df` 复核记入证据。
**精确门（T-02..05 建立后列举确认）**：语言 `cargo t auto-generation-core web-delivery-controls`（实改后按依赖扩 `auto-api-gen`）；UI `cargo check -p auto-ui-tools -p auto-ui-compiler -p auto-ui-runtime` + rust_ui 身份/生成族命名测试单进程；Web `cargo test -p auto-web --lib` 受影响 generation 族；Man `cargo nextest run -p auto-man-runtime --lib http_service`+受影响 service 用例。

### 5.2 构建上下文与来源

生成workspace/config/manifest仅内容变化才写；通过同一构建上下文把有效target、flags/linker、profile/triple与显式覆盖传给真实UI/Web构建入口，不能只写一个不会被实际cwd读取的config。记录来源组合/配置schema/工具链和实际argv的可复验身份；有效配置不明时不能按“兼容”合用。相同producer上下文的两个app获得同一受管缓存根，另一worktree获得不同根；禁止把其他worktree解析为默认fallback。Cargo负责合法feature/profile单位区分，不新造rlib复用器。

新根与工具/语言测试根不同；只读安装与delivery继续支持独立消费根和锁，不把主检出路径/开发target写进发行包。保906轻合同lang owner、Web显式装配与links单源。用户显式配置不兼容时明确使用隔离上下文或具名拒绝，不能静默重写。

### 5.3 成员与内容失效

内部成员目录、Cargo package/bin目标和共享target内产物身份必须避免同basename碰撞；键至少包含规范化工程来源路径和front/back角色，并对实际原始来源作二次一致性校验（不凭短hash宣称永无碰撞）。Windows大小写/分隔符/相对绝对别名、front/back、不同worktree与第二绝对根的等价/非等价规则T-01冻结。移动源码允许产生新身份和一次重建，不能把旧成员当新工程；不承诺跨根二进制身份恒定。

生成有效性基于声明输入内容与来源：实际.at导入闭包、sidecar、pac/目标语义配置、资源清单、生成器/SDK/source-owner/fingerprint/schema身份，Cargo.lock absent/empty/error保持不同语义。编译层flags/profile/triple/toolchain/依赖变化由真实Cargo指纹处理，生成层只在影响输出的输入变更时失效，不把所有构建配置都误判为源码改写。保Web已有generation.json严格准入。源缺失/读失败不能等同无变化；旧版无owner/无内容身份收据必须重建或具名拒绝，不沿用mtime快速成功。

旧basename成员不得被新项目认领、覆写或自动删除；已确认属于本工程可在原位置读回验证后迁移到新身份，其他未知旧成员报告后保留。生成写入和准入需同工程互斥、临时写入/提交边界；失败不能发布“新身份+旧main.rs”的混合体。不改变现有公开业务输出，仅内部构建身份变化时对应更新全部发现者。

### 5.4 并发、清理与回滚

复用Cargo自身目标锁，同时保护workspace成员表/生成收据的读改写；两个不同工程同时注册不能丢成员，同工程并发不能混源。Man继续用owned执行设施，不按映像名杀cargo；取消后确认子树/reader/持有锁和lease实际释放，恢复下一次真实生成/编译/运行。坏件可明确失败后按具名受管单元/整个自有闲置根重建，不承诺任意二进制篡改自动被Cargofresh检测。

缓存清理以注册的精确生成target根为单位，默认dry-run，显式apply且核空闲后才执行。原有reparse祖先/活动进程实际cwd-argv-env/lease/证据/源码/未知归属/其他树拒绝策略保留，并在删除前重核；锁顺序/lease必须与build入口共享，防检查后启动竞态。运行中的exe、活跃构建及正在测量的根不可删；没有可靠观测能力时保守拒绝。仅实验自有root做删除正控/拒绝负控，实际生产历史target不在本轮自动删除范围。

回滚：停止本计划owned构建/服务，按依赖顺序逆向恢复四仓提交/配置接线，保旧target/旧成员作为入口；用原正式命令验证后再预览处理新缓存。新生成状态有版本，旧工具遇新布局应明确拒绝或沿已确认迁移路径使用，不能把另一工程旧exe当恢复结果。无需清sccache、registry、用户项目或全机环境。

### 5.5 有限验收与UI007交接

本期有限M1关闭条件是AC-01..07实际成立、独立review及落地完成，随后由路线核查确认；不要求Cargo所有历史债务清零。收益采用事先固定的同义工作集、profile/features/debug/incremental/flags/工具链：各UI/Web同类A+B共享根与A+B独立根作完整稳态对照，所有必要编译和运行完成后量账。两类各应满足共享总unique字节小于独立总和，B的相同兼容第三方单位复用（真实Cargo fresh记录及单位身份比对），不是只共享空目录；应用/增量/PDB必要新增可存在。具体字节与时间如实报告，无预设50%/50–60GB/10x门，不用改变配置或换小工作负载达标。若可靠计量未知/无正收益，则本项不闭，记录原因修订而非研究式no-go代替实施成功。

比较列生成源码、deps、本地成员、incremental/PDB、工具bootstrap、旧根、缓存store/registry和迁移峰值：比较集合外的现存非自有根仅列已知边界，不全盘扫描。unique file ID去重不是CoW物理块去重；NTFS同卷可得分配字节另列，压缩/稀疏/硬链接/共享误差和无法量测项明确，不能只加目录logical大小。墙钟分准备/生成/发现/编译/运行/清理/总计，不以bin hash当语义oracle，不承诺固定耗时阈值。必要对照根在预算内分阶段执行，测账/摘要完成后才回收。

| 与UI007边界 | 本包交付/接收 | 关闭证据 |
|---|---|---|
| 007前置 | 本包接906正式组合；007接本包最终提交、build-context/target/成员键/receipt版本、调用接缝、旧成员迁移与回退、测试/预算/已知限制 | 交接表绑定四仓SHA与canonical；007不靠905研究归档放行 |
| 示例仍未迁移 | 本包仅scope内002/服务源副本，不迁`examples/ui`、不实施主题；007仍负责全部源归位/路径中性消费者接线及首批主题 | 本包不改示例权威源；V-09以scope内模拟旧根→新根迁移证明新身份重建与输出正确 |
| 007实施时 | 移根视为新工程来源；重新生成，不按旧mtime/旧basename复用；更新受管导入/resource源，不恢复语言→UI/Web依赖 | 007需重跑其V-09/10与受影响生成fresh/locked，接本包V-01/02/03/08/09作为基线而非复用旧目录运行结果 |
| 并行与未闭项 | 共享rust_workspace/source_owner/rust_ui/锁短窗单写；TrackCtx不接管；013-todo和906具名红按是否触本包oracle判定 | 相关红未闭不得宣称对应AC/完整M1通过；不把ACE/AG/全部历史债加成前置 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target（实际落地） | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add（已写稿） | lang docs/specs/repository-boundaries/generated-cargo-build-lifecycle.md（eb77695a5） | 三对象（ws/成员身份/受管缓存根）、producer 单选、ctx8、flags env 钉与用户覆盖隔离、成员/收据/发布/发现、锁与 lease、清理准入 | 定义真实有限交付范围 | AC-01..04/07 |
| SD-02 | modify（已写稿） | lang docs/specs/repository-boundaries/generated-ui-distribution.md | 开发落点：落点 ws 不变但**项目锚**+成员带 id8+缓存根归 producer 受管根；交付模式零变化（独立根、不烘开发协议；旧 basename 成员不认领） | 不把开发缓存协议烘进发行包 | AC-01/02/06 |
| SD-03 | modify（已写稿） | lang docs/specs/auto-test-case/build-storage-lifecycle.md | 生成消费者缓存与工具 target 分离；context.json 归属+lease 活 pid 拒绝/过期忽略；其余拒绝规则同本树 target | 沿896既有守卫有限扩展 | AC-04/05 |
| SD-04 | modify（已写稿） | ui docs/specs/build/ui-compilation-configuration.md（2330d56）；man docs/specs/modules/generated-cargo-context.md（新，67add3c）；web docs/specs/project.md（3de5524） | UI 受管上下文+唯一 cargo bin+发布恢复可见名+收据新鲜度+发现序；Man serve_rust 项目锚+上下文 env+lease+owned 树；Web 成员身份+写前比较+park-rename+混合 ws auto-web 规则 | 单源规则与实际消费者一致 | AC-01/04/06 |
| SD-05 | modify（已并入 SD-01 页 §6/§7） | 同 SD-01 页（有限验收实测口径+UI007 移根交接要求） | 无有限验收/移根交接→真实空间口径/兼容复用、限制与 007 移根重验要求 | 知识只沉淀已证实现状；索引应用派生 | AC-07..09 |

frontmatter仅列本仓路径；跨仓Spec由对应实际改动owner沉淀，T-01核其目录/职责后冻结。计划阶段不写canonical或legacy specs.json。若最终没有某页真实差额，须在review有据收敛SD，不虚报已改。

## 6. 测试设计

下表为必需新/扩展验证，均未执行；新用例由T-02..05建立。复用现有生成/delivery/owned和清理测试入口，不新建通用runner。每项记录实际选择数、执行数、原条件排除数，首次红/环境重放/未执行分列，父exit0不代替生成子项/服务ready。

| Case ID | AC / 输入与真实入口 | 可区分错误实现的oracle / owner_stage与边界 |
|---|---|---|
| V-01 | AC-01；UI build/run、Web serve_rust在项目/生成ws两cwd及显式flags覆盖 | 同上下文的真实Cargo argv/env/config、target/编译单位一致；深栈配置不降；自定义flags隔离或明确拒绝；另一worktree绝不指默认同根。work/T-02 |
| V-02 | AC-02；两个同basename不同根的UI A/B、Web A/B，同依赖不同输出，A→B→A；front/back并存 | 生成成员/package/bin目标/实际运行行为各归自己的源，反向覆盖亦不能成功；mtime相同或B更旧仍正确；不能仅比exe名/hash。work/T-03 |
| V-03 | AC-03；未变二次生成、等长编辑保mtime、导入新增/删/缺失、sidecar、pac/资源、generator/SDK换根/锁absent-empty-error | 未变生成源码/manifest/config及有效receipt字节和mtime稳定；单轴真实变化使对应生成/编译重新准入，缺源读错具名失败不沿旧绿；新业务可运行，合法无变化控制不全量regen。work/T-03 |
| V-04 | AC-01/07；同类UI A+B与Web A+B共享vs隔离的完整构建/运行对照 | 确认实际第三方相同单位fresh/复用，空间账满足§5.5；profile/feature差异作合法分裂控制，时间/物理误差分列；不得把“二次快”当复用证明。work/T-05 |
| V-05 | AC-03/04；生成在提交前失败、缺main/错owner/旧receipt、受管坏产物后恢复 | 旧有效生成字节保留或准确残留/恢复报告；重试真正运行新源；无混合身份准入，不承诺Cargo自行识别所有坏二进制。work/T-03/04 |
| V-06 | AC-05；清理器preview/apply自有闲置根、同时活动build/服务/lease、源码/证据/reparse/未知/其他树根 | preview零删；合法apply实际释放且可重建；所有拒绝负控非零、受保护文件数/hash不变；删除前再核竞态，失败/半删不报成功。work/T-04 |
| V-07 | AC-04；两不同工程同时注册/编译，同工程并发+受控取消 | 不丢成员/混收据；确实形成锁争用并观测等待/恢复，全部owned进程/reader/lease退出，后继构建/运行正常。未实际争用不得称并发证明。work/T-04 |
| V-08 | AC-06；新组合UI外部fresh/locked、Web真实READY/响应、缺/错来源、安装SDK/独立delivery | 保906单源receipt/links/L1L2，包不含开发绝对target；负控拒绝且恢复后可消费；D-906-1若阻塞本路径必须真闭门而非以源码检查替代。work/T-05 |
| V-09 | AC-02/06/07；scope内模拟007旧→新示例根、不同绝对根及旧根不可见 | 依新源生成/导入/resource，旧成员不被认领/误删；新身份和正确业务输出、外部锁消费有证，语言独立输入不新增UI检出必需边。work/T-05 |
| V-10 | AC-04/07；停止新owned任务、逆向四仓接线回退，再用原正式生成/构建/运行 | 原入口可恢复，旧根未动；新缓存只预览/自有控制清理，没有全局环境/缓存变化；回退后输出仍对应源。work/T-05 |
| V-11 | AC-08/09；冻结提交的全部AC/遗漏/workaround/SD与真实落地清理读回 | 独立上下文review核实际通路/反控/数字来源；后续merge冻结并集、canonical/交接、全实际输出路径及guard真实收尾。review/T-07、merge/T-08/09分责 |

职责入口：语言已有`auto-generation-core`（兼有`auto-gen-core`旧固定组）、`auto-api-gen`、必要`http`/`integration`与`web-delivery-controls`，T-01按改动选择完整职责组及必要消费者，不叠跑相同目标的两个别名来凑次数；清理相关用既有infra-fixed/治理命名用例及真实控制。Web按auto-web owner lib/受影响generation族；真宿主继续独立lang-web-e2e必要命名集合，不把Web测试放回语言facade。UI按rust_ui身份/生成/发行族，现有生成bare cargo test单进程；Man按受影响service/CLI/owned协议及tcore/tc/ti入口。可直接落地的语言职责入口为`cargo t auto-generation-core web-delivery-controls`；清单实改再按实际依赖扩大完整职责组。UI编译准入候选`cargo check -p auto-ui-tools -p auto-ui-runtime`，Web准入`cargo test -p auto-web --lib`，Man按其现行入口。新用例的精确命令/过滤/测试ID开工冻结并用列举确认非零，不预造不存在测试名；这些是未来门，本次均未执行。

开发/修复只定向反控及完整受影响职责组；最终独立review后merge在冻结提交跑所改职责/必要消费者并集，输入未变的有效证据可绑定复用，不为送审叠跑全仓。不得默认cargo t all/全ACE/AG/GPU；必要原生UI窗口单独排资源窗口。平台/原ignore/HTTP串行/内存条件保持，Linux未测不宣称已测。独立review使用同组冻结树的独立上下文；本次不指定或创建agent，作者自查不冒独立验收。

### 6.1 验证记录（work 阶段，按执行顺序追加；原始日志在 scope `D:/autostack/plan-artifacts/plan-910/v01/` 等，merge 时随 scope 删除）

**V-01（AC-01）2026-11-02，代码 lang 9e3793bf5 / ui 2308db4 / web 8217075 / man 55a4c20；工具 `auto`+`auto-ui-tool` 组内 debug 构建（auto 2m27s 预热后，auto-ui-tool 29s）。**
- 框架内 UI `lang-910/examples/ui/002-counter` `auto build -r rust`（cwd 在语言仓内，旧行为吃语言仓 config lld+32MiB）：冷 3m42s，582 单位，落 `lang-910/auto-lang/target/generated-consumers/54892519/debug/`（成员 `examples/rust-workspace/002-counter-6a8a3a31`，bin `counter`）。**730 个指纹 json 全部 `rustflags=['-C','link-arg=/STACK:268435456']`**（语言仓 32MiB/lld 零渗入）。首次运行暴露**既有缺陷**：`compute_auto_ui_runtime_rel_path` 对 `../../crates/auto-lang` 做词法 parent 算术，UI owner 解析到 `examples/auto-ui` → "producer UI owner 源缺失"（仓外布局因相对路径尾部恰为 `auto-lang/crates/auto-lang` 而侥幸正确）；修复=先 canonicalize（9e3793bf5，回归用例 `workspace_chooses_ui_owner_and_rejects_lang_shadow` 框架内分支）。
- 同项目 `auto run -r rust`：cargo `Finished 1.45s`、**0 单位重编**（build/run cargo cwd 统一为 ws），打印并直跑发布副本 `…/002-counter-6a8a3a31/bin/debug/counter.exe`（Iced 启动、MCP 监听），25s 超时终止树 exit 143。
- 仓外 UI `plan-artifacts/plan-910/samples/ui-a/002-counter`（发现序 → 兄弟 `D:/autostack/auto-lang` 主检出为 producer）：冷 3m43s/582 单位，根 `D:/autostack/auto-lang/target/generated-consumers/f3f98a9e`，730 单位同一钉值；收据 `generation.json` 含 inputs(pac.at/app.at sha256)/generator+producer/lock `3a1676fb`/ctx/artifact(exe sha256 d9104fa0…, bin counter)；`bin/debug/counter.exe` 发布（原硬链接计数 3 → 改 copy 见下）。
- Web `samples/web-a/auto-chat`（`AUTO_LANG_ROOT` 显式钉 lang-910）`auto service --server rust -d … --http-config-inline '{"profile":"development","listen":{"port":18101}}'`：冷 release 3m05s/436 单位 READY，`GET /api/identities` → `[]`（真实响应）；**与框架内 UI 共用根 54892519**：`debug/`730 + `release/`557 单位并列，全部同一钉值；Web 收据新增 `build_context{54892519,target_root}`、`workspace_lock: absent`（生成时尚无 lock，R5-01 一次绑定沿用）。
- Web 第二 cwd（项目目录直启，端口 18102）：READY 24s，响应相同，**release 指纹集合 557 完全一致**；重编 3 单位（auto-generation-core←我同期改源码→auto-web→back crate，非 cwd 漂移）。首次复跑时服务1未停 → cargo 重 uplift 运行中 exe 失败 "os error 5" → `backend exited with 101 before ready`（具名失败，非混源）；由此 (1) 发布副本改 **exe copy / pdb hardlink**（硬链接共 inode 会锁死 cargo 重链接），(2) Web 生成器无条件重写 main.rs（905 已观察）列入 T-03 写前比较。
- 负控（真实运行）：(a) `AUTO_GENERATED_TARGET_ROOT=D:/autostack/auto-lang/target/generated-consumers/f3f98a9e`（他检出）+ producer lang-910 → exit 1 "位于另一检出 D:\autostack\auto-lang 内（拒绝借用他树可写 target）"，零写入；(a2) `=…/lang-910/auto-lang/target` → "指向 producer 工具 target"；(b) `RUSTFLAGS="-C opt-level=1"` → 新根 `c68eba38`，`context.json{rustflags:null,user_flags_override:"RUSTFLAGS"}`（具名隔离，不注入受管钉值；构建在 context 记录后即终止，该半成根登记为 V-06 清理候选）。
- 边界记录：linker 不钉（不影响依赖 rlib 指纹）；`auto build -d <dir> -r rust` 走 pac/ninja 分支而非 rust UI 分支（rust 分支用 `current_dir()`，既有 CLI 缺陷，债务候选，本包不改）；机器上同时存在他会话 `cargo-nextest`/`auto.exe service --server vm`（主检出）进程，本包只按 PID 清理自有树。
- 结论：AC-01 的 V-01 部分成立（单源上下文、同 owner 兼容工程共根、他树/工具 target 拒绝、用户 flags 具名隔离、256MiB 不降）；V-04 空间账待 T-05。

**V-02/V-03/V-05（AC-02/03）2026-11-02，驱动 `t03_driver.sh`/`t03b_driver.sh`（scope `v03/`、`v03b/`），代码 lang 0e525cdb8 / ui 3a6d81e / web 4435d81；样本 `samples/ui-a|ui-b/002-counter`（同 basename、不同根；B：`Count B:`+初值 100）在**同一共享 ws**（`AUTO_RUST_WORKSPACE=samples/shared-ws`，producer lang-910）。**
- V-02 首轮暴露两个真实缺陷：(1) fresh 构建 cargo **不重 uplift** `debug/counter.exe`，发布从 uplifted 取件→B fresh 后发布到 A 的 exe；追查发现 **Windows/MSVC cargo 对 bin 不加 metadata hash**（`deps/counter.exe` 与 `debug/counter.exe` 被所有同名 bin 共享）→劫持在链接输出层，Cargo 无法区分。定型：cargo bin 名 `<published>-<id8>`（Cargo.toml 内 `# published = "<name>"` 注释），**发布时恢复用户可见名** `<member>/bin/debug/counter.exe`；取件路径来自 `--message-format=json-render-diagnostics` 的 `compiler-artifact.filenames`（本包/本 bin 匹配，fresh 单位亦报告），不再碰共享 uplift 位。(2) 生成器改变输出形状但版本串不变→B 成员不重生成：加 `FRONT_LAYOUT_VERSION`（front-layout/v2）进生成器身份 + 新鲜度判断**重算清单字节比对**（sidecar-dep 成员仅靠版本常量，记边界）；bin 名解析改**清单优先**（旧收据不得重定向取件）。
- V-02 终态（`v03b/results.txt`）：成员 `002-counter-72e31f46`/`002-counter-533b4df0` 并存；A→B→A(fresh 0 单位)→B(fresh 0 单位) 后 A 发布含 `Counter: `/不含 `Count B: `，B 反之；两收据 `artifact.exe` 各指自家 `bin/debug/counter.exe` 且 `exe_sha256` 与文件实测一致；B 源 mtime 置 2020（早于 main.rs）仍正确（内容身份）；B 的 member.json 篡改为 A 路径→"成员目录已被其他工程/角色占用…旧 basename 成员不认领" exit 1。oracle 用 exe 字符串表中的模板字面量（真实编译进二进制），非仅文件名/hash；未做 GUI 自动化点击（边界）。
- V-03（`v03/results.txt`）：未变二次构建 manifest/config/main.rs/receipt/member **bytes+mtime 全稳定**、0 单位；等长编辑+`touch -d` 还原 mtime → regen+1 单位，发布含 `Kounter: `；pac 注释编辑 → "Regenerating" 1 次、main.rs sha+mtime 不变（写前比较）；.at 增/删各 regen；sidecar 增/仅改 .rs/删各 regen（成员 helper.rs 含 V2）；锁删除/清空 → regen（cargo 重解析，内容同→0 单位）；锁改为目录 → exit 1 "failed to open …Cargo.lock"；producer 切主检出 → regen 落 `f3f98a9e`（10 单位）、切回 0 单位；producer 伪造 → exit 1 "显式 AUTO_LANG_ROOT 无效或缺少工作区依赖"。空 .at 集（front 目录存在但无 .at）首轮 exit 0 且收据被改写（混合体风险）→ 修：`regenerate_code_only` 两处早返回均改为**具名拒绝**（"front 输入为空…拒绝以空输入集重生成"），复验 exit 1、收据字节不变。`src/front` 整目录缺失走 CLI pac/ninja 派发（既有行为，边界）。
- V-05：app.at 注入语法错误 → exit 1、成员状态（bytes+mtime）完全不变；删 main.rs → 全量重生成；旧 schema 收据 → 全量重生成并重写 v1；发布副本截断 → 下次构建自愈（86964736）；**受管缓存件（cargo-reported uplifted exe）截断且 cargo fresh** → 首轮被原样发布（0 字节）→ 修：PE 头/可读校验移到**发布前**，复验 exit 1 "cannot read cached artifact…failed to fill whole buffer"、发布位保留好副本；恢复=删该单元 `.fingerprint/counter-72e31f46-*` 后重建（1 单位、尺寸还原）。Cargo fresh 不检测任意篡改——按计划记为边界，恢复路径实证。
- 自伤两例（已澄清不计入产品问题，但有证据价值）：驱动运行中我改 gen-core 源 → 路径依赖链 5 单位重编；我**手动**在共享根跑未钉 flags 的 `cargo build` 诊断命令 → 指纹 rustflags 漂移 → 受管构建 582 单位全量重编（正是钉 flags 要消除的现象）。
- 附带修复（G2 "未变不重写"）：Web 生成器全部产物 `write_if_changed`，收据改 **park-rename**（`generation.json.pending`，未变则原件原 mtime 还原，失败则无有效收据）；sidecar 模块复制改仅内容不同时复制；UI main.rs/Cargo.toml 写前比较；生成期输入快照（构建后输入若变则收据 withheld，同 Web 规则）。

**V-06/V-07（AC-04/05）2026-11-02，驱动 `t04_v06_driver.sh`/`t04_v07_driver.sh`/`t04_v07b_driver.sh`（scope `v06/`、`v07/`），代码 lang 64a4cd450 / ui 3a6d81e / web 4435d81 / man 8a924c3。**
- 清理器（lang-910 `scripts/test-governance/build_storage.py`，无 psutil 环境）：预览 `54892519` → 21385 文件 / 22.58 GB unique，**零删除**；预览 `c68eba38`（RUSTFLAGS 隔离半成根）→ 1 文件 386 B。活 lease（web-a `auto service` 运行时持 `serve_rust` lease）→ 预览与 apply 均 exit 2 "live build/run lease pid 56808 (serve_rust) holds this root"；服务被杀后 dead-pid 过期 lease → 预览通过并列入 `stale_leases_ignored`。他 producer 根（主检出 `f3f98a9e`）→ "not an exact Cargo cache owned by this checkout"；无 context.json 目录 → "unowned"；根内放 results.json → "evidence mixed into cache"；嵌套路径 `…/54892519/debug` → 拒绝；context.json `producer_root` 篡改 → "belongs to another producer"（驱动内 Python 收到 msys 路径未能篡改成功，直接复现确认拒绝）。**apply** `--apply --idle-confirmed` 于空闲 `c68eba38` → 实删（removed/386 B 收据行），同配置再构建按需重建根（context.json 再现）。reparse 负控未重跑（worktree 内禁建链接；既有守卫逻辑未改）。
- 并发/取消：C1 两工程同时 `auto build`（各改源）→ 均 exit 0，A/B 日志各见 **"Blocking waiting for file lock"**（真实 cargo 锁争用），ws members=2 无丢失，各自发布标记 `Counter C1`/`Count B C1` + 收据 sha 一致。C2 同工程并发（改源）→ run1 regen+编译，run2 regen（ws 锁串行）+`blocked=1`+0 编译，两者 exit 0，收据 JSON 合法且 sha 一致（无混合）。C3 编译中外杀 `auto` 父进程 → lang-910 相关进程 0 个（owned 树随父全灭）、被杀构建 exit 1、发布位仍为上一好副本（sha 一致）、留 dead-pid lease（后续 acquire 清扫）、恢复构建 exit 0/1 单位/新标记。C4 `auto service` 仅杀 `auto` 父 → 后端监听消失、back exe 退出（Job/ProcessTree 语义）。
- V-07 首轮暴露**第三个真实缺陷**：C4 服务继承 `AUTO_RUST_WORKSPACE` 把 web back 成员生成进共享 UI ws，随后 UI 侧 ensure 重写 `[workspace.dependencies]` **丢掉 Web facade 事后注入的 `auto-web`** → back 成员清单加载失败 → **整个 ws 任何构建失败**（front/back 并存的真实中毒路径，既有设计缺口）。修：核心 ensure 识别 `manifest_needs_web`（任一成员引用 auto-web）并经 owner 规则自行解析写入 `auto-web = { path, version = "=0.1.0" }`；子进程单测 `mixed_workspace_keeps_web_dependency_across_ui_ensure`（64a4cd450）。

**V-04（AC-01/07）2026-11-02，驱动 `t05_v04_driver.sh`（scope `v04/`，`measure_root.py`：logical / unique-file-ID / NTFS 实际分配（GetCompressedFileSize）/ 指纹单位集），代码 lang 64a4cd450 / ui 3a6d81e / web 4435d81 / man 8a924c3；同 producer lang-910、同受管 flags、同 profile（UI debug / Web release）、全新显式根（scope 内），隔离根测账后即删，共享根保留至 V-09/10。**

| 工作集 | 根 | 文件 | logical | unique(file-ID) | NTFS 分配 | 单位 | 编译单位 | 墙钟 |
|---|---|---|---|---|---|---|---|---|
| UI A 隔离 | iso-ui-a | 9663 | 11.58 GB | 10.51 GB | =unique | 730 | 582 | 171.2 s |
| UI B 隔离 | iso-ui-b | 9663 | 11.58 GB | 10.51 GB | =unique | 730 | 582 | 177.0 s |
| UI 共享 A 后 | shared-ui | 9663 | 11.58 GB | 10.51 GB | =unique | 730 | 582 | 174.2 s |
| UI 共享 A+B 后 | shared-ui | 9936 | 13.62 GB | **11.72 GB** | =unique | 731 | **1** | **20.3 s** |
| Web A 隔离 | iso-web-a | 4174 | 1.74 GB | 1.59 GB | =unique | 557 | 436 | READY 173.3 s |
| Web B 隔离 | iso-web-b | 4174 | 1.74 GB | 1.59 GB | =unique | 557 | 436 | READY 169.6 s |
| Web 共享 A 后 | shared-web | 4174 | 1.74 GB | 1.59 GB | =unique | 557 | 436 | READY 190.7 s |
| Web 共享 A+B 后 | shared-web | 4245 | 1.92 GB | **1.744 GB** | =unique | 566 | **7** | **READY 100.4 s** |

- 结论：UI 共享 11.72 GB < 隔离总和 21.02 GB（**省 9.30 GB / 44%**）；Web 共享 1.744 GB < 3.185 GB（**省 1.44 GB / 45%**）。严格正收益，无预设阈值。
- 复用证据（非"二次快"）：UI B 的 730 个指纹单位 id 中 **729 个在共享根 A 构建后已存在**（真实 Cargo fresh），B 仅新增 1 单位（自身 crate）；Web B 557 中 548 已存在，新增 9（自身 crate + 生成模块/构建脚本单位），编译 7。
- 行为 oracle：UI 发布副本 A 含 `Counter: `；Web `GET /api/identities` A→`[]`、B→`[{"id":"variant-b","name":"VARIANT-B",…}]`（真实 HTTP，隔离与共享根一致）。UI B 发布串当时为 `Count B C1: `（v07 首版驱动坏 sed 未复原标签，非产品问题；收尾统一复原后复验）。
- 口径与误差：logical>unique 的差为根内硬链接（deps/uplift/pdb），unique 用 (st_dev, st_ino) 去重；NTFS 分配 = unique（无压缩/稀疏）；unique-file-ID 去重非 CoW 物理块去重；墙钟含准备/生成/发现/编译/发布（UI）或至 READY（Web），未拆到 cargo 内部阶段；分配/迁移峰值：D: 273G→260G（隔离根删后回升）。比较集合外现存根（主检出 `f3f98a9e` 4.3 GB、lang-910 `54892519` 22.6 GB 含多轮 incremental）仅列边界，不计入对照。

**V-08（AC-06）2026-11-02，scope `v08/`、`v09/`；产品代码 lang 64a4cd450（发行门收据所绑；门脚本修复 19bfb579d 于收据后提交，产品 crate 无改动）/ ui 3a6d81e / web 4435d81 / man d823fdd。**
- 外部 UI 发行门 `scripts/ui-repository-distribution.py`（真 clone、非 worktree 复用）：**DST-1 fresh-lock 2s / DST-2 locked 真实构建 202s / DST-3 整组重定位后 locked 重放 195s / DST-4 无 UI 兄弟的隔离 lang SDK consumer build+test 126s，全 PASS exit 0**；收据 `v08/distribution-receipt.json`（sources：lang 64a4cd450/ui 3a6d81e/down 7c4d5cb/web 4435d81；17G scratch 已回收）。**两处门脚本既有红已精确修复**（19bfb579d）：(1) auto-ui 自 AG 计划起含 183 字符相对路径夹具，Windows MAX_PATH 下任何 scratch clone 均 "Filename too long"→clone 加 `-c core.longpaths=true`；(2) 自 PLAN-890/901 起 auto-ui 硬依赖兄弟仓 auto-web，门仍 clone 三仓→"failed to load manifest for dependency auto-web"→四仓组。DST-2 同时证明本包给 auto-ui-compiler 新增的 auto-generation-core 依赖在独立 clone 的 fresh-lock/locked 构建中正确解析。
- 安装 SDK 独立模式：`man-install.py install --profile full --skip-build` 先撞 **D-906-1 已登记根因**（901 vendor 快照 crate 进 metadata 图："non-crate package input requires explicit mapping"，列成员又 "multiple workspace roots"）→ **man-install 腿精确修复**（d823fdd：vendor/ 显式映射入安装清单 + 排除出 SDK workspace，同语言仓根做法）→ 安装成功（7675 文件：auto.exe/auto-ui-tool.exe/sdk/install-manifest.json/licenses）。安装版 `auto build -r rust` 于样本副本：exit 0、582 单位、4m55s；**SDK 树字节级零写入**（前后快照一致）；ws `.cargo/config.toml target-dir="target"`（项目本地隔离）；`target/context.json` mode=**installed-sdk**、producer=`<install>\sdk`、ctx=2dde6fb1；成员 `002-counter-56ddbc18` 发布件含 `Counter: `。D-906-1 的 **Web 发行门腿**（web-repository-distribution / web 锁漂移）不触本包 oracle（本包 Web 验收=真实 serve_rust READY+响应，已过；auto-web 包面零改动），维持 Web owner 关闭条件；本修复即其根因在 man-install 侧的对应修法。
- 缺/错来源负控：`AUTO_UI_SOURCE_ROOT=D:/nonexistent-ui` → exit 1 "AUTO_UI_SOURCE_ROOT 无效或缺少 crates/auto-ui-runtime"；恢复（去 env）exit 0 fresh。producer 伪造已在 V-03（"显式 AUTO_LANG_ROOT 无效或缺少工作区依赖"）。
- 906 单源边界不回退：api_gen 无 Web 依赖方向回退（delivery e2e 断言修复后绿）；Web 收据 schema_version 1 只增不改（build_context 字段为新增）；`delivery_bundle_end_to_end`（交付 bundle 全量断言+`--locked` 口径由 DST-2/3 承担）绿；包不含开发绝对 target：交付路径独立 root（delivery manifest 检查），SDK 模式与开发根互不渗透。

**V-09（AC-02/06/07）2026-11-02，scope `v09/`。**模拟 007 移根：`samples/ui-a` → `samples/ui-a-moved`（同 basename、新绝对根）→ 新身份 `002-counter-0f9bc297`（旧 `72e31f46`）：moved 构建生成+1 单位、cargo bin `counter-0f9bc297`、发布件 `bin/debug/counter.exe` 含 `Counter: `；收据记 ws lock `72f843a0`+ctx `54892519`（消费共享 Cargo.lock，身份可复验）；**旧成员 Cargo.toml/main.rs/receipt/member.json/发布件 bytes+mtime 全部原样未删**；ws members=3（旧+新+B 共存）；移回后旧成员 0 单位 fresh 复活；迁移遗留的 stale 成员不自动删除（清理仅显式预览/apply）。语言独立输入未新增 UI 检出依赖（producer 全程 lang-910 env 钉定）。

**V-10（AC-04/07）2026-11-02。**回滚=用**主检出 910 前二进制**（`D:/autostack/auto-man/target/debug/auto.exe`，`auto 0.1.0+plan-890-verified-r7-t19-5-g55c98c7`）按原命令 `auto build -r rust` 于全新样本：exit 0、582 单位；**旧布局原样可用**（项目本地 `rust-workspace/002-counter` basename 成员、`target-dir="target"`、exe 落 `D:/autostack/auto-lang/target/debug/counter.exe` 含正确标记）；主检出 `examples/rust-workspace` 未被触碰；新受管缓存仅预览（`"mode": "preview"`）零删除。副作用如实记录：旧工具按其既有行为把 582 单位写入主检出 `target/`（主检出 target 由 40G 计；本包不回收他人/主检出工具缓存）。910 侧停止与恢复：V-07 已证取消/恢复；四仓接线回退即检出自合并前的 master（merge 阶段 ff 语义保证可逆）。

**T-06 冻结（2026-11-02）：健康/遗漏/债务/预算与 007 交接。代码终点：lang eb77695a5 / ui 2330d56 / web 3de5524 / man 67add3c（基面 367258e7a / 30c9027 / 847527e / 7f30eb8；down 7c4d5cb 只读）。**

- 健康：改动文件零新增编译警告（rust_ui.rs 9 个 rustfmt 残差 ≤ 基面 11，均为既有区域；core/api-gen/web/man 触碰文件 fmt 0 残差）；diff 无 dbg!/DEBUG println/todo!；py_compile 三脚本过。受影响门实跑：lang 固定组 `cargo t auto-generation-core auto-api-gen`（2 组 PASS，2m50s 含 web-delivery-controls）；`web-delivery-controls` 12/13 过、1 个 **既有 ERROR**（`web_delivery.py` `execution_ids` 未定义，Plan 890 谱系，主检出 master 同判复现，文件未被本包触碰）→ 登记 D-910-1；ui rust_ui 族 56 过+1 ignore（单进程）；web api_gen 48+1 ignore；man http_service 5/5。
- 修复的既有红（非本包引入，均在必需通路，精确修复+复验）：① api-gen→auto-web 断言过期（906 退役依赖）→ delivery e2e 断言改 906 后合同；② 框架内 UI owner 词法 parent 缺陷（9e3793bf5）；③ 发射门 clone MAX_PATH（AG 183 字符夹具）与三仓组缺 auto-web（19bfb579d）；④ man-install vendor 快照映射（D-906-1 man 腿根因，d823fdd）。
- 债务候选（登记 KNOWN-DEBT）：D-910-1 web_delivery.py `execution_ids` NameError（既有，owner: lang test-governance）；D-910-2 `auto build -d <dir> -r rust` 走 pac/ninja 派发（rust 分支用 `current_dir()`，既有 CLI 缺陷）；D-906-1 Web 发行门腿维持 Web owner 关闭条件（本包已修 man-install 同根因腿）。
- 记录边界（不冒已验证）：linker 不钉；main.rs 层生成器变更需手动 bump `FRONT_LAYOUT_VERSION`（sidecar-dep 成员仅靠版本常量）；GUI 点击 oracle 未自动化（用 exe 字符串表字面量）；reparse 负控未重跑（worktree 内禁建链接；守卫逻辑未改）；`start_api_server` 的数字前缀改写分支因生成端已前缀而成为不触发死码（保留）；tauri `gen/back/rust` 与 `write_shared_cargo_config` 不在范围；Cargo fresh 不检测任意二进制篡改（护栏=发布前 PE 校验+单元级恢复）。
- 资源/预算对账：D: 226G 余（>100G 触发线上方）。实验根现存：lang-910 `54892519` 22G（T-09 清理器 apply 回收）、主检出 `f3f98a9e` 15G（**具名持有**：merge 后以 master 清理器 apply 回收——本包产出的跨检出正控）、`c68eba38` 7K（T-09）；v04/v08 隔离与发行 scratch 已测账即删（17G+18G+10G 已回收）；scope 现存 3.3G。V-10 旧工具按既有行为向主检出 `target/` 写入约 15G（用户工具缓存，本包不回收）。工具 target（组内四仓）≈30G 为构建必需，随 worktree 删除（T-09）。
- **UI007 交接表**（绑四仓 SHA=上行终点；007 不得凭 905 研究归档放行）：
  - 上下文：`GeneratedBuildContext`（env 钉 flags/target、producer 单选、ctx8、context.json）——接缝 `resolve_managed_front`/`cargo_managed_build`/`publish_front_artifact`（rust_ui.rs）；Man=serve_rust；Web=generate_rust_server。
  - 身份：成员 `<basename>-<id8>[-back]`、member.json、cargo bin `<published>-<id8>`、发布件 `<member>/bin/<profile>/<published>.exe`、发现收据优先。
  - 移根：新 id8/新成员/重生成；旧成员不认领不删；收据含 ws lock+ctx 可复验。
  - 必需重验：007 的 V-09/10 与受影响 fresh/locked；以本包 V-01/02/03/08/09 为基线（§6.1）。
  - 限制：上行"记录边界"全部条款；预算对账以实时 df 为准。

## 7. 验收标准

| ID | 可观察验收 / 验证 | owner_stage |
|---|---|---|
| AC-01 | UI/Web实际Cargo调用使用单源生成上下文，同owner兼容工程共用受管target、不同worktree隔离；有效flags/cwd/显式覆盖/只读安装边界明确，保profile和深栈语义；V-01/04 | work |
| AC-02 | 同basename/不同来源与front/back不碰撞，实际A→B→A行为正确；旧成员不被误认领，移根可正确重建与发现exe；V-02/09 | work |
| AC-03 | 生成按完整已声明内容/来源身份准入，保mtime编辑/删导入/sidecar/源切换有效；未变不重写；读错/旧receipt/失败发布不复用错物；V-03/05 | work |
| AC-04 | workspace/生成并发不丢状态，实际争用/取消/坏件/重试正确，owned树和lease释放，原回滚入口可用；V-05/07/10 | work |
| AC-05 | 精确生成缓存有owner/lease/预览/空闲apply，原测试清理保护不降；活动/未知/链接/源码/证据/他树拒绝及合法回收重建实证；V-06 | work |
| AC-06 | 906来源/独立边界不回退，外部UI发行fresh/locked/恢复及Web实际服务通过，移根不新增语言→UI依赖；相应既有红若仍阻塞则未通过；V-08/09 | work |
| AC-07 | 两类真实同义工作集按§5.5取得兼容依赖复用及严格正空间收益，完整账/时间/峰值/误差齐；UI007交接含四仓SHA/调用/身份迁移/必需重验及限制，未主题化/搬权威示例；V-04/09/10 | work |
| AC-08 | 独立复审逐AC/全通路/原反例/遗漏/workaround/新warning/debug与SD核实，无未批准降门或必需gap；V-11 | review |
| AC-09 | 冻结受影响门真实通过、各仓合入、canonical及007交接落地；归档和全路径scope/树/分支清理具名持有可读回，不冒Cargo全历史问题已清；V-11 | merge |

## 8. 执行步骤

| ID | 动作/真实修改入口与关闭条件 | 依赖 / AC | owner_stage |
|---|---|---|---|
| T-01 | [x] 授权后建唯一组，冻结906组合/调用与exe图/两真实样本/支持与配置覆盖表/工作树隔离/精确门和资源预算；判既有红是否阻塞；有界布局选择和公开兼容迁移差额，记录开工决策 —— **完成 2026-11-02**：组五树已建（wt-guard clean）、scope 已建、四仓只读扫描冻结 5 派生/3 发现/1写2读收据点、布局 B 单一冻结、身份/flags/支持模式/预算/既有红判定全部记入 §5.1.1 | 用户明确实施授权；全部work AC前置 | work |
| T-02 | [ ] auto-generation-core::rust_workspace及必要新辅助模块实现单源上下文/写前比较；UI rust_ui与Man serve_rust/Web真实调用接线，target owner分离且安装/delivery独立；建立V-01正反 —— **完成 2026-11-02**：lang d4c1217+9e3793bf5 / ui 2308db4 / web 8217075 / man 55a4c20；V-01 正反控见 §6.1 | T-01；AC-01 | work |；**R1 重开（F-910-R1-01 cwd 合同回归）**
| T-03 | [ ] 统一front/back成员与bin发现身份；UI needs_regeneration内容/receipt与Web已有freshness整合，保source_owner/锁三态、原成员迁移/拒绝和失败发布；补V-02/03/05 —— **完成 2026-11-02**：唯一 cargo bin+发布恢复可见名、内容收据/布局版本/清单比对、空集与坏件护栏、写前比较；V-02/03/05 见 §6.1 | T-02；AC-02/03 | work |；**R1 重开（F-910-R1-02 pdb 发布缺失）**
| T-04 | [x] 成员更新/生成/构建lease与owned取消恢复；有限扩展build_storage精确子根预览/apply守卫，真实竞争/清理正负控V-05/06/07 —— **完成 2026-11-02**：lease（spawn/进程生命周期/服务）、dead-pid 清扫、清理器受管子根准入、混合 ws auto-web 保留；V-06/07 见 §6.1 | T-03；AC-04/05 | work |
| T-05 | [x] 在906新组合完成两类A/B完整收益对照、外部fresh/locked/来源失败恢复、模拟007移根和回滚；必需既有红按精确触面闭门，全部work AC实证，无未批准支持缩减 —— **完成 2026-11-02**：V-04 两类严格正收益（§6.1 表）、发行门 DST-1..4 全过（两处既有红精确修复）、安装 SDK 独立模式实证（D-906-1 man-install 腿根因修复）、V-09 移根/旧成员保全、V-10 原入口回滚；预算对账后 v04 根已回收 | T-04；AC-01..07，V-01..10 | work |
| T-06 | [x] 最终健康/遗漏/workaround扫描，冻结全部work证据/数账/限制、SD-01..05 proposal及007交接表；原始证据摘要可随scope删除而独立读回，execution_done6/9 —— **完成 2026-11-02**：健康扫描过、既有红 4 处精确修复、债务 D-910-1/2 登记、边界全列、SD-01..05 落稿（四仓）、007 交接表+预算对账在案 | T-05；AC-01..07 | work |
| T-07 | 同组冻结提交独立上下文final review，核真实调用/反控/空间数账/残项及SD；finding回同树work且使受影响证据失效 | T-06；AC-08，V-11 | review |
| T-08 | 冻结受影响职责并集与必要消费者门，各仓依赖顺序真实合入，沉淀canonical与最终007交接；路线按本包有限出口判M1，不自动执行007 | T-07 pass；AC-09 | merge |
| T-09 | 全实际输出路径/进程/链接/归属对账，必要摘要折Plan；删除统一scope/树内构建物、每树wt-guard clean后移除树/分支/空组，更新持有登记，归archive终态9/9 | T-08；AC-09 | merge |

材料唯一scope=`D:/autostack/plan-artifacts/plan-910/`，按repo及消费者身份区分，创建时登记owner/用途/在途。源码副本/SDK/发行包/日志/探针/图/截图/外置target只在此；正常工具构建在本组树内。不得写attachments或第二套外部根。merge摘要含命令/退出码/关键数字/节选/源与保留件hash，随后整scope删除，构建中间物不回收暂存；仅具名未解决问题可持有owner/理由/关闭条件。不因archive或时间自动释放未知物，不触其他会话缓存/进程。

## 9. 复审记录

2026-11-01 / stage:new / PLAN-910 r1 / outcome:blocked（仅实施授权/开工决策尚未给出，非规划检查失败） / 905/906已归档，906后源码只读核实target/flags、成员名及mtime缺口仍在；正式实施合同明确有限目标/两真实消费/空间正收益、清理与007交接。静态预检pass：0–10章节、9T/9AC/11V/5SD唯一连续、全部本地Markdown链接有效、无尾空格、910唯一/号源单调；906四仓交付SHA均为当前HEAD祖先。未建树/探针/metadata/构建/测试/容量实验、未改生产/Spec/环境/缓存、未提交或联系其他agent。next：用户审阅合同；明确实施后T-01冻结实际组合/支持/预算与既有红阻点，决定完整接线后进T-02；不得先做生产修复再补合同。

stage: work | plan_id: PLAN-910 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-910/{auto-lang,auto-ui,auto-web,auto-man,auto-down} | branch: plan-910-dev / plan-910-ui-dev / plan-910-web-dev / plan-910-man-dev / (down detached) | base_commit: lang 367258e7a, ui 30c9027, web 847527e, man 7f30eb8, down 7c4d5cb | code_commit: lang eb77695a5, ui 2330d56, web 3de5524, man 67add3c | dependency_revisions: down 7c4d5cb（只读）；toolchain rustc/cargo 1.98.0 | task_ids: T-01..T-06 完成（6/9），T-07 review / T-08、T-09 merge 待 | evidence: §5.1.1 冻结记录；§6.1 V-01..V-10 全部实跑记录（原始日志 scope `D:/autostack/plan-artifacts/plan-910/{v01,v03,v03b,v06,v07,v04,v08,v09}/`，merge 时随 scope 删除）；AC-01..07 逐项实证（见 §6.1 各 V 条目）；语言固定组 auto-generation-core/auto-api-gen PASS、web-delivery-controls 12/13（1 既有 ERROR=D-910-1）、ui rust_ui 56+1ign、web api_gen 48+1ign、man http_service 5/5；外部发行门 DST-1..4 PASS 收据 v08/distribution-receipt.json | blockers: 无（既有红 4 处精确修复、2 处登记债务不阻 work AC） | next_responsible_role: review（T-07，同组冻结树独立上下文，核 AC-08：逐 AC/全通路/反控/空间数账/遗漏/workaround/新 warning/debug 与 SD-01..05 稿）

stage: goal | plan_id: PLAN-910 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-910/auto-lang | commit: eb77695a5 (ui 2330d56, web 3de5524, man 67add3c) | next: review

stage: review | review_scope: final | reviewer_identity/limitations: 独立审查代理（general-purpose，独立上下文，只读+定向重放；非作者自查；从冻结树重建 auto/auto-ui-tool 以覆盖精确 HEAD） | plan_id: PLAN-910 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-910/{auto-lang,auto-ui,auto-web,auto-man,auto-down} | branch: plan-910-dev / plan-910-ui-dev / plan-910-web-dev / plan-910-man-dev | reviewed_commit: lang eb77695a5, ui 2330d56, web 3de5524, man 67add3c（五树 porcelain 全空） | base_commit: 367258e7a / 30c9027 / 847527e / 7f30eb8 | dependency_revisions: down 7c4d5cb | spec_inputs: SD-01..05 稿（lang generated-cargo-build-lifecycle.md 新、generated-ui-distribution.md、build-storage-lifecycle.md；ui ui-compilation-configuration.md；man generated-cargo-context.md 新；web project.md）于各冻结提交 | acceptance_results: AC-01 partial（F-01）；AC-02/03/04/05/06/07 pass（独立重放：core 28/0、web api_gen 48/0、ui 四例 4/0 含 delivery e2e、清理器六负一正、真实构建 build2 0 单位四文件 bytes+mtime 全稳、发布件标记+receipt sha 一致、他树/工具 target 根 exit 1、743 指纹 json 全钉 256MiB）；AC-08 本审；AC-09 pending(merge) | pending_closeout_gates: 冻结固定组并集（web-delivery-controls 12/13，D-910-1 既有 ERROR 按登记不计）、canonical 落地+007 交接、具名持有 f3f98a9e（master 清理器 apply）与 54892519（T-09）、scope 折叠删除、wt-guard 拆树 | findings: **F-910-R1-01 blocking/新回归**：rust_ui.rs `cargo_managed_build` cwd=`find_ws_dir_for_cwd(target_root.parent())` 在 dev 模式上溯至 producer 仓根（吃语言仓 config：linker=rust-lld/jobs/[env]），Explicit 模式可吃显式根祖先无关 config——违反 §5.1.1 "cwd=ws_dir"/SD-01 §2；实测同上下文 PE 头 14.0/无 Rich(lld) vs 显式根 14.44+Rich；修复=cwd 传 managed.ws_dir、删上溯；重放=强制重链后 PE 头 14.4x+Rich 且 build2 0 单位。F-910-R1-02 nonblocking：rustc 产出 `counter_72e31f46.pdb`（下划线）而发布找 `counter-72e31f46.pdb` → 唯一名 bin 的 pdb 从不发布；修=从 cargo filenames 取 .pdb。F-910-R1-03 nonblocking：v08 receipt 绑 lang 64a4cd450 而 §6.1 写 19bfb579d（门脚本修复 41s 后才提交；产品代码在 64a4cd450 已全提交）→ 改措辞。F-910-R1-04 nonblocking：SD-01 §5 锁范围过度声称（锁仅覆盖 try_ensure_workspace_for；生成物/收据靠 tmp+rename 与输入快照）。F-910-R1-05 nonblocking：web api_gen 单测副作用改写真实 producer 根 54892519/context.json 的 producer_origin → 测试钉 AUTO_GENERATED_TARGET_ROOT 到 temp。F-910-R1-06 记录：v06 篡改负控驱动内未落盘，审查已补测闭合。F-910-R1-07 记录：cargo_managed_build docstring "deps/" 措辞与实际升位取件不符 | evidence: D:/autostack/plan-artifacts/plan-910/review/review-report.md（sha256 前16 11916db7645651d7；merge 时折叠入本文件后随 scope 删除）、同目录 e-build1/2.log、e-neg-a/b.log、e-snap0/1/2.txt | next: work（修 F-01 阻断，顺带 F-02/F-03/F-04/F-05/F-07；影响任务 T-02（cwd 合同）、T-03（pdb 发布）；重放范围=F-01 PE 头+0 单位、F-02 pdb 发布存在且与 exe 同轮、core/ui 定向单测；其余 AC 证据保留）

stage: goal | plan_id: PLAN-910 | plan_revision: r1 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-910/auto-lang | commit: eb77695a5 (ui 2330d56, web 3de5524, man 67add3c) | next: work

## 10. 待澄清事项

1. ~~本轮暂不实施~~ → 2026-11-02 用户 `/goal 完成计划910` 授权整链（§4.1）；work 已按合同执行完毕（6/9）。
2. ~~T-01 核落点/设置名/SDK 模式/公开命名~~ → §5.1.1 冻结：落点 `<producer>/target/generated-consumers/<ctx8>`、设置名 `AUTO_GENERATED_TARGET_ROOT`（兼容 `CARGO_TARGET_DIR`）、SDK 模式项目本地隔离；公开面（pac name/exe_name/命令/发布 exe 名）零变化，cargo bin 内部带 id8（V-02 实证 Windows cargo bin 无 metadata hash 的必要性）——判为常规等价接线。
3. ~~预算确认~~ → §5.1.1 按实际 340G 余量以 905 最保守档为上限执行，全程 df 复核（§6.1 T-06 对账）；无改配置/缩样本达标。
4. ~~D-906-1 判定~~ → man-install 腿根因精确修复并实证（d823fdd）；Web 发行门腿不触本包 oracle、维持 Web owner；D-906-2/3、013-todo 不触本包面（§5.1.1）。
5. UI007当前合同仍持原启动门；本包T-08交付有限治理结果/新组合后由路线核门，007再接受最终版本/明确实施授权，不在本包自动启动。较大AutoUI架构/全部示例主题化/AutoOS推广仍分属后继。
