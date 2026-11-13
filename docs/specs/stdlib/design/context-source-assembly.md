# ACE 标准库 context 装配与最小 #in 平台选择（PLAN-1043）

## 支持边界与真实入口

标准库公共模块与其实现来源按平铺文件后缀装配（2026-11-13 用户确定）：
公共 `io.at` 加四类候选 `io.at.at`（纯 Auto）/`io.c.at`（C 适配 Auto 源）/
`io.rs.at`（Rust 适配）/`io.py.at`（Python 适配）；最长保留后缀分类，
`io.at.at` 永不被列为独立公共模块。所有候选都是 Auto 源码，经同一
packages 前门（`packages-ct` 裁剪 + `ac.source.cte.in/v1` 能力）逐文件
parse/CST/span，语义合并在 `ace_core::source::packages`（选择模型
`provider_sources`、绑定合并/`#in` 求值在 `binding`）；不拼文本、不建
第二 loader/binder。旧 `.vm.at` 目标层选择仍归 legacy 旧配方，与本合同
互不混用（退役归 986/987）；legacy inventory 把 `.at.at`/`.py.at` 归属主
模块 Other 层。

## 三轴选择与逻辑模块

`backend`（AV/AC/A2R/A2PY/原A2C）×`provider`（at/c/rs/py）×`target`
（Windows/Linux，Mac 仅可选输入）由显式 `PackageAssemblyContext` 供给。
相容表：at→全部后端；c→AV 与 A2C；rs→AV 与 A2R；py→A2PY。默认序
`Auto`=纯 Auto（at）；at 缺而其他存在 → `assembly.missing-default-provider`
具名拒，绝不静默回落；点名来源缺文件/后端不相容 →
`assembly.provider-missing`/`assembly.provider-backend`。绑定重验输入层
与上下文选择一致（`assembly.provider-selection-mismatch`），手拼输入不可
绕过。

公共文件可带 bodied 声明（跨 provider 单源共享）；bodyless 需求用
`pub spec fn name(params) Ret`（无块；packages-ct 面专用拼写，首 token
`spec`）。装配规则：每个需实现操作恰一个 body——实现文件同签名 `fn`
填充（签名域=参数个数+逐位类型+mode+返回类型，参数名不入合同）；公共
bodied+实现同名=重复实现拒；实现独有 fn=模块私有 helper；实现层声明
type/const/store 或进一步需求=具名拒；未填需求=`source.assembly.missing-
implementation`（附所选来源），均在任何调用前拒绝。

## `#in` 编译期命名空间与条件导入

`#in sys is current { Arm -> use ... }` 单结构式与 `#in sys { ... }` 块式；
内部 `is`/`if`/`for` 继承编译期属性（不重复 `#`），`for i in a..b` 编译期
计数迭代（≤1024）。臂体=单行 `use path[: syms]`（符号表形=逐符号导入，
不得再带 `as` 别名）或块（use/fn/type/const/pub 声明；声明体仍是运行时
block）。落位：选中 use/声明按 `#in` 的源序槽位落回本模块（含 use 的
`#in` 须在声明区前）；未选分支只保 Text 阶段语法检查，不符号绑定/类型
检查/链接。函数定义不因位于 `#in` 内而立即执行；CT 权限/预算沿原合同。

`sys` 是唯一注册命名空间（`SYS_FACTS_SCHEMA`，事实由显式 target 供给：
`current`=目标（目标轴选择，V-09 语义）、`host`=宿主另名、`Windows=1/
Linux=2/Mac=3` 同域 i32 常量）。表达式域=标量事实（i32/bool，对齐
source-cte CtValue；仅 `== != < <= > >= && ||`，无括号分组/算术/调用）。
求值是绑定期静态事实折叠（复用 `is` 值相等匹配与整结构规则，不新增
解释器）：未知命名空间/成员、qual 与 namespace 不一致、非 bool 条件、
等值重复臂（`source.cte.duplicate-arm`）均具名拒；`is` 无匹配=空展开
（必需实现缺口由装配检查拒绝，不改全局空展开规则）。限定形
`is sys.current { sys.Windows -> }` 合法；运行时对象/裸 `in` 不构成 CT
命名空间（语法面拒）。

## 身份、缓存与恢复

绑定 identity 含：公共源 sha、所选实现层（kind+sha）、装配上下文
（backend/provider/target/`PROVIDER_SOURCE_SCHEMA`）、sys 事实表与候选
存在性读集（`module:at+c` 串形）。未选实现 body 内容不入身份（仅改未选
文件→身份不变复用）；增删候选文件即失效。packages store 根 schema
`source-packages-root/2`：单元可携实现层记录，根携上下文；恢复经同一
装配权威 re-parse/re-bind/重验身份；`/1` 根不再互认；无上下文却带实现
层=拒绝。失败/取消不触旧根；不携带授权、进程地址或外库 handle。

## 证据入口

- `ace-core` 单测 `source::packages::provider_sources`（后缀分类/三轴
  选择/相容表/存在集）与 `binding::plan1043_tests`（ASM-01..03/06 正反例：
  填充/重复/签名漂移/缺实现/目标轴/未选不绑定/等值重复臂/域外/诊断归位）。
- `ace-source-query` `packages_store` `plan1043_assembly`（ASM-04：身份
  全轴失效、未选 body 复用、恢复重验、失败保旧、根内容边界、发现面过滤、
  缺默认具名拒）。
- `ace-av` conformance `stdlib_assembly_1043`（ASM-05：AV×at/c/rs 四配置、
  AC 原生 mn、A2R rustc、A2C cl、A2PY 真 OS Python（1042 `PythonTargetProvider`
  +pyrun，拒绝旧 AST emitter）；Linux 目标忽略缺席 Windows SDK；store 往返
  后 A2C 实跑）。纯选择 fixture 不代真实外库 dispatch（967/库 owner 另交）。
- 冻结并集：`cargo t ace-source ace-syntax ace-semantics ace-stdlib stdlib
  ace-a2x ace-av`（合并 1042 后 516/132/275/2/170/235/1091 executed 全
  PASS 0 ignored）。plain packages-modules 裁剪与各旧 profile grammar-id
  逐字节不变（`packages-ct` 独立槽；FRONT_CAPS_VERSION 不经此路径不动）。

## 边界（未交域）

`#in` 内不支持：else-if 链（`else` 后接块）、负数界的融合 `0..3` 单词形
（须空格分解形）、type 别名声明、类型参数需求（`pub spec fn` 无 `<T>`）、
方法体 bodyless 需求（类型方法须公共文件带体）。AV 的 c/rs 配置执行的是
受检 Auto 适配源（纯选择证明），真实 C/Rust 外库 dispatch 归 967；OS
原语归 1039/1040；完整 CTE/宏/metadata 归 966 消费本包。Mac 仅覆盖输入
与选择机制，不构成产品平台 PASS。
