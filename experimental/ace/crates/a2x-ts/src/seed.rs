//! PLAN-976: deterministic prelude seed codecs and JavaScript helper renderers
//! for the TypeScript/JavaScript RecordText and Collections faces.
//!
//! The WIRE FORMAT is inherited verbatim from the PLAN-1042 Python leg
//! (`a2x-python::seed`): a one-line schema header followed by line-oriented
//! records with space-separated tokens (`R`/`F` for record definitions, `V`/`A`
//! for variants, `L` for list element instantiations). Only the schema headers
//! name this target (`a2ts-rt-seed/1`, `a2ts-coll-seed/1`) and only the
//! rejection code changes (`a2ts.seed`); every decode rule stays as strict as
//! the Python codec (bad header, unknown line token, malformed resource flag /
//! count / field type and truncated records are all named rejections).
//!
//! The renderers are the target DIFFERENCE: they emit ES module helper
//! functions instead of Python ones. Composite values (records, variants,
//! lists) are GC-managed in this target, while owned `Str` buffers are
//! ledger-tracked and released explicitly, so a generated record helper pair
//! clones every owned `Str` field with `a2x_str_clone`, releases it with
//! `a2x_str_free`, and recurses into nested record fields through their own
//! helpers. Borrowed `str` views and scalars are copied by value/reference and
//! are never freed.

use ace_a2x_core::{A2xReject, SemanticView, Ty};

/// Schema header for this target's RecordText prelude seeds.
pub const TS_RT_SEED_SCHEMA: &str = "a2ts-rt-seed/1";

/// Schema header for this target's Collections prelude seeds.
pub const TS_COLL_SEED_SCHEMA: &str = "a2ts-coll-seed/1";

/// The single rejection code of both seed codecs.
const SEED_REJECT_CODE: &str = "a2ts.seed";

fn seed_reject(detail: impl Into<String>) -> A2xReject {
    A2xReject::new(SEED_REJECT_CODE, detail)
}

/// Field type within a RecordText seed.
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
            other => Err(seed_reject(format!("unknown record field token `{other}`"))),
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

/// Field / arm / list element type within a Collections seed.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeedCollTy {
    I32,
    I64,
    U64,
    Bool,
    F32,
    Str,
    StrOwned,
    Nil,
    Dyn,
    Object(u32),
    Record(String),
    Variant(String),
    List(u32),
}

impl SeedCollTy {
    fn token(&self) -> String {
        match self {
            SeedCollTy::I32 => "i32".into(),
            SeedCollTy::I64 => "i64".into(),
            SeedCollTy::U64 => "u64".into(),
            SeedCollTy::Bool => "bool".into(),
            SeedCollTy::F32 => "f32".into(),
            SeedCollTy::Str => "str".into(),
            SeedCollTy::StrOwned => "Str".into(),
            SeedCollTy::Nil => "nil".into(),
            SeedCollTy::Dyn => "dyn".into(),
            SeedCollTy::Object(index) => format!("O:{index}"),
            SeedCollTy::Record(n) => format!("R:{n}"),
            SeedCollTy::Variant(n) => format!("V:{n}"),
            SeedCollTy::List(index) => format!("L:{index}"),
        }
    }

    fn parse_token(tok: &str) -> Result<Self, A2xReject> {
        match tok {
            "i32" => Ok(SeedCollTy::I32),
            "i64" => Ok(SeedCollTy::I64),
            "u64" => Ok(SeedCollTy::U64),
            "bool" => Ok(SeedCollTy::Bool),
            "f32" => Ok(SeedCollTy::F32),
            "str" => Ok(SeedCollTy::Str),
            "Str" => Ok(SeedCollTy::StrOwned),
            "nil" => Ok(SeedCollTy::Nil),
            "dyn" => Ok(SeedCollTy::Dyn),
            other if other.starts_with("O:") => other[2..]
                .parse::<u32>()
                .ok()
                .filter(|index| index.to_string() == other[2..])
                .map(SeedCollTy::Object)
                .ok_or_else(|| seed_reject(format!("invalid object type reference `{other}`"))),
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
            other if other.starts_with("L:") => other[2..]
                .parse::<u32>()
                .ok()
                .filter(|index| index.to_string() == other[2..])
                .map(SeedCollTy::List)
                .ok_or_else(|| seed_reject(format!("invalid list type reference `{other}`"))),
            other => Err(seed_reject(format!(
                "unknown collections seed type token `{other}`"
            ))),
        }
    }

    /// Canonical suffix used for generated list helper tags
    /// (`__a2x_list_<tag>`). Variant/list tables are carried by the seed for
    /// identity and diagnostics only; this target does not emit list helpers.
    pub fn elem_tag(&self) -> String {
        match self {
            SeedCollTy::I32 => "i32".into(),
            SeedCollTy::I64 => "i64".into(),
            SeedCollTy::U64 => "u64".into(),
            SeedCollTy::Bool => "bool".into(),
            SeedCollTy::F32 => "f32".into(),
            SeedCollTy::Str | SeedCollTy::StrOwned => "str".into(),
            SeedCollTy::Nil => "nil".into(),
            SeedCollTy::Dyn => "dyn".into(),
            SeedCollTy::Object(index) => format!("obj_{index}"),
            SeedCollTy::Record(n) => format!("rec_{}", sanitize_ident(n)),
            SeedCollTy::Variant(n) => format!("var_{}", sanitize_ident(n)),
            SeedCollTy::List(index) => format!("list_{index}"),
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
        return Err(seed_reject(format!("invalid {what} `{s}` in seed")));
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
            "the a2ts record-text seed does not admit field type `{other:?}`"
        ))),
    }
}

fn seed_coll_ty(view: &SemanticView<'_>, ty: Ty) -> Result<SeedCollTy, A2xReject> {
    match ty {
        Ty::I32 => Ok(SeedCollTy::I32),
        Ty::I64 => Ok(SeedCollTy::I64),
        Ty::U64 => Ok(SeedCollTy::U64),
        Ty::Bool => Ok(SeedCollTy::Bool),
        Ty::F32 => Ok(SeedCollTy::F32),
        Ty::Str if view.is_age_package() => Ok(SeedCollTy::Str),
        Ty::StrOwned => Ok(SeedCollTy::StrOwned),
        Ty::Nil if view.is_age_package() => Ok(SeedCollTy::Nil),
        Ty::Dyn if view.is_age_package() => Ok(SeedCollTy::Dyn),
        Ty::Object(i) if view.is_age_package() => {
            view.object_elem(i)?;
            Ok(SeedCollTy::Object(i))
        }
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
        Ty::List(i) => {
            view.list_elem(i)?;
            Ok(SeedCollTy::List(i))
        }
        other => Err(A2xReject::unsupported_type(format!(
            "the a2ts collections seed does not admit type `{other:?}`"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Wire format: canonical serialisation of the decoded tables
// ---------------------------------------------------------------------------

/// Canonical serialisation of a RecordText record table (see [`encode_rt_seed`]).
/// Identifiers are re-validated here so no caller can emit a non-decodable seed.
fn encode_rt_records(recs: &[SeedRecord]) -> Result<String, A2xReject> {
    let mut out = String::new();
    out.push_str(TS_RT_SEED_SCHEMA);
    out.push('\n');
    for rec in recs {
        validate_ident_token(&rec.name, "record name")?;
        out.push_str(&format!(
            "R {} {} {}\n",
            rec.name,
            if rec.resource { 1 } else { 0 },
            rec.fields.len()
        ));
        for (fname, fty) in &rec.fields {
            validate_ident_token(fname, "field name")?;
            out.push_str(&format!("F {} {}\n", fname, fty.token()));
        }
    }
    Ok(out)
}

/// Canonical serialisation of a Collections seed (see [`encode_coll_seed`]).
fn encode_coll_seed_struct(coll: &SeedColl) -> Result<String, A2xReject> {
    let mut out = String::new();
    out.push_str(TS_COLL_SEED_SCHEMA);
    out.push('\n');
    for rec in &coll.records {
        validate_ident_token(&rec.name, "record name")?;
        out.push_str(&format!(
            "R {} {} {}\n",
            rec.name,
            if rec.resource { 1 } else { 0 },
            rec.fields.len()
        ));
        for (fname, fty) in &rec.fields {
            validate_ident_token(fname, "field name")?;
            out.push_str(&format!("F {} {}\n", fname, fty.token()));
        }
    }
    for var in &coll.variants {
        validate_ident_token(&var.name, "variant name")?;
        out.push_str(&format!("V {} {}\n", var.name, var.arms.len()));
        for (aname, payload) in &var.arms {
            validate_ident_token(aname, "arm name")?;
            let tok = match payload {
                Some(ty) => ty.token(),
                None => "-".to_string(),
            };
            out.push_str(&format!("A {} {}\n", aname, tok));
        }
    }
    for elem in &coll.lists {
        out.push_str(&format!("L {}\n", elem.token()));
    }
    Ok(out)
}

/// Encode the RecordText nominal record table into canonical seed text.
pub fn encode_rt_seed(view: &SemanticView<'_>) -> Result<String, A2xReject> {
    let mut recs = Vec::with_capacity(view.records().len());
    for rec in view.records() {
        let mut fields = Vec::with_capacity(rec.fields.len());
        for f in &rec.fields {
            fields.push((f.name.clone(), seed_field_ty(view, f.ty)?));
        }
        recs.push(SeedRecord {
            name: rec.name.clone(),
            resource: rec.resource,
            fields,
        });
    }
    encode_rt_records(&recs)
}

/// Strictly decode a RecordText seed produced by [`encode_rt_seed`].
pub fn decode_rt_seed(text: &str) -> Result<Vec<SeedRecord>, A2xReject> {
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or_else(|| seed_reject("empty RecordText seed"))?;
    if header != TS_RT_SEED_SCHEMA {
        return Err(seed_reject(format!(
            "unexpected RecordText seed schema `{header}`"
        )));
    }
    let mut recs = Vec::new();
    while let Some(line) = lines.next() {
        let parts: Vec<&str> = line.split(' ').collect();
        if parts.len() != 4 || parts[0] != "R" {
            return Err(seed_reject(format!("malformed record header `{line}`")));
        }
        validate_ident_token(parts[1], "record name")?;
        let resource = match parts[2] {
            "0" => false,
            "1" => true,
            other => {
                return Err(seed_reject(format!("invalid resource flag `{other}`")));
            }
        };
        let nfields: usize = parts[3]
            .parse()
            .map_err(|_| seed_reject(format!("invalid field count `{}`", parts[3])))?;
        let mut fields = Vec::with_capacity(nfields);
        for _ in 0..nfields {
            let fl = lines
                .next()
                .ok_or_else(|| seed_reject("truncated field entry"))?;
            let fp: Vec<&str> = fl.split(' ').collect();
            if fp.len() != 3 || fp[0] != "F" {
                return Err(seed_reject(format!("malformed field entry `{fl}`")));
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
    let mut coll = SeedColl::default();
    for rec in view.records() {
        let mut fields = Vec::with_capacity(rec.fields.len());
        for f in &rec.fields {
            fields.push((f.name.clone(), seed_coll_ty(view, f.ty)?));
        }
        coll.records.push(SeedCollRecord {
            name: rec.name.clone(),
            resource: rec.resource,
            fields,
        });
    }
    for var in view.variants() {
        let mut arms = Vec::with_capacity(var.arms.len());
        for arm in &var.arms {
            let payload = match arm.payload {
                Some(ty) => Some(seed_coll_ty(view, ty)?),
                None => None,
            };
            arms.push((arm.name.clone(), payload));
        }
        coll.variants.push(SeedCollVariant {
            name: var.name.clone(),
            arms,
        });
    }
    // List element types are deduplicated in first-seen view order (the Python
    // leg's rule); the seed carries them for identity/diagnostics only.
    let mut lists: Vec<SeedCollTy> = Vec::new();
    for lt in view.list_types() {
        let st = seed_coll_ty(view, view.list_elem(lt)?)?;
        if !lists.contains(&st) {
            lists.push(st);
        }
    }
    coll.lists = lists;
    encode_coll_seed_struct(&coll)
}

/// Strictly decode a Collections seed produced by [`encode_coll_seed`].
pub fn decode_coll_seed(text: &str) -> Result<SeedColl, A2xReject> {
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or_else(|| seed_reject("empty Collections seed"))?;
    if header != TS_COLL_SEED_SCHEMA {
        return Err(seed_reject(format!(
            "unexpected Collections seed schema `{header}`"
        )));
    }
    let mut seed = SeedColl::default();
    while let Some(line) = lines.next() {
        let parts: Vec<&str> = line.split(' ').collect();
        match parts.first().copied() {
            Some("R") => {
                if parts.len() != 4 {
                    return Err(seed_reject(format!("malformed R line `{line}`")));
                }
                validate_ident_token(parts[1], "record name")?;
                let resource = match parts[2] {
                    "0" => false,
                    "1" => true,
                    other => {
                        return Err(seed_reject(format!("invalid resource flag `{other}`")));
                    }
                };
                let nfields: usize = parts[3]
                    .parse()
                    .map_err(|_| seed_reject("invalid field count"))?;
                let mut fields = Vec::with_capacity(nfields);
                for _ in 0..nfields {
                    let fl = lines
                        .next()
                        .ok_or_else(|| seed_reject("truncated F line"))?;
                    let fp: Vec<&str> = fl.split(' ').collect();
                    if fp.len() != 3 || fp[0] != "F" {
                        return Err(seed_reject(format!("malformed F line `{fl}`")));
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
                    return Err(seed_reject(format!("malformed V line `{line}`")));
                }
                validate_ident_token(parts[1], "variant name")?;
                let narms: usize = parts[2]
                    .parse()
                    .map_err(|_| seed_reject("invalid arm count"))?;
                let mut arms = Vec::with_capacity(narms);
                for _ in 0..narms {
                    let al = lines
                        .next()
                        .ok_or_else(|| seed_reject("truncated A line"))?;
                    let ap: Vec<&str> = al.split(' ').collect();
                    if ap.len() != 3 || ap[0] != "A" {
                        return Err(seed_reject(format!("malformed A line `{al}`")));
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
                    return Err(seed_reject(format!("malformed L line `{line}`")));
                }
                seed.lists.push(SeedCollTy::parse_token(parts[1])?);
            }
            _ => {
                return Err(seed_reject(format!("unknown seed line `{line}`")));
            }
        }
    }
    Ok(seed)
}

// ---------------------------------------------------------------------------
// Renderers: deterministic ES module helper functions (JavaScript, not Python)
// ---------------------------------------------------------------------------

const RENDER_HEADER_RT: &str = "\
// AUTO-GENERATED by ace-a2x-ts (PLAN-976 a2ts record-text seed) -- DO NOT EDIT.
// Deterministic, declaration-order clone/free helpers for the generated module.
// Composite values are GC-managed in this target: only owned `Str` buffers are
// ledger-tracked and released through the runtime's `a2x_str_free`; borrowed
// `str` views are copied by reference and are never freed. Plain ES module code
// only (no CommonJS `require`); the printer imports `a2x_str_clone` and
// `a2x_str_free` next to these helpers.
";

const RENDER_HEADER_COLL: &str = "\
// AUTO-GENERATED by ace-a2x-ts (PLAN-976 a2ts collections seed) -- DO NOT EDIT.
// The collections seed renders RECORD clone/free helpers only: variant and list
// tables are carried by the seed for identity/diagnostics, and composite values
// (records, variants, lists) are managed by the JavaScript garbage collector.
// Only owned `Str` buffers are explicitly freed (through `a2x_str_free`); plain
// ES module code only (no CommonJS `require`).
";

/// Renderer-facing classification of one record field.
#[derive(Clone, Debug, PartialEq, Eq)]
enum FieldKind {
    /// `i32` / `bool` / `char`: copied by value, never freed.
    Scalar,
    /// borrowed `str`: a view into a caller-owned buffer, copied by reference
    /// and never freed.
    StrBorrowed,
    /// owned `Str`: cloned with `a2x_str_clone`, freed with `a2x_str_free`.
    StrOwned,
    /// nested nominal record: cloned/freed through its own generated helper.
    Record(String),
    /// GC-managed collection value: copied by reference and never freed here.
    SharedCollection,
}

/// Renderer input for one record (name + fields in declaration order).
struct RecordPlan {
    name: String,
    fields: Vec<(String, FieldKind)>,
}

impl RecordPlan {
    fn from_rt(rec: &SeedRecord) -> Self {
        RecordPlan {
            name: rec.name.clone(),
            fields: rec
                .fields
                .iter()
                .map(|(name, ty)| {
                    let kind = match ty {
                        SeedFieldTy::I32 | SeedFieldTy::Bool | SeedFieldTy::Char => {
                            FieldKind::Scalar
                        }
                        SeedFieldTy::Str => FieldKind::StrBorrowed,
                        SeedFieldTy::StrOwned => FieldKind::StrOwned,
                        SeedFieldTy::Record(inner) => FieldKind::Record(inner.clone()),
                    };
                    (name.clone(), kind)
                })
                .collect(),
        }
    }

    fn from_coll(rec: &SeedCollRecord) -> Self {
        RecordPlan {
            name: rec.name.clone(),
            fields: rec
                .fields
                .iter()
                .map(|(name, ty)| {
                    let kind = match ty {
                        SeedCollTy::I32 | SeedCollTy::I64 | SeedCollTy::U64 | SeedCollTy::Bool
                        | SeedCollTy::F32 | SeedCollTy::Nil => FieldKind::Scalar,
                        SeedCollTy::Str => FieldKind::StrBorrowed,
                        SeedCollTy::StrOwned => FieldKind::StrOwned,
                        SeedCollTy::Record(inner) => FieldKind::Record(inner.clone()),
                        SeedCollTy::Variant(_) | SeedCollTy::List(_) | SeedCollTy::Dyn
                        | SeedCollTy::Object(_) => FieldKind::SharedCollection,
                    };
                    (name.clone(), kind)
                })
                .collect(),
        }
    }
}

/// Render one ES module block of record clone/free helpers.
///
/// Determinism: no maps, no timestamps, no environment reads — records are
/// emitted in declaration order and fields in declaration order, so the same
/// plan always yields byte-identical text.
fn render_record_helpers(header: &str, plans: &[RecordPlan], typed: bool) -> String {
    if plans.is_empty() {
        return String::new();
    }
    let mut out = String::with_capacity(header.len() + plans.len() * 320);
    out.push_str(header);
    for plan in plans {
        let rname = sanitize_ident(&plan.name);

        out.push('\n');
        if typed {
            out.push_str(&format!(
                "export function a2x_rec_clone_{rname}(x: any): any {{\n"
            ));
            out.push_str("  const out: any = {};\n");
        } else {
            out.push_str(&format!("export function a2x_rec_clone_{rname}(x) {{\n"));
            out.push_str("  const out = {};\n");
        }
        for (fname, kind) in &plan.fields {
            let sfn = sanitize_ident(fname);
            match kind {
                FieldKind::Scalar | FieldKind::StrBorrowed | FieldKind::SharedCollection => {
                    out.push_str(&format!("  out[\"{sfn}\"] = x[\"{sfn}\"];\n"));
                }
                FieldKind::StrOwned => {
                    out.push_str(&format!(
                        "  out[\"{sfn}\"] = a2x_str_clone(x[\"{sfn}\"]);\n"
                    ));
                }
                FieldKind::Record(inner) => {
                    let ir = sanitize_ident(inner);
                    out.push_str(&format!(
                        "  out[\"{sfn}\"] = a2x_rec_clone_{ir}(x[\"{sfn}\"]);\n"
                    ));
                }
            }
        }
        out.push_str("  return out;\n}\n");

        out.push('\n');
        if typed {
            out.push_str(&format!(
                "export function a2x_rec_free_{rname}(x: any | null | undefined): void {{\n"
            ));
        } else {
            out.push_str(&format!("export function a2x_rec_free_{rname}(x) {{\n"));
        }
        out.push_str("  if (x === null || x === undefined) {\n    return;\n  }\n");
        for (fname, kind) in &plan.fields {
            let sfn = sanitize_ident(fname);
            match kind {
                FieldKind::StrOwned => {
                    out.push_str(&format!("  a2x_str_free(x[\"{sfn}\"]);\n"));
                }
                FieldKind::Record(inner) => {
                    let ir = sanitize_ident(inner);
                    out.push_str(&format!("  a2x_rec_free_{ir}(x[\"{sfn}\"]);\n"));
                }
                FieldKind::Scalar | FieldKind::StrBorrowed | FieldKind::SharedCollection => {}
            }
        }
        out.push_str("}\n");
    }
    let kept = out.trim_end().len();
    out.truncate(kept);
    out.push('\n');
    out
}

/// Generate JavaScript helper functions for deterministic RecordText
/// clone/free semantics.
///
/// For every record in declaration order this emits
/// `export function a2x_rec_clone_<name>(x)` (deep clone: scalars by value,
/// owned `Str` via `a2x_str_clone`, nested records through their own clone
/// helper, borrowed `str` views by reference) and
/// `export function a2x_rec_free_<name>(x)` (null-tolerant; releases every
/// owned `Str` field and recurses into nested record fields). Clone
/// correctness does not depend on the `resource` flag.
pub fn render_rt_seed_helpers(recs: &[SeedRecord], typed: bool) -> String {
    let plans: Vec<RecordPlan> = recs.iter().map(RecordPlan::from_rt).collect();
    render_record_helpers(RENDER_HEADER_RT, &plans, typed)
}

/// Generate JavaScript helper functions for deterministic Collections record
/// clone/free semantics.
///
/// The collections seed renders RECORD clone/free helpers only (same shape as
/// [`render_rt_seed_helpers`]): variant and list tables are carried by the seed
/// for identity/diagnostics, and variant/list composites are managed by the
/// JavaScript garbage collector, so no `a2x_var_*` / `a2x_list_*` helper is
/// emitted. The signature is kept identical to the Python leg so callers stay
/// uniform across targets, and the output is byte-deterministic.
pub fn render_coll_seed_helpers(coll: &SeedColl, typed: bool) -> String {
    let plans: Vec<RecordPlan> = coll.records.iter().map(RecordPlan::from_coll).collect();
    render_record_helpers(RENDER_HEADER_COLL, &plans, typed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(name: &str, resource: bool, fields: &[(&str, SeedFieldTy)]) -> SeedRecord {
        SeedRecord {
            name: name.to_string(),
            resource,
            fields: fields
                .iter()
                .map(|(n, t)| ((*n).to_string(), t.clone()))
                .collect(),
        }
    }

    fn coll_rec(name: &str, resource: bool, fields: &[(&str, SeedCollTy)]) -> SeedCollRecord {
        SeedCollRecord {
            name: name.to_string(),
            resource,
            fields: fields
                .iter()
                .map(|(n, t)| ((*n).to_string(), t.clone()))
                .collect(),
        }
    }

    #[test]
    fn rt_seed_round_trip() {
        let recs = vec![
            rec(
                "Point",
                true,
                &[
                    ("x", SeedFieldTy::I32),
                    ("label", SeedFieldTy::StrOwned),
                    ("view", SeedFieldTy::Str),
                ],
            ),
            rec(
                "Box",
                false,
                &[
                    ("inner", SeedFieldTy::Record("Point".to_string())),
                    ("ok", SeedFieldTy::Bool),
                    ("c", SeedFieldTy::Char),
                ],
            ),
        ];
        let text = encode_rt_records(&recs).expect("encode record seed");
        assert_eq!(text.lines().next(), Some(TS_RT_SEED_SCHEMA));
        assert_eq!(decode_rt_seed(&text).expect("decode record seed"), recs);
    }

    #[test]
    fn coll_seed_round_trip() {
        let coll = SeedColl {
            records: vec![
                coll_rec(
                    "Point",
                    true,
                    &[("x", SeedCollTy::F32), ("label", SeedCollTy::StrOwned)],
                ),
                coll_rec(
                    "Entry",
                    false,
                    &[
                        ("pt", SeedCollTy::Record("Point".to_string())),
                        ("maybe", SeedCollTy::Variant("Opt".to_string())),
                    ],
                ),
            ],
            variants: vec![SeedCollVariant {
                name: "Opt".to_string(),
                arms: vec![
                    ("None".to_string(), None),
                    ("Some".to_string(), Some(SeedCollTy::I32)),
                ],
            }],
            lists: vec![
                SeedCollTy::I32,
                SeedCollTy::F32,
                SeedCollTy::Record("Point".to_string()),
                SeedCollTy::Variant("Opt".to_string()),
            ],
        };
        let text = encode_coll_seed_struct(&coll).expect("encode collections seed");
        assert_eq!(text.lines().next(), Some(TS_COLL_SEED_SCHEMA));
        assert_eq!(
            decode_coll_seed(&text).expect("decode collections seed"),
            coll
        );
    }

    #[test]
    fn decode_rejects_bad_header() {
        for text in ["a2ts-rt-seed/2\n", "a2py-rt-seed/1\n", ""] {
            let err = decode_rt_seed(text).expect_err("bad RT header must be rejected");
            assert_eq!(err.code, "a2ts.seed", "detail: {}", err.detail);
        }
        for text in ["a2ts-coll-seed/2\n", "a2ts-rt-seed/1\n", ""] {
            let err = decode_coll_seed(text).expect_err("bad coll header must be rejected");
            assert_eq!(err.code, "a2ts.seed", "detail: {}", err.detail);
        }
    }

    #[test]
    fn decode_rejects_unknown_line_token() {
        let err = decode_rt_seed("a2ts-rt-seed/1\nX 1 0 0\n")
            .expect_err("unknown RT line token must be rejected");
        assert_eq!(err.code, "a2ts.seed");
        let err = decode_coll_seed("a2ts-coll-seed/1\nQ 1\n")
            .expect_err("unknown coll line token must be rejected");
        assert_eq!(err.code, "a2ts.seed");
        assert!(err.detail.contains("unknown seed line"), "{}", err.detail);
    }

    #[test]
    fn decode_rejects_malformed_field_type() {
        let err = decode_rt_seed("a2ts-rt-seed/1\nR P 0 1\nF s Float\n")
            .expect_err("unknown RT field type must be rejected");
        assert_eq!(err.code, "a2ts.seed");
        assert!(
            err.detail.contains("unknown record field token"),
            "{}",
            err.detail
        );
        let err = decode_coll_seed("a2ts-coll-seed/1\nR P 0 1\nF s f64\n")
            .expect_err("unknown coll field type must be rejected");
        assert_eq!(err.code, "a2ts.seed");
        assert!(
            err.detail.contains("unknown collections seed type token"),
            "{}",
            err.detail
        );
    }

    #[test]
    fn decode_rejects_truncated_and_bad_flags() {
        let err = decode_rt_seed("a2ts-rt-seed/1\nR P 0 2\nF s i32\n")
            .expect_err("truncated field list must be rejected");
        assert_eq!(err.code, "a2ts.seed");
        let err = decode_rt_seed("a2ts-rt-seed/1\nR P 2 0\n")
            .expect_err("invalid resource flag must be rejected");
        assert_eq!(err.code, "a2ts.seed");
        let err = decode_coll_seed("a2ts-coll-seed/1\nL\n")
            .expect_err("malformed L line must be rejected");
        assert_eq!(err.code, "a2ts.seed");
    }

    #[test]
    fn rt_renderer_is_deterministic() {
        let recs = vec![
            rec(
                "Point",
                true,
                &[
                    ("s", SeedFieldTy::StrOwned),
                    ("inner", SeedFieldTy::Record("Inner".to_string())),
                ],
            ),
            rec("Empty", false, &[]),
        ];
        let first = render_rt_seed_helpers(&recs, false);
        let second = render_rt_seed_helpers(&recs, false);
        assert_eq!(
            first, second,
            "renderer must be byte-identical across calls"
        );
        assert!(!first.is_empty());
        assert!(
            !first.contains("require("),
            "ES module output must not use require"
        );
        assert!(!first.contains("import "), "imports belong to the printer");
        assert!(first.ends_with("}\n"));
    }

    #[test]
    fn coll_renderer_is_deterministic_and_record_only() {
        let coll = SeedColl {
            records: vec![
                coll_rec(
                    "Point",
                    true,
                    &[("s", SeedCollTy::StrOwned), ("x", SeedCollTy::I32)],
                ),
                coll_rec(
                    "Entry",
                    false,
                    &[
                        ("pt", SeedCollTy::Record("Point".to_string())),
                        ("maybe", SeedCollTy::Variant("Opt".to_string())),
                        ("children", SeedCollTy::List(7)),
                    ],
                ),
            ],
            variants: vec![SeedCollVariant {
                name: "Opt".to_string(),
                arms: vec![("Some".to_string(), Some(SeedCollTy::I32))],
            }],
            lists: vec![SeedCollTy::I32],
        };
        let first = render_coll_seed_helpers(&coll, false);
        assert_eq!(first, render_coll_seed_helpers(&coll, false));
        let encoded = encode_coll_seed_struct(&coll).unwrap();
        assert_eq!(decode_coll_seed(&encoded).unwrap(), coll);
        assert!(first.contains("out[\"children\"] = x[\"children\"];"));
        assert!(!first.contains("a2x_str_free(x[\"children\"])"));
        assert!(first.contains("export function a2x_rec_clone_Point(x)"));
        assert!(first.contains("export function a2x_rec_clone_Entry(x)"));
        assert!(
            !first.contains("a2x_var_"),
            "no variant helpers in this target"
        );
        assert!(
            !first.contains("a2x_list_"),
            "no list helpers in this target"
        );
        assert!(!first.contains("require("));
        assert!(first.contains("RECORD clone/free helpers only"));
        // Variant fields are GC-managed: aliased, never freed.
        assert!(first.contains("out[\"maybe\"] = x[\"maybe\"];"));
        assert!(!first.contains("a2x_str_free(x[\"maybe\"])"));
    }

    #[test]
    fn render_rt_clones_owned_and_delegates_nested() {
        let recs = vec![
            rec(
                "Inner",
                false,
                &[("s", SeedFieldTy::StrOwned), ("x", SeedFieldTy::I32)],
            ),
            rec(
                "Point",
                true,
                &[
                    ("s", SeedFieldTy::StrOwned),
                    ("inner", SeedFieldTy::Record("Inner".to_string())),
                    ("view", SeedFieldTy::Str),
                    ("c", SeedFieldTy::Char),
                ],
            ),
        ];
        let js = render_rt_seed_helpers(&recs, false);
        // Clone: owned Str cloned, nested record delegated, borrowed str+scalar copy.
        assert!(js.contains("export function a2x_rec_clone_Point(x) {"));
        assert!(js.contains("out[\"s\"] = a2x_str_clone(x[\"s\"]);"));
        assert!(js.contains("out[\"inner\"] = a2x_rec_clone_Inner(x[\"inner\"]);"));
        assert!(js.contains("out[\"view\"] = x[\"view\"];"));
        assert!(js.contains("out[\"c\"] = x[\"c\"];"));
        // Free: null-tolerant, frees owned Str only, recurses into nested records.
        assert!(js.contains("export function a2x_rec_free_Point(x) {"));
        assert!(js.contains("if (x === null || x === undefined) {"));
        assert!(js.contains("a2x_str_free(x[\"s\"]);"));
        assert!(js.contains("a2x_rec_free_Inner(x[\"inner\"]);"));
        assert!(!js.contains("a2x_str_free(x[\"view\"])"));
        assert!(!js.contains("a2x_str_free(x[\"c\"])"));
        // A non-resource record still gets a correct clone/free pair.
        assert!(js.contains("export function a2x_rec_clone_Inner(x) {"));
        assert!(js.contains("export function a2x_rec_free_Inner(x) {"));
        // Declaration order is stable: Inner is emitted before Point.
        let inner_at = js.find("a2x_rec_clone_Inner").expect("inner helper");
        let point_at = js.find("a2x_rec_clone_Point").expect("point helper");
        assert!(inner_at < point_at, "helpers must follow declaration order");
    }

    #[test]
    fn renderers_emit_nothing_for_empty_input() {
        assert_eq!(render_rt_seed_helpers(&[], false), "");
        assert_eq!(render_coll_seed_helpers(&SeedColl::default(), false), "");
    }
}
