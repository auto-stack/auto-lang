//! Target/domain physical plan projection (937 plan §5.1 "物理计划" row).
//!
//! Projection INPUTS are logical facts; OUTPUT is the machine plan for
//! the CURRENT frozen candidate domain only:
//! * domain `autoabi`, domain version `0.1`;
//! * target `x86_64-pc-windows-msvc`, profile
//!   `autoabi.win-x64.v0.1-candidate`, packing `natural-align`
//!   — byte-frozen constants that MUST stay consistent with the 797
//!   `ace-abi` layout/descriptor constants (T-03 asserts the match; it
//!   does not move ownership);
//! * per-parameter register/stack classification (Windows x64 is
//!   POSITIONAL: argument position < 4 uses that position's register —
//!   RCX/RDX/R8/R9 or XMM0..3 by lane class — position >= 4 is the
//!   stack), so a C header author and the Cranelift signature builder
//!   spell the SAME physical form;
//! * scalar return lanes: integers/bool/char in RAX, f64 in XMM0, unit
//!   = void.
//!
//! The plan does NOT invent new ABI surface: no f32 lane, no aggregate
//! by-value, no varargs (plan §1 non-goals). Anything the logical facts
//! cannot express is a named rejection here, before any emission.

use crate::facts::{FnForm, InterfaceFacts};
use crate::lane::{LaneKind, PhysLane, SlotBank};
use serde::{Deserialize, Serialize};

pub const PLAN_FORMAT: &str = "autoabi.contract.plan/1";
pub const DOMAIN: &str = "autoabi";
pub const DOMAIN_VERSION: &str = "0.1";
pub const TARGET: &str = "x86_64-pc-windows-msvc";
pub const PROFILE: &str = "autoabi.win-x64.v0.1-candidate";
pub const PACKING: &str = "natural-align";

/// Windows x64 argument registers. The convention is POSITIONAL (not
/// per-type-bank): argument position `n < ARG_SLOTS` travels in the
/// position's register — `RCX/RDX/R8/R9` for integer-class lanes,
/// `XMM0..3` for F64 — and every position `>= ARG_SLOTS` goes on the
/// stack, regardless of type. Verified against BOTH real producers on
/// this toolchain pair: MSVC 14.44 codegen (`int a, double b, int c`
/// puts `b` in **XMM1**; the 5th argument of `int×4,double` is read
/// from `[rsp+0x28]`) and Cranelift 0.126 `WindowsFastcall`
/// (`get_intreg_for_arg`/`get_fltreg_for_arg` select by absolute
/// `arg_idx`). The two producers agree because both are positional.
pub const ARG_SLOTS: usize = 4;

/// Which bank a parameter travels in, and whether it landed in a
/// register or on the stack (stack slots carry their lane's bank in
/// [`PhysParam::stack_bank`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SlotClass {
    GpRegister { index: usize },
    FpRegister { index: usize },
    Stack { index: usize },
}

/// One parameter's physical form.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysParam {
    pub name: String,
    pub lane: PhysLane,
    pub slot: SlotClass,
    /// The stack lane's bank (only meaningful for stack slots).
    #[serde(default)]
    pub stack_bank: Option<SlotBank>,
}

/// Physical return lane (`None` = void/status-only handling at the form
/// level).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum PhysRet {
    RaxI32,
    RaxU32,
    RaxI64,
    RaxU64,
    Xmm0F64,
    RaxBoolU8,
    RaxCharU32,
    Void,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PhysForm {
    Pure {
        params: Vec<PhysParam>,
        ret: PhysRet,
    },
    Fallible {
        /// The explicit ctx lane is materialized as its FIRST param
        /// (i64 pointer-width GP lane).
        ctx: Option<bool>,
        /// Full physical param list INCLUDING the ctx lane (first) and
        /// one pointer-width out-slot lane per declared out (trailing).
        params: Vec<PhysParam>,
        /// The DECLARED out lanes in declaration order (the value types
        /// the caller's out pointers point at).
        outs: Vec<PhysLane>,
    },
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysFn {
    pub symbol: String,
    pub form: PhysForm,
}

/// The full projected plan.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalPlan {
    pub format: String,
    pub domain: String,
    pub domain_version: String,
    pub target: String,
    pub profile: String,
    pub packing: String,
    pub functions: Vec<PhysFn>,
}

/// Classify parameters Windows-x64 style: POSITIONAL slots (see
/// [`ARG_SLOTS`]). Position < 4 takes the position's register (GP for
/// integer-class lanes, FP for F64); position >= 4 goes on the stack.
fn classify(params: &[(String, PhysLane, SlotBank)]) -> Vec<PhysParam> {
    params
        .iter()
        .enumerate()
        .map(|(pos, (name, lane, bank))| {
            let (slot, sbank) = if pos < ARG_SLOTS {
                let s = match bank {
                    SlotBank::Gp => SlotClass::GpRegister { index: pos },
                    SlotBank::Fp => SlotClass::FpRegister { index: pos },
                };
                (s, None)
            } else {
                (
                    SlotClass::Stack {
                        index: pos - ARG_SLOTS,
                    },
                    Some(*bank),
                )
            };
            PhysParam {
                name: name.clone(),
                lane: *lane,
                slot,
                stack_bank: sbank,
            }
        })
        .collect()
}

fn ret_lane(ret: LaneKind) -> PhysRet {
    match ret {
        LaneKind::I32 => PhysRet::RaxI32,
        LaneKind::U32 => PhysRet::RaxU32,
        LaneKind::I64 => PhysRet::RaxI64,
        LaneKind::U64 => PhysRet::RaxU64,
        LaneKind::F64 => PhysRet::Xmm0F64,
        LaneKind::Bool => PhysRet::RaxBoolU8,
        LaneKind::Char => PhysRet::RaxCharU32,
    }
}

/// Scalar (937/938) form projection — unchanged behavior, extracted.
fn project_scalar(form: &FnForm) -> Result<PhysForm, String> {
    Ok(match form {
        FnForm::Pure { params, ret } => {
            let lanes: Vec<(String, PhysLane, SlotBank)> = params
                .iter()
                .enumerate()
                .map(|(i, k)| (format!("p{i}"), k.physical(), k.physical().slot_class()))
                .collect();
            PhysForm::Pure {
                params: classify(&lanes),
                ret: ret_lane(*ret),
            }
        }
        FnForm::Fallible { ctx, params, outs } => {
            // ctx lane (pointer width, GP), then declared params,
            // then one pointer-width GP lane per out slot.
            let mut all: Vec<(String, PhysLane, SlotBank)> = Vec::new();
            if ctx.is_some() {
                // Pointer lanes are GP-class; PhysLane has no raw
                // pointer variant by design — the ctx lane is
                // spelled by the form, classified as GP I64-width.
                all.push(("ctx".into(), PhysLane::I64, SlotBank::Gp));
            }
            for (i, k) in params.iter().enumerate() {
                let p = k.physical();
                all.push((format!("p{i}"), p, p.slot_class()));
            }
            // Out slots travel as trailing pointer-width lanes.
            for (i, k) in outs.iter().enumerate() {
                let p = k.physical();
                all.push((format!("out{i}_ptr"), p, p.slot_class()));
            }
            PhysForm::Fallible {
                ctx: *ctx,
                params: classify(&all),
                outs: outs.iter().map(|k| k.physical()).collect(),
            }
        }
    })
}

/// PLAN-939 resource physical projection (Windows x64). Pointer lanes are
/// GP I64-width by design (the 797 ctx-lane convention); the view pair is
/// (ptr, u64 len); the callback fnptr lane is followed by the sign-
/// extended i64 ctx lane spelled by the adjacent i32 ctx-seed param.
fn project_resource(res: &crate::resource::ResourceSig) -> Result<PhysForm, String> {
    use crate::resource::{ResKind, ResourceForm, ViewRole};
    res.validate()?;
    let mut all: Vec<(String, PhysLane, SlotBank)> = Vec::new();
    let mut out_lanes: Vec<PhysLane> = Vec::new();
    match &res.form {
        ResourceForm::Fallible { ctx, .. } => {
            if ctx.is_some() {
                all.push(("ctx".into(), PhysLane::I64, SlotBank::Gp));
            }
        }
        ResourceForm::Pure { .. } => {}
    }
    let mut i = 0usize;
    while i < res.params.len() {
        let after_cb = i > 0 && matches!(res.params[i - 1], ResKind::Callback { ctx: true });
        let p = &res.params[i];
        match p {
            ResKind::I32 => {
                if after_cb {
                    // The ctx seed: sign-extended to the i64 ctx lane.
                    all.push((format!("p{i}_ctx"), PhysLane::I64, SlotBank::Gp));
                } else {
                    all.push((format!("p{i}"), PhysLane::I32, SlotBank::Gp));
                }
            }
            ResKind::I64 => {
                // PLAN-939 S07 typed-peer wide seed/scalar: a native i64
                // lane (the original wide fold's ctx is a long long).
                all.push((format!("p{i}_w"), PhysLane::I64, SlotBank::Gp));
            }
            ResKind::Bool => all.push((format!("p{i}"), PhysLane::BoolU8, SlotBank::Gp)),
            ResKind::ViewText => {
                all.push((format!("p{i}_ptr"), PhysLane::I64, SlotBank::Gp));
                all.push((format!("p{i}_len"), PhysLane::U64, SlotBank::Gp));
            }
            ResKind::BufViewI64 => {
                // PLAN-939 S07: the element-count lane is spelled _n (NOT
                // _len) so a byte-count projection of the same view can
                // never compare equal to an element-count one.
                all.push((format!("p{i}_ptr"), PhysLane::I64, SlotBank::Gp));
                all.push((format!("p{i}_n"), PhysLane::U64, SlotBank::Gp));
            }
            ResKind::Rec { rec, role, .. } => {
                let tag = match role {
                    ViewRole::Out => "out",
                    _ => "view",
                };
                all.push((format!("p{i}_{tag}_{rec}"), PhysLane::I64, SlotBank::Gp));
            }
            ResKind::Handle { .. } => all.push((format!("p{i}_h"), PhysLane::I64, SlotBank::Gp)),
            ResKind::Callback { .. } => all.push((format!("p{i}_fn"), PhysLane::I64, SlotBank::Gp)),
        }
        i += 1;
    }
    match &res.form {
        ResourceForm::Pure { ret } => Ok(PhysForm::Pure {
            params: classify(&all),
            ret: resource_ret_lane(ret),
        }),
        ResourceForm::Fallible { out, .. } => {
            if let Some(o) = out {
                all.push(("out0_ptr".into(), resource_out_lane(o), SlotBank::Gp));
                out_lanes.push(resource_out_lane(o));
            }
            let ctx = match &res.form {
                ResourceForm::Fallible { ctx, .. } => *ctx,
                _ => None,
            };
            Ok(PhysForm::Fallible {
                ctx,
                params: classify(&all),
                outs: out_lanes,
            })
        }
    }
}

fn resource_ret_lane(ret: &crate::resource::ResOut) -> PhysRet {
    use crate::resource::ResOut;
    match ret {
        ResOut::I32 => PhysRet::RaxI32,
        ResOut::Bool => PhysRet::RaxBoolU8,
        ResOut::I64 | ResOut::Handle { .. } => PhysRet::RaxI64,
        ResOut::U64 => PhysRet::RaxU64,
    }
}

fn resource_out_lane(out: &crate::resource::ResOut) -> PhysLane {
    use crate::resource::ResOut;
    match out {
        ResOut::I32 => PhysLane::I32,
        ResOut::Bool => PhysLane::BoolU8,
        ResOut::I64 | ResOut::Handle { .. } => PhysLane::I64,
        ResOut::U64 => PhysLane::U64,
    }
}

impl PhysicalPlan {
    /// Project logical facts into the physical plan for the current
    /// frozen domain. Named rejections: unknown domain fields, forms the
    /// candidate ABI cannot spell.
    pub fn project(facts: &InterfaceFacts) -> Result<PhysicalPlan, String> {
        crate::work::observe("foreign-plan-project", false);
        facts.validate()?;
        let mut functions = Vec::new();
        for f in &facts.functions {
            let form = if let Some(res) = &f.resource {
                project_resource(res)?
            } else {
                project_scalar(&f.form)?
            };
            functions.push(PhysFn {
                symbol: f.symbol.clone(),
                form,
            });
        }
        let plan = PhysicalPlan {
            format: PLAN_FORMAT.into(),
            domain: DOMAIN.into(),
            domain_version: DOMAIN_VERSION.into(),
            target: TARGET.into(),
            profile: PROFILE.into(),
            packing: PACKING.into(),
            functions,
        };
        plan.validate()?;
        crate::work::observe("foreign-plan-project", true);
        Ok(plan)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.format != PLAN_FORMAT {
            return Err(format!(
                "unknown plan format '{}' (known '{PLAN_FORMAT}')",
                self.format
            ));
        }
        if self.domain != DOMAIN || self.domain_version != DOMAIN_VERSION {
            return Err(format!(
                "plan domain {}/{} is not the frozen {DOMAIN}/{DOMAIN_VERSION}",
                self.domain, self.domain_version
            ));
        }
        if self.target != TARGET || self.profile != PROFILE || self.packing != PACKING {
            return Err(format!(
                "plan target/profile/packing must be {TARGET}/{PROFILE}/{PACKING}"
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        for f in &self.functions {
            if !seen.insert(f.symbol.clone()) {
                return Err(format!("duplicate plan symbol '{}'", f.symbol));
            }
        }
        Ok(())
    }

    pub fn function(&self, symbol: &str) -> Option<&PhysFn> {
        self.functions.iter().find(|f| f.symbol == symbol)
    }

    /// Check a decoded plan against its logical input without projecting or
    /// allocating a replacement plan. Every argument position, bank, lane,
    /// context/out edge and return form is checked, including stack arguments.
    pub fn validate_against(&self, facts: &InterfaceFacts) -> Result<(), String> {
        self.validate()?;
        facts.validate()?;
        if self.functions.len() != facts.functions.len() {
            return Err("plan logical function coverage".into());
        }
        fn parameter(p: &PhysParam, position: usize, name: &str, lane: PhysLane) -> bool {
            let bank = lane.slot_class();
            let (slot, stack_bank) = if position < ARG_SLOTS {
                (
                    match bank {
                        SlotBank::Gp => SlotClass::GpRegister { index: position },
                        SlotBank::Fp => SlotClass::FpRegister { index: position },
                    },
                    None,
                )
            } else {
                (
                    SlotClass::Stack {
                        index: position - ARG_SLOTS,
                    },
                    Some(bank),
                )
            };
            p.name == name && p.lane == lane && p.slot == slot && p.stack_bank == stack_bank
        }
        for physical in &self.functions {
            let logical = facts
                .function(&physical.symbol)
                .ok_or("plan unknown logical symbol")?;
            if let Some(res) = &logical.resource {
                // PLAN-939 resource entry: the decoded physical form must
                // equal a fresh projection of the resource sig (bounded
                // per-form work; the plan itself is not re-projected).
                let expected = project_resource(res)?;
                if physical.form != expected {
                    return Err(format!(
                        "plan resource form mismatch for '{}'",
                        physical.symbol
                    ));
                }
                continue;
            }
            let valid = match (&physical.form, &logical.form) {
                (
                    PhysForm::Pure {
                        params: actual,
                        ret: actual_ret,
                    },
                    FnForm::Pure { params, ret },
                ) => {
                    *actual_ret == ret_lane(*ret)
                        && actual.len() == params.len()
                        && actual
                            .iter()
                            .zip(params)
                            .enumerate()
                            .all(|(i, (p, k))| parameter(p, i, &format!("p{i}"), k.physical()))
                }
                (
                    PhysForm::Fallible {
                        ctx: actual_ctx,
                        params: actual,
                        outs: actual_outs,
                    },
                    FnForm::Fallible { ctx, params, outs },
                ) => {
                    let offset = usize::from(ctx.is_some());
                    actual_ctx == ctx
                        && actual.len() == offset + params.len() + outs.len()
                        && actual_outs.len() == outs.len()
                        && actual_outs
                            .iter()
                            .zip(outs)
                            .all(|(a, k)| *a == k.physical())
                        && (offset == 0 || parameter(&actual[0], 0, "ctx", PhysLane::I64))
                        && params.iter().enumerate().all(|(i, k)| {
                            parameter(
                                &actual[offset + i],
                                offset + i,
                                &format!("p{i}"),
                                k.physical(),
                            )
                        })
                        && outs.iter().enumerate().all(|(i, k)| {
                            let position = offset + params.len() + i;
                            parameter(
                                &actual[position],
                                position,
                                &format!("out{i}_ptr"),
                                k.physical(),
                            )
                        })
                }
                _ => false,
            };
            if !valid {
                return Err(format!(
                    "plan logical/physical shape mismatch: {}",
                    physical.symbol
                ));
            }
        }
        Ok(())
    }

    /// Canonical JSON (functions sorted) + digest.
    pub fn canonical(&self) -> Result<String, String> {
        let mut sorted = self.clone();
        sorted.functions.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        serde_json::to_string(&sorted).map_err(|e| e.to_string())
    }

    pub fn digest(&self) -> Result<String, String> {
        Ok(crate::blake3_hex(self.canonical()?.as_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::FactsSource;
    use crate::lane::LaneKind;

    fn facts_8lanes() -> InterfaceFacts {
        // i32,u32,i64,u64,f64,bool,char,i64 -> i64 : 7 GP-class + 1 FP.
        crate::plan::tests_support::checksum_facts()
    }

    #[test]
    fn mixed_8_lanes_classify_positional() {
        let plan = PhysicalPlan::project(&facts_8lanes()).unwrap();
        let f = plan.function("t937_mix").unwrap();
        match &f.form {
            PhysForm::Pure { params, .. } => {
                assert_eq!(params.len(), 8);
                // Positions 0-3 take the positional GP registers.
                assert!(matches!(params[0].slot, SlotClass::GpRegister { index: 0 }));
                assert!(matches!(params[1].slot, SlotClass::GpRegister { index: 1 }));
                assert!(matches!(params[2].slot, SlotClass::GpRegister { index: 2 }));
                assert!(matches!(params[3].slot, SlotClass::GpRegister { index: 3 }));
                // Position 4 is the f64 — POSITIONAL convention: the 5th
                // argument goes on the STACK (verified against real MSVC
                // and Cranelift WindowsFastcall), NOT into an XMM.
                assert!(matches!(params[4].slot, SlotClass::Stack { index: 0 }));
                assert_eq!(params[4].stack_bank, Some(SlotBank::Fp));
                // Positions 5-7 also stack.
                assert!(matches!(params[5].slot, SlotClass::Stack { index: 1 }));
                assert!(matches!(params[6].slot, SlotClass::Stack { index: 2 }));
                assert!(matches!(params[7].slot, SlotClass::Stack { index: 3 }));
                // Physical lanes carry the bool->u8 / char->u32 projection.
                assert_eq!(params[5].lane, PhysLane::BoolU8);
                assert_eq!(params[6].lane, PhysLane::CharU32);
            }
            _ => panic!("pure form expected"),
        }
    }

    /// The empirical probe shape (int, double, int): MSVC 14.44 puts the
    /// double in XMM1 (position 2!) and Cranelift does the same — pin it.
    #[test]
    fn positional_fp_register_index_is_argument_position() {
        let mut facts = facts_8lanes();
        facts.functions[0].form = crate::facts::FnForm::Pure {
            params: vec![LaneKind::I32, LaneKind::F64, LaneKind::I32],
            ret: LaneKind::I64,
        };
        let plan = PhysicalPlan::project(&facts).unwrap();
        match &plan.function("t937_mix").unwrap().form {
            PhysForm::Pure { params, .. } => {
                assert!(matches!(params[0].slot, SlotClass::GpRegister { index: 0 }));
                assert!(matches!(params[1].slot, SlotClass::FpRegister { index: 1 }));
                assert!(matches!(params[2].slot, SlotClass::GpRegister { index: 2 }));
            }
            _ => panic!("pure form expected"),
        }
    }

    #[test]
    fn fallible_ctx_and_out_pointer_lanes() {
        let mut facts = facts_8lanes();
        facts.functions[0].form = crate::facts::FnForm::Fallible {
            ctx: Some(true),
            params: vec![LaneKind::I64, LaneKind::I64],
            outs: vec![LaneKind::I64],
        };
        facts.functions[0].error_model = Some(crate::facts::ErrorModel {
            kind: "status-i32".into(),
            codes: vec![(-1, "div-by-zero".into())],
        });
        let plan = PhysicalPlan::project(&facts).unwrap();
        match &plan.function("t937_mix").unwrap().form {
            PhysForm::Fallible { ctx, params, outs } => {
                assert_eq!(*ctx, Some(true));
                // ctx + 2 params + 1 out ptr = 4 GP lanes -> all in regs.
                assert_eq!(params.len(), 4);
                assert_eq!(params[0].name, "ctx");
                assert_eq!(params[3].name, "out0_ptr");
                assert_eq!(outs, &vec![PhysLane::I64]);
            }
            _ => panic!("fallible form expected"),
        }
    }

    #[test]
    fn facts_digest_change_changes_plan_projection() {
        let p1 = PhysicalPlan::project(&facts_8lanes()).unwrap();
        let mut f2 = facts_8lanes();
        if let crate::facts::FnForm::Pure { params, .. } = &mut f2.functions[0].form {
            params[0] = LaneKind::U32;
        }
        let p2 = PhysicalPlan::project(&f2).unwrap();
        assert_ne!(p1.digest().unwrap(), p2.digest().unwrap());
        // Source identity alone also changes the facts, not the plan:
        // the plan is a pure function of the FORMS (domain constants
        // frozen). Same forms -> same plan regardless of source kind.
        let mut f3 = facts_8lanes();
        f3.source = FactsSource::CheckedSource {
            module: "m".into(),
            source_sha256: "x".into(),
        };
        f3.effects.push("pure:t937_mix".into());
        let p3 = PhysicalPlan::project(&f3).unwrap();
        assert_eq!(p1.digest().unwrap(), p3.digest().unwrap());
    }

    #[test]
    fn wrong_domain_constants_rejected() {
        let mut plan = PhysicalPlan::project(&facts_8lanes()).unwrap();
        plan.target = "aarch64-linux".into();
        assert!(plan.validate().unwrap_err().contains("target"));
    }

    #[test]
    fn decoded_plan_checks_stack_fp_context_and_out_edges_without_projecting() {
        let facts = facts_8lanes();
        let plan = PhysicalPlan::project(&facts).unwrap();
        crate::work::with_observer(
            std::sync::Arc::new(|stage, _| panic!("validation produced {stage}")),
            || plan.validate_against(&facts),
        )
        .unwrap();
        for axis in 0..3 {
            let mut forged = plan.clone();
            let PhysForm::Pure { params, .. } = &mut forged.functions[0].form else {
                unreachable!()
            };
            match axis {
                0 => params[4].slot = SlotClass::FpRegister { index: 0 },
                1 => params[4].stack_bank = Some(SlotBank::Gp),
                _ => params[7].slot = SlotClass::Stack { index: 2 },
            }
            assert!(forged.validate_against(&facts).is_err());
        }
        let mut facts = facts;
        facts.functions[0].form = FnForm::Fallible {
            ctx: Some(false),
            params: vec![LaneKind::Bool; 5],
            outs: vec![LaneKind::I32],
        };
        facts.functions[0].error_model = Some(crate::ErrorModel {
            kind: "status-i32".into(),
            codes: vec![(-1, "failure".into())],
        });
        let plan = PhysicalPlan::project(&facts).unwrap();
        plan.validate_against(&facts).unwrap();
        for axis in 0..5 {
            let mut forged = plan.clone();
            let PhysForm::Fallible { ctx, params, outs } = &mut forged.functions[0].form else {
                unreachable!()
            };
            match axis {
                0 => *ctx = Some(true),
                1 => params[0].lane = PhysLane::BoolU8,
                2 => params[6].name = "p5".into(),
                3 => outs[0] = PhysLane::BoolU8,
                _ => {
                    params.remove(0);
                }
            }
            assert!(forged.validate_against(&facts).is_err(), "axis {axis}");
        }
    }
}

/// Test-only helpers shared across modules (kept out of the public API).
#[doc(hidden)]
pub mod tests_support {
    use super::*;
    use crate::facts::{ErrorModel, FactsSource, FnFact, FnForm, InterfaceFacts, FACTS_FORMAT};

    pub fn checksum_facts() -> InterfaceFacts {
        InterfaceFacts {
            format: FACTS_FORMAT.into(),
            logical: "typed/t937".into(),
            content_version: "v".into(),
            source: FactsSource::Typed { id: "t937".into() },
            functions: vec![FnFact {
                symbol: "t937_mix".into(),
                form: FnForm::Pure {
                    params: vec![
                        LaneKind::I32,
                        LaneKind::U32,
                        LaneKind::I64,
                        LaneKind::U64,
                        LaneKind::F64,
                        LaneKind::Bool,
                        LaneKind::Char,
                        LaneKind::I64,
                    ],
                    ret: LaneKind::I64,
                },
                error_model: None,
                semantic_ref: None,
                resource: None,
            }],
            effects: vec![],
            capabilities: vec![],
        }
    }

    #[allow(dead_code)]
    pub fn div_error_model() -> ErrorModel {
        ErrorModel {
            kind: "status-i32".into(),
            codes: vec![(-1, "div-by-zero".into())],
        }
    }
}
