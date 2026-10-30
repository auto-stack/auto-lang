//! Named, controlled rejections of the A2X admission gates (plan §5.3:
//! "未知profile/坏引用/图环/初始化/非法类型必须受控拒绝" — the view adds a
//! second, target-facing gate on top of the frontend's own diagnostics).

/// A controlled rejection. `code` is a stable identifier (`a2x.*`), `detail`
/// is human-readable context. Rejections are language/contract-level facts —
/// they never stand for infra or resource failures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct A2xReject {
    pub code: &'static str,
    pub detail: String,
}

impl A2xReject {
    pub fn new(code: &'static str, detail: impl Into<String>) -> Self {
        A2xReject {
            code,
            detail: detail.into(),
        }
    }

    /// The HIR domain is not one of the two supported source domains.
    pub fn domain(detail: impl Into<String>) -> Self {
        Self::new("a2x.domain", detail)
    }

    /// An expression kind outside the core/scalar compute surface.
    pub fn unsupported_expr(detail: impl Into<String>) -> Self {
        Self::new("a2x.unsupported-expr", detail)
    }

    /// A statement kind outside the core/scalar compute surface.
    pub fn unsupported_stmt(detail: impl Into<String>) -> Self {
        Self::new("a2x.unsupported-stmt", detail)
    }

    /// A type outside {i32, bool} (params, results, locals).
    pub fn unsupported_type(detail: impl Into<String>) -> Self {
        Self::new("a2x.unsupported-type", detail)
    }

    /// A place form other than a whole local.
    pub fn unsupported_place(detail: impl Into<String>) -> Self {
        Self::new("a2x.unsupported-place", detail)
    }

    /// An operator outside the domain's frozen accept set.
    pub fn unsupported_op(detail: impl Into<String>) -> Self {
        Self::new("a2x.unsupported-op", detail)
    }

    /// An intrinsic binding without its authorizing capability.
    pub fn unauthorized_intrinsic(detail: impl Into<String>) -> Self {
        Self::new("a2x.unauthorized-intrinsic", detail)
    }

    /// Entry selection failed (missing/ambiguous/shape-mismatched entry).
    pub fn entry(detail: impl Into<String>) -> Self {
        Self::new("a2x.entry", detail)
    }
}

impl std::fmt::Display for A2xReject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for A2xReject {}
