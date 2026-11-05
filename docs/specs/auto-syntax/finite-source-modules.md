# ACE 有限公共函数模块

> PLAN-950；实现位于 `experimental/ace/crates/core/src/source/modules/`、
> `core/src/source/cst/frontdoor.rs` 与 `source-query/src/modules/`。

## 支持入口与边界

`ac.source.modules.scalar/v1` 是显式源码能力，消费 `auto-source.at` 的
`scalar-modules` 裁剪面。原九 profile 的默认入口不扩大；模块与有限模板能力
组合目前具名拒绝。模块入口保留 `ModuleSyntax` 的导出、导入和声明来源，
不能把 public/import 文本删掉后走单源入口。
token也遵守profile裁剪；`pub`只在模块能力中作为关键字，原profile的同名标识符保持可用。

宿主以稳定逻辑 ID 和独立源文本供给有限模块图；只支持 i32/bool 顶层自由函数，
沿原 scalar 算子/语句和类型规则。没有包/目录发现、通配/别名/重导出、循环模块、
公共类型/资源或任意泛型。这里只交付模块语言前置，不声明任何实际公共标准库
操作的行为已经实现；标准操作引用证明和装配身份由944继续接入。

## 名称与可见性

`pub fn answer() int { return 42 }` 导出函数，普通 fn 仅本模块可见。
`use std.answer` 为裸调用引入显式公共绑定，导入必须在函数之前；
`std.answer()` 可直接限定调用宿主提供的模块。用户本模块同名自由函数优先于
裸导入，限定调用仍绑定指定模块；局部值不作为模块命名空间使用。

重复模块/函数/导入、未知模块/符号、跨模块私有访问和循环依赖在执行前拒绝。
签名与实际参数类型由现有 source-query 事实、唯一 resolve/typecheck 和 HIR
verifier 核验；模块绑定本身不铸 Checked 凭证。不同模块同名 private helper
具有不同内部符号身份。内部名字不是标准操作 ID 或 native 数字 ID。
类型拒绝直接将现有事实页的结构节点诊断映射回当前模块/字节span，不重跑源码解析或旧类型引擎。

## 增量与来源

冷入口 `compile_module_product` 经统一 CST→模块 AST 绑定→原 query 事实→
原 HIR 发射/验证。`ModuleSession` 保留同一事实 authority 的 `CheckedEnv`；
恢复后用声明清单装配 stub-body 环境，不从源码重建它。

`edit_module_function` 只解析同名替换声明，包含实际 public 前缀；来源由不可变
CST origin 和 mint/current 声明区间构成。每个 current slice 必须等于其原始
认证 slice；变长编辑只迁移未变声明的区间。`prepare_module_candidate` 复用
原 FnCandidate 的临时 graft→还原→提交事务，单体 resolve 由已准入事实供型。
同签名 body 编辑只有目标函数 parse/namespace/resolve/typecheck/plan/bind；
其他函数的代码与 target 片段复用。签名漂移属于原单体事务的具名非快路径，
不能暗中整源 fallback。

被读签名改变使 query reader 失效，未读者保持命中；名称缺失在绑定门拒绝，
不将失败诊断伪装为已安装事实或可执行产物。后续新增导出须重新经过当前声明
绑定与 query 准入。声明上下文变化的冷编译与同签名 body 快路径分账。

## 存储与恢复

模块根 `source-modules-root/2`（S950、SnapshotRecord）保存 source-set 身份、
entry/package/module、grammar、binding version、声明性 source capabilities、
CST 来源/声明迁移、既有事实页和声明环境；不保存宿主地址、运行授权或信任位。
宿主保有 root artifact ID 与 SHA256 receipt。旧/未知 schema 拒绝；不能用
同值空指纹证明新鲜。

CST/源页/词法数据仍使用 A01/A02/A03 和原 ManagedStore；候选使用
`put_cst_unpublished`，不抢占旧当前根。S934 的模块页地址 recipe 为
`sq-module-page/1`：沿原事实 codec，地址加所读签名/negative 版本，临时请求
revision 规范为0。旧 bare S934 地址及读取合同不重解释。

恢复先验证外部 receipt、schema/grammar/capabilities、source-set、声明唯一性、
required 边和源区间，再从既有 CST 重投影、核事实 key/shape/coverage 并 seed
原 query session；无 scanner、源码 parser 或 solver。声明环境与当前源码的
名字、参数/mode/结果、owner 和 span 逐项核对后才可用于首编辑。

根可持有 source-bound target receipts（`source` 字段为同一 source-set hash），
其 digest 与 required 边均由父 receipt 保护。AV 目标沿原 ABC envelope、
FunctionCode admission 和 ProgramSnapshot link；Rust 目标沿原 A2X provider、
AXS/AXR 片段身份、runtime/print 版本与实际 file-set 收据。恢复重新准入，不把
持久化字节当验证凭证。旧目标 receipt 不得重绑新 source-set。

所有目标候选与数据根先准备，单写宿主核同一 base 后才发布；取消、绑定错误
或根注册失败保留旧 root、代码句柄和已生成文件。注册新完整 required 闭包
成功后才撤旧根。读取/恢复后需新的运行授权；本有限纯函数域没有新增宿主调用。
目标在生产前核候选的原source-set基线；不同源会话的编辑不能贴到旧目标上。
当前根的比较、注册与旧根撤销在同一SQLite事务中提交，任一步失败均回滚。

## 验证责任

永久入口包括 core 模块语法/绑定测试、source-query 模块存储/首编辑测试和
`a2x-worker::conformance::module_sources_950` 的两个 Windows x64 真消费者用例。
分别对独立42/43/7 oracle执行，核未变 AV Arc/AXR artifact ID，并检查下一次
恢复、实际 root 写入失败、旧目标重绑和 producer 版本反例。实际计数与退出码
见归档 Plan；其他平台/资源/标准库族不能据此声明支持。
