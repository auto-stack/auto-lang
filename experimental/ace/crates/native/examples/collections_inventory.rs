//! PLAN-889 T-01: inspect the actual source -> shared MIR closure.
//! This reports admission separately from execution; a rejected native unit
//! never counts as a native pass. The AV expectations are hand-computed values.
use std::collections::BTreeSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let cases = [
        ("L01", Some(42)),
        ("L02", Some(42)),
        ("L03", Some(0)),
        ("L04", Some(42)),
        ("L05", Some(42)),
        ("L06", None),
        ("L07", Some(42)),
        ("V01", Some(42)),
        ("V02", Some(42)),
        ("V03", Some(3)),
        ("V04", Some(42)),
        ("V05", Some(42)),
        ("V06", Some(42)),
        ("V07", Some(42)),
        ("LSTR864", Some(51)),
        ("FIN864A", Some(42)),
        ("FIN864B", Some(53)),
    ];
    let mut sources = Vec::new();
    for (id, expected) in cases {
        let source = std::fs::read_to_string(
            root.join(format!("crates/auto-lang/test/cases/ace-m1/src/{id}.at")),
        )?;
        sources.push((id, source, expected, "original"));
    }
    // Distinct witnesses never rewrite an original corpus row.
    for (id, source, expected) in [
        (
            "list-variant-owned",
            r#"
type Item { name Str; code int }
enum Token { Data(Item)
End }
fn main() int {
    var xs = List<Token>.new()
    xs.push(Token.Data(Item("中", 41)))
    let t = xs.remove(0)
    is t {
        Token.Data(v) -> { return v.name.len() + v.code }
        Token.End -> { return 0 }
    }
}
"#,
            42,
        ),
        (
            "list-clone-owned",
            r#"
type Item { name Str; code int }
fn main() int {
    var xs = List<Item>.new()
    xs.push(Item("中", 41))
    var ys = xs.clone()
    let v = ys.remove(0)
    return xs.get(0).name.len() + xs.get(0).code + v.name.len() + v.code
}
"#,
            84,
        ),
        (
            "variant-clone-owned",
            r#"
enum Token { Data(Str)
End }
fn main() int {
    let t = Token.Data("中")
    let c = t.clone()
    var n = 40
    is c { Token.Data(s) -> { n = n + s.len() }
        Token.End -> { n = 0 } }
    is t { Token.Data(s) -> { return n + s.len() }
        Token.End -> { return 0 } }
}
"#,
            42,
        ),
        (
            "list-bool",
            r#"
fn main() int {
    var xs = List<bool>.new()
    xs.push(true)
    if xs.get(0) { return 42 }
    return 0
}
"#,
            42,
        ),
        ("normal-return-70", "fn main() int { return 70 }", 70),
    ] {
        sources.push((id, source.to_string(), Some(expected), "supplement"));
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => {}
        [flag] if flag == "--original-only" => sources.retain(|s| s.3 == "original"),
        [flag, id] if flag == "--case" => sources.retain(|s| s.0 == id),
        _ => return Err("usage: collections_inventory [--original-only | --case ID]".into()),
    }
    if sources.is_empty() {
        return Err("no source selected".into());
    }
    let mut rows = Vec::new();
    let mut closure = BTreeSet::new();
    for (id, source, expected, origin) in sources {
        let compiled = ace_core::source::compile_collections_source(&source, "inventory.at", &[])
            .map_err(|d| format!("{id}: frontend: {d:?}"))?;
        let mir = ace_lowering::lower_checked_collections(&compiled.checked, "d_main", 0)
            .map_err(|e| format!("{id}: lowering: {e:?}"))?;
        ace_av_core::verify_mir(&mir).map_err(|e| format!("{id}: verify: {e:?}"))?;
        let units = ace_av::cache::split_units(&mir)?;
        let mut instructions = BTreeSet::new();
        for f in &mir.funcs {
            for b in &f.blocks {
                for instruction in &b.instrs {
                    // Diagnostic names only; the full typed MIR remains the
                    // verified producer input, never parsed from this text.
                    let rendered = format!("{instruction:?}");
                    instructions.insert(rendered.split([' ', '{']).next().unwrap().to_string());
                }
            }
        }
        closure.extend(instructions.iter().cloned());
        let execution = ace_av::collections_864_support::compile_and_execute(&source, "d_main");
        let av = match (expected, execution) {
            (Some(want), Ok(run)) => {
                if run.value != want || run.live_after != 0 || run.allocs != run.frees {
                    return Err(format!("{id}: independent oracle/resource mismatch").into());
                }
                serde_json::json!({"value": run.value, "live_after": run.live_after,
                    "resource_allocs": run.allocs, "resource_frees": run.frees})
            }
            (None, Err(error)) if error == "trap: av.trap.list-index" => {
                serde_json::json!({"trap": error})
            }
            (want, actual) => {
                return Err(format!(
                    "{id}: expected {want:?}, observed {}",
                    actual.err().unwrap_or_else(|| "unexpected success".into())
                )
                .into())
            }
        };
        let native_admission: Vec<_> = units
            .iter()
            .map(|u| {
                serde_json::json!({"function": u.funcs[0].name,
                "admission": ace_native::emit::admit_unit(u).map(|_| "accepted".to_string())
                    .unwrap_or_else(|e| e.to_string())})
            })
            .collect();
        rows.push(serde_json::json!({"id": id, "origin": origin, "source_blake3": blake3::hash(source.as_bytes()).to_hex().to_string(),
            "profile": mir.profile, "instructions": instructions, "types": format!("{:?}", mir.types),
            "functions": mir.funcs.len(), "units": native_admission, "av": av,
            "native_executed": false}));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "plan889.inventory/1", "source_rows": rows.len(),
            "instruction_closure": closure, "rows": rows,
        }))?
    );
    Ok(())
}
