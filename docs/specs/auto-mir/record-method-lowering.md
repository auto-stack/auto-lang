# record-method-lowering

PLAN-881 冻结的 record/text-record 类型绑定方法（static/只读 view）共享
lowering 合同：唯一映射、单元身份、准入面与资源语义。

## 唯一映射与身份

- `DefKind::Method`（static / `ReceiverKind::View`）经唯一
  `experimental/ace/crates/lowering` 降为**普通 MIR 函数/调用**，AV 与 mn 同源
  消费；不建第二方法 runtime。View receiver = HIR 签名 `params[0]`（只读
  记录视图，调用端 receiver 求值恰一次且先于显式实参）；Static 方法无
  receiver。
- 方法 MIR/单元身份 = **`m.<Owner>.<name>`**（owner 基命名；`.` 不出现在
  源标识符中故映射注入，与自由函数 `d_<name>` 前缀不碰撞；与 fi 索引的
  HIR def id `d_f<fi>_<name>` 解耦——无关声明插入/重排不 churn 单元与
  cache 身份）。跨函数调用导入符号 `fn:m.<Owner>.<name>`；同名方法/同名
  自由函数不混链。
- 准入面：`lower_checked_data` / `lower_checked_owned` 及数据、owned 单元
  路径放开方法；scalar（`lower_checked`）与 collections（864 域）保持
  `lower.method` 受控拒绝（唯一 lowering 守卫）。未调用的合法方法不再使
  整模块 `lower.method` 拒绝。
- 显式参数模式：方法的 mut/move 显式参数不退化（`owned_fn_sig_for_def`
  与 `callee_mut_record_params` 的模式提取覆盖 `Method`）；`t_MutRet_*`
  包装名随 owner 基 MIR 名。

## mut-record 写回 ABI（按值）

- av.data/1：mut record 参数把被调结果包装为 `t_MutRet_<fn>`（`f0`=源结
  果，`wb*`=各 mut 参数终值）；借用临时与写回均为 **Copy**（纯值语义；
  排他为源级 `verify.mut-conflict` 合同）。owned 域 copyable record 的
  `.mut` 同为 Copy 表示；资源 record（可达 `Str`）保持 Move 传递对。
- 单元路径：`lower_function_data_unit_with_modes` 携保留模式事实（单体文
  档的 body-less scaffold 无法在类型节表达 mut 模式——沿 828 R3-F07 约定，
  binder 对 body-less 方法声明取 `u32::MAX` sentinel；整模块 verify 仍要求
  每 def 实体）。

## 资源与语义保持

- 771/782 名义类型、只读/互斥借用、显式参数模式、move/clone、拥有返回与
  清理、源码诊断全套保留（782 p01=30、Token=57、LIT/CLONE 各一次 free、
  normal live0 台账实测）。
- View receiver 只读借用不复制 owner；显式 clone 才复制；分配失败注入
  （`StrArena::set_fault_at`）下部分输出回滚、账目平衡。
- 递归（static/view，带终止条件）与方法↔自由函数互调经真实单元闭包链接。

## 消费者

- AV：av.data/1（data_driver/会话）与 av.owned/1（owned_driver/会话）整
  链；方法为会话一级单元公民（owner 基键、依赖/签名/模式事实、单体前端
  编辑——见 resident-function-install）。
- mn：escalating frontend → 共享 lower → verify → split → CLIF/COFF → PE，
  无 Method 感知；RecordView 基座的字段路径布局适配见
  native-data-owned-lowering。
- 语言语义来源：source-record / source-text-record（不改）；方法 MIR 面
  `MIR_LOWERING_VERSION = "2"`（见 hir-lowering-boundary）。
