//! PLAN-872 T-01：六族 builtin 绑定面实测探针（证据生成器）。
//!
//! 两个观测面：
//! 1. `plan872_probe_binding_surface` —— 名字 → ID（codegen peek 与运行期
//!    interface.resolve 双视角）→ 是否绑定 → contract.producer（最后写入者）。
//! 2. `plan872_probe_behavior_*` —— 最小 Auto 程序经 run_with_capture 实测
//!    生效语义（字符 vs 字节、Err vs 0 兜底等）。
//!
//! 本文件是 T-01 的证据生成器（--nocapture 输出转录进绑定表）；T-04 会把
//! 关键观测固化为具名区分性测试。

/// registry 就绪 + production() 接口（生产 boot 同款次序，plan755 同款）。
fn production_interface() -> crate::vm::native::NativeInterface {
    let stdlib_root = crate::stdlib_assembly::loader::repo_stdlib_root().unwrap();
    let repo_root = stdlib_root
        .parent()
        .and_then(|p| p.parent())
        .unwrap()
        .to_path_buf();
    let orig = std::env::current_dir().unwrap();
    std::env::set_current_dir(&repo_root).unwrap();
    crate::vm::native_registry::register_builtin_natives();
    let _ = std::env::set_current_dir(orig);
    crate::vm::native::NativeInterface::production()
}

fn report_name(out: &mut String, interface: &crate::vm::native::NativeInterface, name: &str) {
    let peek = crate::vm::native_registry::BIGVM_NATIVES
        .lock()
        .unwrap()
        .peek_qualified(name);
    let resolved = interface.resolve(name);
    let id = resolved.or(peek);
    let bound = id.map(|i| interface.get(i).is_some());
    let producer = id.and_then(|i| interface.contract(i).map(|c| c.producer.clone()));
    let _ = interface.contract(id.unwrap_or(0));
    out.push_str(&format!(
        "{:<22} peek={:?} iface={:?} bound={:?} producer={}\n",
        name, peek, resolved, bound, producer.unwrap_or_else(|| "-".into())
    ));
}

#[test]
fn plan872_probe_binding_surface() {
    let interface = production_interface();
    let mut out = String::new();

    out.push_str("== str.len / byte_len / to_int / parse_int ==\n");
    for name in FAMILY_NAMES_STR {
        report_name(&mut out, &interface, name);
    }

    out.push_str("== Math ==\n");
    for name in FAMILY_NAMES_MATH {
        report_name(&mut out, &interface, name);
    }

    out.push_str("== Time ==\n");
    for name in FAMILY_NAMES_TIME {
        report_name(&mut out, &interface, name);
    }

    println!("{out}");
}

/// 六族涉及的名字全集（T-01 表的机器面；JSON 生成器与探针共用）。
pub const FAMILY_NAMES_STR: &[&str] = &[
    "Str.len",
    "str.len",
    "auto.str.len",
    "auto.str.length",
    "String.len",
    "Str.byte_len",
    "auto.str.byte_len",
    "Str.to_int",
    "str.to_int",
    "auto.str.to_int",
    "Str.parse_int",
    "auto.str.parse_int",
];
pub const FAMILY_NAMES_MATH: &[&str] = &[
    "Math.abs",
    "auto.math.abs",
    "Math.min",
    "Math.max",
    "Math.sqrt",
    "auto.math.sqrt",
    "Math.abs_f",
    "auto.math.abs_f",
    "Math.min_f",
    "Math.max_f",
    "Math.floor",
    "Math.ceil",
    "Math.round",
    "Math.sin",
    "auto.math.sin",
    "Math.cos",
    "Math.tan",
    "Math.exp",
    "Math.ln",
    "Math.log2",
    "Math.log10",
    "Math.signum",
    "Math.asin",
    "Math.acos",
    "Math.atan",
    "Math.atan2",
    "Math.pow",
    "Math.powi",
    "Math.powf",
    "Math.to_radians",
    "Math.to_degrees",
    "Math.clamp",
    "Math.trunc",
    "Math.imul",
];
pub const FAMILY_NAMES_TIME: &[&str] = &[
    "Time.now",
    "Time.now_ms",
    "Time.now_sec",
    "Time.sleep_ms",
    "auto.time.now",
    "auto.time.now_ms",
    "auto.time.now_sec",
    "auto.time.sleep_ms",
];

/// V-01 守卫：`scripts/builtin-binding-table.json` 与按同一提交重新生成的
/// 内容逐字节一致。设置 `AUTO_LANG_872_EMIT_TABLE=1` 时改写该文件
/// （表内容变更后重新冻结用；冻结提交上两次运行逐字节一致即满足 V-01）。
#[test]
fn plan872_binding_table_json_matches_regeneration() {
    let json = generate_binding_table_json();
    let repo_scripts = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("scripts")
        .join("builtin-binding-table.json");
    if std::env::var("AUTO_LANG_872_EMIT_TABLE").ok().as_deref() == Some("1") {
        std::fs::write(&repo_scripts, &json).expect("rewrite scripts/builtin-binding-table.json");
        return;
    }
    let committed =
        std::fs::read_to_string(&repo_scripts).expect("scripts/builtin-binding-table.json committed");
    assert!(
        committed == json,
        "scripts/builtin-binding-table.json drifted from regeneration; \
         rerun with AUTO_LANG_872_EMIT_TABLE=1 and review the diff"
    );
}

fn generate_binding_table_json() -> String {
    use std::collections::BTreeMap;
    let interface = production_interface();
    // `codegen_peek_id` 用**本地新鲜注册表**计算（纯静态表答案，无进程历史
    // 依赖）——全局 BIGVM_NATIVES 会因并发测试的编译活动插入直命中条目
    // （如 "String.len"→171），使 peek 答案在测试并发下漂移；fresh 视角
    // 恒定（canonical 先于 path 静态表，peek/resolve 分歧的 peek 侧）。
    let mut fresh_registry = crate::vm::native_registry::AutoVMNativeRegistry::new();

    // 注册面指纹：NATIVE_ID_ENTRIES 规模 + 同名异 ID（collect 后写胜出的分歧行）。
    let entries = crate::vm::native_catalog::NATIVE_ID_ENTRIES;
    let mut name_ids: BTreeMap<&str, Vec<u16>> = BTreeMap::new();
    for &(name, id) in entries {
        name_ids.entry(name).or_default().push(id);
    }
    let dup_name_ids: BTreeMap<&str, Vec<u16>> = name_ids
        .into_iter()
        .filter(|(_, ids)| ids.len() > 1)
        .collect();

    let mut families = serde_json::Map::new();
    for (family, names) in [
        ("str_len_byte_len_to_int_parse_int", FAMILY_NAMES_STR),
        ("math", FAMILY_NAMES_MATH),
        ("time", FAMILY_NAMES_TIME),
    ] {
        let mut rows = Vec::new();
        for &name in names {
            let peek = fresh_registry.peek_qualified(name);
            let iface = interface.resolve(name);
            let id = iface.or(peek);
            rows.push(serde_json::json!({
                "name": name,
                "codegen_peek_id": peek,
                "runtime_interface_id": iface,
                "bound": id.map(|i| interface.get(i).is_some()),
                "last_writer_producer": id
                    .and_then(|i| interface.contract(i).map(|c| c.producer.clone()))
                    .unwrap_or_else(|| "<no contract: bound via register()/override block>".into()),
            }));
        }
        families.insert(family.to_string(), serde_json::Value::Array(rows));
    }

    let doc = serde_json::json!({
        "plan": "PLAN-872",
        "note": "six-family builtin binding table; see docs/plans/872-builtin-semantics-dual-track-ruling.md and attachments/872/binding-table.md. codegen_peek_id = fresh-registry pure decision view (peek side of the documented peek/resolve divergence); runtime_interface_id = production() steady-state CALL_SPEC view",
        "generate": "cargo test -p auto-lang --lib plan872_binding_table_json -- --nocapture  (set AUTO_LANG_872_EMIT_TABLE=1 to rewrite after a deliberate binding change)",
        "binding_order_production": "register_std_shims(catalog: nv impls + canonical names) -> register_stdlib_ffi(manual) -> register_interop_natives -> build_from_inventory(rust_fn pure shims; same-ID rebind overwrites catalog) -> register_request_io_shims -> nv override block(1504/1505/1506/1516/1523 + auto.json.parse + web natives)",
        "registry_fingerprint": {
            "native_id_entries": entries.len(),
            "dup_name_ids": dup_name_ids,
        },
        "families_not_name_bound": {
            "error_reply_x10": "ApiReply::Full error constructions in server_dispatch.rs/server_argbind.rs/server_scope.rs; body pinned by plan755_http_contract_tests",
            "nv_to_value_dual": "native.rs nv_to_value(nv) vs interop.rs nv_to_value(vm,nv); internal Rust conversion, no builtin name"
        },
        "families": families,
    });
    serde_json::to_string_pretty(&doc).unwrap() + "\n"
}

fn run_capture(program: &str) -> String {
    let (_r, stdout) = crate::run_with_capture(program).unwrap_or_else(|e| {
        panic!("program failed to run: {e}\n---\n{program}\n---");
    });
    stdout
}

fn probe(out: &mut String, label: &str, program: &str) {
    let stdout = run_capture(program);
    out.push_str(&format!("{label:<34} => {:?}\n", stdout.trim_end_matches('\n')));
}

#[test]
fn plan872_probe_behavior_str() {
    let mut out = String::new();
    // CJK "中文" = 2 chars / 6 UTF-8 bytes：字符/字节语义直接可分辨。
    probe(
        &mut out,
        "Str.len(\"中文\") free-call",
        r#"fn main() { print(Str.len("中文")) }"#,
    );
    probe(
        &mut out,
        "\"中文\".len() method-call",
        r#"fn main() { print("中文".len()) }"#,
    );
    probe(
        &mut out,
        "\"中文\".length property",
        r#"fn main() { print("中文".length) }"#,
    );
    probe(
        &mut out,
        "Str.byte_len(\"中文\") free-call",
        r#"fn main() { print(Str.byte_len("中文")) }"#,
    );
    probe(
        &mut out,
        "\"中文\".byte_len() method-call",
        r#"fn main() { print("中文".byte_len()) }"#,
    );
    probe(
        &mut out,
        "Str.len(\"\") / byte_len(\"\")",
        r#"fn main() { print(Str.len("")); print("|"); print(Str.byte_len("")) }"#,
    );
    println!("{out}");
}

/// 字节码证据：两种 len 形态各自发出什么调用（CALL_NAT #id / CALL_SPEC 名）。
#[test]
fn plan872_probe_bytecode_len_forms() {
    let mut out = String::new();
    for (label, code) in [
        ("free Str.len", r#"fn main() { Str.len("x") }"#),
        ("method .len()", r#"fn main() { "x".len() }"#),
        ("property .length", r#"fn main() { "x".length }"#),
        ("method .byte_len()", r#"fn main() { "x".byte_len() }"#),
    ] {
        let (_r, _o, bc, _meta) =
            crate::run_with_capture_and_bytecode_with_meta(code).expect("run");
        out.push_str(&format!("== {label} ==\n"));
        for line in &bc {
            out.push_str(&format!("  {line}\n"));
        }
    }
    println!("{out}");
}

#[test]
fn plan872_probe_behavior_math() {
    let mut out = String::new();
    probe(
        &mut out,
        "Math.sqrt(-1.0)",
        r#"fn main() { print(Math.sqrt(-1.0)) }"#,
    );
    probe(
        &mut out,
        "Math.sqrt(-1) int-arg",
        r#"fn main() { print(Math.sqrt(-1)) }"#,
    );
    probe(
        &mut out,
        "Math.ln(0.0)",
        r#"fn main() { print(Math.ln(0.0)) }"#,
    );
    probe(
        &mut out,
        "Math.abs(-7) int",
        r#"fn main() { print(Math.abs(-7)) }"#,
    );
    probe(
        &mut out,
        "Math.abs_f(-7.5) float",
        r#"fn main() { print(Math.abs_f(-7.5)) }"#,
    );
    probe(
        &mut out,
        "Math.round(2.5) half-away?",
        r#"fn main() { print(Math.round(2.5)) }"#,
    );
    probe(
        &mut out,
        "Math.round(-2.5)",
        r#"fn main() { print(Math.round(-2.5)) }"#,
    );
    probe(
        &mut out,
        "Math.sin(int pi/2 =1)",
        r#"fn main() { print(Math.sin(1)) }"#,
    );
    probe(
        &mut out,
        "Math.sin(1.0) float-lit",
        r#"fn main() { print(Math.sin(1.0)) }"#,
    );
    probe(
        &mut out,
        "Math.floor(int 7) tag-probe",
        r#"fn main() { print(Math.floor(7)) }"#,
    );
    println!("{out}");
}

#[test]
fn plan872_probe_behavior_to_int() {
    let mut out = String::new();
    probe(
        &mut out,
        "\"42\".to_int() method",
        r#"fn main() { print("42".to_int()) }"#,
    );
    probe(
        &mut out,
        "\"12x\".to_int() method",
        r#"fn main() { print("12x".to_int()) }"#,
    );
    probe(
        &mut out,
        "Str.to_int(\"12x\") free",
        r#"fn main() { print(Str.to_int("12x")) }"#,
    );
    probe(
        &mut out,
        "Str.parse_int(\"12x\") free",
        r#"fn main() { print(Str.parse_int("12x")) }"#,
    );
    probe(
        &mut out,
        "Str.parse_int(\"-98\") free",
        r#"fn main() { print(Str.parse_int("-98")) }"#,
    );
    println!("{out}");
}

#[test]
fn plan872_probe_behavior_time() {
    let mut out = String::new();
    probe(
        &mut out,
        "Time.now_sec() > 1_600_000_000",
        r#"fn main() { print(Time.now_sec() > 1600000000) }"#,
    );
    probe(
        &mut out,
        "Time.now_ms() >= now_sec()*1000",
        r#"fn main() { print(Time.now_ms() >= Time.now_sec() * 1000) }"#,
    );
    probe(
        &mut out,
        "Time.now() len==10ish",
        r#"fn main() { print(Str.len(Time.now()) >= 9) }"#,
    );
    println!("{out}");
}

/// String.len（171）相邻行复核：行为按 resolve 槽 171（shim_string_len
/// 常量池臂=**字节数**，实测 "中文"→6）；peek=170 与 interface.resolve=170
/// 是另两个视角（canonical 归一先命中 auto.str.len 静态表）——三视角
/// 分歧登记进绑定表 §1。
#[test]
fn plan872_probe_string_len_adjacent() {
    let mut out = String::new();
    probe(
        &mut out,
        "String.len(\"中文\")",
        r#"fn main() { print(String.len("中文")) }"#,
    );
    println!("{out}");
}
