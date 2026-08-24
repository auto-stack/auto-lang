# M4/S4 闸门目标格式:字节码结构对比(plan-432 S4 考古落盘)

基线 worktree `432-core-port-s2`;2026-08-24 实测黄金输出(Rust 管线
parse→codegen 脚本包装→HALT→单模块链接→反汇编)。

## 序列化格式

每行 `"%04x  {mnemonic} {operands}"`(操作数为空时尾随空格,两侧
trim_end 对比)。助记符与操作数形式对齐 vm/disasm.rs:
`fn.prolog args=N, locals=M` / `reserve N` / `jmp -> 0x%04x` /
`jmp.z -> 0x%04x` / `call 0x%04x` / `call.nat nat#N`(print=10)/
`const.i32 N` / `push.bool true|false` / `ret n_args=N` /
`load.str "<内容>"` / `load.loc.0|1|2 N` / `load.local N`(参数
128+rel 原样)/ `store.loc.0|1 N` / `store.local N` / `.line N` /
`add|sub|mul|div|mod|eq|ne|lt|gt|le|ge|and|or|not|str.cat|dup|pop|
push.nil|halt`。

## 规范化(计划允许的元数据差异)

1. **load.str**:Rust 侧显示池内容(`{:?}` 转义;corpus 限 ASCII 简单串)。
2. **槽释放组**(fn 体块/作用域弹出的连续 `push.nil+store*` 对):
   - Rust pop_scope 按 HashMap 迭代序发射,跨进程不定(实测 3 轮 3 种序);
   - `store.local` 是 2 字节,组内布局随序漂移;
   - Rust 侧 dumper 归一:组内按槽位升序、offset 按规范尺寸重算
     (push=1B + store[loc.0/1=1B,local=2B]),起始取组内最小原 offset。
   AAVM 侧天然按槽位升序发射,与归一形态一致。

## 发射模式考古(指令流对齐依据)

- 脚本 wrapper:`fn.prolog args=0, locals=16` + `reserve 16`;
- 每 fn 一个 jmp-over(patch 至 epilogue ret 后);fn 体 = `fn.prolog
  args=N locals=M`(M=max_locals-n_args)+ 可选 `reserve M` + 体 +
  体块弹出释放组 + `ret n_args=N`(显式 return 各自带 ret → 双 ret);
- `.line` 按语句首 token 行号,由语句步行者发射(循环回跳不含 .line);
- let:表达式 + store(无 dup/pop);赋值表达式语句:纯 `=` 只发 rhs,
  复合 `op=` 发 lhs-load+rhs+op,均接 dup/store/pop;
- if:每分支 cond+jmp.z→下一;体;jmp→end(含 return 后死 jmp);
- while:top(cond 处,语句 .line 之后)…body…jmp top;
- for-in-range:init+store;top:load var+end+lt+jmp.z;body;load var+
  const 1+add+store;jmp top;循环域弹出释放;
- 参数寻址 0x80+rel(load/store.local);局部 rel 0/1 快操作码
  (load 额外有 loc.2),≥2(N)走 store.local;
- print → `call.nat nat#10`;字符串 + → `str.cat`。

## S4 裁剪边界

语料 corpus_m4 十文件(hello/let/assign[纯+复合]/ifelse/while/for-range/
fib/strcat/logic[&&/||/!]/multilet[3 局部+释放组归一])。
Missing:数组/对象/元组/下标/方法调用 native/闭包/浮点/全局变量/顶层
非 fn 语句/一元负号/嵌套赋值/#[vm]/生成器/多模块。
