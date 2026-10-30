//! Separate, feature-only C observation. The caller owns every expectation.
use serde::Deserialize;
use serde_json::json;
use std::io::{Read, Write};

const SCHEMA: &str = "avworker-test-combined/1";
const CAP: u64 = 4 << 20;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema: String,
    nonce: String,
    source: String,
    source_blake3: String,
    ids: Vec<String>,
}

fn reviewed_source() -> String {
    let expressions = [
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../../crates/auto-lang/test/cases/governance-887/src/arithmetic.at"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../../crates/auto-lang/test/cases/governance-887/src/unary.at"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../../crates/auto-lang/test/cases/governance-887/src/group.at"
        )),
    ];
    format!("type Results {{ arithmetic int; unary int; group int }}\nfn main() Results {{\n return Results({}, {}, {})\n}}\n",
            expressions[0], expressions[1], expressions[2])
}

pub fn serve() -> Result<(), String> {
    let fault = std::env::var("AV888_BATCH_FAULT").unwrap_or_default();
    if fault == "no-read" {
        std::thread::sleep(std::time::Duration::from_secs(120));
    }
    let mut bytes = Vec::new();
    std::io::stdin()
        .lock()
        .take(CAP + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > CAP {
        return Err("combined input cap".into());
    }
    let request: Request = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let ids = ["gov.d.arithmetic", "gov.d.unary", "gov.d.group"];
    if request.schema != SCHEMA
        || request.ids != ids
        || request.nonce.len() != 64
        || !request.nonce.bytes().all(|b| b.is_ascii_hexdigit())
        || request.source != reviewed_source()
        || request.source_blake3 != blake3::hash(request.source.as_bytes()).to_hex().as_str()
    {
        return Err("combined source/identity outside reviewed D3 domain".into());
    }
    let (result, metrics) = ace_av::test_observations::capture(|| {
        ace_av::owned_driver::request_owned(None, "plan888-c", &request.source, "d_main", &[])
    })?;
    let mut result = result?;
    let value = match result.av.take() {
        Some(ace_av_core::contract::ExecOutcome::Success { value, .. }) => value,
        other => {
            return Err(format!(
                "combined execution did not return an actual value: {other:?}"
            ))
        }
    };
    let actuals = match &value {
        ace_av_core::contract::Value::Record(record) if record.fields.len() == 3 => record
            .fields
            .iter()
            .map(|v| v.as_i32().ok_or("combined field is not i32"))
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err("combined value is not a three-field record".into()),
    };
    drop(value);
    let live = result
        .arena
        .as_ref()
        .ok_or("combined owned arena missing")?
        .live();
    let mut response = json!({"schema":SCHEMA,"nonce":request.nonce,
        "source_blake3":request.source_blake3,"profile":ace_av_core::contract::AV_OWNED_PROFILE,
        "worker_version":super::EXPECTED_WORKER_VERSION,"metrics":metrics,
        "actuals":ids.iter().zip(actuals).map(|(id,value)|json!({"id":id,"channel":"i32","value":value})).collect::<Vec<_>>(),
        "executor_sequence_observation":result.steps,"live_after_release":live,"worker_wall_ms":result.wall_ms});
    match fault.as_str() {
        "wrong-source" => response["source_blake3"] = json!("wrong"),
        "wrong-index" => response["actuals"][0]["id"] = json!("gov.d.group"),
        "wrong-count" => response["metrics"]["calls"]["compile"] = json!(0),
        "wrong-vm-count" => response["metrics"]["calls"]["vm"] = json!(0),
        _ => (),
    }
    let bytes = serde_json::to_vec(&response).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > CAP {
        return Err("combined output cap".into());
    }
    let mut out = std::io::stdout().lock();
    out.write_all(&bytes)
        .and_then(|_| out.write_all(b"\n"))
        .and_then(|_| out.flush())
        .map_err(|e| e.to_string())?;
    if fault == "extra-after-done" {
        out.write_all(b"{}\n").map_err(|e| e.to_string())?;
    }
    Ok(())
}
