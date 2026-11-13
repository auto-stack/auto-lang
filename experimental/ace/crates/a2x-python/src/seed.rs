//! PLAN-1042: deterministic prelude seed codecs for Python RecordText and
//! Collections restoration (`view: None` restore parity with `a2x-c`).

use ace_a2x_core::{A2xReject, SemanticView, Ty};

/// Schema header for Python RecordText prelude seeds.
pub const PY_RT_SEED_SCHEMA: &str = "a2xpy-rt-seed/1";

/// Schema header for Python Collections prelude seeds.
pub const PY_COLL_SEED_SCHEMA: &str = "a2xpy-coll-seed/1";

/// Field type within a Python RecordText seed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SeedFieldTy {
    I32,
    Bool,
    Char,
    Str,
    StrOwned,
    Record(String),
}

impl SeedFieldTy {
    fn token(&self) -> String {
        match self {
            SeedFieldTy::I32 => "i32".into(),
            SeedFieldTy::Bool => "bool".into(),
            SeedFieldTy::Char => "char".into(),
            SeedFieldTy::Str => "str".into(),
            SeedFieldTy::StrOwned => "Str".into(),
            SeedFieldTy::Record(n) => format!("R:{n}"),
        }
    }

    fn parse_token(tok: &str) -> Result<Self, A2xReject> {
        match tok {
            "i32" => Ok(SeedFieldTy::I32),
            "bool" => Ok(SeedFieldTy::Bool),
            "char" => Ok(SeedFieldTy::Char),
            "str" => Ok(SeedFieldTy::Str),
            "Str" => Ok(SeedFieldTy::StrOwned),
            other if other.starts_with("R:") => {
                let name = &other[2..];
                validate_ident_token(name, "record ref")?;
                Ok(SeedFieldTy::Record(name.to_string()))
            }
            other => Err(A2xReject::new(
                "a2py.seed",
                format!("unknown record field token `{other}`"),
            )),
        }
    }
}

/// One record definition captured in a RecordText seed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeedRecord {
    pub name: String,
    pub resource: bool,
    pub fields: Vec<(String, SeedFieldTy)>,
}

/// Field / arm / list element type within a Python Collections seed.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeedCollTy {
    I32,
    Bool,
    StrOwned,
    Record(String),
    Variant(String),
}

impl SeedCollTy {
    fn token(&self) -> String {
        match self {
            SeedCollTy::I32 => "i32".into(),
            SeedCollTy::Bool => "bool".into(),
            SeedCollTy::StrOwned => "Str".into(),
            SeedCollTy::Record(n) => format!("R:{n}"),
            SeedCollTy::Variant(n) => format!("V:{n}"),
        }
    }

    fn parse_token(tok: &str) -> Result<Self, A2xReject> {
        match tok {
            "i32" => Ok(SeedCollTy::I32),
            "bool" => Ok(SeedCollTy::Bool),
            "Str" => Ok(SeedCollTy::StrOwned),
            other if other.starts_with("R:") => {
                let name = &other[2..];
                validate_ident_token(name, "record ref")?;
                Ok(SeedCollTy::Record(name.to_string()))
            }
            other if other.starts_with("V:") => {
                let name = &other[2..];
                validate_ident_token(name, "variant ref")?;
                Ok(SeedCollTy::Variant(name.to_string()))
            }
            other => Err(A2xReject::new(
                "a2py.seed",
                format!("unknown collections seed type token `{other}`"),
            )),
        }
    }

    /// Canonical suffix used for generated list helpers (`__a2x_list_<tag>`).
    pub fn elem_tag(&self) -> String {
        match self {
            SeedCollTy::I32 => "i32".into(),
            SeedCollTy::Bool => "bool".into(),
            SeedCollTy::StrOwned => "str".into(),
            SeedCollTy::Record(n) => format!("rec_{}", sanitize_ident(n)),
            SeedCollTy::Variant(n) => format!("var_{}", sanitize_ident(n)),
        }
    }
}

/// Record definition inside a Collections seed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeedCollRecord {
    pub name: String,
    pub resource: bool,
    pub fields: Vec<(String, SeedCollTy)>,
}

/// Variant definition inside a Collections seed (arms in declaration order).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeedCollVariant {
    pub name: String,
    pub arms: Vec<(String, Option<SeedCollTy>)>,
}

/// Decoded Collections seed containing records, variants, and list element types.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SeedColl {
    pub records: Vec<SeedCollRecord>,
    pub variants: Vec<SeedCollVariant>,
    pub lists: Vec<SeedCollTy>,
}

pub(crate) fn sanitize_ident(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "_".to_string()
    } else {
        out
    }
}

fn validate_ident_token(s: &str, what: &str) -> Result<(), A2xReject> {
    if s.is_empty() || s.chars().any(char::is_whitespace) {
        return Err(A2xReject::new(
            "a2py.seed",
            format!("invalid {what} `{s}` in seed"),
        ));
    }
    Ok(())
}

fn seed_field_ty(view: &SemanticView<'_>, ty: Ty) -> Result<SeedFieldTy, A2xReject> {
    match ty {
        Ty::I32 => Ok(SeedFieldTy::I32),
        Ty::Bool => Ok(SeedFieldTy::Bool),
        Ty::Char => Ok(SeedFieldTy::Char),
        Ty::Str => Ok(SeedFieldTy::Str),
        Ty::StrOwned => Ok(SeedFieldTy::StrOwned),
        Ty::Record(i) => {
            let rec = view
                .record(i)
                .ok_or_else(|| A2xReject::new("a2x.internal", "seed: unknown record index"))?;
            Ok(SeedFieldTy::Record(rec.name.clone()))
        }
        other => Err(A2xReject::unsupported_type(format!(
            "python record-text seed does not admit field type `{other:?}`"
        ))),
    }
}

fn seed_coll_ty(view: &SemanticView<'_>, ty: Ty) -> Result<SeedCollTy, A2xReject> {
    match ty {
        Ty::I32 => Ok(SeedCollTy::I32),
        Ty::Bool => Ok(SeedCollTy::Bool),
        Ty::StrOwned => Ok(SeedCollTy::StrOwned),
        Ty::Record(i) => {
            let rec = view
                .record(i)
                .ok_or_else(|| A2xReject::new("a2x.internal", "seed: unknown record index"))?;
            Ok(SeedCollTy::Record(rec.name.clone()))
        }
        Ty::Variant(i) => {
            let var = view
                .variant(i)
                .ok_or_else(|| A2xReject::new("a2x.internal", "seed: unknown variant index"))?;
            Ok(SeedCollTy::Variant(var.name.clone()))
        }
        other => Err(A2xReject::unsupported_type(format!(
            "python collections seed does not admit type `{other:?}`"
        ))),
    }
}

/// Encode the RecordText nominal record table into canonical seed text.
pub fn encode_rt_seed(view: &SemanticView<'_>) -> Result<String, A2xReject> {
    let mut out = String::new();
    out.push_str(PY_RT_SEED_SCHEMA);
    out.push('\n');
    for rec in view.records() {
        validate_ident_token(&rec.name, "record name")?;
        out.push_str(&format!(
            "R {} {} {}\n",
            rec.name,
            if rec.resource { 1 } else { 0 },
            rec.fields.len()
        ));
        for f in &rec.fields {
            validate_ident_token(&f.name, "field name")?;
            let ty = seed_field_ty(view, f.ty)?;
            out.push_str(&format!("F {} {}\n", f.name, ty.token()));
        }
    }
    Ok(out)
}

/// Strictly decode a RecordText seed produced by [`encode_rt_seed`].
pub fn decode_rt_seed(text: &str) -> Result<Vec<SeedRecord>, A2xReject> {
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or_else(|| A2xReject::new("a2py.seed", "empty RecordText seed"))?;
    if header != PY_RT_SEED_SCHEMA {
        return Err(A2xReject::new(
            "a2py.seed",
            format!("unexpected RecordText seed schema `{header}`"),
        ));
    }
    let mut recs = Vec::new();
    while let Some(line) = lines.next() {
        let parts: Vec<&str> = line.split(' ').collect();
        if parts.len() != 4 || parts[0] != "R" {
            return Err(A2xReject::new(
                "a2py.seed",
                format!("malformed record header `{line}`"),
            ));
        }
        validate_ident_token(parts[1], "record name")?;
        let resource = match parts[2] {
            "0" => false,
            "1" => true,
            other => {
                return Err(A2xReject::new(
                    "a2py.seed",
                    format!("invalid resource flag `{other}`"),
                ));
            }
        };
        let nfields: usize = parts[3]
            .parse()
            .map_err(|_| A2xReject::new("a2py.seed", format!("invalid field count `{}`", parts[3])))?;
        let mut fields = Vec::with_capacity(nfields);
        for _ in 0..nfields {
            let fl = lines
                .next()
                .ok_or_else(|| A2xReject::new("a2py.seed", "truncated field entry"))?;
            let fp: Vec<&str> = fl.split(' ').collect();
            if fp.len() != 3 || fp[0] != "F" {
                return Err(A2xReject::new(
                    "a2py.seed",
                    format!("malformed field entry `{fl}`"),
                ));
            }
            validate_ident_token(fp[1], "field name")?;
            let ty = SeedFieldTy::parse_token(fp[2])?;
            fields.push((fp[1].to_string(), ty));
        }
        recs.push(SeedRecord {
            name: parts[1].to_string(),
            resource,
            fields,
        });
    }
    Ok(recs)
}

/// Encode records, variants, and list element instantiations into Collections seed text.
pub fn encode_coll_seed(view: &SemanticView<'_>) -> Result<String, A2xReject> {
    let mut out = String::new();
    out.push_str(PY_COLL_SEED_SCHEMA);
    out.push('\n');
    for rec in view.records() {
        validate_ident_token(&rec.name, "record name")?;
        out.push_str(&format!(
            "R {} {} {}\n",
            rec.name,
            if rec.resource { 1 } else { 0 },
            rec.fields.len()
        ));
        for f in &rec.fields {
            validate_ident_token(&f.name, "field name")?;
            let ty = seed_coll_ty(view, f.ty)?;
            out.push_str(&format!("F {} {}\n", f.name, ty.token()));
        }
    }
    for var in view.variants() {
        validate_ident_token(&var.name, "variant name")?;
        out.push_str(&format!("V {} {}\n", var.name, var.arms.len()));
        for arm in &var.arms {
            validate_ident_token(&arm.name, "arm name")?;
            let tok = match arm.payload {
                Some(ty) => seed_coll_ty(view, ty)?.token(),
                None => "-".to_string(),
            };
            out.push_str(&format!("A {} {}\n", arm.name, tok));
        }
    }
    let mut seen_lists: Vec<SeedCollTy> = Vec::new();
    for lt in view.list_types() {
        let elem_ty = view.list_elem(lt)?;
        let st = seed_coll_ty(view, elem_ty)?;
        if !seen_lists.contains(&st) {
            out.push_str(&format!("L {}\n", st.token()));
            seen_lists.push(st);
        }
    }
    Ok(out)
}

/// Strictly decode a Collections seed produced by [`encode_coll_seed`].
pub fn decode_coll_seed(text: &str) -> Result<SeedColl, A2xReject> {
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or_else(|| A2xReject::new("a2py.seed", "empty Collections seed"))?;
    if header != PY_COLL_SEED_SCHEMA {
        return Err(A2xReject::new(
            "a2py.seed",
            format!("unexpected Collections seed schema `{header}`"),
        ));
    }
    let mut seed = SeedColl::default();
    while let Some(line) = lines.next() {
        let parts: Vec<&str> = line.split(' ').collect();
        match parts.first().copied() {
            Some("R") => {
                if parts.len() != 4 {
                    return Err(A2xReject::new("a2py.seed", format!("malformed R line `{line}`")));
                }
                validate_ident_token(parts[1], "record name")?;
                let resource = match parts[2] {
                    "0" => false,
                    "1" => true,
                    other => {
                        return Err(A2xReject::new(
                            "a2py.seed",
                            format!("invalid resource flag `{other}`"),
                        ))
                    }
                };
                let nfields: usize = parts[3]
                    .parse()
                    .map_err(|_| A2xReject::new("a2py.seed", "invalid field count"))?;
                let mut fields = Vec::with_capacity(nfields);
                for _ in 0..nfields {
                    let fl = lines
                        .next()
                        .ok_or_else(|| A2xReject::new("a2py.seed", "truncated F line"))?;
                    let fp: Vec<&str> = fl.split(' ').collect();
                    if fp.len() != 3 || fp[0] != "F" {
                        return Err(A2xReject::new("a2py.seed", format!("malformed F line `{fl}`")));
                    }
                    validate_ident_token(fp[1], "field name")?;
                    fields.push((fp[1].to_string(), SeedCollTy::parse_token(fp[2])?));
                }
                seed.records.push(SeedCollRecord {
                    name: parts[1].to_string(),
                    resource,
                    fields,
                });
            }
            Some("V") => {
                if parts.len() != 3 {
                    return Err(A2xReject::new("a2py.seed", format!("malformed V line `{line}`")));
                }
                validate_ident_token(parts[1], "variant name")?;
                let narms: usize = parts[2]
                    .parse()
                    .map_err(|_| A2xReject::new("a2py.seed", "invalid arm count"))?;
                let mut arms = Vec::with_capacity(narms);
                for _ in 0..narms {
                    let al = lines
                        .next()
                        .ok_or_else(|| A2xReject::new("a2py.seed", "truncated A line"))?;
                    let ap: Vec<&str> = al.split(' ').collect();
                    if ap.len() != 3 || ap[0] != "A" {
                        return Err(A2xReject::new("a2py.seed", format!("malformed A line `{al}`")));
                    }
                    validate_ident_token(ap[1], "arm name")?;
                    let payload = if ap[2] == "-" {
                        None
                    } else {
                        Some(SeedCollTy::parse_token(ap[2])?)
                    };
                    arms.push((ap[1].to_string(), payload));
                }
                seed.variants.push(SeedCollVariant {
                    name: parts[1].to_string(),
                    arms,
                });
            }
            Some("L") => {
                if parts.len() != 2 {
                    return Err(A2xReject::new("a2py.seed", format!("malformed L line `{line}`")));
                }
                seed.lists.push(SeedCollTy::parse_token(parts[1])?);
            }
            _ => {
                return Err(A2xReject::new("a2py.seed", format!("unknown seed line `{line}`")));
            }
        }
    }
    Ok(seed)
}

/// Generate Python helper functions for deterministic RecordText clone/free semantics.
pub fn render_rt_seed_helpers(recs: &[SeedRecord]) -> String {
    let mut out = String::new();
    for r in recs {
        let rname = sanitize_ident(&r.name);
        out.push_str(&format!("def a2x_py_rec_free_{rname}(rec):\n"));
        out.push_str("    if rec is None:\n        return\n");
        if r.fields.is_empty() {
            out.push_str("    return\n\n");
        } else {
            for (fname, fty) in &r.fields {
                let sfn = sanitize_ident(fname);
                match fty {
                    SeedFieldTy::Str | SeedFieldTy::StrOwned => {
                        out.push_str(&format!("    a2x_str_free(rec['{sfn}'])\n"));
                    }
                    SeedFieldTy::Record(inner) => {
                        let ir = sanitize_ident(inner);
                        out.push_str(&format!("    a2x_py_rec_free_{ir}(rec['{sfn}'])\n"));
                    }
                    _ => {}
                }
            }
            out.push('\n');
        }
        out.push_str(&format!("def a2x_py_rec_clone_{rname}(src):\n"));
        out.push_str("    out = {}\n    try:\n");
        for (fname, fty) in &r.fields {
            let sfn = sanitize_ident(fname);
            match fty {
                SeedFieldTy::Str | SeedFieldTy::StrOwned => {
                    out.push_str(&format!(
                        "        out['{sfn}'] = a2x_str_clone(src['{sfn}'])\n"
                    ));
                }
                SeedFieldTy::Record(inner) => {
                    let ir = sanitize_ident(inner);
                    out.push_str(&format!(
                        "        out['{sfn}'] = a2x_py_rec_clone_{ir}(src['{sfn}'])\n"
                    ));
                }
                _ => {
                    out.push_str(&format!("        out['{sfn}'] = src['{sfn}']\n"));
                }
            }
        }
        out.push_str(&format!(
            "        return out\n    except A2xTrap:\n        a2x_py_rec_free_{rname}(out)\n        raise\n\n"
        ));
    }
    out
}

/// Generate Python helper functions for deterministic Collections record/variant/list clone/free semantics.
pub fn render_coll_seed_helpers(coll: &SeedColl) -> String {
    let mut out = String::new();
    for r in &coll.records {
        let rname = sanitize_ident(&r.name);
        out.push_str(&format!("def a2x_py_rec_free_{rname}(rec):\n"));
        out.push_str("    if rec is None:\n        return\n");
        let mut wrote = false;
        for (fname, fty) in &r.fields {
            let sfn = sanitize_ident(fname);
            match fty {
                SeedCollTy::StrOwned => {
                    out.push_str(&format!("    a2x_str_free(rec['{sfn}'])\n"));
                    wrote = true;
                }
                SeedCollTy::Record(inner) => {
                    let ir = sanitize_ident(inner);
                    out.push_str(&format!("    a2x_py_rec_free_{ir}(rec['{sfn}'])\n"));
                    wrote = true;
                }
                SeedCollTy::Variant(inner) => {
                    let iv = sanitize_ident(inner);
                    out.push_str(&format!("    a2x_py_var_free_{iv}(rec['{sfn}'])\n"));
                    wrote = true;
                }
                _ => {}
            }
        }
        if !wrote {
            out.push_str("    return\n");
        }
        out.push('\n');

        out.push_str(&format!("def a2x_py_rec_clone_{rname}(src):\n"));
        out.push_str("    out = {}\n    try:\n");
        for (fname, fty) in &r.fields {
            let sfn = sanitize_ident(fname);
            match fty {
                SeedCollTy::StrOwned => {
                    out.push_str(&format!(
                        "        out['{sfn}'] = a2x_str_clone(src['{sfn}'])\n"
                    ));
                }
                SeedCollTy::Record(inner) => {
                    let ir = sanitize_ident(inner);
                    out.push_str(&format!(
                        "        out['{sfn}'] = a2x_py_rec_clone_{ir}(src['{sfn}'])\n"
                    ));
                }
                SeedCollTy::Variant(inner) => {
                    let iv = sanitize_ident(inner);
                    out.push_str(&format!(
                        "        out['{sfn}'] = a2x_py_var_clone_{iv}(src['{sfn}'])\n"
                    ));
                }
                _ => {
                    out.push_str(&format!("        out['{sfn}'] = src['{sfn}']\n"));
                }
            }
        }
        out.push_str(&format!(
            "        return out\n    except A2xTrap:\n        a2x_py_rec_free_{rname}(out)\n        raise\n\n"
        ));
    }

    for v in &coll.variants {
        let vname = sanitize_ident(&v.name);
        out.push_str(&format!("def a2x_py_var_free_{vname}(val):\n"));
        out.push_str("    if val is None:\n        return\n");
        out.push_str("    tag = val.tag\n");
        let mut any_arm = false;
        for (idx, (_, payload)) in v.arms.iter().enumerate() {
            if let Some(pty) = payload {
                let prefix = if any_arm { "elif" } else { "if" };
                any_arm = true;
                match pty {
                    SeedCollTy::StrOwned => {
                        out.push_str(&format!(
                            "    {prefix} tag == {idx}:\n        a2x_str_free(val.payload)\n"
                        ));
                    }
                    SeedCollTy::Record(inner) => {
                        let ir = sanitize_ident(inner);
                        out.push_str(&format!(
                            "    {prefix} tag == {idx}:\n        a2x_py_rec_free_{ir}(val.payload)\n"
                        ));
                    }
                    SeedCollTy::Variant(inner) => {
                        let iv = sanitize_ident(inner);
                        out.push_str(&format!(
                            "    {prefix} tag == {idx}:\n        a2x_py_var_free_{iv}(val.payload)\n"
                        ));
                    }
                    _ => {}
                }
            }
        }
        if !any_arm {
            out.push_str("    return\n");
        }
        out.push('\n');

        out.push_str(&format!("def a2x_py_var_clone_{vname}(src):\n"));
        out.push_str("    tag = src.tag\n");
        let mut cloned_arm = false;
        for (idx, (_, payload)) in v.arms.iter().enumerate() {
            if let Some(pty) = payload {
                let prefix = if cloned_arm { "elif" } else { "if" };
                cloned_arm = true;
                let expr = match pty {
                    SeedCollTy::StrOwned => "a2x_str_clone(src.payload)".to_string(),
                    SeedCollTy::Record(inner) => {
                        format!("a2x_py_rec_clone_{}(src.payload)", sanitize_ident(inner))
                    }
                    SeedCollTy::Variant(inner) => {
                        format!("a2x_py_var_clone_{}(src.payload)", sanitize_ident(inner))
                    }
                    _ => "src.payload".to_string(),
                };
                out.push_str(&format!(
                    "    {prefix} tag == {idx}:\n        return A2xVariant(tag, {expr})\n"
                ));
            }
        }
        if cloned_arm {
            out.push_str("    return A2xVariant(tag, None)\n\n");
        } else {
            out.push_str("    return A2xVariant(tag, src.payload)\n\n");
        }
    }

    for elem in &coll.lists {
        let tag = elem.elem_tag();
        out.push_str(&format!("def a2x_py_list_free_{tag}(lst):\n"));
        out.push_str("    if lst is None:\n        return\n");
        match elem {
            SeedCollTy::StrOwned => {
                out.push_str(
                    "    for item in lst.items:\n        a2x_str_free(item)\n    a2x_list_clear(lst)\n\n",
                );
            }
            SeedCollTy::Record(inner) => {
                let ir = sanitize_ident(inner);
                out.push_str(&format!(
                    "    for item in lst.items:\n        a2x_py_rec_free_{ir}(item)\n    a2x_list_clear(lst)\n\n"
                ));
            }
            SeedCollTy::Variant(inner) => {
                let iv = sanitize_ident(inner);
                out.push_str(&format!(
                    "    for item in lst.items:\n        a2x_py_var_free_{iv}(item)\n    a2x_list_clear(lst)\n\n"
                ));
            }
            _ => {
                out.push_str("    a2x_list_clear(lst)\n\n");
            }
        }
        out.push_str(&format!("def a2x_py_list_clone_{tag}(src):\n"));
        out.push_str("    out = A2xList([])\n    try:\n");
        match elem {
            SeedCollTy::StrOwned => {
                out.push_str("        for item in src.items:\n            out.items.append(a2x_str_clone(item))\n");
            }
            SeedCollTy::Record(inner) => {
                let ir = sanitize_ident(inner);
                out.push_str(&format!(
                    "        for item in src.items:\n            out.items.append(a2x_py_rec_clone_{ir}(item))\n"
                ));
            }
            SeedCollTy::Variant(inner) => {
                let iv = sanitize_ident(inner);
                out.push_str(&format!(
                    "        for item in src.items:\n            out.items.append(a2x_py_var_clone_{iv}(item))\n"
                ));
            }
            _ => {
                out.push_str("        for item in src.items:\n            out.items.append(item)\n");
            }
        }
        out.push_str(&format!(
            "        return out\n    except A2xTrap:\n        a2x_py_list_free_{tag}(out)\n        raise\n\n"
        ));
    }

    out
}
