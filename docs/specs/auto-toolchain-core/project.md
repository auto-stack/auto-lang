# auto-toolchain-core

> **Status**: active
> 路径：`crates/auto-toolchain-core`  | 技术栈：Rust（auto-val + std）

工具链配置模型、flag 映射与命令模板核心（PLAN-805 自 auto-man
`builder/ninja` 逐字抽出）：`CompilerKind`（MSVC/GCC/Clang/GHS/IAR/
Targeting/Hightec/TICLang 八类，含 `gnu`/`llvm` 别名与大小写不敏感
解析）、`ExecutableType`（cc/ar/link/as 四角色，**大小写敏感**解析）、
`FlagFormat`（Prefix/Postfix/Both 三型）、`FlagMappings` 与八类默认表、
`CompilerLocation`（Env/Dir/Executable）、`CompilerConfig`（八类默认
构造 + 扩展名/可执行文件访问器）、`FlagMapper`（通用标志→编译器特定
格式，含 `map_output` 的 `trim_end_matches` 尾缀语义与
`map_compile_only` 的 Both 拼接）、`CommandTemplates`（四命令内联字符
串模板 + 按变量名长度降序的占位符渲染，TICLang 复用 clang 模板）。
可被无 auto-lang/auto-man/auto-gen/auto-vm/UI/AST 闭包的外部 consumer
独立消费（外部消费方探针
`test/fixtures/modularity/toolchain-core-consumer`，闭包断言
`verify-deps.sh`；auto-val 是规则模型的合法数据层——AutoStr/AutoPath
同型直用，无转换副本）。

## 目标与范围

- 单一实现归属：上述全部类型/默认表/映射/模板只在**本 crate** 定义；
  auto-man 旧公开路径 `builder::ninja::{config,mapper,templates}` 是
  薄 re-export 门面（同一类型身份，可直接互传，类型同一性断言见
  `crates/auto-man/tests/plan805_type_identity.rs`）。
- 迁移与格式：自 base `282e50053` 迁入——迁移提交 `4d1aade5a` 与
  base 三文件 Git blob 逐字节相等（复审 R1 已核实）；R1 修复轮按仓规
  对迁入文件做 rustfmt（仅排版，无语义改动），此后证据面=16 件 golden
  行为对拍 + 定向测试，不再以"字节一致"作为当前修订的相等主张。
- 行为冻结：自 base `282e50053` 逐字迁移（字节一致）；旧臂 oracle 为
  `crates/auto-man/tests/fixtures/plan805/` 的 16 件 golden（八类默认
  dump、alias/未知 kind、三 FlagFormat、mapper 边界含重复尾缀/空格/
  Unicode、八类×四命令模板、render 语义含缺变量保留、八类+linux+
  自定义 flag/含空格工具路径的 NinjaBuilder memory 输出、CompilerStore
  am.at 三态），由 `crates/auto-man/tests/plan805_toolchain_parity.rs`
  在新臂逐字节断言。
- 命令模板是**内联字符串**（`templates.rs` 内 `*_templates()` 函数），
  与 PLAN-803 迁出的 `assets/templates` 资源（rust-embed 文件字节）是
  两个不相交的概念，不得混用或互设依赖。
- 不做：动态 `am.at` 加载与节点解码（CompilerStore 留 auto-man）、
  PATH/磁盘/进程观测与版本检测（CompilerResolver 留 auto-man）、
  Ninja 执行/AutoGen 编排/`quote_if_spaced`（NinjaBuilder 留
  auto-man）、Target/Pac/构建 DAG、新配置模型或宽泛 host trait。

## 依赖边界（红线）

- 依赖仅 auto-val（+std）；dev 侧零额外依赖（纯测试内联）。
- **禁止**依赖 auto-lang / auto-ast / auto-man / auto-gen / auto-vm /
  任何 renderer/UI/解释器/IO/进程栈。闭包由 toolchain-core-consumer
  fixture + module-boundaries 基线钉住（man-ecosystem 域，
  `man-ecosystem-no-lang-monolith` 禁向规则适用）。
- 不进入 PLAN-792 generated-ui 交付闭包（`DELIVERY_CRATES` 不含
  man 系；cargo metadata 反向依赖=仅 auto-man——PLAN-805 T-05 收据）。

## 冻结语义（PLAN-805 T-01 旧臂实读冻结；改动=行为变更须显式决策）

1. **`CompilerKind::from_str` 大小写不敏感**且接受 `gnu`→GCC、
   `llvm`→Clang 别名；**`ExecutableType::from_str` 大小写敏感**精确
   匹配 `cc|compiler`/`ar|archiver`/`link|linker`/`as|assembler`——
   两者口径不同是旧行为，不是疏漏。
2. `FlagMapper::map_output`：Prefix/Both 型用
   `output.trim_end_matches(obj_ext)` 的 base（**重复尾缀全剥**：
   `main.o.o` → `main`）；Postfix 型用原始 output 不剥。
3. `map_compile_only`：Prefix/Postfix 原样返回模板串；Both 型返回
   `prefix+postfix` 拼接（无分隔符）。
4. `CommandTemplates::render` 按 key 字节长度降序做**一轮顺序替换**：
   逐 key 对**当前累计结果**执行 `str::replace`（完整 `{key}` 占位符）。
   因此——(a) 后处理的（更短）key 会展开先前插入值中的占位符
   （`{longkey:"{x}", x:"done"}` 渲染 `{longkey}` 得 `done`）；
   (b) 已处理的 key **不回扫**（非不动点迭代：`{longkey:"done",
   x:"{longkey}"}` 渲染 `{x}` 得 `{longkey}` 定型）；(c) 未知占位符
   原样保留。等长 key 的处理序继承 HashMap 迭代（原未定序角落，
   如实保留；控制断言结果只落两种合法排序之内，见 consumer
   `external_consumer_pins_render_ordering_semantics`）。不加引号、
   不去重、不排序 flags。
5. `CommandTemplates::new` 按编译器 kind 派发模板表（**TICLang →
   clang 模板**）；命令模板不消费 `FlagMappings.output`（自定义
   output 格式只经 `FlagMapper::map_output` 生效）。
6. 批量 `build_*` 保输入次序；`format_flags` = join 单空格（空集合
   得空串；空 ldflags 在模板中产生双空格，不去重不清理）。
7. MSVC 独有默认：`/I{} /D{} {}.lib /LIBPATH:{}`、
   `Both("/Fo", ".obj")`、compile_only `/c`、obj 扩展 `.obj`、exe
   扩展 `.exe`；IAR 独有 compile_only `--debug` 与 12 条默认 cflags；
   TICLang 默认 `-Oz -Wall`；八类默认 executables/cflags 全表见
   config.rs（冻结于 golden `config_defaults.txt`）。

## 模块

| 模块 | 职责 | 状态 |
|---|---|---|
| config | `CompilerKind`/`ExecutableType`/`FlagFormat`/`FlagMappings`/`CompilerLocation`/`CompilerConfig` 与八类默认表、扩展名/可执行文件访问器 | active |
| mapper | `FlagMapper`：通用标志→编译器特定格式（单值/批量/format，`map_output` 尾缀语义） | active |
| templates | `CommandTemplates`：四命令内联字符串模板 + 占位符渲染（长度降序一轮顺序替换） | active |

## 验证（work 阶段实跑；merge 阶段冻结裸 `cargo t` + 定向门）

- `cargo test -p auto-toolchain-core` — 38 单测 + 6 doc 测试（随迁移的纯测试）。
- `cargo test -p auto-man --test plan805_toolchain_parity` — 16 golden 逐字节对拍 + 替换序直接控制（V02/03/04）。
- `cargo test -p auto-man --test plan805_type_identity` — 旧路径↔新 crate 类型同一性（V02）。
- `cargo test --manifest-path test/fixtures/modularity/toolchain-core-consumer/Cargo.toml` — 外部独立消费 + render 顺序/等长 key 角落控制（V01，5 测试）。
- `bash test/fixtures/modularity/toolchain-core-consumer/verify-deps.sh` — normal/build 闭包无语言栈（V01 oracle）。
- `python scripts/modularity-audit.py --mode strict` — 777 边界基线（域登记后重绑）。
