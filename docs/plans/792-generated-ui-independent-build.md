---
plan_id: PLAN-792
status: execution_done
feature_name: 生成式 Rust UI 依赖身份与独立构建交付
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-16
plan_revision: 1
current_step: 6
total_steps: 8
supersedes_spec_components: [docs/specs/auto-man/project.md, docs/specs/auto-lang/ui/overview.md]
new_spec_components: [docs/specs/repository-boundaries/generated-ui-distribution.md]
touched_goals: [GOAL-007, GOAL-016]
affects: [auto-man, auto-lang/ui, repository-boundaries]
depends_on: [PLAN-789, PLAN-790, PLAN-791]
---

# [PLAN-792] 生成式 Rust UI 依赖身份与独立构建交付

## 0. 变更摘要

把779/785遗留的全新独立UI锁解析漂移升级为实际修复任务：生成最小Rust/Iced工程，统一source identity、顶层patch和依赖锁，并从可搬动的受管源码/资源交付包完成外部真实构建。为正式搬仓提供发行地基，不宣称UI已离开auto-lang。

## 1. 目标

在没有开发主检出/临时worktree路径兜底的独立消费者中，真实生成、全新解析并构建一个含事件handler和本地资产的最小Iced工程，再locked重放；依赖/资源来源完整可追溯。非目标：全UI文法迁移、全部renderer后端发行、网络发布、旧仓主分支切换及任意缓存性能承诺。

## 2. 架构方案

以本批前三项合流后的生产闭包为准。开发工程现行path模式保留；新增显式可搬动交付模式，其manifest不隐式搜索D:/autostack或编译时CARGO_MANIFEST_DIR。选择版本/git固定revision或受管本地source bundle之一并记录可重放输入；初版允许本地source bundle（依赖source归一且相对路径），不要求发布到registry。新工程必须拥有独立workspace根与自己的应用Cargo.lock，iced_widget等patch写在消费根，不能误放依赖包内。

## 3. 技术栈

Cargo metadata/tree/lock、现有rust_ui工程生成与ui_gen::rust、根iced_widget patch、显式source/resource清单与hash、隔离consumer脚本。不是新增另一套包管理器。

## 4. 需求分析与背景调查

取材基线 auto-lang@`3e65540cbe6ac019d210318dc93d17accf4ebcc4`；[输入指纹](attachments/extraction-batch-20261016/inputs.json) 记录现行 Specs、代码和 781/784/785/786 归档合同的 SHA256。overview 是导航快照，以相关模块 Spec 和实际源码为准。
用户 2026-10-16 授权“建立下一批实施计划”；本轮只建立合同及更新设计/路线，不实施 Rust、不创建执行树、不更改旧仓 refs、不发布/推送。之后按确认的合同进入 work；不能把本次建计划算作实施授权或完成证据。
781/784/785/786 均已归档；旧仓主分支和所有恢复载荷继续保留。787/788 为 AC/AutoDown 实验与集成线，不是本批全局前置，实施时按实际 diff 复核共用 manifest/数据类型热点。
785的外部no-ui独立manifest已实证；UI证明借workspace锁定rlib，记录wgpu-hal等全新锁漂移。779另记windows-core身份问题，尚不能视为独立UI发行已通过。rust_ui.rs有workspace继承生成模板、shell-pack相对path和主检出/target搜索逻辑；根Cargo.toml有本地iced_widget patch。必须逐入口区分集成开发与交付生成，不能直接批量改path。
本计划在789/790/791 review+merge后进入生产改动与最终基线冻结，可提前只读盘点。与三项共用Cargo.toml/新成员/adapter闭包，故本批收尾串行；不要求无关787/788全部结束。

## 5. 详细设计

T01冻结实际依赖图和失败复现：记录Cargo/rustc、target、feature、包版本/source/patch、resolver错误原文和已有lock；单独区分版本冲突、source重复、系统缺库和资源丢失，不靠沿用旧rlib/主锁掩盖失败。选择满足本合同的可搬动来源布局，产出精确根manifest/应用锁/资源allowlist；若当前公开依赖无法合法解析，先给可实施依赖调整方案，不无限停留为调查报告。
T02–T04修复选定Iced消费闭包和生成模式。保持集成开发模式行为；source bundle只收录实际需要的crate、其build输入和资源，不能把整个主仓复制为隐式workspace。库类型只来自单一source identity，应用根显式patch，生成代码/模板/runtime/stdlib/schema版本与producer fingerprint进入清单；重定位后不残留原绝对路径。
T05用正式生成入口产物完成真实独立build；交付包可以包含语言源码依赖（当前尚未拆完），但必须明示所含包/依赖规模，不将“搬动受管包”说成纯core或独立UI仓已完成。不要求屏蔽Cargo正常依赖缓存；必须禁用开发源码回退、不能复制原target/rlib作oracle。
最小应用覆盖UI创建、一次handler编译和本地资产解析；如需显示验证，用autoui-verifier的对应VM/Iced入口，Vue不是本计划后端。缺资源/坏hash/依赖不兼容应在受管阶段判红，不静默回退主检出；失败不切换任何旧生产入口/产物。

### 规范增量

| ID | 操作 | target | before/after rule | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/repository-boundaries/generated-ui-distribution.md | 新增选定Iced交付布局、应用锁/patch/资源producer身份及独立消费合同 | G/H搬仓硬门的首个可执行发行面 | AC-01/02/03/04 |
| SD-02 | modify | docs/specs/auto-man/project.md | 生成器显式开发/交付模式与错误阶段 | 保持旧开发路径，交付拒绝隐式兜底 | AC-02/03 |
| SD-03 | modify | docs/specs/auto-lang/ui/overview.md | 区分no-ui/锁定rlib历史证据与本轮真正fresh-lock UI构建 | 明确已证平台和未覆盖后端 | AC-01/04 |

## 6. 测试设计

实施、独立 final review、merge 分别交接，未指定外部模型/额外 agent。所有人使用本计划同一棵实施树；本轮只给拟定路径，不创建它。V 用例由 work 建成可执行 fixture；下表是合同而非已通过结果。
开发只运行受影响定向案例及必要 cargo check；先修完全部 findings、提交并完成 final review，merge 再在冻结提交执行裸 cargo t 和触面档。cargo tf 仅主检出批量到期回归，不能在每棵树运行。无相关 Rust/Schema 改动不跑 docs_gen；UI codegen 触面用 tu，不用改金样消红。

| V / owner | AC | 输入/入口 | oracle 与覆盖界限 |
|---|---|---|---|
| V01 / work | 01 | T01原失败输入、全新独立workspace，无应用lock启动解析；正式生成最小Iced app | 新Cargo.lock解析成功且完整cargo build通过；不借workspace lock/rlib，失败原文到修复可对照 |
| V02 / work | 02/04 | 交付包从A目录搬到B目录，去掉开发root环境覆盖，生成handler+本地资产 | B中cargo build --locked通过；metadata所有本地path在交付根内，source唯一；无主仓/target回退，资源hash与producer相等 |
| V03 / work | 03 | 删除allowlist资源/改hash、消费根漏patch或显式不兼容输入 | 资源/依赖阶段确定性失败，无伪成功输出、不碰原主检出/准备树；正常输入恢复复绿 |
| V04 / work | 02/04 | 当前开发工程fixture与交付工程、UI类型互传真实编译 | 开发路由及模板输出保持；同包source重复有显式拒绝/修复，不以rlib链接替代新构建 |

拟实施树 D:/autostack/.wt/lang-792/auto-lang，branch codex/plan-792-dev；独立consumer/交付包建在本树自己的受管scratch，不以junction链接。生成消费cargo build是work-owned必要验收，非全档回归，允许定向执行；记录实际网络/工具依赖与日志，不伪称离线或跨平台均过。
merge-owned裸cargo t；ui_gen改动加tu，若触VM/compiler再tv；不是自动跑tf或全部画廊。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 原UI fresh-lock失败闭合，独立应用全新解析及真实build完成，V01通过 |
| AC-02 | work | 生成交付模式可重定位，开发模式保持，V02/04通过；类型source和顶层patch一致 |
| AC-03 | work | 资源/依赖错误明确失败，不隐式寻主检出或改旧产物，V03通过 |
| AC-04 | work | crate/源码资源allowlist、锁、producer身份可回放；handler/本地资产覆盖，发布边界如实登记 |
| AC-05 | review | 独立复建/重定位/失败输入与SD通过，绑定所有依赖和锁hash |
| AC-06 | merge | 冻结门禁、规范/ledger沉淀、归档与guard清理完成；不越权切旧仓 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 实施/产出/验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 789/790/791合流、合同确认 | 绑定三项交付SHA，复放UI独立锁错误；冻结选定target/feature/source/patch/资源布局决策及失败旧臂 | 01/04 |
[✅ 已完成] 前置 SHA 绑定（789=48dae3c48/790=249a80f2c/791=1120ae3b4）+ worktree 入场记录见 §9。复现结论：779/785 fresh-lock E0277 为时变生态漂移，2026-10-16 探针 fresh 解析收敛至已提交锁版本集且 wgpu-hal 编译通过（不宣称根治，以独立锁+pin+冻结重放闭合）；布局裁定=镜像仓布局受管 bundle（六项原型验证含 workspace 继承断裂/optional path dep manifest 读/include_str 越界面）。收据 attachments/792/t01-dependency-freeze.md。
| [x] | T-02 | work | T01 | 修复选定真实依赖解析/身份冲突；统一消费根patch，生成应用独立锁；metadata/tree与fresh resolve | 01/02 |
[✅ 已完成] commit 11c3f9ca0：投影修剪（剥闭包外 optional path deps）+ 消费根 [patch.crates-io] iced_widget + pin 家族（wgpu/iced 12 项精确钉、windows/windows-core 多槽登记冻结锁控制、同名基础依赖并入基础行）+ 根级编译期输入五件随包。原型与消费 bundle fresh 解析实证：iced_widget patch 单源、pin 12/12 命中、三槽并存同已提交锁。
| [x] | T-03 | work | T02 | rust_ui及必要ui_gen路径加入显式交付根/模式；开发路径保持；定向生成器测试 | 02/03 |
[✅ 已完成] commit 11c3f9ca0：compile_rust_ui_app 相位抽取（开发/交付共享，开发路径回归 31 通过/2 预存红=主检出基线同款）+ generate_rust_ui_delivery + auto gen --delivery-root CLI（非 rust 后端显式报错）；ui_gen 无路径逻辑零改动。定向生成器测试 5 项绿（pins 单/多槽、根 manifest 形态、投影幂等、端到端 bundle、同包重复拒绝）。
| [x] | T-04 | work | T03 | 受管source/resource bundle清单、producer/hash与缺失拒绝；不复制整workspace或原target | 03/04 |
[✅ 已完成] commit 11c3f9ca0：delivery_copy_dir 跳过 target/.git/node_modules；资源 allowlist（src/front 非 .at + blake3）；delivery-manifest.json（producer git/ui_gen 指纹、逐 crate digest、闭包规模披露含 auto-lang 全库）；外来目录/同包重复/缺输入显式拒绝；重定位自检（生成清单零 producer 绝对路径）。
| [x] | T-05 | work | T04 | 新test/fixtures/modularity/generated-ui-consumer与重定位driver；V01..04真实build/locked及失败控制 | 01–04 |
[✅ 已完成] commit 8a83e1f39（fixture+driver+SD 素材）：driver.py 14/14 全绿——V01 fresh 解析（patch 单源+pin 12/12 命中）+ 真实 build 173s 产物在位；V02 重定位（不带 target/）locked build 144s + metadata 零越界 path 依赖 + 资源字节=producer；V03 四负控（等长坏 hash/缺资源/漏 patch exit101/不兼容 pin exit101/外来目录拒绝不覆盖）+ 恢复复绿；V04 开发路由不变（project-local rust-workspace+继承清单）+ dev/交付 main.rs 逐字节同源。收据 attachments/792/driver-receipt.json。首轮 3 FAIL 均为 driver 自身路径规范化/资产层级 bug（已修+等长篡改加强），重跑全绿。
| [x] | T-06 | work | T05 | source/lock/资源hash、平台范围/未迁依赖及构建日志收据、SD素材；提交代码与可复放入口 | 01–04 |
[✅ 已完成] 收据三件入 attachments/792/（t01-dependency-freeze.md：工具链/环境身份+原失败复现结论+闭包冻结+六项原型裁定+预存红登记；driver-receipt.json：14 用例+toolchain+scrubbed env 清单）。SD-01（新 generated-ui-distribution.md）/SD-02（auto-man/project.md 交付模式节）/SD-03（ui/overview.md 现状节）已在 worktree 备稿待 merge 正式沉淀。代码两提交：11c3f9ca0（生成器+CLI）、8965cc994（fixture+driver+SD 素材+exclude 登记）；可复放入口=driver.py（根 Cargo.toml exclude 注释在案）。
| [ ] | T-07 | review | T06 | 独立fresh/locked重定位与负控、生成/资源身份及SD冻结 | 05 |
| [ ] | T-08 | merge | final pass | 裸t+实际触面tu/tv，沉淀/归档/guard清理，后续G/H入场证据交接 | 06 |

## 9. 复审记录

2026-10-16 stage=new | plan_revision=1 | outcome=blocked（合同preflight通过；789/790/791交付前置未满足） | next=三项合流后work；此前仅只读准备 | 0/8；fresh-lock/独立build尚未执行，不借785的rlib证据宣布通过。这里的blocked是交接记录，不新增Plan后端状态，frontmatter维持drafting。

2026-10-16 stage=work | PLAN-792 | plan_revision=1 | 入场执行 | worktree=D:/autostack/.wt/lang-792/auto-lang | branch=plan-792-dev（repo 命名规约覆盖合同拟称 codex/plan-792-dev；PLAN-726 组布局含 auto-down 兄弟） | base_commit=ccf66f013 | 前置绑定：789=48dae3c48b8762dbe35809bccc45238c76e20955、790=249a80f2ca3a0b6f64fb21e230729b704214855b、791=1120ae3b4102d004dc11a3c8ebc0262a2c5be5d2（均 ff-only 落地+归档验证） | 用户显式授权 /auto-plan:work 实施。

2026-10-16 stage=work | PLAN-792 | plan_revision=1 | outcome=pass（work-owned 全部完成，交付 final review） | worktree=D:/autostack/.wt/lang-792/auto-lang | branch=plan-792-dev | base=ccf66f013 | code=8a83e1f39（11c3f9ca0 生成器/CLI → 8a83e1f39 fixture/driver/SD 素材/exclude） | deps=无依赖仓改动（auto-down 兄弟 detached 只读） | tasks=T-01..T-06 | evidence=AC-01 V01 fresh 解析+真实 build（173s）产物在位，原失败闭合方式=独立锁+pin+冻结重放（时变生态漂移如实登记不冒称根治）；AC-02 V02/04 重定位 locked build（144s）+metadata 零越界+dev 路由与模板字节同源，类型 source 与顶层 patch 一致（iced_widget patch 单源）；AC-03 V03 四负控确定性红+恢复复绿，零主检出/原 target 回退；AC-04 闭包 9 crate+patch+5 根级输入+资源 allowlist 全 hash/digest 入 delivery-manifest，含 auto-lang 全库如实披露（33.7MiB）。定向门=生成器测试 5/5 绿 + rust_ui 回归 31 通过/2 预存红（=主检出基线） + cargo check auto-man/auto 0 错。收据=attachments/792/{t01-dependency-freeze.md,driver-receipt.json} | blockers=无 | next=review（T-07）

  **readiness 明细**：work-owned AC-01..04 全部绑定当前测试/脚本证据（driver.py 14 用例可复放，绑定 8a83e1f39）；in-contract findings 空；受影响先例重放=779/785 的 fresh-lock 失败类经 V01 闭合（不再借 workspace rlib），开发路径回归=既有 rust_ui 测试基线不变。**merge-owned 待办（不阻塞 work 交接）**：裸 cargo t + 触面 tu（rust_ui/ui_gen 生成器改动）按 2026-10-14 时序裁定在冻结提交执行；SD-01/02/03 正式沉淀 + INDEX/specs.json；KNOWN-DEBT 候选沉淀（见 §10——主检出 KNOWN-DEBT 文件当前被并行会话未提交修改占用，本计划候选暂记 §10 待 review/merge 沉淀）。

## 10. 待澄清事项

版本发布渠道和registry帐号不作为初版本地交付包硬前置；无远端发布授权。若依赖升级必须改变UI/语言能力、金样或支持平台，先呈现具体冲突修订，不能扩大范围升级整个图形栈或降低fresh-lock验收。完整UI/Man仓外置及所有后端/资源发行另立后续计划。

### 债务候选（work 期间发现，待 review/merge 沉淀 KNOWN-DEBT——文件当前被并行会话未提交修改占用）

- **rustfmt 1.98 全仓漂移**：当前 stable rustfmt 对 master 的 auto-man/auto crate 产生 ~42 文件大规模重排（master 在该工具链下非 fmt-clean；历史 fmt 门疑用旧版 rustfmt）。本计划改动仅触及 3 文件、未引入全仓重排；fmt 工具链钉扎/门禁口径需单独裁定。
- **plan734_commands_fixture 编译红（预存）**：`cargo nextest -p auto-man`（全测试目标）下 tauri 2.12.4 `Wry` configured-out 编译红，主检出同款——影响 nextest 全目标档，裸 `--lib` 不受影响。
- **rust_ui 两预存红（threads=1）**：`merged_api_client_crud_fallback_for_uncovered_endpoints` 与 `test_shell_pack_lib_freshness`（主检出基线同款 31 通过/2 红）；并行裸跑另有 4 例 env/state_file 竞争族（nextest 进程隔离不现）。
- **交付边界如实登记**：bundle 含 auto-lang 全库（33.7MiB，语言源码尚未拆完）；已证平台仅 Windows/x86_64-pc-windows-msvc；`examples/rust-workspace/`（含其 Cargo.lock）整目录被 .gitignore 忽略，任意检出不保证在位（pin 源因此用根 Cargo.lock）。
