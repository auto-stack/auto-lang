//! OS observations, separate from requested bytes and arena pages. The OS
//! maintains process peak commit even when a short allocation escapes polling.
#[repr(C)]
struct Counters {
    cb: u32,
    faults: u32,
    sizes: [usize; 9],
}
extern "system" {
    fn GetCurrentProcess() -> isize;
    fn K32GetProcessMemoryInfo(process: isize, counters: *mut Counters, bytes: u32) -> i32;
}
static mut BASE: usize = 0;
static mut STARTED: bool = false;
static mut HEALTHY: bool = true;
pub fn begin() {
    unsafe {
        if !STARTED {
            let mut c = Counters {
                cb: core::mem::size_of::<Counters>() as u32,
                faults: 0,
                sizes: [0; 9],
            };
            HEALTHY = K32GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut c,
                core::mem::size_of::<Counters>() as u32,
            ) != 0;
            BASE = c.sizes[8];
            STARTED = true;
        }
    }
}
pub struct Observation {
    pub base: u64,
    pub current: u64,
    pub peak_commit: u64,
    pub delta: u64,
    pub ok: bool,
}
pub fn finish() -> Observation {
    begin();
    unsafe {
        let mut c = Counters {
            cb: core::mem::size_of::<Counters>() as u32,
            faults: 0,
            sizes: [0; 9],
        };
        let ok = K32GetProcessMemoryInfo(
            GetCurrentProcess(),
            &mut c,
            core::mem::size_of::<Counters>() as u32,
        ) != 0;
        Observation {
            base: BASE as u64,
            current: c.sizes[8] as u64,
            peak_commit: c.sizes[7] as u64,
            delta: c.sizes[7].saturating_sub(BASE) as u64,
            ok: HEALTHY && ok && BASE > 0 && c.sizes[7] > 0 && c.sizes[8] > 0,
        }
    }
}
