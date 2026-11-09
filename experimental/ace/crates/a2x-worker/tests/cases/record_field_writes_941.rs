//! F-941-05: real compilation of complete field-writing programs, including
//! independent references. Source checks and reference checks are distinct.
use super::rt_common::{compile_case, Prof};
use super::scalar_release_941::{bytes, capture, hex, plain, save, successful};
use ace_a2x_core::sha256::sha256_hex;
use ace_a2x_core::SemanticView;
use ace_a2x_rust::{assemble_record_text_file_set, lower_program};
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

fn compare(name: &str, source: &str, profile: Prof, reference: &str, expected: i32) {
    let retained = std::env::var_os("AUTO_PLAN_ARTIFACTS").map(PathBuf::from);
    let home = retained.clone().unwrap_or_else(std::env::temp_dir);
    plain(&home);
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = home.join(format!(
        "field-writes-941-{name}-{}-{stamp}",
        std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    println!("field-write evidence: {}", root.display());
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    for optimized in [false, true] {
        let whole = Instant::now();
        let flavor = if optimized { "optimized" } else { "default" };
        let dir = root.join(flavor);
        let actual = dir.join("actual");
        let independent = dir.join("reference");
        std::fs::create_dir_all(&actual).unwrap();
        std::fs::create_dir(&independent).unwrap();
        let compiled = compile_case(name, source, profile);
        let view = SemanticView::from_checked(&compiled.checked).unwrap();
        let program = lower_program(&view, "main").unwrap();
        let files = assemble_record_text_file_set(&view, &program, &compiled.input_hash, "main");
        for (path, data) in &files.files {
            std::fs::write(actual.join(path), data).unwrap();
        }
        std::fs::write(actual.join("manifest.json"), &files.manifest).unwrap();
        std::fs::write(independent.join("main.rs"), reference).unwrap();
        let mut record = json!({"schema":"ace.m1.field-write-regression/1","case":name,
            "source_sha256":sha256_hex(source.as_bytes()),"reference_sha256":sha256_hex(reference.as_bytes()),
            "optimized":optimized,"expected":expected,"files":{},"processes":{}});
        for file in ["main.rs", "generated.rs", "a2x_runtime.rs", "manifest.json"] {
            record["files"][file] = sha256_hex(&bytes(&actual.join(file))).into();
        }
        let receipt = dir.join("receipt.json");
        for (leg, work) in [("actual", &actual), ("reference", &independent)] {
            let exe = work.join("program.exe");
            let mut compile = Command::new(&rustc);
            compile.args(["--edition", "2021", "-D", "unused_mut"]);
            if optimized {
                compile.arg("-O");
            }
            compile.arg("main.rs").arg("-o").arg(&exe).current_dir(work);
            record["processes"][leg] = json!({"compile":capture(&mut compile,120_000,whole)});
            save(&receipt, &record);
            successful(&record["processes"][leg]["compile"]);
            let mut run = Command::new(&exe);
            run.current_dir(work);
            record["processes"][leg]["run"] = capture(&mut run, 30_000, whole);
            save(&receipt, &record);
            successful(&record["processes"][leg]["run"]);
            assert_eq!(
                record["processes"][leg]["run"]["terminal"]["stdout_hex"],
                hex(format!("value:{expected}\n").as_bytes())
            );
            assert_eq!(
                record["processes"][leg]["run"]["terminal"]["stderr_hex"],
                ""
            );
        }
    }
    if retained.is_none() {
        plain(&root);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn original_m01_copy_and_method_field_write() {
    compare("M01", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../crates/auto-lang/test/cases/ace-m1/src/M01.at")), Prof::Record,
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../crates/auto-lang/test/cases/ace-m1-expanded/original-data-reference/M01.rs")), 0);
}
#[test]
fn original_r01_field_write_preserves_copy() {
    compare("R01", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../crates/auto-lang/test/cases/ace-m1/src/R01.at")), Prof::Record,
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../crates/auto-lang/test/cases/ace-m1-expanded/original-data-reference/R01.rs")), 19);
}
#[test]
fn original_r02_nested_field_write() {
    compare("R02", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../crates/auto-lang/test/cases/ace-m1/src/R02.at")), Prof::Record,
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../crates/auto-lang/test/cases/ace-m1-expanded/original-data-reference/R02.rs")), 9);
}
#[test]
fn branch_loop_writes_and_readonly_copy() {
    compare("branch-loop", r#"type P { v int }
fn main() int { var p = P { v: 0 }; let old = p
for p.v < 3 { p.v = p.v + 1 }
if old.v != 0 { return 99 }
if p.v == 3 { p.v = p.v + 1 }
return p.v }"#, Prof::Record,
        "#[derive(Clone,Copy)]struct P{v:i32} fn main(){let mut p=P{v:0};let old=p;while p.v<3{p.v+=1;}if old.v!=0{println!(\"value:99\");return}if p.v==3{p.v+=1;}println!(\"value:{}\",p.v);}",4);
}
#[test]
fn borrowed_record_parameter_writes_without_mut_binding() {
    compare("borrowed-record", r#"type P { v int }
fn read(p P) int { return p.v }
fn update(mut p P) int { p.v = p.v + 2; return p.v }
fn main() int { var p = P { v: 40 }; let before = read(p); let after = update(p.mut); return before + p.v + after }"#, Prof::Record,
        "struct P{v:i32} fn read(p:&P)->i32{p.v} fn update(p:&mut P)->i32{p.v+=2;p.v} fn main(){let mut p=P{v:40};let before=read(&p);let after=update(&mut p);println!(\"value:{}\",before+p.v+after);}",124);
}
#[test]
fn move_owned_record_parameter_keeps_value_root_and_cleanup() {
    compare("move-record", r#"type Item { text Str; v int }
fn consume(move p Item) int { p.v = 41; return p.v + p.text.len() }
fn main() int { let p = Item { text: "x", v: 0 }; return consume(p.move) }"#, Prof::TextRecord,
        "struct Item{text:String,v:i32} fn consume(mut p:Item)->i32{p.v=41;p.v+p.text.chars().count() as i32} fn main(){let p=Item{text:\"x\".into(),v:0};println!(\"value:{}\",consume(p));}",42);
}
#[test]
fn view_and_same_root_rejections_stay_in_shared_verifier() {
    let view = "type P { v int } fn bad(p P) int { p.v = 1; return p.v } fn main() int { let p = P { v: 0 }; return bad(p) }";
    let errors = ace_core::source::compile_record_source(view, "view-write.at", &[]).unwrap_err();
    assert!(errors.iter().any(|d| d.code == "verify.immutable-write"));
    let alias = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../crates/auto-lang/test/cases/ace-m1/src/M11.at"
    ));
    let errors = ace_core::source::compile_record_source(alias, "alias-write.at", &[]).unwrap_err();
    assert!(errors.iter().any(|d| d.code == "verify.mut-conflict"));
}

// Controlled pre-fix provider: print/2 plus the exact missing-mut shape from
// the original R01 failure. The real persistent store, not a key-only mock,
// must reject reuse of these fragments under the current print version.
struct Pre941Provider;
impl ace_a2x_core::target::TargetProvider for Pre941Provider {
    fn descriptor(&self) -> ace_a2x_core::target::TargetDescriptor {
        ace_a2x_rust::RustTargetProvider::v1().descriptor()
    }
    fn fragment_key_inputs(&self, hash: &str) -> Vec<(String, String)> {
        ace_a2x_rust::RustTargetProvider::v1()
            .fragment_key_inputs(hash)
            .into_iter()
            .map(|(k, v)| {
                if k == "print_version" {
                    (k, "a2x-print/2".into())
                } else {
                    (k, v)
                }
            })
            .collect()
    }
    fn fragment_layer_id(
        &self,
        f: ace_a2x_core::target::Family,
    ) -> Result<&'static str, ace_a2x_core::A2xReject> {
        ace_a2x_rust::RustTargetProvider::v1().fragment_layer_id(f)
    }
    fn runtime_source(
        &self,
        v: &SemanticView<'_>,
        f: ace_a2x_core::target::Family,
    ) -> Result<String, ace_a2x_core::A2xReject> {
        ace_a2x_rust::RustTargetProvider::v1().runtime_source(v, f)
    }
    fn produce_fragments(
        &self,
        v: &SemanticView<'_>,
        fs: &[&ace_a2x_core::FnFacts<'_>],
        f: ace_a2x_core::target::Family,
    ) -> Result<Vec<String>, ace_a2x_core::A2xReject> {
        let fragments = ace_a2x_rust::RustTargetProvider::v1().produce_fragments(v, fs, f)?;
        assert_eq!(
            fragments
                .iter()
                .map(|s| s.matches("let mut l0_p:").count())
                .sum::<usize>(),
            1
        );
        Ok(fragments
            .into_iter()
            .map(|s| s.replace("let mut l0_p:", "let l0_p:"))
            .collect())
    }
    fn capture_prelude_seed(
        &self,
        v: &SemanticView<'_>,
        f: ace_a2x_core::target::Family,
    ) -> Result<Option<String>, ace_a2x_core::A2xReject> {
        ace_a2x_rust::RustTargetProvider::v1().capture_prelude_seed(v, f)
    }
    fn assemble_fileset(
        &self,
        r: ace_a2x_core::target::FilesetRequest<'_>,
    ) -> Result<ace_a2x_core::target::FileSet, ace_a2x_core::A2xReject> {
        ace_a2x_rust::RustTargetProvider::v1().assemble_fileset(r)
    }
}

#[test]
fn pre941_print_cache_misses_once_then_reuses_correct_fragment() {
    let retained = std::env::var_os("AUTO_PLAN_ARTIFACTS").map(PathBuf::from);
    let home = retained.clone().unwrap_or_else(std::env::temp_dir);
    plain(&home);
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = home.join(format!("field-cache-941-{}-{stamp}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    println!("field-cache evidence: {}", root.display());
    let store = root.join("store");
    std::fs::create_dir(&store).unwrap();
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../crates/auto-lang/test/cases/ace-m1/src/R01.at"
    ));
    let mut receipt = json!({"schema":"ace.m1.field-cache-regression/1","source_sha256":sha256_hex(source.as_bytes()),"old_print":"a2x-print/2","current_print":ace_a2x_rust::RUST_PRINT_VERSION,"generations":[]});
    let whole = Instant::now();
    let old = {
        let mut session = ace_a2x_store::A2xSession::open(
            &store,
            "a2x-probe",
            "probe",
            128 * 1024 * 1024,
            Pre941Provider,
        )
        .unwrap();
        let asm = session
            .cold_assemble_profile(
                source,
                ace_core::source::SourceProfile::Record,
                &[],
                "main",
                false,
            )
            .unwrap();
        assert_eq!(session.stats().lower, 1);
        asm
    };
    let mut previous = None;
    for generation in 0..3 {
        let (asm, lower) = if generation == 0 {
            (old.clone(), 1)
        } else {
            let mut session = ace_a2x_store::A2xSession::open(
                &store,
                "a2x-probe",
                "probe",
                128 * 1024 * 1024,
                ace_a2x_rust::RustTargetProvider::v1(),
            )
            .unwrap();
            let asm = session
                .cold_assemble_profile(
                    source,
                    ace_core::source::SourceProfile::Record,
                    &[],
                    "main",
                    false,
                )
                .unwrap();
            (asm, session.stats().lower)
        };
        assert_eq!(
            lower,
            if generation == 2 { 0 } else { 1 },
            "old miss then stable hit"
        );
        let dir = root.join(format!("generation-{generation}"));
        std::fs::create_dir(&dir).unwrap();
        let mut files = json!({});
        for (name, text) in &asm.files {
            std::fs::write(dir.join(name), text).unwrap();
            files[name] = sha256_hex(text.as_bytes()).into();
        }
        std::fs::write(dir.join("manifest.json"), &asm.manifest).unwrap();
        files["manifest.json"] = sha256_hex(asm.manifest.as_bytes()).into();
        if generation == 2 {
            assert_eq!(
                previous.as_ref(),
                Some(&files),
                "cache hit preserves all current bytes"
            );
        }
        if generation == 1 {
            previous = Some(files.clone());
        }
        let exe = dir.join("program.exe");
        let mut compile = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into()));
        compile
            .args(["--edition", "2021", "-D", "unused_mut", "main.rs", "-o"])
            .arg(&exe)
            .current_dir(&dir);
        let compilation = capture(&mut compile, 120_000, whole);
        let mut row =
            json!({"generation":generation,"lower":lower,"files":files,"compile":compilation});
        receipt["generations"]
            .as_array_mut()
            .unwrap()
            .push(row.clone());
        save(&root.join("receipt.json"), &receipt);
        if generation == 0 {
            assert_eq!(
                compilation["terminal"]["exit_code"], 1,
                "pre-fix shape must fail real rustc"
            );
            assert!(compilation["terminal"]["stderr_hex"]
                .as_str()
                .unwrap()
                .contains(&hex(b"E0594")));
        } else {
            successful(&compilation);
            let mut run = Command::new(&exe);
            run.current_dir(&dir);
            row["run"] = capture(&mut run, 30_000, whole);
            receipt["generations"][generation] = row.clone();
            save(&root.join("receipt.json"), &receipt);
            successful(&row["run"]);
            assert_eq!(row["run"]["terminal"]["stdout_hex"], hex(b"value:19\n"));
            assert_eq!(row["run"]["terminal"]["stderr_hex"], "");
        }
    }
    if retained.is_none() {
        plain(&root);
        std::fs::remove_dir_all(root).unwrap();
    }
}
