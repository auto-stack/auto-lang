---
plan_id: PLAN-899
status: drafting
feature_name: ACE组织收尾、历史probe清退与行号级调试
author: [Codex]
created_at: 2026-10-31
updated_at: 2026-10-31
plan_revision: 1
current_step: 0
total_steps: 7
supersedes_spec_components: [docs/specs/auto-test-case/build-storage-lifecycle.md]
new_spec_components: []
touched_goals: []
affects: [experimental/ace, test-infrastructure]
depends_on: [PLAN-896]
---

# [PLAN-899] ACE组织收尾、历史probe清退与行号级调试

## 0. 变更摘要

承接已交付896：ACE目前已是experimental/ace单workspace、29成员、单Cargo.lock与ace-*包名。清理剩余早期调研程序、过期描述和闲置旧target；成员dev/test改为保留行号，非成员依赖不留调试信息。保留独立workspace及职责明确的crate，不改引擎语义、ABI或已交付能力等级。

这是L1组织与构建配置收尾，不修改compiler/VM管线或运行时合同；沿用896架构与当前Specs，不另起架构设计。当前仅立项，依AGENTS.md实施前展示确认。

## 1. 目标

- G1：现有独立workspace作为ACE唯一实现与常规构建入口；现行说明、脚本及后续计划使用一致路径。
- G2：无长期职责的早期probe有据删除；长期语义、负控、ABI、性能与资源测试保持并按职责命名。
- G3：dev/test成员默认debug=line-tables-only，非成员debug=false；显式完整调试仍可用。
- G4：安全回收主检出残留旧ACE缓存，列真实回收及必要完整构建后稳态空间。

非目标：大crate化、改算法/协议/金样、关闭慢测、缩样本/规模/门限、跨树共享可写target、清其他worktree/全局Cargo缓存、改其他活动计划合同、迁往其他仓。experimental保留当前能力状态标记，不将整理等同于完整ACE稳定化。

## 2. 架构方案

建议维持独立ACE workspace，暂不并入auto-lang。当前29包已共用锁和target；仓规与Spec要求普通语言测试不隐式编ACE，ACE另有四个no_std静态运行时panic-abort配置及专项构建入口。并入主workspace须重审这些隔离合同，本次收尾无必要绑定。未来依据真实重复编译数据另作决策，不声称独立workspace不存在跨workspace重复编译。

实现根保持experimental/ace/{crates,tests,grammars,docs}。普通target与target-batch维持896约定；依赖方向、默认成员及外部锁版本不顺手改变，仅经T-01证明退役的工具及无消费者依赖允许移除。

## 3. 技术栈

现有Cargo workspace/profile、Rust/nextest、固定cargo t组、PowerShell验收脚本及scripts/test-governance/build_storage.py。不引入新runner、永久逐用例目录或缓存框架。

## 4. 需求分析与背景调查

### 4.1 授权、来源与工作区

用户要求统一ACE workspace、清理不适合长期测试的早期probe，并像auto-lang一样保行号关闭其余调试信息。已授权调查与具体整理方案，依仓规L1实施前须展示确认。未授权改oracle/产品能力或清其他会话材料。主检出已有大量其他会话文档WIP，不覆盖、不代提交。

基面ee0a1059e；调查2026-10-31。本轮只读代码、清单和字节统计，未编译、测试、删除源码或缓存。唯一拟实施树D:/autostack/.wt/lang-899/auto-lang，分支plan-899-dev；实现、测试与独立复审同树顺序完成。main仅计划/规范簿记及经守卫确认的本树旧缓存环境收尾。

| 来源 | 观察事实/版本 | 本包约束 |
|---|---|---|
| docs/specs/overview.md、docs/specs/auto-ac/project.md | 总览部分正文仍旧名称；模块Spec已说明896新workspace | 按真实代码与已交付合同核路径，不复制过期名称 |
| docs/plans/archive/896-merged-invocations-and-consumer-closure-audit.md | archived/delivered r7；delivery a9bd4f948c1280c53522ddb343acfa1fe56f6a57 | 不重开896；附件迁移摘要“核验中”是历史，非当前终态 |
| experimental/ace/Cargo.toml | SHA256 347d34b26fb7902ad6af3af1c1d9b2076e68bd91eaa72a03f5d0f93aa11e7ecd；29成员；dev/test debug=0 | 已统一；新配置比0增加行号表，不能谎称这项单独减小体积 |
| docs/specs/auto-test-case/build-storage-lifecycle.md | SHA256 8dc45792dd3210b575be37a033c0ea7015f6d8d1584cced891e123faf40ed573 | 独立workspace、target归属与活动/证据/reparse保护 |
| 根Cargo.toml | dev line-tables-only；dev.package.* debug=false | 用户所指主workspace策略；ACE test显式核生效值 |
| scripts/test-groups.toml、.cargo/config.toml | 固定ACE常规/adapter/batch/正式计量入口 | 不用技能中的旧cargo tm；898正在清退旧设施，改共享文件先核稳定接口 |
| docs/plans/KNOWN-DEBT-AND-RISKS.md 798 R12段 | mem_stage_probe支撑历史内存归因/改进机会 | 名称含probe或历史正文有引用都不能单独决定去留 |

本树11个旧ACE目录顶层已无manifest，主要仅剩target及嵌套历史缓存。非reparse只读遍历约51.8GiB逻辑字节；执行前重测唯一文件数据、owner与保护项，不承诺全部可删。新ACE树826文件约11MB，当前本树没有新ACE target，不能用空构建目录冒充稳态收益。

仍在用的例子：cte_probe_worker被comptime正反例与语言私有adapter实际启动；abi/cside/layout_probe.c是C布局oracle；native collections_runtime_probe fixture是真实ABI消费者。av_wire_826_bench/av_runtime_825_bench输出正式原始样本，须追父分析器。mem_stage_probe核当前长期诊断职责，未知不直接删。

### 4.2 并行边界

898正在独立树改测试设施；893/891草案仍含旧路径。本包不改其合同，交付迁移表供其后续使用。共享runner/固定组以已交付稳定版本对接，不覆盖WIP。不重写历史归档/复审证据，不全仓替换probe字符串。

## 5. 详细设计

### 5.1 probe去留与命名

T-01审计ACE跟踪源码、bin、fixtures、README、manifest描述及正式verify/工具调用者，形成一次性表：真实路径、长期职责、调用者、现行义务/问题、保留/改名/转长期测试/删除、理由和验证入口。不是正常运行前置目录。

删除必须证明一次性可行性/打印/旧实现对比已完成使命，且无有效消费者及未解决问题；重复测试须证明实现、输入、生命周期与oracle等价并指出实际长期替代。不能仅按名称、年代、耗时、归档状态或rg零命中删除。有效工具按职责命名；正式协议/schema/profile与历史证据标识不机械改。

拟将cte_probe_worker改为ace-comptime-worker；T-01核完整调用面，再原子更新Cargo bin、CARGO_BIN_EXE、worker发现/来源hash、PS脚本、固定组和帮助说明，保持cteworker/1及父侧判定。旧编号bench按真实用途命名，保原样本/分析链。README与manifest的“独立workspace”过时注释同步修正。任何删除/保留须在审计表有具体理由，不能大范围一概保留代替本次清退。

### 5.2 调试配置

ACE根profile.dev与profile.test显式debug=line-tables-only；两者package.*设debug=false，仅作用非workspace成员。full-debug成员debug=2；文档说明package override优先级，恢复依赖完整调试需调整该override，不谎称单个环境变量可覆盖通配设置。runtime-dev继承行号级配置并保panic=abort；runtime-release、syntax-release及普通release不借本包改变。优化、断言、溢出与incremental不顺手改。

### 5.3 空间收尾

候选旧根：experimental/{ac-artifact-probe,ac-av-probe,ac-case-worker,ac-core,ac-cte-probe,ac-mir-native,ac-semantics,ac-syntax,auto-a2x,auto-abi-probe,auto-mir-lower}内实际旧Cargo target精确根。T-01枚举嵌套target，不对父目录宽泛递归删。

复用build_storage.py默认预览；--retired-source必须是git核实的迁移前HEAD祖先，确认闲置后才--apply --idle-confirmed。拒绝源码、活动、必要证据、reparse及未知owner，不停止他方进程。先按最新5个完成计划/在途/未解决问题政策核证据保护，不搬整target到plan-artifacts。

实施树完成必要完整组后统计稳态，分target/profile/deps/incremental/PDB/其他。main旧缓存清理由merge负责，实施树不能代替main可删性证明。保护项具名保留；精确缓存清理后仅删除已核空旧目录。不承诺固定节省百分比。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-test-case/build-storage-lifecycle.md | ACE dev/test debug=0→成员line-tables-only、非成员false，保专用profile | 用户行号要求 | AC-02 |
| SD-02 | modify | docs/specs/auto-test-case/build-storage-lifecycle.md | 补实际旧缓存收尾、具名保护与probe长期职责/替代规则 | 防误删有效oracle与证据 | AC-03/04 |

## 6. 测试设计

以下均拟执行，本轮未跑。T-01明确完整职责组及必要集成/消费组理由。profile触及整个ACE，因此完整常规组必验；正式性能body不变的证据仅在身份/输入/工具/profile有效性核实后复用。改名影响的正式调用链必须真实跑通，零选择不算PASS。

| ID | AC/不变量 | 入口及可观察oracle | owner_stage/边界 |
|---|---|---|---|
| V-01 | AC-01 单workspace/语言隔离 | cargo metadata --manifest-path experimental/ace/Cargo.toml --no-deps --format-version 1 --locked；29基线成员减有据退役项，workspace_root一致；普通语言入口无ACE准备 | work；不建动态消费者图 |
| V-02 | AC-02 实际调试信息 | 同树受控verbose build及test-no-run核代表成员/依赖rustc参数：成员line-tables-only、依赖无调试；full-debug成员2、runtime abort | work；不能只读TOML；同配置有效证据复用 |
| V-03 | AC-03 保长期判定 | cargo t ace、cargo t ace-runtime及受影响verify scope；审计表逐删除项/替代项核；改名worker真实IPC/ABI结果通过 | work；列发现/执行/条件排除，原ignore/平台/规模保持 |
| V-04 | AC-03 正式工具链 | 改名benchmark/父分析器按原入口完整原子及负控；必要时cargo t ace-performance，先核触面 | work；正式样本/阈值不降；不能以常规组绿代替专项 |
| V-05 | AC-04 安全回收 | 现有清理器有效守卫控制，main候选preview→保护核查→apply→后验；真实唯一数据回收与完整构建稳态分列 | merge；无法证明安全拒绝该scope，具名例外 |

不跑无触面语言全套或AG。独立复审重核删除理由、遗漏/延后/workaround、所有AC/SD与真实退出状态；warning/fmt/调试输出按触面核验。当前不创建子代理/新聊天；实施后按授权角色安排独立上下文审查，不能以实现自检直接代最终review。

## 7. 验收标准

- [ ] AC-01（owner_stage: work）：现有单workspace及普通语言隔离保持，现行说明/入口不再误述旧结构。V-01/审计表证明。
- [ ] AC-02（owner_stage: work）：实际dev/test成员保行号、非成员无调试；完整调试可显式开启；原release/runtime合同保持。V-02证明。
- [ ] AC-03（owner_stage: work）：所有删除有使命结束/替代证据；有效语义/worker/ABI/资源/性能职责保持；改名真链通过，无空选/减样本/静默条件变更。V-03/04证明。
- [ ] AC-04（owner_stage: merge）：main旧缓存精确安全处置，保留例外具名；真实回收/稳态分列，无他树/证据误删。V-05证明。
- [ ] AC-05（owner_stage: review）：独立逐AC及删除审查pass，零未处理本包warning/格式问题；既有边界具名，SD可沉淀。
- [ ] AC-06（owner_stage: merge）：合入、Spec回写、归档、最新留存及worktree guard清理完成。

## 8. 执行步骤

| ID | owner_stage/依赖 | 路径、操作与结果 | AC/验证 |
|---|---|---|---|
| T-01 | work/方案确认 | 核896、898基面；唯一实施树审计probe及真实消费者/义务，列退役/改名/保留，main旧精确target/保护项及选组理由；主计划摘要，本地原始表不入库 | AC-01/03/04；rg、manifest、真实owner/hash/祖先核对 |
| T-02 | work/T-01 | experimental/ace/Cargo.toml与README落行号配置/override说明，验证实际rustc参数 | AC-02；V-02 |
| T-03 | work/T-01 | 按审计表改ACE源码/bin/fixtures/manifest/README及真实消费脚本；有据删除/职责改名，保协议与oracle | AC-01/03；V-01/03/04 |
| T-04 | work/T-02/03 | 冻结提交跑相关完整固定组/必要消费，记录首红/修复/重放、计数、编译/发现/运行/清理/总时长与空间；准备SD文本 | AC-01..03；V-01..04 |
| T-05 | review/T-04 | 同树冻结写入，独立逐AC/删除/遗漏审查，Spec-impact与债务核；需修回原树 | AC-05；revision-bound review，执行前不勾选 |
| T-06 | merge/T-05 | main旧精确target preview/apply/后验及具名保护；SD沉淀、实际回收/稳态摘要 | AC-04；V-05，归属变化重新preview |
| T-07 | merge/T-06 | 按auto-plan:merge合入、规范索引/归档与最新5计划留存；wt-guard clean后删唯一树/分支/空组 | AC-06；本仓archive路径 |

下一步：r1确认后建lang-899唯一树，T-01先核再处置；T-02、T-03只在审计结论及稳定共享接口具备后执行。同范围等价机械调整继承授权，不重复索权。

## 9. 复审记录

2026-10-31 / stage: new / PLAN-899 r1 / outcome: pass（草案合同核查，非实现验收）/ next: 方案确认后work。任务覆盖AC/SD，来源与入口已核；未实施、编译、清理或授予实现pass。只提交本计划及.next-id，不代交其他WIP。

## 10. 待澄清事项

建议确认：维持现有独立ACE workspace，执行本计划的probe收尾、行号配置和旧缓存安全清理。若用户决定并入auto-lang或移出experimental，须先修订隔离合同、共享profile/默认成员/调用入口及验收，再执行搬迁，不视为等价改名。
