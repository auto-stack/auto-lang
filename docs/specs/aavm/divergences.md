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
