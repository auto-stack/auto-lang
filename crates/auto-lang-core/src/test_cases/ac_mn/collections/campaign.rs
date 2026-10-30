//! Formal PLAN-889 campaign through the landed parent transport/admission.
//! Evidence writes follow each closed measurement window; neither compiler
//! work nor runtime preparation is done by this observer outside requests.
use super::super::{
    b64, call_worker, hex, sha256, worker_binary_for, RoundsWorker, COLLECTIONS_RECIPE,
    EXPECTED_MN_WORKER_VERSION, MN_WORKER_PROTOCOL,
};
use super::{admit_body, admit_identities, admit_raw};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    io::Write,
    path::Path,
    time::{Duration, Instant},
};
const SAMPLES: usize = 30;
const CHURN: usize = 512;
fn request(
    source: &str,
    store: &Path,
    work: &Path,
    mode: Option<&str>,
    manifest: Option<&str>,
    nonce: &str,
) -> Value {
    let mut request = json!({"recipe":COLLECTIONS_RECIPE,"nonce":nonce,"source_b64":b64(source.as_bytes()),"entry":"d_main","caps":[],"store_root":store,"workdir":work});
    if let Some(mode) = mode {
        request["mode"] = json!(mode);
    }
    if let Some(id) = manifest {
        request["manifest_id"] = json!(id);
    }
    request["request_hash"] = json!(hex(&sha256(request.to_string().as_bytes())));
    request
}
fn admit(line: &str, request: &Value, expected: i32) -> Result<Value, String> {
    let response: Value =
        serde_json::from_str(line).map_err(|e| format!("collections response: {e}"))?;
    if response["schema"] != MN_WORKER_PROTOCOL
        || response["nonce"] != request["nonce"]
        || (request.get("request_hash").is_some()
            && response["request_hash"] != request["request_hash"])
    {
        return Err("collections campaign echo mismatch".into());
    }
    let ok = response.get("ok").ok_or_else(|| {
        format!(
            "collections worker failure: {}",
            response.get("error").unwrap_or(&Value::Null)
        )
    })?;
    if ok["recipe"] != COLLECTIONS_RECIPE
        || ok["worker_version"] != EXPECTED_MN_WORKER_VERSION
        || ok["mn_abi"] != "msvc-x64/priv-i32-bool-v1"
    {
        return Err("collections worker identity mismatch".into());
    }
    admit_raw(ok)?;
    if ok["exit_code"].as_i64() != Some(expected as i64) {
        return Err(format!(
            "independent oracle expected {expected}, actual {}",
            ok["exit_code"]
        ));
    }
    if ok["manifest_fp"].as_str().is_none_or(str::is_empty) {
        return Err("missing real manifest identity".into());
    }
    Ok(ok.clone())
}
fn counter(ok: &Value, key: &str) -> u64 {
    ok["counters"][key].as_u64().unwrap_or(0)
}
fn source_at(base: &str, revision: u32) -> Result<String, String> {
    let from = "seed + 41 + 0";
    if base.matches(from).count() != 1 {
        return Err("frozen edit bit is not unique".into());
    }
    Ok(base.replacen(from, &format!("seed + 41 + {revision}"), 1))
}
struct Evidence {
    file: std::fs::File,
    paths: BTreeMap<String, Vec<f64>>,
    binding: Option<Value>,
}
impl Evidence {
    fn record(&mut self, event: &Value) -> Result<(), String> {
        serde_json::to_writer(&mut self.file, event).map_err(|e| e.to_string())?;
        self.file
            .write_all(b"\n")
            .and_then(|_| self.file.flush())
            .map_err(|e| e.to_string())
    }
    fn sample(
        &mut self,
        path: &str,
        index: usize,
        wall: f64,
        result: Result<Value, String>,
    ) -> Result<Value, String> {
        let event = match &result {
            Ok(ok) => json!({"kind":"sample","path":path,"index":index,"wall_ms":wall,"ok":ok}),
            Err(error) => {
                json!({"kind":"failed-sample","path":path,"index":index,"wall_ms":wall,"error":error})
            }
        };
        self.record(&event)?;
        let ok = result?;
        if let Some(binding) = &self.binding {
            if ok["collections_codegen"] != binding["codegen"]
                || ok["runtime_identity"] != binding["runtime"]
                || ok["link_identity"] != binding["link"]
            {
                return Err("producer/runtime binding drift during campaign".into());
            }
        } else {
            self.binding = Some(
                json!({"codegen":ok["collections_codegen"],"runtime":ok["runtime_identity"],"link":ok["link_identity"]}),
            );
        }
        if path != "warmup" && path != "churn" {
            self.paths.entry(path.into()).or_default().push(wall);
        }
        Ok(ok)
    }
}
/// Exactly the frozen 116-function input, five paths x30, and one retained
/// session with512 new real revisions. Errors retain their actual attempts.
pub fn run(source: &str, output: &Path) -> Result<Value, String> {
    if source.len() != 345483
        || hex(&sha256(source.as_bytes()))
            != "df314cc77208bcc115f02b446b8c1012046e96b046784a1a0c6e2e2094a1715c"
    {
        return Err("889 frozen source identity mismatch".into());
    }
    std::fs::create_dir_all(output).map_err(|e| e.to_string())?;
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output.join("samples.jsonl"))
        .map_err(|e| e.to_string())?;
    let mut evidence = Evidence {
        file,
        paths: BTreeMap::new(),
        binding: None,
    };
    evidence.record(&json!({"kind":"contract","source_sha256":hex(&sha256(source.as_bytes())),"functions":116,"source_bytes":source.len(),"samples_per_path":SAMPLES,"churn":CHURN,"p90_ms":2000,"rank":"N-floor(N/10), one-based","started_unix_ms":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis(),"tool_path_hints":"shared parent, first preparation measured; tool contents rehashed on every request","fresh_stores":"independent and initially absent","host":std::env::consts::OS,"parallelism":std::thread::available_parallelism().map(|n|n.get()).unwrap_or(0)}))?;
    let outcome = (|| -> Result<Value, String> {
        // Each fresh sample gets a genuinely empty independent managed store.
        for index in 0..SAMPLES {
            let started = Instant::now();
            let req = request(
                &source.to_owned(),
                &output.join(format!("fresh-store-{index}")),
                &output.join(format!("fresh-{index}/work")),
                None,
                None,
                &format!("889-fresh-{index}"),
            );
            let result = call_worker(&req.to_string())
                .and_then(|line| admit(&line, &req, 1786))
                .and_then(|ok| {
                    if counter(&ok, "codegen_emit_units") != 116
                        || ok["parsed_functions"]
                            .as_object()
                            .is_none_or(|map| map.len() != 116)
                    {
                        return Err("fresh did not produce all116 actual bodies/units".into());
                    }
                    Ok(ok)
                });
            evidence.sample(
                "fresh",
                index,
                started.elapsed().as_secs_f64() * 1000.,
                result,
            )?;
            if index % 10 == 9 {
                println!("889 formal fresh {}/30", index + 1);
            }
        }
        let binary = worker_binary_for(COLLECTIONS_RECIPE)?;
        let binary_hash = hex(&sha256(&std::fs::read(&binary).map_err(|e| e.to_string())?));
        evidence.record(&json!({"kind":"binding","worker":binary,"worker_sha256":binary_hash,"producer":evidence.binding}))?;
        // A bounded persistent campaign needs more than the120s ordinary case
        // lifetime; per-request deadlines and the2000ms path gates are intact.
        struct RestoreWall(Option<std::ffi::OsString>);
        impl Drop for RestoreWall {
            fn drop(&mut self) {
                if let Some(value) = self.0.take() {
                    std::env::set_var("AC_MN_ROUNDS_WALL_SECS", value);
                } else {
                    std::env::remove_var("AC_MN_ROUNDS_WALL_SECS");
                }
            }
        }
        let _restore = RestoreWall(std::env::var_os("AC_MN_ROUNDS_WALL_SECS"));
        std::env::set_var("AC_MN_ROUNDS_WALL_SECS", "600");
        let store = output.join("resident/store");
        let work = output.join("resident/work");
        let started = Instant::now();
        let req = request(source, &store, &work, Some("rounds"), None, "889-warmup");
        let (mut worker, line) = RoundsWorker::start(&req.to_string())?;
        let warm = admit(&line, &req, 1786);
        evidence.sample("warmup", 0, started.elapsed().as_secs_f64() * 1000., warm)?;
        let mut current = 0u32;
        for path in ["hit", "body-edit", "historical-return", "churn"] {
            let count = if path == "churn" { CHURN } else { SAMPLES };
            for index in 0..count {
                let started = Instant::now();
                let next = match path {
                    "hit" => current,
                    "body-edit" => (index + 1) as u32,
                    "historical-return" => index as u32,
                    _ => 1000 + index as u32,
                };
                let nonce = format!("889-{path}-{index}");
                let req = if path == "hit" {
                    json!({"op":"run","nonce":nonce})
                } else {
                    // Parent source copy/literal search and serialization are
                    // inside the measured interval, before enqueue.
                    let old = source_at(source, current)?;
                    let from = format!("seed + 41 + {current}");
                    let to = format!("seed + 41 + {next}");
                    let new = old.replacen(&from, &to, 1);
                    if new != source_at(source, next)? {
                        return Err("parent edit materialized incorrectly".into());
                    }
                    json!({"op":"edit-source","from":from,"to":to,"nonce":nonce})
                };
                let result = worker
                    .round(&req.to_string())
                    .and_then(|line| admit(&line, &req, 1786 + next as i32))
                    .and_then(|ok| {
                        if path == "hit" {
                            admit_identities(&ok, None)?;
                            if counter(&ok, "codegen_emit_units") != 0
                                || ok["bodies"].as_object().is_none_or(|map| !map.is_empty())
                                || ok["parsed_functions"]
                                    .as_object()
                                    .is_none_or(|map| !map.is_empty())
                            {
                                return Err("resident hit performed producer work".into());
                            }
                        } else {
                            admit_body(&ok)?;
                            if ok["changed_fn"] != "collections_helper"
                                || ok["identities_before"]
                                    .as_object()
                                    .is_none_or(|map| map.len() != 116)
                            {
                                return Err("changed the wrong actual producer set".into());
                            }
                            let expected = if path == "historical-return" { 0 } else { 1 };
                            if counter(&ok, "codegen_emit_units") != expected {
                                return Err(format!("{path} unexpected native production"));
                            }
                        }
                        Ok(ok)
                    });
                evidence.sample(path, index, started.elapsed().as_secs_f64() * 1000., result)?;
                current = next;
                if index % 32 == 31 || index + 1 == count {
                    println!("889 formal {path} {}/{count}", index + 1);
                }
            }
        }
        let last_source = source_at(source, current)?;
        let last_id = {
            let req = json!({"op":"run","nonce":"889-before-close"});
            let line = worker.round(&req.to_string())?;
            let ok = admit(&line, &req, 1786 + current as i32)?;
            evidence.record(&json!({"kind":"before-close","ok":ok}))?;
            ok["manifest_id"]
                .as_str()
                .ok_or("missing current persistent closure")?
                .to_owned()
        };
        let close = json!({"op":"close","nonce":"889-close"});
        let line = worker.round(&close.to_string())?;
        let close: Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
        evidence.record(&json!({"kind":"close","raw":close}))?;
        if close["nonce"] != "889-close"
            || close["ok"]["op"] != "close"
            || close["ok"]["allocation_meter"]["current"][0].as_u64() != Some(0)
        {
            return Err("retained MIR allocations survived session close".into());
        }
        if !worker.reap_until_deadline()?.success() {
            return Err("collections worker did not close successfully".into());
        }
        let (over, tail) = worker
            .stderr_done
            .recv_timeout(Duration::from_secs(2))
            .map_err(|e| e.to_string())?;
        if over {
            return Err(format!("collections stderr cap: {} bytes", tail.len()));
        }
        worker.teardown();
        // Each restart is a distinct process, with unit code generation
        // forbidden by the consumer. Source + coordinate + runtime closure is
        // read and deeply re-admitted from persisted untrusted pages.
        for index in 0..SAMPLES {
            let started = Instant::now();
            let req = request(
                &last_source.to_owned(),
                &store,
                &output.join(format!("restart-{index}")),
                None,
                Some(&last_id),
                &format!("889-restart-{index}"),
            );
            let result = call_worker(&req.to_string())
                .and_then(|line| admit(&line, &req, 1786 + current as i32))
                .and_then(|ok| {
                    if counter(&ok, "restored_mir_units_readmitted") != 116
                        || counter(&ok, "restored_source_coordinates") != 116
                        || counter(&ok, "codegen_emit_units") != 0
                        || ok.get("parsed_functions").is_some()
                    {
                        return Err(
                            "restart used frontend/codegen or omitted actual restore admission"
                                .into(),
                        );
                    }
                    Ok(ok)
                });
            evidence.sample(
                "restart",
                index,
                started.elapsed().as_secs_f64() * 1000.,
                result,
            )?;
        }
        if hex(&sha256(&std::fs::read(binary).map_err(|e| e.to_string())?)) != binary_hash {
            return Err("worker binary changed during formal campaign".into());
        }
        let mut gates = BTreeMap::new();
        let mut passed = true;
        for path in ["fresh", "hit", "restart", "body-edit", "historical-return"] {
            let mut samples = evidence
                .paths
                .get(path)
                .cloned()
                .ok_or("missing formal path")?;
            if samples.len() != SAMPLES || samples.iter().any(|n| !n.is_finite() || *n <= 0.) {
                return Err("missing/zero/nonfinite formal samples".into());
            }
            samples.sort_by(f64::total_cmp);
            let n = samples.len();
            let p90 = samples[n - n / 10 - 1];
            let p95 = samples[n - n / 20 - 1];
            let p99 = samples[n - n / 100 - 1];
            passed &= p90 <= 2000.;
            gates.insert(
                path,
                json!({"samples":n,"p90_ms":p90,"p95_ms":p95,"p99_ms":p99,"pass":p90<=2000.}),
            );
        }
        Ok(
            json!({"complete":true,"passed":passed,"gates":gates,"churn":CHURN,"binding":evidence.binding}),
        )
    })();
    let summary = match &outcome {
        Ok(result) => result.clone(),
        Err(error) => {
            json!({"complete":false,"passed":false,"error":error,"completed_samples":evidence.paths.iter().map(|(path,samples)|(path,samples.len())).collect::<BTreeMap<_,_>>(),"binding":evidence.binding})
        }
    };
    std::fs::write(
        output.join("campaign.json"),
        serde_json::to_vec_pretty(&summary).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    outcome
}
