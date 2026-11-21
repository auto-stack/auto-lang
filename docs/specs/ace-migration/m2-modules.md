# ACE 完整模块包与公共类型绑定

> PLAN-960；实现位于 `experimental/ace/crates/core/src/source/packages/`、
> `core/src/source/cst/frontdoor.rs`（packages 前门）、`source-query/src/modules/{packages,packages_store}.rs`、
> `av/src/modules/packages.rs`、`a2x-store/src/modules/packages.rs`、`av-core`（MIR 全局 cell）、
> `lowering/hir_adapter.rs`（访问器降级）。本文件只记录**已验证**的支持、接口、拒绝与边界；
> 未列入的能力不视为已支持。950 有限模块（`ac.source.modules.scalar/v1`）面与行为逐字节不变。

## 支持入口与身份

`ac.source.modules.packages/v1` 是显式源码能力，消费 `auto-source.at` 的 `packages-modules`
裁剪（= scalar-modules 全表面 + text/record 表面 + 模块项）。编译域为 TextRecord；与
`ac.source.modules.scalar/v1` 不得同时携带（`source.module-capability`）。模板能力共存不再
结构性互斥，但模板构造在绑定层具名拒绝 `source.module-templates`（PLAN-963 闭合）。

宿主以包闭包供图：`PackageModuleInput{package, version, id, source}`（包名 `[a-z][a-z0-9_-]*`，
版本非空 `[A-Za-z0-9.+_-]`，模块 id 为标识符）；Core 不做文件系统/包发现。图身份
`input_hash = sha256("source-module-binding/2:{len}:{entry}\n" + 按 `pkg::mod` 排序的
"{plen}:{pkg}:{vlen}:{version}:{mlen}:{mod}:{sha256(source)}\n")`——输入顺序/宿主重定位不改身份，
版本不同即不同身份（V-05）。entry 以 `pkg::mod` 键给出。

## 表面与可见性

模块项：`pub fn`/`fn`、`pub type`/`type`（字段 + static/view 方法，消费既有 record 表面）、
`pub const NAME T = expr`/`const`（不可变全局）、`var NAME T = expr`（模块私有 store，无 `pub` 形）、
`use mod.item [as x]`/`use pkg.mod.item [as x]`（2 段=本包，3 段=跨包，>3 段 `source.module-path`）、
`pub use …`（重导出，别名进入本模块公共面）。导入须先于声明（`source.module-import-order`）；
同名绑定重复 `source.module-duplicate-import`；fn/type/const/store/导入共享模块命名空间
（`source.module-duplicate-declaration`）。

限定引用沿 record 面的点后缀投影：`mod.item(...)`/`pkg.mod.item(...)` 为跨模块调用，
`mod.CONST`/`pkg.mod.CONST` 为常量读取；局部名与类型名遮蔽同名模块。跨模块访问须目标公共
（`source.module-private`）；重导出链逐段解析，私有目标 `source.module-reexport-private`（V-02），
链环 `source.module-reexport-cycle`。store 永不可导入/导出。常量不可赋值
（`source.module-const-assign`）；store 路径赋值不在本面（`source.module-store-path`）。
函数值/间接调用/闭包/集合/CT 构造具名拒绝并指向 962/964/966。

符号：fn/global/type 分别 `p{plen}_{pkg}m{mlen}_{mod}_{name}`/`g…`/`t…`（包名 `-`→`_`，长度前缀
无歧义），entry 模块 `main` 保持 `main`；两个包的同名私有符号互不可见（V-01）。

## 声明图与初始化

use/重导出边构成模块图，须无环（`source.module-cycle`）；**函数调用不构成模块边**，跨模块互递归合法。
const/store 的初始化值在运行期求值：绑定器把读写改写为 `module.global.get/set:<sym>` 访问器调用
（set 产 i32 0，绑定到合成 `let __setN`），每个含全局的模块合成 init 函数 `i{plen}_{pkg}m{mlen}_{mod}`，
按**值初始化依赖的传递闭包**（直接读 + 被调函数的读）拓扑排序；环具名
`source.module-init-cycle` 并列出成员（V-06）。模块 init 序由全局 topo 的最晚依赖决定；单模块
无法作为整体初始化时拒绝 `source.module-init-order`。

执行语义（AV 已验证，mn/a2x 全局 cell 见边界）：MIR `MirGlobal` cell 表 + `GlobalGet/GlobalSet`
（opcode 0x36/0x37）；`ProgramSnapshot::link_with_globals` 建程序级 cell 并按序登记 init；每代
恰好初始化一次（diamond 依赖共享同一 cell），任一 init 失败重置全部 cell、不标记已初始化、
运行报错——不发布半状态。cell 值与 init 状态属运行代，从不持久化。

## 消费者与恢复

- AV：`PackageAvTarget::cold/cold_with_entry/run_value`——i32 入口返回 `Value::I32`，Str 入口
  返回 `Value::StrView`；`global_cells()` 仅供观察。
- Rust/C：`PackageA2xTarget<P: TargetProvider>`（record-text 族 AXR2/AXC2），`cold` 发射片段并
  装配文件集，`receipt` 写根 `a2x-source-packages/1`，`restore` 核 schema/source/backend/caps/
  target_inputs/声明集/required 边后仅重装配（lower/print 计数 0）。片段名限自由函数（类型方法
  片段为具名后继）。
- native/mn：沿 `MnDriver::build_checked(checked, "d_main", …)` owned 臂，`ac_start` 以退出码
  转发入口 i32。
- 根：`PackageStore` `source-packages-root/1` 记录 binding/grammar/entry/caps/input_hash、
  各单元 (pkg, version, mod, sha256, source) 与消费者收据（required 边）；根不含 cell 值或宿主
  地址。恢复核收据 digest，经同一前门/绑定/编译重建并要求 `input_hash` 相等；篡改收据与外来
  产品均拒绝。
- 944 装配：`Manifest` 接纳 `ac.source.modules.packages/v1` profile 与 `ace-native/modules/1`、
  `ace-a2x-c/modules/1` producer。

## 验证责任与边界

永久入口：core `source::packages`（语法 2 + 绑定 3）、source-query `modules::packages`（2）与
`modules::packages_store`（1）、av `modules::packages`（2：store 隔离/diamond once/无半态/Str）、
`av::conformance::module_packages_960`（四端 Str 冷运行+恢复、V-01..V-06、白名单；Windows x64）。
固定组：`cargo t ace-source ace-semantics ace-artifacts ace-av ace-a2x`。

具名边界（非静默回退）：934 事实页 `PortableTy` 仅标量，本面直接跑同一 resolver，增量仅绑定账本
（D-960-1）；mn 全局 cell 具名拒（D-960-2）；a2x 全局 cell 发射与类型方法片段为后继；恢复首刀重
投影+重编译（无 solver 复用）。其他平台/资源/标准库族不据此声明支持。

## 普通 Auto AV 工程（PLAN-1085）

`compile_package_product_auto` 使用公共 CST、包图绑定与普通 Auto 类型检查进入 `av.closures/1`。项目文件集按原虚拟路径和内容校验。初始化合成到包图并全链重新准入每个 ABC 单元。链接时，单元中的全局读、写、取出按 ordinal 与 producer 类型表重新匹配。`av.globals.take/1` 标记可移动 cell 的新指令准入。每代全局状态在内存持有，包根包含源身份和编译 profile，冷恢复沿同一前门而得到新代。初始化失败释放已取得的 owned 值，同代重试重新执行初始化。公开 ABI、类型/环境表和平台/host 权限沿共享契约。原词法/包规则和其它 target 范围见上文，普通工程实测与来源见 [PLAN-1085](../../plans/archive/1085-av-parity-modules-and-global-state.md)。


## PLAN-1095 增量（普通工程组合收口）

普通 Auto 包/工程面（PLAN-1085 之上）新增：本地自由 fn 作值（工程绑定通道与
单源一致）；匿名 record 字面量/字段投影、try/finally、task 组合在同源共存；
CT 表达式与运行时列表/filter 同 session 组合。AGE node 声明在普通 Auto 面由
CST 投影为 `Item::Node`（名字+span，体不重投影），node view 文本经 979 producer
真实执行观察。工程恢复语义：坏编辑具名失败、恢复缓存命中且与冷编一致、双
session 不串 state/cache。共享语法/绑定/IR 改动仅按 AV 及共同正确性验证；非 AV
目标检查保 `#ignore` 与后续 owner。来源与逐 case 结果见
[PLAN-1095](../../plans/1095-av-parity-ordinary-project-composition.md)。
