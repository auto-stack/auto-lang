---
plan_id: PLAN-805
status: drafting
feature_name: Man 工具链配置与构建规则核心抽取
author: [Codex]
created_at: 2026-10-17
updated_at: 2026-10-17
plan_revision: 1
current_step: 0
total_steps: 8
supersedes_spec_components: [docs/specs/auto-man/project.md]
new_spec_components: [docs/specs/auto-toolchain-core/project.md]
touched_goals: [GOAL-003, GOAL-016]
affects: [auto-toolchain-core, auto-man/builder]
depends_on: [PLAN-791, PLAN-795, PLAN-800]
---

# [PLAN-805] Man 工具链配置与构建规则核心抽取

## 0. 变更摘要

把builder/ninja/config.rs、mapper.rs、templates.rs的工具链模型/默认规则/flag映射/命令模板抽为auto-toolchain-core单一实现；旧路径薄re-export，生产NinjaBuilder/CompilerStore/CompilerResolver真实消费新类型与规则。动态am.at、PATH/磁盘/进程观测留Man上层。

## 1. 目标

无auto-lang/auto-man/VM/UI消费者可构造完整现有工具链配置、映射flags并渲染编译/汇编/链接/归档命令；旧公共类型身份、公开路径、默认值和字符串输出保持，不是新增闲置DTO。

非目标：迁Target/Pac、构建DAG/进程runner、Cargo/Vue生成器、模板资产/解释器、改变shell quoting或编译器定位规则、修复Resolver递归旧问题。此轮不宣称Man全部无语言依赖或可独立仓交付。

## 2. 架构方案

auto-man builder/配置加载adapter → auto-toolchain-core → std/必要auto-val轻类型。核心拥有CompilerKind、ExecutableType、FlagFormat/FlagMappings、CompilerLocation、CompilerConfig、FlagMapper、CommandTemplates及现有内聚纯方法；AutoStr/AutoPath保持同型，不制造转换副本。无AutoConfig/Parser/AST/解释器/AutoGen/Pac/Target/进程调用依赖；dev-deps也不拉auto-man做集成测试。

CompilerStore::load_from_am_config/Node解码和加载失败策略、CompilerResolver环境/磁盘/版本检测、NinjaBuilder执行与AutoGen留上层。本计划只抽精确纯规则闭包，不为搬更多代码给核心塞宽泛host trait或新的配置模型。

## 3. 技术栈

Rust workspace轻crate、原auto-val类型、独立consumer与生产Ninja内存输出/专属临时目录对拍。无需安装八类商业/系统工具链，不调用编译器或网络。

## 4. 需求分析与背景调查

用户2026-10-17授权继续规划，并询问799/801前置。本轮只建立合同/设计路线，不执行Rust/建树/agent/旧仓切换，不改他线合同。取材9932962bc157be13bcaa0520011c7c78082cb2fe及[指纹](attachments/extraction-followup-20261017/inputs.json)。791/795/800已归档；无需799/801，也不消费未合入的803资产包。

config.rs/mapper.rs/templates.rs目前只需auto-val/std；配置含MSVC/GCC/Clang/GHS/IAR/Targeting/Hightec/TICLang八类，四可执行角色和三种FlagFormat。CompilerStore则引用AutoConfig读am.at；Resolver引用PATH、磁盘与版本进程，不能把整个ninja目录搬入核心。NinjaBuilder持CompilerConfig并经FlagMapper、CommandTemplates生成规则，是本计划必须证明的生产消费路径。

模板是内联命令字符串，不是803搬的assets/templates或builders/ghs/iar文件；两者不能因为“模板”同名误设依赖或重写资源。与803实现主写面分离，仅Man manifest/lib/Spec/777窄窗口串行。802/799/801不构成AST或运行时API前置。

## 5. 详细设计

T01冻结全部公开类型/字段/构造方法、八类默认配置、枚举alias/大小写行为、四角色命令与扩展名、FlagFormat和mapper边界。区分实际使用的有序flags/参数和原来无序map：不得对命令排序/去重/加引号来“改善”兼容。确认无遗漏inherent impl或私有访问跨crate导致孤儿规则问题。

搬三模块及纯测试至新crate，原builder::ninja::{config,mapper,templates}保薄重导出和ninja::*旧出口；原类型为同一身份，不留重复enum/struct/mapper/render实现。CompilerStore/Resolver/Builder继续按旧公开路径引用即应消费新实现，必要适配不得改算法。

保留map_output现有trim_end_matches及Prefix/Postfix/Both差异、includes/defines/libraries参数次序/空格、CompilerKind别名、MSVC/GCC等默认flags/扩展。CommandTemplates::render仍按原key长度降序替换，未提供占位符不擅删，不增新的escaping/递归插值规范；等长key及含占位符value的原未定序角落T01据旧实现登记，不用排序后的新golden替代已有行为。

真实生产对拍至少覆盖八类CommandTemplates和代表性NinjaBuilder生成（编译/汇编/链接/归档、有include/define/lib/自定义flag、空项/Unicode/含空格路径）。仅观测生成，不执行shell或实际工具链；CompilerStore正常加载和坏配置处置、Resolver已有定位/回退仍是上层合同。

新crate如进入792受管正常闭包，沿已交付清单/身份/staging/admission补齐并重验受影响控制；若不在该闭包须metadata证明，不能强加无关包或把漏包留给792。与803后合入者同步Cargo/lib/Spec素材并集，禁止覆盖资源迁移。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-toolchain-core/project.md | 无轻工具链包→八类配置/flag/命令规则单一定义，零语言/IO闭包 | 可独立消费的构建规则 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-man/project.md | ninja纯规则内嵌→薄门面，动态加载/定位/执行留上层 | 明确实际生产切换及未迁边 | AC-02/03/04 |

## 6. 测试设计

work→同树独立final review→merge，无额外agent/模型指定。拟唯一树D:/autostack/.wt/lang-805/auto-lang，branch codex/plan-805-dev。新fixture由T04建立，本轮不执行Rust。旧臂来自冻结提交/既有oracle，不能从新实现重造expected。

| Case / owner | AC / 不变量 | 输入/生产入口 | oracle与边界 |
|---|---|---|---|
| V01 / work | 01 独立闭包 | 新test/fixtures/modularity/toolchain-core-consumer/Cargo.toml fresh test/metadata | 全模型/mapper/templates可用，normal/build/dev闭包无auto-lang/auto-man/VM/UI/AutoConfig，无进程/文件观测 |
| V02 / work | 02 模型/规则 | 八类默认、全部alias/未知kind、四角色、三FlagFormat、空/多值/含空格与Unicode路径、重复扩展尾缀 | 旧/新字段与具体字符串逐项相同，顺序/空格/trim边界可区分；旧/新路径类型直接互传编译 |
| V03 / work | 02 命令模板 | 四命令、缺变量、相似长短key、原替换次序案例及所有compiler kind | 既有命令字节一致，不偷偷quote/重排/移除未知占位符；未定序情况只按旧承诺比较，不归一掩盖有序差异 |
| V04 / work | 03 真生产/adapter | NinjaBuilder memory输出或独占临时工程；CompilerStore真实am.at正常/坏配置与缺文件；既有Resolver定向控制 | 实际build.ninja规则/命令与旧臂一致，新类型真被生产使用；AutoConfig/定位/错误处置留上层且不漂移；不要求商业编译器执行 |
| V05 / work | 04 单一性/交付 | 删除旧实现、777精确domain、803合流、792生产metadata/包清单 | 无镜像配置/规则/隐藏语言dev依赖；必要792fresh/locked/重定位/缺输入反控触面闭合；无闭包变化给依据 |

work：`cargo check -p auto-toolchain-core`、`cargo check -p auto-man`，新consumer manifest test、新核心定向测试及`cargo test -p auto-man builder::ninja::`（T04核对实际选择数/测试隔离）。全局env控制复用既有串行约束，不能并发改真实PATH/用户am.at。final review后merge冻结裸cargo t+核心/Man定向门；未碰VM/ui_gen不跑tv/tu，tf主检出到期单实例。当前Category A不跑Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 选定完整模型/规则无语言/执行依赖独立消费，V01通过 |
| AC-02 | work | 同类型身份/旧路径/八类默认/flags/命令输出保持，V02/03通过 |
| AC-03 | work | Ninja生产真实委托，上层动态配置/定位/执行合同未漂移，V04通过 |
| AC-04 | work | 旧实现删除、迁位净删/剩余边/803与792触面、SD素材完整，V05通过 |
| AC-05 | review | 同树独立AC/SD/遗漏及反控审查通过，绑定提交，merge门pending |
| AC-06 | merge | 冻结日常/必要定向门、Spec/ledger/归档与guard清理完成 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 路径/交付与验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 791/795/800归档、合同确认 | ninja/config.rs/mapper.rs/templates.rs及store/resolver/builder消费者，闭包/公开面和八类旧臂冻结 | 01/02/03 |
| [ ] | T-02 | work | T01 | 新crates/auto-toolchain-core，单份模型/mapper/templates/纯测试，check与V01/02 | 01/02 |
| [ ] | T-03 | work | T02 | 旧三模块/ninja出口薄re-export，真实CompilerStore/Resolver/NinjaBuilder接线，删除旧实现，定向V03/04 | 02/03 |
| [ ] | T-04 | work | T03 | 新独立consumer、全八类/四命令/无效输入、真实生产与配置控制V01–04，记录实际命令/输出 | 01/02/03 |
| [ ] | T-05 | work | T04 | V05 777精确边/单一性/803窄合流/792闭包，必要交付控制 | 04 |
| [ ] | T-06 | work | T05 | 迁位/净删/剩余管理层与SD/SHA收据，提交送审 | 04 |
| [ ] | T-07 | review | T06 | 同树独立AC/SD及规则/生产反控审查，merge门pending | 05 |
| [ ] | T-08 | merge | final pass | 同步master、受影响复审/冻结门、沉淀/归档/wt-guard清理 | 06 |

## 9. 复审记录

2026-10-17 / stage: new / PLAN-805 revision: 1 / outcome: pass / next: 合同确认后work。本轮无799/801/802/803硬依赖；三个纯模块和803资源根互不重叠，共享manifest/出口短窗口串行。[预检](attachments/extraction-followup-20261017/preflight.md)。

## 10. 待澄清事项

无外部决策阻塞。T01负责全部公开API/实际旧命令边界，不借“纯规则”删掉兼容方法；Target/构建DAG/命令协议/进程退出另立后续计划。auto-toolchain-core是仓内crate名，不新建Git仓。
