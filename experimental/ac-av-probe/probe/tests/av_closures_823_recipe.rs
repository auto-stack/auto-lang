//! PLAN-823 T-06 recipe tests (`av_closures_823` family): the frozen
//! first-case face against the HAND-AUDITED truths table, the judge's
//! wrong-result/wrong-bits/missing-case negative controls (V-10: each
//! lie turns the verdict non-zero — the oracle detects, never trusts),
//! and the managed-partition peak observations. The formal parent-side
//! corpus wiring lands with the merge-stage registration; this file pins
//! the WORKER-side truths those runs depend on.

use auto_av_probe::closures_fixture::{request_closures, run_closures_fixture};

/// The hand-audited truths of the first-case face (independent of the
/// implementation: computed by hand from §5.2's fixture — env text
/// "auto" slen 4, list len 2 after the body's push, bonus 5 → 2*10+4+5
/// = 29 = 0x1d). Five buffers allocate and all free; the list's alloc
/// event carries its ELEMENT COUNT at allocation time (the empty list
/// mints with capacity reserved): [4 "auto", 4 "auto"(note field), 0
/// list, 4 capture clone, 6 "pushed"].
const EXPECTED_RESULT_KIND: &str = "i32";
const EXPECTED_RESULT_BITS: &str = "1d";
const EXPECTED_ALLOC_BYTES: [u64; 5] = [4, 4, 0, 4, 6];
const EXPECTED_ALLOC_COUNT: usize = 5;

/// The parent-side judge: compares one face against the frozen truths.
/// Returns Err on ANY axis — the caller surfaces it as a non-zero
/// verdict (never a skip, never a partial pass).
fn judge_first_case(face: &serde_json::Value) -> Result<(), String> {
    let kind = face["result"]["kind"].as_str().unwrap_or("?");
    if kind != EXPECTED_RESULT_KIND {
        return Err(format!("result kind {kind} != {EXPECTED_RESULT_KIND}"));
    }
    let bits = face["result"]["bits"].as_str().unwrap_or("?");
    if bits != EXPECTED_RESULT_BITS {
        return Err(format!(
            "result bits {bits} != {EXPECTED_RESULT_BITS} (wrong result)"
        ));
    }
    if face["live_after_release"].as_u64() != Some(0) {
        return Err("live_after_release != 0 (leaked env payloads)".to_string());
    }
    let allocs = face["allocs"]
        .as_array()
        .ok_or_else(|| "allocs missing (missing case)".to_string())?;
    if allocs.len() != EXPECTED_ALLOC_COUNT {
        return Err(format!(
            "{} allocs != {EXPECTED_ALLOC_COUNT} (missing case)",
            allocs.len()
        ));
    }
    for (i, a) in allocs.iter().enumerate() {
        let b = a["bytes"].as_u64().unwrap_or(u64::MAX);
        if b != EXPECTED_ALLOC_BYTES[i] {
            return Err(format!(
                "alloc #{i} bytes {b} != {}",
                EXPECTED_ALLOC_BYTES[i]
            ));
        }
    }
    let frees = face["frees"]
        .as_array()
        .ok_or_else(|| "frees missing (missing case)".to_string())?;
    if frees.len() != EXPECTED_ALLOC_COUNT {
        return Err(format!(
            "{} frees != {EXPECTED_ALLOC_COUNT} (cleanup shortfall)",
            frees.len()
        ));
    }
    Ok(())
}

#[test]
fn av_closures_823_recipe_first_case_face_matches_hand_audit() {
    let face = run_closures_fixture("first-case", None, "recipe-test")
        .expect("the first case runs the whole path");
    judge_first_case(&face).expect("the real face passes the frozen judge");
}

#[test]
fn av_closures_823_recipe_judge_detects_lies() {
    // The V-10 negative controls: EVERY wrong-face shape the judge must
    // catch (a lying worker cannot pass by skipping or partially
    // reporting).
    let truth = run_closures_fixture("first-case", None, "recipe-test").expect("runs");

    // (a) wrong result bits.
    let mut lie = truth.clone();
    lie["result"]["bits"] = serde_json::json!("2a");
    assert!(judge_first_case(&lie).is_err(), "wrong bits detected");

    // (b) leaked env (live != 0).
    let mut leak = truth.clone();
    leak["live_after_release"] = serde_json::json!(1);
    assert!(judge_first_case(&leak).is_err(), "leak detected");

    // (c) missing case (allocs list short).
    let mut missing = truth.clone();
    missing["allocs"]
        .as_array_mut()
        .unwrap()
        .truncate(EXPECTED_ALLOC_COUNT - 1);
    assert!(judge_first_case(&missing).is_err(), "missing case detected");

    // (d) wrong byte shape (a 4-byte buffer reported as 5).
    let mut shape = truth.clone();
    shape["allocs"][0]["bytes"] = serde_json::json!(5);
    assert!(judge_first_case(&shape).is_err(), "byte shape detected");

    // (e) frees shortfall (cleanup skipped).
    let mut frees = truth.clone();
    frees["frees"]
        .as_array_mut()
        .unwrap()
        .truncate(EXPECTED_ALLOC_COUNT - 2);
    assert!(
        judge_first_case(&frees).is_err(),
        "cleanup shortfall detected"
    );

    // (f) absent sections.
    let mut gone = truth.clone();
    gone["allocs"].take();
    assert!(judge_first_case(&gone).is_err(), "absent section detected");
}

#[test]
fn av_closures_823_recipe_managed_peaks_recorded() {
    // The managed-partition peak observations ride the request outcome —
    // real numbers within the frozen envelope (T-06's 受管峰值 axis;
    // the process-level private-bytes axis is the merge-stage gate's).
    let r = request_closures(None, "peaks-probe", "first-case").expect("runs");
    assert!(r.peak_hot_bytes > 0, "hot partition observed");
    assert!(r.peak_staging_bytes >= r.peak_hot_bytes, "staging ≥ hot");
    assert!(
        r.peak_exec_slot_bytes > 0,
        "executor slot high-water observed"
    );
    assert!(
        r.peak_exec_slot_bytes as usize <= auto_av_core::contract::MAX_AV_SLOT_BYTES,
        "the executor peak stays under the 8 MiB value-data line"
    );
}
