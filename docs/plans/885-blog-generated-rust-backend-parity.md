---
plan_id: PLAN-885
status: drafting
feature_name: blog-generated-rust-backend-parity
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-29
plan_revision: 1
current_step: 0
total_steps: 7
depends_on: []
supersedes_spec_components: ["docs/specs/auto-api-gen/project.md"]
new_spec_components: []
touched_goals: []
---

# PLAN-885：Blog生成Rust后端的最小兼容性闭环

## 0. 变更摘要

由[BLOG-004](D:/autostack/auto-blog/docs/plans/004-dual-runtime-acceptance.md)提出的语言侧前置，只处理Blog默认Rust服务生成链实际暴露的阻断；不承接应用写作增强或原生UI驱动。语言仓242为长期gap台账、701/742为既有能力来源，不能把它们的历史完成当作Blog当前闭环。

## 1. 目标

从现有纯.at经公开API生成入口得到可编译/运行的Rust后端，修复实际确认的runtime限定/类型/构造/借用路径问题，保持合法输入和业务字符串语义。没有重现的历史错不要求制造补丁，必须记录有效正控及版本差异；重现的问题必须真实关闭。

## 2. 架构方案

主改面归auto-lang：`crates/auto-api-gen/src/{api_gen,rust_workspace}.rs`、`crates/auto-lang-core/src/trans/rust.rs`、内嵌facade与独立a2r-std仅在证据证明需要时修改。不要把缺少time简单理解为独立crate没实现：当前独立time已存在，生成器会改写运行时路径。

计划/进度在auto-lang/master主检出；唯一实现/顺序复审树拟为 `D:/autostack/.wt/lang-885/auto-lang`，分支 `plan-885-dev`，按本仓规则创建并固定所需同组依赖；本轮不创建树、不建链接。Blog独立clone只是消费已交付版本的外部验证对象，不在语言计划里写Blog业务源或用手修生成Rust充当修复。

## 3. 技术栈

现有a2r、auto-api-gen、a2r-std/auto-lang-core，真实Man CLI消费链。保留SDK/UI/Man所有权边界；若诊断落到Man工具编排/安装交付，则提供最小输入和owner，不在语言仓创建第二份编排器。

## 4. 需求分析与背景调查

用户2026-10-29明确授权“建立Blog后续计划，专门补齐Rust后端和原生VM界面的运行与验收，关联必要框架前置计划，并更新总跟踪表”。本轮只编制合同、必要反链与组合pin，不实施代码、不运行全局Cargo门、不宣布运行缺口已关闭。后续执行须按四技能进入work；未指定模型、独立会话、自动续跑或并行agent。

语言基线 `d9718cc63987e93e571eb045732fd2dab041ed0b`；主检出含大量既有文档WIP与已预留884，保留它们，按当前号源885独占取号，不收编无关内容。本轮仅读相关规范与源码，不跑cargo t/docs_gen。

- `crates/auto-api-gen/src/api_gen.rs` — SHA256 `9f860790146f1ee360ce926d49c592082dd14ed74f8cbb362344309aa1e7123e`。
- `crates/auto-api-gen/src/rust_workspace.rs` — SHA256 `244969be38a7c3cb1d4f06af60d1586b5788568acf27756019fb933aac503133`。
- `crates/auto-lang-core/src/a2r_std.rs` — SHA256 `f0c2074399237fe536b56b28cb00e4e2aa928a66936d4fcda5b57181eed6aef5`。
- `crates/a2r-std/src/time.rs` — SHA256 `99b7598b10efdcf9ca6f63c71f7dc8e2dd154bd744d872c6bce8753975e1963b`。
- `docs/specs/auto-api-gen/project.md` — SHA256 `6bfed66e9080732a0087ae886ac4c6d71909cc82ffa318ecf437303083042e4c`。
- `docs/specs/a2r-std/project.md` — SHA256 `caa68ad70bd6a80a90bbd073badffbc070f66e2ffd3be55c39d2ddaff6866041`。

权威输入：docs/specs/auto-api-gen/project.md（adapter归语言）、docs/specs/a2r-std/project.md（time/集合等）、docs/specs/auto-lang/vm/design/time-natives.md（epoch）。观察线索：qualify_a2r_std区分sqlite/sys与通用内嵌路径，业务字符串/标识符已有742防误替换；历史Blog日志59错是待复核输入，不是首轮新实测。

## 5. 详细设计

### 调查与根因边界

T-01先固定CLI二进制hash与实际编译源，使用未手修Blog原源做一次冷生成/编译，按首个根因压缩错误组（time限定、List构造/容器形状、字符串值/借用、模块返回类型等只是候选）。输出拟新增 `docs/plans/attachments/885/baseline-and-root-causes.md`，包含CLI→生成器→Rust→runtime→可执行服务的实际链与归属。当前代码已修/只需重建时，以正控和源版本说明关闭，不做空修复。

T-02将每个实际根因约简为编译可执行fixture；失败输入不能只看生成文本，需在真实Rust编译/运行阶段辨别。保留字符串/注释、同后缀用户模块名、已有canonical路径的合法控制。T-03只修复确认根因；不能全局盲替换a2r_std、把time常量化或修改业务ID协议。

### 规范增量

| delta_id | action | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-api-gen/project.md | 概括adapter职责 → 明确经实测支持的Rust服务runtime路径/参数形状及消费者验证边界 | 不以AST/字符串测试代替可编译消费者 | AC-01..04 |

若T-01确认还需改变a2r-std公开语义，先在本Plan中说明实际差异并补相应Spec增量/修订，不能凭本草案预先批准全标准库扩展。

## 6. 测试设计

证据政策：原始日志/编译诊断/截图/探针输出放本地 `docs/plans/attachments/`（不提交非md证据）；Plan内保留命令、退出码、版本、关键结果及本地路径+内容hash摘要。不得以日志里的pass字样代替断言。所有测试使用独立数据目录与明确端口；先检查占用，只关闭本次创建的PID，不批量杀同仓或其他用户进程。

| case_id | owner / AC | 真实入口与输入 | 关闭oracle |
|---|---|---|---|
| L-01 | T-01/02/04，AC-01/02 | 当前Blog生成链＋约简time.now_ms/now_sec、多参数/容器实际输入 | Rust编译exit0、运行值/结构符合公开合同；epoch ms/sec有效且关系一致，不以0/伪常量替代；历史错误按根因有闭合 |
| L-02 | T-02/04，AC-03 | 用户字符串/注释含a2r_std及custom_a2r_std模块、已限定路径、sqlite/sys合法控制 | 生成器不误写数据/用户标识符、重复规范化幂等，真实编译/执行内容保真 |
| L-03 | T-02/04，AC-03 | 每个实际类型/借用根因的合法控制及不合法输入 | 合法值顺序/结果符合语言合同，不合法不伪造编译成功；适用诊断阶段和预期在T-02固定 |
| L-04 | T-05，AC-04 | 固定交付工具链冷生成完整Blog服务，实际HTTP新建/保存/陈旧rev/重启 | 新artifact被启动，数据根隔离且输出正确；只在旧缓存运行或手改产物不得通过 |

先读现有 `crates/auto-api-gen/src/api_gen.rs` 定向测试，复用而非另造平行框架；拟新增fixture/测试由T-02落地并给精确命令。开发仅受影响模块/fixture；实现与最终review通过后merge持有裸cargo t＋适用tt（转译触面）等本仓门，不在起草/修复轮跑整档；不触UI消费者时不类推UI档。

## 7. 验收标准

| ID | owner_stage | 结果 |
|---|---|---|
| AC-01 | work | 当前实际根因与producer/runtime/consumer身份、输入和归属清晰；历史数量不冒充新测，无未知必需错误留待应用绕过 |
| AC-02 | work | 每个确认阻断有可编译/执行正控与根因关闭证据；time/容器/借用实际语义正确，未通过手修产物/常量/删功能“修复” |
| AC-03 | work | L-02/03合法兄弟路径、字符串/注释、用户模块和已有facade控制不回归，失败控制有非零/诊断 |
| AC-04 | work | L-04完整Blog生成服务在固定新工具链下编译并真实HTTP使用；回执给Blog可消费的精确commit/二进制/CLI及原根因归因；无需等待BLOG-004最终验收 |
| AC-05 | review | 固定实现/依赖，独立重建根因与Spec增量，AC-01..04无未闭项；说明上下文限制 |
| AC-06 | merge | merge-owned统一门通过、主仓交付、canonical/ledger/归档/清理收据与依赖身份完成，再解除Blog Rust前置；不宣称原生VM UI已通过 |

## 8. 执行步骤

- [ ] T-01（owner_stage=work；依赖无；AC-01）：在专用实现树核版本，用完整Blog源复核历史错误；输出baseline-and-root-causes.md并冻结适用输入/资源边界。
- [ ] T-02（owner_stage=work；依赖T-01；AC-01..03）：建立实际根因的最小编译/运行fixture、无误重写/合法兄弟控制和明确错误oracle，补精确复现命令。
- [ ] T-03（owner_stage=work；依赖T-02；AC-02/03）：只修确认的生成器/runtime/转译问题；若当前不重现则提交原因/正控证据，无需虚构代码补丁；越owner则登记最小前置。
- [ ] T-04（owner_stage=work；依赖T-03；AC-02/03）：受影响定向检查及L-01..03；不得用生成文本assert替代实际编译执行。
- [ ] T-05（owner_stage=work；依赖T-04；AC-04）：完整Blog消费者L-04闭环并准备SD-01，提交实现/摘要，输出可消费回执，转execution_done；原生UI不在本计划。
- [ ] T-06（owner_stage=review；依赖T-05；AC-05）：同一实现树暂停写入，最终review根因/兄弟控制及SD-01，不通过则回work。
- [ ] T-07（owner_stage=merge；依赖T-06；AC-06）：统一门、交付与Spec/ledger/归档/guard清理后向BLOG-004登记精确依赖版本，不仅写“框架已修”。

## 9. 复审记录

stage: new | plan_id: PLAN-885 | plan_revision: 1 | outcome: pass（首个有界根因调查就绪，非修复完成） | next: work/T-01（另行执行）。7项未勾选，本轮无Rust改动、构建或运行验收。

## 10. 待澄清事项

技术不确定性归T-01：哪些历史错误仍复现、是否CLI陈旧、根因是否在生成器/发射器/facade或Man。未授权整套a2r重构；超出Blog最小消费者闭环时先定界修订，不能承接语言平台全部gap。
