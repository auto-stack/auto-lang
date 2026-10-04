//! T-01 toolchain probe: emit a minimal COFF object via Cranelift and verify
//! that it can be linked into a Windows PE and executed.
//!
//! The object exports two functions:
//! - `probe_add(a: i32, b: i32) -> i32` — real compiled code (i32 add)
//! - `ac_start` — minimal startup wrapper calling `probe_add(2, 3)` then `ExitProcess`
//!
//! Link with `/entry:ac_start /subsystem:console <obj> kernel32.lib` and run:
//! a healthy toolchain exits with code 5.

use anyhow::{bail, Context, Result};
use cranelift_codegen::ir::{types, AbiParam, InstBuilder, Signature};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::settings::Configurable;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_module::{default_libcall_names, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::path::PathBuf;
use std::process::Command;
use target_lexicon::Triple;

const WINCALL: CallConv = CallConv::WindowsFastcall;

fn i32_params(n: usize) -> Signature {
    Signature {
        call_conv: WINCALL,
        params: vec![AbiParam::new(types::I32); n],
        returns: vec![AbiParam::new(types::I32)],
    }
}

fn main() -> Result<()> {
    let out_dir: PathBuf = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let obj_path = out_dir.join("probe_min.obj");

    // --- emit object ---
    let triple = Triple::host();
    let mut flag_builder = cranelift_codegen::settings::builder();
    flag_builder.set("opt_level", "none").unwrap();
    flag_builder.set("is_pic", "false").unwrap();
    let flags = cranelift_codegen::settings::Flags::new(flag_builder);
    let isa = cranelift_codegen::isa::lookup(triple)
        .expect("host isa")
        .finish(flags)
        .expect("isa finish");
    let obj_builder = ObjectBuilder::new(isa, "ac_probe", default_libcall_names())
        .expect("object builder");
    let mut module = ObjectModule::new(obj_builder);

    // import: void ExitProcess(i32)
    let exit_sig = Signature {
        call_conv: WINCALL,
        params: vec![AbiParam::new(types::I32)],
        returns: vec![],
    };
    let exit_id = module
        .declare_function("ExitProcess", Linkage::Import, &exit_sig)
        .context("declare ExitProcess")?;

    // export: i32 probe_add(i32, i32)
    let add_sig = i32_params(2);
    let add_id = module
        .declare_function("probe_add", Linkage::Export, &add_sig)
        .context("declare probe_add")?;
    let mut ctx = module.make_context();
    ctx.func.signature = add_sig.clone();
    {
        let mut fb_ctx = FunctionBuilderContext::new();
        let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let entry = fb.create_block();
        fb.append_block_params_for_function_params(entry);
        fb.switch_to_block(entry);
        fb.seal_block(entry);
        let (pa, pb) = (fb.block_params(entry)[0], fb.block_params(entry)[1]);
        let sum = fb.ins().iadd(pa, pb);
        fb.ins().return_(&[sum]);
        fb.finalize();
    }
    module.define_function(add_id, &mut ctx).context("define probe_add")?;

    // export: () ac_start() — calls probe_add(2, 3), then ExitProcess(result)
    let start_sig = Signature {
        call_conv: WINCALL,
        params: vec![],
        returns: vec![],
    };
    let start_id = module
        .declare_function("ac_start", Linkage::Export, &start_sig)
        .context("declare ac_start")?;
    let mut ctx = module.make_context();
    ctx.func.signature = start_sig.clone();
    {
        let mut fb_ctx = FunctionBuilderContext::new();
        let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let entry = fb.create_block();
        fb.switch_to_block(entry);
        fb.seal_block(entry);
        let two = fb.ins().iconst(types::I32, 2);
        let three = fb.ins().iconst(types::I32, 3);
        let add_ref = module.declare_func_in_func(add_id, &mut fb.func);
        let call = fb.ins().call(add_ref, &[two, three]);
        let result = fb.inst_results(call)[0];
        let exit_ref = module.declare_func_in_func(exit_id, &mut fb.func);
        fb.ins().call(exit_ref, &[result]);
        fb.ins().return_(&[]);
        fb.finalize();
    }
    module.define_function(start_id, &mut ctx).context("define ac_start")?;

    let product = module.finish().emit().context("emit COFF object")?;
    std::fs::write(&obj_path, &product).context("write probe_min.obj")?;
    println!("wrote {}", obj_path.display());

    // --- link with lld-link (rust-lld COFF mode), fall back to MSVC link.exe ---
    let exe_path = out_dir.join("probe_min.exe");
    let lld = find_rust_lld().context("rust-lld.exe not found")?;
    let sdk_lib = r"D:\Windows Kits\10\Lib\10.0.26100.0\um\x64";
    let lld_args: Vec<String> = vec![
        "-flavor".into(),
        "link".into(),
        "/nologo".into(),
        "/entry:ac_start".into(),
        "/subsystem:console".into(),
        format!("/out:{}", exe_path.display()),
        format!("{}", obj_path.display()),
        format!("/libpath:{}", sdk_lib),
        "kernel32.lib".into(),
    ];
    let link_status = Command::new(&lld)
        .args(&lld_args)
        .status()
        .context("run rust-lld")?;
    if !link_status.success() {
        bail!("rust-lld link failed: {}", link_status);
    }
    println!("linked with {}", lld.display());

    // --- run; expect exit code 5 (2 + 3) ---
    let run = Command::new(&exe_path).status().context("run probe exe")?;
    let code = run.code().context("no exit code")?;
    println!("probe_min.exe exited {}", code);
    if code != 5 {
        bail!("expected exit code 5, got {}", code);
    }
    println!("TOOLCHAIN PROBE PASS");
    Ok(())
}

fn find_rust_lld() -> Result<PathBuf> {
    let sysroot = String::from_utf8(
        Command::new("rustc")
            .arg("--print")
            .arg("sysroot")
            .output()
            .context("rustc --print sysroot")?
            .stdout,
    )?;
    let trimmed = sysroot.trim();
    let candidate = PathBuf::from(trimmed)
        .join("lib/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe");
    if candidate.is_file() {
        return Ok(candidate);
    }
    bail!("rust-lld.exe not found under sysroot {}", trimmed)
}
