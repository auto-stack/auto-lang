# aavm

> **Status**: experimental
> 备注：v2；Plan 429-434 系列收口
> 路径:`auto/lib`(v1 已封存于 `auto/lib-legacy/`)| 转译:`auto build`(pac.at)/ `auto trans`

Auto 自举编译器实验(AAVM v2):用 Auto 语言写的编译器前端 + 字节码 VM +
Rust 转译器(AA2R),验证 Auto 的自举能力。Plan 432 起 v2 六文件按依赖序
重写;Plan 434 增 a2r.at(AA2R)完成终极自举闭环。

## 目标与范围

- 用 Auto 实现 Auto 子集的完整编译链:token → lexer → parser(S-expr dump
  判据层)→ typeinfo → codegen → engine(栈式 VM,`ev_run` 入口)。
- Plan 434(AA2R):a2r 核心子集的 Auto 版 —— **Auto 写的 a2r 转译 Auto
  写的 AutoVM,产物是可独立编译的 Rust**;自举回路中不再有任何 Rust
  手写的编译组件。
- 不做:不追求与主编译器(crates/auto-lang)特性对齐;实验性质,不作为
  生产编译路径。多目标(c/py/js)/r2a/逃逸分析完整版明确不做(Plan 434
  Out of Scope)。

## 模块架构(v2,依赖序 = AUTO_LIB_FILES_V2 单一事实源)

```text
token.at ── lexer.at ── parser.at ── typeinfo.at ── codegen.at ── engine.at
   │           │            │                          │            │
 TokenKind   tokenize    parse_dump(S-expr)        cg_compile    ev_run
             lex_dump    + 434 扩展:泛型实例/       codegen_dump   (栈式 VM)
                         type-decl/enum-decl
                                                        │
                                                     a2r.at(434)
                                                        │
                                        aa2r_transpile / aa2r_transpile_merge
                                        (AA2R:token 游标直走,D39)
```

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| lib/token.at | TokenKind 139 变体 + keyword_kind/kind_name | 432 完结(M1) |
| lib/lexer.at | tokenize + dump;434 增 f-string/三引号(D38c) | 432 完结(M1)+434 |
| lib/parser.at | parse_dump S-expr 直出;434 增泛型实例/type-decl/enum-decl(D38a/b) | 432 完结(M2)+434 |
| lib/typeinfo.at | typecheck_dump(.type 推断层) | 432 完结(M3) |
| lib/codegen.at | cg_compile 字节码(I{op,s,n} 载体) | 432 完结(M4) |
| lib/engine.at | ev_run 栈式 VM(Val 判别结构/数组 arena) | 432 完结(M5) |
| lib/a2r.at | AA2R:主 a2r 核心子集的 Auto 版(Plan 434) | 434(见其文件头 Snapshot) |
| pac.at | 包定义(`auto build` 转译入口) | experimental |

## 判据与闸门

- M1-M5(corpus_m1..m4,`cargo test -p auto-lang --lib --features
  test-vm-files -- test_aavm2 --include-ignored`):与 Rust 参考逐字符一致。
  `.line` 发射语义已定案(Plan 495,2026-08-31:P485-2 清偿——rust 为规范:
  语句边界+同线去重+is 单表达式 arm 体行发射;b14_line_dedup 回归钉;
  规格 `design/m4-bytecode-format.md` §发射模式考古)。
- 五方对比矩阵(Plan 433 四方 + 434 ⑤ aa2r;`parity/` 下
  `cargo run -- --root . --auto-binary ../target/debug/auto.exe aavm`):
  ① reference ② aavm_rust(六文件,见 divergences.md D38 主 a2r 缺口注)
  ③ aavm_vm ④ golden ⑤ aa2r —— 稳定集 corpus_m4 全绿。
- divergence 登记簿:docs/specs/aavm/design/divergences.md(判定规则见
  divergence-rules.md)。
- 风格升级(类 C → 一对一 Rust 对译)前提条件调研:
  idiom-upgrade-prereqs.md(2026-08-25;实证矩阵 + 宿主修复清单 H1-H6 +
  AA2R 扩展 W1-W6 + lib 改写点位与波次建议);立项(三份合并为一,顺序推进):
  [Plan 447](../../plans/archive/447-aavm-prerequisites.md)(aavm-prerequisites:
  ① 宿主加固 → ② aavm 新语法能力 → ③ lib 风格升级)。
