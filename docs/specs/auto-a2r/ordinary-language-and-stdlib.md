# A2R Ordinary Language & Stdlib

> **Status**: experimental
> 路径：`experimental/ace/crates/a2x-rust`（crate `auto-a2x-rust`）+ worker `probe::a2x-worker` | 依据：Design 51/60（PLAN-1115）

普通 Auto 源（`auto.source.language.av/v1`）→ 共享 Checked HIR → `SemanticView` → 统一 language 通道降落 → 真实 rustc → 受控执行。执行闭包不含 AV、旧 VM 或旧 AST emitter。

## 通道 identity（窄通道并列，互不相换）

| 面 | 身份 |
|---|---|
| 运行 recipe | `ac-a2r/hir-language-run-v1` |
| 发射 recipe | `ac-a2r/hir-language-emit-v1` |
| fileset schema | `auto.a2x.rust.language.fileset/1` |
| cert schema | `ace.a2r-hir.language.cert/1` |
| worker profile | `language` |
| 降落入口 | `lower_language_program`（collections 族统一 lane） |
| 固定组 | `case-a2r-language-parity`（target=a2r） |

## 已验域

### 数值/文本/record（AX-A/B 基础+扩面）
- 961 全数值面（i8..f64 字面量、checked 算术、类型转换矩阵含 widening 补对）、text 连接/NUL/char/查找/trim/大小写、record 构造/字段/深克隆、闭包（不可变捕获域）。
- V-01：数值边界（i64 精确/i64::MIN/narrowing）、具名 trap（Overflow/DivByZero/RemByZero）、NUL 文本（4 标量/9 字节/跨 NUL 拼接）、调用副作用序（marks 23 + mark_b→mark_a 序）。

### 动态对象/null（PLAN-964 投影）
- `A2xDyn` 共享值（Rc handle=AV 别名语义）：对象 map（`BTreeMap<A2xKey, A2xDyn>`）、共享列表（`Rc<RefCell<Vec>>`）。
- 身份键（Text/Int/Bool 互异）；运行时分派（map 键查找/列表 i32 索引）。
- 具名陷阱：`ObjectMissing`/`DynType`/`ObjectCycle`（对齐 AV av.dyn.*/av.trap.dyn-type 后缀；反环=插入可达性判定，对象/列表两侧）。
- `NilLit`/`DynCast{test_nil}`/`ObjectCtor/Get/Set`；dyn 相等；owned 文本读=拷贝。

### 模块/全局（981 模式）
- 绑定器全局图（init_order/module_init_fns）→ `static mut` 零值+init 链包装器前置；module.global 访问器→`GlobalRead/GlobalWrite`。标量域。

### 借用/move（A2R 具名拒）
- 普通面整体移动=纯局部绑定（HIR 无 Move 节点）→ owned 源局部杀；A2R 具名拒 `use-after-move`（mir.use-after-move 的 A2R 形式）；Assign 复活。
- MoveOwn 参数按值传递+消费过滤（wrap 释放+temp 名）；View+Variant 调用点克隆（源保活）。

### 错误/捕获（跨帧）
- `fail(msg)` 经帧错误槽（thread-local）跨帧携带消息。
- 受保护区（`Loop.unwind{kind,message}`）内 fallible 包 `WrapIntercept`（LangError+ListIndex+TextIndex→kind=4+取消息+break 至 catch 臂；其他 trap 照旧传播）；未捕获以消息重抛。

### 泛型/编译期/基础库
- 泛型：共享实例化 pass（`instantiate_auto_program`）；spec-methods 实例化后为具体 HIR 函数。
- 编译期：`#{...}` CT 服务 bind 期求值（MemoryEnvelopeSink+prepare_auto_ct），冻结常量入产物。
- 基础库：嫁接 stdlib（972 冻结文件）——`.to(int)` 转换+text-scan 族（find/trim/trim_len/upper）+List 模块，经 language 通道。
- metadata：`#[c, vm, rs]` 目标属性体。

### 降落细节
- 捕获闭包：Capture 参数=body 前置参数（Move=值转移/Borrow=不可变 Copy/str-view）；Shared（var-cell）具名拒。
- List 字段读=独占借用别名（`&mut`）；owned 文段读=克隆（部分移动安全）。
- OwnedClone：List 深拷贝/A2xDyn 深/Str 视图克隆/record 生成助手。
- 通道字节稳定：COLL_CORE 按 trap 枚举切分参数化重建（904 通道冻结字节不变）；语言陷阱+DYN_RT 仅 language 通道附加。

## 观察协议（a2xworker/1）

- stdout：精确一行 `value:<n>` 或 `trap:<Name>`。
- stderr：authorized marks（mark_a/mark_b 按执行序）。
- 前端拒绝：结构化 `a2x.source-diagnostic/1` 信封（stage/code/span）。

## 边界

- Shared（var-cell 写穿）捕获=具名债（"closure-env unit"）。
- dyn 深相等（列表逐元素/跨对象）只做标量/ptr。
- dyn→typed List 恢复、record/variant 进 dyn=具名拒。
- 多变体臂 is-链重检查消耗模型=深层缺口。
- 嵌套 dyn place 基（`m[0][1]` 型）=place_ref 限制。
- sched intrinsic（1092 async 行）归 1116 调度面。
- store 会话面（a2x-store Family 路由）归 1116 工程交付。
