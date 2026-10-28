---
plan_id: PLAN-879
status: archived
feature_name: AutoGraphics纯源码迁入auto-ui与单一权威切换
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-29
plan_revision: 1
supersedes_spec_components:
  - docs/specs/auto-graphics/project.md
  - docs/specs/auto-graphics/prototype-profile.md
  - docs/specs/auto-graphics/integration-contract.md
  - docs/specs/auto-graphics/auto-source-driver.md
  - docs/specs/auto-ui/project.md
new_spec_components:
  - docs/specs/auto-graphics/source-repository-and-sdk.md
touched_goals: [GOAL-008]
affects: [auto-graphics, auto-ui, repository-boundaries]
depends_on: [PLAN-875, PLAN-878]
current_step: 10
total_steps: 10
---

# [PLAN-879] AutoGraphics纯源码迁入auto-ui与单一权威切换

## 0. 变更摘要

875角色/相机链与878静态审计已交付；在下一批公共运行协议开发前，将AG两个experimental workspace的生产源码、测试、工具及必要小资源从auto-lang/autographics-dev迁入auto-ui/experimental。先保全旧SDK、证明独立消费，再按可恢复的两仓顺序切换唯一开发权威。与[880 GY-B](880-autographics-gallery-character-and-cameras.md)固定消费线并行。

## 1. 目标

- 新auto-ui检出可独立准备旧SDK、验证来源并构建/运行原公开AG工程，不借旧工作目录、生产者target或网络取回SDK。
- 两个嵌套workspace和两把Cargo.lock保持独立；auto-ui默认五包不新增AG依赖/构建负担。
- 单一开发权威明确切至auto-ui；auto-lang保留历史、设计/Specs索引及显式归属指引，不留两套持续演进的源码。

非目标：重构/抽crate、更新SDK或依赖、ACE/HIR/增量/HMR、生产UI/声明融合、切默认生产入口、Web/移动适配、新图形能力、改Gallery引擎锁、远端推送/发布、历史过滤/改标签。M2验收分母和旧版本行为保持；迁仓成本单列。

## 2. 架构方案

目标仍为`auto-ui/experimental/autographics-prototype`与`autographics-auto-driver`，各自`[workspace]`和锁；不加入根`crates/*`，不接生产auto-ui-runtime。迁仓只改变源码归属及必要路径/准备/身份工具，正常玩法仍由真实Auto执行。

来源链：旧repo+完整commit+path+内容SHA → 迁移清单 → 新repo+完整commit+path+内容SHA。路径工具的必要修正逐文件给出原因；行为源码/资源/两锁应恒等，文本换行规范化须同时记录原字节与规范化hash。不把旧提交虚构为新仓祖先，不改历史身份来伪造连续性。

## 3. 技术栈

Git对象导出/树清单、Python标准库、现有prepare_sdk/verify_source/package_consumer与Rust独立workspace。只修迁仓所必需的脚本/配置/说明，无新增业务依赖。

已核入口：driver `tools/{prepare_sdk.py,verify_source.py,package_consumer.py,gen_plan875_identity.py,verify_plan875.py}`、`research/{baseline.json.txt,source-lock.json,sdk-Cargo.lock.txt}`；AGP/driver各Cargo.toml/lock。新路径：driver `sdk/frozen/`（681文件源快照，不是新开发SDK）、新迁仓来源清单/验证器（具体命名T-02冻结）、语言仓新归属指引。现有`.sdk-source`继续为实际依赖路径，由受检快照实体化，不用链接。

## 4. 需求分析与背景调查

授权：用户2026-10-29同意继续规划“纯迁仓＋GY-B”。本轮仅起草合同、记录基线和路线；未授权立即搬仓/实现、建树、新建agent、推送或发布。执行需后续work指令，既定范围内不重复索取许可。

权威输入：`docs/specs/overview.md`、`auto-graphics/{overview,project,prototype-profile,integration-contract,auto-source-driver,runtime-game-cameras,game-character3d,platform-boundary-audit}.md`、`auto-ui/project.md`；设计[分离门](../design/autographics/repository-move-and-integration-gates.md)，[878评估](../reports/autographics-platform-audit-878/migration-readiness.md)。规划只提出SD，不修改canonical Specs。

已交付AG固定源`1cc68aba012f76951af2556763a5fb21be6a6c8b`（875最终交付，autographics-dev）；auto-ui规划观察master为`154a39657e96bb88806b4a31d105a91d633758fe`，clean，root members仍crates/*，已不同于878审计的7095f53。work须复核目标自该pin差异及owner，不把旧审计直接当新目标实测。

SDK逻辑pin`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`对象仍不可达；保留副本`D:/autostack/plan-artifacts/plan-860/consumer-package/experimental/autographics-auto-driver/.sdk-source`实测681文件、13,681,490字节、最大文件1,528,601字节（均<5MB）。须按source-lock逐文件核验，不以数目/大小代替身份。875消费包保留最新`research/plan875-identity.json`，但875 Git树未跟踪该文件；新迁仓不能照搬package_consumer即宣称闭包可用。两锁/关键blob与全部观察指纹见[本轮基线](attachments/879/planning/baseline.md)。

875角色包络：宽/稀疏128、密集深度64，127 solids初始化超预算受控拒绝；相机读免费，控制器32credits封顶。迁仓不重新解释这些限制。M1历史源11911a4a与当前tag peel679681b3映射未核实，本计划只迁875固定源，不更改标签/历史SHA，也不宣称解决该差异。

## 5. 详细设计

### 5.1 唯一树、入场与冻结

未来唯一组`D:/autostack/.wt/lang-879/`，auto-lang树从上述AG源建`plan-879-dev`；auto-ui树从实施时已提交目标master建`codex/plan-879-dev`，记录完整SHA及与154a3965差异。每仓一树，同树顺序work/独立review/merge；主语言检出只簿记。默认master建组脚本不可直接套用AG源码基面。无junction/symlink，不替其他owner接管WIP。

T-01确认875/878交付收据与当前owner；T-02先冻结`attachments/879/work/move-contract.md`：全量路径清单/保留排除理由、源与目标base、默认UI闭包基线、SDK snapshot身份、导出补齐、可执行测试ID、切源/回退步骤与期限。若停写时autographics-dev偏离875，先分类差异并修订合同，不覆盖新提交。

两个workspace内受管生产/测试/工具/小资产逐项核销；root构建栈配置、必要未跟踪生成身份也有显式来源。target、PDB、exe、截图、原始日志/复审证据不作为源码迁移；历史收据在旧仓保全并给定位，不按目录名盲删必要测试夹具。逐文件大小清单遵守5MB门，遇超限先选择受检固定包，不自动豁免。

### 5.2 旧SDK保全与冷准备（必须先于迁入闭环）

选择可版本保全的源快照：新driver `sdk/frozen/`保存锁定681文件（包括原SDK Cargo.toml/lock与许可），正常源码身份独立列清单、总hash、逻辑pin和“保留副本、非可达Git对象”来源说明；不可把SDK文件改为当前语言仓版本。`.sdk-source`继续是生成的实际消费副本且忽略target；prepare_sdk增加显式快照准备路径，逐文件锁校验后实体化，拒绝已存在目录覆盖、逃逸/链接、缺文件/多文件/hash错。不得静默回退当前SDK。

快照受git版本控制（生产依赖源，不是attachments证据）；T-02确认ignore规则允许精确跟踪，身份生成器排除构建输出并覆盖快照来源。prepare/verify/package的正常入口与冷启动说明一致。新A目录只拿新仓源快照，删除生成SDK后重建；坏快照在临时输出发布前拒绝，半准备目录不被后续verify误认成功；修复后同入口绿。再搬B并重开验证，不依赖原绝对目录/全局cache/链接。离线registry缓存可作为声明的第三方输入，不能假称无工具链/零缓存即可离线构建。

### 5.3 迁移、配置与身份

先从875固定Git对象导入，不读取源WIP。875未跟踪identity从保留消费包取证，或按已提交gen_plan875_identity的确定规则重建，逐文件对齐固定源；不能有证据便捷性所需的无来源额外源码。新来源身份覆盖实际构建配置、测试夹具、快照和必要工具，历史身份留作历史，当前验证器明确选新身份；新身份/导出不自含易变报告。

显式保留32MiB PE主线程栈及16MiB worker配置（consumer根配置是旧遗漏风险），AG目标目录仍各自在workspace内，不继承auto-ui生产patch/default targets。两把AG锁字节保持；auto-ui根manifest/锁和crates源码不作功能修改。工具仅做必要路径适配，算法/Auto API/scene格式/预算改动不在本Plan。

### 5.4 单一权威切换与可恢复顺序

work在两树完成候选：目标迁入/独立消费可验证；源树将迁出目录替换为明确指引（仅在计划树，主分支未切），保留旧历史与875 pin。独立review绑定两个冻结提交，检查候选与切换脚本/步骤。

merge先重核源tip与两仓基面/owner，短暂停写：①保存两repo可恢复ref/内容清单及目标候选commit；②先落目标候选但标prepared、开发权威暂仍旧源；③确认目标HEAD、默认隔离及消费证据后，再落语言autographics-dev迁出/指引；④主检出Specs/索引/权威收据更新，明确新repo+commit，解除停写后只在新仓开发。不能要求跨Git原子事务；②后③前失败必须报“prepared、未切源”，继续保留旧权威，或按恢复ref回退目标。不得覆盖已发生的其他提交/强推，不能两边都声称active。

只在目标已可消费、两冻结提交review通过之后实际落迁出；旧源历史/refs保留不做历史重写。迁仓后语言仓master仍主要知识，autographics-dev保历史+指引，不再持有活跃AG源码。auto-ui默认五包/闭包/锁无变化，生产UI融合pending。

### 5.5 并行与保持面

880从旧875固定对象独立导出消费包，不等879完成，也不自动切新仓版本；879不写ag-examples。两线target/SDK准备/报告分离，GPU单实例；主检出Plan/ledger/知识合入串行。下一批对象/触发协议实施等本计划切源后开展。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/source-repository-and-sdk.md | 旧归属→新单一权威/内容映射/冷SDK准备/恢复/保持面 | 可重建来源 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-graphics/project.md; prototype-profile.md; integration-contract.md; auto-source-driver.md | 源在autographics-dev→auto-ui experimental；实验与生产I2/I3分离保持 | 修正当前入口，历史不改写 | AC-03/05 |
| SD-03 | modify | docs/specs/auto-ui/project.md | 默认五包→默认五包不变＋独立AG实验所有权 | 防迁仓被当生产融合 | AC-04/05 |

## 6. 测试设计

用户未指定实施模型/agent；work自查、独立上下文最终review、merge顺序同树。当前只规划，不运行下列验证。门遵守2026-10-28裁定：变更相关∩约1/10可复现抽样，seed/选择全集/样本记录；来源与安全不变量采用小型直接正反例，不用随机漏检替代必要AC。全量AG里程碑档pending。暂停所有抢焦点/置顶/OS输入自动化，不运行character/camera窗口探针；历史OS证据仅在相关实现/配置/输入未变时复用并声明身份边界，不拿重建exe冒充旧runner。

| Case | AC/规则 | 实际入口/输入 | 区别错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 来源闭合 | Git树清单、SDK锁、新旧映射，含缺件/错hash/夹具遗漏 | 全受管路径逐项处置，必要输入有内容来源；删件/多件/错hash拒绝后恢复绿 | T-02/03；不证明M1映射 |
| V-02 | AC-02 SDK冷准备 | prepare_sdk新快照路径；无旧repo/空生成SDK、篡改副本 | 681逐字节一致；坏包受控拒绝、无可接受半包；修复/冷建/搬B后verify绿 | T-03/04；不换SDK |
| V-03 | AC-03 独立消费 | 新identity --controls、verify_source、package_consumer导出A→B | 两锁恒等、target不逃逸、17本地包/29边保持；构建栈在场；离线实际Auto源码/Host无窗口GPU结果与875接受位置/投影一致 | T-04/05；非OS重测 |
| V-04 | AC-04 默认隔离 | auto-ui根metadata --locked --offline --no-deps+manifest/锁差异；静态path闭包 | 默认成员仍五包，无AG或新SDK快照依赖边，锁/五包零功能diff；旧默认源码目标抽样check仅必要时 | T-01/05；不重跑全UI |
| V-05 | AC-05 切换/恢复 | 两树候选+指引、新消费者；模拟目标prepared/源落地前失败 | 此时旧权威仍可用；恢复不覆盖他线；最终唯一新权威、新包下次读取/重建成功 | T-05；候选演练，实际落地merge所有 |

定向命令先在T-02冻结实际参数：`python tools/prepare_sdk.py --verify-only`、`python tools/verify_source.py`、新迁仓identity验证器`--controls`、包内`cargo build --locked --offline --bin autographics-authoring`；AG原测试从`behavior_source_identity`、`character3d_samples`、`runtime_camera_api/runtime_camera_host`选受影响命名case（单线程、无原生窗口），记录非零选择与预期。新来源/SDK准备正反例须全闭合；无必要不新增通用审计框架。GPU实际像素由既有无窗口捕获路径检查，不能只比较矩阵/哈希自报绿。若无窗口消费者公开路径不可用，T-02回new，不私加生产probe/恢复OS自动化。

根crate源码零修改，不跑根cargo t/tf/tv/tu/docs_gen；不为搬仓跑全AG或UI档。迁仓前后的实际依赖/构建环境变化必须针对性验证，复用仅覆盖确实未变部分。原始证据本地`D:/autostack/plan-artifacts/plan-879/`，attachments仅md摘要/契约，记录完整命令/exit/计数/路径/hash。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 固定两仓输入/唯一树/逐文件迁移清单、内容映射、历史来源保持，V-01 |
| AC-02 | work | 旧SDK可版本保全并从新源冷准备、坏包拒绝/恢复/A→B，V-02 |
| AC-03 | work | 两独立AG workspace/锁/栈/新身份/consumer闭合，真实Auto与无窗口内容对照成立，V-03 |
| AC-04 | work | 默认UI成员/依赖/锁与五包保持，V-04，无生产融合或平台声明 |
| AC-05 | work | 候选迁出指引/唯一权威与失败恢复演练、并行固定pin保持，V-05 |
| AC-06 | review | 同两树独立核AC/V/SD/来源/遗漏/权限与warnings/fmt，绑定两冻结commit；merge门pending |
| AC-07 | merge | 冻结两commit/输入复核、受影响抽样与证据复用审计通过，切源前所有必需门已闭合 |
| AC-08 | merge | 目标后源按序实际落地、两仓SHA/权威收据、Spec/ledger/索引/归档、guard clean清理实绩 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际操作与产出 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 875/878已交付，用户启动 | 两仓owner/base/唯一树、停写窗口候选、默认闭包与SDK输入核验 | AC-01/04 | [x] |
| T-02 | work | T-01 | move-contract/全清单/来源/切换恢复与无窗口整路径可行性先冻结；不可行回new | AC-01..05 | [x] |
| T-03 | work | T-02通过 | 两树候选导入、快照保全与prepare/路径工具/new identity适配、V-01/02 | AC-01/02 | [x] |
| T-04 | work | T-03 | 新源导出fresh consumer、A→B冷准备/独立构建/公开源码无窗口对照V-03 | AC-02/03 | [x] |
| T-05 | work | T-04 | UI默认隔离、源迁出指引候选/恢复演练、定向/fmt/两树提交/SD摘要 | AC-01..05 | [x] |
| T-06 | review | T-05 | 独立上下文逐项复核两冻结提交/清单/路径/失败恢复/SD | AC-06 | [x] |
| T-07 | merge | T-06 pass | 冻结抽样/来源/受影响对照及复用审计，失败回work受影响复审 | AC-07 | [x] |
| T-08 | merge | T-07 | 两仓停写，目标prepared→源迁出→权威切换，失败按合同恢复 | AC-08 | [x] |
| T-09 | merge | T-08 | 主语言检出知识/ledger/索引、目标文档归属/维护入口、Plan归档收据 | AC-08 | [x] |
| T-10 | merge | T-09 | 两仓各wt-guard clean后清树/分支/空组目录；必要本地产物/恢复refs保全 | AC-08 | [x] |

## 9. 复审记录

2026-10-29 new r1：`stage: new | plan_id: PLAN-879 | plan_revision: 1 | outcome: pass | next: 用户启动work后T-01→T-02`。875/878前置已交付；范围为纯归属、SDK保全及必要路径工具，不含生产融合/ACE/新能力。T-02负责全清单/局部命名/整路径可行性冻结；失败回new，不默认更换SDK或放宽验收。本轮只读核验与规划，未创建实现树/运行Cargo/GPU/复制SDK/切源/新agent/远端操作，所有AC/V/T/SD未执行。

2026-10-29 new文档预检：0..10节、10个唯一T、8个唯一AC、5个V、stage/依赖与SD覆盖已核；两Plan及四份路线文档共90个本地链接可解析、879/880编号各唯一、中央next-id=881。这是合同结构/来源/并行写面自查，不是独立final review或实现验收。旧SDK681文件与875固定source-lock逐项hash一致，仅作规划输入；无Cargo/OS/GPU运行。

2026-10-29 work r1 pass：`stage: work | plan_id: PLAN-879 | plan_revision: 1 | outcome: pass | worktree: D:/autostack/.wt/lang-879/{auto-lang,auto-ui,auto-down} | branch: plan-879-dev / codex/plan-879-dev | base: 1cc68aba(源) / 154a3965(目标，与规划pin零漂移) | code_commit: 4adf2d037a1804741adc9c5a012f08cfa566439d(auto-ui候选) + f07b5ec8adc96c3a183882365920a34b570d07b2(语言侧迁出指引候选) | dependency: auto-down c3464d0a detached 兄弟（组内cargo解析，PLAN-726 T-04惯例） | task_ids: T-01..T-05 | evidence: docs/plans/attachments/879/work/move-contract.md（冻结合同）+ 本地 D:/autostack/plan-artifacts/plan-879/ | blockers: 无 | next: review（T-06 绑定两冻结提交）`。

work r1 证据摘要（命令/退出码/关键数字，原始输出在 plan-artifacts/plan-879/）：
- **T-01**：autographics-dev=1cc68aba 无worktree占用零漂移；auto-ui master=154a3965 clean，closure blob 三件与规划一致（6d528f03/a1060a44/a7dd2c11）；保留副本对 source-lock 逐文件 sha256 **681/681 一致、0缺0多0错、无链接**，字节 13,681,490、最大 1,528,601（<5MB）；702 迁移文件最大 1,465,711B；17本地包/29边为 source-lock+baseline 记录值（V-03 实测复核）。
- **T-02**：move-contract 冻结（§3 全量处置：702 恒等迁移、未跟踪产物不迁移、C 新增/D 修改逐项理由、§5 V-01..05 实际命令与 26 个无窗口测试 ID、§6 切换/恢复顺序）。
- **T-03/V-01**：两树建成（auto-lang 树自 1cc68aba 建 plan-879-dev，非 master；auto-ui 树自 154a3965 建 codex/plan-879-dev）；git archive 导出 702 文件；快照复制后 681/681 原地核验（aggregate 4c15c86f…）；对**已提交候选树**逐 blob 对照：**695/702 恒等 + 7 个声明修改（.gitattributes/.gitignore/两README/prepare_sdk/verify_source/package_consumer）+ 686 新增（681快照+2配置+gen/verify+identity）**，零未声明差异（plan-artifacts/plan-879/{old-blobs-z,committed-blobs-z}.txt）。167 个被 driver .gitignore 规则命中的历史被跟踪文件按迁移程序一次性 `git add -f` 入库（记录于合同§3处置A）。sdk/frozen 加 `-text` 属性保证字节往返（681 blob 全部与工作副本字节相等实测）。
- **T-03/V-02**：`gen_plan879_identity.py`→**1387 文件身份（702迁移+4声明新增+681快照）**；`verify_plan879.py --controls` **PASS**（删源件/篡改快照字节/篡改工具/未列新增/错identity哈希/错aggregate 六控全拒、还原绿；702 blob 全绑定；快照681对source-lock全符）；`prepare_sdk.py --from-snapshot` 冷准备 **PASS**（A删生成SDK后重建，`--verify-only` PASS）。
- **T-04/V-03**：package_consumer 导出 consumer-A（2069文件，verify_source 全链绿）；A→B 整体复制；B 删SDK→`--from-snapshot` 冷准备绿→`verify_source` 绿（**17本地包/29边/forbidden=0/AGP锁保留**，target不逃逸内建断言过）；`cargo build --locked --offline --bin autographics-authoring` **exit 0**（1m23s；auto-lang-core 146条警告为冻结SDK既有内容非本次引入）；PE `SizeOfStackReserve=33554432`（32MiB 主线程栈实测自随迁 .cargo/config.toml，零环境变量——修复旧导出缺口）；4 个无窗口测试档 **26/26 PASS**（`--test-threads=1`：behavior_source_identity 1、character3d_samples 1、runtime_camera_api 12、runtime_camera_host 12，含 character3d 位置/坠落分异、runtime相机 clamp/letterbox/投影/同tick/20轮寿命 oracle——与 875 冻结断言一致）。
- **T-05/V-04**：auto-ui 根 `cargo metadata --locked --offline --no-deps` = **恰五包**（compiler/dialect/runtime/tools/shell-pack）；`git diff 154a3965..HEAD -- Cargo.toml Cargo.lock crates/ .cargo/ .config/` **零diff**；默认侧 experimental 引用 **0**；默认源码零改动故不跑抽样 check（合同"仅必要时"）。
- **T-05/V-05**（候选演练）：旧权威在位（autographics-dev@1cc68aba 树 702 文件可读）；回退点=auto-ui master 154a3965 未动；本线仅新建 plan-879-dev/codex/plan-879-dev 两ref，他线 worktree（lang-864/872/os-051）未触；新包重建成功=B 实证。当前即"目标prepared、源未落地"演练态：旧权威继续可用成立。
- **fmt**：driver/prototype `cargo fmt -- --check` 双 **exit 0**。Python 工具全链实跑无语法/运行错误。
- **SD 确认**：SD-01/02/03 与实际一致（新来源身份/快照/冷准备=SD-01；源归属改 auto-ui experimental、实验生产分离保持=SD-02；auto-ui 默认五包不变+独立AG实验所有权=SD-03）；规范正文deposit归 merge（T-09 主检出）。
- **pending（不属work）**：T-06 独立 review 绑定 4adf2d03+f07b5ec8；merge-owned：冻结抽样/来源复用审计（T-07）、两仓落地切换（T-08）、知识/ledger/归档（T-09）、guard清理（T-10）。根 auto-lang crates 零改动，未跑根 cargo t/tf/tv/tu/docs_gen（Category-A/合同裁定）。

2026-10-29 goal round 1：`stage: goal | plan_id: PLAN-879 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-879/{auto-lang,auto-ui} | commit: 4adf2d03(auto-ui) / f07b5ec8(auto-lang) | next: review`。用户 /goal 授权整链（work/review/有界修复/merge），本轮执行 work 阶段并到 execution_done；下一轮进 review。
2026-10-29 merge T-07/T-08：`stage: merge | plan_id: PLAN-879:r1 | outcome: pass | completion_kind: delivered | reviewed_commit: 4adf2d03(auto-ui)+f07b5ec8(auto-lang) | delivery_commit: 4adf2d03(auto-ui master)+f07b5ec8(autographics-dev) | landing: ①冻结审计 PASS（两候选HEAD/干净树/基座未动/两锁blob=合同pin 47176cc0+bc7287b5/双ff证明；唯一脏项=未跟踪__pycache__运行中间物非评审内容）；②auto-ui master 154a3965→4adf2d03 `--ff-only`（无merge commit；落地后烟测=cargo metadata 恰五包）→prepared；③确认消费证据（V-03绑4adf2d03内容）后 autographics-dev 1cc68aba→f07b5ec8 `git push .` ff-only（两目录仅存指引README；语言master零触）→单一权威=auto-ui。恢复锚=154a3965/1cc68aba 均为祖先永久保留；全程无强推/无历史改写/未触他线worktree | gates(T-07): 根auto-lang crates零改动→裸cargo t/tf/tv/tu/docs_gen按合同§6不适用；AG受影响面=26无窗口测试+controls+隔离检查已在冻结提交上绿；全量AG套件=里程碑档pending（2026-10-28裁定不阻合并）；批回归：.last-batch-regression.json last_covered=875（2026-10-28T21:55Z，本日），879%5=4≠0且<48h → 不到期 | next: T-09 deposit/archive → T-10 cleanup`。

2026-10-29 merge T-09：`canonical_spec: docs/specs/auto-graphics/source-repository-and-sdk.md（新，SD-01，落地commit见master）+ project/prototype-profile/integration-contract/auto-source-driver 四文件 879 横幅与落点更新（SD-02）+ auto-ui/project.md 所有权节（SD-03）+ overview/plans 索引 + INDEX.md（spec-index.py 再生）+ design/autographics/repository-move-and-integration-gates.md 状态行 | ledger: .autoos/specs.json v122→v123（guarded offline projection，store writer 不可达——HTTP探针无响应，沿875/878先例；字节级备份 plan-artifacts/plan-879/specs.backup.v122.json；脚本化surgical编辑；scope守卫=未触item字节恒等+节序不变+顶层字段恒等；schema六节+新item七字段完整；读回验证；+designs P879-SOURCE-REPO/+reviews P879-REVIEW-FINAL/+reports P879-MERGE/五既有item注记） | archive: 本文件→docs/plans/archive/879-autographics-source-move-to-auto-ui.md status: archived | landing-not-deployment: 仅 experimental/（auto-ui）+docs/（auto-lang）触面，生产二进制/web bundle/daemon 零接触 → 显式观察项：无生产面产物需要重建 | next: T-10 cleanup`。

2026-10-29 review final pass：`stage: review | review_scope: final | reviewer_identity: 会话内独立子代理（agent_4cd3f45f，新建上下文非实现上下文，自跑全部命令；局限=同会话派生，已按工件重推导不采信执行摘要） | plan_id: PLAN-879 | plan_revision: 1 | outcome: pass | worktree: D:/autostack/.wt/lang-879/{auto-lang,auto-ui} | branch: plan-879-dev / codex/plan-879-dev | reviewed_commit: f07b5ec8 + 4adf2d03 | base_commit: 1cc68aba / 154a3965 | dependency: auto-down c3464d0a detached | spec_inputs: SD-01/02/03（未发布，deposit归merge） | acceptance_results: AC-01..AC-06 全 pass（695/702 blob恒等+7声明修改+686新增零未声明；快照681/681逐文件符；verify_plan879 --controls exit 0；consumer-B 26/26 无窗口测试复跑绿；PE reserve=33554432 实证；两锁字节恒等且 blob=合同pin；默认恰五包/根零diff/0引用；源树两目录仅存指引README、700D+2M、历史@1cc68aba可读、回退点154a3965未动） | pending_closeout_gates: T-07冻结抽样/来源复用审计、T-08两仓实际落地切换、T-09规范deposit/ledger/归档、T-10 wt-guard清理；全量AG里程碑档pending（2026-10-28裁定不阻合并）；根t/tf不适用（零改动） | findings: 无（O-1 aggregate口径两套自洽免行动：快照相对4c15c86f=复制步核验口径，身份内全路径5f825027=验证器重算口径；O-2 .cargo配置仅复述栈设置、linker/jobs/alias未复述=合同范围，消费构建/PE实测不依赖） | evidence: 子代理自跑记录+D:/autostack/plan-artifacts/plan-879/ | next: merge`。

## 10. 待澄清事项

无新增产品决策；用户本轮只要求规划，实施等待后续明确work指令。work须确认实际owner/短暂停写安排并复核auto-ui新base；如发生他线冲突或必要快照无法逐字证明，具体阻塞回new，不覆盖/强推。SDK快照681源文件允许随源码版本保全（每文件<5MB，非复审证据）；若盘点发现超大新增件，先按仓库门处理。迁仓仍不解决M1标签映射，也不放行UI融合/ACE或跨端。

work r1 后新增：§10 前两项已由 work 消解——owner 已核（两分支均无他线占用、目标 pin 零漂移），快照已逐字证明（681/681）。目标 base 与规划 pin 一致（154a3965），无差异需复核。无新增待澄清。

2026-10-29 merge 收尾（T-10 cleaned）+ goal close：`stage: merge | plan_id: PLAN-879 | outcome: pass | completion_kind: delivered`。**cleaned**：三树 wt-guard 全 clean（无 reparse point）→ worktree lang-879/{auto-lang,auto-ui,auto-down} 移除；plan-879-dev（tip f07b5ec8 经 merge-base --is-ancestor 证明在 autographics-dev，主检出在 master 故 -D，同 875 先例）与 codex/plan-879-dev（tip 4adf2d03 在 master，-d 即删）删除；auto-down 兄弟残余 gitdir 指针经 worktree prune 清除（其余 lang-864/872/os-051 注册属他线未触）；组目录 lang-879 移除。plan-artifacts/plan-879 回收：consumer-B target 树 4.6G（CACHEDIR.TAG 确认）+164 个 .pdb 删除；12 个测量/证据相关 .exe 锚保留于 exe-anchors/（4 二进制 root+deps 形态 + 4 测试二进制）；保留 consumer-A/B 源码包、blob 清单、ledger 脚本与 v122 备份、review 记录；现总量 875M。**landing**：auto-ui master=4adf2d03（ff-only）、autographics-dev=f07b5ec8（ff-only）、本仓 master=77f9301c7（deposit）。**批回归**：不到期（last_covered=875 当日、879%5=4）。**landing-not-deployment**：experimental+docs only，无生产二进制/web bundle/daemon 触面，无 stale 项。最终 10/10（T-01..T-05 work、T-06 review 独立上下文 pass、T-07..T-08 merge 落地、T-09 规范/ledger v123/归档、T-10 guard 清理）。

2026-10-29 goal round 2：`stage: goal | plan_id: PLAN-879 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-879/{auto-lang,auto-ui} | commit: 4adf2d03/f07b5ec8 | next: merge`。独立子代理最终 review pass 后路由 merge。

2026-10-29 goal round 3：`stage: goal | plan_id: PLAN-879 | plan_revision: r1 | round: 3 | from: merge | to: done | status: archived | outcome: pass | worktree: 已清理 | commit: 交付=4adf2d03(auto-ui master)+f07b5ec8(autographics-dev)；deposit=77f9301c7+收尾提交(本仓master) | next: done`。交付链闭合：merge pass、Specs/ledger v123/归档/清理全部实证；无 pending 门（批回归不到期；全量 AG 套件=里程碑档 pending 属既定裁定非本计划门）。
