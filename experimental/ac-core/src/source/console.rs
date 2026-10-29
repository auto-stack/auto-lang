//! PLAN-863 typed source host symbols. No provider or host addresses in HIR.
use super::resolve::VT;
pub const STDOUT: &str = "console.stdout";
pub const STDERR: &str = "console.stderr";
pub fn symbol(callee: &str, ty: VT) -> Option<&'static str> {
    match (callee, ty) {
        ("print", VT::I32) => Some("auto.console.print.i32.v1"),
        ("print", VT::Bool) => Some("auto.console.print.bool.v1"),
        ("print", VT::Str | VT::StrOwned) => Some("auto.console.print.str.v1"),
        ("print", VT::Char) => Some("auto.console.print.char.v1"),
        ("eprint", VT::I32) => Some("auto.console.eprint.i32.v1"),
        ("eprint", VT::Bool) => Some("auto.console.eprint.bool.v1"),
        ("eprint", VT::Str | VT::StrOwned) => Some("auto.console.eprint.str.v1"),
        ("eprint", VT::Char) => Some("auto.console.eprint.char.v1"),
        _ => None,
    }
}
pub fn signature(symbol: &str) -> Option<(VT, &'static str)> {
    for callee in ["print", "eprint"] {
        for ty in [VT::I32, VT::Bool, VT::Str, VT::Char] {
            if self::symbol(callee, ty) == Some(symbol) {
                return Some((ty, if callee == "print" { STDOUT } else { STDERR }));
            }
        }
    }
    None
}
