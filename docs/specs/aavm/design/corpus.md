# AAVM v2 分层 corpus（plan-431 Phase D）

## 1. 分层语料（D1）

| 层 | 语料 | 一致性判据 | 备注 |
|---|---|---|---|
| lexer | `test/vm/` + `test/a2r/` 全部 `.at` 源文件做 tokenize 输入 | token 流对比 | 序列化格式:`(kind, lexeme, line, col)` 行式 dump,Rust 侧 debug 打印即可(无稳定性承诺);对比 runner 挂 432 基建 |
| parser | 同上语料 | AST dump 对比 | 复用 `.expected.rust_ast` 思路(99_bootstrap Phase 2 机制) |
| 执行 | `99_bootstrap` 038-053 求值类 + `test/vm/` 行为用例子集 | stdout diff(`.expected.out` 断言语料回收,`auto/lib-legacy/` 归档保留) | |
| 终局 | `parity/libs/` 非 UI 库(如 string_utils) | a2r 编译产物运行 diff | 433 四向对比主战场;骨架已就位(test_aavm2_compile) |

## 2. 性能预算（D2，依据 429-B2）

429-B2 实测:VM 解释 ~7µs/迭代,dispatch 与内建 opcode 差距 ~1µs。据此:

- 单用例 VM 解释执行时限:**10s**(默认 cargo test 无超时,runner 内以
  瞬时检查实现——432 runner 落地);
- 超限语料标记 `slow.corpus`(清单内注明),仅参与 AAVM→a2r→Rust 编译路径,
  不进 VM 解释路径;
- lexer/parser 层 tokenize/dump 语料无预算约束(无执行)。

## 3. corpus 登记方式

新语料放 `test/vm/aavm2/<NNN_name>/`(沿用全库命名约定:`name.at` +
`name.expected.out`);跨层复用同一 `.at`,期望文件按层后缀区分
(`.expected.tokens` / `.expected.rust_ast` / `.expected.out`)。
