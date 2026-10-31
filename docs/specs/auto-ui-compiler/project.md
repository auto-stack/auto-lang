# auto-ui-compiler

> **Status**: active（M814-A 发布 @ master f6e07acee；PLAN-814 r3 合同）
> 路径：`crates/auto-ui-compiler` | 技术栈：Rust
>
> PLAN-814 T-03 交付：UI **编译/生成域**独立成包——原 auto-lang 的 UI
> 分析（aura 提取/schema）、design token 校验、action_config、lucide
> 全量表、a2ui 互操作、route 分析、widget registry、blueprint（bp）与
> **vue/jet/ark/rust 四生成器** ≈11.3 万行单源迁入；旧路径
> （`ui::style`/`aura`/`a2ui`/`ui_gen` 等）经上层再导出全保持。

## 职责与边界

- 依赖纯语言 SDK（auto-lang-core）+ auto-ui-dialect（文法单源，本包
  直接清单边）；**不反依赖**旧 auto-lang 兼容包/auto-ui-runtime。
  777 域=ui。
- 生成器源指纹：本 crate `build.rs` 扫 `src/ui_gen`（Plan 692 W-1 失效
  语义单源），经 `generator_fingerprint()` 公开（None=foreign build），
  SDK UICache 经注入消费。
- 资产随 owner：`test/a2ark`、`test/a2vue`、语料 fixtures；交付闭包
  DELIVERY_CRATES 19 包含本包（SD-08，792 driver 25/25）。
- 外部消费门：`test/fixtures/modularity/ui-package-consumer/`（真实
  Vue SFC 生成出口+指纹+style+lucide，4/4；闭包=SDK+显式文法组合零
  旧包边）。

## UI owned AST 与显式领域组合（PLAN-841）

`parse_ui` 只经 `Parser::parse_domain(&UiDialect)` 取得原始 `UiCode`；
`parse_validated_ui` 与 `lower_ui` 共用 dialect 的 `validate_ui`；
`validate_ui_file_with_modules` 向工程工具提供文件及直接导入模块准入；
主文件和每个直接导入模块均先按各自目录执行
`prepare_style_recipe_imports`，再解析/lowering，冷态命名/通配符配方
不依赖前一文件注册残留。公开转译、Vue/Rust/Jet/Ark request 工程、 incremental、组件包和跨文件 模块在缓存、预扫描、aura 提取、生成或写盘前完成校验/lowering；准入错误 不得被 warn-and-continue 吞成成功或半壳工程，Rust 先编译再创建输出。 四生成器不再匹配 Core `Stmt` UI 兼容臂。旧 `assemble_ui_grammar` 空桩 与 `UiGrammarSource` 登记已删除。`ui`/`ui-headless` 空 feature 仅保留
原编译面开关。

准入与解析失败双轨（PLAN-842 r3 SD-08）：前置预校验（含工程侧
`validate_ui_tree` 整树扫描）的硬拒绝对象是**受信准入失败**——即本节
domain/module 校验错误（错误串 `UI domain validation failed` /
`UI module validation failed`）——任何模式、先于缓存/输出；语法解析
失败不属受信准入：普通（非严格）模式保持逐文件 warn-and-skip 统一
语义（坏文件跳过、正常文件照常产出），严格模式硬失败，fresh 与
incremental 一致。双轨不得互相吞并：预校验不得把非严格解析宽容变成
硬拒绝，逐文件处理也不得把准入失败降级为警告。

`UI_AST_SCHEMA_VERSION` 与生成器指纹共同组成缓存身份；旧 schema 缓存
失效并重建。compiler doctest 直接使用 `auto_ui_compiler` 当前 crate
路径，不再引用已迁出的 `auto_lang` 实现路径。

## feature 面

`ui`（UI 门面装配族）、`ui-headless`（headless 样式适配器）——均为
空门（实现依赖恒在），由旧 auto-lang feature 转发。

---

## PLAN-814 B 增量（T-05/T-08/R9-R14；M814-B 发布落库）

### canonical import 路径策略（单源 `ui_gen::import_policy`）

- canonical 规则：UI 面 `auto_ui_runtime::ui::*`、语言/VM 面
  `auto_lang_core::*`；新旧路径显式映射表在档（view/code_editor/
  scroll/style/auto_val/action_config/frame_bench/dialog/iced→
  runtime::ui；vm/a2r_std→core）。rust 生成器发射统一切换；vue/jet/
  ark 不发射 Rust import 不在射面；**a2r trans 泳道发射明确不在本
  策略内**（消费者仍依赖旧包，后续批次）。负控 pin：生成物含
  canonical 头且零 `auto_lang::` 残留。

### 生成器行为规则（预存缺陷修复，shell-pack 再生暴露）

1. **比较位值投影的完整表达式类型感知**（PLAN-710 D-6 扩展，
   R9-F01→R14 复修收口）：dot 访问处于值比较位（前导/后随六种比较符
   `==`/`!=`/`<`/`>`/`<=`/`>=`，含字段自身透明分组括号；调用实参
   括号如 `len( p.tags )` 非分组，实参位保持 truthy；比较符两方向
   完整消费——前向双字符 `<=`/`>=` 整体跳过）时不得 truthy 包裹；
   投影按另一操作数**完整表达式类型**分流——形状已知走 shaped 访问
   器（bool→as_bool/int→as_i64 as i32，循环形状语义保持）；形状
   未知按操作数区间解析：bool 字面量→as_bool/整数（含负号）→
   as_i64 as i32/浮点→as_f64/已声明状态引用（`.limit` 形）按
   state_types 声明环境分流（i32/f64/bool 与同名字面量域一致；
   非默认声明宽度 float=f32、u32/i64/u64 投影跟随声明宽度）/
   四则算式按宽度合并（float 支配：任一 f64→f64、仅 f32 声明→f32、
   纯字面量→f64 默认，与 Rust 无类型字面量推断同源；int 取更宽）/
   布尔取反 `!` 传播（剥 `!` 递归保 bool）/括号内 `&&`/`||` 逻辑
   组合（bool⊕bool→bool）/比较子表达式结果是 bool（六符全集、
   括号外、字符串字面量外——`"a<b"` 不误报）；任意一侧透明分组
   剥壳递归不改型、左右位对称。String/Value/未声明引用/非数值混型
   不命中，默认 str 语义保持（`p.running == .wanted` 不变）。
   反例矩阵（r9..r15 全集 112 组真实编译）见
   docs/plans/archive 计划附件 r9-r15 各 closeout。
2. **on_click 单元兜底**：点击族消息承载事件（onclick/
   oncontextmenu[.prevent]→on_right_click/ondoubleclick）在场时跳过
   `on_click (|_| ())` 兜底（E0308 反例：shell 语料按钮带
   oncontextmenu.prevent）。

### e2e 生成工程 manifest 合同

双直依赖（`auto-ui-runtime`[ui-iced]+`auto-lang-core`）真编译门
（2/2，#[ignore] 档 harness）。

### 并集归属增补（PLAN-814 T-11 merge，2026-10-21）

`app_registry`/`i18n_lookup` 单源定于本包（PLAN-815 T-04 先落，
auto-ui-tools 消费 `auto_ui_compiler::app_registry`；DAG 禁
compiler→runtime）；auto-ui-runtime 经 `pub use` 再导出，旧
`auto_lang::ui::{app_registry,i18n_lookup}` 路径全保持。

## AutoOS 产品路径与稳定注册 ID（OS PLAN-050）

`app_registry::scan_apps` 与公开的 `expand_apps_container` 按容器父级的
`apps.manifest` 中 active `repo`/`local` 条目的真实目录路径匹配稳定 ID；
例如 `apps/auto-notes` 仍注册 `015-notes`。路径比较先 canonicalize，支持
清单中的 `./` 等等价路径。ID 不跟随产品目录改名，pac 名称/展示名不改。
未匹配、无/坏清单时回退原来的目录名；非活跃/未知 kind/空 ID 不提供别名。
匹配只读取相邻清单，不用全局 AUTO_OS_ROOT 改写其他扫描根的身份。

Vue `auto-ui-tools::vue::desktop_extra_app_roots` 共用此容器展开，保持主根、
VM/iced extra 根和 manifest 聚合时去重；空子模块继续按无入口/pac 跳过。
存储里的快捷方式、图标映射、launch/daemon lookup 仍按原 ID，不迁移用户值。
`AppRegistryEntry::app_root()` 从发现入口还原物理应用根；Vue 桌面生成读取该根，
不再按稳定 ID 拼目录，生成后的路由/组件命名仍使用稳定 ID。

回归：`product_directory_names_*` 覆盖改名/路径匹配/local、无效/缺失清单、
空子模块、显式 extra 优先及聚合去重；Vue `extra_roots_three_track_parity`
覆盖正式目录 `auto-notes`→稳定 `015-notes` 且没有第二个新名入口。
物理迁移归 [AutoOS PLAN-050](../../../../auto-os/docs/plans/archive/050-app-directory-product-names.md)。

## PLAN-831 仓属与来源（SD-05，as-built）

本 crate 物理属 auto-ui 仓（迁移基线=语言仓 e199caf8a 内容；测试/资源/接口 owner 随迁）。schema/aura.at 自持（crate 相对 include）；a2gd 语料单源留语言仓，dialect 回归以兄弟路径读。blueprints 消费经 resolve_blueprints_root 三级解析（env → cwd 向上 → 编译期组内兄弟语言仓兜底；兜底须精确解析到实际存在且含真实包的单源——独立 clone 无兄弟布局不在生成支持矩阵）。跨仓 CI 保留原测试 ID/feature 选择/integration/单进程 ui_gen 档（tu 863 唯一 ID 恒等基线）。

## PLAN-841 来源与验证

四生成器消费 UI owned 模型；文件/模块准入按各自目录先初始化配方导入再校验，cold/负控均验证。 规范与验收来源：[PLAN-841](../../plans/archive/841-ui-domain-ast-token-lowering-finalization.md)；复审及统一门证据见其归档记录。

## PLAN-842：生成消费与版本边界

四类生成器、迁移示例和 Man 生产消费直接使用 UI runtime/compiler 及语言 SDK 的 canonical owner；生成的 Rust 应用不直接依赖旧 auto-lang 门面，入口显式安装 SDK hook 与 runtime provider。生成器指纹和 UI schema 共同决定缓存身份；旧身份失效后重建。原 design-token 测试及 recipe integration 测试跨 owner 映射并保留 full 测试选择。来源：[PLAN-842](../../plans/archive/842-legacy-compat-and-distribution-retirement.md)

## PLAN-876：desktop_exe 语义澄清

注册表对 `pac.at` 的 `desktop_exe:` 仍按"声明入册"（扫描/注册语义不变），但
"声明即信、产物可缺"不再延伸为运行时语义：运行时是否走原生由 auto-ui-runtime
按产物在场性与 apps.manifest `launch` 裁定，contract 见
[autos-desktop-entry](../repository-boundaries/autos-desktop-entry.md)。
来源：[PLAN-876](../../plans/876-autos-desktop-entry-and-launch-health.md)

## PLAN-890 Vue局部快照与同名状态

Vue store handler按真实词法local绑定发射：快照变量/字段保留原名与原始值，只有被遮蔽且实际使用的state才经保留内部module别名访问。块/循环/闭包/catch恢复外层绑定，同名初始化保留对state的访问；委托函数/lambda捕获保留local名，公开getter与普通输出不变。
原ui_gen单进程866PASS/2原ignore及四个新增反例、固定Blog Vue/native结果见[四仓合同](../repository-boundaries/web-four-repository-delivery.md)（plan-890）；无新AST/schema/runtime协议。
