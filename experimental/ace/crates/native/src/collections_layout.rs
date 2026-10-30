//! Canonical native collection descriptors. Table indices and source positions
//! never enter identity; an iterative dependency walk handles deep acyclic types.
use crate::MnError;
use ace_av_core::mir::*;
use std::collections::VecDeque;

pub const LAYOUT: &str = "mn.collections.layout/1";
pub const CODEGEN: &str = "mn.collections.codegen/1";
pub const PROFILE: &str = "mn.collections.source/1";
pub const RECIPE: &str = "ac/mir-native-collections-v1";
pub fn codegen_build() -> &'static str {
    static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    HASH.get_or_init(|| {
        let mut h = blake3::Hasher::new();
        h.update(CODEGEN.as_bytes());
        for bytes in [
            include_bytes!("collections_lower.rs").as_slice(),
            include_bytes!("collections_layout.rs").as_slice(),
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Descriptor {
    pub fingerprint: [u8; 32],
    /// 0 scalar/signature, 1 UTF-8 owner, 2 record, 3 list, 4 variant.
    pub kind: u32,
    /// Record fields, list element-type count (1), or variant arm count.
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
fn children(ty: &MirType) -> Vec<TypeIdx> {
    match ty {
        MirType::Record(r) | MirType::RecordView(r) => r.fields.iter().map(|f| f.ty).collect(),
        MirType::List(l) => vec![l.elem],
        MirType::Variant(v) => v.arms.iter().filter_map(|a| a.payload).collect(),
        MirType::Fn(s) => s.params.iter().copied().chain([s.result]).collect(),
        _ => vec![],
    }
}
fn text(h: &mut blake3::Hasher, value: &str) {
    h.update(&(value.len() as u64).to_le_bytes());
    h.update(value.as_bytes());
}
/// Caller first runs shared verify_mir and the native type whitelist. This also
/// bounds references/cycles defensively for direct descriptor consumers.
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
                    h.update(&[matches!(
                        m.types[*p as usize],
                        MirType::StrView | MirType::RecordView(_)
                    ) as u8]);
                    h.update(&result[*p as usize].fingerprint);
                }
                h.update(&[matches!(
                    m.types[s.result as usize],
                    MirType::StrView | MirType::RecordView(_)
                ) as u8]);
                h.update(&result[s.result as usize].fingerprint);
                (0, s.params.len() as u32)
            }
            _ => {
                return Err(MnError::Unsupported(
                    "type",
                    "type outside collection layout contract".into(),
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
