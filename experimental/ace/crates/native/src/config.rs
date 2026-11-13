//! Frozen identities, layers and budgets of the MIR-native backend
//! (PLAN-818 T-01 contract-decision.md; r1.1 note: 798's merged ABC wire
//! moved to `auto.abc.av-probe/2` — irrelevant to A09' keys, which bind
//! the MIR schema/semantics that stayed `/1`/`1`).

use ace_artifacts::schema::{LayerCoverage, LayerSpec, Regeneration};
use std::time::Duration;

/// Producer identity of this backend (store envelopes and keys).
pub const PRODUCER_ID: &str = "plan818/mir-native";
/// A09' layer: per-function relocatable COFF objects.
pub const COFF_SCHEMA: &str = "ac818/a09-coff/v1";
/// A10' layer: module link manifests and link receipts.
pub const LINK_SCHEMA: &str = "ac818/a10-link/v1";

/// The ONE admitted target triple — never the host triple silently.
pub const MN_TARGET: &str = "x86_64-pc-windows-msvc";
/// Private test ABI tag (NOT the PLAN-797 public AutoABI).
pub const MN_ABI: &str = "msvc-x64/priv-i32-bool-v1";
/// Backend identity tag (Cranelift family; the workspace lock pins the
/// exact version — key axis stays coarse on purpose).
pub const MN_BACKEND_TAG: &str = "cranelift/0.126";
/// Codegen flags frozen by the contract (opt_level/is_pic/cpu baseline).
pub const MN_FLAGS: &str = "opt=none,pic=false,cpu=baseline";
/// Runtime policy tag: trace intrinsics + trap exit contract.
pub const MN_RUNTIME_TAG: &str = "trace=hir.test.trace.v1,trapexit=70";

/// Exit code of the frozen experiment trap path (landed contract).
pub const TRAP_EXIT_CODE: i32 = 70;
/// Startup wrapper symbol (landed experiment contract).
pub const START_SYMBOL: &str = "ac_start";

// ---- PLAN-947 console lane (console/1 on the native backend) -----------

/// Exit code of the console-lane resource rejection (per-call or
/// cumulative console/1 budget exceeded, or provider write failure).
/// Distinct from the frozen trap exit 70; console corpus entry results
/// keep away from both codes by contract.
pub const CONSOLE_REJECT_EXIT_CODE: i32 = 71;
/// The console shim symbol every console unit's HostCall emission
/// targets; defined once per console link by the shim object (kernel32
/// GetStdHandle/WriteFile + the cumulative run-byte counter).
pub const CONSOLE_WRITE_SYMBOL: &str = "mn_console_write";
/// Cumulative console output counter (writable data symbol in the shim).
pub const CONSOLE_RUN_BYTES_SYMBOL: &str = "mn_console_run_bytes";
/// Unit-local console staging scratch (fill→dispatch window holds no
/// guest call; one private copy per emitted console unit).
pub const CONSOLE_SCRATCH_SYMBOL: &str = "mn_console_scratch";

// ---- PLAN-891 typed-data ABI / transport identities (freeze T-02) -------
//
// An INDEPENDENT, versioned mn-private typed ABI — deliberately NOT the
// scalar `MN_ABI` above and NOT the PLAN-797 public AutoABI. Six logical
// scalars (i32/bool/i64/u64/f32/f64) cross as integer bit carriers:
// I32/Bool/F32 ride I32 (low 32 bits, 4-byte frame/storage width), I64/
// U64/F64 ride I64 (8 bytes). F32/F64 are OPAQUE bit patterns end to end
// — no float arithmetic, formatting or widening anywhere in the channel.

/// Private typed-data ABI tag (PLAN-891 §2 freeze).
pub const TYPED_ABI: &str = "msvc-x64/priv-typed-data-v1";
/// Runtime policy tag of the typed wrapper/helper channel.
pub const TYPED_RUNTIME_TAG: &str =
    "typed-frames=mn891.typed.frames/v1,helper=ace-typed-entry-runtime";
/// Store schema of the cached typed wrapper object.
pub const TYPED_WRAPPER_SCHEMA: &str = "ac891/typed-wrapper/v1";
/// Typed export symbol domain (isolated from the scalar `ac8.` domain).
pub const TYPED_SYMBOL_PREFIX: &str = "ac9";
/// One-shot stdin request frame cap (header + payload + hash).
pub const TYPED_MAX_FRAME_BYTES: usize = 256 * 1024;
/// Typed argument payload cap (checked BEFORE any allocation).
pub const TYPED_MAX_ARGS_BYTES: usize = 64 * 1024;

/// Logical MIR schema/semantics axes consumed from the 798 surface.
pub const MIR_SCHEMA_AXIS: &str = "auto.mir.av-probe/1";
pub const MIR_SEMANTICS_AXIS: &str = "1";

// ---- frozen native-side budgets (contract-decision §7.2) ----------------

pub const MAX_INPUT_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_COFF_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_REQUEST_OBJECTS_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_LINK_INPUT_BYTES: usize = 128 * 1024 * 1024;
pub const STREAM_CAP: usize = 1024 * 1024;
pub const METADATA_CAP: usize = 4 * 1024 * 1024;
pub const PAGE_BYTES: usize = 256 * 1024;
pub const BUILD_BUDGET: Duration = Duration::from_secs(60);
pub const RUN_BUDGET: Duration = Duration::from_secs(5);
pub const REQUEST_BUDGET: Duration = Duration::from_secs(70);

// ---- store layers (independent producer namespace) -----------------------

pub const COFF_LAYER: LayerSpec = LayerSpec {
    id: "A09",
    name: "backend-object (plan818 per-function COFF units)",
    schema_id: COFF_SCHEMA,
    producer: PRODUCER_ID,
    consumers: &["plan818 link driver", "plan818 worker restore"],
    key_inputs: &[
        "producer",
        "owner-module",
        "unit",
        "profile",
        "unit-semantic-fp",
        "mir-schema",
        "mir-semantics",
        "backend",
        "target",
        "flags",
        "abi",
        "runtime",
    ],
    ref_kinds: &["required-data", "provenance"],
    storage: "one relocatable COFF per function unit, paged <=256KiB (LayoutData carrier)",
    residency: "restored objects are structurally admitted (COFF + sha/len bind) before link",
    regeneration: Regeneration::FromUpstreamRecipe,
    coverage: LayerCoverage::Codec,
};

pub const LINK_LAYER: LayerSpec = LayerSpec {
    id: "A10",
    name: "link manifest (plan818 module closures)",
    schema_id: LINK_SCHEMA,
    producer: PRODUCER_ID,
    consumers: &["plan818 link driver", "plan818 worker restore"],
    key_inputs: &[
        "producer",
        "owner-module",
        "entry",
        "unit-fps",
        "link-inputs",
        "abi",
        "entry-symbol",
    ],
    ref_kinds: &["required-data"],
    storage: "one manifest per closure composition (LayoutData carrier)",
    residency: "manifest binds unit artifact ids + tool identities; restore admits before link",
    regeneration: Regeneration::FromUpstreamRecipe,
    coverage: LayerCoverage::Codec,
};

/// PLAN-891 typed wrapper layer: the signature-shaped startup wrapper is a
/// RESIDENT product cached by (abi, entry symbol, signature id, target,
/// flags, transport, helper library identity) — never by argument values
/// or nonces (a same-signature rerun produces nothing).
pub const TYPED_WRAPPER_LAYER: LayerSpec = LayerSpec {
    id: "A16",
    name: "backend-object (plan891 typed startup wrappers)",
    schema_id: TYPED_WRAPPER_SCHEMA,
    producer: PRODUCER_ID,
    consumers: &["plan891 typed pipeline", "plan891 worker restore"],
    key_inputs: &[
        "producer",
        "schema",
        "abi",
        "entry-symbol",
        "sig-id",
        "target",
        "flags",
        "transport",
        "helper-library",
    ],
    ref_kinds: &["required-data"],
    storage: "one relocatable COFF wrapper per typed signature (LayoutData carrier)",
    residency: "restored wrappers are structurally admitted (COFF + sha/len bind) before link",
    regeneration: Regeneration::FromUpstreamRecipe,
    coverage: LayerCoverage::Codec,
};

/// PLAN-1017: the source-memo schema — a content-addressed
/// source→link-manifest pointer keyed by the FULL compile-request identity
/// (source sha256 + entry + capabilities + the pipeline identity axes).
pub const SOURCE_MEMO_SCHEMA: &str = "ac818/a17-source-memo/v1";

/// PLAN-1017: the A17 source memo. A hit does NOT recompile the source:
/// it restores the recorded link manifest and re-admits every unit from
/// its bytes through the SAME admission discipline the worker restore
/// uses (identity axes + sha/len binding), then re-links and runs. The
/// memo never re-trusts a "verified" flag and never bypasses admission;
/// a missing unit, toolchain drift (manifest link axes vs the current
/// identities) or a foreign/corrupt carrier is a clean miss back to the
/// full pipeline. The pipeline identity axes in the key mean a semantic
/// frontend/lowering change MUST bump one of the axes constants — the
/// same re-keying discipline the CTE engine identity carries.
pub const SOURCE_MEMO_LAYER: LayerSpec = LayerSpec {
    id: "A17",
    name: "source-memo (plan1017 source→manifest pointer)",
    schema_id: SOURCE_MEMO_SCHEMA,
    producer: PRODUCER_ID,
    consumers: &["plan1017 mn driver hot path", "plan818 mn-worker case-run"],
    key_inputs: &["producer", "schema", "src-sha256", "entry", "caps", "pipeline"],
    ref_kinds: &["required-data"],
    storage: "one memo per compile-request identity (LayoutData carrier)",
    residency: "memo binds one link-manifest artifact id; restore admits before link",
    regeneration: Regeneration::FromUpstreamRecipe,
    coverage: LayerCoverage::Codec,
};
