//! 语法程序规范二进制 codec（身份计算 + 语言包装载格式）。
//!
//! 实验格式（PROGRAM_CODEC_VERSION 前缀），非正式 Batom 标准。布局全
//! little-endian、字符串长度前缀、集合按声明序——同一程序字节确定，
//! 供 `GrammarId` 计算与磁盘往返（静态 Atom→实验二进制→执行）。

use crate::program::*;

pub fn encode_canonical(p: &GrammarProgram) -> Vec<u8> {
    let mut w = Vec::with_capacity(1024);
    w.extend_from_slice(&PROGRAM_CODEC_VERSION.to_le_bytes());
    w.extend_from_slice(&p.schema_version.to_le_bytes());
    put_str(&mut w, &p.name);
    w.extend_from_slice(&p.revision.to_le_bytes());

    // scanner
    put_u32(&mut w, p.scanner.mode_names.len() as u32);
    for m in &p.scanner.mode_names {
        put_str(&mut w, m);
    }
    put_u32(&mut w, p.scanner.rules.len() as u32);
    for r in &p.scanner.rules {
        w.extend_from_slice(&r.mode.to_le_bytes());
        put_matcher(&mut w, &r.matcher);
        w.extend_from_slice(&r.token.to_le_bytes());
        w.push(r.trivia as u8);
        w.extend_from_slice(&r.next_mode.to_le_bytes());
    }

    // tokens
    put_u32(&mut w, p.tokens.len() as u32);
    for t in &p.tokens {
        put_str(&mut w, &t.name);
    }

    // rules
    put_u32(&mut w, p.rules.len() as u32);
    for r in &p.rules {
        put_str(&mut w, &r.name);
        w.push(r.recover as u8);
        put_u32(&mut w, r.alts.len() as u32);
        for a in &r.alts {
            put_u32(&mut w, a.steps.len() as u32);
            for s in &a.steps {
                match s.atom {
                    Atom::Tok(t) => {
                        w.push(0);
                        w.extend_from_slice(&t.to_le_bytes());
                    }
                    Atom::Rule(r) => {
                        w.push(1);
                        w.extend_from_slice(&r.to_le_bytes());
                    }
                    Atom::Pratt(x) => {
                        w.push(2);
                        w.extend_from_slice(&x.to_le_bytes());
                    }
                    Atom::CtxName(c) => {
                        w.push(3);
                        w.extend_from_slice(&c.to_le_bytes());
                    }
                    Atom::TokAdjacent(t) => {
                        w.push(4);
                        w.extend_from_slice(&t.to_le_bytes());
                    }
                }
                w.push(match s.repeat {
                    Repeat::Once => 0,
                    Repeat::Optional => 1,
                    Repeat::ZeroOrMore => 2,
                    Repeat::ListPlain => 3,
                    Repeat::ListNewline => 4,
                });
                w.push(s.same_line as u8);
                w.push(s.cut as u8);
                match s.field {
                    None => w.push(0),
                    Some(f) => {
                        w.push(1);
                        w.extend_from_slice(&f.to_le_bytes());
                    }
                }
            }
        }
    }

    // 规则诊断标签（PLAN-979）：与 rules 平行、声明序。
    put_u32(&mut w, p.rule_expects.len() as u32);
    for e in &p.rule_expects {
        match e {
            None => w.push(0),
            Some(s) => {
                w.push(1);
                put_str(&mut w, s);
            }
        }
    }

    // pratt
    put_u32(&mut w, p.pratt.len() as u32);
    for t in &p.pratt {
        w.extend_from_slice(&t.primary.to_le_bytes());
        put_u32(&mut w, t.ops.len() as u32);
        for op in &t.ops {
            w.extend_from_slice(&op.op.to_le_bytes());
            w.push(op.binding);
            w.push(match op.assoc {
                Assoc::Left => 0,
                Assoc::Right => 1,
            });
        }
        put_opt_tok(&mut w, t.group_open);
        put_opt_tok(&mut w, t.group_close);
        match &t.call_paren {
            None => w.push(0),
            Some(c) => {
                w.push(1);
                w.extend_from_slice(&c.open.to_le_bytes());
                w.extend_from_slice(&c.close.to_le_bytes());
                w.push(c.newline_free_inside as u8);
                w.push(c.callee_same_line as u8);
            }
        }
        put_opt_tok(&mut w, t.call_args);
        put_opt_tok(&mut w, t.postfix_tail);
    }

    w.extend_from_slice(&p.start.to_le_bytes());

    put_u32(&mut w, p.field_names.len() as u32);
    for f in &p.field_names {
        put_str(&mut w, f);
    }

    // 上下文名称表（PLAN-979）：声明序，身份内容的一部分。
    put_u32(&mut w, p.ctx_names.len() as u32);
    for n in &p.ctx_names {
        put_str(&mut w, &n.name);
        put_str(&mut w, &n.spelling);
        w.extend_from_slice(&n.token.to_le_bytes());
        put_str(&mut w, &n.role);
    }

    put_u32(&mut w, p.newline.stmt_end.len() as u32);
    for t in &p.newline.stmt_end {
        w.extend_from_slice(&t.to_le_bytes());
    }
    put_u32(&mut w, p.newline.sep_consume.len() as u32);
    for t in &p.newline.sep_consume {
        w.extend_from_slice(&t.to_le_bytes());
    }
    w.push(p.newline.significant as u8);

    put_u32(&mut w, p.recovery.sync.len() as u32);
    for t in &p.recovery.sync {
        w.extend_from_slice(&t.to_le_bytes());
    }

    match &p.outline {
        None => w.push(0),
        Some(o) => {
            w.push(1);
            put_str(&mut w, &o.decl_rule);
            put_str(&mut w, &o.name_field);
            match &o.params_field {
                None => w.push(0),
                Some(s) => {
                    w.push(1);
                    put_str(&mut w, s);
                }
            }
        }
    }
    w
}

/// 装载预算（PLAN-1029 ADM-01，DEC-ADM r4）。
///
/// * `FromWire`（默认）：装载封套由**已验证 wire** 推导——每个 count 先按
///   该表最小 wire 字节对借入剩余字节准入，才允许受控预留；请求计费
///   全程累计，但不设固定装载门（`Budget::work_bytes` 约束解析工作，
///   不冒充旧 loader 上限——r3 无条件 256MiB 装载门已撤销）。
/// * `Bytes(B)`：调用方显式提供装载预算（测试或明确传递装载预算的真实
///   调用者）；计费超出 B 即 `ResourceExhausted`，不让旧调用暗中继承。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadBudget {
    FromWire,
    Bytes(usize),
}

/// 请求计费：分配前先计费（count×size_of 预留、字符串字节、规范重编码
/// 与身份临时），全部 checked；`Bytes(B)` 模式逐笔对照上限。计费的是
/// **请求字节**，实际 capacity 与 allocator 额外驻留另行观测，不冒充
/// RSS 硬界。
struct Charge {
    used: usize,
    cap: Option<usize>,
}

impl Charge {
    fn add(&mut self, bytes: usize) -> Result<(), RtError> {
        self.used = self.used.checked_add(bytes).ok_or_else(|| {
            RtError::InvalidProgram("grammar program load charge overflow".into())
        })?;
        if let Some(b) = self.cap {
            if self.used > b {
                return Err(RtError::ResourceExhausted {
                    what: "grammar program load",
                    limit: b,
                });
            }
        }
        Ok(())
    }
}

/// 表项最小 wire 字节（必要下界，非充分格式证明；reader 仍逐字段核
/// 截断/引用）。嵌套表使用各自当前位置的剩余字节，不按总输入反复授予。
const MIN_MODE_NAMES: usize = 4; // 空字符串长度 u32
const MIN_SCAN_RULES: usize = 16; // mode4 + StringLit matcher3 + token4 + trivia1 + next_mode4
const MIN_TOKENS: usize = 4; // 空字符串长度 u32
const MIN_GRAMMAR_RULES: usize = 9; // name 长度4 + recover1 + alt_count4
const MIN_ALTS: usize = 4; // step_count4
const MIN_STEPS: usize = 9; // atom(tag1+ref4) + repeat1 + same_line1 + cut1 + field_presence1
const MIN_RULE_EXPECTS: usize = 1; // None tag
const MIN_PRATT: usize = 13; // primary4 + op_count4 + 五个可选字段 presence 各1
const MIN_PRATT_OPS: usize = 6; // token4 + binding1 + assoc1
const MIN_FIELD_NAMES: usize = 4; // 字符串长度 u32
const MIN_CTX_NAMES: usize = 16; // name4 + spelling4 + token4 + role4
const MIN_TOKEN_LIST: usize = 4; // token u32（stmt_end/sep_consume/sync 同）

/// 严格往返（语言包装载：静态 Atom → 程序 → 实验二进制 → 程序）。
/// 身份字段（PackageIdentity）不序列化——由内容重算，装载后必须一致。
///
/// 结构性错误（缺字节/溢出/错版本/tag/引用/UTF-8/尾随字节）=
/// `InvalidProgram`；预算或 allocator 拒绝 = `ResourceExhausted`（不转成
/// 源码语法诊断）。
pub fn decode_program(bytes: &[u8], budget: LoadBudget) -> Result<GrammarProgram, RtError> {
    let mut charged = Charge {
        used: 0,
        cap: match budget {
            LoadBudget::FromWire => None,
            LoadBudget::Bytes(b) => Some(b),
        },
    };
    // 规范重编码（几何增长峰值 capacity ≤ max(1024, 2×len)）与身份临时
    // （version blob/GrammarId 串/名称克隆）计入同一封套。
    {
        let reencode = bytes.len().saturating_mul(2).max(1024);
        charged.add(reencode)?;
        charged.add(4096)?;
    }
    let mut r = Reader { b: bytes, pos: 0 };
    let v = r.u16()?;
    if v != PROGRAM_CODEC_VERSION {
        return Err(RtError::InvalidProgram(format!(
            "program codec version {v} != {PROGRAM_CODEC_VERSION}"
        )));
    }
    let schema_version = r.u32()?;
    let name = r.str(&mut charged)?;
    let revision = r.u64()?;

    let n = r.count("mode_names", MIN_MODE_NAMES)?;
    let mut mode_names: Vec<String> = Vec::new();
    r.reserve(&mut mode_names, n, "mode_names", &mut charged)?;
    for _ in 0..n {
        mode_names.push(r.str(&mut charged)?);
    }
    let n = r.count("scanner.rules", MIN_SCAN_RULES)?;
    let mut rules: Vec<ScanRule> = Vec::new();
    r.reserve(&mut rules, n, "scanner.rules", &mut charged)?;
    for _ in 0..n {
        let mode = r.u32()?;
        let matcher = get_matcher(&mut r, &mut charged)?;
        let token = r.u32()?;
        let trivia = r.u8()? != 0;
        let next_mode = r.u32()?;
        rules.push(ScanRule {
            mode,
            matcher,
            token,
            trivia,
            next_mode,
        });
    }

    let n = r.count("tokens", MIN_TOKENS)?;
    let mut tokens: Vec<TokenSpec> = Vec::new();
    r.reserve(&mut tokens, n, "tokens", &mut charged)?;
    for _ in 0..n {
        tokens.push(TokenSpec {
            name: r.str(&mut charged)?,
        });
    }

    let n = r.count("rules", MIN_GRAMMAR_RULES)?;
    let mut grammar_rules: Vec<RuleSpec> = Vec::new();
    r.reserve(&mut grammar_rules, n, "rules", &mut charged)?;
    for _ in 0..n {
        let rname = r.str(&mut charged)?;
        let recover = r.u8()? != 0;
        let na = r.count("alts", MIN_ALTS)?;
        let mut alts: Vec<Alt> = Vec::new();
        r.reserve(&mut alts, na, "alts", &mut charged)?;
        for _ in 0..na {
            let ns = r.count("steps", MIN_STEPS)?;
            let mut steps: Vec<Step> = Vec::new();
            r.reserve(&mut steps, ns, "steps", &mut charged)?;
            for _ in 0..ns {
                let atom = match r.u8()? {
                    0 => Atom::Tok(r.u32()?),
                    1 => Atom::Rule(r.u32()?),
                    2 => Atom::Pratt(r.u32()?),
                    3 => Atom::CtxName(r.u32()?),
                    4 => Atom::TokAdjacent(r.u32()?),
                    t => return Err(RtError::InvalidProgram(format!("bad atom tag {t}"))),
                };
                let repeat = match r.u8()? {
                    0 => Repeat::Once,
                    1 => Repeat::Optional,
                    2 => Repeat::ZeroOrMore,
                    3 => Repeat::ListPlain,
                    4 => Repeat::ListNewline,
                    t => return Err(RtError::InvalidProgram(format!("bad repeat tag {t}"))),
                };
                let same_line = r.u8()? != 0;
                let cut = r.u8()? != 0;
                let field = if r.u8()? != 0 { Some(r.u32()?) } else { None };
                steps.push(Step {
                    atom,
                    field,
                    repeat,
                    same_line,
                    cut,
                });
            }
            alts.push(Alt { steps });
        }
        grammar_rules.push(RuleSpec {
            name: rname,
            alts,
            recover,
        });
    }

    let n = r.count("rule_expects", MIN_RULE_EXPECTS)?;
    let mut rule_expects: Vec<Option<String>> = Vec::new();
    r.reserve(&mut rule_expects, n, "rule_expects", &mut charged)?;
    for _ in 0..n {
        rule_expects.push(if r.u8()? != 0 {
            Some(r.str(&mut charged)?)
        } else {
            None
        });
    }

    let n = r.count("pratt", MIN_PRATT)?;
    let mut pratt: Vec<PrattSpec> = Vec::new();
    r.reserve(&mut pratt, n, "pratt", &mut charged)?;
    for _ in 0..n {
        let primary = r.u32()?;
        let no = r.count("pratt.ops", MIN_PRATT_OPS)?;
        let mut ops: Vec<PrattOp> = Vec::new();
        r.reserve(&mut ops, no, "pratt.ops", &mut charged)?;
        for _ in 0..no {
            let op = r.u32()?;
            let binding = r.u8()?;
            let assoc = match r.u8()? {
                0 => Assoc::Left,
                1 => Assoc::Right,
                t => return Err(RtError::InvalidProgram(format!("bad assoc tag {t}"))),
            };
            ops.push(PrattOp { op, binding, assoc });
        }
        let group_open = opt_tok(&mut r)?;
        let group_close = opt_tok(&mut r)?;
        let call_paren = if r.u8()? != 0 {
            let open = r.u32()?;
            let close = r.u32()?;
            let newline_free_inside = r.u8()? != 0;
            let callee_same_line = r.u8()? != 0;
            Some(CallParenRule {
                open,
                close,
                newline_free_inside,
                callee_same_line,
            })
        } else {
            None
        };
        let call_args = opt_tok(&mut r)?;
        let postfix_tail = opt_tok(&mut r)?;
        pratt.push(PrattSpec {
            primary,
            ops,
            group_open,
            group_close,
            call_paren,
            call_args,
            postfix_tail,
        });
    }

    let start = r.u32()?;

    let n = r.count("field_names", MIN_FIELD_NAMES)?;
    let mut field_names: Vec<String> = Vec::new();
    r.reserve(&mut field_names, n, "field_names", &mut charged)?;
    for _ in 0..n {
        field_names.push(r.str(&mut charged)?);
    }

    let n = r.count("ctx_names", MIN_CTX_NAMES)?;
    let mut ctx_names: Vec<CtxNameSpec> = Vec::new();
    r.reserve(&mut ctx_names, n, "ctx_names", &mut charged)?;
    for _ in 0..n {
        let name = r.str(&mut charged)?;
        let spelling = r.str(&mut charged)?;
        let token = r.u32()?;
        let role = r.str(&mut charged)?;
        ctx_names.push(CtxNameSpec {
            name,
            spelling,
            token,
            role,
        });
    }

    let n = r.count("newline.stmt_end", MIN_TOKEN_LIST)?;
    let mut stmt_end: Vec<u32> = Vec::new();
    r.reserve(&mut stmt_end, n, "newline.stmt_end", &mut charged)?;
    for _ in 0..n {
        stmt_end.push(r.u32()?);
    }
    let n = r.count("newline.sep_consume", MIN_TOKEN_LIST)?;
    let mut sep_consume: Vec<u32> = Vec::new();
    r.reserve(&mut sep_consume, n, "newline.sep_consume", &mut charged)?;
    for _ in 0..n {
        sep_consume.push(r.u32()?);
    }
    let significant = r.u8()? != 0;
    let n = r.count("recovery.sync", MIN_TOKEN_LIST)?;
    let mut sync: Vec<u32> = Vec::new();
    r.reserve(&mut sync, n, "recovery.sync", &mut charged)?;
    for _ in 0..n {
        sync.push(r.u32()?);
    }
    let outline = if r.u8()? != 0 {
        let decl_rule = r.str(&mut charged)?;
        let name_field = r.str(&mut charged)?;
        let params_field = if r.u8()? != 0 {
            Some(r.str(&mut charged)?)
        } else {
            None
        };
        Some(OutlineSpec {
            decl_rule,
            name_field,
            params_field,
        })
    } else {
        None
    };
    r.end()?;

    let scanner = ScannerProgram { mode_names, rules };
    // 引用命名空间准入（PLAN-1029 ADM-02）：结构/预算/计数之外，引用在
    // 返回给调用方前同样受控拒绝（坏引用绝不进入 FirstSets/parse）。
    let mut p = GrammarProgram {
        name,
        revision,
        schema_version,
        scanner,
        tokens,
        rules: grammar_rules,
        pratt,
        field_names,
        ctx_names,
        rule_expects,
        start,
        newline: NewlinePolicy {
            significant,
            stmt_end,
            sep_consume,
        },
        recovery: RecoverySpec { sync },
        outline,
        identity: crate::identity::PackageIdentity {
            name: String::new(),
            grammar_id: crate::identity::GrammarId(String::new()),
        },
    };
    p.validate_references()?;
    // 身份 = 名称 + 规范内容 + 版本（装载后与编译期一致才可信）。
    let canonical = encode_canonical(&p);
    let versions = [
        crate::GRAMMAR_SCHEMA_VERSION_STR,
        crate::RUNTIME_VERSION,
        crate::SCANNER_ABI_VERSION,
        crate::PARSER_ABI_VERSION,
    ];
    p.identity = crate::identity::package_identity(&p.name, &canonical, &versions);
    let _ = canonical; // identity 已绑定
    Ok(p)
}

/// 旧字符串入口：`FromWire` 装载 + 错误渲染（保持历史调用面不变）。
pub fn decode_canonical(bytes: &[u8]) -> Result<GrammarProgram, String> {
    decode_program(bytes, LoadBudget::FromWire).map_err(|e| e.to_string())
}

fn put_str(w: &mut Vec<u8>, s: &str) {
    w.extend_from_slice(&(s.len() as u32).to_le_bytes());
    w.extend_from_slice(s.as_bytes());
}
fn put_u32(w: &mut Vec<u8>, v: u32) {
    w.extend_from_slice(&v.to_le_bytes());
}
fn put_opt_tok(w: &mut Vec<u8>, t: Option<u32>) {
    match t {
        None => w.push(0),
        Some(v) => {
            w.push(1);
            w.extend_from_slice(&v.to_le_bytes());
        }
    }
}

fn opt_tok(r: &mut Reader) -> Result<Option<u32>, RtError> {
    if r.u8()? != 0 {
        Ok(Some(r.u32()?))
    } else {
        Ok(None)
    }
}

struct Reader<'a> {
    b: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], RtError> {
        // pos+n 一律 checked（结构溢出按 InvalidProgram 分账，不回绕）。
        let end = self
            .pos
            .checked_add(n)
            .filter(|&e| e <= self.b.len())
            .ok_or_else(|| RtError::InvalidProgram("truncated program".into()))?;
        let s = &self.b[self.pos..end];
        self.pos = end;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, RtError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, RtError> {
        let s = self.take(2)?;
        Ok(u16::from_le_bytes([s[0], s[1]]))
    }
    fn u32(&mut self) -> Result<u32, RtError> {
        let s = self.take(4)?;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
    fn u64(&mut self) -> Result<u64, RtError> {
        let s = self.take(8)?;
        let mut a = [0u8; 8];
        a.copy_from_slice(s);
        Ok(u64::from_le_bytes(a))
    }
    /// 计数准入（PLAN-1029 ADM-01）：读取表计数后，先按该表每项最小
    /// wire 字节对**当前位置的剩余字节**核对（count×min 溢出或超出即
    /// malformed），然后才允许任何受控预留——巨大 count 在分配前被拒。
    fn count(&mut self, table: &str, min_item: usize) -> Result<usize, RtError> {
        let n = self.u32()? as usize;
        let need = n
            .checked_mul(min_item)
            .ok_or_else(|| RtError::InvalidProgram(format!("{table} count {n} overflows")))?;
        let remaining = self.b.len() - self.pos;
        if need > remaining {
            return Err(RtError::InvalidProgram(format!(
                "{table} count {n} needs ≥{need}B but only {remaining}B remain"
            )));
        }
        Ok(n)
    }
    /// 受控预留（try_reserve_exact，防几何增长；分配前按请求计费）。
    fn reserve<T>(
        &mut self,
        v: &mut Vec<T>,
        n: usize,
        table: &str,
        charged: &mut Charge,
    ) -> Result<(), RtError> {
        let bytes = n
            .checked_mul(std::mem::size_of::<T>())
            .ok_or_else(|| RtError::InvalidProgram(format!("{table} reserve overflows")))?;
        charged.add(bytes)?;
        v.try_reserve_exact(n)
            .map_err(|_| RtError::ResourceExhausted {
                what: "grammar program load",
                limit: bytes,
            })
    }
    /// 字符串：先 checked 取片、核 UTF-8，再受控分配（坏 UTF-8 不浪费分配）。
    fn str(&mut self, charged: &mut Charge) -> Result<String, RtError> {
        let n = self.u32()? as usize;
        let s = self.take(n)?;
        let t = core::str::from_utf8(s)
            .map_err(|_| RtError::InvalidProgram("bad utf-8 in program".into()))?;
        charged.add(t.len())?;
        let mut out = String::new();
        out.try_reserve_exact(t.len())
            .map_err(|_| RtError::ResourceExhausted {
                what: "grammar program load",
                limit: t.len(),
            })?;
        out.push_str(t);
        Ok(out)
    }
    fn end(&mut self) -> Result<(), RtError> {
        if self.pos != self.b.len() {
            return Err(RtError::InvalidProgram("trailing bytes in program".into()));
        }
        Ok(())
    }
}

fn put_matcher(w: &mut Vec<u8>, m: &Matcher) {
    match m {
        Matcher::Literal(s) => {
            w.push(0);
            put_str(w, s);
        }
        Matcher::Charset {
            first,
            cont,
            min_len,
        } => {
            w.push(1);
            w.extend_from_slice(&first.bits[0].to_le_bytes());
            w.extend_from_slice(&first.bits[1].to_le_bytes());
            w.push(first.non_ascii as u8);
            w.extend_from_slice(&cont.bits[0].to_le_bytes());
            w.extend_from_slice(&cont.bits[1].to_le_bytes());
            w.push(cont.non_ascii as u8);
            w.extend_from_slice(&min_len.to_le_bytes());
        }
        Matcher::LineComment(start) => {
            w.push(2);
            put_str(w, start);
        }
        Matcher::BlockComment { start, end, nested } => {
            w.push(3);
            put_str(w, start);
            put_str(w, end);
            w.push(*nested as u8);
        }
        Matcher::StringLit { quote, escape } => {
            w.push(4);
            w.push(*quote);
            w.push(*escape as u8);
        }
        Matcher::LineText { start, close } => {
            w.push(5);
            put_str(w, start);
            w.push(*close);
        }
    }
}

fn get_matcher(r: &mut Reader, charged: &mut Charge) -> Result<Matcher, RtError> {
    let cs = |lo: u64, hi: u64, non_ascii: bool| CharSet {
        bits: [lo, hi],
        non_ascii,
    };
    Ok(match r.u8()? {
        0 => Matcher::Literal(r.str(charged)?),
        1 => {
            let fl = u64::from_le_bytes(r.take(8)?.try_into().unwrap());
            let fh = u64::from_le_bytes(r.take(8)?.try_into().unwrap());
            let fna = r.u8()? != 0;
            let cl = u64::from_le_bytes(r.take(8)?.try_into().unwrap());
            let ch = u64::from_le_bytes(r.take(8)?.try_into().unwrap());
            let cna = r.u8()? != 0;
            let min_len = r.u32()?;
            Matcher::Charset {
                first: cs(fl, fh, fna),
                cont: cs(cl, ch, cna),
                min_len,
            }
        }
        2 => Matcher::LineComment(r.str(charged)?),
        3 => {
            let start = r.str(charged)?;
            let end = r.str(charged)?;
            let nested = r.u8()? != 0;
            Matcher::BlockComment { start, end, nested }
        }
        4 => {
            let quote = r.u8()?;
            let escape = r.u8()? != 0;
            Matcher::StringLit { quote, escape }
        }
        5 => Matcher::LineText {
            start: r.str(charged)?,
            close: r.u8()?,
        },
        t => return Err(RtError::InvalidProgram(format!("bad matcher tag {t}"))),
    })
}

#[cfg(test)]
mod tests {
    use crate::program::RtError;
    use crate::program_codec::{self, LoadBudget};
    use crate::testkit;

    #[test]
    fn program_codec_roundtrip_and_identity_binding() {
        let p = testkit::mini_program();
        let bytes = program_codec::encode_canonical(&p);
        let mut q = program_codec::decode_canonical(&bytes).expect("decode");
        // 身份由内容重算，必须与编译期一致。
        assert_eq!(p.identity.grammar_id, q.identity.grammar_id);
        q.identity = p.identity.clone();
        assert_eq!(&p, &q);
        // 截断/尾随字节 → 受控错误。
        assert!(program_codec::decode_canonical(&bytes[..bytes.len() - 1]).is_err());
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(program_codec::decode_canonical(&trailing).is_err());
    }

    #[test]
    fn line_text_matcher_roundtrips_canonically() {
        let mut program = testkit::mini_program();
        let rule = &mut program.scanner.rules[0];
        rule.matcher = crate::program::Matcher::LineText {
            start: "|".into(),
            close: b'}',
        };
        let bytes = program_codec::encode_canonical(&program);
        let recovered = program_codec::decode_canonical(&bytes).expect("decode line-text matcher");
        assert_eq!(program.scanner.rules[0], recovered.scanner.rules[0]);
        assert_eq!(bytes, program_codec::encode_canonical(&recovered));
    }

    /// PLAN-1029 ADM-01：坏 count 在巨大预留前受控拒绝（每表最小合法前缀 +
    /// 多 1 项计数 / u32::MAX）；恰等剩余的计数通过准入、在后续截断处受控
    /// 失败；FromWire 与显式 Bytes(B) 的结果/身份相同，计费等界（恰等成功、
    /// 少 1 拒绝）。
    #[test]
    fn audit1029_adm_01_count_admission_before_reserve_and_budget_envelope() {
        fn u32b(v: u32) -> [u8; 4] {
            v.to_le_bytes()
        }
        // v3 时代的 22 字节头 + mode count = u32::MAX（修前在 with_capacity
        // 处请求 ~103GB → abort；修后在准入处受控拒绝）。
        let mut header = Vec::new();
        header.extend_from_slice(&program_codec::encode_canonical(&testkit::mini_program())[..2]); // version
        header.extend_from_slice(&u32b(1)); // schema
        header.extend_from_slice(&u32b(0)); // name ""
        header.extend_from_slice(&[0u8; 8]); // revision
        assert_eq!(header.len(), 18);
        let mut bad = header.clone();
        bad.extend_from_slice(&u32b(u32::MAX));
        assert_eq!(bad.len(), 22);
        match program_codec::decode_program(&bad, LoadBudget::FromWire) {
            Err(RtError::InvalidProgram(m)) => assert!(
                m.contains("mode_names"),
                "mode count admission must name the table: {m}"
            ),
            other => panic!("u32::MAX mode count must be rejected before reserve: {other:?}"),
        }

        // 14 处计数预留逐项：最小合法前缀 + “多 1 项”计数（剩余 0 字节时
        // 任何非零 count 都不可能成立）。
        let mut pfx = header.clone(); // hdr = 18B
        let cases: Vec<(&str, Vec<u8>)> = {
            let mut c = Vec::new();
            // mode_names（其后 scanner.rules=0）
            let mut b = pfx.clone();
            b.extend_from_slice(&u32b(1));
            c.push(("mode_names", b));
            pfx.extend_from_slice(&u32b(0)); // mode_names = 0
                                             // scanner.rules
            let mut b = pfx.clone();
            b.extend_from_slice(&u32b(1));
            c.push(("scanner.rules", b));
            pfx.extend_from_slice(&u32b(0)); // scan rules = 0
                                             // tokens
            let mut b = pfx.clone();
            b.extend_from_slice(&u32b(1));
            c.push(("tokens", b));
            pfx.extend_from_slice(&u32b(0)); // tokens = 0
                                             // rules
            let mut b = pfx.clone();
            b.extend_from_slice(&u32b(1));
            c.push(("rules", b));
            pfx.extend_from_slice(&u32b(0)); // rules = 0
                                             // rule_expects
            let mut b = pfx.clone();
            b.extend_from_slice(&u32b(1));
            c.push(("rule_expects", b));
            pfx.extend_from_slice(&u32b(0)); // rule_expects = 0
                                             // pratt
            let mut b = pfx.clone();
            b.extend_from_slice(&u32b(1));
            c.push(("pratt", b));
            pfx.extend_from_slice(&u32b(0)); // pratt = 0
                                             // start
            pfx.extend_from_slice(&u32b(0));
            // field_names
            let mut b = pfx.clone();
            b.extend_from_slice(&u32b(1));
            c.push(("field_names", b));
            pfx.extend_from_slice(&u32b(0)); // field_names = 0
                                             // ctx_names
            let mut b = pfx.clone();
            b.extend_from_slice(&u32b(1));
            c.push(("ctx_names", b));
            pfx.extend_from_slice(&u32b(0)); // ctx_names = 0
                                             // newline.stmt_end
            let mut b = pfx.clone();
            b.extend_from_slice(&u32b(1));
            c.push(("newline.stmt_end", b));
            pfx.extend_from_slice(&u32b(0));
            // newline.sep_consume
            let mut b = pfx.clone();
            b.extend_from_slice(&u32b(1));
            c.push(("newline.sep_consume", b));
            pfx.extend_from_slice(&u32b(0));
            // significant
            pfx.push(0);
            // recovery.sync
            let mut b = pfx.clone();
            b.extend_from_slice(&u32b(1));
            c.push(("recovery.sync", b));
            pfx.extend_from_slice(&u32b(0)); // sync = 0
                                             // outline absent → 完整最小空程序：
            pfx.push(0);
            // 嵌套表：alts / steps / pratt.ops 需要父项存在。
            let mut nested_base = header.clone();
            nested_base.extend_from_slice(&u32b(0)); // modes
            nested_base.extend_from_slice(&u32b(0)); // scan rules
            nested_base.extend_from_slice(&u32b(0)); // tokens
            nested_base.extend_from_slice(&u32b(1)); // rules = 1
            nested_base.extend_from_slice(&u32b(0)); // rule name ""
            nested_base.push(0); // recover
            let mut b = nested_base.clone();
            b.extend_from_slice(&u32b(1)); // alts = 1（仅 0 剩余，不可能）
            c.push(("alts", b));
            nested_base.extend_from_slice(&u32b(1)); // alts = 1
            let mut b = nested_base.clone();
            b.extend_from_slice(&u32b(1)); // steps = 1
            c.push(("steps", b));
            // pratt.ops：pratt=1（恰等 13B 剩余）、primary=0、ops=1 但只剩
            // 5 个 presence 字节（1×6 > 5 → ops 计数准入拒绝）。
            let mut ops_base = header.clone();
            ops_base.extend_from_slice(&u32b(0)); // modes
            ops_base.extend_from_slice(&u32b(0)); // scan rules
            ops_base.extend_from_slice(&u32b(0)); // tokens
            ops_base.extend_from_slice(&u32b(0)); // rules
            ops_base.extend_from_slice(&u32b(0)); // rule_expects
            ops_base.extend_from_slice(&u32b(1)); // pratt = 1
            ops_base.extend_from_slice(&u32b(0)); // primary
            let mut b = ops_base.clone();
            b.extend_from_slice(&u32b(1)); // ops = 1
            b.extend_from_slice(&[0u8; 5]); // 五个可选字段 presence（gate 剩余）
            c.push(("pratt.ops", b));
            // 最小空程序走完全部 14 表到达引用校验（零规则下 start=0 在
            // validate_references 处受控拒绝——证明全部计数都被消费）。
            match program_codec::decode_program(&pfx, LoadBudget::FromWire) {
                Err(RtError::InvalidProgram(m)) => assert!(
                    m.contains("start rule"),
                    "minimal empty program must traverse all tables to validation: {m}"
                ),
                other => panic!("empty grammar has no legal start rule: {other:?}"),
            }
            c
        };
        assert_eq!(cases.len(), 14, "14 count-reserve positions covered");
        for (table, bytes) in cases {
            match program_codec::decode_program(&bytes, LoadBudget::FromWire) {
                Err(RtError::InvalidProgram(m)) => assert!(
                    m.contains(table) && m.contains("count"),
                    "{table} count admission must name the table: {m}"
                ),
                other => panic!("{table} one-item-too-many count must be rejected: {other:?}"),
            }
        }

        // 恰等剩余方向：count×min 恰等剩余（mode_names=0 与 rules=1 的
        // 9 字节规则头）通过计数准入，在后续字段处受控截断——错误不是
        // 计数准入，且全程无预留放大。
        let mut exact = header.clone();
        exact.extend_from_slice(&u32b(0)); // mode_names = 0 → 0×4 == 0 剩余
        match program_codec::decode_program(&exact, LoadBudget::FromWire) {
            Err(RtError::InvalidProgram(m)) => assert!(
                !m.contains("mode_names count"),
                "exact-fit count must pass the gate: {m}"
            ),
            other => panic!("exact-fit count must fail later, controlled: {other:?}"),
        }
        let mut exact_rules = header.clone();
        exact_rules.extend_from_slice(&u32b(0)); // modes
        exact_rules.extend_from_slice(&u32b(0)); // scan rules
        exact_rules.extend_from_slice(&u32b(0)); // tokens
        exact_rules.extend_from_slice(&u32b(1)); // rules = 1 → 1×9 == 9 剩余
        exact_rules.extend_from_slice(&u32b(0)); // name ""
        exact_rules.push(0); // recover
        exact_rules.extend_from_slice(&u32b(0)); // alt_count → 0 剩余
        match program_codec::decode_program(&exact_rules, LoadBudget::FromWire) {
            Err(RtError::InvalidProgram(m)) => assert!(
                !m.contains("rules count"),
                "exact-fit rule count must pass the gate: {m}"
            ),
            other => panic!("exact-fit rule count must fail later, controlled: {other:?}"),
        }

        // FromWire 与显式 Bytes(B)：结果/身份相同；计费等界（最小可行 B
        // 成功、B-1 必须 ResourceExhausted）。
        let bytes = program_codec::encode_canonical(&testkit::mini_program());
        let from_wire =
            program_codec::decode_program(&bytes, LoadBudget::FromWire).expect("FromWire decode");
        let mut b = 0usize;
        let bound = loop {
            match program_codec::decode_program(&bytes, LoadBudget::Bytes(b)) {
                Ok(_) => break b,
                Err(RtError::ResourceExhausted { .. }) => b += 1,
                Err(other) => panic!("unexpected error at budget {b}: {other}"),
            }
        };
        let by_bytes =
            program_codec::decode_program(&bytes, LoadBudget::Bytes(bound)).expect("exact bound");
        assert_eq!(
            from_wire.identity.grammar_id, by_bytes.identity.grammar_id,
            "FromWire and sufficient Bytes(B) must agree on identity"
        );
        assert_eq!(&from_wire, &by_bytes);
        match program_codec::decode_program(&bytes, LoadBudget::Bytes(bound - 1)) {
            Err(RtError::ResourceExhausted { .. }) => {}
            other => panic!("B-1 must be refused as ResourceExhausted: {other:?}"),
        }
    }
}
