// PLAN-998 independent reference (hand-written from the scenario oracle
// text; no code shared with the a2r emission). Prints the frozen
// `auto.res:*` vocabulary through its own observation machinery.
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static HEAP_N: AtomicUsize = AtomicUsize::new(0);
static FAULT_AT: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
static FAULT_OFF: AtomicBool = AtomicBool::new(false);
static IN: AtomicBool = AtomicBool::new(false);

struct Counting;
unsafe impl std::alloc::GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: std::alloc::Layout) -> *mut u8 {
        if !IN.load(Ordering::SeqCst) {
            let _ = HEAP_N.fetch_add(1, Ordering::SeqCst);
        }
        std::alloc::System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: std::alloc::Layout) { std::alloc::System.dealloc(p, l) }
    unsafe fn realloc(&self, p: *mut u8, l: std::alloc::Layout, n: usize) -> *mut u8 {
        // growth via realloc is part of the reference's allocation domain
        // (the grow-site checks cover it symmetrically)
        if !IN.load(Ordering::SeqCst) {
            let _ = HEAP_N.fetch_add(1, Ordering::SeqCst);
        }
        std::alloc::System.realloc(p, l, n)
    }
}
#[global_allocator]
static GLOBAL: Counting = Counting;

fn evt(kind: &str, arg: &str) {
    let was = IN.swap(true, Ordering::SeqCst);
    if arg.is_empty() { println!("auto.res:{}", kind); } else { println!("auto.res:{}:{}", kind, arg); }
    IN.store(was, Ordering::SeqCst);
}

fn fault_at() -> Option<usize> {
    *FAULT_AT.get_or_init(|| {
        let was = IN.swap(true, Ordering::SeqCst);
        let r = std::env::var("A2R_FAULT").ok().and_then(|v| {
            let (dom, k) = v.split_once('#')?;
            if dom != "heap" { return None; }
            k.parse::<usize>().ok()
        });
        IN.store(was, Ordering::SeqCst);
        r
    })
}

/// The reference's REAL allocation points. Each helper call performs
/// exactly ONE never-elidable heap allocation (a leaked probe — the
/// compiler cannot stack-allocate or remove a leak) preceded atomically by
/// the fault check; the returned VALUE is constructed under the IN guard so
/// its (possibly elided) allocations never enter the ordinal space. A
/// strike therefore happens exactly at a real allocation point and the N+1
/// control can never false-strike.
static PROBE_N: AtomicUsize = AtomicUsize::new(0);

/// One never-elidable, never-mergeable heap allocation per call: the size
/// varies per probe (defeats CSE) and the leak's unbounded lifetime forbids
/// heap-to-stack demotion; the fetch_add side effect pins every call.
fn probe() {
    let n = PROBE_N.fetch_add(1, Ordering::SeqCst);
    let sz = 16 + (n % 64);
    let v = std::vec![0u8; sz].into_boxed_slice();
    let leaked: &'static mut [u8] = Box::leak(v);
    black_box(leaked.as_mut_ptr());
}

fn probe_fault_check() {
    if !FAULT_OFF.load(Ordering::SeqCst) {
        if let Some(k) = fault_at() {
            if HEAP_N.load(Ordering::SeqCst) + 1 == k { typed_fail("fault"); }
        }
    }
}

fn guarded<T>(f: impl FnOnce() -> T) -> T {
    let was = IN.swap(true, Ordering::SeqCst);
    let r = f();
    IN.store(was, Ordering::SeqCst);
    r
}

fn alloc_string(lit: &str) -> String {
    probe_fault_check();
    probe();
    guarded(|| {
        let mut b: Vec<u8> = Vec::with_capacity(lit.len());
        unsafe {
            for (i, byte) in lit.bytes().enumerate() {
                std::ptr::write_volatile(b.as_mut_ptr().add(i), byte);
            }
            b.set_len(lit.len());
        }
        String::from_utf8(b).unwrap()
    })
}

fn alloc_clone(src: &str) -> String {
    probe_fault_check();
    probe();
    guarded(|| {
        let mut b: Vec<u8> = Vec::with_capacity(src.len());
        unsafe {
            for (i, byte) in src.bytes().enumerate() {
                std::ptr::write_volatile(b.as_mut_ptr().add(i), byte);
            }
            b.set_len(src.len());
        }
        String::from_utf8(b).unwrap()
    })
}

/// A list growth-preparation point (the vec domain): the probe allocation
/// happens only when capacity is really exhausted (reserve on available
/// capacity allocates nothing).
fn alloc_grow(v: &mut Vec<impl Sized>) {
    if v.capacity() == v.len() {
        probe_fault_check();
        probe();
        // the growth site's OWN allocation (the reserve) is also a real
        // allocation — the second check covers it.
        probe_fault_check();
        v.reserve_exact(1);
    }
}

fn typed_fail(kind: &'static str) -> ! {
    evt(kind, "");
    eprintln!("auto.res:fail:{}", kind);
    std::panic::panic_any(format!("auto.res:fail:{}", kind))
}

/// Independent RAII observation: the owner guard prints its release when it
/// drops (a transfer disarms by moving out through `transfer`).
struct Owner { name: &'static str, live: bool }
impl Owner {
    fn new(name: &'static str) -> Self { evt("create", name); Owner { name, live: true } }
    fn transfer<T>(mut self, v: T) -> T {
        self.live = false;
        evt("transfer", self.name);
        v
    }
}
impl Drop for Owner {
    fn drop(&mut self) { if self.live { evt("release", self.name); } }
}

fn drive<F: FnMut() -> i64>(mut f: F) {
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f()));
    match r {
        Ok(v) => {
            println!("auto.res:heap:{}", HEAP_N.load(Ordering::SeqCst));
            println!("auto.res:result:{}", v);
        }
        Err(p) => {
            let msg = p.downcast_ref::<String>().cloned()
                .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()));
            match msg.as_deref() {
                Some(s) if s.starts_with("auto.res:fail:fault")
                    && std::env::var("A2R_RETRY").map(|v| v == "1").unwrap_or(false) => {
                    println!("auto.res:retry");
                    FAULT_OFF.store(true, Ordering::SeqCst);
                    let v2 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f()))
                        .unwrap_or_else(|_| { eprintln!("auto.res:fail:retry-foreign"); std::process::exit(101) });
                    println!("auto.res:heap:{}", HEAP_N.load(Ordering::SeqCst));
                    println!("auto.res:result:{}", v2);
                }
                Some(s) if s.starts_with("auto.res:fail:") => std::process::exit(71),
                _ => { eprintln!("auto.res:fail:foreign"); std::process::exit(101) }
            }
        }
    }
}
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    drive(|| {
        // Variant switch: the old arm's payload releases EXACTLY once at the
        // replacement (replace_arm); the inactive payload is never read; 42.
        let mut r: Result<i64, String> = Err(alloc_string("old"));
        let g1 = Owner::new("ref:r#1");
        let _old = std::mem::replace(&mut r, Ok(42));
        drop(_old); // the REAL drop of the old payload at the switch point
        let g2 = replace_arm(g1, "ref:r#2");
        let out = match r {
            Ok(v) => v,
            Err(e) => e.chars().count() as i64,
        };
        let _ = g2;
        out
    })
}

fn replace_arm(mut old: Owner, new_name: &'static str) -> Owner {
    old.live = false;
    evt("release", old.name);
    Owner::new(new_name)
}
