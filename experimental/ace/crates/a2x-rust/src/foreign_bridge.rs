//! PLAN-967 (AXR0 / T-12): the A2R foreign bridge for the frozen F0/rich
//! domain. The generated Rust consumes the SAME C-compatible shim exports
//! every other lane uses (the provider cdylib's `#[no_mangle]` symbols):
//! one `extern "C"` declaration plus an `A2xResult` wrapper per entry.
//! Fallible statuses cross as `AutoTrap::Foreign(status)` — the
//! error-model table stays the naming authority; panics stay contained
//! in the provider (the shim's flag protocol is the provider-side belt;
//! the A2R wrapper adds the process-level belt).
use ace_a2x_core::A2xReject;
use ace_foreign_contract::manual::SourceDecl;
use ace_foreign_contract::resource::{ResKind, ResourceForm};

/// The wrapper source for ONE foreign declaration. Out-of-domain shapes
/// keep the historical named rejection (`a2r.foreign-unsupported`).
pub fn wrapper_source(decl: &SourceDecl) -> Result<String, A2xReject> {
    let Some(res) = decl.resource() else {
        return Err(reject("the entry has no resource signature"));
    };
    // the frozen domain: params ⊆ {ViewText, I32, Bool} with AT MOST one
    // ViewText; forms Pure{I32|Bool} | Fallible{out:I32}
    let mut view = false;
    let mut scalars: Vec<&'static str> = Vec::new();
    for lane in &res.params {
        match lane {
            ResKind::ViewText => {
                if view {
                    return Err(reject("two view lanes"));
                }
                view = true;
            }
            ResKind::I32 => scalars.push("i32"),
            ResKind::Bool => scalars.push("bool"),
            _ => return Err(reject("a lane outside the frozen bridge domain")),
        }
    }
    if scalars.len() > 3 {
        return Err(reject("more than three scalar lanes"));
    }
    let (ret_c, wrap_ret, fallible) = match &res.form {
        ResourceForm::Pure {
            ret: ace_foreign_contract::ResOut::I32,
        } => ("i32", "i32", false),
        ResourceForm::Pure {
            ret: ace_foreign_contract::ResOut::Bool,
        } => ("u8", "bool", false),
        ResourceForm::Fallible {
            out: Some(ace_foreign_contract::ResOut::I32),
            ..
        } => ("i32", "i32", true),
        _ => return Err(reject("a result form outside the frozen bridge domain")),
    };
    let symbol = &decl.function.symbol;
    let wrapper_name = format!("a2x_foreign_{}", decl.name);
    let mut extern_params = String::new();
    let mut wrapper_params = String::new();
    let mut call_args = String::new();
    if view {
        extern_params.push_str("data: *const u8, byte_len: u64");
        wrapper_params.push_str("__view: &str");
        call_args.push_str("__view.as_ptr(), __view.len() as u64");
    }
    for (i, scalar) in scalars.iter().enumerate() {
        if !extern_params.is_empty() {
            extern_params.push_str(", ");
            call_args.push_str(", ");
        }
        extern_params.push_str(&format!("p{i}: {scalar}"));
        call_args.push_str(&format!("p{i}"));
    }
    if !view && scalars.is_empty() {
        // a zero-lane entry keeps a (void) C frame
        extern_params.push_str("_pad: u8");
        call_args.push_str("0");
        wrapper_params.push_str("_pad: u8");
    }
    if fallible {
        // the out lane is part of the extern frame (BEFORE the decl is
        // rendered — the wrapper appends &mut out at its call site)
        extern_params.push_str(", out: *mut i32");
    }
    let mut out = format!(
        "#[link(name = \"f0_text_provider\")]\nextern \"C\" {{ pub fn {symbol}({extern_params}) -> {ret_c}; }}\n"
    );
    let mut wrapper_params = wrapper_params.clone();
    for (i, scalar) in scalars.iter().enumerate() {
        if !wrapper_params.is_empty() {
            wrapper_params.push_str(", ");
        }
        wrapper_params.push_str(&format!("p{i}: {scalar}"));
    }
    if fallible {
        if !wrapper_params.is_empty() {
            wrapper_params.push_str(", ");
        }
        out.push_str(&format!(
            "pub fn {wrapper_name}({wrapper_params}) -> A2xResult<{wrap_ret}> {{\n    let mut out: i32 = 0;\n    let status = unsafe {{ {symbol}({call_args}, &mut out) }};\n    if status != 0 {{ return Err(AutoTrap::Foreign(status)); }}\n    Ok(out)\n}}\n"
        ));
    } else {
        let conv = if wrap_ret == "bool" {
            format!("unsafe {{ {symbol}({call_args}) }} != 0")
        } else {
            format!("unsafe {{ {symbol}({call_args}) }}")
        };
        out.push_str(&format!(
            "pub fn {wrapper_name}({wrapper_params}) -> A2xResult<{wrap_ret}> {{\n    Ok({conv})\n}}\n"
        ));
    }
    Ok(out)
}

fn reject(detail: &str) -> A2xReject {
    A2xReject::new(
        "a2r.foreign-unsupported",
        format!("Rust target does not execute source foreign calls: {detail}"),
    )
}
