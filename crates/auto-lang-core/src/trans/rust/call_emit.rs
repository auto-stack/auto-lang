//! PLAN-843 结构拆分（自 rust.rs 原样搬移；算法/行为零变化）。
use super::*;

mod call_bare_builtins;
mod call_bina_methods;
mod call_dot_methods;
mod call_generic_fallback;
mod call_macro_shapes;
mod call_module_methods;
pub(crate) use call_module_methods::{
    emit_http_verb_call, expr_root_is_builder_var, expr_root_is_http_request,
    for_range_is_http_stream,
};
mod call_tag_ctor;
mod call_task;
mod call_value_ctors;

/// PLAN-884 T-05：call_inner 分支族 helper 的显式三态——旧代码用同一个
/// `return Ok(())` 表达"已发射"与守卫落穿两种含义，此处强制区分：
/// `Miss` = 调用形状不属于本族（落穿下一族）；`Handled` = 已发射（立即
/// 返回）；`Err` = 发射/校验失败。末族为无条件终止（无总 miss）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CallShape {
    Miss,
    Handled,
}

impl RustTrans {
    pub(super) fn call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        let mut emitted = Vec::new();
        self.call_inner(call, &mut emitted)?;
        let text = std::str::from_utf8(&emitted)
            .map_err(|e| e.to_string())?
            .to_string();
        // R7-03：实参面从 AST 序列化（部分直发臂的文本丢参——AST 是
        // 真实调用事实）。
        let args_text = self.serialize_positional_args(call);
        self.collect_core_reference(call, &text, &args_text)?;
        out.write_all(&emitted)?;
        Ok(())
    }

    fn call_inner(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        // PLAN-884 T-05：原顺序 if-let 链（22 个顶层阶段，审计图
        // attachments/884/t01-audit-call.md）按族拆入 call_emit/* 主题模块。
        // 此处只保留审计过的派发序：首匹配即返回，Miss 落穿下一族；
        // 末族为无条件终止（无总 miss）。不得重排。
        if self.emit_call_bare_diff_frame(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_foreign_value_ctor(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_rust_macro_shape(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_collection_ctor_strip(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_task_spawn(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_task_handle_send(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_generic_json_decode(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_print_write_family(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_union_ctor(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_imported_rust_macro(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_assert_family(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_plain_name_map(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_dot_module_methods(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_bina_module_methods(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_send_await_ask(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_bina_method_translate(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_dot_method_dispatch(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_struct_construction(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_min_max(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_qualified_type_ctor(call, out)? == CallShape::Handled {
            return Ok(());
        }
        if self.emit_call_const_reference(call, out)? == CallShape::Handled {
            return Ok(());
        }
        self.emit_call_generic_fallback(call, out)?;
        Ok(())
    }

    /// Plan 610 ⑤: the pointer bridge kit — raw-buffer/out-param access from
    /// `#[export]` bodies is confined to this generated module (004 §3.5:
    /// unsafe lives in generated code only; Auto bodies stay safe). Emitted
    /// at most once per product, ahead of the first export wrapper that has
    /// any pointer in its face.
    pub(super) fn emit_cabi_kit(&mut self, sink: &mut Sink) -> AutoResult<()> {
        if self.cabi_kit_emitted {
            return Ok(());
        }
        self.cabi_kit_emitted = true;
        writeln!(
            sink.body,
            r#"/// Plan 610 ⑤: pointer bridge kit — raw-buffer/out-param access from #[export]
/// bodies is confined here (004 §3.5: unsafe lives in generated code only;
/// Auto bodies stay safe). All writes are null-tolerant no-ops.
mod auto_cabi_kit {{
    /// Null pointer of the statically expected pointee type (spawn-failure paths).
    pub fn cabi_null<T>() -> *mut T {{
        std::ptr::null_mut()
    }}

    /// Default shell name (COMSPEC → fallback "cmd"), mirroring engine ffi.rs.
    pub fn cabi_default_shell() -> String {{
        std::env::var("COMSPEC").unwrap_or_else(|_| "cmd".to_owned())
    }}

    /// Copy UTF-8 bytes of `s` + NUL into `p` (≤cap). Returns total bytes incl.
    /// NUL (truncation mirrors the engine row_text contract); null p is a
    /// length-only probe.
    pub fn cabi_copy_cstr(p: *mut u8, cap: i64, s: &str) -> i64 {{
        if p.is_null() || cap <= 0 {{
            return (s.len() + 1) as i64;
        }}
        let bytes = s.as_bytes();
        let n = bytes.len().min(cap as usize - 1);
        unsafe {{
            let out = std::slice::from_raw_parts_mut(p, n + 1);
            out[..n].copy_from_slice(&bytes[..n]);
            out[n] = 0;
        }}
        (bytes.len() + 1) as i64
    }}

    /// Write one u32 element at `idx` (C `unsigned int`/`int` out-params — the
    /// engine face models non-negative int outs as u32, bit-compatible).
    pub fn cabi_write_u32(p: *mut u32, idx: i64, v: u32) {{
        if p.is_null() {{
            return;
        }}
        unsafe {{ *p.add(idx as usize) = v; }}
    }}

    /// Write one u8 element at `idx` (byte buffers).
    pub fn cabi_write_u8(p: *mut u8, idx: i64, v: u8) {{
        if p.is_null() {{
            return;
        }}
        unsafe {{ *p.add(idx as usize) = v; }}
    }}

    /// Copy `len` bytes from a raw const buffer into an owned Vec (host→child
    /// input path); null/empty buffer → empty Vec.
    pub fn cabi_bytes_vec(p: *const u8, len: i64) -> Vec<u8> {{
        if p.is_null() || len <= 0 {{
            return Vec::new();
        }}
        unsafe {{ std::slice::from_raw_parts(p, len as usize).to_vec() }}
    }}

    /// Reclaim a boxed value through its raw handle (free paths); null no-op.
    pub fn cabi_drop_boxed<T>(p: *mut T) {{
        if p.is_null() {{
            return;
        }}
        drop(unsafe {{ Box::from_raw(p) }});
    }}

    /// Bridge for `Option<(usize, usize)>` payloads (cursor): Some → write both
    /// u32 slots, return 1; None → 0. Auto has no tuple face, so foreign
    /// option-tuple values cross through this kit helper untouched.
    pub fn cabi_write_opt_tuple2_u32(cur: Option<(usize, usize)>, p0: *mut u32, p1: *mut u32) -> i64 {{
        match cur {{
            Some((a, b)) => {{
                cabi_write_u32(p0, 0, a as u32);
                cabi_write_u32(p1, 0, b as u32);
                1
            }}
            None => 0,
        }}
    }}
}}
pub use auto_cabi_kit::*;"#
        )?;
        Ok(())
    }
}
