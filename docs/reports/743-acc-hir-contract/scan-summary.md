# PLAN-743 盘点扫描摘要（工具生成，勿手改）

性质：词法观察（compiler-source-demand 证据），非语义结论、非支持声明。

## 输入覆盖
- `auto/lib/token.at` `963ccc1df73b86e5`
- `auto/lib/lexer.at` `c92bcce8c6432ffd`
- `auto/lib/parser.at` `377129b9b06afec7`
- `auto/lib/typeinfo.at` `9eed6f5f243225c1`
- `auto/lib/codegen.at` `a95a484913b95084`
- `auto/lib/engine.at` `d8db6813edf82e0d`
- `auto/lib/a2r.at` `e3eddb46ab2e5d3e`
- `auto/aavm.at` `0ff06f099253e1aa`
- `crates/auto-lang/src/lib.rs` `4d6c4703c667f99c`

## 注册表 AUTO_LIB_FILES_V2
- 顺序：token → lexer → parser → typeinfo → codegen → engine → a2r
- 与工具合同一致：是

## 模块观察

### `auto/lib/token.at`
- 声明 3（方法 0）；use 边 0；调用候选 0（unknown 0）
- 能力证据：enum-declaration×1, free-function×2, if-else×2, is-match-statement×2, string-literal×199, string-type-annotation×2

### `auto/lib/lexer.at`
- 声明 15（方法 0）；use 边 1；调用候选 35（unknown 0）
- 能力证据：boolean-type-annotation×4, char-literal×81, comparison-operator×60, free-function×13, generic-list-type×3, if-else×95, integer-type-annotation×15, is-match-statement×5, logical-operator×23, mut-parameter×1, plain-list-value×1, record-type-declaration×2, return-statement×14, string-literal×71, string-type-annotation×12, use-module×1, while-loop×10

### `auto/lib/parser.at`
- 声明 64（方法 15）；use 边 2；调用候选 157（unknown 15）
- 能力证据：boolean-type-annotation×6, char-literal×8, comparison-operator×298, enum-declaration×1, free-function×75, generic-list-type×25, if-else×364, integer-type-annotation×16, is-match-statement×14, logical-operator×64, mut-parameter×35, plain-list-value×16, record-type-declaration×3, return-statement×229, string-literal×400, string-type-annotation×76, type-body-method×15, use-module×2, while-loop×59
- unknown 候选（交人工核对，逐条列出）：
  - p.bind(unknown-receiver) 首行 1402 ×8
  - p.decl_lookup(unknown-receiver) 首行 420 ×4
  - p.decl_register(unknown-receiver) 首行 1802 ×4
  - p.expect(unknown-receiver) 首行 453 ×48
  - p.fail(unknown-receiver) 首行 449 ×63
  - p.kind(unknown-receiver) 首行 376 ×161
  - p.lookup(unknown-receiver) 首行 804 ×3
  - p.next(unknown-receiver) 首行 390 ×145
  - self.next(unknown-receiver) 首行 161 ×1
  - p.peek(unknown-receiver) 首行 1109 ×3
  - p.peek_text(unknown-receiver) 首行 1692 ×1
  - p.pop_scope(unknown-receiver) 首行 1521 ×6
  - p.push_scope(unknown-receiver) 首行 1493 ×7
  - p.skip_empty_lines(unknown-receiver) 首行 434 ×47
  - p.text(unknown-receiver) 首行 414 ×45

### `auto/lib/typeinfo.at`
- 声明 18（方法 0）；use 边 3；调用候选 71（unknown 13）
- 能力证据：boolean-type-annotation×1, char-literal×4, comparison-operator×89, free-function×17, generic-list-type×13, if-else×95, integer-type-annotation×2, is-match-statement×5, logical-operator×34, mut-parameter×8, plain-list-value×3, record-type-declaration×1, return-statement×36, string-literal×77, string-type-annotation×20, use-module×3, while-loop×20
- unknown 候选（交人工核对，逐条列出）：
  - p.bind(unknown-receiver) 首行 87 ×3
  - p.decl_lookup(unknown-receiver) 首行 245 ×3
  - p.expect(unknown-receiver) 首行 62 ×16
  - p.fail(unknown-receiver) 首行 52 ×11
  - p.kind(unknown-receiver) 首行 51 ×62
  - p.lookup(unknown-receiver) 首行 471 ×1
  - p.next(unknown-receiver) 首行 50 ×41
  - p.peek(unknown-receiver) 首行 143 ×2
  - p.peek_text(unknown-receiver) 首行 147 ×1
  - p.pop_scope(unknown-receiver) 首行 110 ×3
  - p.push_scope(unknown-receiver) 首行 61 ×2
  - p.skip_empty_lines(unknown-receiver) 首行 64 ×17
  - p.text(unknown-receiver) 首行 59 ×6

### `auto/lib/codegen.at`
- 声明 61（方法 34）；use 边 4；调用候选 361（unknown 51）
- 能力证据：boolean-type-annotation×6, char-literal×5, comparison-operator×400, enum-declaration×1, free-function×82, generic-list-type×72, if-else×400, integer-type-annotation×68, is-match-statement×8, logical-operator×179, mut-parameter×26, plain-list-value×50, record-type-declaration×11, return-statement×185, string-literal×400, string-type-annotation×127, type-body-method×34, use-module×4, while-loop×153
- unknown 候选（交人工核对，逐条列出）：
  - c.add_var(unknown-receiver) 首行 1452 ×16
  - self.add_var(unknown-receiver) 首行 560 ×1
  - c.add_var_reuse(unknown-receiver) 首行 3117 ×1
  - c.const_lookup(unknown-receiver) 首行 1590 ×3
  - c.ctor_lookup(unknown-receiver) 首行 1590 ×3
  - c.emit(unknown-receiver) 首行 1158 ×151
  - self.emit(unknown-receiver) 首行 403 ×12
  - c.emit_load(unknown-receiver) 首行 1293 ×32
  - c.emit_store(unknown-receiver) 首行 2735 ×17
  - self.emit_store(unknown-receiver) 首行 475 ×1
  - self.esc_disasm(unknown-receiver) 首行 797 ×3
  - p.expect(unknown-receiver) 首行 1381 ×30
  - c.fail(unknown-receiver) 首行 1175 ×50
  - self.fail(unknown-receiver) 首行 379 ×2
  - c.field_idx(unknown-receiver) 首行 2158 ×2
  - c.field_ty(unknown-receiver) 首行 1267 ×4
  - c.gkey(unknown-receiver) 首行 1761 ×6
  - c.import_symbol(unknown-receiver) 首行 1702 ×1
  - c.is_global(unknown-receiver) 首行 1761 ×4
  - c.is_module(unknown-receiver) 首行 1659 ×1
  - p.kind(unknown-receiver) 首行 1362 ×162
  - pp.kind(unknown-receiver) 首行 4047 ×2
  - c.line(unknown-receiver) 首行 2400 ×2
  - p.line(unknown-receiver) 首行 2400 ×2
  - m.link_symbol(unknown-receiver) 首行 4517 ×1
  - c.lookup(unknown-receiver) 首行 1288 ×14
  - c.loop_enter(unknown-receiver) 首行 3588 ×4
  - c.loop_exit(unknown-receiver) 首行 3601 ×4
  - c.loop_jump(unknown-receiver) 首行 3009 ×1
  - self.max_alive_idx(unknown-receiver) 首行 488 ×2
  - p.next(unknown-receiver) 首行 1139 ×133
  - pp.next(unknown-receiver) 首行 4051 ×6
  - p.peek(unknown-receiver) 首行 1508 ×20
  - pp.peek(unknown-receiver) 首行 4072 ×1
  - p.peek_text(unknown-receiver) 首行 1588 ×6
  - pp.peek_text(unknown-receiver) 首行 4065 ×4
  - c.pool_add(unknown-receiver) 首行 1157 ×11
  - c.pool_push(unknown-receiver) 首行 2125 ×6
  - c.pop_scope(unknown-receiver) 首行 1466 ×11
  - c.push_scope(unknown-receiver) 首行 1445 ×7
  - c.serialize(unknown-receiver) 首行 4759 ×2
  - p.skip_empty_lines(unknown-receiver) 首行 1361 ×58
  - pp.skip_empty_lines(unknown-receiver) 首行 4044 ×1
  - p.text(unknown-receiver) 首行 1140 ×38
  - pp.text(unknown-receiver) 首行 4072 ×1
  - c.tys_lookup(unknown-receiver) 首行 1267 ×9
  - self.tys_lookup(unknown-receiver) 首行 255 ×2
  - c.var_arr(unknown-receiver) 首行 1772 ×1
  - c.var_gtor(unknown-receiver) 首行 3312 ×1
  - c.var_ty(unknown-receiver) 首行 1231 ×7
  - c.var_ty2(unknown-receiver) 首行 1223 ×1

### `auto/lib/engine.at`
- 声明 21（方法 0）；use 边 2；调用候选 99（unknown 7）
- 异常：[{'line': 314, 'kind': 'unterminated-string'}, {'line': 315, 'kind': 'unterminated-string'}, {'line': 333, 'kind': 'unterminated-string'}, {'line': 334, 'kind': 'unterminated-string'}]；括号余额 0
- 能力证据：boolean-type-annotation×4, char-literal×2, comparison-operator×62, enum-declaration×1, free-function×20, generic-list-type×25, if-else×120, integer-type-annotation×16, is-match-statement×24, logical-operator×10, multiline-string-literal×4, mut-parameter×5, plain-list-value×15, return-statement×35, string-literal×33, string-type-annotation×14, use-module×2, while-loop×15
- unknown 候选（交人工核对，逐条列出）：
  - VArr(unknown-bare-call) 首行 59 ×1
  - VBool(unknown-bare-call) 首行 58 ×1
  - VClo(unknown-bare-call) 首行 63 ×1
  - VInst(unknown-bare-call) 首行 60 ×1
  - VInt(unknown-bare-call) 首行 56 ×1
  - VStr(unknown-bare-call) 首行 57 ×1
  - c.field_idx(unknown-receiver) 首行 586 ×2

### `auto/lib/a2r.at`
- 声明 98（方法 25）；use 边 5；调用候选 306（unknown 33）
- 能力证据：boolean-type-annotation×13, char-literal×14, comparison-operator×400, free-function×114, generic-list-type×73, if-else×400, integer-type-annotation×66, is-match-statement×13, logical-operator×214, mut-parameter×55, plain-list-value×46, record-type-declaration×8, return-statement×252, string-literal×400, string-type-annotation×141, type-body-method×25, use-module×5, while-loop×136
- unknown 候选（交人工核对，逐条列出）：
  - a.blank(unknown-receiver) 首行 3141 ×10
  - a.emit(unknown-receiver) 首行 4474 ×5
  - a.enum_find(unknown-receiver) 首行 2613 ×7
  - self.enum_find(unknown-receiver) 首行 309 ×1
  - a.enum_has_variant(unknown-receiver) 首行 2000 ×1
  - p.expect(unknown-receiver) 首行 632 ×47
  - a.fail(unknown-receiver) 首行 1616 ×25
  - p.fail(unknown-receiver) 首行 605 ×23
  - a.field_ty(unknown-receiver) 首行 1772 ×3
  - a.fn_find(unknown-receiver) 首行 1503 ×6
  - self.fn_find(unknown-receiver) 首行 392 ×1
  - a.global_find(unknown-receiver) 首行 1861 ×2
  - a.imports_has(unknown-receiver) 首行 2000 ×1
  - self.indent_str(unknown-receiver) 首行 220 ×1
  - a.is_cur_mut_param(unknown-receiver) 首行 1886 ×1
  - p.kind(unknown-receiver) 首行 600 ×210
  - a.line(unknown-receiver) 首行 3388 ×75
  - a.lu_after(unknown-receiver) 首行 2149 ×3
  - a.lu_record(unknown-receiver) 首行 1435 ×3
  - a.method_find(unknown-receiver) 首行 1482 ×3
  - p.next(unknown-receiver) 首行 601 ×165
  - p.peek(unknown-receiver) 首行 724 ×8
  - a.result(unknown-receiver) 首行 4745 ×1
  - p.skip_empty_lines(unknown-receiver) 首行 616 ×67
  - a.struct_derive(unknown-receiver) 首行 3917 ×1
  - p.text(unknown-receiver) 首行 608 ×41
  - a.ty_find(unknown-receiver) 首行 1274 ×11
  - self.ty_find(unknown-receiver) 首行 266 ×2
  - a.vparam(unknown-receiver) 首行 1877 ×5
  - a.vpush(unknown-receiver) 首行 829 ×7
  - a.vscope(unknown-receiver) 首行 825 ×7
  - a.vty(unknown-receiver) 首行 1479 ×9
  - a.vunscope(unknown-receiver) 首行 833 ×7

### `auto/aavm.at`
- 声明 1（方法 0）；use 边 1；调用候选 12（unknown 0）
- 能力证据：comparison-operator×1, free-function×1, if-else×4, plain-list-value×1, return-statement×1, string-literal×3, use-module×1, while-loop×2

## 人工结论层
- `manual-decisions.json`：每次 --check 绑定输入 hash 校验；过期即失败。
