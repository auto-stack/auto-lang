# language-sdk（auto-lang-core）

> **Status**: active（M814-A 发布 @ master f6e07acee；PLAN-814 r3 合同）
> 路径：`crates/auto-lang-core` | 技术栈：Rust
>
> PLAN-814 交付（A 阶段）：原 auto-lang 巨型包中的**非 UI 语言栈单源**——
> scanner/parser/type/compile/VM/trans 核心——独立为纯语言 SDK。normal/
> build/dev 闭包零 UI 文法实现、UI compiler/runtime/renderer 与 Man；
> 清单（含 optional path）零 auto-ui-dialect，无 UI checkout 的源码快照
> 可独立消费（fresh+locked，独立消费者 5/5）。

## 职责与边界

- **单源面**：前端门面组（ast/token/error/session/lexer 门面、symbols/
  scope）、类型系统/infer、parser+dialect、compile/解释器、VM（engine/
  codegen/native/ffi）、trans（rust/c/gdscript/tscn...）、libs/http 服务、
  stdlib_assembly、design_tokens（parser 通道）、a2r_std、use_scanner/
  dep_scanner/mode/os_paths。仍支持的上层 `auto_lang::*` 正式路径保持同源类型；四个 `ast/parser/session/error` 过渡模块于 0.2 退役，消费者按迁移规格切换 owner。
- **禁止边**：不依赖 auto-ui-dialect/auto-ui-compiler/auto-ui-runtime/
  auto-lang（旧包）/auto-man；不提交 UI 注册项、不硬编码 UI factory。
  777 域=lang-core；build 输入=具名清单（SDK 编译闭包 17 crate +
  vendor/autodown-test-parser + 根锁；UI/Man/旧包/生成器零条目，
  双负控测试钉住；受管投影缺目录即跳过——"存在即扫"）。
- **真实消费者**：auto-vm 直接依赖本 SDK（闭包零旧包/UI）；语言侧
  fixture `test/fixtures/modularity/language-sdk-consumer/`（Core
  parse/compile/VM 真实执行、手工字节码管线、trans_rust 产物、UI 文法
  显式拒绝负控）。

## 中立领域端口（PLAN-841）

- `Parser::parse()` 只产 Core `Code`，不含 UI 兼容臂；排他 UI 声明未经
  显式组合时确定性拒绝。Core `.view` 借用、普通 `msg`、SceneDecl 与
  on/task 语义保持。
- `Parser::parse_domain(&D)` 经 auto-parser-api 的中立
  `DomainParseContext` 返回 `DomainCode<D::Output>`。UI 由上层显式传
  `UiDialect`，产物归 UI owner；SDK 不知道 `UiStmt` 或 UI codec。
- 旧 `UiGrammar`/`set_ui_grammar`/`UiGrammarSource`/Parser UI 返回门面
  已删除。`DomainParseContext for ParserBridge` 只提供通用游标、scope、
  Core expr/type/stmt 委托与快照能力，无 UI 字段或全局 side-table。
- `design_tokens` 等跨域名称查询经中立 `set_external_ident_checker`
  末位咨询，不把 UI 注册表或 AST 类型反灌进 SDK。

## 生成器指纹与 UI 缓存身份（PLAN-698 分包精确化）

- `set_generator_fingerprint(Option<String>)`：None→Some=**升级**（晚装配
  不阻断真实身份）；已装配=首装优先。
- `UICache::save` 按缓存对象**出生身份**落盘（不重盖当前身份）；
  `load` 经 cached-vs-current 比对判失效——**一侧缺席=整缓存失效**
  （None/None 除外）。真实身份消费方（auto-man vue/jet、compiler
  ui_gen::api）在 load 前注入。

## 特性

`with-file-history`（default）、`code-editor`（a2r diff 双轨门）、
`python`、`streaming-http`、`mem-profile`、`test-cases`。UI 特性族
（ui-iced/ui/autodown 等）留上层兼容包转发。

---

## PLAN-814 B 增量（T-06；M814-B 发布落库）

- `use_diag`/`take_use_diags` 下沉 SDK 单源（与 `STORE_EXTRA_FILES`
  同族 thread_local 带外通道；auto-lang 旧路径再导出）。
- `test_code`/`test_code_with_deadline_and_input`/
  `test_code_with_path_deadline_and_input`/`test_file` 执行族下沉
  （纯 Core 面；宿主装配经本 crate 钩子分发器——无 UI 实现时 no-op）。
- 非 UI 消费者基线升至四枚：auto-vm（A 期）+ auto-gen/
  auto-playground/a2r-actor-tests（B 期；闭包零 UI 零旧包）。

## Dependency scan lexical recognition (PLAN-840)

use_scanner and dep_scanner share code-line lexical state in dependency_scan.rs. Inert comments, ordinary/multiline/triple/backtick strings and escaped text do not become use/dep declarations during Session compilation or fresh/cached module scans. Actual dependency forms, ecosystem names, arguments and ordered deduplication remain supported. This scanner correction changes no Parser/VM syntax or module capability; strict single-file user behavior still rejects imports. The four-file canonical SDK patch is5e2f15806; standalone artifact932ad487 fixes the dependency input without adding UI/Man/old-package edges. Scanner38, actual fresh/cache/module controls and full frozen gates bind the correction.

Provenance: [PLAN-840](../../plans/archive/840-autographics-public-behavior-api-and-user-scripts.md), [affected final review](../../plans/attachments/840/review-r2/review.md), [complete merge receipt](../../plans/attachments/840/merge-r1/receipt.md). Graphics source: autographics-dev. Production I2/I3 remain pending.

## PLAN-840 Phase 2：Case 测试可选边界

`test-cases` 仍为显式可选 feature；Case/AV/CTE/probe 测试注册必须与其
`auto-test-case` 可选依赖、Core `test_cases` 再导出使用同一 feature 门控。
合法 feature-off 的测试构建不得引用该依赖；feature-on 必须保留并执行全部
既有 CTE 正例、错误退出、未知 recipe、伪 worker 与身份漂移反例。
本修复不增加默认依赖、不改变既有测试档的 feature 选择。


## PLAN-840 Phase 4：状态文件锁的 Windows 瞬态竞争

`state_file::with_lock` 保留独占创建锁文件、2s 缺省截止时间、10s stale 判定和
best-effort 降级执行合同。Windows 延迟删除的文件名在最后一个句柄关闭前，
`create_new` 可以返回 ACCESS_DENIED(5)；sharing/lock violation(32/33)及
DELETE_PENDING(303)同为需在调用方既定截止时间内重试的竞争。错误码5也可能
表示真实权限拒绝，不能无界等待；截止仍到期后按原合同降级并执行闭包一次。
只有 AlreadyExists 分支可检查 stale 并尝试接管，其他错误不能据此删除锁。
非竞争性硬错误仍直接降级，诊断保留最近系统错误。未改变锁文件路径/内容或
跨仓约定，不宣称超时降级状态下仍提供互斥保证。

## PLAN-831 SDK 独立源（SD-06，as-built）

纯 SDK 缺 UI 检出仍可 parse/compile/VM 执行：normal/build/dev 闭包不拉 UI 组合（隔离 clone 反控实测）。旧 auto-lang UI 面为上层再导出（指向独立 UI 仓真实 owner），不混入纯 SDK 承诺。

## PLAN-832 Man 固定消费（SD-06，as-built）

Man 的 runtime、CLI、three-repository-consumer 均从语言仓固定 source 消费 SDK；语言仓不因 Man 迁移反向依赖 Man 实现。独立 Core 安装与第三方工程真实 parse/compile/VM 通过，类型和 source 唯一；旧 Man/CLI 原件已从语言仓删除。

## PLAN-841 来源与验证

纯语言 SDK 保持无 UI 实现反向依赖；UI 上层经显式领域端口与受检 lowering 消费。 规范与验收来源：[PLAN-841](../../plans/archive/841-ui-domain-ast-token-lowering-finalization.md)；复审及统一门证据见其归档记录。

## PLAN-842：宿主装配与兼容边界

纯 Core 不链接 UI 时 SDK hook 缺席或 no-op。已链接 UI 语言包但直接调用 SDK 的既有消费者仍依赖现有单份 `UiCompatInstaller` inventory 提供默认宿主能力；显式 provider 优先，默认、缺席、多实例、线程与退出语义保持。该宿主登记不同于已退出的 UI 文法 inventory；删除它会丢失实际默认能力。生成应用显式安装 SDK hook 与 UI runtime provider。来源：[PLAN-842](../../plans/archive/842-legacy-compat-and-distribution-retirement.md)
