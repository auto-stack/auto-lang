# collections/1 类型化 MIR 与 Place 合同（PLAN-822）

> Status: experimental（实现于 auto-av-core `mir.rs`/`mir_verify.rs`/`ownership.rs`）。

av.collections/1 的 MIR 层合同：类型域、guard/payload 数据流格点、Place 参照与程序化 typed fixture 的未信任地位。

## 类型域（冻结）

- `MirType::List(ListTypeRef)`（wire tag 13）/ `MirType::Variant(VariantTypeRef)`（tag 14）/ `MirType::Place(ListTypeRef)`（tag 15，R3 起参照参数化——铸造/读写三指令全绑定元素类型，错型 typing 拒 `mir.list-type`）。
- 域检查：`mir.type.list-elem` / `mir.type.variant-payload` / 嵌套容器拒 / variant 无环（边集驱动，表序无关，自环与间接环双门拒；R9 移除 Record-only 前置过滤）；类型表 shape pass 拒越界参照 `mir.type-ref` 与 Place 自引用/容器参照 `mir.type.place-elem`（未使用表项同拒——类型表整体不可信）。

## guard/payload 数据流

- **guard 事实格点**：前驱 join 处 must-intersect（活借用态与 provenance 分离——provenance may-事实无条件合并驱动工作流收敛，R6）；PACK/SWITCH/可能写入调用后失效。块内线性语义为历史记录，非现行规则。
- **payload-moved**：may-union（moved 事实跨前驱并集）；投影后 drop 不复位。
- 局部根 place 的 Return 逃逸在 MIR 拒（`mir.borrow-escape`，含 Jump 边跨块形态）；record 字段域拒 token 类（`mir.field-borrow`）；Place param 为外部根（caller 帧持有期间可用可返回）。
- LHS reservation 语义沿 821（下标与 RHS 只求值一次）；Place 槽并入 token_slots 预算分类（每保留 roots 槽按 (n+1) bits 全额计，先于分配）。

## 程序化 typed fixture

builder 输出为公开未信任输入：仅经真 `verify_mir` 授信后编码/准入/执行；programmatic builder 不冒称 Checked。签名等值在 link 期经共享 scratch 类型空间内容比较（容器型接口变化结构性可捕）。

证据：`av_collections_822` MIR/ABC 反例族与 attachments/822 复审记录（R3-R9 类型/逃逸/预算/无环各轮）。

## 源码 lowering 约束（PLAN-864）

`auto.source.collections.draft` → 本 profile 的 lowering（`auto-mir-lower` collections 模式）**不新增指令**，
全部展开为既有 822 指令序列：

- `for-in` 降为显式索引循环（len 循环前读一次；`LIST_LEN` + `LIST_GET`；无迭代器对象）。
- `is` 降为受检分支链：每臂 `VARIANT_GUARD` → `Branch` → 臂内 `VARIANT_PROJECT`（guard 事实格点的
  must-fact 经分支边传播使正臂投影合法）；穷尽缺失在编译期拒。
- 变体槽赋值/绑定经 `VARIANT_SWITCH`（非裸 Move——旧活臂恰释放一次、guard 失效）。
- `clear()` 展开为尾删循环（`LIST_LEN`/`LIST_REMOVE`/`DROP`），无 LIST_CLEAR opcode。
- owned 元素投影：标量终场 `LIST_FIELD_GET`；owned Str 终场 `LIST_REMOVE → exact-type FieldGet → VIEW →
  STR_LEN → BorrowEnd → LIST_INSERT`（运行期 handle-share，无隐式深拷贝、无台账事件）。
- unit 效果语句（push/insert/set/clear 语句形态）不产生结果槽（Unit 永不入 profile 4 类型表）。
- 单元拆分（`split_units`）的类型闭包**必须含内嵌类型操作数**：Aggregate.record_ty、ListNew.list_ty、
  VariantPack.variant_ty、DynIs/DynTake.ty——820 起潜伏的悬空索引缺陷由 864 源单元浮出并修复。
