# AAVM v2 divergence 登记册(plan-432 起随移植进度维护)

格式:编号(定义见 divergence-rules.md §2 + 本册新增)× 文件 × 理由。
433 四向对比以本册为"合法差异白名单"。

## auto/lib/token.at(S1)

- **D11**(×1):`Option<TokenKind>` 返回 → `Ident` 哨兵(keyword_kind 的 None 语义
  折叠进 Ident 分支;调用方以 != Ident 判定)。
- **D11b**(×1):match 语句 → if 链(VM 对 match-on-string/enum 的支持未验证,
  v1 全程 if 链是已证路径;kind_name 139 臂同)。

## auto/lib/lexer.at(S1)

- **D12**: `Peekable<Chars>` → `char_at(i)` + p/at 游标(431 D1 的变体:
  免 Vec 构造,char_at 返回 UTF-32 码点,a2r 侧映射 `.chars().nth()`)。
- **D13**: Lexer struct 状态 → `lex_dump` 单遍局部变量(S2 需要 push_token 回退时
  再封装成真实 Lexer 类型)。
- **D14**: token 流/Vec<Token> → dump 串直出(kind_name 层)——M1 闸门判据;
  S2 换真实 Token 结构(dump 驱动保留为测试面)。
- **D15**: 注释 buffer 后继(CommentContent/CommentEnd 经 VecDeque 缓冲)→
  直出同序 dump。
- **D16**: Pos.len 按 char 计(基线按 UTF-8 字节)——corpus 限定 ASCII;
  多字节语料进 M1 前需补字节宽度记账。
- **D17**(结构性,最重要):循环体内一律不用 `continue`——**VM bug 实证**:
  循环体含调用语句(如 `var c = s.char_at(p)`)时,`continue` 穿透执行后续语句
  (probe p11/p12:分支执行后落入同迭代余下语句,p 双增、丢尾字符);
  `break` 不受影响。全部改写为 else-if 链。**挂 242 tracker:S1-432**。
- **D18**: 基线块注释 quirk 照抄——Rust `slash_or_comment` 的内容循环在 `*`
  分支 `push(c)+next()` 连吞两字符(内容中 `*` 后一字符丢失,如 `/* block` 的
  内容是 `*block`);M1 闸门以基线为唯一依据故照抄。**上游修复会改 golden**,
  挂 432 债务簿(建议修 Rust 侧并同步本文件)。
- **D19**: 数字 token 的 text/len 语义照抄基线——text 剥下划线与 f/d/u/i/u8/i8
  后缀,Pos.len 用剥离后长度(与真实消耗不等,at 记账随基线漂移);
  Tok.len(记账)与 Tok.adv(p 推进)分离承载。

## 计数(S1 时点)

| 文件 | divergence 处数 |
|---|---|
| token.at | 2 |
| lexer.at | 8 |
| 合计 | 10 |


## auto/lib/parser.at(S2)

- **D20**: AST 结构(Stmt/Expr 枚举 + Box 树)→ `parse_dump(source) -> str`
  S-expr 直出(判据层;真实 AST 推迟到 S4 codegen 需要时定)。dump 由各
  parse_* 构造期拼装,E 载体 `{kind,word,name,args,dump,ity}`。
- **D21**: token 流承载 `Vec<Token>` + pushback → `List.new()/push/get/len`
  (sanctioned);前瞻 = 索引算术(p_peek / paren_prefix 扫描)。
- **D22**: Result 错误传播 + add_error 收集 → `p.err` 单槽 + 全函数入口短路;
  错误信息编码行号与 token kind,parse_dump 直返错误串使闸门显式爆红。
- **D23**: Type 枚举 → Display 字符串载体(builtin 表折叠;StrFixed 恒显
  "str" 使该折叠无损);let 无注解时的 parser 内联推断(infer_type_expr
  核心子集:字面量/Ident 作用域查找/Bina unify 简式/Array/Tuple)在 E.ity
  构造期完成。unify coercion 分支语料未及,按 Unknown 传播。
- **D24**: float/double 值显示 = 字面量文本剥分数尾零("1.0"→"1");
  Rust 为 f32/f64 shortest-roundtrip Display,十进制文本字面量二者一致,
  科学计数/大数留 Missing。
- **D25**(VM 缺陷规避,probe 实证,挂 242:S2-432):
  - ①原生调用(auto.list.*)内联作结构体构造参数时返回值丢失——
    `Q(toks, List.new())` 的字段成垃圾;提升局部变量后恢复。
  - ②`List.pop` 目录签名 Void 而 shim 压返回值:语句位调用即静默毁栈
    (后续 print 消失),消费位返回垃圾(-2)。作用域退栈改为 depth 计数。
- **D26**(blocker,挂 242:S2-432):**VM 字符串池 RC 回归阻断 M2**——
  循环体内以运行期字符串(concat/函数返回值,字面量与循环外均正常)调
  `List.push`,之后读回即 UAF:`[RC canary] string tombstone access`。
  最小复现(~12 行):`while i < 3 { l.push("(s" + i.str() + ")"); i = i + 1 }`
  后 `l.get(2)` → print 即 canary;P419_TRACE_POOL 追踪显示池槽
  retain(0→1)/release(1→0)/FREE 循环复用后,复活的槽 tombstone 未清。
  提升临时变量、改 for 循环均不可绕。**master 上 conformance_bootstrap
  同类 canary 已红**(heap 4000001 UAF),99_bootstrap parser 系列 ignored,
  疑似 Plan 419/423 RC 改造的存量回归。S2 的 parse_dump 必经
  "循环内 push 语句串"路径,M2 闸门在 VM 修复前无法转绿(aavm2_m2
  闸门测试已挂 ignore 并注明)。

## 计数(S2 时点)

| 文件 | divergence 处数 |
|---|---|
| token.at | 2 |
| lexer.at | 8(+S2 重构后 tokenize/lex_dump 包装,D13/D14 落地) |
| parser.at | 7 |
| 合计 | 17 |
