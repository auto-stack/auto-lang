//! Logical symbol scheme (contract-decision §3): function symbols derive
//! from the owner scope + unit name + declared signature identity — never
//! from a body hash — so a body edit keeps the symbol and swaps only the
//! object content.

use ace_av_core::contract::fnv1a64;
use ace_av_core::mir::OwnerScope;

/// 16-hex content axis of an owner scope (MIR schema + module name).
pub fn owner_axis(owner: &OwnerScope) -> String {
    format!(
        "{:016x}",
        fnv1a64(format!("{}\n{}", owner.schema, owner.module).as_bytes())
    )
}

/// Export symbol of one function unit: `ac8.<owner-axis>.<unit-name>`.
pub fn function_symbol(owner: &OwnerScope, unit_name: &str) -> String {
    format!("ac8.{}.{}", owner_axis(owner), unit_name)
}

/// Import symbol resolution for a unit import table entry: test trace
/// intrinsics pass through under their landed symbol; `fn:<name>` linkage
/// imports resolve to the callee's logical symbol.
pub fn import_symbol(owner: &OwnerScope, symbol: &str) -> Result<String, crate::MnError> {
    if let Some(name) = symbol.strip_prefix("fn:") {
        if name.is_empty() {
            return Err(crate::MnError::Unsupported(
                "import",
                format!("empty fn: linkage import `{symbol}`"),
            ));
        }
        return Ok(function_symbol(owner, name));
    }
    match symbol {
        "hir.test.mark_a" | "hir.test.mark_b" => Ok(symbol.to_string()),
        other => Err(crate::MnError::Unsupported(
            "import",
            format!("import `{other}` is outside the admitted runtime surface"),
        )),
    }
}
