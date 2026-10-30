//! PLAN-820 F820-R6-01: the frozen `av.data-fixture/1` typed-fixture
//! table — the formal recipe's typed entry face. Wide scalars have no
//! source syntax; a `typed:<id>` entry selects one of these frozen MIR
//! fixtures (`fn () -> T { return <const> }`), run through the SAME data
//! pipeline as every admitted unit (verify_mir → AVAD encode →
//! verify_abc_bytes → AV execute_entry). The WORKER reports the actually
//! produced value as a canonical `{kind, bits}` pair; the JUDGING PARENT
//! holds its OWN copy of the expected table and compares tag and exact
//! raw bits independently — the worker's word is never the oracle.
//!
//! Canonical bits encoding (both sides): lowercase hex, no `0x`; i64/u64/
//! f64 are 16 digits, f32 is 8 digits, i32/bool are unpadded.

use ace_av_core::abc::encode_abc;
use ace_av_core::abc_verify::verify_abc_bytes;
use ace_av_core::av::{execute_entry, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value, MIR_DATA_SCHEMA};
use ace_av_core::mir::OwnerScope;
use ace_av_core::mir::*;
use ace_av_core::mir_verify::verify_mir;

/// One frozen fixture: id, result kind tag, expected canonical bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedFixture {
    pub id: &'static str,
    pub kind: &'static str,
    pub bits: &'static str,
}

/// The frozen fixture set (contract-decision §3 bit samples + the bool/i32
/// faces the record domain already supports).
pub const TYPED_FIXTURES: &[TypedFixture] = &[
    TypedFixture {
        id: "u64-max",
        kind: "u64",
        bits: "ffffffffffffffff",
    },
    TypedFixture {
        id: "i64-min",
        kind: "i64",
        bits: "8000000000000000",
    },
    TypedFixture {
        id: "fneg-nan",
        kind: "f64",
        bits: "fff80000deadbeef",
    },
    TypedFixture {
        id: "neg-zero",
        kind: "f64",
        bits: "8000000000000000",
    },
    TypedFixture {
        id: "f32-one",
        kind: "f32",
        bits: "3f800000",
    },
    // The contract's frozen f32 NaN payload sample (contract-decision §3:
    // NaN(payload)=0x7F80_AD00) — F820-R7-02.
    TypedFixture {
        id: "f32-nan-payload",
        kind: "f32",
        bits: "7f80ad00",
    },
    TypedFixture {
        id: "bool-true",
        kind: "bool",
        bits: "1",
    },
    TypedFixture {
        id: "i32-42",
        kind: "i32",
        bits: "2a",
    },
];

pub fn typed_fixture(id: &str) -> Option<&'static TypedFixture> {
    TYPED_FIXTURES.iter().find(|f| f.id == id)
}

fn owner() -> OwnerScope {
    OwnerScope {
        schema: MIR_DATA_SCHEMA.to_string(),
        module: "m_typed_fixture".to_string(),
    }
}

fn scalar(kind: &str) -> MirType {
    match kind {
        "i32" => MirType::I32,
        "bool" => MirType::Bool,
        "i64" => MirType::I64,
        "u64" => MirType::U64,
        "f32" => MirType::F32,
        _ => MirType::F64,
    }
}

fn konst(f: &TypedFixture) -> MirConst {
    let bits = u64::from_str_radix(f.bits, 16).expect("frozen hex bits");
    match f.kind {
        "i32" => MirConst::I32(bits as i32),
        "bool" => MirConst::Bool(bits != 0),
        "i64" => MirConst::I64(bits as i64),
        "u64" => MirConst::U64(bits),
        "f32" => MirConst::F32Bits(bits as u32),
        _ => MirConst::F64Bits(bits),
    }
}

/// Build the frozen fixture's one-function data module:
/// `fn main() -> T { return <const>; }` — an untrusted typed input the
/// REAL admission chain must verify before execution.
pub fn fixture_module(f: &TypedFixture) -> MirModule {
    let mut m = MirModule::new_data(owner());
    let t = m.intern_type(scalar(f.kind));
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: t,
    }));
    let c = m.intern_const(konst(f));
    m.funcs.push(MirFunction {
        owner: owner(),
        name: "main".into(),
        sig,
        slots: vec![MirSlot {
            kind: SlotKind::Local,
            ty: t,
            name: None,
        }],
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::Const { dst: 0, value: c }],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        env: None,
        spans: vec![],
    });
    m.entry = 0;
    m
}

/// Canonical `{kind, bits}` of an actually produced AV value; `None` for
/// non-scalar results (records never occur on this face).
pub fn canonical_typed(value: &Value) -> Option<(&'static str, String)> {
    Some(match value {
        Value::I32(v) => ("i32", format!("{:x}", v)),
        // PLAN-862: the unit result reports its own kind with an EMPTY bits
        // field — distinguishable from i32:0 by construction.
        Value::Unit => ("unit", String::new()),
        Value::Bool(v) => ("bool", if *v { "1".into() } else { "0".into() }),
        Value::I64(v) => ("i64", format!("{:016x}", v)),
        Value::U64(v) => ("u64", format!("{:016x}", v)),
        Value::F32(v) => ("f32", format!("{:08x}", v.to_bits())),
        Value::F64(v) => ("f64", format!("{:016x}", v.to_bits())),
        // PLAN-821 kinds: the scalar face gains char; owned kinds report
        // their content fingerprint (records-with-resources never occur on
        // this face — entry results only).
        Value::Char(c) => ("char", format!("{:x}", *c as u32)),
        Value::Str(h) => (
            "Str",
            format!("{:016x}", ace_av_core::contract::fnv1a64(h.bytes())),
        ),
        // PLAN-824: a Dyn reports the fingerprint of its frozen portable
        // bytes (deterministic content identity).
        Value::Dyn(d) => (
            "dyn",
            match ace_av_core::portable::freeze(
                d,
                &ace_av_core::dynamic::TraversalOptions::default(),
            ) {
                Ok(bytes) => format!("{:016x}", ace_av_core::contract::fnv1a64(&bytes)),
                Err(_) => "unfreezable".to_string(),
            },
        ),
        Value::StrView(_) | Value::MutRef(_) => return None,
        Value::Record(_) => return None,
        // PLAN-822 containers never occur on the scalar typed face
        // (entry results only); the collections face uses canonical_owned.
        Value::List(_) | Value::Variant(_) => return None,
        Value::Place(_) => return None,
        // PLAN-823: closure metadata is execution-internal, never an entry
        // result face.
        Value::Closure(_) => return None,
    })
}

/// Run one frozen fixture through the real admission chain and return the
/// produced value. `(kind, bits)` reporting is the caller's — this stays
/// the pure pipeline.
pub fn run_fixture(f: &TypedFixture) -> Result<Value, String> {
    let m = fixture_module(f);
    verify_mir(&m).map_err(|e| format!("fixture {}: verify: {:?}", f.id, e))?;
    let wire = encode_abc(&m).map_err(|e| format!("fixture {}: encode: {}", f.id, e.code()))?;
    let loaded = verify_abc_bytes(&wire.to_flat(), &m.owner)
        .map_err(|e| format!("fixture {}: admit: {}", f.id, e))?;
    let (out, _trace) = execute_entry(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &[],
    );
    match out {
        ExecOutcome::Success { value, .. } => Ok(value),
        other => Err(format!("fixture {}: run: {:?}", f.id, other)),
    }
}
