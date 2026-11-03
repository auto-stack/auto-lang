//! PLAN-923 (`ac/mir-native-resource-guards-v1`): the parent-side
//! INDEPENDENT oracle for the native per-seam fault-injection facts
//! (`MN-COLL-INJECT seam=/nth=` on the PE's stderr) and the checked
//! List-place machine chain (STALE trap 27 refused BEFORE any buffer
//! access; every owner/token retired by the abort sweep).
//!
//! The request carries only the operation/seam/index/budget — never the
//! expected outcome. THIS module strictly decodes the stderr facts,
//! checks the frozen generic invariants (seam/nth taxonomy, the trap
//! mapping, the zero-live/zero-bytes cleanup after a trap) and judges the
//! frozen per-case table. The native and av channels are separate frozen
//! vocabularies (native trap 27 = `av.trap.stale-place`; native 25 =
//! INJECTED, distinct from a real RESOURCE refusal on the same seam).

use super::hex_stream;
use auto_test_case::compare::Mismatch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardsFacts {
    /// Ordered (seam, nth) pairs, one per injected failure.
    pub injects: Vec<(u32, u64)>,
    /// The trap code (21..=27), if the run ended in a language trap.
    pub trap: Option<u32>,
    /// The abort sweep retired every owner/token: live=0 AND bytes=0 AND
    /// invalid=0 AND arena_live=0 on stderr.
    pub cleanup: bool,
}

fn bad(msg: impl Into<String>) -> String {
    format!("native guards admission: {}", msg.into())
}

/// Strict decode of the fact lines inside one observed stderr TEXT.
/// `text` is the already-unhexed stderr.
pub fn decode_facts(text: &str) -> Result<GuardsFacts, String> {
    let mut injects: Vec<(u32, u64)> = Vec::new();
    let mut pending_seam: Option<u32> = None;
    let mut trap: Option<u32> = None;
    let mut live0 = false;
    let mut bytes0 = false;
    let mut invalid0 = false;
    let mut arena0 = false;
    for line in text.split_terminator('\n') {
        if let Some(rest) = line.strip_prefix("MN-COLL-INJECT seam=") {
            let seam: u32 = rest
                .parse()
                .map_err(|_| bad("malformed injected seam"))?;
            if rest != seam.to_string() || !(0..=8).contains(&seam) {
                return Err(bad("unknown injected seam"));
            }
            if pending_seam.replace(seam).is_some() {
                return Err(bad("duplicate injected seam without nth"));
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("MN-COLL-INJECT nth=") {
            let nth: u64 = rest
                .parse()
                .map_err(|_| bad("malformed injected nth"))?;
            if rest != nth.to_string() {
                return Err(bad("malformed injected nth"));
            }
            let seam = pending_seam.take().ok_or_else(|| bad("nth without seam"))?;
            injects.push((seam, nth));
            continue;
        }
        if let Some(rest) = line.strip_prefix("MN-COLL-TRAP code=") {
            let code: u32 = rest
                .parse()
                .map_err(|_| bad("malformed native trap reason"))?;
            if rest != code.to_string() || !matches!(code, 21..=27 | 30 | 31) {
                return Err(bad("unknown native trap reason"));
            }
            if trap.replace(code).is_some() {
                return Err(bad("duplicate native trap"));
            }
            continue;
        }
        if line == "MN-COLL live=0" {
            live0 = true;
            continue;
        }
        if line == "MN-COLL bytes=0" {
            bytes0 = true;
            continue;
        }
        if line == "MN-COLL invalid=0" {
            invalid0 = true;
            continue;
        }
        if line == "MN-COLL arena_live=0" {
            arena0 = true;
            continue;
        }
    }
    if let Some(seam) = pending_seam {
        return Err(bad(format!("seam {seam} without a matching nth")));
    }
    Ok(GuardsFacts {
        injects,
        trap,
        cleanup: live0 && bytes0 && invalid0 && arena0,
    })
}

pub fn decode_facts_hex(stderr_hex: &str) -> Result<GuardsFacts, String> {
    let bytes = hex_stream(stderr_hex).map_err(|e| bad(e))?;
    let text = String::from_utf8(bytes).map_err(|_| bad("stderr is not utf-8"))?;
    decode_facts(&text)
}

/// The frozen parent table — independent of anything the worker reports.
/// Keys are `case-id/check-id` from the AutoDown case file; the request
/// never carries any of these expectations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardsOracle {
    pub exit: Option<i32>,
    pub trap: Option<u32>,
    pub injects: Vec<(u32, u64)>,
    pub cleanup: bool,
}

macro_rules! oracle {
    ($key:expr, $exit:expr, $trap:expr, $injects:expr, $cleanup:expr) => {
        ($key, GuardsOracle { exit: $exit, trap: $trap, injects: $injects, cleanup: $cleanup })
    };
}

pub fn oracle_for(key: &str) -> Option<&'static GuardsOracle> {
    use std::sync::OnceLock;
    static TABLE: OnceLock<Vec<(&'static str, GuardsOracle)>> = OnceLock::new();
    TABLE
        .get_or_init(|| {
            vec![
                // Base positive controls: an un-injected run returns the
                // frozen exit and the sweep retires every owner (the
                // runtime summary lines are the cleanup face).
                oracle!("ace.m1.r03.l04/base", Some(42), None, vec![], true),
                oracle!("ace.m1.r03.grow/base", Some(24), None, vec![], true),
                oracle!("ace.m1.r03.lstr/base", Some(51), None, vec![], true),
                // Per-seam fault sweep over real source PEs: the injected
                // failure traps INJECTED (25) with the named seam/nth fact
                // and a fully cleaned sweep (live/bytes/invalid/arena 0).
                oracle!("ace.m1.r03.l04/fault-s1", None, Some(25), vec![(1, 1)], true),
                oracle!("ace.m1.r03.l04/fault-s2", None, Some(25), vec![(2, 1)], true),
                oracle!("ace.m1.r03.l04/fault-s4", None, Some(22), vec![(4, 1)], true),
                oracle!("ace.m1.r03.grow/fault-s3", None, Some(25), vec![(3, 1)], true),
                // The text seams surface through the embedded text
                // runtime's own channels (ERR_ENV_OOM -> trap 22); they do
                // not emit a MN-COLL-INJECT pair of this worker's channel.
                oracle!("ace.m1.r03.lstr/fault-s6", None, Some(22), vec![], true),
                oracle!("ace.m1.r03.lstr/fault-s7", None, Some(22), vec![], true),
            ]
        })
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, o)| o)
}

pub fn judge_native(
    oracle: &GuardsOracle,
    exit: Option<i32>,
    facts: &GuardsFacts,
    mismatches: &mut Vec<Mismatch>,
) {
    fn push(
        mismatches: &mut Vec<Mismatch>,
        kind: &'static str,
        expected: String,
        actual: String,
    ) {
        mismatches.push(Mismatch {
            observation: "native-guards".into(),
            kind,
            expected,
            actual,
            first_diff: None,
        });
    }
    if let Some(want) = oracle.exit {
        if exit != Some(want) {
            push(mismatches, "exit", format!("PE exit {want}"), format!("{exit:?}"));
        }
    } else if exit.is_none() {
        push(mismatches, "exit", "a real PE exit code".into(), "none".into());
    }
    if let Some(want) = oracle.trap {
        if facts.trap != Some(want) {
            push(
                mismatches,
                "trap",
                format!("MN-COLL-TRAP code={want}"),
                format!("{:?}", facts.trap),
            );
        }
    } else if facts.trap.is_some() {
        push(
            mismatches,
            "trap",
            "no language trap".into(),
            format!("trap {:?}", facts.trap),
        );
    }
    if facts.injects != oracle.injects {
        push(
            mismatches,
            "inject",
            format!("{:?}", oracle.injects),
            format!("{:?}", facts.injects),
        );
    }
    if oracle.cleanup && !facts.cleanup {
        push(
            mismatches,
            "cleanup",
            "live=0 bytes=0 invalid=0 arena_live=0 after the trap".into(),
            "some owner/token survived the sweep".into(),
        );
    }
}
