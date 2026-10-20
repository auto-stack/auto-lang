//! Finite generation / native-artifact admission model (PLAN-798 §5.3,
//! T-08; F798-R3-09).
//!
//! This is a CONTRACT-LAYER admission over NATIVE ARTIFACT REFERENCES —
//! descriptions of relocatable build products (COFF object bytes +
//! provenance), not AV payloads. What it does:
//! - validates a `NativeArtifactRef`'s generation / execution profile /
//!   capabilities against the CURRENT admission context (unknown values
//!   are rejected, never forward-granted);
//! - validates the referenced object as a RELOCATABLE COFF machine object
//!   (AMD64, has sections and a symbol table, is NOT a linked image) — a
//!   reference claiming process addresses (an image base / non-relocatable
//!   format) is rejected;
//! - checks the content hash binds the EXACT bytes offered.
//!
//! What it deliberately does NOT do (plan non-goals, unchanged): no AJ, no
//! DLL loading, no hot swap, no publisher, no new native emitter. The
//! positive control is a REAL COFF produced by the landed Cranelift
//! lowering (`native::lower_object`) — see `native_admission_` tests.

use auto_ac_prototype::native;
use auto_ac_prototype::verify::CheckedModule;
use std::collections::BTreeSet;

/// Generations this build admits (one per landed ExecutionProfile
/// generation of the probe; bumping a generation does not admit older
/// artifacts silently — old refs re-admit only through an explicit entry).
pub const ADMITTED_GENERATIONS: [&str; 1] = ["plan798-gen-1"];
/// The one native execution profile admitted (reference backend identity).
pub const ADMITTED_NATIVE_PROFILE: &str = "native.msvc-x64/1";
/// Capabilities a native artifact ref may carry (none in v1 — trace rides
/// the AV profile, and no FFI/IO capability exists).
pub const ADMITTED_NATIVE_CAPABILITIES: [&str; 0] = [];
/// The reference backend/target triple pair.
pub const ADMITTED_TARGET: &str = "x86_64-pc-windows-msvc";
pub const ADMITTED_ABI: &str = "msvc-x64";
pub const ADMITTED_BACKEND: &str = "cranelift";
pub const ADMITTED_RECIPE: &str = "plan798/native-reference";

/// Object format of the referenced artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjectFormat {
    /// A relocatable COFF machine object (.obj) with sections + symbols.
    CoffRelocatable,
    /// A linked image carrying absolute/process addresses — REJECTED for
    /// portable artifact references (a process address is not a product).
    LinkedImage,
    /// Any other/unknown format.
    Other(String),
}

/// The artifact reference under admission. Every field is CLAIMED
/// metadata; admission cross-checks the claims against the actual bytes
/// and the current context.
#[derive(Clone, Debug)]
pub struct NativeArtifactRef {
    pub generation: String,
    pub profile: String,
    pub capabilities: Vec<String>,
    pub format: ObjectFormat,
    pub target: String,
    pub abi: String,
    pub backend: String,
    pub flags: Vec<String>,
    pub recipe: String,
    /// Claimed content hash of the object bytes (hex sha256).
    pub object_sha256: String,
    /// Claimed object byte length.
    pub object_len: u64,
    /// Manifest presence: a relocatable reference must carry its build
    /// recipe outputs (symbol/receipt sidecar); `false` rejects.
    pub has_manifest: bool,
}

/// The current admission context (what THIS build knows and grants).
#[derive(Clone, Debug, Default)]
pub struct NativeAdmissionCtx {
    /// Extra generations admitted by an explicit compatibility entry
    /// (empty by default — fail-closed on unknown generations).
    pub legacy_generations: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeAdmissionError {
    UnknownGeneration(String),
    UnknownProfile(String),
    UnknownCapability(String),
    NotRelocatable(&'static str),
    HashMismatch { claimed: String, actual: String },
    LengthMismatch { claimed: u64, actual: u64 },
    UnknownTarget(String),
    UnknownAbi(String),
    UnknownBackend(String),
    UnknownRecipe(String),
    MissingManifest,
    MalformedObject(&'static str),
}

impl std::fmt::Display for NativeAdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NativeAdmissionError::UnknownGeneration(g) => {
                write!(f, "native admission: generation `{g}` is not admitted")
            }
            NativeAdmissionError::UnknownProfile(p) => {
                write!(f, "native admission: profile `{p}` is not admitted")
            }
            NativeAdmissionError::UnknownCapability(c) => {
                write!(f, "native admission: capability `{c}` is not admitted")
            }
            NativeAdmissionError::NotRelocatable(w) => {
                write!(f, "native admission: artifact is not relocatable ({w})")
            }
            NativeAdmissionError::HashMismatch { claimed, actual } => write!(
                f,
                "native admission: object hash {claimed} does not bind offered bytes {actual}"
            ),
            NativeAdmissionError::LengthMismatch { claimed, actual } => write!(
                f,
                "native admission: object length {claimed} != offered {actual}"
            ),
            NativeAdmissionError::UnknownTarget(t) => {
                write!(f, "native admission: target `{t}` is not admitted")
            }
            NativeAdmissionError::UnknownAbi(a) => {
                write!(f, "native admission: abi `{a}` is not admitted")
            }
            NativeAdmissionError::UnknownBackend(b) => {
                write!(f, "native admission: backend `{b}` is not admitted")
            }
            NativeAdmissionError::UnknownRecipe(r) => {
                write!(f, "native admission: recipe `{r}` is not admitted")
            }
            NativeAdmissionError::MissingManifest => {
                write!(f, "native admission: reference carries no build manifest")
            }
            NativeAdmissionError::MalformedObject(w) => {
                write!(f, "native admission: object bytes malformed ({w})")
            }
        }
    }
}

/// Admit one native artifact reference against the current context and
/// the ACTUAL object bytes offered with it.
pub fn admit_native_ref(
    r: &NativeArtifactRef,
    object_bytes: &[u8],
    ctx: &NativeAdmissionCtx,
) -> Result<(), NativeAdmissionError> {
    if !ADMITTED_GENERATIONS.contains(&r.generation.as_str())
        && !ctx.legacy_generations.contains(&r.generation)
    {
        return Err(NativeAdmissionError::UnknownGeneration(
            r.generation.clone(),
        ));
    }
    if r.profile != ADMITTED_NATIVE_PROFILE {
        return Err(NativeAdmissionError::UnknownProfile(r.profile.clone()));
    }
    for c in &r.capabilities {
        if !ADMITTED_NATIVE_CAPABILITIES.contains(&c.as_str()) {
            return Err(NativeAdmissionError::UnknownCapability(c.clone()));
        }
    }
    if r.target != ADMITTED_TARGET {
        return Err(NativeAdmissionError::UnknownTarget(r.target.clone()));
    }
    if r.abi != ADMITTED_ABI {
        return Err(NativeAdmissionError::UnknownAbi(r.abi.clone()));
    }
    if r.backend != ADMITTED_BACKEND {
        return Err(NativeAdmissionError::UnknownBackend(r.backend.clone()));
    }
    if r.recipe != ADMITTED_RECIPE {
        return Err(NativeAdmissionError::UnknownRecipe(r.recipe.clone()));
    }
    if !r.has_manifest {
        return Err(NativeAdmissionError::MissingManifest);
    }
    // Content binding: the reference names the EXACT bytes offered.
    let actual_len = object_bytes.len() as u64;
    if r.object_len != actual_len {
        return Err(NativeAdmissionError::LengthMismatch {
            claimed: r.object_len,
            actual: actual_len,
        });
    }
    let actual_hash = content_hash(object_bytes);
    if !r.object_sha256.eq_ignore_ascii_case(&actual_hash) {
        return Err(NativeAdmissionError::HashMismatch {
            claimed: r.object_sha256.clone(),
            actual: actual_hash,
        });
    }
    // Relocatability: the bytes must BE a COFF machine object, and the
    // CLAIMED format must be the relocatable one.
    check_coff_relocatable(object_bytes)?;
    match &r.format {
        ObjectFormat::CoffRelocatable => Ok(()),
        ObjectFormat::LinkedImage => Err(NativeAdmissionError::NotRelocatable(
            "reference claims a linked image (process addresses)",
        )),
        ObjectFormat::Other(f) => Err(NativeAdmissionError::NotRelocatable(Box::leak(
            f.clone().into_boxed_str(),
        ))),
    }
}

/// Structural COFF object validation (little-endian COFF header + the
/// table RANGES the header claims): machine = AMD64 (0x8664), ≥1 section,
/// no optional header (objects), a symbol table with ≥1 record, and NOT a
/// linked image (no `MZ`/PE signature prefix). F798-R6-06: the header's
/// claimed structures must actually FIT the offered bytes — section
/// headers, the symbol table and the string table length are range
/// checked, so a 20-byte stub with out-of-range pointers never admits.
pub fn check_coff_relocatable(bytes: &[u8]) -> Result<(), NativeAdmissionError> {
    const SYM_SIZE: u64 = 18;
    const SECTION_HEADER_SIZE: u64 = 40;
    if (bytes.len() as u64) < 20 {
        return Err(NativeAdmissionError::MalformedObject(
            "shorter than a COFF header",
        ));
    }
    if bytes[0] == b'M' && bytes[1] == b'Z' {
        return Err(NativeAdmissionError::MalformedObject(
            "MZ/PE image bytes are not a relocatable object",
        ));
    }
    let machine = u16::from_le_bytes([bytes[0], bytes[1]]);
    if machine != 0x8664 {
        return Err(NativeAdmissionError::MalformedObject(
            "machine is not AMD64",
        ));
    }
    let n_sections = u16::from_le_bytes([bytes[2], bytes[3]]) as u64;
    if n_sections == 0 {
        return Err(NativeAdmissionError::MalformedObject("no sections"));
    }
    let n_symbols = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as u64;
    if n_symbols == 0 {
        return Err(NativeAdmissionError::MalformedObject(
            "no symbol table records — not a linkable object",
        ));
    }
    let size_of_optional = u16::from_le_bytes([bytes[16], bytes[17]]) as u64;
    if size_of_optional != 0 {
        return Err(NativeAdmissionError::MalformedObject(
            "optional header present — object form required for a relocatable reference",
        ));
    }
    let len = bytes.len() as u64;
    // Section headers must fit between the COFF header and the end.
    let sections_end = 20 + n_sections * SECTION_HEADER_SIZE;
    if sections_end > len {
        return Err(NativeAdmissionError::MalformedObject(
            "section table overruns the object",
        ));
    }
    let sym_ptr = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as u64;
    if sym_ptr == 0 {
        return Err(NativeAdmissionError::MalformedObject(
            "no symbol table — not a linkable object",
        ));
    }
    let symbols_end = sym_ptr
        .checked_add(n_symbols.checked_mul(SYM_SIZE).expect("bounded sizes"))
        .expect("bounded offsets");
    if symbols_end > len {
        return Err(NativeAdmissionError::MalformedObject(
            "symbol table overruns the object",
        ));
    }
    // F798-R7-04: every section header's claimed raw-data and relocation
    // windows must fit the offered bytes — a header declaring data at an
    // out-of-range pointer is not a valid relocatable object, whatever the
    // outer tables claim.
    for si in 0..n_sections {
        let sh = (20 + si * SECTION_HEADER_SIZE) as usize;
        let u16_at = |o: usize| u16::from_le_bytes([bytes[sh + o], bytes[sh + o + 1]]);
        let u32_at = |o: usize| {
            u32::from_le_bytes([
                bytes[sh + o],
                bytes[sh + o + 1],
                bytes[sh + o + 2],
                bytes[sh + o + 3],
            ]) as u64
        };
        let size_of_raw = u32_at(16);
        let ptr_raw = u32_at(20);
        if size_of_raw > 0 {
            // saturating: a u32::MAX pointer plus a size must reject, not
            // overflow the checked add.
            let end = ptr_raw.saturating_add(size_of_raw);
            if end > len || ptr_raw < sections_end {
                return Err(NativeAdmissionError::MalformedObject(
                    "section raw-data window overruns the object",
                ));
            }
        }
        let n_relocs = u16_at(32) as u64;
        if n_relocs > 0 {
            let ptr_rel = u32_at(24);
            let end = ptr_rel
                .checked_add(n_relocs.saturating_mul(10))
                .expect("bounded sizes");
            if end > len {
                return Err(NativeAdmissionError::MalformedObject(
                    "section relocation table overruns the object",
                ));
            }
        }
    }
    // The string table follows the symbol table: a u32 byte length that
    // must close exactly at the object end (≥4 for the length field).
    if symbols_end + 4 <= len {
        let str_len = u32::from_le_bytes([
            bytes[symbols_end as usize],
            bytes[symbols_end as usize + 1],
            bytes[symbols_end as usize + 2],
            bytes[symbols_end as usize + 3],
        ]) as u64;
        if str_len < 4 || symbols_end + str_len != len {
            return Err(NativeAdmissionError::MalformedObject(
                "string table length does not close the object",
            ));
        }
    } else if symbols_end != len {
        return Err(NativeAdmissionError::MalformedObject(
            "symbol table does not close the object",
        ));
    }
    Ok(())
}

/// Content hash used for artifact binding (blake3, hex) — same primitive
/// family the shared sema identity domain uses.
pub fn content_hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Mint the reference for bytes produced by the landed native lowering —
/// the POSITIVE control path (a REAL COFF object, not a mock).
pub fn ref_for_coff(bytes: &[u8]) -> NativeArtifactRef {
    NativeArtifactRef {
        generation: ADMITTED_GENERATIONS[0].to_string(),
        profile: ADMITTED_NATIVE_PROFILE.to_string(),
        capabilities: vec![],
        format: ObjectFormat::CoffRelocatable,
        target: ADMITTED_TARGET.to_string(),
        abi: ADMITTED_ABI.to_string(),
        backend: ADMITTED_BACKEND.to_string(),
        flags: vec![],
        recipe: ADMITTED_RECIPE.to_string(),
        object_sha256: content_hash(bytes),
        object_len: bytes.len() as u64,
        has_manifest: true,
    }
}

/// Lower a checked module with the LANDED backend and mint+admit its
/// reference in one step (the reference positive control used by tests
/// and the boundary tier).
pub fn mint_and_admit_lowered(
    checked: &CheckedModule,
    entry: &str,
    ctx: &NativeAdmissionCtx,
) -> Result<(NativeArtifactRef, Vec<u8>), NativeAdmissionError> {
    let sel = native::find_entry(checked, entry)
        .map_err(|_| NativeAdmissionError::MalformedObject("entry not found for lowering"))?;
    let bytes = native::lower_object(checked, &sel, &native::RuntimeInjection::default())
        .map_err(|_| NativeAdmissionError::MalformedObject("lowering failed"))?;
    let r = ref_for_coff(&bytes);
    admit_native_ref(&r, &bytes, ctx)?;
    Ok((r, bytes))
}
