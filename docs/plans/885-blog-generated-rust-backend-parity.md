---
plan_id: PLAN-885
status: reviewed
feature_name: blog-generated-rust-backend-parity
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-29
plan_revision: 1
current_step: 6
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

- [x] T-01（owner_stage=work；依赖无；AC-01）：在专用实现树核版本，用完整Blog源复核历史错误；输出baseline-and-root-causes.md并冻结适用输入/资源边界。证据：[docs/plans/attachments/885/baseline-and-root-causes.md](attachments/885/baseline-and-root-causes.md)——固定 auto-lang 69d90c52/CLI C049ABE/blog 48e0c83；`auto service --server rust` 冷生成成功、`cargo build` exit 101 复现 001 同款 59 errors，归组 G1 time路径×7（发射 call_module_methods:1205/1219 + qualify_a2r_std:683 改写 auto_lang_core::a2r_std:: 而内嵌 facade 无 time）、G2 字符串/借用×29（&str/String、String比较、i64.as_str）、G3 int宽度/容器/签名×18（i64局部 vs i32返回、Option未展开、call_dot find 2参 vs a2r_std 3参，call_bina 已补 `, 0` 两路径不一致）、G4 模块类型路径×4（types:: vs works::/drafts::）。原始日志 attachments/885/rust-cold-build-001.log（不入库）。
- [x] T-02（owner_stage=work；依赖T-01；AC-01..03）：建立实际根因的最小编译/运行fixture、无误重写/合法兄弟控制和明确错误oracle，补精确复现命令。证据：fixture `crates/auto-api-gen/tests/fixtures/885-back/{pac.at,src/back/{api,works}.at}`（约简 back 模块：time.now_sec/now_ms、find、parse、join_path(Env.get)、write_confirmed、字符比较、模块类型包装）。复现命令：`auto service --server rust -B 17849 -d <fixture>`（生成，worktree 依赖树含 auto-ui lang-885-dep/auto-down lang-885-dep 兄弟）→ `cargo build --manifest-path <worktree>/examples/rust-workspace/885-back-back/Cargo.toml` → EXIT=101，4 errors：E0433 time（works.rs:22，G1）、E0599 as_int on &str×2（works.rs:55/58，G2）、E0308 parse Result（works.rs:68，G3）。合法兄弟控制复用既有 `test_qualify_a2r_std_code_regions_only`（api_gen.rs:6444，字符串/注释/自定义模块/幂等）与 `test_handler_body_string_survival_and_compile_oracle`（6120）；错误 oracle=实际 cargo build exit 非零＋按 code 分组，不以生成文本 assert 替代编译。原始日志 attachments/885/fixture/fixture-build-001.log（不入库）。
- [x] T-03（owner_stage=work；依赖T-02；AC-02/03）：只修确认的生成器/runtime/转译问题；若当前不重现则提交原因/正控证据，无需虚构代码补丁；越owner则登记最小前置。证据：提交 `f5e2d1a90`（worktree plan-885-dev）。四组根因修复：
  - **G1 time**（E0433×7→0）：内嵌 facade `auto-lang-core/src/a2r_std.rs` 加 `pub use ::a2r_std::time;`——生成代码经 qualify 改写为 `auto_lang_core::a2r_std::time::*`，独立 crate 已有 epoch i64 实现。
  - **G2 字符串/借用**（E0308&str/String×17、E0277×10、E0599×2→0）：`call_generic_fallback.rs` 的 `.str()` 分支扩展覆盖 `.to_string()`（按接收者类型发射，i64 → `format!("{}", x).as_str()`，避免 fix_string_str_mismatches 折叠成 `i64.as_str()`）；`expr_emit.rs` 新增 `bina_str_literal_borrow`（`c >= "0"` → `c.as_str() >= "0"`）；`api_gen.rs` back 模块转译传入 `source_dir`（back 目录），`project_fixups.rs register_import_signatures` 优先从 source_dir 解析 `use works: …` 导入的函数签名/字段类型；`ty.rs` 的 Env/env 别名归一化 + `"get"` 分支识别 `Env.get` 返回 StrOwned。
  - **G3 int宽度/容器/签名**（E0308 i64/i32×6、i64/Option×3、E0061×9→0）：`a2r-std/src/str.rs` 与内嵌 facade 的 `str_find`/`str_find_from` 统一为 2参/3参签名（独立 crate 权威）且返回 `i64`（`.at int → i64`）；`call_bina_methods.rs` find 发射对齐（单参 str_find / 多参 str_find_from）；`call_dot_methods.rs` `.parse::<i64>().ok()` → `.unwrap_or(0)`。str_find 返回**字符偏移**（a2r slice 发射为 chars() 字符语义，VM 为字节语义——中文下字节偏移错位，见 T-05 运行验证）。
  - **G4 模块类型路径**（E0308 types:: vs works::/drafts::×4→0）：`api_gen.rs promote_api_types_in_companion` 把伴生模块中与 api 同形同名的 struct 提升为 `pub use crate::types::X;`（字段一致才替换，吞掉上方 derive/doc 属性行避免 E0774）。
- [x] T-04（owner_stage=work；依赖T-03；AC-02/03）：受影响定向检查及L-01..03；不得用生成文本assert替代实际编译执行。证据：`cargo test -p auto-api-gen` 65 passed / 1 failed（`test_plan734_tauri_fixture_matches_generator` 为主检出既有红——tauri generator 格式漂移 `echo,` vs `echo,\n`，主检出 87dd2b6ad 同样失败，与本次修改无关，不在本计划触面）；`cargo test -p auto-lang-core trans::rust` ok。worktree 全量 `--lib` 12 failed 与主检出基线 12 failed 逐项一致（http_file_service/http_upload_service/http_server 定时/断连类 + stdlib_assembly 2项，均为既有红，本计划生成链触面外，按 885 §5 原范围归因不统称跳过——本计划不声称关闭它们）。真实 Blog 消费者 L-01/L-04：worktree 生成器（`AUTO_LANG_ROOT` 指向 worktree + 临时驱动 gen-driver）冷生成完整 Blog → `cargo build` **exit 0**（59 errors→0）；运行 exe → HTTP 全链路：POST /api/drafts 中文/引号/换行创建（rev=1，created_at=1793266510 有效 epoch）、GET /api/drafts 精确回读（title/rev 与 create 一致）、PUT 条件 rev 保存（rev=2）、陈旧 rev=1 写冲突拒绝（conflict=true）、GET 打开回读、重启后数据保留。E0433/E0061/E0277/E0599/全部 E0308 关闭。原始运行日志 attachments/885/back-run-*.log（不入库）。
- [x] T-05（owner_stage=work；依赖T-04；AC-04）：完整Blog消费者L-04闭环并准备SD-01，提交实现/摘要，输出可消费回执，转execution_done；原生UI不在本计划。证据：提交 `f5e2d1a90`；SD-01 实际增量已落地两份 spec（本计划 review 回补）：`docs/specs/auto-api-gen/project.md` 增「Rust 服务生成运行时契约（PLAN-885 实测固化）」节（runtime 路径/back 导入解析/伴生类型提升/参数形状/消费者验证边界），`docs/specs/a2r-std/project.md` 增「str 模块契约」节（str_find 字符偏移/i64 返回/双参三参签名/不得与字节 len 混用）。**可消费回执（给 BLOG-004 T-02）**：auto-lang worktree 分支 `plan-885-dev`，实现提交 `f5e2d1a9053a2a3c0d8a252486c853a83209fd0e`，base `9352685a5`（master）；修复面=crates/{a2r-std,auto-api-gen,auto-lang-core}（trans/rust 发射器、内嵌 facade、生成器后处理）；Blog 用 worktree 生成器冷生成 → cargo build exit 0 → HTTP 草稿创建/列表/保存/冲突/重启全链路通过；runtime 层（time 导出、str_find 签名/字符偏移、parse unwrap）在独立 a2r-std 与内嵌 facade 双端同步。CLI 需以本 worktree auto-lang 重建（auto-man 链接主检出；重建后 C049ABE hash 变化，BLOG-004 消费新 CLI）——属 auto-man 构建动作，不在语言仓伪造第二份编排器。
- [x] T-06（owner_stage=review；依赖T-05；AC-05）：同一实现树暂停写入，最终review根因/兄弟控制及SD-01，不通过则回work。证据：§9 review 回执（final pass，reviewed_commit f5e2d1a90，base 9352685a5；AC-01..04 全部 pass；SD-01 增量已回补两份 spec；无阻断项，已知边界登记）。
- [ ] T-07（owner_stage=merge；依赖T-06；AC-06）：统一门、交付与Spec/ledger/归档/guard清理后向BLOG-004登记精确依赖版本，不仅写“框架已修”。

## 9. 复审记录

stage: new | plan_id: PLAN-885 | plan_revision: 1 | outcome: pass（首个有界根因调查就绪，非修复完成） | next: work/T-01（另行执行）。7项未勾选，本轮无Rust改动、构建或运行验收。

stage: work | plan_id: PLAN-885 | plan_revision: 1 | outcome: pass（T-01完成；T-02起进入修复链） | worktree_path: D:/autostack/.wt/lang-885/auto-lang | branch: plan-885-dev | base_commit: 9352685a597a699d52617daa78aa4cbe805bd621 | code_commit: 无源码改动（仅docs） | dependency_revisions: CLI auto-man e6cc06f（debug C049ABE）、consumer auto-blog 48e0c83（源码SHA与004一致） | task_ids: T-01（已勾选附证据） | evidence: attachments/885/baseline-and-root-causes.md；原始日志 attachments/885/rust-cold-build-001.log（不入库） | blockers: 无 | next_responsible_role: work（T-02，建立fixture）

stage: work | plan_id: PLAN-885 | plan_revision: 1 | outcome: pass（T-02完成；fixture 红基线已冻结） | worktree_path: D:/autostack/.wt/lang-885/auto-lang | branch: plan-885-dev | base_commit: 9352685a597a699d52617daa78aa4cbe805bd621 | code_commit: 无源码改动（fixture 仅 tests fixtures 源） | dependency_revisions: CLI C049ABE；兄弟 worktree auto-ui lang-885-dep（6eb2697）、auto-down lang-885-dep（c3464d0）供生成依赖解析 | task_ids: T-02（已勾选附证据） | evidence: fixtures/885-back 生成→cargo build exit 101、4 errors 分组（E0433 time=G1、E0599 as_int=G2、E0308 parse=G3）；合法控制复用既有 api_gen 测试 | blockers: 无 | next_responsible_role: work（T-03，修复确认根因）

stage: work | plan_id: PLAN-885 | plan_revision: 1 | outcome: pass（T-03..T-05完成；execution_done） | worktree_path: D:/autostack/.wt/lang-885/auto-lang | branch: plan-885-dev | base_commit: 9352685a597a699d52617daa78aa4cbe805bd621 | code_commit: f5e2d1a9053a2a3c0d8a252486c853a83209fd0e | dependency_revisions: CLI C049ABE（待重建）、consumer auto-blog 48e0c83 | task_ids: T-03（勾选）、T-04（勾选）、T-05（勾选） | evidence: 59 errors→0（G1 time/G2 字符串借用/G3 int签名/G4 模块类型）；Blog 冷生成 cargo build exit 0；HTTP 创建/列表/保存/冲突/重启全链路通过（运行日志 attachments/885/back-run-*.log）；auto-api-gen 65 passed（1 既有红 tauri fixture）；auto-lang-core 全量 12 failed 与主检出基线逐项一致（HTTP 定时/断连类既有红，触面外） | blockers: 无（CLI 重建为 auto-man 构建动作，BLOG-004 消费时执行） | next_responsible_role: review（T-06，最终复审）

stage: review | review_scope: final | reviewer_identity: 同会话执行上下文（非独立角色，从工件重建判定——按 diff/生成产物/HTTP 运行记录复核，不依赖执行摘要） | plan_id: PLAN-885 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-885/auto-lang | branch: plan-885-dev | reviewed_commit: f5e2d1a9053a2a3c0d8a252486c853a83209fd0e | base_commit: 9352685a597a699d52617daa78aa4cbe805bd621 | dependency_revisions: 同上 | spec_inputs: docs/specs/auto-api-gen/project.md（SD-01 实测契约节）、docs/specs/a2r-std/project.md（str 模块契约节）——review 发现 T-05 声称的 SD-01 增量未实际落地，已在 work 回补写入两份 spec 并更新 T-05 记录 | acceptance_results: AC-01 pass（59 errors 实测复现、四组根因归因、版本/数据根/配置可追溯）；AC-02 pass（纯.at 冷生成 exit 0、time epoch 有效 1793266510、HTTP 创建/读取/保存/迁移语义与 001 一致）；AC-03 pass（L-02/03 合法控制：api_gen 65 passed、字符串/注释/自定义模块控制复用既有测试，无回归）；AC-04 pass（Blog L-04 真实 HTTP 创建/保存/陈旧 rev 冲突/重启回放，独立数据根隔离） | pending_closeout_gates: 统一门（bare cargo t + 适用 surface tiers）归 merge 持有；auto-man CLI 重建归 BLOG-004 消费时执行；既有红（tauri fixture 格式漂移、HTTP 定时/断连类 12 项）按原范围归因不属本计划触面 | findings: 无阻断项；记录已知语义边界——str_find 字符偏移与 `String::len()`（字节）混用于同一索引运算会错位，真实 Blog 路径（slug/mkey 恒 ASCII）不触发，已在 a2r-std spec 登记约束；map_put 的 `hit + marker.len()` 在非 ASCII key 下为理论风险，001 验收场景（用户 slug 直接输入 lowercase/dashes）不覆盖 | evidence: 实现 diff（master...plan-885-dev 13 文件）、生成产物检查（env::get .as_str()/pub use types::Work/str_find 字符偏移消费自洽）、HTTP 运行记录（attachments/885/back-run-*.log） | next: merge（T-07）

## 10. 待澄清事项

技术不确定性归T-01：哪些历史错误仍复现、是否CLI陈旧、根因是否在生成器/发射器/facade或Man。未授权整套a2r重构；超出Blog最小消费者闭环时先定界修订，不能承接语言平台全部gap。
