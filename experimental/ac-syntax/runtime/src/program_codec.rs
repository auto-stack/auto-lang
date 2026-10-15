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
                }
                w.push(match s.repeat {
                    Repeat::Once => 0,
                    Repeat::Optional => 1,
                    Repeat::ZeroOrMore => 2,
                    Repeat::ListPlain => 3,
                    Repeat::ListNewline => 4,
                });
                w.push(s.same_line as u8);
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
    }

    w.extend_from_slice(&p.start.to_le_bytes());

    put_u32(&mut w, p.field_names.len() as u32);
    for f in &p.field_names {
        put_str(&mut w, f);
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

/// 严格往返（语言包装载：静态 Atom → 程序 → 实验二进制 → 程序）。
/// 身份字段（PackageIdentity）不序列化——由内容重算，装载后必须一致。
pub fn decode_canonical(bytes: &[u8]) -> Result<GrammarProgram, String> {
    let mut r = Reader { b: bytes, pos: 0 };
    let v = r.u16()?;
    if v != PROGRAM_CODEC_VERSION {
        return Err(format!("program codec version {v} != {PROGRAM_CODEC_VERSION}"));
    }
    let schema_version = r.u32()?;
    let name = r.str()?;
    let revision = r.u64()?;

    let n = r.u32()? as usize;
    let mut mode_names = Vec::with_capacity(n);
    for _ in 0..n {
        mode_names.push(r.str()?);
    }
    let n = r.u32()? as usize;
    let mut rules = Vec::with_capacity(n);
    for _ in 0..n {
        let mode = r.u32()?;
        let matcher = get_matcher(&mut r)?;
        let token = r.u32()?;
        let trivia = r.u8()? != 0;
        let next_mode = r.u32()?;
        rules.push(ScanRule { mode, matcher, token, trivia, next_mode });
    }

    let n = r.u32()? as usize;
    let mut tokens = Vec::with_capacity(n);
    for _ in 0..n {
        tokens.push(TokenSpec { name: r.str()? });
    }

    let n = r.u32()? as usize;
    let mut grammar_rules = Vec::with_capacity(n);
    for _ in 0..n {
        let rname = r.str()?;
        let recover = r.u8()? != 0;
        let na = r.u32()? as usize;
        let mut alts = Vec::with_capacity(na);
        for _ in 0..na {
            let ns = r.u32()? as usize;
            let mut steps = Vec::with_capacity(ns);
            for _ in 0..ns {
                let atom = match r.u8()? {
                    0 => Atom::Tok(r.u32()?),
                    1 => Atom::Rule(r.u32()?),
                    2 => Atom::Pratt(r.u32()?),
                    t => return Err(format!("bad atom tag {t}")),
                };
                let repeat = match r.u8()? {
                    0 => Repeat::Once,
                    1 => Repeat::Optional,
                    2 => Repeat::ZeroOrMore,
                    3 => Repeat::ListPlain,
                    4 => Repeat::ListNewline,
                    t => return Err(format!("bad repeat tag {t}")),
                };
                let same_line = r.u8()? != 0;
                let field = if r.u8()? != 0 { Some(r.u32()?) } else { None };
                steps.push(Step { atom, field, repeat, same_line });
            }
            alts.push(Alt { steps });
        }
        grammar_rules.push(RuleSpec { name: rname, alts, recover });
    }

    let n = r.u32()? as usize;
    let mut pratt = Vec::with_capacity(n);
    for _ in 0..n {
        let primary = r.u32()?;
        let no = r.u32()? as usize;
        let mut ops = Vec::with_capacity(no);
        for _ in 0..no {
            let op = r.u32()?;
            let binding = r.u8()?;
            let assoc = match r.u8()? {
                0 => Assoc::Left,
                1 => Assoc::Right,
                t => return Err(format!("bad assoc tag {t}")),
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
            Some(CallParenRule { open, close, newline_free_inside, callee_same_line })
        } else {
            None
        };
        let call_args = opt_tok(&mut r)?;
        pratt.push(PrattSpec { primary, ops, group_open, group_close, call_paren, call_args });
    }

    let start = r.u32()?;

    let n = r.u32()? as usize;
    let mut field_names = Vec::with_capacity(n);
    for _ in 0..n {
        field_names.push(r.str()?);
    }

    let n = r.u32()? as usize;
    let mut stmt_end = Vec::with_capacity(n);
    for _ in 0..n {
        stmt_end.push(r.u32()?);
    }
    let n = r.u32()? as usize;
    let mut sep_consume = Vec::with_capacity(n);
    for _ in 0..n {
        sep_consume.push(r.u32()?);
    }
    let significant = r.u8()? != 0;
    let n = r.u32()? as usize;
    let mut sync = Vec::with_capacity(n);
    for _ in 0..n {
        sync.push(r.u32()?);
    }
    let outline = if r.u8()? != 0 {
        let decl_rule = r.str()?;
        let name_field = r.str()?;
        let params_field = if r.u8()? != 0 { Some(r.str()?) } else { None };
        Some(OutlineSpec { decl_rule, name_field, params_field })
    } else {
        None
    };
    r.end()?;

    let scanner = ScannerProgram { mode_names, rules };
    // 防御性边界检查（错位字节→受控错误，不越界 panic）。
    for r in &scanner.rules {
        if (r.token as usize) >= tokens.len() {
            return Err("scan rule references unknown token".into());
        }
        if (r.mode as usize) >= scanner.mode_names.len()
            || (r.next_mode as usize) >= scanner.mode_names.len()
        {
            return Err("scan rule references unknown mode".into());
        }
    }
    if (start as usize) >= grammar_rules.len() {
        return Err("start rule out of range".into());
    }

    let mut p = GrammarProgram {
        name,
        revision,
        schema_version,
        scanner,
        tokens,
        rules: grammar_rules,
        pratt,
        field_names,
        start,
        newline: NewlinePolicy { significant, stmt_end, sep_consume },
        recovery: RecoverySpec { sync },
        outline,
        identity: crate::identity::PackageIdentity {
            name: String::new(),
            grammar_id: crate::identity::GrammarId(String::new()),
        },
    };
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

fn opt_tok(r: &mut Reader) -> Result<Option<u32>, String> {
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
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        if self.pos + n > self.b.len() {
            return Err("truncated program".into());
        }
        let s = &self.b[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, String> {
        let s = self.take(2)?;
        Ok(u16::from_le_bytes([s[0], s[1]]))
    }
    fn u32(&mut self) -> Result<u32, String> {
        let s = self.take(4)?;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
    fn u64(&mut self) -> Result<u64, String> {
        let s = self.take(8)?;
        let mut a = [0u8; 8];
        a.copy_from_slice(s);
        Ok(u64::from_le_bytes(a))
    }
    fn str(&mut self) -> Result<String, String> {
        let n = self.u32()? as usize;
        let s = self.take(n)?;
        String::from_utf8(s.to_vec()).map_err(|_| "bad utf-8 in program".to_string())
    }
    fn end(&mut self) -> Result<(), String> {
        if self.pos != self.b.len() {
            return Err("trailing bytes in program".into());
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
        Matcher::Charset { first, cont, min_len } => {
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
    }
}

fn get_matcher(r: &mut Reader) -> Result<Matcher, String> {
    let cs = |lo: u64, hi: u64, non_ascii: bool| CharSet {
        bits: [lo, hi],
        non_ascii,
    };
    Ok(match r.u8()? {
        0 => Matcher::Literal(r.str()?),
        1 => {
            let fl = u64::from_le_bytes(r.take(8)?.try_into().unwrap());
            let fh = u64::from_le_bytes(r.take(8)?.try_into().unwrap());
            let fna = r.u8()? != 0;
            let cl = u64::from_le_bytes(r.take(8)?.try_into().unwrap());
            let ch = u64::from_le_bytes(r.take(8)?.try_into().unwrap());
            let cna = r.u8()? != 0;
            let min_len = r.u32()?;
            Matcher::Charset { first: cs(fl, fh, fna), cont: cs(cl, ch, cna), min_len }
        }
        2 => Matcher::LineComment(r.str()?),
        3 => {
            let start = r.str()?;
            let end = r.str()?;
            let nested = r.u8()? != 0;
            Matcher::BlockComment { start, end, nested }
        }
        4 => {
            let quote = r.u8()?;
            let escape = r.u8()? != 0;
            Matcher::StringLit { quote, escape }
        }
        t => return Err(format!("bad matcher tag {t}")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program_codec;
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
}
