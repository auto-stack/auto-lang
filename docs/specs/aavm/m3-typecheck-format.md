# M3/S3 闸门目标格式:类型层一致性(plan-432 S3 前置考古)

基线 worktree `432-core-port-s2`;2026-08-24 考古结论 + 实证 ground truth。

## 考古结论(Rust 侧可观察的类型层输出)

解释器路径(execute_autovm:parse→CTEE→codegen)**没有独立 typecheck pass**;
类型层的三个可观察通道:

1. **解析期 TypeError**(check_field_type / TraitChecker / spec 检查,汇入
   MultipleErrors 随 parse 失败)——归 M2 域(解析错误),且 `.expected.error`
   harness 只验存在不比文本;
2. **let 无注解时的 parser 内联推断**(parse_store_stmt→infer_type_expr)——
   已进 M2 的 `(type T)` dump 通道(S2 完成);
3. **`.type` 行为输出**(codegen 的 .type 属性 → infer_expr_type → infer_expr)
   —— infer_tests.rs 16 用例的通道,**含 fn 返回传播**(parse 内联通道不做),
   是 S3 的天然锚点。

infer/stmt.rs 的 check_body 仅 a2r 调用且结果被丢弃;ParamChecker 零调用者
(死代码);TypeStore 无 Display/序列化。故 S3 闸门 = **通道 3**:
语料程序本身打印 `.type`,Rust 侧真执行取 stdout,AAVM 侧 typeinfo.at
静态推理产出相同行。

## 闸门协议(M3)

- 语料:`test/vm/aavm2/corpus_m3/*.at` —— 可执行程序,查询语句形如
  `print(EXPR.type)`,EXPR 为:字面量 / let 绑定标识 / `ident[下标]` /
  函数调用经 let 绑定。初始化式覆盖:字面量、字面量二元运算、调用、
  数组字面量、标识符。
- Rust 侧:`run_with_capture(语料)` 的 stdout(真 VM + 真 infer_expr_type)。
- AAVM 侧:`auto/lib/typeinfo.at` 的 `typecheck_dump(source) -> str`——
  复用 lexer.at 的 tokenize 与 parser.at 的底层游标/类型解析助手
  (p_kind/p_next/parse_type/infix 表),自带:
  - **TFnSig 注册表**(fn 名→返回类型 Display 串;参数类型经 p_bind 入作用域)
    ——"单一 TypeStore"的 S3 裁剪(类型/spec 声明表留 S4 按需);
  - **infer 传播**:字面量/标识符作用域查找/调用→注册表返回类型/
    数组元素提取/二元运算(unify 简式:算术→操作数统一型,比较→bool)/
    元组连接;
  - 未知类型输出 `unknown`(对齐 Type::Unknown 的 unique_name)。
- 判据:两侧逐行相等(diff=0)。

## Ground truth(2026-08-24 实测,corpus_m3 六文件)

| 文件 | 输出 |
|---|---|
| t01_literals | int / float / str / bool / char |
| t02_vars | int / float / str / bool |
| t03_fnret | int / str |
| t04_fnparam | int |
| t05_arrays | int / str |
| t06_binops | int / int / bool |

## Missing(登记于 typeinfo.at 头)

- 混合算术(1 + 1.5)的 coercion 语义(Rust unify_with_coercion 的数值提升
  分支未考古,语料未含);`let` 显式注解与推断的冲突检查;块级作用域遮蔽
  (fn 体扁平遍历);对象/下标链/方法调用推断;类型错误样例通道
  (无独立 Rust typeck pass 可镜像,见考古结论 1)。
