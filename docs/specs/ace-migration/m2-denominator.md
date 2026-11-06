# ace-migration: M2 替换分母台账（m2 denominator ledger）

> Status: frozen inventory（PLAN-959 SD-01，2026-11-07 冻结）。本文件冻结 M2 替换的
> **行集、来源身份、四类语言分账与后继责任**；只登记与归属，**绝不声明实现**——实现状态
> 以各模块 Specs 与已交付计划收据为准，任何登记不替代 REP/V06 门的实际执行。
> 口径与五类状态（IV/IW/ES/CM/UK）、证据边界、义务迁移/中立库/disable 证据归属见
> [replacement-scope-and-evidence.md](replacement-scope-and-evidence.md) SD-01/SD-02。
> 种子：[922 盘点报告](../../plans/reports/ace-m2-replacement-inventory-2026-11-02.md)
> （A–G 68 行审计快照）；本台账是该快照的**现行分母账**：A–G 行在此刷新/引用，L/S/K 为
> 959 新增逐项行。冻结后新增/变更按 §10 delta 规程追加，不重写已冻结行。

## 1. 冻结基线（来源身份）

读取时间 2026-11-07；跨仓只读，WIP 一律不计入交付身份。语言仓基准 = 本 spec 所在
worktree 基线 commit `92bbf298b1f785ad6075914e83ed3eedaa215644`（master；首轮读取于
141a57ccd，期间 auto-lang/auto-ui/auto-os 三仓前移，受影响行按 §10 第 2 条只重核受影响锚，
全部原位解析——见 §4 漂移登记）。

| repo | branch@SHA | wip | 备注 |
|---|---|---|---|
| auto-lang | master@92bbf298b | 61 项 docs/** WIP + 未跟踪计划草案 | 他方会话文档 WIP，只具名；141a57ccd→92bbf298b 含 8 个 crates 文件变更（expr_calls/expr_emit/ops_calls/http_async/http_client/strings_json/host_bridge/back_prefix），L-01/L-25/S-16 锚复验原位 |
| auto-ui | master@b4094acd8 | 15 项 | c5c6fe05e→38f9f1f86→0b3ee62d3→cbeb65c50→b4094acd8 四轮前移（末两轮纯 docs 簿记）（060/068 落地+runtime 修复+068 归档簿记）；C 表五锚+driver Cargo:11 复验原位（§4）。该仓他方会话仍在合并，后续前移按 §10 delta 只重核受影响锚 |
| auto-man | master@2a9b16e50 | clean | |
| auto-web | master@2d698413b | clean | |
| auto-os | main@07646ddd7 | 16 项（docs/子模块指针） | a2290f1b0→07646ddd7 落地 054（host/desktop 代码+8 个 apps gitlink 前移）；apps.manifest 仍 28 id 同集；host main.rs:46-49 锚复验原位 |
| auto-down | master@58ea33f69 | clean | jade-garden 在内 |
| auto-musk | main@721ecfa07 | 1（pycache） | |
| auto-gen | master@593b4eecd | clean | 对新 main 不可编译（F-06 保持 ES） |
| auto-edit | main@b19029736 | 2 | |
| auto-shell | main@46bf48975 | 3 | ash/ash-gui 两形态 |
| auto-vscode / auto-launcher / auto-coder(1246 项生成物 wip) / auto-code-rs / auto-ai / book / ag-examples / ag-platformer-2d / auto-zed / auto-skills / jade-edit / readest | 见 §6/§7 | — | 逐仓分账见 §6 |
| 20 个 auto-* 应用仓（calc…weather）+ auto-os-config | 见 §6 OS064 表 | — | OS064 A01–A28 载体 |
| 非引擎目录 hello/mybook/sandbox/tools/docs/scratch-consumer | — | — | 零引擎引用（§6 零引用账） |

关键文件身份（本台账行证据锚，开工复核以符号为准）：`crates/auto-ast/src/ast.rs`
（Stmt:194/Expr:309）；`crates/auto-lang-core/src/vm/codegen/*`（stmt_emit/expr_*）；
`experimental/ace/crates/core/src/source/ast.rs`（ExprKind:264）与
`source/mod.rs`（九 profile）；`crates/auto-lang/test/cases/ace-m1/binding.tsv`
（131 数据行）；`stdlib/contracts/operations.toml`（17 操作）。

## 2. 行口径与四类语言分账

- **行身份** = 前缀 + 序号，冻结后不复用：`A/B/C/D/E/F/G` 沿用 922（公共入口/A2X 目标/
  AutoUI/AutoMan/AutoWeb/其余仓/退役分类）；`L` = 语言语法逐项（959 新增）；`S` = 标准库
  域（959 新增）；`K` = 发行/安装/冻结副本（细化 922 G-6）。
- **四类语言分账**（每 L 行必落其一，禁止用百分比混合）：
  1. `规范已定` —— 规范/合同明文定义（spec path 或 binding.tsv 行）；
  2. `真实支持` —— legacy 管线 parser + codegen 双证（file:line）；
  3. `已知旧bug/差异` —— legacy 执行与冻结语义不符（binding.tsv `diff-*`/`accepts` 家族、
     KNOWN-DEBT 条目、源码 unimplemented/TODO 锚）；
  4. `未决` —— 规范冲突/平台条件类待裁定（必须有 owner 与关闭任务，登记 §9）。
  另有 `无此形` —— 语言不存在该构造（防凭空新增，不是缺陷）。
- **"新AST在场"列只表示 ACE source AST 有无对应节点，不构成运行支持**；"AST 枚举存在"
  在 legacy 侧同样不等于支持（parser-only 行即此意）。
- 状态五类（IV/IW/ES/CM/UK）沿 SD-01；本文行内"分类"列给出产品接线视角的状态，
  与四类分账正交。
- 每行证据 = repo@rev + file:line（或 spec path §节）；922 报告中的原始 file:line 证据
  继续有效，本文只登记漂移（§4/§6）。

## 3. L 表：语言语法分母（旧 AST 逐项行级归属）

来源：`crates/auto-ast/src/ast.rs`@141a57ccd 全枚举（Stmt 30 变体 / Expr ~60 变体 /
Type ~45 变体 / Op 39 / 辅助枚举 20+），legacy 双证 = `auto-lang-core/src/parser/**`
+ `auto-lang-core/src/vm/codegen/**`。分族给出公共 consumer/oracle；行内：
**规范**=四类分账，**legacy**=真实支持程度，**新AST**=ACE source AST 在场，
**后继**=行级归属（Plan/批次）。

### L-A 数值与字面量族（oracle：binding.tsv B01–B20/W01–W05；consumer：全产品执行/转译）

| 行 | 项（AST 锚） | 规范 | legacy | 新AST | 后继 |
|---|---|---|---|---|---|
| L-01 | i32/uint/i8/u8/i64/u64 字面量与后缀（Expr Int/Uint/I8/U8/I64/U64 ast.rs:311-316；parser expressions.rs:1494-1630；codegen expr_emit.rs:803-854） | 规范已定（core/scalar specs） | 真实支持；**已知差异**：legacy int=i64（binding B13/B15 `diff-old-int-i64`） | 部分（IntLit i32 ast.rs:267） | 961 §0/T-08/AC-06；stdlib/JSON/TS 腿 972/975/976 |
| L-02 | float/double 字面量（:318-319；W01–W05） | 规范已定（binding W 行） | 真实支持；**已知差异**：avm/a2r `no-typed-entry` 5 格 | 无 | 961 §0/T-08（NaN/-0/payload 政策） |
| L-03 | byte/char/CStr 字面量（:317/321/323；Byte 无 parser 构造=AST-only 死臂） | 规范已定 | char/CStr 真实支持；byte 字面量 **parser-only（AST 死臂）** | 部分（CharLit/StrLit） | 961 §0；byte 域 961 T-01 重核 |
| L-04 | i128/f32 后缀、进制扩展 | 无此形 | —（token.rs 无此后缀） | 无 | 无（不得凭空新增；如需归 961 T-01 裁定转未决） |
| L-05 | 位运算/移位/符号扩展（Op value.rs:450-489） | 规范已定 | 真实支持（标量子集见 binding B03/B04） | 部分（BinOp 12 种 ast.rs:198-212，无位运算） | 961 §0/T-08 |

### L-B 借用/所有权/智能指针族（oracle：binding T/R/K 行）

| 行 | 项 | 规范 | legacy | 新AST | 后继 |
|---|---|---|---|---|---|
| L-06 | `.view/.mut/.move`（Expr View/Mut/Move :329-331；ParamMode/AccessMode fun.rs:195-238） | 规范已定 | 真实支持；**已知差异**：T06 `diff-legacy-accepts`（借外逃 legacy 收） | 部分（MutArg/MovePos ast.rs:328/255） | 962 T-08（Hold/view/mut/move 义务） |
| L-07 | Hold（:333；codegen expr_async.rs:257-264） | 规范已定 | 真实支持（MVP：无借用纪律，源码自注） | 无 | 962 T-08 |
| L-08 | Box(v)/Arc(v)（:389-390；parser expressions.rs:1907-1918） | 规范已定 | **parser-only**：VM 无臂→expr_fallback.rs:433 unimplemented | 无 | Box 递归表示→962 T-08；Arc/共享/弱引用→964 T-08 |
| L-09 | Take（:332，Move 弃用别名） | 规范已定 | 真实支持（随 Move） | 无 | 962 T-08（随 L-06） |

### L-C 组合类型/模式族（oracle：binding R/L/V/D 行）

| 行 | 项 | 规范 | legacy | 新AST | 后继 |
|---|---|---|---|---|---|
| L-10 | type 声明/字段/方法/mut 写回（TypeDecl types.rs:690；binding R01/R03/R04/R07） | 规范已定 | 真实支持；**已知差异**：R01/R03/R04/R07 `diff-legacy-value/runtime` 4+ 格 | 部分（TypeItem ast.rs:155） | 962 §0；ACE 已交付域内 IV（record/text spec） |
| L-11 | enum 三形态（EnumKind scalar/homogeneous/heterogeneous enums.rs:26） | 规范已定 | 真实支持（stmt_emit.rs:1642） | 部分（Item::Enum :169，单 payload 受限） | 962 §0（variant 扩面） |
| L-12 | union（ast.rs:205；parser decls.rs:2495） | 规范已定（AST/规范存在） | **parser-only**：VM stmt_emit.rs:3589 `_` 静默丢弃 | 无 | 962 T-08 |
| L-13 | tag（:206；tag→enum 别名路径 statements.rs:662；Stmt::Tag 死臂） | 规范已定 | **parser-only**（Stmt::Tag 无 VM 臂） | 无 | 962 T-08 |
| L-14 | grid（:354；parser scenario.rs:974；keyword 已降级 token.rs:412 Plan-408） | 规范已定（AutoData 域） | **parser-only**：VM expr_fallback.rs:433 panic | 无 | 962 T-08 |
| L-15 | tuple（:422；codegen expr_containers.rs:57）+ TupleDestruct（:379） | 规范已定 | 真实支持 | 无 | 962 §0（元组/解构） |
| L-16 | array/List literal（:344）/map=object literal（:347）/set literal | 规范已定（array/object）；set **无此形**（无 AST 变体/parser） | array/object 真实支持 | 部分（Generic List/Result :72，无字面量语法） | 962 §0；object 动态字典面→964 §0 |
| L-17 | Option/Result 构造与模式（:358-361/384-387；binding V 行） | 规范已定 | 真实支持；**已知差异**：V03 `diff-legacy-runtime` | 部分（Optional/Generic Result + IsMatch） | 962 §0（组合域已交付部分 IV） |
| L-18 | is 语句/表达式 + Cover/StructPattern（:355/424；stmt_emit.rs:2856-3078） | 规范已定 | 真实支持；SEM-02：legacy 负索引保留（L06 差异登记，不得冒绿） | 部分（IsMatch :441 语句位） | 模式/解构→962 §0；is-表达式→965 §0 |
| L-19 | range `..`/`..=`（Range :343） | 规范已定 | 真实支持 | 无 | 962 §0（范围循环） |

### L-D 动态/闭包族（oracle：binding D/K 行）

| 行 | 项 | 规范 | legacy | 新AST | 后继 |
|---|---|---|---|---|---|
| L-20 | object 动态成员/反射（:347；binding D01–D09） | 规范已定 | 真实支持（动态值域）；**已知差异**：avm/a2r `no-dyn-api` 9 格（av.dynamic/1 已 IV） | 部分（CtorNamed 为名义构造） | 964 §0/T-08（Dyn/反射/循环回收） |
| L-21 | nil/null/`??`（:366-367/369；PLAN-053 统一） | 规范已定 | 真实支持 | 部分（Optional 无 nil 字面量） | 964 §0；`??`→965 §0 |
| L-22 | closure `x => e`（:352；expr_control.rs:69；binding K01–K06） | 规范已定 | 真实支持；**已知差异**：K02/K05 `diff-legacy-grammar(parse)`（avm 与 a2r 同） | 有（Closure :348，865 已 IV 域内） | 964 §0；native 边界 924 已裁定 |
| L-23 | lambda（Expr::Lambda :351；parser 从不构造） | 规范已定（AST 在场） | **AST-only 桩**：codegen expr_fallback.rs:394-400 发 `CONST_I32 0` 空占位——**已知旧bug 登记** | 无（Closure 取代） | 964 §0（函数值/命名函数项为值一并裁定） |
| L-24 | 默认参数（Param.default fun.rs:256；parser decls.rs:1599-1604） | 规范已定（AST 在场） | **parser-only**：VM 按全元数调用，无默认填充 | 无（Param :98 无 default） | 963 T-08（默认/命名/可变实参） |
| L-25 | 命名实参（Arg::Name/Pair call.rs:61-64；codegen expr_calls.rs:2946-2975） | 规范已定 | 真实支持；**已知差异**：B11 `diff-legacy-value`；print 族关键字形态编译拒（PLAN-746 expr_calls.rs:2268） | 有（Arg::Named :256） | 963 T-08；TS 名实参序 976 V-02 |

### L-E 控制与效果族（oracle：binding C/G/I/H 行）

| 行 | 项 | 规范 | legacy | 新AST | 后继 |
|---|---|---|---|---|---|
| L-26 | if/else、for（Iter 6 形 for_.rs:16）、while/loop/break/continue/return | 规范已定 | 真实支持；**已知边界**：非 range/array 索引 for 不支持（stmt_emit.rs:2476）、continue 个别循环变体（:2846） | 部分（If/While/Loop/ForIn/Return/Break/Continue :416-450） | 965 §0；for-in 已 IV（collections 域 864） |
| L-27 | try/catch/finally（try_.rs:15；stmt_emit.rs:116） | 规范已定 | 真实支持 | 无 | 965 §0/T-08（catch 权限政策继承） |
| L-28 | `.?` 错误传播（:370；expr_control.rs:115） | 规范已定 | 真实支持 | 无 | 965 §0 |
| L-29 | async 块 `~{}` / `.await` / `.go`（:399/405/411；expr_async.rs:15/143/204） | 规范已定 | 真实支持 | 无 | 968 §0（967 交完成端口） |
| L-30 | yield（:417；expr_async.rs:222） | 规范已定 | 真实支持 | 无 | 965 §0（生成器）；调度/流所有权 968 T-08 |
| L-31 | reply（ask/reply :220；stmt_emit.rs:1420） | 规范已定 | 真实支持 | 无 | 968 §0（ask/reply） |

### L-F 编译期/宏/元数据族（oracle：936 CT 合同 + binding E/H 行）

| 行 | 项 | 规范 | legacy | 新AST | 后继 |
|---|---|---|---|---|---|
| L-32 | `#if/#for/#is/#{}` 语句级（comptime.rs:31-253；CTEE transformer.rs:91-203） | 规范已定（hash-syntax spec） | 真实支持 | 部分（HashExpr :343，936 已交付单表达式 CT） | 966 §0（Hash 各形态完整域，Design55 §5 归属在此行级登记） |
| L-33 | `#{}` 表达式级编译期替换 | 规范已定（设计） | **已知缺口**：legacy 编译为普通运行期表达式（expr_async.rs:248-254 TODO 自注）；ACE 侧 936 交付 `ac.source.cte.scalar/v1` | 有 | 966 §0（沿 936/958 worker 前门） |
| L-34 | 宏 `#name()`（MacroCall comptime.rs:290；stmt_emit.rs:3547） | 规范已定 | 真实支持（文本展开级） | 无 | 966 §0（hygiene/预算/循环） |
| L-35 | metadata/`#[attr]`（c/vm/rs/pub/api/export/test；decls.rs:625） | 规范已定 | 真实支持（元数据路由；vm stub stmt_emit.rs:189-215） | 无 | 966 §0（metadata/生成声明）；`#[test]`→971 §0；`#[api]` 路由→980/982 |
| L-36 | 平台条件（platform conditions） | **未决**（无独立语法规范；`#if` 谓词/构建 cfg 混用） | legacy 无专用形（978 的 cfg/feature 层另计） | 无 | **行级归属**：语言侧 966 T-01 冻结谓词边界 + 构建侧 978 T-08（profile/feature/cfg 混装）；裁定前不得宣称支持 |

### L-G 声明/模块/别名族

| 行 | 项 | 规范 | legacy | 新AST | 后继 |
|---|---|---|---|---|---|
| L-37 | use（Auto/C/Rust/Py 四形 use_.rs:7-15）/dep（dep_.rs:21；statements.rs:1166-1458） | 规范已定 | 真实支持（stmt_emit.rs:3543/3578/3804-3824） | 无（模块外置，950 有限模块已交付 `ac.source.modules.scalar/v1`） | 960 §0（完整模块包）；C/Rust/Py 桥语义→967 §0 |
| L-38 | alias / type alias（alias.rs:6 / type_alias.rs:8；parser 768/decls.rs:1925） | 规范已定 | **parser-only**：VM `_` 臂静默忽略（stmt_emit.rs:3589） | 无 | 963 §0（alias）+ 960（重导出/别名解析） |
| L-39 | spec（trait 声明 spec.rs:45；stmt_emit.rs:1750-1755 仅元数据） | 规范已定 | 真实支持（元数据级；源码自注"不生成字节码"） | 无 | 963 §0（spec/约束/关联类型） |
| L-40 | ext/impl（ext.rs:48；stmt_emit.rs:1585 方法独立编译） | 规范已定 | 真实支持 | 部分（TypeItem 内方法 :158） | 963 §0（ext/方法接收者/优先级） |
| L-41 | 全局 store：let/var/const/shared/CVar（store.rs:6-16；statements.rs:2701-2816） | 规范已定 | 真实支持；**无 `global` 关键字**（token.rs:369-451 无；shared 为对应物） | 部分（Let :396 单形态） | 960 T-08（全局 store/常量初始化次序/once/session/diamond/卸载）；局部 store 语义随各族 |
| L-42 | 泛型：type/const 参数、GenericInstance、turbofish（types.rs:385-443；call.rs:14-19） | 规范已定（935 已交付有限单参模板 `ac.source.instances.scalar/v1`） | 真实支持（模板实例化老管线；935 SQ-B 域内新侧 IV） | 部分（TyVar :63） | 963 §0（通用泛型/跨模块实例） |
| L-43 | TaskDef / on => 事件（task.rs:45；on.rs:26；stmt_emit.rs:3275） | 规范已定 | 真实支持（任务元数据+handler） | 无 | **行级归属**：968 §0（Task/actor/ask/reply；计划文本未逐名 TaskDef，本行补齐归属） |
| L-44 | Node 语句/表达式（node.rs:5；expr_containers.rs:110/stmt_emit.rs:3526）与 OnEvents/SceneDecl（VM 无臂丢弃） | 规范已定（领域） | Node 真实支持；OnEvents/SceneDecl **parser-only** | 无 | 领域节点→979 §0（Schema/领域 IR）；SceneDecl→977/SCENE；UI 方言→980/DIALECT |
| L-45 | NavCall 路由调用（:392；VM 无臂） | 规范已定 | **parser-only** | 无 | 980 §0（routes 批次） |
| L-46 | spread / set literal / `when`/`match` 关键字 / `global` 关键字 | **无此形**（token.rs 369-451 无；匹配由 `is` 承担 L-18） | — | — | 无（防凭空新增；如产品需要→按 delta 规程立未决行） |

### L-H 调用/索引/管道族

| 行 | 项 | 规范 | legacy | 新AST | 后继 |
|---|---|---|---|---|---|
| L-47 | 调用语义：动态调用/方法分派/管道 `|>`（parse 期脱糖为方法调用 expressions.rs:546-575） | 规范已定 | 真实支持；**已知边界**：动态调用名 unimplemented（expr_calls.rs:3392） | 部分（Call/MethodCall :288/322） | 963 T-08（调用语义/函数值签名/返回推导/运算符） |
| L-48 | 索引/place 赋值/复合赋值（expr_ops.rs:247-324/543；:86 捕获变量复合赋值不支持） | 规范已定 | 真实支持；**已知边界**：复杂 LHS unimplemented（:543）、捕获复合赋值（:86） | 部分（AssignTarget local+field path :380） | 962 §0（索引 place 扩面） |
| L-49 | cast `as` / 转换 `to`（:371-377；expr_control.rs:123/155） | 规范已定 | 真实支持 | 部分（TypeWordArg :287，text 域） | 961 §0（cast/to 完整数值转换） |

**L 表合计 49 行**，覆盖旧 AST 全部具名语法族与全部"parser-only/AST-only/无此形"项；
已知旧 bug/差异共 14 处行内登记（L-01/02/03/06/17/18/20/22/23/25/26/33/47/48），
未决 1 行（L-36）。legacy 侧 parser-only/AST-only 合计 11 行（L-03 byte/L-08/L-12/L-13/
L-14/L-23/L-24/L-38/L-44 部分/L-45 + L-04 无此形），**不得把 AST 枚举在场当运行支持**。

## 4. A/B 表刷新：公共入口与 A2X 目标

922 A-01..A-12 / B-01..B-11 行身份、file:line 证据与分类继续有效（复验 @141a57ccd /
auto-man@2a9b16e50），仅登记漂移与后继刷新：

- **A 表刷新**（后继按路线 §4 换正式号）：A-01/02/03 执行/config 求值族→969（+982/CONFIG）；
  A-04 调试族→971；A-05 转译族→976/977/982 各目标批次；A-06 REPL/daemon/MCP→970；
  A-07 `#[test]`/shim→971；A-08 仓内二进制→969/987（playground→971；auto-case→971）；
  A-09 引擎选择分派→969/987；A-10 `pub use vm` 整包直连→967（REP-06 核心）+980/983；
  A-11 AutoInterpreter→982/TEMPLATE（现仅 auto-man asset.rs:3,43 一处生产消费，已核）；
  A-12 a2r_std→中立保留（SD-02）。facade pub fn 逐项（24 直定义 + 50 re-export 符号 +
  entry/script_entry 全表）已在 work 证据件登记（plan-artifacts/plan-959/agents/），
  无新增入口；`old-repl` 命令实测启动 AutoVM REPL（auto-man main.rs:770）而非
  TreeWalker，A-06 证据修正。
- **B 表刷新**：B-01 Rust→969 域外接线在 980/RUST、982/RUST、983/GENERATED；B-02 C→
  982/C；B-05 TS→976（UI 生产依赖先行）；B-03/04/06/07→977/PY、977/GD、977/SCENE；
  B-08 r2a/B-09 s2s→977/REVERSE；B-10 escape 未接线保持退役候选→987；B-11 相邻发射器→
  972-975 发射 + 987 退役。921 已 archived（CA-A 交付），CA-B/C 已由 925/926 archived，
  CA-D=937 archived，CA-E→985。
- **漂移登记**（922 锚复验）：auto-ui vm_bridge 移至
  `crates/auto-ui-runtime/src/ui/vm_bridge.rs:43-47`（符号同行）；tool_ops 分派行
  `:334→:353`；其余 15 处锚原位解析（tauri_backend.rs:131-134、ts_adapter.rs:28、
  cmd_a2c_stdlib.rs:63-87、automan.rs:132/169、cmd_test.rs:106、asset.rs:43-45、
  main.rs:769/867/873/917、host.rs:8-130、server_dispatch.rs:856-859、api_gen.rs:30/34、
  os host main.rs:46-49、term.rs:191、vm_server.rs:41、vm_backend.rs:63、auto-gen
  lib.rs:90-93、driver lib.rs:1-4/36/86、book harness.rs:27）。

## 5. S 表：标准库域分母

来源：`stdlib/`（188 .at：auto 122/c 3/collections 1/aura 62 + iter 树）+ 原生绑定
（`for_each_native!` 427 目录项 native_catalog.rs:11、BigVM 859 项 :860、`#[vm]` 声明
366/25 文件 native_registry.rs:383-401、手工域注册 ~280）+ `a2r_std` 19 模块 +
ACE 合同（`stdlib/contracts/operations.toml` 17 操作 + compatibility.toml 17 政策，
943 冻结；944 装配已 archived）。oracle：943 语义实册 + 各域 spec；consumer：全产品。

| 行 | 域 | .at 面（fns） | 原生面 | a2r_std | 现状 | 后继 |
|---|---|---|---|---|---|---|
| S-01 | math/int | math.at 10+math.vm.at 8 | math 21+int 19 | math | ACE 17 操作合同含 abs/min/max（私有平方/立方切片） | 945（切片）→972（全集） |
| S-02 | text/str/char/fmt | str.at 20/dstr 10/char 7/fmt 10/format 4 | string 12/str 5/stringbuilder 8 | str/string_builder | byte_len/equal/clone 四端切片=945；text 全集→972；FStr 格式算法→972（求值→964 L-22 行） | 945→972 |
| S-03 | bytes | （file/str 字节函数承载） | auto.str.bytes 等 | fs 字节族 | 未单列 | 972 |
| S-04 | collections：list/hashmap/hashset/btreemap/vecdeque/sort/iter | 12+11+6+10+10+5+21+适配器树 | natives_collections 90；目录 list 28/hashmap 13/… | list/hashmap（hashset/btreemap/vecdeque 缺=漂移登记） | 946 集合切片；全集→972 | 946→972 |
| S-05 | parse/format/encoding | conv 2/fmt/format/base64 2/csv 4/hex 2 | natives_formats 21 | （conv/log/path routed-but-missing 登记） | 972 | 972 |
| S-06 | hash | hash.at 7+vm 7 | hash 7/sha2_opaque 4 | hash（file_sha256 956） | 972 | 972 |
| S-07 | json | json.at 20+vm 19+rs 19 | json 值族+http helpers | json | 948 JSON 消费者探针；全集→975（宽整数/大数） | 948→975 |
| S-08 | fs/path/file | fs.at 16/file 11+14+14/path 5 | natives_fs 24/file_io 43 | fs（File） | 947 File/Task/Stream 合同前探；全集→973 | 947→973 |
| S-09 | env | env.at 5+vm 3+rs 4 | sys_env 1682 行 | env | 973 | 973 |
| S-10 | process | process.at 5+vm 6 | spawn 2 | process | 973（进程树/管道/信号 973 T-08） | 973 |
| S-11 | time/datetime/duration | 4+12+6（+rs/vm） | natives_chrono 16；**已知 bug**：D-956-2 嵌套多参 helper 错位（open） | time | 973；旧 bug 作迁移 oracle 输入不冲销 | 973 |
| S-12 | rand/random | 2+7 | natives_rand 14 | rand（907） | 973 | 973 |
| S-13 | zip | 3+vm 3 | zip_io（897） | zip | 973 T-08 | 973 |
| S-14 | console/io/log | io.at 10+vm 8+c 7/log 12+vm 4 | natives_io 11/print 族 15/console 3 | （io rust 不支持=providers 登记） | 947 console 切片（AV 已 863 IV；native/a2r 轴 45 格未闭） | 947→973（console 四端） |
| S-15 | storage | 3+vm 5+rs 4+c 4 | storage/heap 4 | — | 973 | 973 |
| S-16 | net/http/sse/url | net 14+vm 14/http 79+vm 68/sse 1+4/url 22+vm 23 | http_client 7 个 register_* 子函数（:3109-3469）合计 145 regs+server 族+ws 8 | http（client）/sse（parse） | 948 TCP/HTTP 消费者链；974 全库（transport 独立真实证据）；产品腿 983 | 948→974→983 |
| S-17 | sqlite | 5+vm 5+rs 5 | sqlite 9 | sqlite | 975（事务/兼容升级 975 T-08） | 975 |
| S-18 | redis | 7+rs 7 | **无 VM 原生**（rust-only） | redis | 975 | 975 |
| S-19 | task/async/sched(timer) | async 11+vm 10/sched 2+vm 2 | task_run 1/Task 族 | task（mapping 未验=登记） | 947 Task/Stream 前探；968 调度；973/974 接资源 | 947→968→973/974 |
| S-20 | media(image) | image 17+vm 17+rs 17 | image 17（auto-media-core 中立） | — | **领域 owner**：媒体库不入 972-975；产品交付随 980/OS064 追踪（Design55 §6） | 980/OS064 追踪 |
| S-21 | terminal | term 23+vm 23 | term_engine 34 shim（AutoTerm 桥） | — | 领域 owner：984/SHELL+OS064 | 984/OS064 |
| S-22 | clipboard/dialog/dnd/i18n/frame | **无 .at**（仅 natives） | natives_host 41（clipboard 6/dialog 3/dnd 1）+i18n hook | — | 宿主能力：967（natives 先迁）+980（UI 宿主面） | 967→980 |
| S-23 | ui.at/widget/aura（62 文件 215 声明） | ui.at 15/widget 2/aura 树 | UI natives 30+（runtime） | — | 领域：980/DIALECT+NATIVE；schema→979 | 979/980 |
| S-24 | test.at/test_mod | 10+2 | natives_test 12+shim 4 | — | 971（#[test] 执行接管） | 971 |
| S-25 | c 面（stdio.c.at 44/stdlib.c.at 14/collections） | — | — | — | C 目标装配（emission 旧臂） | 982/C+987 |

**S 表合计 25 行**。948 负责把每条未迁域映射到 972–975 的 AC（其 T-08 明文绑定 959 行）；
redis/clipboard 等"仅 natives 面"行不得因无 .at 而漏记。

## 6. C–F 表刷新：产品/消费者分母

922 C-01..C-11 / D-01..D-09 / E-01..E-06 / F-01..F-12 行身份与分类继续有效（§4 漂移表）。
本节登记刷新与新增行：

- **C（auto-ui）→ 980 九批次**：C-01 vm/Iced→NATIVE；C-02 vue 双档→WEB（vm 档）/RUST；
  C-03 tauri→TAURI；C-04 rust 静态→RUST；C-05 jet/ark→JET/ARK；C-06 vscode 模式→
  971（编辑器客户端）+986（发行）；C-07 VM 内部直连面→967+980/STATE；C-08 冻结 SDK→
  K 表/K-02+981/SOURCE+987；C-09 aura→980/DIALECT；C-10 测试→985+980/PARITY；
  C-11 安装→986。
- **D（auto-man 全命令账，@2a9b16e50 复验）→ 982 十一批次**：new/init/build/gen/ui/bp/
  wizard/add/fetch/deps/device/export/info/open/env/clean → CONFIG；run/service→EXEC；
  test/test:ui→EXEC+971；watch→DEVSDK；trans/c/rust/r2a/python/javascript/gdscript/
  a2c-stdlib→RUST/C/REVERSE/PY/WEBTARGET/GODOT 各批；old-repl/parse/eval/serve/req/
  mcp/debug/cffi→970/971/967（A 表归口）；rqhost（main.rs:275/291 desktop RQ 守护）→
  980/NATIVE+OS064/HOST；stdlib 命令→973/975；docs→971（schema 文档）。CLI 隐藏/弃用
  命令（block/old-repl/parse/eval/config）全部在账，退役→987。
- **E（auto-web）→ 983 四批次**：PORT（http_server_handle/嵌入面，UK-02 消费者）；AVSERVICE
  （serve/dispatch/parked owner 线程模型）；GENERATED（api_gen 生成 Rust 轨）；CONSUMERS
  （四仓真实 TCP 闭环：Man/UI/Down/jade-garden）。
- **F 刷新**：F-01/02→OS064/HOST+A04；F-03 jade-garden→OS064/A03+983/CONSUMERS；
  F-04 musk→OS064/A02；F-05 auto-edit→984/EDIT；F-06 auto-gen→982/TEMPLATE（复活/归档
  用户裁定门保持 UK-10）；F-07 vscode→971+986；F-08 launcher→OS064/A08；F-09 coder/ai→
  984/CODER、AI、CODERS；F-10 book→984（书稿 harness；**11 .at 含 `#[test]`**，971 消费）；
  F-11 ag-examples→981（外部 `--engine` 二进制消费，engine-lock.json 钉分支；非旧 API 直连）；
  F-12 安装脚本→K 表+986。
- **F 新增行**：
  - F-13 auto-shell（ash-server `backend_abi` backend.rs:5、`autovm_persistent` REPL
    session shell.rs:10、host.rs:3 set_host、a2r-parity 夹具）→ 984/SHELL；分类 CM。
  - F-14 20 个 auto-* 应用仓 + jade-edit + auto-os-config：CLI/生成物级消费（`auto run
    -r vm`/`--server vm`/rust-workspace 生成 manifest），Rust API 引用仅生成物
    （`*.a2r.rs` use a2r_std）→ OS064 A01–A28 + 984/CLOSURE；**破损钉**登记：auto-clock
    `.wt/os-055` 悬空、auto-man/examples 与 auto-notes/examples rust-workspace 相对路径
    断、auto-todo `.wt/os-054` 在途钉 → OS064/ASSEMBLY T-01 重核。
  - F-15 零引擎引用账（不得列为迁移义务，也不得从中删去任何真实消费者）：
    ag-platformer-2d/auto-zed/auto-forge/readest/hello/mybook/sandbox/tools/docs =
    零引用；auto-vscode/auto-skills/ag-examples = 文档/外部二进制级；scratch-consumer =
    生成骨架（pac.at/app.at/state.at）。auto-forge-legacy 仅 `"spawn_task_plan"` 字面量。

### OS064 28 App 承接表（apps.manifest@auto-os a2290f1b0，全 active）

kanban/A01、auto-musk/A02、jade-garden/A03、auto-term/A04、037-klondike/A05、036-tetris/A06、
038-minesweeper/A07、028-launcher/A08、039-syslog/A09、jade-edit/A10、011-calculator/A11、
012-clock/A12、013-todo/A13、014-weather/A14、015-notes/A15、016-calendar/A16、017-chat/A17、
018-book-reader/A18、023-realworld/A19、030-video/A20、020-music/A21、029-photo-gallery/A22、
031-paint/A23、025-sys-monitor/A24、026-database/A25、027-file-manager/A26、024-charts/A27、
os-config/A28。子模块 SHA 与顶层 clone 差异 9 处（blog/calendar/charts/chat/database/notes/
paint/reader/todo），OS064/ASSEMBLY 以 gitlink 同 SHA 为准。**28 与 OS064 §5 表逐 ID 一致**。

## 7. K 表：发行/安装/冻结副本分母（细化 922 G-6）

| 行 | 项 | 身份/证据 | 现状 | 后继 |
|---|---|---|---|---|
| K-01 | man-install.py（auto-man scripts/:529 `--install-root` 必填；载荷 auto.exe+sdk/ 源闭包+stdlib .at+auto-ui-tool+autodown；manifest `man-install/1` 每文件 sha256+git HEAD；SCRUB 12 env :38-43；事务=staging+backup-swap+失败回滚 :474-523；core 禁 UI 源校验 :363-374） | 生产安装器 | REP-08 oracle 主体 | 986（948/978 备前腿） |
| K-02 | autographics 冻结 SDK ×2（`.sdk-source` 681 文件 == `sdk/frozen` 字节等同；含 vm/trans/engine 全量 vendored；钉 `research/source-lock.json` 932ad487 与 `baseline.json.txt` 7b987ca8 **两钉不一致**→UK-06） | auto-ui 仓内 | 退役必清（REP-10） | 981/SOURCE 核身份→987 清 |
| K-03 | auto-os apps/auto-notes/vendor | **非引擎副本**（autodown JS 子模块 + jade-garden back server 一份，其 Cargo.toml:8 以绝对路径引活 auto-lang）——修正 922 F-03 "vendored 副本"表述 | 消费者副本在账 | OS064/A03、A15 |
| K-04 | web-install.py（auto-web scripts/:229；载荷 auto-web+sdk+consumer+examples；vendored man-install.py 钉 auto-man cb4ab3dd，**滞后 owner**（缺 PLAN-036 主题校验）→UK-12） | Web 安装器 | REP-08 | 983/T-01→986 |
| K-05 | auto-musk install.sh（:49 gitee depth-1 clone auto-lang master `reset --hard`；无 manifest/hash/回滚） | 兄弟克隆安装 | UK（无身份安装面） | OS064/A02 T-01 核定 |
| K-06 | `C:/Users/zhaop/.auto`（legacy auto-man 主页 2025-05 二进制 auto-man.exe×2 版本；libs=stdlib **symlink→活仓**；sandbox 29 wrapper DLL；auto-up.at 指针） | 机器现存安装物 | 独立旧版回滚证据基底 + 清理面 | 986（回滚物）/987（清 symlink 面） |
| K-07 | 本机引擎二进制：PATH `auto`=auto-man target/debug/auto.exe（2026-11-05，142MB）；auto-ui-tool 同源；`auto-lang/target/{release,debug}/auto.exe` **不存在**——launch-vm.cmd:33 与 regen.sh:8 引用落空→UK-11；plan-054 frozen-sdk-r161..r195 快照（plan-artifacts，随 054 merge 清） | 机器现存 | 安装图核对输入 | OS064/986 |
| K-08 | 启动/脚本硬编码：launch-vm.cmd:33 `--render vm`、vm-first-run.mjs:50、start-all.cmd（cargo run/npm）、regen.sh:8、build-lsp.ps1:22/30/55（auto-lsp release→vscode bin） | 引擎/渲染选择 | REP-08 全入口图 | 986（+OS064/971） |

## 8. 后继责任核对（喂给关系与依赖可执行性）

- **逐包喂给**（本台账行 → 后继 AC/批次；后继计划 AC/T 引用见各计划 §7/§8，均为
  drafting/0 进度，喂给不提高其状态）：945←S-01/02；946←S-04；947←S-08/14/19/22 切片；
  948←S-01..S-25（全表差额映射）；960←L-37/38/41/42（+L-44 模块面）；961←L-01..05/49；962←L-06..19/48；
  963←L-24/25/38/39/40/42/47；964←L-08(Arc)/20/21/22/23；965←L-18(is)/21/26..30；
  966←L-32..36；967←L-37(桥)/A-10/S-22；968←L-29/30/31/43；969←A-01..03/08/09；
  970←A-06/D-07；971←A-04/07/F-07/F-10(test)/S-24；972←S-01..06；973←S-08..15；
  974←S-16；975←S-07/17/18；976←B-05/C-05；977←B-03/04/06/07/08/09/L-44(Scene)；
  978←L-36(构建侧)/K-07；979←L-44/领域；980←C 全表/S-20/22/23；981←F-11/K-02；
  982←D 全表/A-11/F-06/K-01(DEVSDK)；983←E 全表/K-04/F-14(Down)；984←F-05/09/10/13/
  F-15 账外核对；OS064←F-01..04/08/14/K-05；985←MIX 绑定（其 T-08）；986←K-01/04/06/
  07/08；987←B-10/G 全表/K-02。
- **无主项处置**（959 行级补齐，防"计划没逐名就没有 owner"）：平台条件→L-36（966+978）；
  TaskDef/on=>→L-43（968）；Hash 各形态→L-32（966，Design55 §5 归属落行）；vscode 模式
  →C-06（971+986）；lambda 桩→L-23（964，已知旧bug 不随迁移静默消失，作 oracle 输入）。
- **依赖可执行性**：由 frontmatter `depends_on` 重建的语言仓 M2 DAG（959→960/961→962→
  963→964→965/966/967/968→969..982→983/984→985→986→987；945-948 挂 960/962 后汇 972）
  复验**无环**；在途前置仅 941（executing，985 显式依赖）、938（executing）、939
  （drafting）归 967；已 archived 前置（929-937/942-944/950/958）不阻塞。跨仓节点
  OS064（959/969..983 前置）与 984 双向核对：OS064 不反等 984/985，无互等环。
- **全 M2 缺口不因盘点关闭**：本台账所有 CM/UK/已知差异行在 REP/V06 实际执行前保持
  open；盘点完成≠产品通过（SD-01 证据边界）。

## 9. UK 登记（unknown，逐项 owner + 关闭任务）

| UK | 内容 | owner / 下一步 |
|---|---|---|
| UK-01 | rust_ui.rs（~7800 行）a2r 调用点外是否直调 trans::rust | 980/RUST T-01 逐行核 |
| UK-02 | serve_owned/listen_borrowed 公开嵌入面仓外 SDK 消费者 | 983/CONSUMERS T-01 安装收据/文档 survey |
| UK-03 | 旧 `auto_lang::ui` 面消费者（auto-term term.rs:191 已复核仍在用 TERMINAL_PALETTE_SLOTS、at-gen、auto-edit、musk 生成 back）对新 main 可编译性 | 984/EMBED+EDIT、OS064 T-01 cargo check 基线 |
| UK-04 | apps 兄弟 clone rust-workspace 逐仓审计（agent 已枚举，破损钉 4 处登记 §6/F-14） | OS064/ASSEMBLY T-01 |
| UK-05 | corpus a2vue/a2ark/a2j 真实消费方 | 977 T-01 |
| UK-06 | autographics 冻结 SDK 两钉不一致（source-lock 932ad487 vs baseline 7b987ca8；.sdk-source==sdk/frozen 字节等同） | 981/SOURCE T-01 定操作钉→987 清 |
| UK-07 | auto-musk rust-workspace 生成物 producer 身份（钉活 auto-lang path） | OS064/A02 T-01 |
| UK-08 | experimental/ace worker 运行期定位策略产品化 | 967/978 |
| UK-09 | （已闭）922 UK-09：919 merge 在途对 H 面漂移 | 已由 delta 规程（§10 第 2 条）承接关闭；行保留保 ID 稳定 | —（closed 2026-11-07，PLAN-959） |
| UK-10 | auto-gen 复活 vs 归档（F-06，用户工具意向） | 982/T-01 呈用户裁定 |
| UK-11 | launch-vm.cmd:33/regen.sh:8 引用的 auto-lang target 二进制现存缺失（安装图断链） | OS064/HOST+986 T-01 |
| UK-12 | auto-web vendored man-install.py 滞后 owner（cb4ab3dd，缺 PLAN-036 主题校验） | 983/T-01 同步裁定→986 |
| UK-13 | L-36 平台条件语言形是否存在/需不需要（谓词边界） | 966 T-01 冻结 + 978 T-08 构建侧 |

（922 UK-09 漂移项已由 delta 规程承接关闭；UK-10 沿用原号。unknown 仍阻 M2 终签。）

## 10. delta 规程（冻结后追加）

1. **新增行**：新发现的语法项/API/消费者/安装物以 `delta-<date>` 注记追加到对应表尾，
   带完整来源身份（repo@rev + file:line）；不插入、不改写已冻结行的 ID 或证据。
2. **来源变更**：某行所引 repo/文件 SHA 前移时，该行标 `stale@<date>`，**只重核受影响行**，
   不触发全仓重盘（V-04 口径）；未变行证据继续有效。
3. **状态提升**：仅当后继计划交付并经独立 review 后，才可在 delta 注记中提升该行状态，
   并引用收据 commit；本台账自身不产生"已实现"声明。
4. **退役/删除**：行不删除；旧实现退出以 SD-02 证据（依赖图/dispatch/包内容/安装收据）
   为准，在 delta 注记中登记退役收据指针。
5. **口径变更**：分类法或前缀含义变化须修订 PLAN（needs_replan），不允许在台账内静默改口径。
