# 闭包捕获 MIR（av.closures/1）

> 来源 PLAN-823；实际实现 `experimental/ace/crates/av-core/src/{mir,mir_verify,instr_spec,abc,ownership}.rs`。

## 源码捕获映射（PLAN-865）

显式 auto.source.closures.draft producer 在 HIR→MIR lowering 前分析自由局部变量：排除 lambda 参数及块内局部，按首次源码出现位置去重排序。闭包 HIR descriptor 携带已决的捕获模式和顺序；lowering 不推测捕获模式。

- Move 映射为 owned env 字段并消耗非 Copy 源局部；Copy 类型在 move lambda 中仍为 Copy。
- 源码闭包值的 .clone() 映射到既有 ClosureClone，深复制已构造的 owned env 并生成独立句柄；它不改变 lambda 捕获列表或字段顺序。
- Borrow 仅接受可证明同步且不逃逸的源码 Str 捕获，映射为冻结的 StrView env 条目。

lowered env vector 仍须与闭包 body 声明的 MirFunction.env 逐字段相等。既有 CalleeRef、唯一创建点和 fn: 链接期 env 等价规则不变；声明漂移在候选发布前拒绝。函数类型参数/返回值沿用现有闭包签名和间接调用机制。PLAN-865 不增加 MIR 指令或 wire 身份。

**PLAN-909 共享可变捕获（标量切片）**：新增捕获态 `CaptureKind::Shared`（CREATE_CLOSURE 捕获 **wire tag 4**；0/1/2 既有，3=MoveFromEnv）——env 字段获得源槽 `List<i32|bool|char>`（单元素 cell）的**句柄共享**：源不 kill、不 pin、闭包可逃逸；verify 门 `mir.capture.shared-domain` 仅准该 cell 表示，其他值类共享具名拒。私有可写 env（K06）与共享 cell 的读写均走既有 LIST_GET/LIST_SET 于索引 0（ENV_GET 合同既有的 list 内部可变性路径），**零新增 opcode**；未知捕获 tag 旧解码器即拒，不猜含义。闭包 CLONE 对同根同 cell（buf_id）只复制一次，副本内部共享关系保持。PLAN-912：单函数文档的 env 布局与 cell 位来自验证器凭证（`ClosureBodyFacts`，`lower_function_closures_unit_checked`），不再由 lowering 扫描其他 body 推导——整模块路径的扫描推导保留且两者字节等价；详见[HIR lowering 边界](hir-lowering-boundary.md)与[闭包单函数前端生产](../auto-ac/closures-function-production.md)。闭包字段/List 元素及集合方法 HOF 仍不支持，详见[源码闭包规范](../auto-ac/source-closures.md)。

## 既有 MIR 合同（PLAN-823）

`av.closures/1` 在已交付集合 profile 上增加 Closure 槽与 `CREATE_CLOSURE`、`CALL_CLOSURE`、`ENV_GET`。闭包 body 自带 env 声明：`MirFunction.env: Option<Vec<TypeIdx>>`（Some=闭包体；声明即 env 布局，borrow 捕获条目=StrView），`ENV_GET` 对本函数声明定型（单元独立可验）；`CREATE_CLOSURE.callee: CalleeRef`（模块=Func、单元=Import，import 表携 env 声明），captures 逐位等于被调声明；link 期 `fn:` 绑定做 env 等价检查（漂移精确拒绝）。site 唯一规则：每 callee 至多一个创建 site，重复 site、entry 为 body、直接 Call body 均拒绝。

捕获三态 `CaptureKind`（Move/Clone/Borrow，wire tag 0/1/2）：move 经 ownership 分析杀源（活跃借用冲突拒绝，镜像 MOVE）；clone 读不杀源；borrow 限 §5.1 允许集的同步非逃逸纪律（保守扫描：源槽钉住、terminator 覆盖 Return 逃逸、仅 CallClosure/Drop 消费）。逃逸admissible 集拒绝 StrView/MutRef/Place/Closure/Dyn 值（`mir.capture.kind`，attempt=0）。

语义 MIR、受检 ABC 与执行态布局分层；函数/环境必要类型、owner 与清理元数据为语义信息，不随 debug 剥离；strip 前后结果与清理一致。帧复用不复活旧 token：frame 实例 UID、env generation 与 store 句柄单调不复用，旧代际读取在消费前拒绝。错误索引、类型、状态及不支持 profile 在 verify/ABC 准入门拒绝，不以 panic 或未授信执行代替。环境拥有权与调用合同见 [闭包环境合同](../auto-av/closure-environments.md)。
