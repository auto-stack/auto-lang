//! PLAN-924 (`ac/mir-native-closures-resource-v1`): the parent-side
//! INDEPENDENT oracle for the native closure facts — the frozen K02/K05
//! exit values, the per-seam injection taxonomy extended with S9 (the
//! closure environment node), the full cleanup baseline after a trap, and
//! R05's discriminating env-lifecycle event shape (the closure environment
//! is OWNED inside the creator's frame, still LIVE after the creator's
//! frame leaves, and freed exactly at the final sweep).
//!
//! The request carries only the operation/seam/index — never the expected
//! outcome. THIS module strictly decodes the stderr/MN-EVT facts and
//! judges the frozen per-case table; the worker's own words are never
//! evidence.

use super::hex_stream;
use auto_test_case::compare::Mismatch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosuresFacts {
    /// Ordered (seam, nth) pairs, one per injected failure.
    pub injects: Vec<(u32, u64)>,
    /// The trap code, if the run ended in a language trap.
    pub trap: Option<u32>,
    /// The abort sweep retired every owner: live=0 AND bytes=0 AND
    /// invalid=0 AND arena_live=0 on stderr.
    pub cleanup: bool,
    /// R05: the line index of the first closure-env OWN event, if any.
    pub closure_own: Option<usize>,
    /// R05: the line index of the first frame-LEAVE event AFTER that own
    /// (the creator exited while the env was alive).
    pub creator_exit_after_own: Option<usize>,
    /// R05: closure-related FREE events observed.
    pub frees: usize,
}

fn bad(msg: impl Into<String>) -> String {
    format!("native closures admission: {}", msg.into())
}

/// Strict decode of the fact lines inside one observed stderr TEXT.
pub fn decode_facts(text: &str) -> Result<ClosuresFacts, String> {
    let mut injects: Vec<(u32, u64)> = Vec::new();
    let mut pending_seam: Option<u32> = None;
    let mut trap: Option<u32> = None;
    let mut live0 = false;
    let mut bytes0 = false;
    let mut invalid0 = false;
    let mut arena0 = false;
    let mut closure_own: Option<usize> = None;
    let mut creator_exit: Option<usize> = None;
    let mut frees = 0usize;
    for (index, line) in text.split_terminator('\n').enumerate() {
        if let Some(rest) = line.strip_prefix("MN-COLL-INJECT seam=") {
            let seam: u32 = rest
                .parse()
                .map_err(|_| bad("malformed injected seam"))?;
            if rest != seam.to_string() || !(0..=9).contains(&seam) {
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
        // MN-EVT channel (the closures recipe implies observation): the
        // closure environment's own/disown/free lifecycle and the frame
        // activations.
        if line.contains("act=own kind=closure") && closure_own.is_none() {
            closure_own = Some(index);
            continue;
        }
        if line.contains("act=frame-leave") && closure_own.is_some() && creator_exit.is_none() {
            creator_exit = Some(index);
            continue;
        }
        if line.starts_with("MN-EVT ") && line.contains("act=free") {
            frees += 1;
            continue;
        }
    }
    if let Some(seam) = pending_seam {
        return Err(bad(format!("seam {seam} without a matching nth")));
    }
    Ok(ClosuresFacts {
        injects,
        trap,
        cleanup: live0 && bytes0 && invalid0 && arena0,
        closure_own,
        creator_exit_after_own: creator_exit,
        frees,
    })
}

pub fn decode_facts_hex(stderr_hex: &str) -> Result<ClosuresFacts, String> {
    let bytes = hex_stream(stderr_hex).map_err(|e| bad(e))?;
    let text = String::from_utf8(bytes).map_err(|_| bad("stderr is not utf-8"))?;
    decode_facts(&text)
}

/// The frozen parent table — independent of anything the worker reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosuresOracle {
    pub exit: Option<i32>,
    pub trap: Option<u32>,
    pub injects: Vec<(u32, u64)>,
    pub cleanup: bool,
    /// R05's env-lifecycle shape is required (own + creator-exit + frees).
    pub env_lifecycle: bool,
}

macro_rules! oracle {
    ($key:expr, $exit:expr, $trap:expr, $injects:expr, $cleanup:expr, $env:expr) => {
        (
            $key,
            ClosuresOracle {
                exit: $exit,
                trap: $trap,
                injects: $injects,
                cleanup: $cleanup,
                env_lifecycle: $env,
            },
        )
    };
}

pub fn oracle_for(key: &str) -> Option<&'static ClosuresOracle> {
    use std::sync::OnceLock;
    static TABLE: OnceLock<Vec<(&'static str, ClosuresOracle)>> = OnceLock::new();
    TABLE
        .get_or_init(|| {
            vec![
                // The frozen corpus K02: factory moves "hello" into the env,
                // the creator returns, main's indirect call adds 5 + 37.
                oracle!("ace.m1.r02r05.k02/base", Some(42), None, vec![], true, true),
                // The env-node seam (S9): K02 stages exactly ONE env node —
                // nth=1 traps INJECTED with the full cleanup baseline; the
                // N+1 control runs the true 42.
                oracle!(
                    "ace.m1.r02r05.k02/fault-s9",
                    None,
                    Some(25),
                    vec![(9, 1)],
                    true,
                    false
                ),
                oracle!("ace.m1.r02r05.k02/fault-s9-n2", Some(42), None, vec![], true, true),
                // The node seam (S1): the captured literal's fresh node.
                oracle!(
                    "ace.m1.r02r05.k02/fault-s1",
                    None,
                    Some(25),
                    vec![(1, 1)],
                    true,
                    false
                ),
                // The frame seam (S4): the delivered frame-open shim maps an
                // injected failure to its null path (trap 22) while keeping
                // the INJECT fact — the frozen 923 taxonomy.
                oracle!(
                    "ace.m1.r02r05.k02/fault-s4",
                    None,
                    Some(22),
                    vec![(4, 1)],
                    true,
                    false
                ),
                // The frozen corpus K05 (explicit deep clone): the original
                // AND the clone stay callable; TWO env nodes stage (nth=1
                // and nth=2 both trap; nth=3 is the N+1 control).
                oracle!("ace.m1.r02r05.k05/base", Some(42), None, vec![], true, true),
                oracle!(
                    "ace.m1.r02r05.k05/fault-s9a",
                    None,
                    Some(25),
                    vec![(9, 1)],
                    true,
                    false
                ),
                oracle!(
                    "ace.m1.r02r05.k05/fault-s9b",
                    None,
                    Some(25),
                    vec![(9, 2)],
                    true,
                    false
                ),
                oracle!("ace.m1.r02r05.k05/fault-s9-n3", Some(42), None, vec![], true, true),
            ]
        })
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, o)| o)
}

pub fn judge_native(
    oracle: &ClosuresOracle,
    exit: Option<i32>,
    facts: &ClosuresFacts,
    mismatches: &mut Vec<Mismatch>,
) {
    fn push(
        mismatches: &mut Vec<Mismatch>,
        kind: &'static str,
        expected: String,
        actual: String,
    ) {
        mismatches.push(Mismatch {
            observation: "native-closures".into(),
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
            "live=0 bytes=0 invalid=0 arena_live=0 after the run".into(),
            "some owner/env survived the sweep".into(),
        );
    }
    if oracle.env_lifecycle {
        // R05: the env was owned inside the creator's frame, the creator
        // left while it was alive, frees happened, and nothing survived.
        if facts.closure_own.is_none() {
            push(
                mismatches,
                "env-lifecycle",
                "an act=own kind=closure event".into(),
                "no closure own event".into(),
            );
        }
        if facts.creator_exit_after_own.is_none() {
            push(
                mismatches,
                "env-lifecycle",
                "a frame-leave AFTER the closure own (creator exit with the env alive)".into(),
                "no creator exit after the own".into(),
            );
        }
        if facts.frees == 0 {
            push(
                mismatches,
                "env-lifecycle",
                "closure/text free events".into(),
                "no free events".into(),
            );
        }
    }
}
