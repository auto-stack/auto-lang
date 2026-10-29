//! Test-only bounded transport. The existing case_request remains the sole
//! compiler/VM producer; no expectations or worker-side pass decisions exist.
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::io::{BufRead, Write};
use std::time::{Duration, Instant};

const SCHEMA: &str = "avworker-test-batch/1";
const CAP: usize = 4 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Frame {
    Init {
        schema: String,
        group: String,
        items: Vec<String>,
    },
    Item {
        schema: String,
        group: String,
        index: usize,
        id: String,
        request: Value,
    },
    Finish {
        schema: String,
        group: String,
        count: usize,
    },
}

pub fn capabilities() -> Value {
    json!({"schema": SCHEMA, "max_items": 10, "frame_cap": CAP,
           "worker_version": super::EXPECTED_WORKER_VERSION,
           "profiles": ["scalar"], "expected_in_worker": false})
}

fn read_frame(reader: &mut impl BufRead, consumed: &mut usize) -> Result<Frame, String> {
    // A bounded read before serde allocation, also when stdin has no newline.
    let mut bytes = Vec::new();
    loop {
        let available = reader.fill_buf().map_err(|e| e.to_string())?;
        if available.is_empty() {
            return Err("truncated/missing batch frame".into());
        }
        let n = available
            .iter()
            .position(|&b| b == b'\n')
            .map_or(available.len(), |n| n + 1);
        if bytes.len() + n > CAP {
            return Err("batch frame exceeds 4MiB".into());
        }
        let finished = available[n - 1] == b'\n';
        bytes.extend_from_slice(&available[..n]);
        reader.consume(n);
        if finished {
            break;
        }
    }
    *consumed += bytes.len();
    // Ten item frames plus tiny init/finish. No cap is multiplied by an
    // untrusted item count; at most ten requests can be dispatched.
    if *consumed > 10 * CAP {
        return Err("batch cumulative input cap".into());
    }
    serde_json::from_slice(&bytes).map_err(|e| format!("invalid batch frame: {e}"))
}

fn emit(writer: &mut impl Write, value: Value, emitted: &mut usize) -> Result<(), String> {
    let mut bytes = serde_json::to_vec(&value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    if bytes.len() > CAP || *emitted + bytes.len() > 10 * CAP {
        return Err("batch output metadata cap".into());
    }
    *emitted += bytes.len();
    writer.write_all(&bytes).map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())
}

fn validate_request(request: &Value) -> Result<std::path::PathBuf, String> {
    let object = request.as_object().ok_or("request must be object")?;
    let allowed = [
        "recipe",
        "nonce",
        "request_hash",
        "source_b64",
        "entry",
        "caps",
        "store_root",
        "steps",
    ];
    if object.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err(
            "batch only accepts scalar requests: unexpected field/profile/session/expected".into(),
        );
    }
    if request["recipe"] != "ac-av/core-run-v1"
        || request["steps"] != json!([{"op":"run"}])
        || request["caps"] != json!([])
        || request["entry"] != "d_main"
    {
        return Err("batch request is outside the scalar whitelist".into());
    }
    let nonce = request["nonce"].as_str().ok_or("missing nonce")?;
    let hash = request["request_hash"]
        .as_str()
        .ok_or("missing request hash")?;
    for value in [nonce, hash] {
        if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid nonce/request identity".into());
        }
    }
    let root = request["store_root"].as_str().ok_or("missing store root")?;
    if root.is_empty() {
        return Err("empty store root".into());
    }
    Ok(std::path::Path::new(root).join(format!("case-{}", &nonce[..16])))
}

pub fn serve(handler: impl FnMut(Value) -> Value) -> Result<(), String> {
    if std::env::var("AV888_BATCH_FAULT").as_deref() == Ok("no-read") {
        std::thread::sleep(Duration::from_secs(240));
    }
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    serve_stream(&mut stdin.lock(), &mut stdout.lock(), handler)
}

fn serve_stream(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    mut handler: impl FnMut(Value) -> Value,
) -> Result<(), String> {
    let started = Instant::now();
    let fault = std::env::var("AV888_BATCH_FAULT").unwrap_or_default();
    let (mut consumed, mut emitted) = (0, 0);
    let Frame::Init {
        schema,
        group,
        items,
    } = read_frame(reader, &mut consumed)?
    else {
        return Err("first batch frame must be init".into());
    };
    if schema != SCHEMA
        || group.is_empty()
        || group.len() > 128
        || items.is_empty()
        || items.len() > 10
        || items.iter().any(|s| s.is_empty() || s.len() > 256)
        || items.iter().collect::<BTreeSet<_>>().len() != items.len()
    {
        return Err("invalid batch manifest/identity/count".into());
    }
    if fault == "hang-init" {
        std::thread::sleep(Duration::from_secs(240));
    }
    if fault == "stdout-flood" {
        writer
            .write_all(&vec![b'x'; CAP + 1])
            .map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
        std::thread::sleep(Duration::from_secs(240));
    }
    if fault == "stderr-flood" {
        std::io::stderr()
            .write_all(&vec![b'x'; (1 << 20) + 1])
            .map_err(|e| e.to_string())?;
        std::thread::sleep(Duration::from_secs(240));
    }
    emit(
        writer,
        json!({"kind":"ready", "schema":SCHEMA, "group":group, "items":items,
                       "capabilities":capabilities()}),
        &mut emitted,
    )?;
    if fault == "no-read-after-ready" {
        std::thread::sleep(Duration::from_secs(240));
    }
    for (expected_index, expected_id) in items.iter().enumerate() {
        if started.elapsed() >= Duration::from_secs(60) {
            return Err("batch group wall".into());
        }
        let Frame::Item {
            schema,
            group: item_group,
            index,
            id,
            request,
        } = read_frame(reader, &mut consumed)?
        else {
            return Err("missing batch item".into());
        };
        if schema != SCHEMA || item_group != group || index != expected_index || &id != expected_id
        {
            return Err("batch item identity/order mismatch".into());
        }
        let store = validate_request(&request)?;
        // Driver/module/VM/outputs are request-local in the common handler.
        if fault == "exit-item-1" && index == 1 {
            std::process::exit(9);
        }
        let mut response = handler(request);
        if fault == "wrong-nonce" {
            response["nonce"] = json!("wrong");
        }
        if fault == "wrong-hash" {
            response["request_hash"] = json!("wrong");
        }
        if fault == "wrong-source" {
            response["ok"]["steps"][0]["source_hash"] = json!("ffffffffffffffff");
        }
        if store.exists() {
            std::fs::remove_dir_all(&store).map_err(|e| format!("case store cleanup: {e}"))?;
        }
        if fault == "truncated-item" {
            writer
                .write_all(b"{\"kind\":\"actual\"")
                .map_err(|e| e.to_string())?;
            writer.flush().map_err(|e| e.to_string())?;
            std::process::exit(9);
        }
        let reported_index = if fault == "wrong-index" {
            index + 1
        } else {
            index
        };
        emit(
            writer,
            json!({"kind":"actual", "schema":SCHEMA, "group":group,
                           "index":reported_index, "id":id, "response":response}),
            &mut emitted,
        )?;
        // Parent sends the next item only after its ordinary admission/compare.
    }
    let Frame::Finish {
        schema,
        group: finish_group,
        count,
    } = read_frame(reader, &mut consumed)?
    else {
        return Err("missing batch finish".into());
    };
    if schema != SCHEMA
        || finish_group != group
        || count != items.len()
        || started.elapsed() >= Duration::from_secs(60)
    {
        return Err("batch finish identity/count/wall mismatch".into());
    }
    // Require EOF after finish: an extra frame cannot be silently ignored.
    if !reader.fill_buf().map_err(|e| e.to_string())?.is_empty() {
        return Err("extra batch input after finish".into());
    }
    if fault == "no-done" {
        return Ok(());
    }
    emit(
        writer,
        json!({"kind":"done", "schema":SCHEMA, "group":group,
                       "count":count, "worker_starts":1, "case_calls":count}),
        &mut emitted,
    )?;
    if fault == "extra-after-done" {
        writer.write_all(b"{}\n").map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
    }
    if fault == "exit-after-done" {
        std::process::exit(9);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_expected_and_unknown_request_fields_before_pipeline() {
        let mut request = json!({"recipe":"ac-av/core-run-v1", "nonce":"a".repeat(64),
            "request_hash":"b".repeat(64), "source_b64":"", "entry":"d_main", "caps":[],
            "store_root":"unused", "steps":[{"op":"run"}]});
        assert!(validate_request(&request).is_ok());
        request["expected"] = json!(3);
        assert!(validate_request(&request).is_err());
    }
    #[test]
    fn rejects_oversized_unterminated_frame_without_dispatch() {
        let input = vec![b'x'; CAP + 1];
        assert!(read_frame(&mut input.as_slice(), &mut 0).is_err());
    }
    #[test]
    fn rejects_duplicate_manifest_and_outer_unknown_fields() {
        for input in [
            json!({"kind":"init","schema":SCHEMA,"group":"g","items":["a","a"]}),
            json!({"kind":"init","schema":SCHEMA,"group":"g","items":["a"],"expected":3}),
        ] {
            let line = format!("{input}\n");
            let result = serve_stream(&mut line.as_bytes(), &mut Vec::new(), |_| {
                panic!("must not dispatch")
            });
            assert!(result.is_err());
        }
    }
}
