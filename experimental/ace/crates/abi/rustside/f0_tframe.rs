//! auto-ffi provider f0_tframe/1 target=x86_64-pc-windows-msvc profile=f0-tframe.v1
//!
//! PLAN-967 T-05 F1 (AC-INT-1026 / V-INT-1026): the frozen INDEPENDENT
//! generic Rust provider — receiver/positionals/kwargs arrive as the
//! checked `p967.frame/1` tag frame (the generated shim defines F0Frame
//! in this same flat build closure); 标量/text/opaque/kwargs/失败 all
//! cross for real. 绝非 Torch — a deliberately tiny generic provider.

// auto-ffi fn tframe_sum_i32 symbol=f0.tframe.sum_i32.v1 form=fallible errors=no-i32:-1 frame=p967.frame/1
pub fn tframe_sum_i32(frame: &F0Frame) -> Option<i32> {
    let mut sum = 0i32;
    let mut seen = false;
    for v in frame
        .positionals
        .iter()
        .chain(frame.kwargs.iter().map(|k| &k.value))
    {
        if let F0Val::I32(n) = v {
            sum += *n;
            seen = true;
        }
    }
    if seen {
        Some(sum)
    } else {
        None
    }
}

// auto-ffi fn tframe_text_len symbol=f0.tframe.text_len.v1 form=pure ret=i32 frame=p967.frame/1
pub fn tframe_text_len(frame: &F0Frame) -> i32 {
    let mut n = 0i32;
    if let F0Val::Text(t) = &frame.receiver {
        n += t.len() as i32;
    }
    for v in frame
        .positionals
        .iter()
        .chain(frame.kwargs.iter().map(|k| &k.value))
    {
        if let F0Val::Text(t) = v {
            n += t.len() as i32;
        }
    }
    n
}

// The opaque registry: the fixture's OWN object store (ids are opaque
// references on the `o` tag lane; the provider is the single owner).
fn tframe_registry() -> &'static std::sync::Mutex<std::collections::BTreeMap<u64, i32>> {
    static R: std::sync::OnceLock<std::sync::Mutex<std::collections::BTreeMap<u64, i32>>> =
        std::sync::OnceLock::new();
    R.get_or_init(|| std::sync::Mutex::new(std::collections::BTreeMap::new()))
}

// auto-ffi fn tframe_opaque_store symbol=f0.tframe.opaque_store.v1 form=fallible errors=no-seed:-1 frame=p967.frame/1
pub fn tframe_opaque_store(frame: &F0Frame) -> Option<i32> {
    let seed = frame
        .positionals
        .iter()
        .find_map(|v| match v {
            F0Val::I32(n) => Some(*n),
            _ => None,
        })
        .or_else(|| {
            frame.kwargs.iter().find_map(|k| match &k.value {
                F0Val::I32(n) => Some(*n),
                _ => None,
            })
        })?;
    let mut r = tframe_registry().lock().ok()?;
    let id = (r.len() as u64) + 1;
    r.insert(id, seed);
    Some(id as i32)
}

// auto-ffi fn tframe_opaque_read symbol=f0.tframe.opaque_read.v1 form=fallible errors=no-handle:-1,bad-handle:-2 frame=p967.frame/1
pub fn tframe_opaque_read(frame: &F0Frame) -> Option<i32> {
    let id = frame
        .kwargs
        .iter()
        .find(|k| k.name == "handle")
        .and_then(|k| match &k.value {
            F0Val::Opaque(o) => Some(*o),
            _ => None,
        })?;
    let r = tframe_registry().lock().ok()?;
    Some(*r.get(&id)?)
}

// auto-ffi fn tframe_panic_probe symbol=f0.tframe.panic_probe.v1 form=pure ret=i32 frame=p967.frame/1
pub fn tframe_panic_probe(frame: &F0Frame) -> i32 {
    if let F0Val::Text(t) = &frame.receiver {
        if t == "boom" {
            panic!("tframe panic probe");
        }
    }
    7
}
