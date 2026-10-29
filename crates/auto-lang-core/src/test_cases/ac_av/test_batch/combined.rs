//! C is a separate owned-record observation; original scalar checks remain A/B.
use super::*;
use serde_json::{json, Value};

const C_SCHEMA: &str = "avworker-test-combined/1";
const IDS: [&str; 3] = ["gov.d.arithmetic", "gov.d.unary", "gov.d.group"];

pub fn run(values: &[pilot::Bound], budget: Duration) -> Result<Value, String> {
    run_with_fault(values, budget, None)
}

#[doc(hidden)]
pub fn run_with_fault(
    values: &[pilot::Bound],
    budget: Duration,
    fault: Option<&str>,
) -> Result<Value, String> {
    let started = Instant::now();
    let mut parent_clock = parent_observations::Clock::new();
    if budget.is_zero()
        || budget > SESSION_WALL
        || values.len() != 3
        || values.iter().zip(IDS).any(|(b, id)| {
            b.case.spec.id != id || !matches!(b.oracle.completion, Completion::Success)
        })
    {
        return Err("combined requires original ordered D3 and wall in (0,60s]".into());
    }
    let deadline = started + budget;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../auto-lang/test/cases/governance-887/src");
    let mut source="type Results { arithmetic int; unary int; group int }\nfn main() Results {\n return Results(".to_owned();
    let mut origins = Vec::new();
    for (index, name) in ["arithmetic", "unary", "group"].iter().enumerate() {
        if index > 0 {
            source.push_str(", ");
        }
        let path = root.join(format!("{name}.at"));
        let bytes = auto_test_case::admission::read_bounded(
            &path,
            "combined expression",
            auto_test_case::admission::Limits::defaults().max_sidecar_bytes,
        )
        .map_err(|e| format!("{}: {e}", path.display()))?;
        let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
        let begin = source.len();
        let line = source.bytes().filter(|&b| b == b'\n').count() + 1;
        source.push_str(text);
        origins.push(json!({"path":path,"sha256":hex(&sha256(&bytes)),
            "source_byte_range":[begin,source.len()],"source_line_range":[line,line+text.bytes().filter(|&b|b==b'\n').count()],
            "original_scalar":values[index].origin}));
    }
    source.push_str(")\n}\n");
    let source_blake3 = blake3::hash(source.as_bytes()).to_hex().to_string();
    let nonce = hex(&sha256(
        format!("{}|{started:?}|{source_blake3}", std::process::id()).as_bytes(),
    ));
    parent_clock.lap("prepare_ms");
    let mut worker = Transport::start_command(&nonce, deadline, fault, "test-combined-serve")?;
    let observed = (|| -> Result<Value, String> {
        let mut clock = parent_observations::Clock::new();
        worker.send(
            json!({"schema":C_SCHEMA,"nonce":nonce,"source":source,
            "source_blake3":source_blake3,"ids":IDS}),
            deadline,
        )?;
        worker.writer.take();
        let response = worker
            .frame(deadline)?
            .ok_or("combined missing actual response/capability")?;
        if worker.frame(deadline)?.is_some() {
            return Err("extra combined output".into());
        }
        loop {
            worker.check(deadline)?;
            if let Some(status) = worker.child.try_wait().map_err(|e| e.to_string())? {
                if !status.success() {
                    return Err(format!("combined worker nonzero exit: {status}"));
                }
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        clock.lap("transport_ms");
        let allowed = [
            "schema",
            "nonce",
            "source_blake3",
            "profile",
            "worker_version",
            "metrics",
            "actuals",
            "executor_sequence_observation",
            "live_after_release",
            "worker_wall_ms",
        ];
        let object = response
            .as_object()
            .ok_or("combined response is not object")?;
        if object.len() != allowed.len()
            || object.keys().any(|k| !allowed.contains(&k.as_str()))
            || response["schema"] != C_SCHEMA
            || response["nonce"] != nonce
            || response["source_blake3"] != source_blake3
            || response["profile"] != EXPECTED_AV_OWNED_CORE_VERSION
            || response["worker_version"] != EXPECTED_AV_WORKER_VERSION
            || response["live_after_release"] != 0
            // Historic owned execution leaves the sequence field at zero.
            // It is not an instruction or invocation counter. Producer calls
            // are checked below at the actual compile/execute boundaries.
            || response["executor_sequence_observation"].as_u64().is_none()
            || !response["worker_wall_ms"].as_f64().is_some_and(|n|n.is_finite() && n>=0.0)
        {
            return Err(format!(
                "combined source/profile/identity/resource mismatch: {response}"
            ));
        }
        let stages = [
            "compile",
            "lower",
            "verify_mir",
            "encode",
            "verify_abc",
            "vm",
        ];
        let calls = response["metrics"]["calls"]
            .as_object()
            .ok_or("combined missing producer counts")?;
        let spans = response["metrics"]["spans_ms"]
            .as_array()
            .ok_or("combined missing stage spans")?;
        if response["metrics"].as_object().map(|m| m.len()) != Some(2)
            || calls.len() != stages.len()
            || stages.iter().any(|s| calls.get(*s) != Some(&json!(1)))
            || spans.len() != stages.len()
            || stages.iter().any(|s| {
                spans
                    .iter()
                    .filter(|v| {
                        v[0] == *s && v[1].as_f64().is_some_and(|n| n.is_finite() && n >= 0.0)
                    })
                    .count()
                    != 1
            })
        {
            return Err("combined producer call count/span mismatch".into());
        }
        let actuals = response["actuals"]
            .as_array()
            .ok_or("combined actuals missing")?;
        if actuals.len() != 3 {
            return Err("combined actual cardinality mismatch".into());
        }
        for (actual, id) in actuals.iter().zip(IDS) {
            if actual.as_object().map(|o| o.len()) != Some(3)
                || actual["id"] != id
                || actual["channel"] != "i32"
                || actual["value"]
                    .as_i64()
                    .and_then(|v| i32::try_from(v).ok())
                    .is_none()
            {
                return Err("combined actual identity/type mismatch".into());
            }
        }
        clock.lap("admission_compare_receipt_ms");
        Ok(response)
    })();
    let pid = worker.child.id();
    let worker_sha = worker.exe_sha.clone();
    {
        let _cleanup = parent_observations::Phase::start("cleanup_ms");
        worker.shutdown()?;
    }
    let response = observed?;
    let mut comparison_clock = parent_observations::Clock::new();
    let mut items = Vec::new();
    for (index, bound) in values.iter().enumerate() {
        let actual = response["actuals"][index]["value"]
            .as_i64()
            .ok_or("missing admitted actual")? as i32;
        // Reuse the ordinary parent's exact scalar expectation comparator.
        // This is an explicitly admitted C field, not an invented worker-v1 response.
        let step: StepOut = serde_json::from_value(json!({"op":"run","outcome":"success",
            "value":actual,"channel":"value"}))
        .map_err(|e| e.to_string())?;
        let mut mismatches = Vec::new();
        judge_expectation(RecipeFamily::Core, &step, &bound.oracle, &mut mismatches);
        items.push(json!({"case":bound.case.spec.id,"check":bound.check.id,
            "observation_profile":"C/owned-record-field","status":if mismatches.is_empty(){"PASS"}else{"FAIL"},
            "expected":bound.oracle.exit,"actual":actual,"origin":origins[index],
            "mismatches":mismatches.iter().map(|m|json!({"observation":m.observation,"expected":m.expected,"actual":m.actual})).collect::<Vec<_>>(),
            "replay_argv":["test-case-pilot","--mode","independent","--case",bound.case.spec.id]}));
    }
    comparison_clock.lap("admission_compare_receipt_ms");
    if Instant::now() >= deadline {
        return Err("combined deadline includes parent comparison/cleanup".into());
    }
    Ok(
        json!({"schema":"test-combined-pilot/1","mode":"combined","source":source,
        "source_blake3":source_blake3,"worker_sha256":worker_sha,"worker_pid":pid,"worker_starts":1,
        "wall_ms":started.elapsed().as_secs_f64()*1000.0,"items":items,"observation":response,"cleanup_complete":true}),
    )
}
