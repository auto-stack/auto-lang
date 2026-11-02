# 共享可变捕获（标量切片，PLAN-909）

> 状态：实验性，属于 `auto.source.closures.draft` profile 的语义扩面。由 PLAN-909 交付（2026-11-02）；承接 865 源码闭包与 823/824 闭包环境，不改变其他源码 profile、wire 版本或默认入口。C-19 的共享 owned/循环图/跨挂起借用等残余仍属后继，不在本切片。

## 交付的语义（SEM-04 扩面）

共享可变捕获分两种模式，值域均为函数局部 **`var int/bool/char`**（Copy 标量）：

| 模式 | 规则 | 区分性 oracle |
|---|---|---|
| 私有可写 env（move） | `move` 闭包对 Copy 标量的写捕获：创建时复制当前值入闭包私有 cell，跨调用保留；外部源仍活、**不写回** | K06 原源 42；外部 base 相加为 43，误共享为 63 |
| 共享 cell（非 move） | 同一词法绑定的所有非 move 读/写捕获共享一个 cell；外部赋值与闭包调用更新同一受检位置；各次函数激活独立 cell | 兄弟 writer f(20)+g(0)=42，误复制为 22；先创建的 reader 也读当前共享值（82） |

混合行：同一绑定同时被 move 与非 move 闭包捕获时，move 闭包在创建点取当前值快照并与原 cell 分离（84）；非 move 侧继续共享。`let` 写入仍是静态不可变赋值拒绝（`verify.immutable-write`）；非 move 写捕获非 Copy 值（Str/record/List/Variant/closure/Dyn）具名拒绝 `source.closure-shared-mutable`；mut/view 参数提升不在域内。原 865 的 owned move/clone/同步 Str 借用域不变（K01..K05 保持）。

## 绑定身份与静态分组

BindingId = (函数, 声明标识符 token 起始偏移)，绝不按名合并。`scan_shared_capture_cells` 以自带作用域模型（params/let/var/for-in/is-match/闭包参数遮蔽；闭包捕获穿透到外层绑定）扫描函数体内全部 lambda：**非 move** 闭包的 written 自由名按当前可见 `var` 标量绑定入共享组 S——分组是静态的：写者在未执行分支、或 reader 先创建，cell 语义同样成立。cell 在 `var` 声明点创建（初始化值只求值一次，先求值后记账/分配/提交）；构造失败是资源/infra 分态，不吞副作用。同名遮蔽、不同激活、嵌套词法来源不串 cell。

## 物理表示与受检事实

cell 的物理表示是**单元素 typed `List<T>` 句柄**（`T∈{i32,bool,char}`）——823 ENV_GET 合同既有的"list 内部可变性即可变 env 字段路径"：creator 槽、每个共享闭包 env 字段、嵌套闭包持有同一缓冲的句柄；读写经 `LIST_GET/LIST_SET`(索引0)。**零新增 opcode**；唯一新增 MIR/ABC 面是 `CaptureKind::Shared`（CREATE_CLOSURE 捕获 tag 4）：env 字段获得源槽的**句柄共享**（owner 计数 +1，同缓冲），源不 kill、不 pin、可逃逸；旧解码器遇未知 tag 即拒（`abc.decode.arity`），不猜含义。

受检事实由 core 生成并验证，lowering 只消费：`hir::CaptureMode::Shared` + 闭包 body 捕获局部 `mutable`（K06 私有可写的受检事实）；descriptor/closure atom schema 词表 `shared`；`verify.closure-shared-domain` 门（Shared 源须 mutable 标量且 writability 镜像；Move 可写捕获须标量；Borrow 捕获不可写）；MIR 侧 `mir.capture.shared-domain`（源须 `List<i32|bool|char>`）；ownership 对 Shared 不 kill 不 pin。普通 List 复制或无校验 env 别名不能充 cell 合同——表示必须经上述受检事实与双门。

## 写回、逃逸与 clone

外部赋值与闭包调用沿原源码顺序更新同一 cell；求值/写回顺序沿原语义，允许顺序重入，共享值不留宿主借用跨 guest 调用。shared cell 是提升后的拥有对象：可由返回/既有闭包参数传递保活，创建者返回不悬空（factory 逃逸 42）；原 Str Borrow、mut/view 真实借用仍在编译/verify 门拒逃逸。`move` 转移 env owner；显式 `.clone()` 深复制当前可达 env 与 cell，原/新互不写回（44，误共享 65）；**同一 clone 根内同一 cell 只复制一次**（同根 buf_id memo，合法内部别名图的共享关系在副本中保持，不回连原图）。私有可写 env 不随 CALL 重建。

## 生命周期、失败原子与资源

构造/clone 沿 823 prepare/commit 两相：全部可失败工作（cell/env 分配、深复制、预算）在 prepare，commit 纯校验转移；失败时源/旧值/既有 owner 不被消费，临时产物全部释放（逐分配点故障扫描全部 Infra 且 live=0）。cell 字节计入唯一活值预算线（`slot_bytes` 深计量，共享按引用保守计费），不改任何既有 cap；代码 cache 只存代码/布局/效果，活 cell 不入 cache、不恢复运行结果——重启从初始源状态真实执行。

## 函数增量（源码入口）

lambda 显示身份 `__closure_<owner>_<k>`（owner 内序号；resolved IR 按名引用 callee）；PLAN-912 为每个 lambda 另驻留**逻辑 ID**（冷init铸造、跨重建按 (owner, 文本) 唯一匹配保持，拓扑变化的存活单元不串、歧义不误命中）。PLAN-912 暖路径为单函数粒度：lambda body 暖编辑在保留创建点事实上仅重解析该 body（raw 捕获 uses/参数/move/结果任一变化＝具名转 owner 路由）；owner body 暖编辑经 retained-lambda 复用 oracle——文本同且每个保留捕获按当前创建点 locals 经唯一决策表重推全等的 lambda 原样复用（嵌套 span 平移），真读者（捕获 mode/cell 变化的兄弟）精确失效重走。未改单元全阶段生产 0、同对象/代际；whole 轴暖路径必须 0（结构门）。全复用应用 = 零产命中；失败候选不损缓存/已装线。restart 经 store read/decode/重验（零写盘、全命中；重 lower/encode 如实非零＝R3，属 913）。详见[闭包单函数前端生产](closures-function-production.md)。

## 语料与验收

原 K06 `.at`（hash 不变）真实链 42，AV check 由 resolve 期拒绝迁移为 success/42（PLAN-909 裁定）；`shared-mutable-909` suite（S01..S06 正例 fresh/body-edit/restart + S07/S08 不支持域负控 + 手写 Rust 参考）以 `case-ace-m1-shared-mutable-909` 固定组运行；父侧判决表持独立 oracle。K01..K05 与旧 typed 823 证据不变；legacy AVM/a2r、golden/cert 不动。

详见[源码闭包域](source-closures.md)、[闭包捕获 MIR](../auto-mir/closure-capture-profile.md)、[闭包环境与执行合同](../auto-av/closure-environments.md)与[ACE M1 一致性语料](../auto-test-case/ace-m1-conformance.md)。
