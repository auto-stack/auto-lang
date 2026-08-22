//! Plan 417-E4 (DIV-TRAIT-VM-2): spec default-method inheritance on the VM.

#[test]
fn test_spec_default_method_inherited_and_overridden() {
    let code = r#"
spec Announcer {
    fn label() str
    fn announce() {
        print("[ANN] " + self.label())
    }
}
type Robot as Announcer {
    id int
    fn label() str { return "robot-" + self.id.to(str) }
}
type LoudRobot as Announcer {
    id int
    fn label() str { return "robot-" + self.id.to(str) }
    fn announce() { print("[LOUD] " + self.label()) }
}
fn main() {
    var r Robot = Robot { id: 7 }
    print("label=" + r.label())
    r.announce()
    var l LoudRobot = LoudRobot { id: 9 }
    l.announce()
}
"#;
    let (_result, out) = crate::run_with_capture(code).unwrap();
    assert!(
        out.contains("[ANN] robot-7"),
        "inherited default body runs: {out}"
    );
    assert!(
        out.contains("[LOUD] robot-9"),
        "implementer override wins: {out}"
    );
}

/// The checker no longer demands re-declaration of default-bodied methods
/// (abstract methods are still enforced).
#[test]
fn test_abstract_spec_method_still_enforced() {
    let code = r#"
spec Speaker {
    fn speak() str
}
type Mute as Speaker {
    id int
    fn quiet() bool { return true }
}
"#;
    let result = crate::run_with_capture(code);
    assert!(
        result.is_err(),
        "abstract method without default must still fail conformance"
    );
    let msg = format!("{:?}", result.err().unwrap());
    assert!(
        msg.contains("does not implement required method 'speak'"),
        "error names the missing abstract method: {msg}"
    );
}

/// Plan 417-E3 (DIV-TRAIT-VM-1): bounded generic function — `fn max_of<T has
/// Comparable>` parses, and the method call on the generic receiver dispatches
/// dynamically on the runtime type (CALL_SPEC on the heap tag) instead of a
/// static `T.compare` reloc that can never link.
#[test]
fn test_bounded_generic_fn_dispatches_on_runtime_type() {
    let code = r#"
spec Comparable {
    fn compare(other int) int
}

fn max_of<T has Comparable>(a T, b T) T {
    if a.compare(0) >= b.compare(0) { a } else { b }
}

type Score as Comparable {
    val int
    fn compare(other int) int {
        return self.val - other
    }
}

fn main() {
    var x Score = Score { val: 3 }
    var y Score = Score { val: 9 }
    print(max_of(x, y).val)
}
"#;
    let (_result, out) = crate::run_with_capture(code).unwrap();
    assert!(
        out.trim().contains("9"),
        "bounded generic fn picks the larger implementer: {out}"
    );
}

/// Two different implementers flow through the same generic fn body — the
/// dispatch must follow each receiver's own runtime type, not the first.
#[test]
fn test_bounded_generic_fn_multiple_implementers() {
    let code = r#"
spec Labeler {
    fn label() str
}

fn shout<T has Labeler>(item T) str {
    return "[" + item.label() + "]"
}

type User as Labeler {
    id int
    fn label() str { return "user-" + self.id.to(str) }
}

type Bot as Labeler {
    id int
    fn label() str { return "bot-" + self.id.to(str) }
}

fn main() {
    var u User = User { id: 7 }
    var b Bot = Bot { id: 2 }
    print(shout(u))
    print(shout(b))
}
"#;
    let (_result, out) = crate::run_with_capture(code).unwrap();
    assert!(
        out.contains("[user-7]"),
        "first implementer dispatches to User.label: {out}"
    );
    assert!(
        out.contains("[bot-2]"),
        "second implementer dispatches to Bot.label: {out}"
    );
}

/// A bare (unbounded) generic fn must also dispatch dynamically — the `has`
/// bound is optional syntax, not a prerequisite for CALL_SPEC dispatch.
#[test]
fn test_unbounded_generic_fn_dispatches_dynamically() {
    let code = r#"
fn name_of<T>(item T) str {
    return item.name()
}

type Pair {
    nm str
    fn name() str { return self.nm }
}

fn main() {
    var p Pair = Pair { nm: "kite" }
    print(name_of(p))
}
"#;
    let (_result, out) = crate::run_with_capture(code).unwrap();
    assert!(
        out.trim().contains("kite"),
        "unbounded generic fn dispatches on runtime type: {out}"
    );
}

/// Plan 417-E3-P4: call-site bound checking — a call whose argument's static
/// type demonstrably does not implement the bound is rejected at compile time.
#[test]
fn test_call_site_bound_violation_rejected() {
    let code = r#"
spec Comparable {
    fn compare(other int) int
}

spec Labeler {
    fn label() str
}

fn max_of<T has Comparable>(a T, b T) T {
    if a.compare(0) >= b.compare(0) { a } else { b }
}

type Score as Comparable {
    val int
    fn compare(other int) int {
        return self.val - other
    }
}

type Widget as Labeler {
    id int
    fn label() str { return "w" }
}

fn main() {
    var s Score = Score { val: 3 }
    var w Widget = Widget { id: 1 }
    print(max_of(s, w).val)
}
"#;
    let result = crate::run_with_capture(code);
    assert!(result.is_err(), "non-implementer argument must be rejected");
    let msg = format!("{:?}", result.err().unwrap());
    assert!(
        msg.contains("does not implement spec 'Comparable'"),
        "error names the unimplemented bound: {msg}"
    );
}

/// The happy path still compiles: both args implement the bound.
#[test]
fn test_call_site_bound_satisfied_passes() {
    let code = r#"
spec Comparable {
    fn compare(other int) int
}

fn max_of<T has Comparable>(a T, b T) T {
    if a.compare(0) >= b.compare(0) { a } else { b }
}

type Score as Comparable {
    val int
    fn compare(other int) int {
        return self.val - other
    }
}

fn main() {
    var x Score = Score { val: 3 }
    var y Score = Score { val: 9 }
    print(max_of(x, y).val)
}
"#;
    let (_result, out) = crate::run_with_capture(code).unwrap();
    assert!(out.trim().contains("9"), "valid implementers pass: {out}");
}

/// Pass-through generics stay dynamic: a generic wrapper fn calling the
/// bounded fn with its OWN type param must NOT be rejected (the wrapper's
/// param type is not a concrete implementer — dispatch stays runtime).
#[test]
fn test_call_site_bound_pass_through_generic_allowed() {
    let code = r#"
spec Comparable {
    fn compare(other int) int
}

fn max_of<T has Comparable>(a T, b T) T {
    if a.compare(0) >= b.compare(0) { a } else { b }
}

fn wrapper<T has Comparable>(a T, b T) T {
    return max_of(a, b)
}

type Score as Comparable {
    val int
    fn compare(other int) int {
        return self.val - other
    }
}

fn main() {
    var x Score = Score { val: 4 }
    var y Score = Score { val: 7 }
    print(wrapper(x, y).val)
}
"#;
    let (_result, out) = crate::run_with_capture(code).unwrap();
    assert!(out.trim().contains("7"), "pass-through generic dispatches: {out}");
}
