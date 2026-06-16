//! CompletionSpec ↔ Auto/Atom (.at) text serialization (Plan 315).
//!
//! A self-contained recursive-descent parser/serializer for the nested
//! object-literal subset of Auto used by completion specs. Lives in ash-core
//! (which must not depend on auto-lang), so it does NOT use `auto_lang::parse`.
//!
//! ## Format
//!
//! ```auto
//! spec {
//!     command : "git"
//!     desc    : "Git version control"
//!     flags   : [
//!         flag { short : "b", long : "branch", takes_arg : "name" }
//!     ]
//!     subcommands : [
//!         sub {
//!             name : "checkout"
//!             desc : "Switch branches"
//!             args : [ arg { position : 0, source : "cmd:git branch --list|line", when : "flags_absent:b" } ]
//!             subcommands : []
//!         }
//!     ]
//! }
//! ```
//!
//! `source` / `when` are string-encoded (see `encode_source` / `encode_when`).

use crate::completions::spec::{
    ArgSpec, CompletionSource, CompletionSpec, FlagSpec, ParseMode, SubcommandSpec, WhenCondition,
    ARG_ANY_POSITION,
};

// ── Serialization: CompletionSpec → .at text ─────────────────────────────

/// Serialize a [`CompletionSpec`] to .at text.
pub fn serialize(spec: &CompletionSpec) -> String {
    let mut out = String::new();
    out.push_str("spec {\n");
    write_field(&mut out, "command", &quote(&spec.command), 1);
    if let Some(d) = &spec.desc {
        write_field(&mut out, "desc", &quote(d), 1);
    }
    if !spec.flags.is_empty() {
        write_array(&mut out, "flags", &spec.flags.iter().map(serialize_flag).collect::<Vec<_>>(), 1);
    }
    if !spec.subcommands.is_empty() {
        write_array(
            &mut out,
            "subcommands",
            &spec.subcommands.iter().map(serialize_sub).collect::<Vec<_>>(),
            1,
        );
    }
    if !spec.args.is_empty() {
        write_array(&mut out, "args", &spec.args.iter().map(serialize_arg).collect::<Vec<_>>(), 1);
    }
    out.push_str("}\n");
    out
}

fn serialize_flag(f: &FlagSpec) -> String {
    let mut fields: Vec<(String, String)> = Vec::new();
    if let Some(s) = &f.short {
        fields.push(("short".to_string(), quote(s)));
    }
    if let Some(l) = &f.long {
        fields.push(("long".to_string(), quote(l)));
    }
    if let Some(a) = &f.arg {
        fields.push(("takes_arg".to_string(), quote(a)));
    }
    if let Some(d) = &f.desc {
        fields.push(("desc".to_string(), quote(d)));
    }
    tagged_block("flag", &fields, 2)
}

fn serialize_sub(s: &SubcommandSpec) -> String {
    let mut fields = vec![("name".to_string(), quote(&s.name))];
    if let Some(d) = &s.desc {
        fields.push(("desc".to_string(), quote(d)));
    }
    if !s.flags.is_empty() {
        let items: Vec<String> = s.flags.iter().map(serialize_flag).collect();
        fields.push(("flags".to_string(), array_literal(&items, 3)));
    }
    if !s.args.is_empty() {
        let items: Vec<String> = s.args.iter().map(serialize_arg).collect();
        fields.push(("args".to_string(), array_literal(&items, 3)));
    }
    if !s.subcommands.is_empty() {
        let items: Vec<String> = s.subcommands.iter().map(serialize_sub).collect();
        fields.push(("subcommands".to_string(), array_literal(&items, 3)));
    }
    tagged_block("sub", &fields, 2)
}

fn serialize_arg(a: &ArgSpec) -> String {
    let mut fields = Vec::new();
    if a.position != ARG_ANY_POSITION {
        fields.push(("position".to_string(), a.position.to_string()));
    }
    if a.repeat {
        fields.push(("repeat".to_string(), "true".to_string()));
    }
    if let Some(n) = &a.name {
        fields.push(("name".to_string(), quote(n)));
    }
    if let Some(d) = &a.desc {
        fields.push(("desc".to_string(), quote(d)));
    }
    if let Some(src) = &a.source {
        if let Some(enc) = encode_source(src) {
            fields.push(("source".to_string(), quote(&enc)));
        }
    }
    if let Some(w) = &a.when {
        if let Some(enc) = encode_when(w) {
            fields.push(("when".to_string(), quote(&enc)));
        }
    }
    tagged_block("arg", &fields, 2)
}

/// Encode a [`CompletionSource`] to its string form, or None if not serializable.
fn encode_source(src: &CompletionSource) -> Option<String> {
    match src {
        CompletionSource::Static(list) => Some(format!("static:{}", list.join(","))),
        CompletionSource::Command { cmd, parse } => {
            let p = match parse {
                ParseMode::Line => "line".to_string(),
                ParseMode::Field(n) => format!("field:{}", n),
            };
            Some(format!("cmd:{}|{}", cmd, p))
        }
        CompletionSource::Files { filter: None } => Some("files".to_string()),
        CompletionSource::Files { filter: Some(g) } => Some(format!("files:{}", g)),
        CompletionSource::Directories => Some("dirs".to_string()),
        CompletionSource::Variables => Some("vars".to_string()),
    }
}

/// Encode a [`WhenCondition`] to its string form.
fn encode_when(w: &WhenCondition) -> Option<String> {
    match w {
        WhenCondition::FlagsPresent(f) => Some(format!("flags_present:{}", f.join(","))),
        WhenCondition::FlagsAbsent(f) => Some(format!("flags_absent:{}", f.join(","))),
        WhenCondition::PrevArg(v) => Some(format!("prev:{}", v)),
    }
}

// ── Deserialization: .at text → CompletionSpec ───────────────────────────

/// Parse .at text into a [`CompletionSpec`]. Returns Err on malformed input.
pub fn deserialize(text: &str) -> Result<CompletionSpec, String> {
    let tokens = tokenize(text)?;
    let mut p = Parser { tokens, pos: 0 };
    let top = p.parse_value()?;
    spec_from_atom(&top).ok_or_else(|| "expected a `spec { … }` object".to_string())
}

#[derive(Debug, Clone)]
enum Atom {
    Str(String),
    Obj {
        tag: Option<String>,
        fields: Vec<(String, Atom)>,
    },
    Arr(Vec<Atom>),
}

fn spec_from_atom(atom: &Atom) -> Option<CompletionSpec> {
    let fields = match atom {
        Atom::Obj { fields, .. } => fields,
        _ => return None,
    };
    let command = get_str(fields, "command")?;
    let mut spec = CompletionSpec::new(&command);
    if let Some(d) = get_str(fields, "desc") {
        spec = spec.desc(&d);
    }
    if let Some(Atom::Arr(items)) = get_field(fields, "flags") {
        for it in items {
            if let Some(f) = flag_from_atom(it) {
                spec = spec.flag(f);
            }
        }
    }
    if let Some(Atom::Arr(items)) = get_field(fields, "subcommands") {
        for it in items {
            if let Some(s) = sub_from_atom(it) {
                spec = spec.subcommand(s);
            }
        }
    }
    if let Some(Atom::Arr(items)) = get_field(fields, "args") {
        for it in items {
            if let Some(a) = arg_from_atom(it) {
                spec = spec.arg(a);
            }
        }
    }
    Some(spec)
}

fn flag_from_atom(atom: &Atom) -> Option<FlagSpec> {
    let fields = obj_fields(atom)?;
    Some(FlagSpec {
        short: get_str(fields, "short"),
        long: get_str(fields, "long"),
        desc: get_str(fields, "desc"),
        arg: get_str(fields, "takes_arg"),
    })
}

fn sub_from_atom(atom: &Atom) -> Option<SubcommandSpec> {
    let fields = obj_fields(atom)?;
    let name = get_str(fields, "name")?;
    let mut sub = SubcommandSpec::new(&name);
    if let Some(d) = get_str(fields, "desc") {
        sub = sub.desc(&d);
    }
    if let Some(Atom::Arr(items)) = get_field(fields, "flags") {
        for it in items {
            if let Some(f) = flag_from_atom(it) {
                sub = sub.flag(f);
            }
        }
    }
    if let Some(Atom::Arr(items)) = get_field(fields, "args") {
        for it in items {
            if let Some(a) = arg_from_atom(it) {
                sub = sub.arg(a);
            }
        }
    }
    if let Some(Atom::Arr(items)) = get_field(fields, "subcommands") {
        for it in items {
            if let Some(s) = sub_from_atom(it) {
                sub = sub.subcommand(s);
            }
        }
    }
    Some(sub)
}

fn arg_from_atom(atom: &Atom) -> Option<ArgSpec> {
    let fields = obj_fields(atom)?;
    let position = get_str(fields, "position")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(ARG_ANY_POSITION);
    let repeat = get_str(fields, "repeat")
        .map(|s| s == "true")
        .unwrap_or(false);
    let mut arg = ArgSpec::new(position);
    if repeat {
        arg = arg.repeat();
    }
    if let Some(n) = get_str(fields, "name") {
        arg = arg.name(&n);
    }
    if let Some(d) = get_str(fields, "desc") {
        arg = arg.desc(&d);
    }
    if let Some(enc) = get_str(fields, "source") {
        if let Some(src) = decode_source(&enc) {
            arg = arg.source(src);
        }
    }
    if let Some(enc) = get_str(fields, "when") {
        if let Some(w) = decode_when(&enc) {
            arg = arg.when(w);
        }
    }
    Some(arg)
}

fn decode_source(enc: &str) -> Option<CompletionSource> {
    let enc = enc.trim();
    if let Some(rest) = enc.strip_prefix("static:") {
        let list: Vec<String> = if rest.is_empty() {
            Vec::new()
        } else {
            rest.split(',').map(|s| s.to_string()).collect()
        };
        return Some(CompletionSource::Static(list));
    }
    if let Some(rest) = enc.strip_prefix("cmd:") {
        // cmd:<cmd>|<parse>
        let (cmd, parse) = rest.split_once('|')?;
        let pm = if let Some(n) = parse.strip_prefix("field:") {
            ParseMode::Field(n.parse::<usize>().ok()?)
        } else {
            ParseMode::Line
        };
        return Some(CompletionSource::Command {
            cmd: cmd.to_string(),
            parse: pm,
        });
    }
    if let Some(glob) = enc.strip_prefix("files:") {
        return Some(CompletionSource::Files {
            filter: Some(glob.to_string()),
        });
    }
    match enc {
        "files" => Some(CompletionSource::Files { filter: None }),
        "dirs" => Some(CompletionSource::Directories),
        "vars" => Some(CompletionSource::Variables),
        _ => None,
    }
}

fn decode_when(enc: &str) -> Option<WhenCondition> {
    let enc = enc.trim();
    if let Some(rest) = enc.strip_prefix("flags_present:") {
        let list: Vec<String> = rest.split(',').filter(|s| !s.is_empty()).map(String::from).collect();
        return Some(WhenCondition::FlagsPresent(list));
    }
    if let Some(rest) = enc.strip_prefix("flags_absent:") {
        let list: Vec<String> = rest.split(',').filter(|s| !s.is_empty()).map(String::from).collect();
        return Some(WhenCondition::FlagsAbsent(list));
    }
    if let Some(rest) = enc.strip_prefix("prev:") {
        return Some(WhenCondition::PrevArg(rest.to_string()));
    }
    None
}

// ── Tokenizer ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Colon,
    Comma,
    Str(String),
    Ident(String),
}

fn tokenize(text: &str) -> Result<Vec<Tok>, String> {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut out = Vec::new();
    while i < n {
        let c = chars[i];
        match c {
            ' ' | '\t' | '\n' | '\r' => i += 1,
            '/' if i + 1 < n && chars[i + 1] == '/' => {
                while i < n && chars[i] != '\n' {
                    i += 1;
                }
            }
            '{' => {
                out.push(Tok::LBrace);
                i += 1;
            }
            '}' => {
                out.push(Tok::RBrace);
                i += 1;
            }
            '[' => {
                out.push(Tok::LBracket);
                i += 1;
            }
            ']' => {
                out.push(Tok::RBracket);
                i += 1;
            }
            ':' => {
                out.push(Tok::Colon);
                i += 1;
            }
            ',' => {
                out.push(Tok::Comma);
                i += 1;
            }
            '"' => {
                i += 1;
                let mut s = String::new();
                while i < n && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < n {
                        i += 1;
                        match chars[i] {
                            '"' => s.push('"'),
                            '\\' => s.push('\\'),
                            'n' => s.push('\n'),
                            't' => s.push('\t'),
                            other => s.push(other),
                        }
                    } else {
                        s.push(chars[i]);
                    }
                    i += 1;
                }
                if i >= n {
                    return Err("unterminated string".to_string());
                }
                i += 1; // closing "
                out.push(Tok::Str(s));
            }
            c if c.is_alphabetic() || c == '_' => {
                let start = i;
                while i < n && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '-') {
                    i += 1;
                }
                let id: String = chars[start..i].iter().collect();
                out.push(Tok::Ident(id));
            }
            c if c.is_ascii_digit() || c == '-' => {
                // number-like token (position: 0) — treat as ident string.
                let start = i;
                if c == '-' {
                    i += 1;
                }
                while i < n && (chars[i].is_ascii_digit()) {
                    i += 1;
                }
                let id: String = chars[start..i].iter().collect();
                out.push(Tok::Ident(id));
            }
            _ => return Err(format!("unexpected char {:?}", c)),
        }
    }
    Ok(out)
}

// ── Recursive parser ─────────────────────────────────────────────────────

struct Parser {
    tokens: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.tokens.get(self.pos)
    }
    fn next(&mut self) -> Option<Tok> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    /// Parse a value: string | number | `tag? { … }` (object) | `[ … ]` (array).
    fn parse_value(&mut self) -> Result<Atom, String> {
        // Optional leading tag (ident immediately followed by `{`).
        let tag = if let Some(Tok::Ident(id)) = self.peek() {
            if matches!(self.tokens.get(self.pos + 1), Some(Tok::LBrace)) && !is_keyword(id) {
                let t = id.clone();
                self.pos += 1; // consume ident
                Some(t)
            } else {
                None
            }
        } else {
            None
        };
        match self.peek() {
            Some(Tok::LBrace) => self.parse_object(tag),
            Some(Tok::LBracket) => self.parse_array(),
            Some(Tok::Str(_)) => {
                if let Some(Tok::Str(s)) = self.next() {
                    Ok(Atom::Str(s))
                } else {
                    unreachable!()
                }
            }
            Some(Tok::Ident(_)) => {
                // bare ident value (e.g. position: 0, repeat: true) → treat as string
                if let Some(Tok::Ident(s)) = self.next() {
                    Ok(Atom::Str(s))
                } else {
                    unreachable!()
                }
            }
            _ => Err("expected value".to_string()),
        }
    }

    fn parse_object(&mut self, tag: Option<String>) -> Result<Atom, String> {
        // consume `{`
        self.next(); // LBrace
        let mut fields = Vec::new();
        loop {
            // skip commas
            while matches!(self.peek(), Some(Tok::Comma)) {
                self.next();
            }
            match self.peek() {
                None => return Err("unterminated object".to_string()),
                Some(Tok::RBrace) => {
                    self.next();
                    break;
                }
                Some(Tok::Ident(_)) => {
                    let key = match self.next() {
                        Some(Tok::Ident(s)) => s,
                        _ => unreachable!(),
                    };
                    // expect ':'
                    if !matches!(self.peek(), Some(Tok::Colon)) {
                        return Err(format!("expected ':' after key '{}'", key));
                    }
                    self.next(); // Colon
                    let val = self.parse_value()?;
                    fields.push((key, val));
                }
                _ => return Err("expected key or '}'".to_string()),
            }
        }
        Ok(Atom::Obj { tag, fields })
    }

    fn parse_array(&mut self) -> Result<Atom, String> {
        self.next(); // LBracket
        let mut items = Vec::new();
        loop {
            while matches!(self.peek(), Some(Tok::Comma)) {
                self.next();
            }
            match self.peek() {
                None => return Err("unterminated array".to_string()),
                Some(Tok::RBracket) => {
                    self.next();
                    break;
                }
                _ => items.push(self.parse_value()?),
            }
        }
        Ok(Atom::Arr(items))
    }
}

/// Words that are value literals, not object tags (currently none — but reserved
/// so `true`/`false`/numbers never start an object).
fn is_keyword(_id: &str) -> bool {
    false
}

// ── Atom field helpers ───────────────────────────────────────────────────

fn obj_fields(atom: &Atom) -> Option<&[(String, Atom)]> {
    if let Atom::Obj { fields, .. } = atom {
        Some(fields)
    } else {
        None
    }
}
fn get_field<'a>(fields: &'a [(String, Atom)], key: &str) -> Option<&'a Atom> {
    fields.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}
fn get_str(fields: &[(String, Atom)], key: &str) -> Option<String> {
    match get_field(fields, key)? {
        Atom::Str(s) => Some(s.clone()),
        _ => None,
    }
}

// ── Text emission helpers ────────────────────────────────────────────────

fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("    ");
    }
}

fn write_field(out: &mut String, key: &str, val: &str, level: usize) {
    indent(out, level);
    out.push_str(key);
    out.push_str(" : ");
    out.push_str(val);
    out.push('\n');
}

fn write_array(out: &mut String, key: &str, items: &[String], level: usize) {
    indent(out, level);
    out.push_str(key);
    out.push_str(" : [\n");
    for it in items {
        out.push_str(it);
        out.push('\n');
    }
    indent(out, level);
    out.push_str("]\n");
}

fn array_literal(items: &[String], level: usize) -> String {
    let mut out = String::from("[\n");
    for it in items {
        out.push_str(it);
        out.push('\n');
    }
    indent(&mut out, level);
    out.push_str("]");
    out
}

fn tagged_block(tag: &str, fields: &[(String, String)], level: usize) -> String {
    let mut out = String::new();
    indent(&mut out, level);
    out.push_str(tag);
    out.push_str(" { ");
    for (i, (k, v)) in fields.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(k);
        out.push_str(" : ");
        out.push_str(v);
    }
    out.push_str(" }");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_spec() -> CompletionSpec {
        CompletionSpec::new("git")
            .desc("Git version control")
            .flag(FlagSpec::both("b", "branch").takes_arg("name").desc("Create branch"))
            .subcommand(
                SubcommandSpec::new("checkout")
                    .desc("Switch branches")
                    .flag(FlagSpec::long("track").takes_arg("remote"))
                    .arg(
                        ArgSpec::new(0)
                            .desc("Branch")
                            .when(WhenCondition::flags_absent(&["b"]))
                            .source(CompletionSource::command("git branch --list")),
                    ),
            )
    }

    #[test]
    fn round_trip_basic() {
        let spec = sample_spec();
        let text = serialize(&spec);
        let back = deserialize(&text).expect("deserialize failed");
        assert_eq!(back.command, "git");
        assert_eq!(back.desc.as_deref(), Some("Git version control"));
        // flag b/branch
        let f = back.flags.iter().find(|f| f.long.as_deref() == Some("branch")).unwrap();
        assert_eq!(f.short.as_deref(), Some("b"));
        assert_eq!(f.arg.as_deref(), Some("name"));
        // subcommand checkout
        let sub = back.subcommands.iter().find(|s| s.name == "checkout").unwrap();
        assert_eq!(sub.desc.as_deref(), Some("Switch branches"));
        assert!(sub.flags.iter().any(|f| f.long.as_deref() == Some("track")));
        let arg = &sub.args[0];
        assert_eq!(arg.position, 0);
        assert!(matches!(arg.source, Some(CompletionSource::Command { .. })));
        assert!(matches!(arg.when, Some(WhenCondition::FlagsAbsent(_))));
    }

    #[test]
    fn source_encoding_roundtrip() {
        let cases = vec![
            CompletionSource::Static(vec!["a".into(), "b".into()]),
            CompletionSource::command("git branch --list"),
            CompletionSource::command_field("ls", 1),
            CompletionSource::files(),
            CompletionSource::files_with_filter("*.rs"),
            CompletionSource::Directories,
            CompletionSource::Variables,
        ];
        for src in cases {
            let enc = encode_source(&src).expect("encode");
            let back = decode_source(&enc).expect("decode");
            // Compare via re-encoding (CompletionSource isn't Eq).
            let enc2 = encode_source(&back).expect("re-encode");
            assert_eq!(enc, enc2, "source roundtrip mismatch");
        }
    }

    #[test]
    fn when_encoding_roundtrip() {
        let cases = vec![
            WhenCondition::flags_present(&["a", "b"]),
            WhenCondition::flags_absent(&["m"]),
            WhenCondition::prev_arg("--message"),
        ];
        for w in cases {
            let enc = encode_when(&w).unwrap();
            let back = decode_when(&enc).unwrap();
            let enc2 = encode_when(&back).unwrap();
            assert_eq!(enc, enc2);
        }
    }

    #[test]
    fn deserialize_minimal() {
        let text = "spec { command : \"rg\" }\n";
        let spec = deserialize(text).unwrap();
        assert_eq!(spec.command, "rg");
        assert!(spec.flags.is_empty());
    }

    #[test]
    fn deserialize_ignores_comments() {
        let text = "// a comment\nspec {\n  command : \"x\"  // inline\n}\n";
        let spec = deserialize(text).unwrap();
        assert_eq!(spec.command, "x");
    }

    #[test]
    fn deserialize_malformed_returns_err() {
        assert!(deserialize("not valid").is_err());
        assert!(deserialize("spec { command : ").is_err());
    }

    #[test]
    fn empty_flags_subcommands_omitted() {
        let spec = CompletionSpec::new("solo");
        let text = serialize(&spec);
        assert!(!text.contains("flags"));
        assert!(!text.contains("subcommands"));
    }
}
