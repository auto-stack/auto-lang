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
  **相邻同线去重**(镜像宿主 `emit_source_line` 的 `current_source_line`
  状态机:aavm 侧 `CG.cur_line`,`line>0 且 ≠当前已发行` 才发;Plan 495
  定案 rust 为规范);**is 单表达式/单跳转 arm 体**按臂体首 token 行经
  同款去重发射(镜像宿主 `parse_expr_or_body` 的 `stmt_line`;块体 arm
  归语句步行)。已知未对齐边界见 KNOWN-DEBT P495-2(块体 arm 作用域);
- let:表达式 + store(无 dup/pop);赋值表达式语句:纯 `=` 只发 rhs,
  复合 `op=` 发 lhs-load+rhs+op,均接 dup/store/pop;
- if:每分支 cond+jmp.z→下一;体;jmp→end(含 return 后死 jmp);
- while:top(cond 处,语句 .line 之后)…body…jmp top;
- for-in-range:init+store;top:load var+end+lt+jmp.z;body;load var+
  const 1+add+store;jmp top;循环域弹出释放;
- 参数寻址 0x80+rel(load/store.local);局部 rel 0/1 快操作码
  (load 额外有 loc.2),≥2(N)走 store.local;
- print → `call.nat nat#10`;字符串 + → `str.cat`;
- [M4 扩]数组字面量:elems 依次 + `create.arr count=N`(1B 操作数,
  disasm 作 `count=N`;codegen.rs 4865);下标读:arr+idx+`get.elem`
  (0 操作数);下标写(quick-fix 栈序,5954):rhs+arr+idx+
  `set.elem`(void,语句级不发 pop;value 展栈底);
- [M4 扩]数组 `.len()`:receiver+`arr.len`(0 操作数;Rust 闸:
  infer_object_type==Array 且无实参,7177;str.len 走 CALL_NAT 不在此);
- [M4 扩]一元负号:operand+`neg`(int 路径,6414;一元 + 为 no-op);
- [M4 扩]顶层非 fn 语句:wrapper 内按源序步行,**不发 .line**
  (compile_and_link 的 other_stmts 直走 compile_stmt,行号发射属
  fn 体路径)。

## S4 裁剪边界

语料 corpus_m4 十文件(hello/let/assign[纯+复合]/ifelse/while/for-range/
fib/strcat/logic[&&/||/!]/multilet[3 局部+释放组归一])。
[M4 扩]新增 20 文件:b11-b26(99_bootstrap 038-052+051a 的 run_eval
内层程序回收,顶层语句形态)+ b27-b30(06_arrays 001-003 同源构造
+组合:字面量/下标含负/更新/循环累加)。053(List/Map native 建节点)
超 v2 范围未回收。
Missing:对象/元组字面量/方法调用 CALL_NAT 家族/字符串下标(码点)/
闭包/浮点/全局变量/下标复合赋值/嵌套赋值/#[vm]/生成器/多模块。
