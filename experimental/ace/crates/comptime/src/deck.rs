//! The `cte817/1` atom-data deck: the strict, line-oriented request
//! document the `cte/scalar-query-v1` recipe consumes. Unknown
//! directives, duplicate ids, malformed literals and unterminated source
//! blocks are all controlled parse errors — this parser owns the recipe
//! body validation (never the .ad case format itself).

use ace_comptime_contract::{
    CteAdmissionError, CteBindingV1, CteCode, CteExecMode, CteExpectType, CteRequestV1, CteScalar,
    CteSourceProfile, CteValue, QueryNodeKind, QueryNodeV1, CTE_META_CAP, CTE_RECIPE,
    CTE_REQUEST_SCHEMA, MAX_REQUEST_INPUT_BYTES,
};

/// Parsed deck (identical shape to `CteRequestV1`).
pub type Deck = CteRequestV1;

#[derive(Debug)]
struct DeckError(String);

impl From<DeckError> for CteAdmissionError {
    fn from(e: DeckError) -> Self {
        CteAdmissionError::new(CteCode::WrongSchema, format!("deck: {}", e.0))
    }
}

fn err<T>(msg: impl Into<String>) -> Result<T, DeckError> {
    Err(DeckError(msg.into()))
}

fn chr_newline() -> char {
    0x0A as u8 as char
}

fn parse_scalar(s: &str) -> Result<CteScalar, DeckError> {
    let s = s.trim();
    if s == "true" {
        return Ok(CteValue::Bool(true));
    }
    if s == "false" {
        return Ok(CteValue::Bool(false));
    }
    if let Ok(v) = s.parse::<i32>() {
        return Ok(CteValue::I32(v));
    }
    err(format!("`{s}` is not an i32/bool literal"))
}

/// `id, id, 42, true` → argument node list (ids stay `QueryRef`).
fn parse_arg_list(s: &str) -> Result<Vec<QueryNodeKind>, DeckError> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for part in s.split(',') {
        let p = part.trim();
        if p.is_empty() {
            return err("empty argument in call list");
        }
        if p.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            && p.parse::<i32>().is_err()
            && p != "true"
            && p != "false"
        {
            out.push(QueryNodeKind::QueryRef(p.to_string()));
        } else {
            out.push(QueryNodeKind::Literal(parse_scalar(p)?));
        }
    }
    Ok(out)
}

fn is_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !s.chars().next().unwrap().is_ascii_digit()
}

/// Parse one `cte817/1` deck document.
pub fn parse_deck(text: &str) -> Result<Deck, CteAdmissionError> {
    parse_inner(text).map_err(Into::into)
}

fn parse_inner(text: &str) -> Result<Deck, DeckError> {
    if text.len() > MAX_REQUEST_INPUT_BYTES {
        return err(format!(
            "deck exceeds the {} byte input ceiling",
            MAX_REQUEST_INPUT_BYTES
        ));
    }
    let lines: Vec<&str> = text.lines().collect();
    let mut idx = 0usize;
    // Skip leading comments/blank lines.
    while idx < lines.len()
        && (lines[idx].trim().is_empty() || lines[idx].trim_start().starts_with('#'))
    {
        idx += 1;
    }
    if idx >= lines.len() || lines[idx].trim() != "deck cte817/1" {
        return err("first directive must be `deck cte817/1`");
    }
    idx += 1;

    let mut module: Option<String> = None;
    let mut profile = CteSourceProfile::Core;
    let mut exec = CteExecMode::Closure;
    let mut meta_source: Option<String> = None;
    let mut source: Option<String> = None;
    let mut entry: Option<String> = None;
    let mut native = true;
    let mut caps: Vec<String> = Vec::new();
    let mut queries: Vec<QueryNodeV1> = Vec::new();
    let mut bindings: Vec<CteBindingV1> = Vec::new();

    while idx < lines.len() {
        let line = lines[idx].trim();
        if line.is_empty() || line.starts_with('#') {
            idx += 1;
            continue;
        }
        if line == "meta <<" {
            idx += 1;
            let mut buf: Vec<&str> = Vec::new();
            loop {
                if idx >= lines.len() {
                    return err("unterminated `meta <<` block (missing `>>`)");
                }
                if lines[idx].trim() == ">>" {
                    idx += 1;
                    break;
                }
                buf.push(lines[idx]);
                idx += 1;
            }
            if meta_source.is_some() {
                return err("duplicate `meta` block");
            }
            let mut m = buf.join(
                "
",
            );
            m.push(chr_newline());
            meta_source = Some(m);
            continue;
        }
        if line == "source <<" {
            idx += 1;
            let mut buf: Vec<&str> = Vec::new();
            loop {
                if idx >= lines.len() {
                    return err("unterminated `source <<` block (missing `>>`)");
                }
                if lines[idx].trim() == ">>" {
                    idx += 1;
                    break;
                }
                buf.push(lines[idx]);
                idx += 1;
            }
            if source.is_some() {
                return err("duplicate `source` block");
            }
            let mut s = buf.join("\n");
            s.push('\n');
            source = Some(s);
            continue;
        }
        let (head, rest) = match line.split_once(char::is_whitespace) {
            Some((h, r)) => (h, r.trim()),
            None => (line, ""),
        };
        match head {
            "module" => {
                if !is_ident(rest) {
                    return err(format!("`module {rest}` is not an identifier"));
                }
                if module.is_some() {
                    return err("duplicate `module` directive");
                }
                module = Some(rest.to_string());
            }
            "entry" => {
                if !is_ident(rest) {
                    return err(format!("`entry {rest}` is not an identifier"));
                }
                if entry.is_some() {
                    return err("duplicate `entry` directive");
                }
                entry = Some(rest.to_string());
            }
            "caps" => {
                for c in rest.split_whitespace() {
                    if caps.contains(&c.to_string()) {
                        return err(format!("duplicate capability `{c}`"));
                    }
                    caps.push(c.to_string());
                }
            }
            "meta" => {
                return err("`meta` takes a `<< ... >>` block, not arguments");
            }
            "exec" => {
                exec = CteExecMode::parse(rest)
                    .ok_or_else(|| DeckError(format!("`exec {rest}` must be closure|snapshot")))?;
            }
            "profile" => {
                profile = CteSourceProfile::parse(rest)
                    .ok_or_else(|| DeckError(format!("`profile {rest}` must be core|record")))?;
            }
            "native" => match rest {
                "on" => native = true,
                "off" => native = false,
                _ => return err(format!("`native {rest}` must be on|off")),
            },
            "query" => {
                // query <id> = <kind>
                let Some((id, kind)) = rest.split_once('=') else {
                    return err(format!("`{line}` lacks `=`"));
                };
                let id = id.trim();
                if !is_ident(id) {
                    return err(format!("query id `{id}` is not an identifier"));
                }
                if queries.iter().any(|q| q.id == id) {
                    return err(format!("duplicate query id `{id}`"));
                }
                let kind = kind.trim();
                let node = if let Some(args) = kind.strip_prefix("call ") {
                    let (callee, argstr) = args
                        .split_once('(')
                        .and_then(|(c, r)| r.strip_suffix(')').map(|a| (c.trim(), a)))
                        .ok_or_else(|| DeckError(format!("`{kind}` is not call <fn>(<args>)")))?;
                    if !is_ident(callee) {
                        return err(format!("callee `{callee}` is not an identifier"));
                    }
                    QueryNodeKind::Call {
                        callee: callee.to_string(),
                        args: parse_arg_list(argstr)?,
                    }
                } else if let Some(rec) = kind.strip_prefix("field_count ") {
                    let rec = rec.trim();
                    if !is_ident(rec) {
                        return err(format!("record `{rec}` is not an identifier"));
                    }
                    QueryNodeKind::FieldCount {
                        record: rec.to_string(),
                    }
                } else if let Some(rest) = kind.strip_prefix("has_field ") {
                    let mut it = rest.split_whitespace();
                    let (Some(rec), Some(field), None) = (it.next(), it.next(), it.next()) else {
                        return err(format!("`{kind}` must be has_field <record> <field>"));
                    };
                    if !is_ident(rec) || !is_ident(field) {
                        return err(format!("`{kind}` has non-identifier operands"));
                    }
                    QueryNodeKind::HasField {
                        record: rec.to_string(),
                        field: field.to_string(),
                    }
                } else {
                    QueryNodeKind::Literal(parse_scalar(kind)?)
                };
                queries.push(QueryNodeV1 {
                    id: id.to_string(),
                    kind: node,
                });
            }
            "bind" => {
                // bind <owner>.<binding> = <query> expect <ty> placeholder <lit>
                let Some((anchor, rest)) = rest.split_once('=') else {
                    return err(format!("`{line}` lacks `=`"));
                };
                let anchor = anchor.trim();
                let Some((owner, binding)) = anchor.split_once('.') else {
                    return err(format!("`{anchor}` must be <owner>.<binding>"));
                };
                let owner = owner.trim();
                let binding = binding.trim();
                if !is_ident(owner) || !is_ident(binding) {
                    return err(format!("`{anchor}` has non-identifier parts"));
                }
                let mut it = rest.split_whitespace();
                let (Some(query), Some(expect_kw), Some(ty), Some(ph_kw), Some(ph)) =
                    (it.next(), it.next(), it.next(), it.next(), it.next())
                else {
                    return err(format!(
                        "`{rest}` must be <query> expect <i32|bool> placeholder <literal>"
                    ));
                };
                if it.next().is_some() {
                    return err(format!("`{rest}` has trailing tokens"));
                }
                if expect_kw != "expect" || ph_kw != "placeholder" {
                    return err("bind syntax: <query> expect <type> placeholder <literal>");
                }
                let Some(expect) = CteExpectType::parse(ty) else {
                    return err(format!("`{ty}` is not i32|bool"));
                };
                let placeholder = parse_scalar(ph)?;
                if !is_ident(query) {
                    return err(format!("query id `{query}` is not an identifier"));
                }
                let ct_ref = format!("{owner}.{binding}");
                if bindings.iter().any(|b| b.ct_ref == ct_ref) {
                    return err(format!("duplicate bind for `{ct_ref}`"));
                }
                bindings.push(CteBindingV1 {
                    ct_ref,
                    owner_fn: owner.to_string(),
                    binding_name: binding.to_string(),
                    placeholder,
                    expect,
                    source_query: query.to_string(),
                });
            }
            other => return err(format!("unknown directive `{other}`")),
        }
        idx += 1;
    }

    let Some(module) = module else {
        return err("missing `module` directive");
    };
    let Some(source) = source else {
        return err("missing `source` block");
    };
    let Some(entry) = entry else {
        return err("missing `entry` directive");
    };
    // Capability admission: only the one granted capability exists.
    for c in &caps {
        if c != CTE_META_CAP {
            return err(format!(
                "unknown capability `{c}` (only {CTE_META_CAP} exists)"
            ));
        }
    }
    Ok(CteRequestV1 {
        schema: CTE_REQUEST_SCHEMA,
        recipe: CTE_RECIPE,
        module,
        source,
        queries,
        bindings,
        entry,
        caps,
        native,
        profile,
        exec,
        meta_source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST: &str = r#"
# first vertical: sum(20,22)=42
deck cte817/1
module main
source <<
fn cte_sum(a: i32, b: i32) -> i32 { return a + b; }
fn main() -> i32 { let ct = 0; return ct; }
>>
query q1 = call cte_sum(20, 22)
bind main.ct = q1 expect i32 placeholder 0
entry main
"#;

    #[test]
    fn deck_first_vertical_parses() {
        let d = parse_deck(FIRST).unwrap();
        assert_eq!(d.module, "main");
        assert_eq!(d.entry, "main");
        assert_eq!(d.queries.len(), 1);
        assert_eq!(d.bindings.len(), 1);
        assert!(d.native);
        assert_eq!(d.schema, CTE_REQUEST_SCHEMA);
        assert_eq!(d.recipe, CTE_RECIPE);
    }

    #[test]
    fn deck_unknown_directive_rejected() {
        let bad = "deck cte817/1\nmodule m\nfrobnicate x\n";
        assert!(parse_deck(bad).is_err());
    }

    #[test]
    fn deck_unterminated_source_rejected() {
        let bad = "deck cte817/1\nmodule m\nsource <<\nfn f() -> i32 { return 1; }\n";
        let e = parse_deck(bad).unwrap_err();
        assert!(e.message.contains("unterminated"));
    }

    #[test]
    fn deck_unknown_cap_rejected() {
        let bad = "deck cte817/1\nmodule m\nsource <<\nfn f() -> i32 { return 1; }\n>>\ncaps not.a.cap\nentry f\n";
        assert!(parse_deck(bad).is_err());
    }
}
