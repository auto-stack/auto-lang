# ACE 组合数据结构与递归拥有资源

> PLAN-962；实现位于 `experimental/ace/grammars/auto-source.at`（aggregates profile 与聚合语法）、
> `core/src/source/cst/{project,kinds}.rs`（投影/原子）、`core/src/source/{ast,resolve,typecheck,adapter}.rs`
> （结构形状/脱糖/物化）、`core/src/verify.rs`（环检查）与 `collections_facts.rs`、
> `lowering/src/hir_adapter.rs`（不透明门控两阶段内联）、`av-core`（PLACE 视图读准入）、
> `native/src/{driver,collections_lower}.rs`（升梯/collections 发射）、`a2x-core/src/semantic_view.rs`
> （capability 区分+collections 族准入）、`a2x-c/src/seed*.rs`（位置字段名）、`a2x-store`（profile/族）。
> 本文件只记录**已验证**的支持、接口、拒绝与边界；未列入的能力不视为已支持。
> 既有 collections/closures/numeric 冻结面字节不变（822 一层元素域对旧面恒生效）。语义基线为 959
> 分母 L-06..L-19/L-48 与本包 T-01 冻结表（计划 §9 收据），四端同一矩阵（AV/native/a2x-rust/a2x-c）。

## 支持入口与身份

聚合面经 `SourceProfile::Aggregates` 显式进入：grammar `aggregates` 裁剪（numeric 全表面超集；
冻结面 scanner/裁剪字节恒等）。**准入键 = 文档级 capability `av.aggregates/1`**（emitter 盖章 →
MIR capabilities → mir_verify 白名单 → native/a2x 同键放宽）——旧面严格性逐字节保持。消费入口
`ace_core::source::compile_aggregates_source`（verify/lower 走 closures 族）；native driver 升梯
Aggregates 末位；a2x-store Aggregates profile → collections 族。

## 分母行支持（L-06..L-19/L-48）

| 行 | 支持 | 表示 |
|---|---|---|
| L-06 `.view/.mut/.move` | ✓ | `.view`=只读借用标记（record 参数默认即借用，771 语义；record 基检查）；`.mut`/`.move` 既有 |
| L-07 Hold | ✓ | `let hold x as v { }` 整局部形（v=块内名别名，块末失效；≥旧 MVP 等义——旧引擎自注无借用纪律）。裸 `hold` 拼写=具名待办（D-962-C1 同根因） |
| L-08 Box | ✓ | `Box<T>`=单元素 List 胞；`let b = Box(v)` let 位构造；**递归合法化**：`Node{next Box<Node>}` 经不透明边（List 字段）成合法递归，直 `next Node` 拒（环检查不含 List 边） |
| L-09 take | ✓ | `take(x)` ≡ Move 弃用别名（r-take 专用原子形；用户同名 fn 优先） |
| L-10 R01/03/04/07 | ✓ | 值拷贝/命名 ctor 乱序/深克隆/mut 写回（既有 record 域机器，聚合面全四行验证） |
| L-11 enum 三形 | 部分 | 载荷臂+unit 臂（既有）；scalar 显式判别值/homo 糖未加 |
| L-12 union | ✓ | tagged 安全联合（一字段一臂；非活动臂=is 守卫不匹配；旧 parser-only 故语义新定） |
| L-13 tag | ✓ | enum 声明别名（上下文关键字） |
| L-14 grid | ✓ | `grid (c1,c2){[r..];..}` let 位脱糖列元组（每列 List+静态维检查：行等长/列名唯一/列内同型）；grid-kw 聚合面上下文关键字 |
| L-15 tuple | ✓ | `(T1,T2)`/`(e1,e2)`/`let (a,b)=e`——**零 schema 新面**：VT 结构内联（CollTables::tuples）+ adapter 物化匿名位置 record（`t_rec___tuple_<i>`，字段 "0".."n-1"） |
| L-16 array/List 字面量 | ✓ | `[e,...]` let 位脱糖 ListNew+Push；嵌套 List 准入（capability 键控）；空表=语法层拒；整元素读=**视图**（LIST_PLACE+PLACE_GET，物化全拒 `source.view-escape`） |
| L-17 Option/Result | ✓ | 既有+载荷域随元素域解锁（`List<Option<T>>` 等） |
| L-18 模式 | 部分 | is 臂单名+载荷（既有）；StructCover 嵌套/is 表达式→965 |
| L-19 range | ✓ | `for i in a .. b`/`..=b` 计数循环脱糖（**界需空格**：affix-num 最长匹配，具名边界）；值位 range 拒 |
| L-48 索引 place | ✓ | `xs[i]=v`（list-set 全合同：类型/视图拒/新右值转移/迭代中守卫）；位置式**读**=`.get(i)`（具名裁定 D-962-C1）；`xs[i].f=v` 嵌套写经 remove/view/read/reinsert 序列（既有） |

Map/Set：旧语言无此形（L-46 防凭空）；object 动态字典→964、typed 容器→946/972（经 963）。

## 资源与所有权

整元素视图（ElemView 空路径）为污染值：仅读穿透合法；let/assign/实参/push/return 物化全拒
（`source.view-escape`）。alias 临时槽不入事实释放表（属主 list 的释放级联）。递归经不透明边（List/Box
字段=固定大小句柄）合法；`record_reaches_self_through_opaque` 谓词门控 lowering 两阶段内联（非环模块
plain 路径字节恒等）。环检查（`verify.record-cycle`）对 collections 族全生效（V-04 缺口闭合）。

## 拒绝与边界（具名诊断）

`source.view-escape`（视图物化）、`source.grid-dims`（行≠列）、`source.grid-cols`（列重复/空）、
`source.grid-position`/`source.array-lit-position`/`source.range-position`（位置纪律）、
`source.hold-base`（非 record 基）、`verify.record-cycle`（直递归）。已知拼写债：D-962-C1..C5。

## 四端证据

AV：`aggregates_v_rows` 8 fixture + `aggregates_t01_probe` 21 腿（真全链+资源账）。native：arena-free
组合形真 PE 执行（mn driver）+ arena 形 admit/COFF 发射（collections 车道；递归 Box=D-962-C2）。
a2x-rust：4 案例真实 rustc（tuple/解构/参数/命名 ctor/range）。a2x-c：3 案例真实 MSVC cl（tuple 调用
实参=D-962-C3）。六职责组零代码红（`t04-six-groups.log`/`t04-rerun-native-a2x.log`，具名环境红归因）。
