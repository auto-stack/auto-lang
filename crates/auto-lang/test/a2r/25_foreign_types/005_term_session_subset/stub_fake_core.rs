// Plan 599 capstone stub: the fake "terminal core" crate face — compiled as
// a lib crate named `fake_core` and shared by BOTH the a2r product and the
// hand-written Rust oracle (black-box parity, same dependency).
pub trait EventListener {
    fn on_event(&self, code: i64) -> i64;
}

pub trait Dimensions {
    fn area(&self) -> i64;
}

pub trait FakeMaster {}

pub struct RealMaster;
impl FakeMaster for RealMaster {}
impl RealMaster {
    pub fn new() -> Self {
        RealMaster
    }
}

pub struct FakeTerm<L: EventListener> {
    listener: Box<L>,
    fed: i64,
    echo: i64,
}

impl<L: EventListener> FakeTerm<L> {
    pub fn new(listener: Box<L>) -> Self {
        Self { listener, fed: 0, echo: 0 }
    }
    pub fn feed(&mut self, code: i64) {
        self.fed += 1;
        self.echo = self.listener.on_event(code);
    }
    pub fn drained(&self) -> i64 {
        self.fed
    }
    pub fn echo(&self) -> i64 {
        self.echo
    }
}
