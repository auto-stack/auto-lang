//! PLAN-863 typed source host symbols. No provider or host addresses in HIR.
//! PLAN-947: `say` joins as the 943-frozen public spelling — old stdlib
//! `io.say(msg str)` routed to `auto.print_str` whose writer appends exactly
//! one LF, byte-identical to the console/1 stdout str lane.
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
        ("say", VT::Str | VT::StrOwned) => Some("auto.console.print.str.v1"),
        _ => None,
    }
}
pub fn signature(symbol: &str) -> Option<(VT, &'static str)> {
    for callee in ["print", "eprint", "say"] {
        for ty in [VT::I32, VT::Bool, VT::Str, VT::Char] {
            if self::symbol(callee, ty) == Some(symbol) {
                return Some((ty, if callee == "eprint" { STDERR } else { STDOUT }));
            }
        }
    }
    None
}
/// Console spellings cannot be rebound as names in the host-console profile.
pub fn is_reserved_name(name: &str) -> bool {
    matches!(name, "print" | "eprint" | "say")
}
