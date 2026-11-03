//! PLAN-925 CA-B: the C record-text PRELUDE SEED codec.
//!
//! The C record-text file set is not re-assemblable from fragments alone:
//! `generated.c` needs the module's record type table (struct definitions
//! + the per-record deep-clone helpers reference field types). On the cold
//! path the provider captures the seed from the live view
//! ([`TargetProvider::capture_prelude_seed`]); the session stores it
//! provider-opaque and feeds it back on the pure-restore re-assembly
//! (`view: None`). The seed is a strict, line-oriented, deterministic
//! projection of the record table — nothing else (no fingerprints, no
//! bodies; the anchor/fragment layers carry identities).
//!
//! Grammar (one record per block, fields in declaration order):
//!
//! ```text
//!   a2xc-rt-seed/1
//!   R <name> <resource:0|1> <nfields>
//!   F <name> <ty>
//!   ...
//! ```
//!
//! `ty` tokens: `i32` `bool` `char` `str` `Str` `R:<name>`. Encode and
//! decode are inverse; decode is STRICT (schema line, counts, no trailing
//! bytes, no whitespace inside names).

use ace_a2x_core::{A2xReject, SemanticView, Ty};

/// The seed schema line (the first line of every seed).
pub const C_RT_SEED_SCHEMA: &str = "a2xc-rt-seed/1";

/// One seed record type: name, resource flag, fields in declaration order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeedRecord {
    pub name: String,
    pub resource: bool,
    pub fields: Vec<(String, SeedFieldTy)>,
}

/// A seed field type; record references carry the NAME (the C emission
/// resolves struct names directly — no index indirection in the seed).
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
            SeedFieldTy::Record(n) => format!("R:{}", n),
        }
    }
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
            "seed: field type `{:?}` is outside the record-text domain",
            other
        ))),
    }
}

/// Encode the view's record table into the seed text (deterministic:
/// records in view order, fields in declaration order).
pub fn encode_seed(view: &SemanticView<'_>) -> Result<String, A2xReject> {
    let mut out = String::new();
    out.push_str(C_RT_SEED_SCHEMA);
    out.push('\n');
    for rec in view.records() {
        out.push_str(&format!(
            "R {} {} {}\n",
            rec.name,
            if rec.resource { 1 } else { 0 },
            rec.fields.len()
        ));
        for f in &rec.fields {
            out.push_str(&format!(
                "F {} {}\n",
                f.name,
                seed_field_ty(view, f.ty)?.token()
            ));
        }
    }
    Ok(out)
}

fn valid_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(true)
}

/// STRICT decode: schema line, well-formed record/field lines, counts
/// agree, nothing trails. A seed naming an unknown record TYPE inside a
/// field is accepted structurally (name-level reference; the assembler
/// fails loudly on a dangling struct name when it emits the field).
pub fn decode_seed(text: &str) -> Result<Vec<SeedRecord>, A2xReject> {
    let mut lines = text.split('\n');
    let schema = lines.next().unwrap_or("");
    if schema != C_RT_SEED_SCHEMA {
        return Err(A2xReject::new(
            "a2x.seed",
            "seed schema line mismatch (expected a2xc-rt-seed/1)",
        ));
    }
    let mut recs = Vec::new();
    let rest: Vec<&str> = lines.collect();
    let mut i = 0;
    while i < rest.len() {
        let line = rest[i];
        i += 1;
        if line.is_empty() {
            // a single trailing empty line from the final '\n' is fine;
            // anything else trails
            if i == rest.len() && rest.iter().skip(i).all(|l| l.is_empty()) {
                break;
            }
            return Err(A2xReject::new("a2x.seed", "blank line inside the seed"));
        }
        let parts: Vec<&str> = line.split(' ').collect();
        if parts.len() != 4 || parts[0] != "R" || !valid_name(parts[1]) {
            return Err(A2xReject::new(
                "a2x.seed",
                format!("malformed record line: {:?}", line),
            ));
        }
        let nfields: usize = parts[3]
            .parse()
            .map_err(|_| A2xReject::new("a2x.seed", "record field count not a number"))?;
        let resource = match parts[2] {
            "0" => false,
            "1" => true,
            _ => return Err(A2xReject::new("a2x.seed", "resource flag not 0/1")),
        };
        if i + nfields > rest.len() {
            return Err(A2xReject::new(
                "a2x.seed",
                "seed ends inside a record block",
            ));
        }
        let mut fields = Vec::with_capacity(nfields);
        for _ in 0..nfields {
            let fline = rest[i];
            i += 1;
            let fparts: Vec<&str> = fline.split(' ').collect();
            if fparts.len() != 3 || fparts[0] != "F" || !valid_name(fparts[1]) {
                return Err(A2xReject::new(
                    "a2x.seed",
                    format!("malformed field line: {:?}", fline),
                ));
            }
            let ty = match fparts[2] {
                "i32" => SeedFieldTy::I32,
                "bool" => SeedFieldTy::Bool,
                "char" => SeedFieldTy::Char,
                "str" => SeedFieldTy::Str,
                "Str" => SeedFieldTy::StrOwned,
                other => {
                    let name = other.strip_prefix("R:").unwrap_or("");
                    if !valid_name(name) {
                        return Err(A2xReject::new(
                            "a2x.seed",
                            format!("malformed field type: {:?}", other),
                        ));
                    }
                    SeedFieldTy::Record(name.to_string())
                }
            };
            fields.push((fparts[1].to_string(), ty));
        }
        recs.push(SeedRecord {
            name: parts[1].to_string(),
            resource,
            fields,
        });
    }
    Ok(recs)
}
