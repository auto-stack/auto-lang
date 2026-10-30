//! Parent admission of raw collections observations. This module executes no
//! guest operations and receives no expected result from the native worker.
use serde_json::Value;
use std::collections::BTreeMap;
pub(super) mod campaign;
const RESOURCES: &[&str] = &[
    "live",
    "bytes",
    "guest",
    "peak",
    "allocs",
    "frees",
    "invalid",
    "arena_live",
    "arena_commit",
    "arena_peak",
    "arena_invalid",
    "text_metadata",
    "registered_bytes",
    "managed_commit_peak",
    "process_private_base",
    "process_private_end",
    "process_commit_peak",
    "process_commit_delta",
    "process_sample_ok",
];
fn unhex(raw: &str) -> Result<Vec<u8>, String> {
    if raw.len() > 2 << 20 || raw.len() % 2 != 0 {
        return Err("collections raw stream size/hex".into());
    }
    raw.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let a = (pair[0] as char)
                .to_digit(16)
                .ok_or("collections raw hex")?;
            let b = (pair[1] as char)
                .to_digit(16)
                .ok_or("collections raw hex")?;
            Ok((a * 16 + b) as u8)
        })
        .collect()
}
fn decimal(raw: &str) -> Result<u64, String> {
    if raw.is_empty() || !raw.bytes().all(|c| c.is_ascii_digit()) {
        return Err("malformed native metric".into());
    }
    raw.parse().map_err(|_| "native metric overflow".into())
}
fn fields(raw: &str, keys: &[&str]) -> Result<BTreeMap<String, u64>, String> {
    let mut out = BTreeMap::new();
    for field in raw.split(' ') {
        let (key, value) = field
            .split_once('=')
            .ok_or("missing native metric separator")?;
        if !keys.contains(&key) || out.insert(key.into(), decimal(value)?).is_some() {
            return Err("unknown/duplicate native metric".into());
        }
    }
    if out.len() != keys.len() {
        return Err("missing native metric".into());
    }
    Ok(out)
}
fn raw_metrics(
    raw: &str,
) -> Result<
    (
        BTreeMap<String, u64>,
        BTreeMap<String, u64>,
        BTreeMap<String, u64>,
        Option<u32>,
    ),
    String,
> {
    let bytes = unhex(raw)?;
    if bytes.last() != Some(&b'\n') {
        return Err("missing/truncated native observation".into());
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| "native observation UTF8")?;
    let mut resources = BTreeMap::new();
    let mut summary = None;
    let mut free = None;
    let mut trap = None;
    for line in text.split_terminator('\n') {
        if line.contains("MN-COLL-TRAP") {
            let raw = line
                .strip_prefix("MN-COLL-TRAP code=")
                .ok_or("forged native trap prefix")?;
            let code = decimal(raw)?;
            if raw != code.to_string()
                || !matches!(code, 21..=26 | 30 | 31)
                || trap.replace(code as u32).is_some()
            {
                return Err("unknown/duplicate native trap".into());
            }
        } else if let Some(raw) = line.strip_prefix("MN-COLL ") {
            let (key, value) = raw.split_once('=').ok_or("native metric separator")?;
            if !RESOURCES.contains(&key) || resources.insert(key.into(), decimal(value)?).is_some()
            {
                return Err("unknown/duplicate native resource".into());
            }
        } else if let Some(raw) = line.strip_prefix("SUMMARY ") {
            if summary
                .replace(fields(
                    raw,
                    &["attempts", "allocs", "frees", "live", "live_bytes", "peak"],
                )?)
                .is_some()
            {
                return Err("duplicate native text summary".into());
            }
        } else if let Some(raw) = line.strip_prefix("FREESTAT ") {
            if free
                .replace(fields(raw, &["free_attempts", "frees_ok", "invalid"])?)
                .is_some()
            {
                return Err("duplicate native free summary".into());
            }
        }
    }
    if resources.len() != RESOURCES.len() {
        return Err("missing native resources".into());
    }
    Ok((
        resources,
        summary.ok_or("missing native text summary")?,
        free.ok_or("missing native free summary")?,
        trap,
    ))
}
pub(super) fn trap_from_hex(raw: &str) -> Result<Option<u32>, String> {
    Ok(raw_metrics(raw)?.3)
}
pub(super) fn trap_code(reason: u32) -> &'static str {
    match reason {
        21 => "mn.trap.list-index",
        24 => "mn.trap.variant-arm",
        30 => "mn.trap.division",
        31 => "mn.trap.overflow",
        _ => "mn.trap.resource",
    }
}
pub(super) fn admit_raw(ok: &Value) -> Result<(), String> {
    let meter = ok
        .get("allocation_meter")
        .ok_or("missing actual compiler allocation meter")?;
    if meter.get("enabled").and_then(Value::as_bool) != Some(true) {
        return Err("inactive compiler allocation meter".into());
    }
    let peaks = meter
        .get("peak")
        .and_then(Value::as_array)
        .filter(|a| a.len() == 3)
        .ok_or("missing compiler partition peaks")?;
    for (peak, cap) in peaks.iter().zip([16u64 << 20, 24 << 20, 8 << 20]) {
        if peak.as_u64().is_none_or(|n| n > cap) {
            return Err("compiler allocation partition cap".into());
        }
    }
    if meter
        .get("total_peak")
        .and_then(Value::as_u64)
        .filter(|n| *n > 0 && *n <= 48 << 20)
        .is_none()
    {
        return Err("compiler allocation total cap".into());
    }
    let total = meter["total_peak"].as_u64().unwrap();
    let peak_values: Vec<u64> = peaks.iter().map(|n| n.as_u64().unwrap()).collect();
    let current = meter
        .get("current")
        .and_then(Value::as_array)
        .filter(|a| a.len() == 3)
        .ok_or("missing compiler current allocations")?;
    if total < *peak_values.iter().max().unwrap()
        || total > peak_values.iter().sum::<u64>()
        || current
            .iter()
            .zip(&peak_values)
            .any(|(current, peak)| current.as_u64().is_none_or(|n| n > *peak))
    {
        return Err("inconsistent compiler allocation peaks".into());
    }
    let memory = ok
        .get("worker_memory")
        .ok_or("missing physical worker observation")?;
    let get = |key| {
        memory
            .get(key)
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("missing physical worker {key}"))
    };
    let base = get("base")?;
    let current = get("current")?;
    let peak = get("peak")?;
    let delta = get("delta")?;
    if get("ok")? != 1
        || base == 0
        || current == 0
        || peak == 0
        || delta != peak.saturating_sub(base)
        || delta > 128 << 20
    {
        return Err("physical worker private commit cap/observation".into());
    }
    if ok.get("collections_abi").and_then(Value::as_str) != Some("mn.collections.cell/2")
        || ok
            .get("collections_codegen")
            .and_then(Value::as_str)
            .filter(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
            .is_none()
    {
        return Err("collections ABI/codegen identity".into());
    }
    let raw = ok
        .get("stderr_hex")
        .and_then(Value::as_str)
        .ok_or("missing raw native stderr")?;
    let (resources, text, free, trap) = raw_metrics(raw)?;
    let exit = ok
        .get("exit_code")
        .and_then(Value::as_i64)
        .ok_or("missing actual native exit")?;
    if trap.is_some() && exit != 70 {
        return Err("native trap with non-trap exit".into());
    }
    if matches!(trap, Some(22 | 23 | 25 | 26)) {
        return Err("native execution resource/invalid/cancel failure".into());
    }
    if resources["process_sample_ok"] != 1
        || [
            "process_private_base",
            "process_private_end",
            "process_commit_peak",
        ]
        .iter()
        .any(|key| resources[*key] == 0)
        || resources["process_commit_delta"]
            != resources["process_commit_peak"].saturating_sub(resources["process_private_base"])
        || resources["process_commit_delta"] > 128 << 20
    {
        return Err("native process memory observation".into());
    }
    if [
        "live",
        "bytes",
        "guest",
        "invalid",
        "arena_live",
        "arena_commit",
        "arena_invalid",
        "text_metadata",
        "registered_bytes",
    ]
    .iter()
    .any(|key| resources[*key] != 0)
        || resources["allocs"] != resources["frees"]
        || text["allocs"] != text["frees"]
        || text["live"] != 0
        || text["live_bytes"] != 0
        || free["invalid"] != 0
        || free["frees_ok"] != text["frees"]
    {
        return Err("native owned resource leak/invalid release".into());
    }
    if ["peak", "arena_peak", "managed_commit_peak"]
        .iter()
        .any(|key| resources[*key] > 32 << 20)
        || text["peak"] > 8 << 20
    {
        return Err("native execution memory cap".into());
    }
    for counter in ["link_invocations", "run_invocations"] {
        if ok
            .get("counters")
            .and_then(|c| c.get(counter))
            .and_then(Value::as_u64)
            != Some(1)
        {
            return Err(format!("collections {counter} must be exactly one"));
        }
    }
    let runtime = ok
        .get("runtime_identity")
        .ok_or("missing bound collections runtime identity")?;
    if runtime.as_object().is_none_or(|map| map.len() != 3)
        || runtime.get("abi").and_then(Value::as_str) != Some("mn.collections.runtime/2")
        || ["build_key", "content_hash"].iter().any(|key| {
            runtime
                .get(*key)
                .and_then(Value::as_str)
                .is_none_or(|hash| hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()))
        })
    {
        return Err("unbound collections runtime content/build identity".into());
    }
    let link = ok
        .get("link_identity")
        .and_then(Value::as_array)
        .filter(|array| array.len() == 3)
        .ok_or("missing actual linker/SDK identities")?;
    if [0usize, 2].into_iter().any(|i| {
        link[i]
            .as_str()
            .is_none_or(|hash| hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()))
    }) || link[1].as_str().is_none_or(str::is_empty)
    {
        return Err("malformed actual linker/SDK identity".into());
    }
    Ok(())
}
pub(super) fn admit_identities(ok: &Value, changed: Option<&str>) -> Result<(), String> {
    let before = ok
        .get("identities_before")
        .and_then(Value::as_object)
        .ok_or("missing actual before identities")?;
    let after = ok
        .get("identities_after")
        .and_then(Value::as_object)
        .ok_or("missing actual after identities")?;
    if before.is_empty() || before.len() != after.len() {
        return Err("producer identity set changed".into());
    }
    for (name, old) in before {
        let new = after.get(name).ok_or("producer identity removed")?;
        if Some(name.as_str()) != changed && old != new {
            return Err(format!("unchanged producer/code identity changed: {name}"));
        }
        for value in [old, new] {
            if value.as_array().is_none_or(|list| {
                list.len() != 3
                    || list
                        .iter()
                        .any(|ptr| ptr.as_u64().is_none_or(|ptr| ptr == 0))
            }) {
                return Err("missing actual source/MIR/COFF allocation address".into());
            }
        }
    }
    if let Some(changed) = changed {
        if before.get(changed).is_none() || before.get(changed) == after.get(changed) {
            return Err("changed source producer was not replaced".into());
        }
    }
    Ok(())
}
pub(super) fn admit_body(ok: &Value) -> Result<(), String> {
    let changed = ok
        .get("changed_fn")
        .and_then(Value::as_str)
        .ok_or("missing actual changed function")?;
    admit_identities(ok, Some(changed))?;
    let parsed = ok
        .get("parsed_functions")
        .and_then(Value::as_object)
        .ok_or("missing actual parser observation")?;
    let bodies = ok
        .get("bodies")
        .and_then(Value::as_object)
        .ok_or("missing actual body observation")?;
    if parsed.len() != 1
        || parsed.get(changed).and_then(Value::as_u64) != Some(1)
        || bodies.len() != 1
        || bodies.get(changed) != Some(&serde_json::json!([1, 1, 1, 1]))
    {
        return Err("collections edit is not an actual single-body producer".into());
    }
    for key in [
        "unaffected_stage_delta",
        "whole_module_stage_delta",
        "fallback_whole_lower",
    ] {
        if ok.get(key).and_then(Value::as_u64) != Some(0) {
            return Err(format!("collections {key} must be present and zero"));
        }
    }
    if ok
        .get("codegen_units")
        .and_then(Value::as_u64)
        .is_none_or(|count| count > 1)
    {
        return Err("collections edit produced multiple native units".into());
    }
    Ok(())
}

#[cfg(all(test, feature = "test-ace"))]
mod controls {
    use super::*;
    #[test]
    fn native_collections_identity_request_hash_position_and_forgery() {
        let unsigned = serde_json::json!({"recipe":super::super::COLLECTIONS_RECIPE,"nonce":"889-identity-position","mode":"identity"});
        let hash = super::super::hex(&super::super::sha256(unsigned.to_string().as_bytes()));
        let mut signed = serde_json::Map::new();
        for (index, (key, value)) in unsigned.as_object().unwrap().iter().enumerate() {
            signed.insert(key.clone(), value.clone());
            if index == 0 {
                signed.insert("request_hash".into(), serde_json::json!(hash));
            }
        }
        let mut signed = Value::Object(signed);
        let line = super::super::call_worker(&signed.to_string()).unwrap();
        let response: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(response["nonce"], signed["nonce"]);
        assert_eq!(
            response["ok"]["recipe"],
            super::super::COLLECTIONS_RECIPE,
            "{response}"
        );
        assert!(response["ok"]["runtime_identity"]["content_hash"]
            .as_str()
            .is_some_and(|hash| hash.len() == 64));
        signed["request_hash"] = serde_json::json!("0".repeat(64));
        let line = super::super::call_worker(&signed.to_string()).unwrap();
        let response: Value = serde_json::from_str(&line).unwrap();
        assert!(
            response["error"]["message"]
                .as_str()
                .is_some_and(|message| message.contains("request hash mismatch")),
            "{response}"
        );
        assert!(response.get("ok").is_none());
    }
    fn valid() -> Value {
        let mut wire = String::new();
        for key in RESOURCES {
            let value = match *key {
                "allocs" | "frees" | "process_sample_ok" => 1,
                "peak" | "arena_peak" | "managed_commit_peak" => 4096,
                "process_private_base" | "process_private_end" | "process_commit_delta" => 1024,
                "process_commit_peak" => 2048,
                _ => 0,
            };
            wire.push_str(&format!("MN-COLL {key}={value}\n"));
        }
        wire.push_str("SUMMARY attempts=1 allocs=1 frees=1 live=0 live_bytes=0 peak=5\nFREESTAT free_attempts=1 frees_ok=1 invalid=0\n");
        serde_json::json!({"collections_abi":"mn.collections.cell/2","collections_codegen":"a".repeat(64),"allocation_meter":{"enabled":true,"current":[0,0,0],"peak":[1024,2048,512],"total_peak":2304},"worker_memory":{"base":1024,"current":1024,"peak":2048,"delta":1024,"ok":1},"exit_code":42,"stderr_hex":super::super::hex(wire.as_bytes()),"counters":{"link_invocations":1,"run_invocations":1},"runtime_identity":{"abi":"mn.collections.runtime/2","build_key":"b".repeat(64),"content_hash":"c".repeat(64)},"link_identity":["d".repeat(64),"10.0.fixture","e".repeat(64)]})
    }
    #[test]
    fn native_collections_parent_raw_and_actual_stage_controls() {
        let control = valid();
        assert!(admit_raw(&control).is_ok());
        for fault in 0..18 {
            let mut bad = control.clone();
            let raw =
                String::from_utf8(unhex(bad["stderr_hex"].as_str().unwrap()).unwrap()).unwrap();
            match fault {
                0 => {
                    bad.as_object_mut().unwrap().remove("allocation_meter");
                }
                1 => bad["allocation_meter"]["enabled"] = serde_json::json!(false),
                2 => bad["allocation_meter"]["peak"][0] = serde_json::json!((16 << 20) + 1),
                3 => bad["allocation_meter"]["total_peak"] = serde_json::json!(1),
                4 => bad["worker_memory"]["ok"] = serde_json::json!(0),
                5 => bad["worker_memory"]["delta"] = serde_json::json!(128 << 20),
                6 => bad["collections_abi"] = serde_json::json!("wrong"),
                7 => {
                    bad.as_object_mut().unwrap().remove("runtime_identity");
                }
                8 => bad["link_identity"][0] = serde_json::json!("forged"),
                9 => bad["counters"]["link_invocations"] = serde_json::json!(0),
                10 => bad["exit_code"] = Value::Null,
                11 => {
                    bad["stderr_hex"] =
                        serde_json::json!(super::super::hex(raw.trim_end().as_bytes()))
                }
                12 => {
                    bad["stderr_hex"] = serde_json::json!(super::super::hex(
                        raw.replace("MN-COLL live=0", "MN-COLL live=1").as_bytes()
                    ))
                }
                13 => {
                    bad["stderr_hex"] = serde_json::json!(super::super::hex(
                        format!("{raw}MN-COLL live=0\n").as_bytes()
                    ))
                }
                14 => {
                    bad["stderr_hex"] = serde_json::json!(super::super::hex(
                        format!("{raw}prefix MN-COLL-TRAP code=21\n").as_bytes()
                    ))
                }
                15 => {
                    bad["stderr_hex"] = serde_json::json!(super::super::hex(
                        raw.replace("frees_ok=1", "frees_ok=0").as_bytes()
                    ))
                }
                16 => {
                    bad["stderr_hex"] = serde_json::json!(super::super::hex(
                        format!("{raw}MN-COLL-TRAP code=21\n").as_bytes()
                    ))
                }
                _ => {
                    bad["stderr_hex"] = serde_json::json!(super::super::hex(
                        raw.replace("MN-COLL arena_peak=4096", "MN-COLL arena_peak=33554433")
                            .as_bytes()
                    ))
                }
            }
            assert!(admit_raw(&bad).is_err(), "fault {fault}");
        }
        let mut normal70 = control.clone();
        normal70["exit_code"] = serde_json::json!(70);
        assert!(admit_raw(&normal70).is_ok());
        assert_eq!(
            trap_from_hex(normal70["stderr_hex"].as_str().unwrap()).unwrap(),
            None
        );
        let mut body = control;
        body["changed_fn"] = serde_json::json!("helper");
        body["parsed_functions"] = serde_json::json!({"helper":1});
        body["bodies"] = serde_json::json!({"helper":[1,1,1,1]});
        body["codegen_units"] = serde_json::json!(1);
        for key in [
            "unaffected_stage_delta",
            "whole_module_stage_delta",
            "fallback_whole_lower",
        ] {
            body[key] = serde_json::json!(0);
        }
        body["identities_before"] = serde_json::json!({"helper":[1,2,3],"main":[4,5,6]});
        body["identities_after"] = serde_json::json!({"helper":[7,8,9],"main":[4,5,6]});
        assert!(admit_body(&body).is_ok());
        for fault in 0..5 {
            let mut bad = body.clone();
            match fault {
                0 => bad["bodies"]["main"] = serde_json::json!([1, 1, 1, 1]),
                1 => bad["parsed_functions"]["main"] = serde_json::json!(1),
                2 => bad["identities_after"]["main"][2] = serde_json::json!(99),
                3 => bad["codegen_units"] = serde_json::json!(2),
                _ => {
                    bad.as_object_mut()
                        .unwrap()
                        .remove("whole_module_stage_delta");
                }
            }
            assert!(admit_body(&bad).is_err(), "body fault {fault}");
        }
    }
}
