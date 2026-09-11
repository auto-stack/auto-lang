// Plan 599 实编 stub: shared foreign face for 002/003 (compiled as a lib
// crate named `foreign_shape`, linked via --extern; see a2r_tests.rs gate).
pub trait ForeignTrait {
    #[allow(dead_code)]
    fn on_event(&self, evt: i64) -> i64;
}
pub trait FakeMaster {}
