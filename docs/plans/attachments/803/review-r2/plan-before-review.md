---
plan_id: PLAN-803
status: execution_done
feature_name: Man 模板资源与脚手架规划独立
author: [Codex]
created_at: 2026-10-17
updated_at: 2026-10-17
plan_revision: 1
current_step: 6
total_steps: 8
supersedes_spec_components: [docs/specs/auto-man/project.md]
new_spec_components: [docs/specs/auto-man-assets/project.md]
touched_goals: [GOAL-003, GOAL-016]
affects: [auto-man-assets, auto-man]
depends_on: [PLAN-786, PLAN-791, PLAN-792]
---

# [PLAN-803] Man 模板资源与脚手架规划独立

## 0. 变更摘要

新增无语言引擎auto-man-assets，承接assets/templates及assets/builders/ghs、iar的唯一资源、索引和脚手架路径规划；旧asset.rs门面及生产Templates::copy/exporter真实委托。Auto求值和文件物化仍在auto-man上层，不借抽资源改变模板/失败行为。

## 1. 目标

纯资源消费者无需auto-lang/auto-man/VM/renderer即可枚举读取全部选定资源、按模板族和包名生成源资源→相对输出路径计划。原脚手架和GHS/IAR读取同一字节，原公开API兼容，原位置无第二份副本。

非目标：抽全部Man构建器/CLI、修复set_global作用域、改字符串替换渲染、修改覆盖/失败规则、统一guard算法或搬所有UI资产。gallery/shadcn/wm不移动；完整Man仓仍需管理核心/语言adapter/CLI后续切口。

## 2. 架构方案

auto-man::asset（兼容门面/Auto渲染/IO adapter）→ auto-man-assets（rust-embed资源+纯选择/路径计划）。底层只返回原始bytes和无语言含义的键/相对路径，不依赖AutoInterpreter/Value/AST/auto-man，原则上无auto-val需求；旧AutoStr/AutoError返回类型由上层转换保持。语言执行不反向注入底层，不新建模板引擎。

auto-gen-core已有render/guard/输出编排，但后render再写/批次错误收集与当前copy不同，**本轮不将copy改接generate**。资源/计划迁位与输出策略改造分开，不能暗中改变失败行为，也不复制guard或输出事务算法。

## 3. 技术栈

Rust轻workspace crate、rust-embed、标准库字节/路径；旧公开RustEmbed门面转发、独立consumer、资源hash/输出投影对照。无需IAR/GHS工具链，导出验收不冒称商业编译器编译通过。

## 4. 需求分析与背景调查

用户2026-10-17授权继续规划迁移，本轮仅合同/路线，不执行Rust、不创建agent/树、不切换旧仓；初次取材`143aff0a258fdca8bda9a3b8068dc8a9cced623c`，规划期间复核至`2d246b4e86c1ebf5138f6a316c1896e56a5d35f8`，800已归档8/8、799 executing，见[指纹](attachments/extraction-next-20261017/inputs.json)。786/791/792已合；本计划不消费auto-package-resolver API，800新增package-resolver Spec已核对，asset/exporter源码未改。本计划可与799/802并行，仅共享Man manifest/lib和边界规则窄接线，见[编排](reports/extraction-next-parallel-order-20261017.md)。

asset.rs当前Templates嵌assets/templates，copy按首目录选择，以目标path.file_name为包名，对相对路径字面replace("name", pac_name)，创建目录/文件后逐项set_global(name)→eval_template→repr→write。app/lib/capp/clib/libdemo含嵌套目录，不能只测app/pac.at。GHSTemplate三项/IARTemplate四项由exporter消费，derive的get/iter/RustEmbed也是旧公开面。

解释器Spec指出set_global只写侧表，当前模板VM不一定可见，不能假定`${name}`成功；copy还有unwrap及先create再render副作用。须冻结真实旧臂，不用新渲染器/golden伪造兼容；错误改善另立行为计划。SD如实记录仍在上层的限制，不把这些遗留行为宣传为新安全保证。

## 5. 详细设计

T01穷举三组根全部相对键/bytes hash/数量与get/iter/trait/copy/exporter消费者。纯计划的顺序、未知族空结果、name多次出现、嵌套目录和包名取值按旧臂冻结；不以通用路径规范改写接受域。底层API接收包名，不读pac/执行Auto。

资源实际移动至新crate，保留templates、builders/ghs、builders/iar子布局，Embed根相对新crate；所有脚本/打包/build输入一并盘点接线。无绝对主检出/临时树搜索。字节/键/顺序保持；路径name替换和内容求值分属两层，不用字符串替换`${name}`冒充原Auto求值。

新纯计划只返回资源引用或owned条目及相对路径，**不创建目录、不渲染、不写盘**。保留旧Templates/GHSTemplate/IARTemplate薄类型及inherent方法，必要时手动实现RustEmbed、转发get/iter；不能在外来类型补inherent impl，也不能再derive第二份嵌入。旧AutoStr/AutoError转换留门面。

Templates::copy真实使用新计划枚举，仍按原每文件求值/输出步骤执行，包括repr和失败阶段；GHS/IAR从新资源取唯一字节。旧臂成功的对输出文件集/bytes，旧臂失败的对错误/阶段/前后文件状态，不要求伪造成功或静默空文件。失败测试只在专属临时目录，禁止真实工程覆盖反控。

本轮不新增输出安全承诺：先创建/截断再render等遗留限制进入SD/剩余债收据，后续可单列render-before-write及错误传播计划。792自身已承诺的发行失败保护不受此豁免，不能套用copy旧规则。不得在本轮私自修复模板注入可见性或改通用解释器。

如影响792选定闭包，沿现有清单/身份/admission/staging接口补包/资源并定向验证；若不在Iced正常闭包，给metadata/调用面证据，不强加无关包，但仍验证Man资源打包可消费。800合流后核Cargo/lib兼容，不改resolver/Target/pac.lock。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-man-assets/project.md | 无独立包→三组资源/纯计划单一来源，无语言闭包 | 可独立交付资源层 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-man/project.md | 资源与解释器混合→薄门面真委托，语言/IO及遗留失败/注入限制在adapter | 不误称Man全部无语言依赖 | AC-02/03/04 |

## 6. 测试设计

拟唯一树`D:/autostack/.wt/lang-803/auto-lang`、branch `codex/plan-803-dev`；work→同树独立final review→merge，无额外agent/模型。以下新fixture由T02/T04建成，本轮未执行。

| Case / owner | AC / 不变量 | 输入、实际入口 | oracle/反控及覆盖边界 |
|---|---|---|---|
| V01 / work | 01 轻资源闭包 | 新test/fixtures/modularity/man-assets-consumer/Cargo.toml，fresh test/metadata | 三组全部资源/纯计划可用，normal/build/dev无auto-lang/auto-man/VM/UI；离开原assets仍能读内嵌字节 |
| V02 / work | 02 字节/路径 | 原三组全量键/hash，五模板族、嵌套/重复name片段、未知族/Unicode包名 | core/门面/旧提交清单逐项相同，计划一致；`${name}`内容未渲染；纯API对空目录零副作用 |
| V03 / work | 03 真脚手架 | Templates::copy/Automan创建路径，T01冻结成功模板、注入失败模板、无匹配族 | 成功文件集/bytes同旧臂；失败错误/阶段及截断等状态同旧臂；每个内嵌模板有旧/新成功或失败分类，不用新repr造expected |
| V04 / work | 02/03 builder门面 | GHS三项/IAR四项，真实exporter/ghs.rs、iar.rs最小工程 | get/iter/RustEmbed旧公开调用可编译；原模板/导出字节及关键结构保持；不声称执行商业编译器 |
| V05 / work | 04 单一性/交付 | 原路径移除、crate打包清单/临时解包轻消费、777审计与792闭包 | 无副本/原仓绝对路径，新包include完整，缺资源控制真失败；影响792则fresh/locked重定位及坏hash/缺输入/已有目录保护保持 |

work命令：`cargo check -p auto-man-assets`、`cargo check -p auto-man`，新consumer `cargo test --manifest-path test/fixtures/modularity/man-assets-consumer/Cargo.toml`，`cargo test -p auto-man asset::`及新增脚手架/exporter定向测试（T04记录实际filter/选择数）。旧panic反控须隔离捕获/子进程，不能吞异常当成功。T05记录crate包清单/临时解包消费命令，不授权registry发布。

final review通过后merge冻结跑裸`cargo t`及新crate/Man定向门。未改VM/生成器不跑tv/tu，实际触及则补相应门；tf仍主检出到期单实例，work/review不整档。本轮Category A不跑cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 三组资源/纯规划无语言消费者可用，V01通过 |
| AC-02 | work | 全量键/字节/路径及旧get/iter/API兼容，纯计划零IO，V02/04通过 |
| AC-03 | work | 真copy/GHS/IAR委托，成功/失败阶段同旧臂，语言/IO留上层，V03/04通过 |
| AC-04 | work | 唯一归属、打包消费、777/792/800触面及遗留限制/SD完整，V05通过 |
| AC-05 | review | 独立AC/SD/API/全资源与失败反控审查通过，绑定提交，merge门pending明确 |
| AC-06 | merge | 冻结日常/必要定向门、Spec/ledger/归档及guard清理完成 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 已核对路径/交付与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 786/791/792已合、合同确认 | 盘点完成：13+3+4 键序冻结（walkdir sort_by_file_name）、20 文件 SHA-256 基线表、旧臂 copy 五族全部 Ok（`${name}` 对模板 VM **可见**，插值含旧 VM 怪癖）、未知族 Ok+零副作用、`..`路径 panic 消息冻结、GHS/IAR 七访问器字节；探针（tests/plan803_t01_probe.rs，基线运行后已退役）收据 [t01-oldarm-probe.txt](attachments/803/t01-oldarm-probe.txt) | 01/02/03 |
| [x] | T-02 | work | T01 | crates/auto-man-assets 建成（rust-embed+std）；三组资源 git mv 逐字迁入（20 rename 100% 相似）；plan_template_family 纯计划（余段字面 name 替换/未知族空/**零输出 IO**——R1 后措辞含调试态读盘边界）；单测 6/6（键序/SHA-256/投影/`plan_has_no_output_io`/split 镜像；R1 后 rustfmt 全净）；cargo check -p auto-man-assets 绿 | 01/02 |
| [x] | T-03 | work | T02 | asset.rs 门面真委托：copy 保持旧臂逐文件求值/物化/失败阶段（unwrap panic、先建后求值遗留顺序）；GHS/IAR inherent 访问器 + inherent/RustEmbed 双形态 get/iter/compressed 转发（补齐 compression 必需项）；**F803-R1-01 修复：三个 trait impl iter() 恢复旧 derive 具体返回 `Filenames`，经底层 trait 入口转发**（RPITIT 具体隐藏类型跨 crate 可见性经实验定案）；exporter/automan 消费零改动；project_model_adapter 两处路径随迁位更新（clib 豁免×2 + app/pac.at 直读）；探针重放 68 观测与旧臂**逐字节一致**（[t01-newarm-replay-identical.txt](attachments/803/t01-newarm-replay-identical.txt)）；check -p auto-man / -p auto 绿 | 02/03 |
| [x] | T-04 | work | T03 | V03/V04 tests/plan803_asset_facade_tests.rs **9/9（R1 后）**：五族+Unicode 冻结投影（期望程序化转写自基线探针）、未知族零副作用、`..` panic 同臂、门面双形态字节转发、7 访问器字节对照、IAR exporter 最小工程端到端（demo.eww/ewp/ewt/ewd 真实生成）、GHS 预存 app mold 失败边界冻结（default.gpj 生成钉住）、**F803-R1-01 编译正控 `trait_iter_returns_concrete_filenames_type`（复审反例同形）**；V01 consumer 4/4（`external_consumer_planning_has_no_output_io` 更名）+ verify-deps PASS；asset:: 1/1、project_model 11/11 + 语料对照 v02 绿；**R1 格式修复：三文件 rustfmt --check 全净 + git diff --check 干净** | 01/02/03 |
| [x] | T-05 | work | T04 | 原路径零残留（residue 扫描 + git status 全 rename）；打包：cargo package --list 26 文件含全部资源、解包 SHA-256 对照 20/20（缺资源即真失败；registry 未发布）；777：域映射 man-ecosystem + 资产根 man-template-assets + 基线重采样绑定 904b5e5e0，strict verdict=pass violations=0；792 触面：交付资源闭包=src/front allowlist，与三组零交集（不触发）；800 同步：workspace members 登记，resolver/Target/pac.lock 未动 | 04 |
| [x] | T-06 | work | T05 | 迁移收据+三项预存遗留债登记（KNOWN-DEBT：copy 先建后求值窗口/旧 VM 插值怪癖/GHS program.gpj 解析级失败——均非本轮修复）；SD-01 auto-man-assets/project.md 新建（**R1 后含构建模式边界节**）、SD-02 auto-man/project.md 抽取分工节+asset 行门面化；**F803-R1-03 可重复反控 `mode-boundary-probe.py`**（解包副本移开资源 debug=101/release=0、缺资源=101，不触真实 crate 资源）；提交链 904b5e5e0 → bbad82a9f → **R1 修复 a9b401421**（worktree plan-803-dev），送 T-07 R2 复审；merge 门 pending 归 merge（分列 filter 按 pending_closeout_gates） | 04 |
| [ ] | T-07 | review | T06 | 同树独立AC/SD/API/全资源及失败阶段审查，merge门pending | 05 |
| [ ] | T-08 | merge | final pass | 同步master、受影响复审/冻结门、Spec/ledger/归档/wt-guard清理 | 06 |

## 9. 复审记录

2026-10-17 / stage: new / PLAN-803 revision: 1 / outcome: pass / next: work（合同确认后）。仅合同预检通过，T/AC未实施；不等799/800整体完成，见[预检](attachments/extraction-next-20261017/preflight.md)及批次编排。

2026-10-17 / stage: work 入口 / PLAN-803 revision: 1 / 用户授权实施（"计划803: auto-plan-work 实施"）。worktree `D:/autostack/.wt/lang-803/auto-lang`、branch `plan-803-dev`（repo 规范命名，计划§6拟名 `codex/plan-803-dev` 为 Codex 作者拟用名，登记调整，无所有权冲突）、base `1dd123361a`（=master，800已合入）。主检出预检：crates/test 零 WIP，仅 docs/plans 簿记。

2026-10-17 / stage: work / PLAN-803 / revision: 1 / outcome: pass（work-owned 全完成，execution_done）/ worktree: `D:/autostack/.wt/lang-803/auto-lang`（组内含 auto-down 兄弟 detached c39ac7f，PLAN-726 解析序）/ branch: plan-803-dev / base: 1dd123361a / code: 904b5e5e0（实现，35 files，20 rename 100%）+ bbad82a9f（777 基线重采样）/ dependency_revisions: rust-embed 8.13.0（lock 既有 pin，未升版）/ task_ids: T-01..T-06 全勾

- **AC-01**（V01）：man-assets-consumer 4/4 + verify-deps.sh PASS（闭包= consumer→auto-man-assets，无语言栈）。
- **AC-02**（V02）：auto-man-assets 单测 6/6（13/3/4 键序 + 20 文件 SHA-256 冻结 + 计划投影/零 IO）；门面双形态转发逐键字节对照。
- **AC-03**（V03/04）：五族+Unicode 冻结投影 8/8；未知族零副作用；`..` panic 同臂；探针 68 观测新旧臂逐字节一致；IAR exporter 端到端 demo.eww/ewp/ewt/ewd；GHS 预存失败边界冻结。
- **AC-04**（V05）：原路径零残留；打包解包 hash 20/20；777 strict pass 0 violations（基线绑定 904b5e5e0）；792 零交集；800 无触面；剩余债 3 项入 KNOWN-DEBT；SD-01/02 已在 worktree 就绪。
- **开放项/反控边界**：GHS app mold 解析级失败为预存（flipped 代码与数据无关 + 模板字节 hash 同臂证明），已冻结为断言而非静默跳过；13 模板无任何 eval 失败案例 ⇒ copy 破坏性失败窗口仅结构保留（panic 链逐字），无内嵌反例可触发。
- **T01 事实修正**：`set_global` 侧表对 eval_template **可见**（计划§4 的"不一定可见"担忧排除，实测五族全渲染成功但插值形态含旧 VM 怪癖，已冻结不改）。
- **merge 门 pending**（归 T-08，work/review 不触发）：冻结提交裸 `cargo t` + `cargo test -p auto-man-assets` + consumer manifest 门 + `cargo test -p auto-man asset:: project_model plan803`；未改 VM/生成器不跑 tv/tu；tf 主检出到期判定。
- next_responsible_role: review（T-07，同树独立审查）。


2026-10-17 / stage: review / review_scope: final / reviewer_identity: 当前 Codex 独立审查上下文（未承担本计划实现；未另开 agent/任务） / PLAN-803 revision: 1 / outcome: **needs_fix** / worktree: `D:/autostack/.wt/lang-803/auto-lang` / branch: `plan-803-dev` / reviewed_commit: `bbad82a9f9a5b667d33fec28dcf680f6d95a1e7e` / base_commit: `1dd123361a75aa4b683376dcb2049299122724c9`。

- **报告与冻结输入**：[review-r1/review.md](attachments/803/review-r1/review.md)、[baseline.json](attachments/803/review-r1/baseline.json)、[SD 冻结](attachments/803/review-r1/spec-delta-reviewed.patch)（SHA-256 `5a379a28589af4dcc00baa991191a00ba5b9217f19434a9582d0373d3ee14f1b`）。依赖 rust-embed 8.13.0；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；Cargo.lock hash 与审查前后实现树洁净状态均已冻结。
- **F803-R1-01 / P2 / AC-02 / T-03、T-04**：三个 RustEmbed::iter 手写门面丢失旧 derive 的具体 `rust_embed::Filenames` 返回类型；显式类型赋值旧正控编译通过、新门面 E0308。恢复旧公开返回类型并补编译正控。
- **F803-R1-02 / P2 / AC-05 / T-02、T-04**：三个新增测试文件 rustfmt --check 红，门面测试文件 EOF 多余空行导致 diff --check 红；仅修本次触面格式。
- **F803-R1-03 / P2 / AC-04 / T-06**：无条件“无任何 IO/内嵌资源”说明缺构建模式边界。默认 debug 会读源码资源目录，移开解包资源后消费者 1/4；release 无源码仍 4/4。需明确无输出 IO 与调试态读盘/发行态内嵌的区别，保留相应验收；不要求顺手改变旧调试态行为或新增全模式保证。
- **acceptance_results**：AC-01 pass；AC-02 fail（F01）；AC-03 pass；AC-04 partial（F03）；AC-05 fail；AC-06 merge pending。资源 6/6、独立 consumer 4/4、门面 8/8；额外三种 IO 失败新旧错误和完整文件/目录状态一致；实际解包 consumer debug/release 各 4/4；20/20 资源与 Git 基线/归档全等，真实缺资源反控非零；777 strict pass，0 violations。普通 Cargo 门面命令受 sccache 端口及镜像缺锁版本阻塞，已用 Cargo 原样 rustc 参数独立重编当前库和测试验证 8/8，未冒称标准 Cargo 入口已通过，见报告环境限制。
- **SD/元数据**：SD-01/02 分工/路径/目标成立，公开面保证受 F01、构建模式说明受 F03 约束；supersedes/new/touched_goals 保留。未发布 canonical Spec/ledger。
- **状态回写**：T-02/03/04/06 重开，T-01/05 保留完成；executing，current_step=2/8。此前 work 收据与原始勾选快照保留，开放项登记 KNOWN-DEBT 不代表批准延期。
- **pending_closeout_gates**：最终 pass 后由 merge 在新冻结提交上跑裸 cargo t、新资源 crate、consumer，以及分别执行 `cargo test -p auto-man --lib asset::`、`cargo test -p auto-man --lib project_model_adapter::`、`cargo test -p auto-man --test plan803_asset_facade_tests`；不得照抄旧收据的多个位置 filter 合并命令。tv/tu 无触发，tf 仍批量到期判定；Spec/ledger/归档/guard 清理全部 pending。
- **next**：work 在原树关闭 F01..03、定向验证并提交，再对新 SHA 复审；本轮未修实现、未合入。


2026-10-17 / stage: work（R1 修复轮）/ PLAN-803 revision: 1 / outcome: pass（F803-R1-01..03 全闭，重交 T-07 R2 复审）/ worktree: `D:/autostack/.wt/lang-803/auto-lang` / branch: plan-803-dev / base: 1dd123361a / reviewed_commit(R1): bbad82a9f / **code(R1 修复): a9b401421** / task_ids: T-02/03/04/06 重勾 / next_responsible_role: review（R2，对 a9b401421 复审受影响内容）

- **F803-R1-01 → 不变量**：门面 trait impl 签名必须与旧 derive 逐形态一致（含具体返回类型），否则跨 crate `let _: Filenames = <T as RustEmbed>::iter()` E0308。**受影响路径**：Templates/GHSTemplate/IARTemplate 三 trait impl（asset.rs:75/123/177 原实现）。**修复**：`fn iter() -> Filenames { <Inner as RustEmbed>::iter() }`（转发底层 trait 入口，非 inherent opaque）；语义前提经独立实验定案——RPITIT 隐藏类型为具体类型时跨 crate 可见（临时 rpitit 探测证实后删除）。**永久反例**：`trait_iter_returns_concrete_filenames_type`（门面 9/9 含此项）。**剩余缺口**：无（inherent iter 保持旧臂同样 opaque，与 derive inherent 一致）。
- **F803-R1-02 → 不变量**：新增文件必须过 rustfmt --check 与 git diff --check。**受影响路径**：auto-man-assets/src/tests.rs、auto-man/tests/plan803_asset_facade_tests.rs（含 EOF 多余空行）、consumer tests/assets_consumer.rs。**修复**：rustfmt 三文件 + 清 EOF 空行；未触碰无关旧文件。**结果**：rustfmt --check 全净、git diff --check 干净（提交 a9b401421 上复核）。
- **F803-R1-03 → 不变量**：文档不得无条件声称零 IO/内嵌——须区分「零输出 IO（不建目录/不渲染/不写文件）」与 rust-embed 调试态动态读源码资源目录（无 debug-embed；目录缺失枚举静默为空，同旧臂机理）/发行态内嵌。**受影响路径**：lib.rs 模块+函数文档、tests.rs 测试名与注释、SD-01 project.md「零 IO」节、consumer fixture 文档。**修复**：全部改写为零输出 IO + 构建模式边界；新增可重复反控 `test/fixtures/modularity/man-assets-consumer/mode-boundary-probe.py`：解包副本移开资源目录 → debug exit=101 / release exit=0，缺资源 → exit=101（真失败）；只动 target/ 下探针副本，不触真实 crate 资源。**行为零改动**（未改调试态行为/未新增全模式内嵌承诺）。
- **R1 定向验证**（a9b401421）：auto-man-assets 6/6、门面 9/9、consumer 4/4 + verify-deps PASS、mode-boundary-probe PASS、rustfmt --check 三文件净、git diff --check 净、cargo check -p auto-man 0 error。整档门禁仍归 merge（分列 filter 执行，见 R1 pending_closeout_gates）。
- **环境备注**：修复轮 sccache 以 `SCCACHE_SERVER_PORT=18426` 规避本机端口排除段问题（与 R1 报告环境限制同源）。

## 10. 待澄清事项

无外部决策阻塞。T01须记录旧模板注入失败/copy破坏性失败事实，行为改善不夹在结构迁移。auto-man-assets只是仓内crate名，不新建Git仓/改变既有auto-man仓归属。
