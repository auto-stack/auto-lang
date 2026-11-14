//! PLAN-967 (T-12 / A2PY ctypes leg): the Python foreign bridge for the
//! frozen F0/rich domain. The generated Python loads the SAME provider
//! cdylib every other lane uses through `ctypes.CDLL` and calls the
//! `#[no_mangle]` shim exports (view lanes as `(c_char_p data,
//! c_uint64 len)`; fallible statuses cross as a typed exception carrying
//! the status code — the error-model table stays the naming authority).
//! Shapes outside the frozen domain keep the historical named rejection
//! `a2py.foreign-authority`.
use ace_a2x_core::A2xReject;
use ace_foreign_contract::manual::SourceDecl;
use ace_foreign_contract::resource::{ResKind, ResourceForm};

/// The bridge source for ALL foreign declarations of a view. Empty when
/// the view carries none.
pub fn bridge_source(view: &ace_a2x_core::SemanticView<'_>) -> Result<String, A2xReject> {
    let mut out = String::new();
    let mut count = 0usize;
    for decl in view.foreign_declarations() {
        out.push_str(&wrapper_source(decl)?);
        count += 1;
    }
    if count > 0 {
        // Python is indentation-sensitive: the continuation-line escapes
        // carry explicit spaces (\x20 runs) for the class body.
        let header = concat!(
            "# PLAN-967 (A2PY ctypes leg): generated foreign bridge —\n",
            "# the SAME provider cdylib the other lanes load.\n",
            "import ctypes as _a2x_ct\n",
            "import ctypes.wintypes as _a2x_wt\n",
            "from ctypes import c_char_p, c_uint64, c_int, c_bool, POINTER, byref\n",
            "import os as _a2x_os\n",
            "\n",
            "# the provider DLL rides the fileset directory (an absolute-path\n",
            "# load — the CWD is not on the safe DLL search path)\n",
            "_A2X_F0_DLL_PATH = _a2x_os.path.join(\n",
            "    _a2x_os.path.dirname(_a2x_os.path.abspath(__file__)),\n",
            "    \"f0_text_provider.dll\",\n",
            ")\n",
            "try:\n",
            "    _A2X_F0_DLL = _a2x_ct.CDLL(_A2X_F0_DLL_PATH)\n",
            "except OSError as _e:\n",
            "    raise RuntimeError(\"a2py foreign: provider DLL missing\") from _e\n",
            "\n",
            "class A2xForeignStatus(Exception):\n",
            "    \"\"\"A foreign fallible status (PLAN-967): the error-model table names it.\"\"\"\n",
            "    def __init__(self, status):\n",
            "        super().__init__(f\"foreign.status={status}\")\n",
            "        self.status = status\n",
            "\n",
        );
        out.insert_str(0, &header);
    }
    Ok(out)
}

/// The wrapper for ONE declaration; out-of-domain shapes keep the named
/// rejection.
pub fn wrapper_source(decl: &SourceDecl) -> Result<String, A2xReject> {
    let Some(res) = decl.resource() else {
        return Err(reject("the entry has no resource signature"));
    };
    let mut view_lane = false;
    let mut scalars: Vec<&'static str> = Vec::new();
    for lane in &res.params {
        match lane {
            ResKind::ViewText => {
                if view_lane {
                    return Err(reject("two view lanes"));
                }
                view_lane = true;
            }
            ResKind::I32 => scalars.push("c_int"),
            ResKind::Bool => scalars.push("c_bool"),
            _ => return Err(reject("a lane outside the frozen bridge domain")),
        }
    }
    if scalars.len() > 3 {
        return Err(reject("more than three scalar lanes"));
    }
    let (ret_c, fallible) = match &res.form {
        ResourceForm::Pure {
            ret: ace_foreign_contract::ResOut::I32,
        } => ("c_int", false),
        ResourceForm::Pure {
            ret: ace_foreign_contract::ResOut::Bool,
        } => ("c_bool", false),
        ResourceForm::Fallible {
            out: Some(ace_foreign_contract::ResOut::I32),
            ..
        } => ("c_int", true),
        _ => return Err(reject("a result form outside the frozen bridge domain")),
    };
    let symbol = &decl.function.symbol;
    let wrapper_name = format!("a2x_foreign_{}", decl.name);
    let mut argtypes = vec!["c_char_p".to_string(), "c_uint64".to_string()];
    let mut params = vec!["__s: bytes".to_string()];
    let mut call_args = vec!["__s".to_string(), "len(__s)".to_string()];
    if !view_lane {
        argtypes.clear();
        params.clear();
        call_args.clear();
    }
    for (i, scalar) in scalars.iter().enumerate() {
        argtypes.push(scalar.to_string());
        params.push(format!("p{i}"));
        call_args.push(format!("p{i}"));
    }
    if fallible {
        argtypes.push("POINTER(c_int)".to_string());
    }
    let mut out = format!(
        "_a2x_f0_{symbol} = _A2X_F0_DLL.{symbol}\n\
         _a2x_f0_{symbol}.argtypes = [{args}]\n\
         _a2x_f0_{symbol}.restype = {ret_c}\n\n",
        args = argtypes.join(", "),
    );
    if fallible {
        out.push_str(&format!(
            "def {wrapper_name}({params}):\n    __out = _a2x_ct.c_int(0)\n    __status = _a2x_f0_{symbol}({call_args}, _a2x_ct.byref(__out))\n    if __status != 0:\n        raise A2xForeignStatus(__status)\n    return __out.value\n\n",
            params = params.join(", "),
            call_args = call_args.join(", "),
        ));
    } else {
        out.push_str(&format!(
            "def {wrapper_name}({params}):\n    return _a2x_f0_{symbol}({call_args})\n\n",
            params = params.join(", "),
            call_args = call_args.join(", "),
        ));
    }
    Ok(out)
}

fn reject(detail: &str) -> A2xReject {
    A2xReject::new(
        "a2py.foreign-authority",
        format!("foreign FFI declarations require explicit C/Rust FFI provider: {detail}"),
    )
}
