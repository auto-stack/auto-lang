// Plan 599 capstone oracle: hand-written Rust true-form of the subset.
// Black-box contract: stdout must equal the a2r product's byte for byte.
use fake_core::{Dimensions, EventListener, FakeMaster, FakeTerm, RealMaster};

struct GridSize {
    cols: i64,
    rows: i64,
}

impl Dimensions for GridSize {
    fn area(&self) -> i64 {
        self.cols * self.rows
    }
}

#[allow(dead_code)]
enum Damage {
    Full,
    Lines(Vec<i64>),
}

struct ChannelListener {
    hits: i64,
}

impl EventListener for ChannelListener {
    fn on_event(&self, code: i64) -> i64 {
        self.hits + code
    }
}

struct TermSession {
    term: FakeTerm<ChannelListener>,
    size: GridSize,
    master: Box<dyn FakeMaster + Send>,
}

fn main() {
    let mut term = FakeTerm::new(Box::new(ChannelListener { hits: 5 }));
    let size = GridSize { cols: 80, rows: 24 };
    let area = size.area();
    term.feed(3);
    let drained = term.drained();
    let echo = term.echo();
    println!("area= {}", area);
    println!("drained= {}", drained);
    println!("echo= {}", echo);
    let session = TermSession { term, size, master: Box::new(RealMaster::new()) };
    println!("session_ok  {}", session.term.echo());
}
