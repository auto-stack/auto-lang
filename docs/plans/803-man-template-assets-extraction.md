---
plan_id: PLAN-803
status: drafting
feature_name: Man 模板资源与脚手架规划独立
author: [Codex]
created_at: 2026-10-17
updated_at: 2026-10-17
plan_revision: 1
current_step: 0
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
| [ ] | T-01 | work | 786/791/792已合、合同确认 | asset.rs/automan.rs/exporter/ghs.rs/iar.rs及三组资源全量盘点，旧API/顺序/成功失败/副作用冻结 | 01/02/03 |
| [ ] | T-02 | work | T01 | 新crates/auto-man-assets、资源实际移动、纯索引/路径计划，check/V01/02 | 01/02 |
| [ ] | T-03 | work | T02 | asset.rs兼容门面/copy真委托、GHS/IAR接线，保持解释/IO及失败阶段，V03/04 | 02/03 |
| [ ] | T-04 | work | T03 | 独立consumer、全旧臂投影、真脚手架/exporter测试及无效/Unicode/失败反控 | 01/02/03 |
| [ ] | T-05 | work | T04 | 原路径移除/打包消费、777精确边、792触面/800同步核查，V05 | 04 |
| [ ] | T-06 | work | T05 | 迁位/净删/资源hash与剩余语言/IO债、SD收据，提交送审 | 04 |
| [ ] | T-07 | review | T06 | 同树独立AC/SD/API/全资源及失败阶段审查，merge门pending | 05 |
| [ ] | T-08 | merge | final pass | 同步master、受影响复审/冻结门、Spec/ledger/归档/wt-guard清理 | 06 |

## 9. 复审记录

2026-10-17 / stage: new / PLAN-803 revision: 1 / outcome: pass / next: work（合同确认后）。仅合同预检通过，T/AC未实施；不等799/800整体完成，见[预检](attachments/extraction-next-20261017/preflight.md)及批次编排。

## 10. 待澄清事项

无外部决策阻塞。T01须记录旧模板注入失败/copy破坏性失败事实，行为改善不夹在结构迁移。auto-man-assets只是仓内crate名，不新建Git仓/改变既有auto-man仓归属。
