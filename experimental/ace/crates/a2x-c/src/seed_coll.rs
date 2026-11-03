//! PLAN-926 CA-C: the C collections PRELUDE SEED codec.
//!
//! The C collections file set is not re-assemblable from fragments alone:
//! `generated.c` needs the module's NOMINAL type table — record structs,
//! variant tagged unions, and the typed list instantiations (struct +
//! per-op helpers reference element types). On the cold path the provider
//! captures the seed from the live view
//! ([`TargetProvider::capture_prelude_seed`]); the session stores it
//! provider-opaque and feeds it back on the pure-restore re-assembly
//! (`view: None`). The seed is a strict, line-oriented, deterministic
//! projection of the type table — nothing else (no fingerprints, no
//! bodies; the anchor/fragment layers carry identities).
//!
//! Grammar (records and variants in view order, list instantiations in
//! view list order; fields/arms in declaration order):
//!
//! ```text
//!   a2xc-coll-seed/1
//!   R <name> <resource:0|1> <nfields>
//!   F <name> <ty>
//!   ...
//!   V <name> <narms>
//!   A <arm> <payload-ty|->          (- = unit arm)
//!   ...
//!   L <elem-ty>
//!   ...
//! ```
//!
//! `ty` / `elem-ty` tokens: `i32` `bool` `Str` `R:<name>` `V:<name>`.
//! Encode and decode are inverse; decode is STRICT (schema line, counts,
//! no trailing bytes, no whitespace inside names).

use ace_a2x_core::{A2xReject, SemanticView, Ty};

/// The collections seed schema line (the first line of every seed).
pub const C_COLL_SEED_SCHEMA: &str = "a2xc-coll-seed/1";

/// One seed record type: name, resource flag, fields in declaration order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeedCollRecord {
    pub name: String,
    pub resource: bool,
    pub fields: Vec<(String, SeedCollTy)>,
}

/// One seed variant type: name, arms in declaration order (arm order IS
/// the disc: arm 0 has disc 0).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeedCollVariant {
    pub name: String,
    pub arms: Vec<(String, Option<SeedCollTy>)>,
}

/// The decoded collections seed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SeedColl {
    pub records: Vec<SeedCollRecord>,
    pub variants: Vec<SeedCollVariant>,
    /// List element types to instantiate (deduplicated, view list order).
    pub lists: Vec<SeedCollTy>,
}

/// A seed type reference (field, payload, or list element).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeedCollTy {
    I32,
    Bool,
    StrOwned,
    Record(String),
    Variant(String),
}

impl SeedCollTy {
    /// The token form (seed line spelling).
    fn token(&self) -> String {
        match self {
            SeedCollTy::I32 => "i32".into(),
            SeedCollTy::Bool => "bool".into(),
            SeedCollTy::StrOwned => "Str".into(),
            SeedCollTy::Record(n) => format!("R:{}", n),
            SeedCollTy::Variant(n) => format!("V:{}", n),
        }
    }

    /// The C element tag of a list instantiation (`a2x_list_<etag>`).
    pub fn elem_tag(&self) -> String {
        match self {
            SeedCollTy::I32 => "i32".into(),
            SeedCollTy::Bool => "bool".into(),
            SeedCollTy::StrOwned => "str".into(),
            SeedCollTy::Record(n) => format!("rec_{}", crate::lower::sanitize(n)),
            SeedCollTy::Variant(n) => format!("var_{}", crate::lower::sanitize(n)),
        }
    }

    /// The C value type of this seed type (element / field / payload).
    pub fn c_ty(&self) -> String {
        match self {
            SeedCollTy::I32 => "int32_t".into(),
            SeedCollTy::Bool => "bool".into(),
            SeedCollTy::StrOwned => "a2x_str".into(),
            SeedCollTy::Record(n) => crate::lower_rt::c_record_name(n),
            SeedCollTy::Variant(n) => c_variant_name(n),
        }
    }
}

/// Hygienic C struct name of a variant type.
pub fn c_variant_name(name: &str) -> String {
    format!("a2x_var_{}", crate::lower::sanitize(name))
}

fn seed_coll_ty(view: &SemanticView<'_>, ty: Ty) -> Result<SeedCollTy, A2xReject> {
    match ty {
        Ty::I32 => Ok(SeedCollTy::I32),
        Ty::Bool => Ok(SeedCollTy::Bool),
        Ty::StrOwned => Ok(SeedCollTy::StrOwned),
        Ty::Record(i) => view
            .record(i)
            .map(|r| SeedCollTy::Record(r.name.clone()))
            .ok_or_else(|| A2xReject::new("a2x.internal", "seed: unknown record index")),
        Ty::Variant(i) => view
            .variant(i)
            .map(|v| SeedCollTy::Variant(v.name.clone()))
            .ok_or_else(|| A2xReject::new("a2x.internal", "seed: unknown variant index")),
        other => Err(A2xReject::unsupported_type(format!(
            "seed: type `{:?}` is outside the collections domain",
            other
        ))),
    }
}

/// Encode the view's collections type table into the seed text
/// (deterministic: records/variants in view order, lists deduplicated in
/// view list order, fields/arms in declaration order).
pub fn encode_seed_coll(view: &SemanticView<'_>) -> Result<String, A2xReject> {
    let mut out = String::new();
    out.push_str(C_COLL_SEED_SCHEMA);
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
                seed_coll_ty(view, f.ty)?.token()
            ));
        }
    }
    for v in view.variants() {
        out.push_str(&format!("V {} {}\n", v.name, v.arms.len()));
        for a in &v.arms {
            let payload = match a.payload {
                Some(t) => seed_coll_ty(view, t)?.token(),
                None => "-".to_string(),
            };
            out.push_str(&format!("A {} {}\n", a.name, payload));
        }
    }
    let mut seen: Vec<SeedCollTy> = Vec::new();
    for lt in view.list_types() {
        let elem_ty = view.list_elem(lt)?;
        let elem = seed_coll_ty(view, elem_ty)?;
        if !seen.contains(&elem) {
            seen.push(elem);
        }
    }
    for elem in &seen {
        out.push_str(&format!("L {}\n", elem.token()));
    }
    Ok(out)
}

fn valid_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(true)
}

fn parse_ty(tok: &str) -> Result<SeedCollTy, A2xReject> {
    match tok {
        "i32" => Ok(SeedCollTy::I32),
        "bool" => Ok(SeedCollTy::Bool),
        "Str" => Ok(SeedCollTy::StrOwned),
        other => {
            if let Some(name) = other.strip_prefix("R:") {
                if valid_name(name) {
                    return Ok(SeedCollTy::Record(name.to_string()));
                }
            }
            if let Some(name) = other.strip_prefix("V:") {
                if valid_name(name) {
                    return Ok(SeedCollTy::Variant(name.to_string()));
                }
            }
            Err(A2xReject::new(
                "a2x.seed",
                format!("malformed type token: {:?}", other),
            ))
        }
    }
}

/// STRICT decode: schema line, well-formed blocks, counts agree, nothing
/// trails. A seed naming an unknown record/variant TYPE inside a field,
/// payload, or list is accepted structurally (name-level reference; the
/// type-table emitter fails loudly on a dangling struct name).
pub fn decode_seed_coll(text: &str) -> Result<SeedColl, A2xReject> {
    let mut lines = text.split('\n');
    let schema = lines.next().unwrap_or("");
    if schema != C_COLL_SEED_SCHEMA {
        return Err(A2xReject::new(
            "a2x.seed",
            "seed schema line mismatch (expected a2xc-coll-seed/1)",
        ));
    }
    let rest: Vec<&str> = lines.collect();
    let mut i = 0;
    let total = rest.len();
    // allow exactly one trailing empty line (the final '\n')
    let total = if total > 0 && rest[total - 1].is_empty() {
        total - 1
    } else {
        total
    };
    let mut seed = SeedColl::default();
    while i < total {
        let line = rest[i];
        i += 1;
        if line.starts_with("R ") {
            let parts: Vec<&str> = line.split(' ').collect();
            if parts.len() != 4 || !valid_name(parts[1]) {
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
            if i + nfields > total {
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
                fields.push((fparts[1].to_string(), parse_ty(fparts[2])?));
            }
            seed.records.push(SeedCollRecord {
                name: parts[1].to_string(),
                resource,
                fields,
            });
        } else if line.starts_with("V ") {
            let parts: Vec<&str> = line.split(' ').collect();
            if parts.len() != 3 || !valid_name(parts[1]) {
                return Err(A2xReject::new(
                    "a2x.seed",
                    format!("malformed variant line: {:?}", line),
                ));
            }
            let narms: usize = parts[2]
                .parse()
                .map_err(|_| A2xReject::new("a2x.seed", "variant arm count not a number"))?;
            if i + narms > total {
                return Err(A2xReject::new(
                    "a2x.seed",
                    "seed ends inside a variant block",
                ));
            }
            let mut arms = Vec::with_capacity(narms);
            for _ in 0..narms {
                let aline = rest[i];
                i += 1;
                let aparts: Vec<&str> = aline.split(' ').collect();
                if aparts.len() != 3 || aparts[0] != "A" || !valid_name(aparts[1]) {
                    return Err(A2xReject::new(
                        "a2x.seed",
                        format!("malformed arm line: {:?}", aline),
                    ));
                }
                let payload = if aparts[2] == "-" {
                    None
                } else {
                    Some(parse_ty(aparts[2])?)
                };
                arms.push((aparts[1].to_string(), payload));
            }
            seed.variants.push(SeedCollVariant {
                name: parts[1].to_string(),
                arms,
            });
        } else if line.starts_with("L ") {
            let parts: Vec<&str> = line.split(' ').collect();
            if parts.len() != 2 {
                return Err(A2xReject::new(
                    "a2x.seed",
                    format!("malformed list line: {:?}", line),
                ));
            }
            let elem = parse_ty(parts[1])?;
            if seed.lists.contains(&elem) {
                return Err(A2xReject::new(
                    "a2x.seed",
                    "duplicate list instantiation in the seed",
                ));
            }
            seed.lists.push(elem);
        } else {
            return Err(A2xReject::new(
                "a2x.seed",
                format!("unexpected seed line: {:?}", line),
            ));
        }
    }
    Ok(seed)
}
