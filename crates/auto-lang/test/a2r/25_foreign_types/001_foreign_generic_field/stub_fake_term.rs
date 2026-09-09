// Plan 599 实编 stub: the foreign crate face 001 exercises (compiled as a
// lib crate named `fake_term`, linked via --extern; see a2r_tests.rs gate).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FakeTerm<L> {
    inner: std::boxed::Box<L>,
}

impl<L> FakeTerm<L> {
    pub fn new(inner: std::boxed::Box<L>) -> Self {
        Self { inner }
    }
    pub fn received(&self) -> i64 {
        42
    }
}
