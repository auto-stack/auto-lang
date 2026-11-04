// PLAN-931 hand-written independent reference — wrap-931 (std String,
// independent of the generated a2x runtime): two literals passed to a
// constructor that CLONES them into a record; span = head.len() +
// tail.len() = 2 + 3 = 5 (byte lengths; the Auto len counts scalars —
// all-ASCII here, so identical).
fn main() {
    struct Wrap {
        head: String,
        tail: String,
    }
    impl Wrap {
        fn mk(h: &str, t: &str) -> Wrap {
            Wrap { head: h.to_string(), tail: t.to_string() }
        }
        fn span(&self) -> i32 {
            (self.head.chars().count() + self.tail.chars().count()) as i32
        }
    }
    let w = Wrap::mk("ab", "cde");
    println!("value:{}", w.span());
}
