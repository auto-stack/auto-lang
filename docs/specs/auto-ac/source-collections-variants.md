# av.collections/1 源码域：List/变体源码面（PLAN-864）

> Status: experimental（source 支持；实现=`experimental/ace/crates/core/src/source/*` + `schema/collections.atom` + `experimental/ace/crates/lowering` collections 模式 + `experimental/ace/crates/av` 会话）。

`auto.source.collections.draft`（HirDomain::Collections，schema `auto.hir.collections.draft/1/collections-draft`）：
`av.collections/1`（profile 4）的源码域孪生。真实 `.at` 源中的 `List<T>`、`?T`/`Result<T,E>`、
带载荷有限 `enum`、`is` 分支与载荷绑定、`for x in <List>`、拥有型元素的受检只读 view，
经前端 → Checked HIR → 共享 lowering → **既有 822 执行链**（verify_mir → ABC → AV）全链执行。
**不新增执行语义、不改 profile 4 的已冻结指令/布局/预算**；新增的只有源码域 + lowering 覆盖 + 消费者绑定。

## 支持域（v1 冻结；合同 C2）

- 标量：`int`(i32)、`bool`、`Str`（owned；字面量 + `.len()` 的受限 text 面）。`str` 切片/`char` 具名拒绝。
- record：`type R { ... }`（字段 ∈ 标量/Str/record；禁容器字段、禁环）；record 构造（源码序求值）；字段路径读。
- `List<T>`：T ∈ {int, bool, Str, record, variant}（逐字对齐 822 element domain）；禁嵌 List。
- `?T` = 名义 Variant `Option<T>`；`Result<T,E>`；`enum E { A(T)? ... }`（有限臂、至多单载荷、非递归）。arm 声明序 = disc。
- **List 是函数局部值**（v1）：作参数/返回/record 字段具名拒绝（`source.list-value-context`）。

## 方法签名（源码形态 → 822 指令，合同 C3）

| 源码 | 语义 |
|---|---|
| `List<T>.new()` | `LIST_NEW{cap:4}`（恰好一次 buffer 分配入台账） |
| `push(v)` / `insert(i,v)` / `set(i,v)` | Copy 元素传值；owned 元素要求 `.move` 局部或新构 rvalue（`source.move-required`）；precheck→commit |
| `get(i)` | Copy 元素拷出；owned 元素 = 受检只读 view（下节） |
| `remove(i) → T` | owned 转移出 / Copy 拷出 |
| `len()` / `clear()` | len 直读；clear=lowering 展开尾删循环（无 LIST_CLEAR opcode，profile 4 不新增指令） |
| `Str`局部 `.clone()` | 合法源码显式深拷贝；List/Variant源码clone仍拒绝，typed容器Clone另验 |

## owned 元素受检只读 view（合同 C4）

`xs.get(i)` 在 owned 元素 List 上**不是值**：只允许作为字段投影根或 `.len()` 接收者，
逐使用点展开、从不物化——标量终场走 `LIST_FIELD_GET`；owned Str 终场走
`LIST_REMOVE → exact-type FieldGet（运行期 handle-share，无深拷贝、无台账事件）→ VIEW → STR_LEN → BorrowEnd → LIST_INSERT`
（元素原样归还、len 恢复、once-release 保持）。违禁使用编译拒（`source.view-escape`）。

## is / for-in（合同 C6/C7）

- `is <局部> { 臂(v)? -> {...} }`：穷尽 + 每臂至多一次（`source.pattern-non-exhaustive` / `source.pattern-duplicate` /
  `source.pattern-unknown-arm`；scrutinee 非变体局部 → `source.pattern-scrutinee`）；无 guard/通配。
  resolve 降级为 guard/branch 链（**每臂都受检**——投影需要 live guard fact；内层 else 为合成 `return <默认值>` 填充，
  静态不可达：穷尽性已编译校验，int/unit/Str 结果可构，其余结果具名拒绝 `source.is-tail-return`）。
  owned 载荷投影为 move（再投影静态拒）；Copy 载荷拷出。
- 变体槽赋值/绑定经 `VARIANT_SWITCH`（旧活臂恰释放一次、guard 事实随 switch 失效）。
- `for x in <List 局部>`：len **循环前读一次**；Copy 元素每轮物化拷贝绑定，owned 元素为投影上下文
  （无物化/move/clone）；循环体内对被迭代列表的结构修改（push/insert/remove/clear）编译拒
  `source.iterate-mutate`（v1 List 函数局部 ⇒ 词法闭包完备；set/字段写非结构性，允许）；break/continue 合法。

## SEM-02 与错误边界（合同 C5，冻结）

- **唯一索引陷阱码 `av.trap.list-index`**：负下标、≥len、空容器 get/remove/set、insert 越界、view 铸造越界——
  全部先于任何 buffer 访问 trap（822 `list_index_of` 拒负值）。insert 上界 len（闭区间）。无编译期下标检查。
- 旧 AutoVM 保留负索引（null-family）→ **profile 差异**（871 覆盖门登记），不静默统一，legacy 对 L06 类行不得冒绿。
- 变体错臂/已 move 臂投影：静态拒（`mir.variant.payload-moved`/guard-missing 以编译诊断浮现）；运行时防御 `av.trap.variant-branch`。
- 资源失败边界沿 822：变异指令 precheck→commit，预算拒先于任何变异；帧退出由 `release_frame_owned_lifo`
  对 Str/Record/List/Variant 槽 LIFO 精确释放一次（lowering 不排程 per-exit drop）。

## 增量（合同 C8）

函数级单元 + 每函数规范内容 fp（`CollectionsSourceSession` = 822 `CollectionsUpdateSession` 同型扩源，
非平行会话）：未改函数字节不变重提交、走 carry 路径、production counters（verified/encoded/admitted/store_writes）= 0
——**未改函数生产工作=0 由计数器实证**。body 编辑精确重降该函数；record 字段接口编辑使全部构造方精确失效。
restart 经全门重准入（decode+verify，不信任字节）。单元类型闭包含内嵌类型操作数
（Aggregate.record_ty / ListNew.list_ty / VariantPack.variant_ty / Dyn*）。

## 诊断码（新增，合同 C10）

`source.pattern-non-exhaustive` / `source.pattern-duplicate` / `source.pattern-unknown-arm` /
`source.pattern-scrutinee` / `source.view-escape` / `source.iterate-mutate` / `source.move-required` /
`source.list-value-context` / `source.type-ambiguous` / `source.is-tail-return`。
既有码沿用：`source.lex/syntax/undefined/unsupported/duplicate/type-mismatch/argument`。

## 验证锚

`cargo test --manifest-path experimental/ace/crates/core/Cargo.toml --test collections_864 --test collections_864_rejects`
（3 前端编译 + 13 具名拒绝）；
`cargo test --manifest-path experimental/ace/crates/av/Cargo.toml --test av_collections_864 --test av_collections_864_corpus --test av_collections_864_session`
（4 全链 + 16 语料 + 3 会话计数器）；
文件化 17 行 `test/cases/ace-m1/collections-source-864.suite.ad`（经 `ac-av/collections-source-run-v1` 真 worker，
parent 判定）；`scripts/verify-av-864.ps1 -Scope acceptance`（含 852 基套件 97 检查零漂移 compat）。
touched_goals 为空（有限实验扩面）。


## PLAN-889：名义类型查找与原生/单体消费者

局部、结果、载荷与List元素允许的命名enum沿同一名义图解析，前向/无环嵌套按依赖序处理；
环及原上下文排除仍具名拒绝。修正此前仅查record导致合法List<Variant>误拒的实现，
不新增源码List.clone/Variant.clone。Str.clone源码与受检typed容器Clone的入口、证据分列。

`CheckedEnv`保留类型/签名；fragment只词法/解析目标UTF8函数，候选将新body与body-local intern池
共同临时检查、还原/提交，失败/取消/foreign/stale/close保旧或具名拒绝。borrowed function verifier
执行原完整规则，冷批只复用接口scaffold；不授予whole-module credential。
真正source→native增量由`CollectionsIncrementalSession`消费，源坐标区分mint/current与snapshot，
未改AST/MIR/COFF分配身份保持。旧AV CollectionSourceSession仍为whole-source重编，
不能据native交付冒称其前端0工作或native在线HMR。见
[原生合同](../auto-mir/native-collections-variants.md)与[驻留消费](../auto-av/resident-function-install.md)。
