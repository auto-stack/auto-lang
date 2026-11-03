//! PLAN-924 native closure descriptors. A SEPARATE identity family from the
//! collections recipes (§5.1): the closures profile consumes the shared
//! `av.closures/1` MIR semantics, but native admission freezes a limited
//! source/value domain — captures are explicit `move` of `Str`/owned
//! records or by-value scalars, plus typed-only deep-clone captures; the
//! remaining AV capture modes (Borrow/MoveFromEnv/Shared) are NAMED
//! unsupported here, never silently widened, and the shared frontend/AV
//! contracts keep their own wider support.

use crate::MnError;
use ace_av_core::mir::*;
use std::collections::VecDeque;

pub const LAYOUT: &str = "mn.closures.layout/1";
pub const CODEGEN: &str = "mn.closures.codegen/1";
pub const PROFILE: &str = "mn.closures.source/1";
pub const RECIPE: &str = "ac/mir-native-closures-resource-v1";
/// The closure-family cell ABI: `(env: i64, params: *Cell.., out: *Cell)`
/// bodies, staged CreateClosure transactions, ENV_GET handle shares and
/// pre-dispatch target admission. Not a public/FFI ABI (797 stays separate).
pub const ABI: &str = "mn.closures.cell/1";
pub fn codegen_build() -> &'static str {
    static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    HASH.get_or_init(|| {
        let mut h = blake3::Hasher::new();
        h.update(CODEGEN.as_bytes());
        for bytes in [
            include_bytes!("closures_lower.rs").as_slice(),
            include_bytes!("closures_layout.rs").as_slice(),
            include_bytes!("fingerprint.rs").as_slice(),
            include_bytes!("emit.rs").as_slice(),
            include_bytes!("symbols.rs").as_slice(),
            include_bytes!("../../../Cargo.lock").as_slice(),
        ] {
            h.update(&(bytes.len() as u64).to_le_bytes());
            h.update(bytes);
        }
        h.finalize().to_hex().to_string()
    })
}

/// Reuses the collections descriptor SHAPE (kind codes 0..4 identical; see
/// `collections_layout`) plus kind 5 = a Closure value (env handle) carrying
/// its signature. Kept as a local walk so the collections family's codegen
/// identity never depends on closure admissions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Descriptor {
    pub fingerprint: [u8; 32],
    pub kind: u32,
    pub arity: u32,
    pub borrowed: bool,
}
impl Descriptor {
    pub fn words(&self) -> [u64; 4] {
        std::array::from_fn(|i| {
            u64::from_le_bytes(self.fingerprint[i * 8..i * 8 + 8].try_into().unwrap())
        })
    }
}
fn text(h: &mut blake3::Hasher, value: &str) {
    h.update(&(value.len() as u64).to_le_bytes());
    h.update(value.as_bytes());
}
fn children(ty: &MirType) -> Vec<TypeIdx> {
    match ty {
        MirType::Record(r) | MirType::RecordView(r) => r.fields.iter().map(|f| f.ty).collect(),
        MirType::List(l) => vec![l.elem],
        MirType::Variant(v) => v.arms.iter().filter_map(|a| a.payload).collect(),
        MirType::Fn(s) | MirType::Closure(s) => {
            s.params.iter().copied().chain([s.result]).collect()
        }
        _ => vec![],
    }
}
/// Caller first runs shared verify_mir and the native limited-domain
/// whitelist (`closures_lower::admit_layouts`). This bounds references and
/// cycles defensively for direct descriptor consumers.
pub fn descriptors(m: &MirModule) -> Result<Vec<Descriptor>, MnError> {
    let n = m.types.len();
    let mut remaining = vec![0usize; n];
    let mut parents = vec![Vec::new(); n];
    for (i, ty) in m.types.iter().enumerate() {
        for child in children(ty) {
            let parent = parents
                .get_mut(child as usize)
                .ok_or_else(|| MnError::Admit("layout", "type reference out of range".into()))?;
            parent.push(i);
            remaining[i] += 1;
        }
    }
    let mut ready: VecDeque<_> = remaining
        .iter()
        .enumerate()
        .filter_map(|(i, &n)| (n == 0).then_some(i))
        .collect();
    let empty = Descriptor {
        fingerprint: [0; 32],
        kind: 0,
        arity: 0,
        borrowed: false,
    };
    let mut result = vec![empty; n];
    let mut completed = 0;
    while let Some(i) = ready.pop_front() {
        let mut h = blake3::Hasher::new();
        text(&mut h, LAYOUT);
        let (kind, arity) = match &m.types[i] {
            MirType::I32 => {
                text(&mut h, "i32");
                (0, 0)
            }
            MirType::Bool => {
                text(&mut h, "bool");
                (0, 0)
            }
            MirType::Str | MirType::StrView => {
                text(&mut h, "utf8");
                (1, 0)
            }
            MirType::Record(r) | MirType::RecordView(r) => {
                text(&mut h, "record/cell16");
                text(&mut h, &r.owner_module);
                text(&mut h, &r.name);
                h.update(&(r.fields.len() as u64).to_le_bytes());
                for f in &r.fields {
                    text(&mut h, &f.name);
                    h.update(&result[f.ty as usize].fingerprint);
                }
                (2, r.fields.len() as u32)
            }
            MirType::List(l) => {
                text(&mut h, "list/cell16");
                h.update(&result[l.elem as usize].fingerprint);
                (3, 1)
            }
            MirType::Variant(v) => {
                text(&mut h, "variant/active-cell16");
                text(&mut h, &v.owner_module);
                text(&mut h, &v.name);
                h.update(&(v.arms.len() as u64).to_le_bytes());
                for a in &v.arms {
                    text(&mut h, &a.name);
                    h.update(&[a.payload.is_some() as u8]);
                    if let Some(t) = a.payload {
                        h.update(&result[t as usize].fingerprint);
                    }
                }
                (4, v.arms.len() as u32)
            }
            MirType::Fn(s) => {
                text(&mut h, "fn/out-cell/2");
                h.update(&(s.params.len() as u64).to_le_bytes());
                for p in &s.params {
                    h.update(&result[*p as usize].fingerprint);
                }
                h.update(&result[s.result as usize].fingerprint);
                (0, s.params.len() as u32)
            }
            MirType::Closure(s) => {
                text(&mut h, "closure/env-handle");
                h.update(&(s.params.len() as u64).to_le_bytes());
                for p in &s.params {
                    h.update(&result[*p as usize].fingerprint);
                }
                h.update(&result[s.result as usize].fingerprint);
                (5, s.params.len() as u32)
            }
            _ => {
                return Err(MnError::Unsupported(
                    "type",
                    "type outside closure layout contract".into(),
                ))
            }
        };
        result[i] = Descriptor {
            fingerprint: *h.finalize().as_bytes(),
            kind,
            arity,
            borrowed: matches!(m.types[i], MirType::StrView | MirType::RecordView(_)),
        };
        completed += 1;
        for &parent in &parents[i] {
            remaining[parent] -= 1;
            if remaining[parent] == 0 {
                ready.push_back(parent);
            }
        }
    }
    if completed != n {
        return Err(MnError::Admit("layout", "cyclic type layout".into()));
    }
    Ok(result)
}

/// The signature identity the runtime binds into the env and re-checks
/// before every indirect dispatch (the body and its creators derive this
/// from the same MIR tables + descriptor fingerprints — never from a host
/// address; two structurally different signatures never collide).
pub fn sig_id(sig: &FnSig, ds: &[Descriptor]) -> u64 {
    let mut h = blake3::Hasher::new();
    text(&mut h, "mn.closures.sig/1");
    h.update(&(sig.params.len() as u64).to_le_bytes());
    for p in &sig.params {
        h.update(&ds[*p as usize].fingerprint);
    }
    h.update(&ds[sig.result as usize].fingerprint);
    let fp: [u8; 32] = h.finalize().into();
    u64::from_le_bytes(fp[..8].try_into().unwrap())
}
/// Nominal owner identity of one callee (module owner scope + unit name) —
/// the runtime binds it for pre-call owner admission on the probe surface.
pub fn owner_id(m: &MirModule, unit_name: &str) -> u64 {
    let mut h = blake3::Hasher::new();
    text(&mut h, "mn.closures.owner/1");
    text(&mut h, &m.owner.schema);
    text(&mut h, &m.owner.module);
    text(&mut h, unit_name);
    let fp: [u8; 32] = h.finalize().into();
    u64::from_le_bytes(fp[..8].try_into().unwrap())
}
/// The closures family's unit identity: the spans-cleared ABC wire bytes —
/// the SAME identity the AV closures sessions compare (PLAN-865/912 wire
/// discipline). The scalar/ collections fingerprint renders never widen to
/// closure instructions; two byte-identical wires are one identity.
pub fn unit_fp(m: &MirModule) -> Result<String, MnError> {
    Ok(blake3::hash(&unit_wire_bytes(m)?).to_hex().to_string())
}
/// The spans-cleared ABC wire bytes (the manifest MIR blob + restore input).
pub fn unit_wire_bytes(m: &MirModule) -> Result<Vec<u8>, MnError> {
    let mut identity = m.clone();
    for function in &mut identity.funcs {
        function.spans.clear();
    }
    Ok(ace_av_core::abc::encode_abc(&identity)
        .map_err(|e| MnError::Admit("mir-wire", format!("encode closures unit: {e:?}")))?
        .to_flat())
}

/// Stable type closure identity for persisted native objects/manifests.
pub fn closure_fingerprint(descriptors: &[Descriptor]) -> String {
    let mut shapes: Vec<_> = descriptors
        .iter()
        .map(|d| (d.fingerprint, d.borrowed))
        .collect();
    shapes.sort();
    shapes.dedup();
    let mut h = blake3::Hasher::new();
    text(&mut h, LAYOUT);
    for (shape, borrowed) in shapes {
        h.update(&shape);
        h.update(&[borrowed as u8]);
    }
    h.finalize().to_hex().to_string()
}
