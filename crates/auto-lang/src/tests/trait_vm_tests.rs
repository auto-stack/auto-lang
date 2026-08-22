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
