use auto_lang::{compile::CompileSession, stdlib_assembly::{loader, model::Environment, validate}};
use std::fs;

fn names(s: &CompileSession) -> Vec<String> {
    s.type_store().read().unwrap().lookup_module("proto").unwrap().store.pub_fn_names()
}
fn main() {
    let original = std::env::current_dir().unwrap();
    let a = tempfile::tempdir().unwrap();
    fs::write(a.path().join("proto.at"), "pub fn answer() int {\n return 1\n}\n").unwrap();
    fs::write(a.path().join("proto.vm.at"), "pub fn old_layer() int {\n return 2\n}\n").unwrap();
    let mut s = CompileSession::new();
    s.add_source_dir(a.path().to_owned());
    s.resolve_uses("use proto\n").unwrap();
    fs::write(a.path().join("proto.vm.at"), "pub fn new_layer() int {\n return 3\n}\n").unwrap();
    let result = s.resolve_uses("use proto\n");
    println!("SAME_SESSION result={result:?} names={:?}", names(&s));

    let mut clone = s.clone();
    let b = tempfile::tempdir().unwrap();
    fs::write(b.path().join("proto.at"), "pub fn from_b() int {\n return 4\n}\n").unwrap();
    // Restore A to its cached bytes; B now has resolver precedence via CWD.
    fs::write(a.path().join("proto.vm.at"), "pub fn old_layer() int {\n return 2\n}\n").unwrap();
    std::env::set_current_dir(b.path()).unwrap();
    let result = clone.resolve_uses("use proto\n");
    println!("ROOT_SWITCH result={result:?} names={:?} selected={}", names(&clone), clone.layer_selections[0].public_file);
    std::env::set_current_dir(&original).unwrap();

    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join("proto.at"), "pub fn val() int {\n return 1\n}\n").unwrap();
    fs::write(d.path().join("proto.vm.at"), "pub fn val() int {\n return 2\n}\n").unwrap();
    let mut duplicate = CompileSession::new();
    duplicate.add_source_dir(d.path().to_owned());
    println!("DUPLICATE_BODIES result={:?}", duplicate.resolve_uses("use proto\n"));

    auto_lang::vm::native_registry::register_builtin_natives();
    let mut shims = auto_lang::vm::native::NativeInterface::new();
    shims.register_std_shims();
    auto_lang::vm::ffi::stdlib::register_stdlib_ffi(&mut shims);
    shims.build_from_inventory();
    let root = loader::repo_stdlib_root().unwrap();
    let mut inv = loader::scan_inventory(&root);
    let reg = auto_lang::vm::native_registry::BIGVM_NATIVES.lock().unwrap();
    let before = validate::validate_core_vm_bindings(&inv, &reg, &shims, Environment::Native);
    let sym = inv.modules.iter_mut().find(|m| m.module == "net").unwrap().layers.iter_mut()
        .flat_map(|l| &mut l.symbols).find(|s| s.name == "tcp_listener_close" && s.is_vm_decl).unwrap();
    sym.arity = 999;
    let after = validate::validate_core_vm_bindings(&inv, &reg, &shims, Environment::Native);
    println!("SIGNATURE before={:?} after={:?}", before.iter().find(|v| v.native_name == "auto.net.tcp_listener_close"), after.iter().find(|v| v.native_name == "auto.net.tcp_listener_close"));
    drop(reg);

    let fp_root = tempfile::tempdir().unwrap();
    fs::write(fp_root.path().join("m.at"), "pub fn f() int;\n").unwrap();
    fs::write(fp_root.path().join("m.vm.at"), "pub fn f() int {\n return 1\n}\n").unwrap();
    let fp_before = loader::stdlib_assembly_fingerprint(fp_root.path(), auto_lang::stdlib_assembly::model::AssemblyTarget::Vm).unwrap();
    fs::rename(fp_root.path().join("m.vm.at"), fp_root.path().join("m.rs.at")).unwrap();
    let fp_after = loader::stdlib_assembly_fingerprint(fp_root.path(), auto_lang::stdlib_assembly::model::AssemblyTarget::Vm).unwrap();
    println!("FINGERPRINT_LAYER_KIND before={fp_before:016x} after={fp_after:016x} equal={}", fp_before == fp_after);

    let bad = tempfile::tempdir().unwrap();
    fs::write(bad.path().join("bad.at"), [0xff, 0xfe]).unwrap();
    let unreadable = loader::scan_inventory(bad.path());
    println!("UNREADABLE_INVENTORY files_total={} modules={} diagnostics={:?}", unreadable.files_total, unreadable.modules.len(), unreadable.diagnostics);

    let error_dir = tempfile::tempdir().unwrap();
    fs::write(error_dir.path().join("proto.at"), "// one\n// two\n// three\npub fn f() int;\n").unwrap();
    fs::write(error_dir.path().join("proto.vm.at"), "pub fn broken( [[[;\n").unwrap();
    let mut errors = CompileSession::new();
    errors.add_source_dir(error_dir.path().to_owned());
    println!("SOURCE_ERROR result={:?}", errors.resolve_uses("use proto\n"));
    if std::env::args().any(|a| a == "--quick") { return; }

    let target = tempfile::tempdir().unwrap();
    let file = target.path().join("main.at");
    fs::write(&file, "use auto.net\nfn main() {\n let x = 1\n}\n").unwrap();
    let mut rust = CompileSession::new();
    println!("RUST_UNSUPPORTED result={:?}", auto_lang::trans_rust_with_session(&mut rust, file.to_str().unwrap()));
    println!("RUST_MANIFEST count={} emits_missing_provider={}", rust.layer_selections.len(), fs::read_to_string(file.with_file_name("main.a2r.rs")).unwrap().contains("a2r_std::net"));
    let mut c = CompileSession::new();
    println!("C_UNSUPPORTED result={:?}", auto_lang::trans_c_with_session(&mut c, file.to_str().unwrap()));
    println!("C_MANIFEST count={}", c.layer_selections.len());
}
