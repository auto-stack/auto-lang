use std::sync::{Arc, Mutex, OnceLock};
use auto_lang::vm::{engine::AutoVM, host_capabilities::*, native::*, task::AutoTask, virt_memory::VirtualFlash};

#[derive(Default)]
struct Host(Mutex<Vec<(String, ScrollIntentDto)>>);
impl ScrollHostProvider for Host {
    fn next_controller_handle(&self) -> String { "review:句柄🎉".into() }
    fn enqueue_intent(&self, h: &str, i: ScrollIntentDto) { self.0.lock().unwrap().push((h.into(), i)); }
    fn controller_snapshot(&self, _: &str) -> ScrollSnapshotDto {
        ScrollSnapshotDto { offset_x: 1.0, offset_y: 2.0, viewport_w: 3.0, viewport_h: 4.0, content_w: 5.0, content_h: 6.0, progress_x: 0.7, progress_y: 0.8 }
    }
    fn enqueue_anchor_request(&self, _: &str, k: &str) -> Result<(), &'static str> {
        if k == "拒" { Err("unsupported") } else { Ok(()) }
    }
    fn anchor_request_state(&self, _: &str, k: &str) -> Option<ScrollAnchorStateDto> {
        match k { "待" => Some(ScrollAnchorStateDto::Pending), "成" => Some(ScrollAnchorStateDto::Applied), "败" => Some(ScrollAnchorStateDto::Invalid("not_found".into())), _ => None }
    }
}
fn push(vm: &AutoVM, t: &mut AutoTask, s: &str) {
    let id = vm.add_string(s.as_bytes().to_vec()); vm.rc_push_str_idx(t, id);
}
fn drop_result(vm: &AutoVM, t: &mut AutoTask) {
    let nv = t.ram.pop_nv(); vm.rc_release(nv);
}

#[test]
fn source_eight_natives_and_optional_axis_controls() {
    let _lock = serial();
    let host = Arc::new(Host::default());
    set_process_default_scroll_host_provider(host.clone());
    let src = r#"
let h = scroll_controller()
print(h)
print(scroll_to_start(h, "x"))
print(scroll_to_end(h, "unknown"))
print(scroll_by(h, "x", -2.5))
print(scroll_to(h, "x", 23.5))
let s = scroll_state(h)
print(s.progress_x)
print(scroll_to_child(h, "拒"))
print(scroll_anchor_state(h, "待"))
print(scroll_anchor_state(h, "成"))
print(scroll_anchor_state(h, "败"))
print(scroll_anchor_state(h, "未知"))
"#;
    let result = auto_lang::run_with_capture(src);
    clear_process_default_scroll_host_provider();
    let (_, out) = result.expect("source pipeline");
    assert_eq!(out.lines().collect::<Vec<_>>(), vec!["review:句柄🎉", "true", "true", "true", "true", "0.7", "error:unsupported", "pending", "applied", "invalid:not_found", "unknown"]);
    assert_eq!(*host.0.lock().unwrap(), vec![
        ("review:句柄🎉".into(), ScrollIntentDto::ToStart {axis: ScrollAxisDto::X}),
        ("review:句柄🎉".into(), ScrollIntentDto::ToEnd {axis: ScrollAxisDto::Y}),
        ("review:句柄🎉".into(), ScrollIntentDto::ScrollBy {axis: ScrollAxisDto::X, delta: -2.5}),
        ("review:句柄🎉".into(), ScrollIntentDto::ScrollTo {axis: ScrollAxisDto::X, offset: 23.5}),
    ]);
}

#[test]
fn ownership_all_shims_present_absent_empty_and_rejected() {
    let _lock = serial();
    clear_process_default_scroll_host_provider();
    for present in [false, true] {
        let vm = AutoVM::new(VirtualFlash::new(0), 64);
        if present { vm.set_scroll_host_provider(Arc::new(Host::default())); }
        let mut t = AutoTask::new(0, 1024, 0);
        let sp = t.ram.sp;
        let health = vm.pool_health(); let heap = vm.rc_stats().live_heap;
        for _ in 0..32 {
            shim_scroll_controller(&mut t, &vm).unwrap(); drop_result(&vm, &mut t);
            for h in ["review:句柄🎉", ""] {
                for axis in [None, Some("x"), Some("unknown")] {
                    for shim in [shim_scroll_to_start, shim_scroll_to_end] {
                        push(&vm, &mut t, h);
                        if let Some(axis) = axis { push(&vm, &mut t, axis); }
                        t.pending_native_arg_count = if axis.is_some() {2} else {1};
                        shim(&mut t, &vm).unwrap(); drop_result(&vm, &mut t);
                    }
                    push(&vm, &mut t, h);
                    if let Some(axis) = axis { push(&vm, &mut t, axis); }
                    t.ram.push_f64(3.5);
                    t.pending_native_arg_count = if axis.is_some() {3} else {2};
                    shim_scroll_by(&mut t, &vm).unwrap(); drop_result(&vm, &mut t);
                }
                for axis in [false, true] {
                    push(&vm, &mut t, h);
                    if axis { push(&vm, &mut t, "x"); } else { t.ram.push_f64(12.5); }
                    t.ram.push_f64(4.5);
                    shim_scroll_to(&mut t, &vm).unwrap(); drop_result(&vm, &mut t);
                }
                push(&vm, &mut t, h);
                shim_scroll_state(&mut t, &vm).unwrap(); drop_result(&vm, &mut t);
                for k in ["", "拒", "待", "成", "败", "未知"] {
                    for shim in [shim_scroll_to_child, shim_scroll_anchor_state] {
                        push(&vm, &mut t, h); push(&vm, &mut t, k);
                        shim(&mut t, &vm).unwrap(); drop_result(&vm, &mut t);
                    }
                }
                assert_eq!(t.ram.sp, sp);
            }
        }
        vm.reap_all();
        let now = vm.pool_health();
        assert_eq!(now.live_shares, health.live_shares, "present={present}: pool shares");
        assert_eq!(now.underflow_events, health.underflow_events, "pool underflow");
        assert_eq!(now.phantom_drops, health.phantom_drops, "pool phantom");
        assert_eq!(vm.rc_stats().live_heap, heap, "heap references");
    }
}
fn serial() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}
