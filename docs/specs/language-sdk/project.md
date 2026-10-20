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
  dep_scanner/mode/os_paths。旧 `auto_lang::*` 路径经上层兼容包再导出
  保类型身份（消费者零改动）。
- **禁止边**：不依赖 auto-ui-dialect/auto-ui-compiler/auto-ui-runtime/
  auto-lang（旧包）/auto-man；不提交 UI 注册项、不硬编码 UI factory。
  777 域=lang-core；build 输入=具名清单（SDK 编译闭包 17 crate +
  vendor/autodown-test-parser + 根锁；UI/Man/旧包/生成器零条目，
  双负控测试钉住；受管投影缺目录即跳过——"存在即扫"）。
- **真实消费者**：auto-vm 直接依赖本 SDK（闭包零旧包/UI）；语言侧
  fixture `test/fixtures/modularity/language-sdk-consumer/`（Core
  parse/compile/VM 真实执行、手工字节码管线、trans_rust 产物、UI 文法
  显式拒绝负控）。

## UI 文法装配端口（r3 合同 §5.3）

- `UiGrammar` 函数表（定义在 auto-parser-api）：11 项文法函数指针
  （widget/msg/model/component/actions/style/view×2/store/scene/门），
  内建 UiDialect 与 Parser 公开 UI 门面经表派发；**表缺席=原 feature off
  语义逐位一致**（Scenario::UI 方言表空、公开门面显式拒绝诊断
  "not assembled ... UI grammar table"、scene/store 门恒 false）。
- `set_ui_grammar(UiGrammar)`：显式装配（幂等，首装优先）。
- `UiGrammarSource`（inventory 收集点）：SDK 零提交；上层兼容包/
  compiler 经源码可见 submit 语句登记（启动构造器机制，只登记不执行
  UI 业务/I/O），仅覆盖不经装配入口的直达 `Parser::from(..)
  .with_session(UI)` 构造——r3 §5.3 具名例外。
- `UiParseContext for ParserBridge`：无条件编译（trait 定义在
  auto-parser-api；孤儿规则下 impl 落类型所属 crate）。

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
