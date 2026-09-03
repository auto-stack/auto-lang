//! Plan 531 W0b/W2: aavm.at a2r 模式入口红证 + 回归锚。
//!
//! 形态:auto/aavm.at(位置参数入口)经主 a2r 平铺 merge 转译 + 原生 shim
//! 后 cargo build,产物即"转译版 aavm"。两条验收(计划口径):
//! - b07_fib 位置参数直达 55(P523-2①:argv.get 解包修复前 E0308 红);
//! - b34_struct_basic 运行 10/20(P523-2②:转译版 struct 字段表修复前
//!   RUNTIME-ERROR:no field x in Point 红)。
//!
//! `#[ignore]`(shells cargo),按需:
//! `cargo test -p auto-lang --lib --features test-vm-files aavm_at_mode -- --ignored --nocapture`

use std::path::PathBuf;

fn aavm_at_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("auto")
}

/// 剥除 use 行的临时目录(stripped lib + aavm.at)——merge 输入。
/// 镜像 vm_file_tests::build_aavm_rust_bin 的剥离规则。
fn stripped_merge_dir() -> PathBuf {
    let root = aavm_at_dir().parent().unwrap().to_path_buf();
    let stripped = std::env::temp_dir().join(format!("p531-aavm-at-stripped-p{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&stripped);
    std::fs::create_dir_all(&stripped).expect("mkdir p531 stripped");
    for file in crate::AUTO_LIB_FILES_V2 {
        let content = std::fs::read_to_string(root.join(file)).expect("read lib file");
        let mut out = String::new();
        for line in content.lines() {
            if line.trim_start().starts_with("use auto.lib.") {
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        let name = std::path::Path::new(file).file_name().unwrap().to_str().unwrap().to_string();
        std::fs::write(stripped.join(name), out).expect("write stripped lib");
    }
    let aavm = std::fs::read_to_string(aavm_at_dir().join("aavm.at")).expect("read aavm.at");
    let mut out = String::new();
    for line in aavm.lines() {
        if line.trim_start().starts_with("use auto.lib.") {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    std::fs::write(stripped.join("aavm.at"), out).expect("write stripped aavm.at");
    stripped
}

/// 原生 shim(镜像 523 W4-14 实测形态):a2r 转译侧无原生映射的宿主原语。
/// - process.args():List 契约(P524;VM 参考侧 shim_process_args 同形=
///   [程序路径]+透传)。
/// - IO.read_line() / str.parse_int():stdin 行读取与十进制解析。
/// - a2r_std::value_len:merge 模式 List.len() 的映射位。
const NATIVE_SHIMS: &str = r#"
#[allow(dead_code)]
struct ProcessShim;
#[allow(dead_code)]
impl ProcessShim {
    // 发射形 `process.args()`=值位方法调用:const 实例承载
    // (mod/单元结构体关联函数在 `process.args()` 位是 E0423/E0599)。
    pub fn args(&self) -> Vec<String> { std::env::args().collect() }
}
#[allow(dead_code)]
const process: ProcessShim = ProcessShim;
#[allow(dead_code)]
mod IO {
    pub fn read_line() -> String {
        let mut buf = String::new();
        match std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut buf) {
            Ok(_) => buf.trim_end_matches('\n').trim_end_matches('\r').to_string(),
            Err(_) => String::new(),
        }
    }
}
#[allow(dead_code)]
trait A2rParseInt { fn parse_int(&self) -> i64; }
impl A2rParseInt for String {
    fn parse_int(&self) -> i64 { self.trim().parse::<i64>().unwrap_or(0) }
}
#[allow(dead_code)]
mod a2r_std {
    pub fn value_len<T>(v: &Vec<T>) -> i64 { v.len() as i64 }
}
"#;

/// 构建"转译版 aavm"main.rs 全文(merge 目录入口=剥离 lib+aavm.at 全集 +
/// 原生 shim;argv_patch=文本垫片,红证阶段复现 523 产品位垫片,修复后 None)。
fn build_aavm_at_main(argv_patch: Option<&str>) -> String {
    let stripped = stripped_merge_dir();
    // 目录入口:merge 装入全部 .at(lib 引擎件+aavm.at 入口);文件入口
    // 只经 use 发现——use 行已剥,单装 aavm.at(lib 符号全缺)。
    let merged = crate::trans::rust::transpile_rust_project_merged(
        stripped.to_str().expect("utf8"),
    )
    .expect("a2r merge transpile aavm.at");
    let mut full = String::from_utf8_lossy(&merged).to_string();
    full.push_str(NATIVE_SHIMS);
    if let Some(patch) = argv_patch {
        let (from, to) = patch.split_once("=>").expect("patch form: from=>to");
        let before = full.clone();
        full = full.replace(from.trim(), to.trim());
        assert!(full != before, "argv patch did not apply: {}", patch);
    }
    full
}

fn cargo_build(main_rs: &str) -> (bool, String, PathBuf) {
    let proj = std::env::temp_dir().join(format!("p531-aavm-at-bin-p{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&proj);
    let src = proj.join("src");
    std::fs::create_dir_all(&src).expect("mkdir src");
    std::fs::write(proj.join("Cargo.toml"),
        "[package]\nname = \"aavm_at\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n\n[dependencies]\n").unwrap();
    std::fs::write(src.join("main.rs"), main_rs).unwrap();
    let out = std::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&proj)
        .output()
        .expect("cargo spawn");
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    (out.status.success(), stderr, proj.join("target/release/aavm_at.exe"))
}

fn run_bin(exe: &std::path::Path, arg: &std::path::Path) -> String {
    let out = std::process::Command::new(exe).arg(arg).output().expect("run aavm_at");
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn corpus(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test/vm/aavm2/corpus_m4")
        .join(name)
}

/// P523-2① 红证/回归:无垫片构建转译版 aavm(修复后 b07→55)。
#[test]
#[ignore = "shells cargo; aavm.at a2r-mode acceptance (Plan 531)"]
fn aavm_at_mode_b07_positional() {
    let main_rs = build_aavm_at_main(None);
    let (ok, stderr, exe) = cargo_build(&main_rs);
    assert!(
        ok,
        "aavm.at a2r-mode cargo build failed (P523-2① argv.get?)\n{}",
        stderr
    );
    let out = run_bin(&exe, &corpus("b07_fib.at"));
    assert_eq!(out.trim_end(), "55", "b07 positional run diverged");
}

/// P523-2② 红证/回归:b34 struct 语料经转译版 aavm 运行。
/// 修复前红证链:洞①( argv.get E0308)先以 523 同款文本垫片桥接,拿到
/// 运行期 `RUNTIME-ERROR:no field x in Point`(洞②);两洞修复后无垫片
/// 直达 10/20(垫片 replace 不中=已修复,自然走无垫片路径)。
#[test]
#[ignore = "shells cargo; aavm.at a2r-mode acceptance (Plan 531)"]
fn aavm_at_mode_b34_struct() {
    let main_rs = build_aavm_at_main(None);
    let (ok, stderr, mut exe) = cargo_build(&main_rs);
    if !ok {
        eprintln!("[p531 red evidence: P523-2① unpatched build fail]\n{}", stderr);
        // 523 产品位文本垫片形态:Option 形态 → 索引形(argv: Vec<String>)
        let patched = build_aavm_at_main(Some(
            "ev_run_files(argv.get(1)) => ev_run_files(&argv[1])",
        ));
        let (ok2, stderr2, exe2) = cargo_build(&patched);
        assert!(
            ok2,
            "aavm.at a2r-mode cargo build failed even with argv patch\n{}",
            stderr2
        );
        exe = exe2;
    }
    let out = run_bin(&exe, &corpus("b34_struct_basic.at"));
    assert_eq!(out.trim_end(), "10\n20", "b34 struct run diverged (P523-2②?)");
}
