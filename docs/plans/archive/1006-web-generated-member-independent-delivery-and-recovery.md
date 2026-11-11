---
plan_id: PLAN-1006
status: archived
feature_name: Web生成成员独立交付与有限恢复
author: [Codex]
created_at: 2026-11-11
updated_at: 2026-11-11
plan_revision: 1
current_step: 11
total_steps: 11
depends_on: [PLAN-890, PLAN-910, PLAN-1003]
supersedes_spec_components: [auto-web/docs/specs/delivery.md, docs/specs/repository-boundaries/generated-cargo-build-lifecycle.md]
new_spec_components: [auto-web/docs/specs/generated-service-member-delivery.md]
touched_goals: []
affects: [auto-web, auto-generation-core, auto-man, generated-delivery]
---

# [PLAN-1006] Web生成成员独立交付与有限恢复

## 0. 变更摘要

关闭D-942-P2：交付公开Web服务成员导出入口，让普通Auto源生成的Rust服务不借原开发workspace、主检出或全局AUTO_*路径，也能在仓外fresh/locked构建并完成真实HTTP与持久数据任务。auto-web是领域生产owner；语言只提供中立生成/源闭包支撑，Man只编排公共CLI。

用户2026-11-11明确授权“先立计划与恢复清单，暂不实施”。本r1为drafting、0/11；没有实现、构建、测试运行、安装、恢复演练或发布。989接收本计划正式交付后才可能过E-03；有编号不代表P2关闭。

## 1. 目标

- G1：一个明确的export-only公开入口，输出可移根的Web成员包，包括源、锁、SDK依赖闭包、必要资源及可核身份。
- G2：保持正常开发生成/服务启动和1003 C/full策略；拒绝坏源/错SDK/不完整依赖/路径逃逸，失败不破坏既有可用交付包。
- G3：真实Auto业务而非手写Rust占位响应通过仓外HTTP、状态写入与独立读回；auto-chat与auto-database的真实导出路径都纳入。
- G4：固定首版A/B包替换和恢复合同，交付给989复用；不冒历史发行版本兼容、不把目录复制或冷启动当恢复通过。

非目标：集中共享workspace、ACE M2接管、换HTTP/DB实现、Vue/Tauri/二进制跨平台安装器、Linux发行版准入、远程发布、用户数据迁移或清缓存。包包含可重建源闭包；不保证无网络冷缓存构建，不随包复制整个Cargo registry/target。现有UI/Man/Web SDK安装仍由各owner维护，不另造安装系统。

## 2. 架构方案

普通项目输入→Web的真实解析/assembly/共享lowering→显式输出位置的受管成员→中立源/SDK闭包解析→隔离staging校验→完整交付目录提交→仓外消费者locked构建→实际TCP服务/文件或DB独立读回。

| 边界 | 主责与允许变化 | 不变量 |
|---|---|---|
| Web领域 | auto-web/api_gen、rust_workspace及新delivery模块（新增）；选择资源、成员内容与HTTP/DB语义 | 不依赖UI/Man/Tauri，不复制旧混合auto-api-gen或服务算法；auto-web-contract保持纯 |
| 中立生成 | auto-lang/crates/auto-generation-core现有source_owner、shared_generation、rust_workspace、sidecar | 仅公共源选择、生成载体、依赖闭包/身份与原子提交小机制；无Web业务、VM/AST/Store复制 |
| 公开入口 | auto-man/crates/auto的命令定义/分派（按需薄接线） | 调Web公开库接口，不在Man重写导出、SDK vendor或服务生成算法 |
| 消费验证 | Web定向integration测试及新独立consumer driver；语言既有固定controls | 从实际CLI生成，不手改生成源/manifest补绿；禁用原开发根后仍真正构建/运行 |
| 收口 | 本计划交付P2及RC-WEB-01能力/结果；989核跨渠道最终组合 | 不等989先通过，989不实施P2；不重做1003/068 |

依赖890负责Web独立边界、910负责受管生成生命周期、1003负责生产debug/交付源输入。930的UI delivery方法是参考与必要回归来源，不把UI工具当Web生产依赖。990 DB导入修复已在Web正式主线，作为基线事实；不等待983/978/986/987全部实施。与989无循环：1006交付→989 T-02冻结→T-03～05最终组合验证。

## 3. 技术栈

现有Rust/Cargo、AutoWeb/Axum与公开语言SDK，Python标准库仅用于隔离消费/HTTP及独立文件/SQLite读回。沿用现有锁/资源/owned-process机制；不引入新后台服务、包管理器或通用部署框架。

中央登记沿Web AGENTS及README的890/990模式：Web尚无本地号源，本计划在语言仓正式取号，**不表示生产归语言仓**。未来唯一实施组拟`D:/autostack/.wt/lang-1006/{auto-lang,auto-web,auto-man}`，各仓分支`codex/plan-1006-{lang,web,man}`；无实际改动的仓只读钉正式源，不为凑三仓建树。一个仓一个权威树，work/review顺序在同树进行，无junction/symlink。

## 4. 需求分析与背景调查

### 授权与现状

立约轮（2026-11-11早前）仅做计划/恢复清单与文档同步。**开工授权（2026-11-11）**：用户以 `/goal 完成计划1006` 显式启动交付，按 auto-plan-goal 技能一次性授予全链：在本计划专属worktree内实施（含建树）、独立复审、有界修复与merge（Spec沉淀、归档、清理、落地核查）。该授权不含目标/验收/范围/预算/仓集变更，不含tag、部署或远程发布。§8原"当前无实施授权"自本授权起失效。恢复对象以[统一恢复清单](reports/split-refactor-recovery-objects.md)为唯一跨渠道合同，本文只定义P2实现面。

已读语言[overview](../specs/overview.md)、[生成生命周期](../specs/repository-boundaries/generated-cargo-build-lifecycle.md)、[C生产Spec](../specs/repository-boundaries/generated-debug-policy-production.md)及Web的[project](../../../auto-web/docs/specs/project.md)、[architecture](../../../auto-web/docs/specs/architecture.md)、[delivery](../../../auto-web/docs/specs/delivery.md)、[testing](../../../auto-web/docs/specs/testing.md)。当前SDK交付已有能力，但没有成员级独立导出合同，不能把目标描述成现状。

| 已观察来源 | 实际事实/身份 |
|---|---|
| Web HEAD `2d698413b368896f913eba375ff788c795f395cd` | 990 DB sibling-import修复已交；不是P2导出交付 |
| Web `crates/auto-web/src/api_gen.rs` | `generate_api`→`generate_rust_server`→`generate_cargo_toml`；后者继承workspace依赖；SHA256 `684200fd0e00b4031322b1cc9884cd88493094dd6c7d42f053a1183c07b34794` |
| Web `crates/auto-web/src/rust_workspace.rs` | `try_ensure_workspace_for`先调中立根，再注入开发Web path依赖；SHA256 `49f8ec90ce252056261aacf76831e2e4be8309b088df98c55f49941adc53eb31` |
| Web `scripts/web-install.py`、`scripts/web-repository-distribution.py` | SDK source/verify/install及prepare/verify/swap负控已有；现消费把整个SDK group搬走，不是单独Web成员自足包；不新造第二套安装器 |
| Web `docs/specs/delivery.md` | SHA256 `f3f3546375e9bfb70dac01564d5bbdb2aad487fc577591551d7634ffd049ecdb`；现状与P2差额分开 |
| Man HEAD `347c29f7ddc6650a94548f4523b3bc8f0e7cae17` | `gen --delivery-root`接UI；`auto service --server rust`生成后启动；皆不能直接认作Web独立成员导出 |
| Man `crates/auto/src/{cli_spec,main,cmd_service,tool_ui_ops}.rs`及`auto-man-runtime/src/http_service.rs` | 现有分派/owned child/serve_rust是薄入口接线参考；cli_spec SHA256 `5909dfa77ba5d22e35792db11af3adf2bdeaada4a03b9b08c1be5b2c677f31a9` |
| 语言1003正式lang `11e1f1fd5e79766266ce8d54356155f018b8d661`、UI `393ea17ceaaea19c58da294ea292c19b273263b8` | C/full与UI vendor二级闭包已修，P1/R7关闭；不能把修P1等同修P2 |
| 真实业务及原测试 | auto-chat/src/back/api.at的POST委托真实db，auto-database/src/back/api.at的SQLite读API；Web tests/http_generation.rs、http_runtime.rs、db_import_context.rs与885-back夹具均存在。885-back用HOME写数据，新增验证须改为独立合成数据根，绝不触碰用户HOME |

以上是只读事实；新接口/新测试下文均是拟建，不是已执行证据。开工T-01冻结实际干净源，记录较上述观察的漂移；不能用未提交WIP或旧PATH工具冒生产版本。

## 5. 详细设计

### 5.1 公开入口与输出

拟新增Web库接口`export_rust_service`及Man命令：`auto service-export --dir <project> --delivery-root <bundle>`。这是**拟新增、目前不可执行**的命令；只导出Rust服务，不启动监听，不复用UI专属`gen`路由。正常`generate_api`、`auto service --server rust`及默认开发目录行为保持。等价命名可由T-01核现有CLI约定后记录，不能取消export-only语义或换成手工拷贝命令。

接收显式source-selection、生成context与输出位置；解析仍以原项目为语义根，输出重定向不改变import/assembly身份。不靠进程级环境改写生成根。依真实back成员声明/role确定成员，不用包名glob或散列猜目录；无API/VM-only/非法或lenient占位生成必须明确拒绝，不能成功导出空目录。

包包含app成员、闭包SDK源、必要资源、独立根manifest、Cargo.lock及版本化delivery manifest（新增）。所有workspace继承依赖/包字段在包内解析；依赖版本遵守实际锁/SDK单源，`auto-web-contract`不得同源异份形成类型分裂。源码闭包包括1003修复的semantics-contract与stdlib/contracts等真实传递输入。允许SDK source复制但不复制算法，不从开发主检出偷偷补件。未变输入导出内容/manifest稳定；新源/锁/工具/配置/资源变动可观察地更新身份。

### 5.2 源、资源与提交事务

源选择在写入前校验：显式选中SDK/兄弟源→既定受管解析，不把缺失选中源降级到主检出；记录repo完整SHA、文件内容身份、实际SDK closure/工具及锁。C/full及用户有效覆盖沿1003规则，不能把full包标成默认C；source manifest/交付receipt纳入有效策略。

资源由实际import/运行配置给出最小必要集合，支持项目内普通文件；拒绝越界`..`、不可解析外链、junction/reparse及错误源。用户私有数据库/凭据/日志、.git、target、node_modules不随包复制；运行数据由显式外部合成数据根提供。资源缺失报可识别错误，不用空响应/默认资源掩盖。可支持的额外外部资源须先形成显式owner/输入合同，不通过扫盘发现。

prepare→校验闭包/manifest→提交整个owned目录。原包在提交前失败时逐文件hash不变，错误中给出阶段与自有残留；提交/恢复失败必须准确报告可用旧包位置，不能虚报rollback成功。不得替换未知归属非空目录或删除活动输出。T-01对“现有中立提交原语复用”与“从现有owner事务最小抽出中立原语”最多两种方案做全链可行性决策；Web不能反向依赖Man，禁止复制一套安装算法。该决策不重开1003，也不扩为全仓通用发布改造；若两种均不能满足不变量，记设计阻断再修合同。

### 5.3 仓外消费与真实任务

新driver拟`auto-web/scripts/generated-member-delivery.py`，测试拟`crates/auto-web/tests/member_delivery.rs`，A/B普通Auto夹具拟`crates/auto-web/tests/fixtures/member-delivery/{a,b}/`（均新增，由T-04/06落地）。调用正式公开CLI导出，移至不同独立目录；生成使用scope内钉正式源的owned项目/SDK副本，导出后令这些自有原根不可达，消费者AUTO_*覆盖清空或指向不存在路径；不重命名/改权限影响用户主检出。核Cargo metadata/实际编译输入仅来自bundle及获准Rust工具链/registry，结合绝对路径/源码闭包审计与原根不可达探针证明不借主检出；单靠搜索无字符串或相关环境变量已清空不能签闭包成立。T-01落实可观测的输入审计方式。

正式导出生成并自带Cargo.lock；消费从空target做fresh `cargo build --locked`，同输入再次locked构建，移根后再用空target locked构建；缺锁反例拒绝。，读取真实TCP响应、资源内容、业务错误码，并在关停/重启后独立读持久文件/SQLite；实际子进程退出和端口释放要核。auto-chat真实注册/会话/消息及auto-database真实SQLite读取纳入支持面，不把总请求200或预编译旧binary算通过。若真实产品暴露所需缺口，修复仅限成员交付/资源闭包；新增产品业务缺陷回owner、不得换空夹具绕开本计划必需任务。

### 5.4 有限恢复与989交接

按[RC-WEB-01](reports/split-refactor-recovery-objects.md#rc-web-01web成员包)实施：同一正式导出器由有版本差异的普通Auto源A/B产出两个真实自足包，明确这是首版受控包对而非历史发布。A成功写状态→停进程并备份数据→B独立准备/验证→切换到B继续同schema读写→B的prepare/verify/提交失败保持A与其数据→按既有受支持目录切换/备份恢复A并独立读回。包与用户数据分离；首次恢复/应用后数据恢复的断言分别记录。

Schema固定为同版本；不承诺任意DB升级/原地降级，不用删数据再启动制造成功。持久状态fixture用现有File.write/read或已有db API表达普通Auto业务，路径只来自显式测试配置；T-01/04核实际签名。POST返回与磁盘读回必须同时断言，错误目录/不可写与新包资源损坏有负控。

交接989须给实际生产入口/帮助、Plan revision、review/merge/Spec、各仓正式SHA、有限支持面、manifest/lock/资源/C策略、A/B可重建源与预期结果及实际摘要。生成后才能填写工件hash；本计划不等989终验，989可按身份/断言复用充分证据，只补新耦合或失效部分。

### 5.5 场地、证据与规范增量

未来开工才建`D:/autostack/plan-artifacts/plan-1006/`，按repo/arm登记owner/用途/状态；driver显式`--scratch`、工具/源/target，所有源副本、安装、包、DB、日志、外置target均在此，不依默认输出另散落。正式构建可用专属树target。不能删除用户持有f3f98a9e缓存或其他计划目录。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | auto-web/docs/specs/generated-service-member-delivery.md | SDK整体安装/开发成员生成→公开成员导出、闭包/资源/锁/身份与失败保全、首版恢复支持 | 关闭P2、可重复仓外消费 | AC-01～07 |
| SD-02 | modify | auto-web/docs/specs/delivery.md | 区分SDK交付与成员导出；绑定实际公开命令/资源/恢复/限制 | 不扩大现有能力声明 | AC-01/04/07/09 |
| SD-03 | modify（仅实际公共变更） | docs/specs/repository-boundaries/generated-cargo-build-lifecycle.md | 若抽出中立闭包/提交原语，明确owner与输入；C/full和开发模式不变 | 防领域倒依赖及重复算法 | AC-02/03/06/09 |

SD-02/03的supersedes路径已暂记；SD-03若实际无公共变更，review记录无差额并移出，不发布虚构变更。本轮不修改canonical。T-11将命令/退出码/关键断言、源/工具/产物hash及失败/限制折入本Plan后，按现行guard/所有实际输出路径对账删除本计划自有scope/树内产物；具名未解决持有须owner+理由+关闭条件，不因归档自动释放。

### 5.6 T-01 冻结裁定（2026-11-11 work，全文见 plan-artifacts/plan-1006/t01-design.md）

- **提交机制=方案B′**：中立提交原语缺位（方案A不成立）→在 auto-generation-core 新增中立 `commit` 模块（staging标记+拒未知非空+backup/swap/还原+fail-point+洁净拷贝/文件映射；staging 仅清同 pid 自有残留，他 pid 保留具名报告），Web delivery 消费之；不 vendor/复制 man-install.py，语义合同对齐既有 owner 事务。SD-03 记 owner=auto-lang。
- **导出路径=正常生成后收割**：export 先跑正常 `generate_api`（语义根/落点/收据不变），delivery 把新鲜成员 crate 拷入 bundle staging；生成核心不加落点参数。语言侧唯一改动=中立 commit 原语。
- **Bundle 布局**：根 Cargo.toml（members=app/*，exclude=["sdk"]，workspace.dependencies 锚 sdk/crates/* 相对路径+managed_profile_toml 保 C/full）+ Cargo.lock（staging 内 generate-lockfile）+ .cargo/config（本地 target）+ app/<member>/ + sdk/{crates 闭包, vendor 同形镜像, stdlib/contracts 编译输入, man-install ROOT_INPUTS 集, licenses} + resources/ + delivery-manifest.json（web-member-delivery/1，无时间戳保字节稳定）。闭包=path 依赖图传递遍历（离线），禁 auto-ui*/auto-man/auto-lang/auto-api-gen/tauri。非 crates 布局白名单（T-02 实测定稿，复审 F-1006-R1-03 修订）：vendor 树=experimental/ace/crates/semantics-contract + vendor/autodown-test-parser；数据树=crates/auto-lang/test/cases（build.rs 身份输入）。
- **漂移记录**：web-install `stage_source` 的 `crates/<name>` 布局校验对当前闭包（含 experimental 语义合同，PLAN-944 起）不可用——旧脚本不修，成员交付用同形镜像绕开。
- **接口**：`export_rust_service(&ServiceExportRequest{project_dir,delivery_root,resources}) -> ServiceExportReceipt`；源选择沿用既有 env 显式合同（export 不改写进程环境）；strict-parse 失败/无端点/lenient 脚手架=拒绝导出；fail-point=`AUTO_EXPORT_FAIL_POINT=prepare|verify|swap`。
- **数据根**：fixture 只认 `MEMBER_DELIVERY_DATA`（缺失=命名错误，无 HOME 回退）；chat=`CHAT_DATA_DIR`（整目录）+`CHAT_TEST_MODE=1`；database=请求体 path。全程不触用户 HOME/APPDATA。
- **命令面**：`auto service-export --dir <project> --delivery-root <bundle> [--resource <rel>]...`（cli_spec `ServiceExport` 变体+real_main 薄臂，Core/full 共用，export-only 不监听不复用 UI gen）。

## 6. 测试设计

所有V均未执行，owner_stage=work。T-04新增测试/driver使V可执行，T-06补恢复面。现存Web scoped入口为`cargo nextest run -p auto-web --test http_generation`、`--test http_runtime`及`--test db_import_context`；新`--test member_delivery`在T-04创建后才能执行。HTTP进程沿原串行/资源隔离约束，不缩原断言。

| ID | AC/不变量 | 实际入口/区分性输入 | 观察oracle/关闭条件 | 边界/任务 |
|---|---|---|---|---|
| V-01 | AC-01/02 成员闭包 | 新公开CLI→普通A与真实chat/database→独立包 | workspace继承均在包内可解；无开发根后fresh/locked构建及TCP真实业务成立 | Windows Rust，T-02～05 |
| V-02 | AC-02 移根 | driver将包移到另一目录，自有原生成根不可达，实际输入仅来自获准工具链/registry与bundle | 再次locked构建/HTTP/资源同值；故意漏SDK必须失败 | T-04/05；非离线保证 |
| V-03 | AC-03 源拒绝 | 坏Auto、无API、VM-only、lenient占位、显式SDK缺/同名异源 | 非零/可识别阶段，无监听，old包hash/正常行为不变 | T-01～04；不静默fallback |
| V-04 | AC-02/04 资源边界 | 合法资源、缺资源、越界/reparse、源码私有DB/凭据 | 合法内容准确，非法拒绝；包无私有数据和开发绝对路径 | T-03/04 |
| V-05 | AC-03 事务 | 现有A→B，prepare/verify/提交注入故障 | 提交前A字节不变；错误/残留准确，恢复后A真实任务成立 | T-03/06；不制造系统断电保证 |
| V-06 | AC-05 状态/错误 | A/B业务POST/GET、错误参数及不可写数据根 | HTTP值/码与独立文件或SQLite读回一致，关停重启持久值仍在 | T-04～06 |
| V-07 | AC-05 真实产品 | chat注册/会话/消息及database已种SQLite读API | 生成包实际TCP业务与独立数据一致，非空壳200/旧exe | T-05；原产品支持面 |
| V-08 | AC-06 策略 | 默认C/full有效覆盖分别导出；manifest/receipt造假 | 有效策略沿1003，成员分类/身份正确，full不冒C；release不受dev块改变 | T-02/05 |
| V-09 | AC-06 开发回归 | 原generate_api、service --server rust及UI独立交付consumer | 原入口/输出/owned shutdown保持，无Web反依赖UI/Man | T-05/07；按实际共享触面 |
| V-10 | AC-07 升级 | 正式导出器A→B同schema | A/B源/包/版本/lock身份分明，B读取A数据且完成新业务读回 | T-06；非历史跨schema迁移 |
| V-11 | AC-07 恢复 | 破坏B依赖/manifest/资源及提交失败，恢复A+备份 | A原资源/功能/持久值独立读回，错误回滚不许虚报成功 | T-06；仅自有合成环境 |
| V-12 | AC-02/03 新鲜度 | 相同输入重复、改import/lock/资源/工具 | 相同包稳定，相关变化使receipt失效/重建，不能仅改标签 | T-03～05 |
| V-13 | AC-08 生产交接 | 读实际CLI/help、源/Spec/review/merge与989 E表 | 有正式P2交付才能关债，有草案/SDK安装不够；989不倒灌实现 | T-07～10 |
| V-14 | AC-03/07 残留与活动 | 未知非空目的地、活动owned服务/Windows文件占用、失败清理 | 不破坏未知目录/活动包，准确报告占用/可恢复位置及自有残留 | T-03/06；不杀用户进程 |

语言如改中立生成，完整并集`cargo t auto-generation-core web-delivery-controls generated-delivery-controls`及必要真实UI consumer；仅Man薄CLI则跑其受影响CLI/dispatch命名测试。最终独立review后冻结实际触面完整职责/必要消费者并集，复用有效同输入证据；不把全仓/ACE/AGE、全Web发行gate重复列成每阶段门。现有Web SDK installer不改时只消费有效收据与关联负控，若实质修改则必须补受影响SDK安装交易真路径。prepare/编译/发现/运行/清理与条件排除、父/子真实终态分账。

实施者work自查，T-09由独立上下文沿同一权威树逐AC/SD核；本轮不指定模型、不创建代理或新聊天，自查不冒独立pass。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 |
|---|---|---|
| AC-01 | work | 正式公开export-only入口/帮助与Web库实现交付，普通项目生成非空真实服务包；V-01/03成立 |
| AC-02 | work | 包源/SDK/资源/锁闭包自足，原开发根不可达后fresh/locked与移根TCP业务通过；V-01/02/04/12，不靠预编译旧产物 |
| AC-03 | work | 坏源/错SDK/非法资源/事务失败写前或受控拒绝，last-good/活动/未知所有权保护与准确残留；V-03/05/12/14 |
| AC-04 | work | 合法资源与显式外部数据根正确，秘密/用户数据不入包，错误缺资源不假成功；V-04/06 |
| AC-05 | work | 普通Auto状态夹具、真实chat/database的HTTP、业务错误、持久化/独立读回与退出/端口释放成立；V-06/07 |
| AC-06 | work | C/full/覆盖与生成身份保持，Web/语言/Man依赖方向及原开发/UI受影响消费不回归；V-08/09/12 |
| AC-07 | work | RC-WEB-01 A/B真实包替换、失败保全、备份恢复及独立数据读回成立；V-05/10/11/14，明确首版同schema边界 |
| AC-08 | work | V-13交接表字段和可重建命令完整，P2实际实现/证据可供989审计，未预签989渠道/ready |
| AC-09 | review | 独立上下文逐AC-01～08/V/SD检查遗漏/延后/workaround、绑定revision/源/工具/结果，所有in-scope缺口关闭 |
| AC-10 | merge | 正式源/Spec/归档与被测身份一致，P2债务按真实交付关闭、989/路线状态同步，所有输出/持有/guard/清理有收据，无tag/部署 |

## 8. 执行步骤

| 完成 | ID | owner_stage | 依赖/进入门 | 文件/符号、动作及可观察结果 | 验收 |
|---|---|---|---|---|---|
| [✅] | T-01 | work | 用户以后启动；890/910/1003正式源可取回 | 冻结Web/语言/Man与必要产品源、现CLI/源选择/事务；最多两提交机制候选全链可行性分析，定新接口/fixture签名/显式data root，给差额与路径命令清单。未解决事务/数据语义不得往下实现 → 2026-11-11 完成：三仓HEAD与§4记录零漂移（lang f29f6facc/web 2d698413/man 347c29f；chat 5cdca0f9/db dee81e00）；裁定见§5.6（方案B′中立commit原语+正常生成后收割+bundle布局/闭包白名单+接口/数据根/fail-point）；漂移（web-install stage_source 布局校验 vs experimental语义合同闭包）已记录；事务/数据语义闭合，无未决阻断 | AC-01/03/04/07准备；V-03/05/14 |
| [✅] | T-02 | work | T-01 | Web api_gen/rust_workspace与语言source_owner/shared_generation必要小支撑；保语义根和C策略，生成独立manifest/SDK闭包/lock而非开发继承逃逸 → 2026-11-11 完成：语言侧=中立 commit 模块+local_build_closure（抽出走法，dev-dep排除；cargo test auto-generation-core --lib commit::/source_owner:: 8+9 绿）；Web侧=delivery 模块闭包枚举（实测23包：18 crates+auto-web+semantics-contract/autodown-test-parser vendor+test-cases数据树——比§4表多两层，全链构建证实）；sdk投影（members替换+workspace.package钉版+producer Cargo.lock身份）；bundle根manifest锚 sdk/crates/*+C/full块；bundle内 generate-lockfile；闭包实测含 auto-lang/test/cases 数据树（build.rs身份输入）；lang f0dd4189/web 待哈希提交 | AC-01/02/06；V-01/03/08 |
| [✅] | T-03 | work | T-02 | Web新delivery模块，最小资源/receipt/owned staging/提交与失败保全；现入口开发行为不变，不复制Man算法 → 2026-11-11 完成；2026-11-12 复审 F-1006-R1-01 重开后修复闭合：消毒 generated_at/created_at_ms，新增重复导出字节稳定测试（6/6 绿），driver policy 面断言重复导出==bundle-a manifest。原记录：export_rust_service 全链（严格合同前置→正常生成→staging→中性事务提交）；fail-point=prepare/verify/swap 旧包逐字节保全+staging清理+具名残留（commit单测+member_delivery fail-point 测试绿）；未知非空根拒绝且不动；资源=显式声明（缺失/越界/reparse命名错误，合法逐字拷贝）；manifest web-member-delivery/1 无时间戳保字节稳定；生成收据消毒（build_context置空/member.json project_path剥离）+全bundle dev锚扫描 | AC-02/03/04/06；V-03～05/12/14 |
| [✅] | T-04 | work | T-03 | Man cli_spec/main新增薄service-export分派；Web新增member_delivery.rs/driver/A/B夹具，独立移根构建/HTTP/状态/错误与反控可实际执行 → 2026-11-12 完成：Man ServiceExport变体+cmd_service_export薄臂（Core/full共用；解析测试4/4绿，auto.exe实跑）；member_delivery.rs 5/5绿（移根fresh/locked构建+缺锁反例+HTTP状态+重启持久+独立工件）；外部driver经正式CLI全链（含混根fail-closed实测：无env钉扎时plan-artifacts内项目被拒，具名错误） | AC-01～05；V-01～06/12 |
| [✅] | T-05 | work | T-04 | 原http_generation/runtime/db_import_context及真实chat/database导出消费；C/full与原开发/必要UI consumer按触面验证，真实进程/数据读回 → 2026-11-12 完成：Web原测试回归5/5（http_generation存量初红=legacy成员名vs PLAN-910 id8名，与本计划无关，已修；http_runtime 2、db_import 1）；driver真实chat（CHAT_DATA_DIR+CHAT_TEST_MODE：注册甲/乙→会话→消息→历史+独立sqlite读回 md-1 行）与真实database（make_fixtures种子→open/tables=4+/table_data 20/10/query→write-guard rejected）全过；C/full：默认c带profile块、full仅标记、manifest记静态label；UI consumer：语言侧改动仅新增中立模块+走法抽出（成员命名/生成路径零改动，git diff 证），UI侧无需另跑（930档由generated-delivery-controls组覆盖） | AC-02/04/05/06；V-01/02/06～09/12 |
| [✅] | T-06 | work | T-05 | RC-WEB-01 A/B/故障/备份/目录切换恢复、独立读回与残留负控，给实际源/包hash及命令摘要 → 2026-11-12 完成（driver 30/30 PASS，receipt sha256 a4993d961d64ba7e3f8a1030dfe125fbc131bc493da6b29bc2c489852c1f691b）：A写7→备份数据→B切换读A数据×2=14→swap fail-point重导出（旧包2442文件逐字节不变+数据hash不变）→恢复A+数据备份→独立sqlite读回=7；同schema（counter表id=1 TEXT value）；A/B包manifest sha256 de20744c…/c8ccb35d… | AC-03/05/07；V-05/06/10/11/14 |
| [✅] | T-07 | work | T-06 | 为989准备公开入口/正式来源候选、资源/C/schema/支持边界/恢复包对，修文档与提议SD，不勾989任务 → 2026-11-12 完成：SD-01新增 auto-web/docs/specs/generated-service-member-delivery.md；SD-02 delivery.md 区分SDK交付/成员导出；SD-03 lang lifecycle §7b 中立原语owner；989交接表见§8后「989交接包」小节（实际SHA/哈希已填） | AC-08；V-13 |
| [✅] | T-08 | work | T-07 | 受影响完整职责/必要消费者并集、负控及work自查；初红/环境/排除/覆盖边界分别记，准备execution_done交独立review → 2026-11-12 完成：冻结并集=语言 cargo t auto-generation-core/web-delivery-controls/generated-delivery-controls 3组全PASS + Web member_delivery 5/5 + 原回归5/5 + driver 30/30 + Man 解析测试4/4；初红记录：http_generation存量成员名缺陷（已修，非本计划引入）、aliyun镜像缺const-oid 0.10.3（环境归因，driver离线优先+在线回退绕开）、driver pins-only env loader失败（自缺陷已修）；V-08 full档/canonical由driver覆盖，V-13由本表+交接包承载 | AC-01～08；V-01～14 |
| [✅] | T-09 | review | T-08，work owned齐 | 独立上下文同树核AC/V/SD/源身份与遗漏，绑定revision；所需定向修复回work后重核，不另建同步树 | AC-09 |
| [✅] | T-10 | merge | T-09 pass且最终冻结触面门齐 | 各仓正式落地/读回/Spec、中央归档；真实P2关闭和989依赖/路线指针，独立包身份可取回 | AC-10落地 |
| [✅] | T-11 | merge | T-10 | 命令/真实数字/失败/hash折Plan，核全部实际输出/持有、guard clean后删自有scope与树/分支，刷新登记并读回收尾 | AC-10清理 |


### 989交接包（T-07，2026-11-12 实际值）

- 公开入口：`auto service-export --dir <project> --delivery-root <bundle> [--resource <rel>]...`（export-only；帮助文本已含）；库面 `auto_web::delivery::export_rust_service`。
- 本计划 revision r1；review/merge 记录见 §9（本行之后追加）。
- 正式来源：auto-lang `7296df7d848c628a00e65feec8e8799a6311471c`、auto-web `c3d126cdd5d0ce62565d2346b6ef116b596ec5ea`、auto-man `8d1976d722caff7d4daf4a93b0b79c91a2cbd6fa`（冻结 HEAD，driver 复跑于其上；含 F-1006-R1 修复）（分支 codex/plan-1006-{lang,web,man}，merge 后以主检出落地SHA为准）；真实产品 auto-chat `5cdca0f9e33883c7a72c3c9afbeb88234300c918`、auto-database `dee81e00f7ba28e045c274cbae6c9f2140635933`。
- Spec：SD-01 auto-web/docs/specs/generated-service-member-delivery.md（新）、SD-02 delivery.md（成员导出节）、SD-03 lang generated-cargo-build-lifecycle.md §7b（中立原语）。
- 支持面/边界：Windows Rust 工具链；显式数据根（fixture=MEMBER_DELIVERY_DATA 必填，chat=CHAT_DATA_DIR 整目录，database=请求内路径）；资源=显式声明项目内普通文件；闭包白名单（crates/* + experimental/ace/crates/semantics-contract + vendor/autodown-test-parser + crates/auto-lang/test/cases 数据树）；禁 UI/Man/门面/tauri；非目标=无网络冷缓存、跨平台安装器、历史跨schema迁移。
- C/full：bundle 根 manifest 静态策略块（c 默认 / AUTO_GENERATED_DEBUG_POLICY=full 省略），delivery manifest 只记静态 label。
- A/B 可重建源：crates/auto-web/tests/fixtures/member-delivery/{a,b}（同 counter schema：单表 id=1 TEXT value；A marker=100/B=200）；driver 命令 `python scripts/generated-member-delivery.py --binary <auto> --scratch <dir> --lang-root <lang> --web-root <web> [--chat-root …] [--database-root …]`（修复后复跑 30/30 PASS，receipt sha256 3f72415b0e846d11e12c1f2f36c76c233c3f0e122289dcc427dde677993c23eb；policy 面含重复导出==bundle-a 稳定断言）。
- 关键工件 hash（本机跑，receipt 全量见 driver-run/member-delivery-receipt.json sha256 3f72415b0e846d11e12c1f2f36c76c233c3f0e122289dcc427dde677993c23eb，修复后复跑）：A包manifest 3f29f92c60274fe5c334a937ca5ab2077dd019730c5e1a4f0c35e4296a693001、B包 05a4d69620d8dfdbb13db42c42ff42b461cbdfcbe7a1a5561560489c4eda7242（消毒时间戳后）；chat.db 2d4c04fc…（53248B）、seed basic_a.db 7cad5cdd…。
- 989 注意：989 E 表消费以上身份/断言即可复用；不预签989渠道/ready。

8 work + 1 review + 2 merge。work 全部完成（2026-11-12 execution_done）。历史进度：T-01～03 完成（见各行）；T-04 Man 薄分派+解析测试4/4绿（service_export_command_shape/delivery_root_is_required 等）；T-05 Web 原测试回归 5/5 绿（http_generation 修复 PLAN-910 成员名后全绿：初红=存量缺陷——测试用 legacy back_member_name 而生成物为 <name>-<id8>-back，与本计划改动无关，属计划内回归面修复；http_runtime 2、db_import_context 1 均绿）。可以先执行本计划，不等989/OS062/ACE M2；未来989 T-01准备可与之并行，T-02必须等待实际P2交付及其余E门齐。当前无实施授权。

## 9. 复审记录

2026-11-11，stage:new，PLAN-1006 r1，outcome:pass（起草合同结构可交后续work，非实施授权/独立复审通过）；next:用户以后启动work/T-01。本次只创建计划/恢复清单和文档同步，不声称独立设计复审、测试通过或P2关闭。



规划静态自检（2026-11-11）：139条当前本地链接有效，8个恢复旧正式Git对象均可解析；1006/989各11章，T/AC/V/SD分别11/10/14/3与9/8/18/1，任务未勾选、drafting0进度；硬依赖闭包88份唯一且无环，路线跟踪23份（20归档、3草案）。6份关键canonical/生产源码hash未变，号源1007。语义SHA256=`c3b95102bb8a670e89a7ced46669fe288f849a4ac5c3188943c4f2eddabd913f`（UTF-8/LF，§0～8及§10，排除§9）。这是文档结构/来源自检，非V执行、独立设计复审或ready PASS。

2026-11-12，stage:work，PLAN-1006 r1，outcome:pass（execution_done）。
- worktree：D:/autostack/.wt/lang-1006/{auto-lang,auto-web,auto-man}（分支 codex/plan-1006-{lang,web,man}；auto-down/auto-ui 为组内只读依赖树 detached）。
- 基面→代码：lang f29f6facc→57ac35394（commit 425e5e985+57ac35394）；web 2d698413b→7c85d820（356e1cc+7c85d82）；man 347c29f7d→2725b731（eebfbbb+2725b73）。
- 依赖版本：chat 5cdca0f9 / database dee81e00（只读消费）；auto-down 58ea33f / auto-ui 69104765（组内依赖钉）。
- 任务：T-01..T-08 全 [✅]（证据见各行）；AC-01..08 work 面全闭。
- 验证（冻结并集，2026-11-12）：cargo t auto-generation-core PASS；cargo t web-delivery-controls generated-delivery-controls 全 PASS；member_delivery 5/5（161.7s）；http_generation/http_runtime/db_import_context 5/5（226.3s）；Man cli_service 4/4；driver 30/30 PASS（receipt sha256 a4993d961d64ba7e3f8a1030dfe125fbc131bc493da6b29bc2c489852c1f691b）。
- 初红/环境归因：http_generation 存量成员名缺陷（已修）；aliyun 镜像 const-oid 0.10.3 缺失（环境，driver 离线优先+在线回退）；driver pins-only env loader 失败（自缺陷已修）。
- 统一回归门：pending，owner=最终review/merge（按 AGENTS 冻结触面后跑明确固定组并集——本节所列即冻结并集，merge 复核）。
- blockers：无。next：独立 review（T-09，独立上下文同树逐AC/V/SD核，绑定 revision r1/57ac35394/7c85d820/2725b731）。

stage: goal | plan_id: PLAN-1006 | plan_revision: r1 | round: 1 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1006 | commit: lang 57ac35394/web 7c85d820/man 2725b731 | next: review

2026-11-12，stage:review，review_scope:final，reviewer:ZCode 子代理（独立上下文，general-purpose，无实现记忆；局限=未重跑>5min构建，断言级核验+收据交叉核验+捆绑源码hash对账替代），PLAN-1006 r1，outcome:needs_fix。
- 基线：lang 57ac35394/web 7c85d82/man 2725b731（三树clean，diff 无夹带；man Cargo.lock 刷新=accesskit 全档构建带入，合法；http_generation 修复=存量成员名缺陷，合法计划内）。
- AC 结果：AC-01/03/04/05/07/08 pass；AC-02/06 partial（V-12 未执行且字节稳定声明为假）。
- 发现（stable ID）：
  - F-1006-R1-01 major：generation.json `generated_at`（api_gen.rs:833）与 member.json `created_at_ms`（generated_build.rs:742）随包携带→同输入两次导出 manifest 不同（de20744c… vs 089b9e95…），违 §5.1/T-03/SD-01 字节稳定声明；V-12 无任何重复导出断言。修复=消毒两时间戳+重复导出稳定测试+driver 复跑+§8/SD-01 更新。
  - F-1006-R1-02 minor：.source-revision 溯源回退（delivery.rs:196-207）未在 SD-01 记载。
  - F-1006-R1-03 minor：§5.6 白名单文本（1 vendor 树）与实现（VENDOR_TREES 2 + DATA_TREES 1）漂移未修。
  - F-1006-R1-04 minor：新代码非 rustfmt-clean（delivery.rs 9 块/member_delivery 6/commit.rs 9/cli_spec 1）。
  - F-1006-R1-05 note：driver 捆绑 manifest sources 记父提交（钉扎在最终提交前）；已 hash 对账捆绑 delivery.rs==HEAD blob，证据链健全；R1-01 修复重跑后自然消除。
  - F-1006-R1-06 note：driver failed-export-preserves detail 串仍写 counter.txt（断言本身用 counter.db，正确）。
  - F-1006-R1-07 note：phase_a_state 残留无区分度 assert（/counter/get?token=）。
  - F-1006-R1-08 minor：V-04/06/14 负面：data-root-missing HTTP 分支未跑（fixture 已支持）；活动服务/文件占用与秘密非入包按构造安全但无断言钉。
- SD 判定：SD-01 需修（R1-01/R1-02）；SD-02 pass；SD-03 pass（括号链接为标签非相对链接）。frontmatter 与实际 spec 路径一致。
- 覆盖边界：未重跑>5min 构建/全套件（断言级核验+收据sha对账+捆绑源hash对账）；chat/database HEAD 未验（只读消费）；wt-guard/清理=merge 职责。
- next：work 定向修复（R1-01/02/03/04/06/07/08）→ 受影响重核。

2026-11-12，stage:work，PLAN-1006 r1，outcome:pass（repair cycle 1/3：复审 needs_fix 定向修复）。
- 修复：F-1006-R1-01（delivery.rs 消毒 generated_at/created_at_ms + repeated_export_of_unchanged_inputs_is_byte_stable 测试 + driver policy 面稳定断言）；R1-02（SD-01 记 .source-revision 回退与修正后的身份/稳定合同）；R1-03（§5.6 白名单三Entry定稿）；R1-04（rustfmt 触面文件，范围限于本计划 diff 文件）；R1-06/07（driver 细节串/残句）；R1-08 部分（data-root-missing HTTP 负控入 oracle 测试；活动服务/文件占用与秘密非入包=构造安全，记为接受边界，见 R1 复审边界）。
- 修复验证：member_delivery 6/6（271.7s，含新稳定测试与负控）；driver 修复后复跑 30/30 PASS（receipt sha256 3f72415b…，冻结 HEAD lang 7296df7d/web c3d126c/man 8d1976d）。
- 未重跑（复用有效证据）：语言 3 固定组、Web 原回归 5/5、Man 解析测试——修复未触其输入（diff=消毒字段/测试/driver/spec/fmt；fmt 仅触面文件格式）。
- next：review 受影响重核（F-1006-R1-01/02/03/04/06/07/08 闭合 + AC-02/06 升 pass 核验）。

2026-11-12，stage:review，review_scope:final（repair cycle 1 受影响重核），reviewer:ZCode 子代理（独立上下文，同一Reviewer延续R1基线；局限=断言级核验+工件hash重算，未重跑>5min构建），PLAN-1006 r1，outcome:pass。
- 审核提交：lang 7296df7d848c628a00e65feec8e8799a6311471c / web c3d126cdd5d0ce62565d2346b6ef116b596ec5ea / man 8d1976d722caff7d4daf4a93b0b79c91a2cbd6fa（基线 f29f6facc/2d698413b/347c29f7d）；三树 clean，修复提交仅触 R1 范围文件，无范围蔓延。
- R1 闭合：F-1006-R1-01（消毒 generated_at/created_at_ms+重复导出稳定测试+driver跨拷贝稳定断言，工件证实 A==policy-C manifest 字节相同；MemberRecord null 容忍无下游破坏）、R1-02（SD-01 回退已记）、R1-03（§5.6 白名单一致）、R1-04（7触面文件0 fmt diff；style提交逐hunk=纯格式）、R1-05（manifest sources=冻结HEAD，自解）、R1-06/07（已修）、R1-08（data-root-missing HTTP负控已钉；活动服务/文件占用=fail-point swap 同路径已钉+Windows占用夹具固有flaky，接受边界；秘密非入包=构造安全+文件映射/锚扫描守卫，接受边界）。
- AC 结果：AC-01..AC-08 全 pass（AC-02/06 由 partial 升 pass，V-12 真闭合）。
- 新发现：F-1006-R2-01 minor（§8 关键工件hash串仍引修复前 receipt a4993d96…，merge 簿记期修正，无需再验）；R2-02 note（同秒理论窗，driver 面确定性覆盖）；R2-03 note（fmt 一处既有双空行，零语义）。
- SD 终判：SD-01 修后 pass；SD-02 pass；SD-03 pass；frontmatter 一致。
- pending merge 门：冻结触面固定组并集已由 work 期跑绿并经复审核验（语言3组+web member_delivery 6/6+原回归5/5+driver 30/30 receipt 3f72415b…+man解析4/4）；merge 复核证据绑定并执行 F-1006-R2-01 修正、989/路线状态同步、清理收据。
- next：merge（T-10/T-11）。

2026-11-12，stage:merge，PLAN-1006 r1，outcome:pass（T-10/T-11 收尾）。
- prepared：复审 pass 绑 r1 + 冻结 HEAD（lang 7296df7d/web c3d126c/man 8d1976d）；canonical=SD-01 auto-web/docs/specs/generated-service-member-delivery.md（master blob 1f949d47…）、SD-02 delivery.md（289ddc21…）、SD-03 generated-cargo-build-lifecycle.md（95637674…）；F-1006-R2-01 过期哈希串已修（§8）。
- merge 门（冻结 HEAD 实跑）：cargo t auto-generation-core 首跑 FAIL=plan988 既有集成测试在20路并发下超时（隔离重放 PASS 114.7s，整组复跑 PASS——环境归因，非代码）；web-delivery-controls/generated-delivery-controls PASS；man cli_service 4/4；web member_delivery 6/6+原回归5/5+driver 30/30 已绑 c3d126cd（§9 前record）。
- landed：三仓均 --ff-only（无 merge commit）：auto-lang master=7296df7d848c628a00e65feec8e8799a6311471c、auto-web master=c3d126cdd5d0ce62565d2346b6ef116b596ec5ea、auto-man master=8d1976d722cff7d4daf4a93b0b79c91a2cbd6fa。man 主检出既有 Cargo.lock WIP 与分支提交字节相同（e0cb4139），丢弃后由合并原样带回（零损失，已记录）。
- canonical_verified：三 spec 路径/hashes 如上，master 落地核验。
- 债务/路线：D-942-P2 已关（KNOWN-DEBT-AND-RISKS.md 附关闭记录）；989 交接包=归档计划 §8。
- archived：docs/plans/archive/1006-web-generated-member-independent-delivery-and-recovery.md，status: archived，随本提交入库。
- cleaned：见下一条 T-11 收尾回执（worktree/分支/组目录/plan-artifacts scope 删除+登记刷新）。
- 落地后观察项：主检出 auto-lang `auto`/release 二进制与 web 产物未随本合并重建（代码合并后自然陈旧；本计划交付为库/CLI源码与合同，无运行中的生产进程依赖其重启——记录为观察，不阻塞）。
- 批量回归到期检查：1006%5=1 非整除；.last-batch-regression.json 状态见 T-11 回执。

2026-11-12，stage:merge（T-11 清理收尾），PLAN-1006 r1，outcome:pass。
- guard：五树（lang/web/man/down/ui）wt-guard 全 clean 后移除；分支 codex/plan-1006-{lang,web,man} 已删（7296df7d/c3d126c/8d1976d 均已落地 master）；组目录 .wt/lang-1006 已删，无残留。
- plan-artifacts/plan-1006/ 整scope已删（driver-run 收据/哈希/日志全部折入本计划 §8/§9；构建中间产物随 worktree 死亡，未回收）。登记表 .retention/policy.json 已刷新（delete-at-merge，无具名持有）。
- 输出路径对账：本计划全部命令输出=worktree target（随树删除）、plan-artifacts/plan-1006（已删）、driver scratch（同scope）；无 D:/autostack-build/其他盘符/散落第二证据目录；主检出仅有会话前既有 docs WIP 与空 run.txt（非本计划产物，未动）。
- 批量回归到期检查：receipt last_covered_plan_id=995（2026-11-08）；到期动因=PLAN-1000（11-11落地，%5=0）+>48h有合并——先于本计划即到期，非本计划引入；按 AGENTS「不成为每计划全局门」记录交接 auto-plan:regress（主检出单实例执行），不在本 merge 内跑。
- 终态：8 work + 1 review + 2 merge 全 [✅]；AC-01..10 全闭；P2 债务关闭；归档完成。

stage: goal | plan_id: PLAN-1006 | plan_revision: r1 | round: 3 | from: merge | to: done | status: archived | outcome: pass | worktree: (removed, group lang-1006 clean) | commit: lang 7296df7d/web c3d126cd/man 8d1976d + docs 67abfc377 | next: done

## 10. 待澄清事项

没有需要本轮用户补充才能立约的产品决定。T-01负责有界核定事务复用/新命令等价命名与真实Auto数据API签名；不允许以探索结果减掉旧包保全、真实状态/移根或产品任务。若实际机制不能满足，记录阻断并修订后再进入依赖任务。

具体导出器交付SHA、A/B工件hash/锁与候选工具只在真实实施/产出后填写，属于必需执行证据，不是当前已冻结/已验收。989最终候选必须核这些实际值。用户只授权立约，下一阶段需另行启动work；不重做1003/068。
