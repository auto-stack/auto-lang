//! PLAN-808: terminal host-domain extraction — in-crate acceptance matrix.
//!
//! V-01: 34 native shapes / default assembly / no-UI defaults.
//! V-02: directional input+geometry+menu and once-consumption (real DLL).
//! V-03: display_offset single source + UI projection (real DLL).
//! V-04: session lifecycle (spawn/feed/resize/interrupt/exit/free, two
//!       sessions, duplicate free, provider replacement routing) — real DLL.
//! V-05: absent capability matrix (no DLL / load failure / missing required
//!       symbol / missing optional palette / no provider).
//! V-06: closure/source cut (term_engine is codec only; 34/34 mapped).
//!
//! Real-DLL tests require `AUTOTERM_ENGINE_DLL` to point at a built
//! `autoterm_core.dll` (see docs/plans/attachments/808/contract-baseline.md).
//! Without it they print an explicit `PENDING` line and assert nothing — the
//! work receipt only counts the env-enabled run, never a silent skip.

// ── V-06: source cut / inventory ─────────────────────────────────────────

mod plan808_core_cut {
    const SHIMS: [&str; 34] = [
        "shim_term_spawn",
        "shim_term_spawn_ex",
        "shim_term_write_line",
        "shim_term_write_raw",
        "shim_term_rows",
        "shim_term_rows_for",
        "shim_term_resize",
        "shim_term_interrupt",
        "shim_term_is_exited",
        "shim_term_free",
        "shim_term_pump_input",
        "shim_term_pump_for",
        "shim_term_apply_resize",
        "shim_term_apply_resize_for",
        "shim_term_menu_take",
        "shim_term_cursor_row",
        "shim_term_cursor_col",
        "shim_term_viewport_cols",
        "shim_term_viewport_rows",
        "shim_term_backlog_pending_mb",
        "shim_term_backlog_paused",
        "shim_term_backlog_take_alerts",
        "shim_term_backlog_dropped",
        "shim_term_engine_history",
        "shim_term_window_width",
        "shim_term_window_height",
        "shim_term_config_profiles",
        "shim_term_config_default_profile",
        "shim_term_config_profiles_full",
        "shim_term_config_spawn_program",
        "shim_term_config_spawn_argv",
        "shim_term_config_spawn_cwd",
        "shim_term_scroll_pending_for",
        "shim_term_engine_pend_resize_geom",
    ];

    #[test]
    fn terminal_glue_is_codec_only() {
        let src = include_str!("../vm/ffi/term_engine.rs");
        for shim in SHIMS {
            assert!(src.contains(shim), "missing shim {shim}");
        }
        for forbidden in [
            "libloading",
            "Library",
            "autoterm_engine_",
            "NEXT_HANDLE",
            "HANDLES",
            "SNAPSHOTS",
            "BACKLOG_STATE",
            "decode_style_color",
            "terminal_feed_cells",
            "terminal_drain_",
            "terminal_take_",
            "terminal_pending_",
        ] {
            assert!(
                !src.contains(forbidden),
                "vm/ffi/term_engine.rs must be codec-only; found {forbidden:?}"
            );
        }
        // The only UI references are the two window-pixel compatibility
        // facade delegations to the named view adapter.
        let ui_refs: Vec<&str> = src.lines().filter(|l| l.contains("crate::ui::")).collect();
        assert_eq!(ui_refs.len(), 2, "facade-only UI refs: {ui_refs:?}");
        assert!(ui_refs.iter().all(|l| l.contains("terminal_provider")));
    }

    #[test]
    fn catalog_and_shims_match_34() {
        let catalog = include_str!("../vm/native_catalog.rs");
        for canonical in [
            "auto.term.engine_spawn",
            "auto.term.engine_write_line",
            "auto.term.engine_rows",
            "auto.term.engine_resize",
            "auto.term.engine_interrupt",
            "auto.term.engine_is_exited",
            "auto.term.engine_free",
            "auto.term.engine_pump_input",
            "auto.term.engine_cursor_row",
            "auto.term.engine_cursor_col",
            "auto.term.engine_apply_resize",
            "auto.term.engine_viewport_cols",
            "auto.term.engine_viewport_rows",
            "auto.term.engine_backlog_pending_mb",
            "auto.term.engine_backlog_paused",
            "auto.term.engine_backlog_take_alerts",
            "auto.term.engine_backlog_dropped",
            "auto.term.engine_spawn_ex",
            "auto.term.engine_rows_for",
            "auto.term.engine_pump_for",
            "auto.term.engine_apply_resize_for",
            "auto.term.engine_menu_take",
            "auto.term.config_profiles",
            "auto.term.config_default_profile",
            "auto.term.config_profiles_full",
            "auto.term.config_spawn_program",
            "auto.term.config_spawn_argv",
            "auto.term.config_spawn_cwd",
            "auto.term.engine_scroll_pending_for",
            "auto.term.engine_pend_resize_geom",
            "auto.term.engine_history",
            "auto.term.window_width",
            "auto.term.window_height",
            "auto.term.engine_write_raw",
        ] {
            assert!(catalog.contains(canonical), "catalog missing {canonical}");
        }
    }
}

// ── V-01/V-05: assembly identity + 34-shape roundtrip ────────────────────

mod plan808_assembly {
    use std::sync::{Arc, Mutex};

    use auto_terminal_core::TerminalHostProvider;
    use auto_val::{decode_bool, decode_i32, decode_string, is_string};

    use crate::vm::engine::AutoVM;
    use crate::vm::ffi::VMConvertible;
    use crate::vm::native::NativeInterface;
    use crate::vm::task::AutoTask;
    use crate::vm::virt_memory::VirtualFlash;

    /// Recording fake with distinct return values per method.
    #[derive(Default)]
    struct Recording {
        calls: Mutex<Vec<String>>,
    }

    impl Recording {
        fn record(&self, call: String) {
            self.calls.lock().unwrap().push(call);
        }
        fn take(&self) -> Vec<String> {
            std::mem::take(&mut *self.calls.lock().unwrap())
        }
    }

    impl TerminalHostProvider for Recording {
        fn spawn(&self, cols: i64, rows: i64) -> i32 {
            self.record(format!("spawn({cols},{rows})"));
            111
        }
        fn spawn_ex(&self, program: &str, argv: &[String], cwd: &str, cols: i64, rows: i64) -> i32 {
            self.record(format!(
                "spawn_ex({program},[{}],{cwd},{cols},{rows})",
                argv.join("|")
            ));
            112
        }
        fn write_line(&self, handle: i64, line: &str) {
            self.record(format!("write_line({handle},{line})"));
        }
        fn write_raw(&self, handle: i64, vt: &str) {
            self.record(format!("write_raw({handle},{vt})"));
        }
        fn rows(&self, handle: i64) -> Vec<String> {
            self.record(format!("rows({handle})"));
            vec!["R".to_string()]
        }
        fn rows_for(&self, handle: i64, key: &str) -> Vec<String> {
            self.record(format!("rows_for({handle},{key})"));
            vec!["RF".to_string()]
        }
        fn resize(&self, handle: i64, cols: i64, rows: i64) {
            self.record(format!("resize({handle},{cols},{rows})"));
        }
        fn apply_resize(&self, handle: i64) -> i32 {
            self.record(format!("apply_resize({handle})"));
            1
        }
        fn apply_resize_for(&self, handle: i64, key: &str) -> i32 {
            self.record(format!("apply_resize_for({handle},{key})"));
            1
        }
        fn pump_input(&self, handle: i64) -> i32 {
            self.record(format!("pump_input({handle})"));
            7
        }
        fn pump_for(&self, handle: i64, key: &str) -> i32 {
            self.record(format!("pump_for({handle},{key})"));
            8
        }
        fn scroll_pending_for(&self, handle: i64, key: &str) -> i32 {
            self.record(format!("scroll_pending_for({handle},{key})"));
            9
        }
        fn pend_resize_geom(&self, key: &str, geom: &str) -> i32 {
            self.record(format!("pend_resize_geom({key},{geom})"));
            1
        }
        fn menu_take(&self) -> i32 {
            self.record("menu_take()".to_string());
            3
        }
        fn interrupt(&self, handle: i64) -> i32 {
            self.record(format!("interrupt({handle})"));
            1
        }
        fn is_exited(&self, handle: i64) -> bool {
            self.record(format!("is_exited({handle})"));
            true
        }
        fn free(&self, handle: i64) {
            self.record(format!("free({handle})"));
        }
        fn cursor_row(&self, handle: i64) -> i32 {
            self.record(format!("cursor_row({handle})"));
            4
        }
        fn cursor_col(&self, handle: i64) -> i32 {
            self.record(format!("cursor_col({handle})"));
            5
        }
        fn viewport_cols(&self, handle: i64) -> i32 {
            self.record(format!("viewport_cols({handle})"));
            80
        }
        fn viewport_rows(&self, handle: i64) -> i32 {
            self.record(format!("viewport_rows({handle})"));
            24
        }
        fn backlog_pending_mb(&self, handle: i64) -> i32 {
            self.record(format!("backlog_pending_mb({handle})"));
            2
        }
        fn backlog_paused(&self, handle: i64) -> i32 {
            self.record(format!("backlog_paused({handle})"));
            1
        }
        fn backlog_take_alerts(&self, handle: i64) -> i32 {
            self.record(format!("backlog_take_alerts({handle})"));
            1
        }
        fn backlog_dropped(&self, handle: i64) -> i32 {
            self.record(format!("backlog_dropped({handle})"));
            3
        }
        fn history(&self, handle: i64) -> i32 {
            self.record(format!("history({handle})"));
            42
        }
        fn window_width(&self) -> i32 {
            self.record("window_width()".to_string());
            999
        }
        fn window_height(&self) -> i32 {
            self.record("window_height()".to_string());
            777
        }
        fn config_profiles(&self) -> Vec<String> {
            self.record("config_profiles()".to_string());
            vec!["p1".into(), "p2".into()]
        }
        fn config_default_profile(&self) -> String {
            self.record("config_default_profile()".to_string());
            "def".into()
        }
        fn config_profiles_full(&self) -> Vec<String> {
            self.record("config_profiles_full()".to_string());
            vec!["f1".into()]
        }
        fn config_spawn_program(&self, profile: &str) -> String {
            self.record(format!("config_spawn_program({profile})"));
            "prog".into()
        }
        fn config_spawn_argv(&self, profile: &str) -> Vec<String> {
            self.record(format!("config_spawn_argv({profile})"));
            vec!["a1".into(), "a2".into()]
        }
        fn config_spawn_cwd(&self, profile: &str) -> String {
            self.record(format!("config_spawn_cwd({profile})"));
            "cwd".into()
        }
    }

    fn vm_with(provider: Arc<Recording>) -> AutoVM {
        let vm = AutoVM::new(VirtualFlash::new(0), 2048);
        vm.set_terminal_provider(provider);
        vm
    }

    fn push_str(vm: &AutoVM, task: &mut AutoTask, s: &str) {
        let idx = vm.add_string(s.as_bytes().to_vec());
        vm.rc_push_str_idx(task, idx);
    }

    fn pop_i32(task: &mut AutoTask) -> i32 {
        decode_i32(task.ram.pop_nv())
    }

    fn pop_bool(task: &mut AutoTask) -> bool {
        decode_bool(task.ram.pop_nv())
    }

    fn pop_str(vm: &AutoVM, task: &mut AutoTask) -> String {
        let nv = task.ram.pop_nv();
        assert!(is_string(nv), "expected string");
        let idx = decode_string(nv);
        vm.get_string(idx)
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default()
    }

    fn pop_list(vm: &AutoVM, task: &mut AutoTask) -> Vec<String> {
        // `Vec<String>::push_to_stack` pushes a raw heap-object id (ListData<i32>),
        // not a NanoValue list tag.
        let id = task.ram.pop_i32() as u64;
        let obj = vm.get_heap_object(id).expect("heap list object");
        let guard = obj.read().unwrap();
        let list = guard
            .as_any()
            .downcast_ref::<crate::vm::types::ListData<i32>>()
            .expect("ListData<i32>");
        let strings = vm.strings.read().unwrap();
        let mut out = Vec::new();
        for i in 0..list.len() {
            let v = list.get(i).copied().unwrap_or(0);
            if v < 0 {
                if let Some(bytes) = strings.get((-v - 1) as usize) {
                    out.push(String::from_utf8_lossy(bytes).into_owned());
                }
            }
        }
        out
    }

    /// V-01: every one of the 34 shims pops the legacy argument order and
    /// pushes the legacy stack shape through the provider port.
    #[test]
    fn all_34_shims_roundtrip_through_provider() {
        use crate::vm::ffi::term_engine::*;
        let rec = Arc::new(Recording::default());
        let vm = vm_with(rec.clone());

        let mut task = AutoTask::new(0, 2048, 0);
        // 2943 spawn(cols, rows) — pops rows first.
        task.ram.push_nv(auto_val::encode_i32(80));
        task.ram.push_nv(auto_val::encode_i32(24));
        shim_term_spawn(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 111);

        // 2983 spawn_ex(program, argv, cwd, cols, rows).
        push_str(&vm, &mut task, "cmd");
        vec!["/C".to_string(), "echo x".to_string()]
            .push_to_stack(&mut task, &vm)
            .unwrap();
        push_str(&vm, &mut task, "D:/cwd");
        task.ram.push_nv(auto_val::encode_i32(80));
        task.ram.push_nv(auto_val::encode_i32(24));
        shim_term_spawn_ex(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 112);

        // 2944 write_line(handle, line).
        task.ram.push_nv(auto_val::encode_i32(7));
        push_str(&vm, &mut task, "hi");
        shim_term_write_line(&mut task, &vm).unwrap();

        // 3000 write_raw(handle, vt).
        task.ram.push_nv(auto_val::encode_i32(7));
        push_str(&vm, &mut task, "VT");
        shim_term_write_raw(&mut task, &vm).unwrap();

        // 2945 rows(handle).
        task.ram.push_nv(auto_val::encode_i32(7));
        shim_term_rows(&mut task, &vm).unwrap();
        assert_eq!(pop_list(&vm, &mut task), vec!["R".to_string()]);

        // 2984 rows_for(handle, key).
        task.ram.push_nv(auto_val::encode_i32(7));
        push_str(&vm, &mut task, "k");
        shim_term_rows_for(&mut task, &vm).unwrap();
        assert_eq!(pop_list(&vm, &mut task), vec!["RF".to_string()]);

        // 2946 resize(handle, cols, rows).
        task.ram.push_nv(auto_val::encode_i32(7));
        task.ram.push_nv(auto_val::encode_i32(100));
        task.ram.push_nv(auto_val::encode_i32(30));
        shim_term_resize(&mut task, &vm).unwrap();

        // 2947 interrupt / 2948 is_exited / 2949 free.
        task.ram.push_nv(auto_val::encode_i32(7));
        shim_term_interrupt(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 1);
        task.ram.push_nv(auto_val::encode_i32(7));
        shim_term_is_exited(&mut task, &vm).unwrap();
        assert!(pop_bool(&mut task));
        task.ram.push_nv(auto_val::encode_i32(7));
        shim_term_free(&mut task, &vm).unwrap();

        // 2957 pump_input / 2985 pump_for.
        task.ram.push_nv(auto_val::encode_i32(7));
        shim_term_pump_input(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 7);
        task.ram.push_nv(auto_val::encode_i32(7));
        push_str(&vm, &mut task, "k");
        shim_term_pump_for(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 8);

        // 2976 apply_resize / 2986 apply_resize_for / 3002 pend_resize_geom.
        task.ram.push_nv(auto_val::encode_i32(7));
        shim_term_apply_resize(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 1);
        task.ram.push_nv(auto_val::encode_i32(7));
        push_str(&vm, &mut task, "k");
        shim_term_apply_resize_for(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 1);
        push_str(&vm, &mut task, "k");
        push_str(&vm, &mut task, "80x30");
        shim_term_engine_pend_resize_geom(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 1);

        // 3001 scroll_pending_for / 2987 menu_take.
        task.ram.push_nv(auto_val::encode_i32(7));
        push_str(&vm, &mut task, "k");
        shim_term_scroll_pending_for(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 9);
        shim_term_menu_take(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 3);

        // Cached geometry/query surfaces.
        task.ram.push_nv(auto_val::encode_i32(7));
        shim_term_cursor_row(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 4);
        task.ram.push_nv(auto_val::encode_i32(7));
        shim_term_cursor_col(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 5);
        task.ram.push_nv(auto_val::encode_i32(7));
        shim_term_viewport_cols(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 80);
        task.ram.push_nv(auto_val::encode_i32(7));
        shim_term_viewport_rows(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 24);
        task.ram.push_nv(auto_val::encode_i32(7));
        shim_term_backlog_pending_mb(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 2);
        task.ram.push_nv(auto_val::encode_i32(7));
        shim_term_backlog_paused(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 1);
        task.ram.push_nv(auto_val::encode_i32(7));
        shim_term_backlog_take_alerts(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 1);
        task.ram.push_nv(auto_val::encode_i32(7));
        shim_term_backlog_dropped(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 3);
        task.ram.push_nv(auto_val::encode_i32(7));
        shim_term_engine_history(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 42);

        // 2998/2999 window + 2988..2993 config.
        shim_term_window_width(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 999);
        shim_term_window_height(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 777);
        shim_term_config_profiles(&mut task, &vm).unwrap();
        assert_eq!(
            pop_list(&vm, &mut task),
            vec!["p1".to_string(), "p2".to_string()]
        );
        shim_term_config_default_profile(&mut task, &vm).unwrap();
        assert_eq!(pop_str(&vm, &mut task), "def");
        shim_term_config_profiles_full(&mut task, &vm).unwrap();
        assert_eq!(pop_list(&vm, &mut task), vec!["f1".to_string()]);
        push_str(&vm, &mut task, "prof");
        shim_term_config_spawn_program(&mut task, &vm).unwrap();
        assert_eq!(pop_str(&vm, &mut task), "prog");
        push_str(&vm, &mut task, "prof");
        shim_term_config_spawn_argv(&mut task, &vm).unwrap();
        assert_eq!(
            pop_list(&vm, &mut task),
            vec!["a1".to_string(), "a2".to_string()]
        );
        push_str(&vm, &mut task, "prof");
        shim_term_config_spawn_cwd(&mut task, &vm).unwrap();
        assert_eq!(pop_str(&vm, &mut task), "cwd");

        let calls = rec.take();
        assert!(
            calls.contains(&"spawn(80,24)".to_string()),
            "calls: {calls:?}"
        );
        assert!(calls.contains(&"spawn_ex(cmd,[/C|echo x],D:/cwd,80,24)".to_string()));
        assert!(calls.iter().any(|c| c == "write_line(7,hi)"));
        assert!(calls.iter().any(|c| c == "write_raw(7,VT)"));
        assert!(calls.iter().any(|c| c == "rows_for(7,k)"));
        assert!(calls.iter().any(|c| c == "resize(7,100,30)"));
        assert!(calls.iter().any(|c| c == "pump_for(7,k)"));
        assert!(calls.iter().any(|c| c == "apply_resize_for(7,k)"));
        assert!(calls.iter().any(|c| c == "pend_resize_geom(k,80x30)"));
        assert!(calls.iter().any(|c| c == "scroll_pending_for(7,k)"));
        assert!(calls.iter().any(|c| c == "config_spawn_program(prof)"));
        assert!(calls.iter().any(|c| c == "free(7)"));
    }

    /// V-01: construction snapshot / instance override / Clone+merge keep the
    /// provider and do not touch binding identity.
    #[test]
    fn snapshot_override_clone_merge_identity() {
        let _slot = crate::vm::host_capabilities::test_default_slot_lock();
        let saved = crate::vm::host_capabilities::process_default_terminal_host_provider();

        let bare = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 64);
        assert!(bare.terminal_provider().is_none(), "构造式缺席");

        let a = Arc::new(Recording::default());
        crate::vm::host_capabilities::set_process_default_terminal_host_provider(a.clone());
        let vm1 = AutoVM::new(VirtualFlash::new(0), 256);
        let b = Arc::new(Recording::default());
        crate::vm::host_capabilities::set_process_default_terminal_host_provider(b.clone());
        let vm2 = AutoVM::new(VirtualFlash::new(0), 256);

        // vm1 keeps its construction-time snapshot: calling a shim hits A.
        let mut task = AutoTask::new(0, 256, 0);
        crate::vm::ffi::term_engine::shim_term_window_width(&mut task, &vm1).unwrap();
        assert_eq!(pop_i32(&mut task), 999);
        assert_eq!(a.take(), vec!["window_width()".to_string()], "vm1 uses A");
        let mut task = AutoTask::new(0, 256, 0);
        crate::vm::ffi::term_engine::shim_term_window_width(&mut task, &vm2).unwrap();
        assert_eq!(b.take(), vec!["window_width()".to_string()], "vm2 uses B");

        // Instance override only affects its VM.
        let c = Arc::new(Recording::default());
        vm1.set_terminal_provider(c.clone());
        let mut task = AutoTask::new(0, 256, 0);
        crate::vm::ffi::term_engine::shim_term_window_width(&mut task, &vm1).unwrap();
        assert_eq!(
            c.take(),
            vec!["window_width()".to_string()],
            "instance override"
        );
        assert!(a.take().is_empty(), "old snapshot untouched");

        // Later process-default changes do not affect existing instances.
        let d = Arc::new(Recording::default());
        crate::vm::host_capabilities::set_process_default_terminal_host_provider(d.clone());
        let mut task = AutoTask::new(0, 256, 0);
        crate::vm::ffi::term_engine::shim_term_window_width(&mut task, &vm2).unwrap();
        assert_eq!(b.take(), vec!["window_width()".to_string()]);
        assert!(d.take().is_empty());

        // Provider injection is not a binding change.
        let ni = NativeInterface::production();
        let rev0 = ni.binding_revision();
        ni.set_terminal_provider(Arc::new(Recording::default()));
        assert_eq!(ni.binding_revision(), rev0, "注入不 bump revision");
        assert!(
            ni.clone().terminal_provider().is_some(),
            "Clone keeps provider"
        );
        let mut vm = AutoVM::new(VirtualFlash::new(0), 64);
        vm.set_terminal_provider(Arc::new(Recording::default()));
        let mut carrier = NativeInterface::new();
        carrier.register_dynamic(|_t: &mut AutoTask, _v: &AutoVM| Ok(()));
        vm.merge_native_interface(&carrier);
        assert!(vm.terminal_provider().is_some(), "merge keeps provider");

        match saved {
            Some(p) => crate::vm::host_capabilities::set_process_default_terminal_host_provider(p),
            None => crate::vm::host_capabilities::clear_process_default_terminal_host_provider(),
        }
    }

    /// V-01/V-05: no provider → the engine-only default (not an error / not
    /// all-empty). Window and config defaults match the pre-extraction no-UI
    /// arm; handle 0 queries return the frozen sentinels.
    #[test]
    fn no_provider_falls_back_to_engine_default() {
        use crate::vm::ffi::term_engine::*;
        let vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 256);
        assert!(vm.terminal_provider().is_none());

        let mut task = AutoTask::new(0, 256, 0);
        shim_term_window_width(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 1024);
        shim_term_window_height(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 768);
        shim_term_config_profiles(&mut task, &vm).unwrap();
        assert!(pop_list(&vm, &mut task).is_empty());
        shim_term_config_default_profile(&mut task, &vm).unwrap();
        assert_eq!(pop_str(&vm, &mut task), "");
        shim_term_menu_take(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), -1);
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_apply_resize(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 0);
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_pump_input(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 0);
        push_str(&vm, &mut task, "k");
        push_str(&vm, &mut task, "80x30");
        shim_term_engine_pend_resize_geom(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 0);
        task.ram.push_nv(auto_val::encode_i32(0));
        push_str(&vm, &mut task, "k");
        shim_term_scroll_pending_for(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 0);
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_interrupt(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), -1);
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_is_exited(&mut task, &vm).unwrap();
        assert!(pop_bool(&mut task));
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_rows(&mut task, &vm).unwrap();
        assert!(pop_list(&vm, &mut task).is_empty());
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_cursor_row(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 0);
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_viewport_cols(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 0);
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_engine_history(&mut task, &vm).unwrap();
        assert_eq!(pop_i32(&mut task), 0);
    }
}

// ── V-05: deterministic absent-capability matrix ─────────────────────────

mod plan808_absent {
    use std::sync::Arc;

    use auto_terminal_core::EngineService;

    use crate::vm::engine::AutoVM;
    use crate::vm::task::AutoTask;
    use crate::vm::virt_memory::VirtualFlash;

    fn vm_with_service(service: Arc<EngineService>) -> AutoVM {
        let vm = AutoVM::new(VirtualFlash::new(0), 256);
        vm.set_terminal_provider(service);
        vm
    }

    /// No DLL: spawn=0, rows=[], interrupt=-1, is_exited=true, history=0,
    /// backlog=0, cursor/viewport=(0,0), config empty, pump/apply=0, menu=-1.
    #[test]
    fn no_dll_legacy_sentinels_through_shims() {
        use crate::vm::ffi::term_engine::*;
        let vm = vm_with_service(Arc::new(EngineService::without_dll()));
        let mut task = AutoTask::new(0, 256, 0);

        task.ram.push_nv(auto_val::encode_i32(80));
        task.ram.push_nv(auto_val::encode_i32(24));
        shim_term_spawn(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), 0, "spawn failure sentinel");
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_rows(&mut task, &vm).unwrap();
        let id = task.ram.pop_i32() as u64;
        let obj = vm.get_heap_object(id).expect("rows returns a heap list");
        let guard = obj.read().unwrap();
        let list = guard
            .as_any()
            .downcast_ref::<crate::vm::types::ListData<i32>>()
            .expect("ListData<i32>");
        assert_eq!(list.len(), 0, "absent engine rows is an empty list");
        drop(guard);
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_interrupt(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), -1);
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_is_exited(&mut task, &vm).unwrap();
        assert!(auto_val::decode_bool(task.ram.pop_nv()));
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_engine_history(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), 0);
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_backlog_pending_mb(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), 0);
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_backlog_paused(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), 0);
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_backlog_take_alerts(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), 0);
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_backlog_dropped(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), 0);
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_menu_take(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), -1);
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_cursor_row(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), 0);
        task.ram.push_nv(auto_val::encode_i32(0));
        shim_term_viewport_rows(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), 0);
    }

    /// A loadable library that is not the engine: required symbol missing
    /// panics (legacy `.expect`), optional symbols degrade to their old
    /// values (spawn_ex 0 / palette empty).
    #[cfg(windows)]
    #[test]
    fn non_engine_library_missing_symbol_contract() {
        let kernel32 = std::path::PathBuf::from(
            std::env::var("SystemRoot").unwrap_or_else(|_| "C:/Windows".to_string())
                + "/System32/kernel32.dll",
        );
        assert!(kernel32.is_file(), "kernel32.dll must exist");
        let service = EngineService::with_candidates(vec![kernel32]);
        assert!(service.has_library(), "kernel32 loads as a library");

        // Optional symbol: spawn_ex degrades to 0.
        assert_eq!(service.spawn_ex("cmd", &[], "", 80, 24), 0);
        // Optional palette symbol: empty table (UI keeps built-in fallback).
        assert!(service.palette_tables().is_empty());

        // Required symbol: legacy panic (not a silent value).
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| service.spawn(80, 24)));
        assert!(
            result.is_err(),
            "missing required symbol must panic (legacy)"
        );
    }
}

// ── V-02/V-03/V-04: real DLL acceptance ─────────────────────────────────

mod plan808_real_dll {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc, Mutex};
    use std::time::{Duration, Instant};

    use auto_terminal_core::{EngineService, TerminalHostProvider, TerminalRetiredCache};

    use crate::vm::engine::AutoVM;
    use crate::vm::task::AutoTask;
    use crate::vm::virt_memory::VirtualFlash;

    const DEADLINE: Duration = Duration::from_secs(20);

    fn dll_path() -> Option<PathBuf> {
        match std::env::var("AUTOTERM_ENGINE_DLL") {
            Ok(p) if PathBuf::from(&p).is_file() => Some(PathBuf::from(p)),
            _ => {
                eprintln!(
                    "PENDING(plan808): AUTOTERM_ENGINE_DLL is not set/does not exist — \
                     real-DLL lifecycle evidence not produced in this run"
                );
                None
            }
        }
    }

    fn service_with_dll(path: &PathBuf) -> Arc<EngineService> {
        Arc::new(EngineService::with_candidates(vec![path.clone()]))
    }

    fn wait_marker(service: &EngineService, handle: i64, marker: &str) -> Vec<String> {
        let end = Instant::now() + DEADLINE;
        let mut last = Vec::new();
        while Instant::now() < end {
            last = service.rows(handle);
            if last.iter().any(|l| l.contains(marker)) {
                return last;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        last
    }

    fn wait_exited(service: &EngineService, handle: i64) -> bool {
        let end = Instant::now() + DEADLINE;
        while Instant::now() < end {
            if service.is_exited(handle) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(30));
        }
        false
    }

    /// Drop guard: asks the child to exit and frees the session even when an
    /// assertion below panics (bounded cleanup, no leaked PTY).
    struct Reaper<'a> {
        service: &'a EngineService,
        handles: Vec<i64>,
    }

    impl<'a> Reaper<'a> {
        fn new(service: &'a EngineService, handles: Vec<i64>) -> Self {
            Self { service, handles }
        }
        fn add(&mut self, handle: i64) {
            self.handles.push(handle);
        }
        fn disarm(mut self) {
            self.handles.clear();
        }
    }

    impl Drop for Reaper<'_> {
        fn drop(&mut self) {
            for &h in &self.handles {
                if h != 0 {
                    self.service.write_line(h, "exit");
                    self.service.free(h);
                }
            }
        }
    }

    /// V-04: real PTY lifecycle — two sessions, natural exit, interactive
    /// exit, duplicate free, post-free reads; every child is reaped.
    #[test]
    fn real_dll_session_lifecycle() {
        let Some(path) = dll_path() else { return };
        let service = service_with_dll(&path);
        assert!(service.has_library(), "explicit DLL must load");

        // Natural-exit session.
        let h1 = service.spawn_ex(
            "cmd.exe",
            &["/C".into(), "echo PLAN808_DLL_A".into()],
            "D:/autostack/auto-lang",
            80,
            24,
        );
        assert_ne!(h1, 0, "spawn_ex must return a handle");
        let mut reaper = Reaper::new(&service, vec![h1]);
        let rows = wait_marker(&service, h1, "PLAN808_DLL_A");
        assert!(
            rows.iter().any(|l| l.contains("PLAN808_DLL_A")),
            "marker row missing: {rows:?}"
        );
        assert!(wait_exited(&service, h1), "natural exit not observed");
        service.free(h1);
        assert!(service.rows(h1).is_empty(), "freed handle reads empty rows");
        assert!(service.is_exited(h1), "freed handle is exited");
        service.free(h1); // repeated free is a no-op

        // Interactive session B + second session C isolation.
        let hb = service.spawn_ex(
            "cmd.exe",
            &["/K".into(), "echo PLAN808_DLL_B".into()],
            "D:/autostack/auto-lang",
            80,
            24,
        );
        assert_ne!(hb, 0);
        reaper.add(hb);
        wait_marker(&service, hb, "PLAN808_DLL_B");
        let hc = service.spawn_ex(
            "cmd.exe",
            &["/C".into(), "echo PLAN808_DLL_C".into()],
            "D:/autostack/auto-lang",
            60,
            12,
        );
        assert_ne!(hc, 0);
        reaper.add(hc);
        assert_ne!(hb, hc, "two sessions get distinct handles");
        let rows_c = wait_marker(&service, hc, "PLAN808_DLL_C");
        assert!(rows_c.iter().any(|l| l.contains("PLAN808_DLL_C")));
        let rows_b = wait_marker(&service, hb, "PLAN808_DLL_B");
        assert!(
            !rows_b.iter().any(|l| l.contains("PLAN808_DLL_C")),
            "session B must not see C's output"
        );
        assert!(
            !rows_c.iter().any(|l| l.contains("PLAN808_DLL_B")),
            "session C must not see B's output"
        );
        service.free(hc);

        // Interrupt and resize are callable on the live session.
        let intr = service.interrupt(hb);
        assert!(intr == 0 || intr == 1, "interrupt tri-state: {intr}");
        service.resize(hb, 100, 30);
        let rows_after_resize = service.rows(hb);
        assert!(rows_after_resize.len() >= 12, "resize keeps a viewport");

        // Graceful exit, then free.
        service.write_line(hb, "exit");
        assert!(wait_exited(&service, hb), "interactive exit not observed");
        service.free(hb);

        assert_eq!(
            service.live_handle_count(),
            0,
            "private handle table drained"
        );
        assert_eq!(service.cached_snapshot_count(), 0, "snapshot cache drained");
        reaper.disarm();
    }

    fn shim_rows(vm: &AutoVM, handle: i64) -> Vec<String> {
        let mut task = AutoTask::new(0, 1024, 0);
        task.ram.push_nv(auto_val::encode_i32(handle as i32));
        crate::vm::ffi::term_engine::shim_term_rows(&mut task, vm).unwrap();
        let id = task.ram.pop_i32() as u64;
        let obj = vm.get_heap_object(id).expect("heap list");
        let guard = obj.read().unwrap();
        let list = guard
            .as_any()
            .downcast_ref::<crate::vm::types::ListData<i32>>()
            .expect("ListData<i32>");
        let strings = vm.strings.read().unwrap();
        (0..list.len())
            .filter_map(|i| {
                let v = list.get(i).copied().unwrap_or(0);
                (v < 0)
                    .then(|| strings.get((-v - 1) as usize))
                    .flatten()
                    .map(|b| String::from_utf8_lossy(b).into_owned())
            })
            .collect()
    }

    fn shim_write_line(vm: &AutoVM, handle: i64, line: &str) {
        use crate::vm::ffi::VMConvertible;
        let mut task = AutoTask::new(0, 1024, 0);
        task.ram.push_nv(auto_val::encode_i32(handle as i32));
        line.to_string().push_to_stack(&mut task, vm).unwrap();
        crate::vm::ffi::term_engine::shim_term_write_line(&mut task, vm).unwrap();
    }

    fn shim_free(vm: &AutoVM, handle: i64) {
        let mut task = AutoTask::new(0, 1024, 0);
        task.ram.push_nv(auto_val::encode_i32(handle as i32));
        crate::vm::ffi::term_engine::shim_term_free(&mut task, vm).unwrap();
    }

    fn wait_until(mut f: impl FnMut() -> bool, timeout: Duration) -> bool {
        let end = Instant::now() + timeout;
        while Instant::now() < end {
            if f() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(30));
        }
        false
    }

    #[cfg(windows)]
    fn pid_alive(pid: u32) -> bool {
        let out = std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
            .output()
            .expect("tasklist");
        if !out.status.success() {
            return false;
        }
        String::from_utf8_lossy(&out.stdout).lines().any(|line| {
            line.split(',')
                .nth(1)
                .map(|field| field.trim_matches('"').trim() == pid.to_string())
                .unwrap_or(false)
        })
    }

    /// F808-R1-01 permanent regression: two services with **live** sessions
    /// and process-globally unique handles; after the VM switches A→B, a
    /// handle exposed by A still reads/writes/frees through A. Duplicate free
    /// does not touch B; a second VM and the shared-engine fallback are
    /// covered. (The previous test used a DLL-less B, which masked the
    /// collision.)
    #[test]
    fn provider_replacement_keeps_old_handle_bound_to_owner() {
        let Some(path) = dll_path() else { return };
        let svc_a = service_with_dll(&path);
        let svc_b = service_with_dll(&path);
        let ha = svc_a.spawn_ex(
            "cmd.exe",
            &["/K".into(), "echo PLAN808_OWNER_A".into()],
            "D:/autostack/auto-lang",
            80,
            24,
        );
        let hb = svc_b.spawn_ex(
            "cmd.exe",
            &["/K".into(), "echo PLAN808_OWNER_B".into()],
            "D:/autostack/auto-lang",
            80,
            24,
        );
        assert_ne!(ha, 0);
        assert_ne!(hb, 0);
        assert_ne!(ha, hb, "handle namespace must be process-globally unique");
        let reaper_a = Reaper::new(&svc_a, vec![ha]);
        let reaper_b = Reaper::new(&svc_b, vec![hb]);
        assert!(wait_marker(&svc_a, ha, "PLAN808_OWNER_A")
            .iter()
            .any(|l| l.contains("PLAN808_OWNER_A")));
        assert!(wait_marker(&svc_b, hb, "PLAN808_OWNER_B")
            .iter()
            .any(|l| l.contains("PLAN808_OWNER_B")));

        let vm = AutoVM::new(VirtualFlash::new(0), 1024);
        vm.set_terminal_provider(svc_a.clone());
        vm.set_terminal_provider(svc_b.clone());

        // Read the pre-switch handle: owner A, not the current B.
        let rows_a = shim_rows(&vm, ha);
        assert!(rows_a.iter().any(|l| l.contains("PLAN808_OWNER_A")));
        assert!(!rows_a.iter().any(|l| l.contains("PLAN808_OWNER_B")));

        // Write the pre-switch handle: A receives, B does not.
        shim_write_line(&vm, ha, "echo PLAN808_OWNER_A2");
        assert!(wait_marker(&svc_a, ha, "PLAN808_OWNER_A2")
            .iter()
            .any(|l| l.contains("PLAN808_OWNER_A2")));
        assert!(!svc_b
            .rows(hb)
            .iter()
            .any(|l| l.contains("PLAN808_OWNER_A2")));

        // Free the pre-switch handle: A only; duplicate free is a no-op.
        shim_free(&vm, ha);
        assert_eq!(svc_a.live_handle_count(), 0, "A's session is freed");
        assert_eq!(svc_b.live_handle_count(), 1, "B's session is untouched");
        shim_free(&vm, ha);
        assert_eq!(
            svc_b.live_handle_count(),
            1,
            "duplicate free cannot touch B"
        );

        // B's handle routes to B through the same VM and a second VM.
        assert!(shim_rows(&vm, hb)
            .iter()
            .any(|l| l.contains("PLAN808_OWNER_B")));
        let vm2 = AutoVM::new(VirtualFlash::new(0), 1024);
        vm2.set_terminal_provider(svc_b.clone());
        assert!(shim_rows(&vm2, hb)
            .iter()
            .any(|l| l.contains("PLAN808_OWNER_B")));
        shim_free(&vm2, hb);
        assert_eq!(svc_b.live_handle_count(), 0);
        reaper_a.disarm();
        reaper_b.disarm();

        // Default shared-engine fallback still owns and frees its handles.
        let shared = auto_terminal_core::shared();
        let hs = shared.spawn_ex(
            "cmd.exe",
            &["/C".into(), "echo PLAN808_OWNER_SHARED".into()],
            "D:/autostack/auto-lang",
            80,
            24,
        );
        assert_ne!(hs, 0);
        assert!(wait_marker(&shared, hs, "PLAN808_OWNER_SHARED")
            .iter()
            .any(|l| l.contains("PLAN808_OWNER_SHARED")));
        let bare = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 1024);
        let rows_s = shim_rows(&bare, hs);
        assert!(rows_s.iter().any(|l| l.contains("PLAN808_OWNER_SHARED")));
        shim_free(&bare, hs);
        assert!(!shared.owns_handle(hs));
    }

    fn vm_cache(which: &str, vm: &AutoVM, handle: i64) -> i32 {
        let mut task = AutoTask::new(0, 1024, 0);
        task.ram.push_nv(auto_val::encode_i32(handle as i32));
        match which {
            "cursor_row" => crate::vm::ffi::term_engine::shim_term_cursor_row(&mut task, vm),
            "cursor_col" => crate::vm::ffi::term_engine::shim_term_cursor_col(&mut task, vm),
            "viewport_cols" => crate::vm::ffi::term_engine::shim_term_viewport_cols(&mut task, vm),
            "viewport_rows" => crate::vm::ffi::term_engine::shim_term_viewport_rows(&mut task, vm),
            _ => unreachable!(),
        }
        .unwrap();
        task.ram.pop_i32()
    }

    /// F808-R2-02 permanent regression: the frozen cursor/viewport cache of a
    /// freed handle survives provider replacement and owner pruning. Both
    /// orders (free→switch and switch→free) and the retired/unknown/repeat
    /// free controls are covered; the cached values are copied into the VM,
    /// so the old service need not stay alive.
    #[test]
    fn freed_handle_cache_survives_provider_switch() {
        let Some(path) = dll_path() else { return };
        let svc_a = service_with_dll(&path);
        let svc_b = Arc::new(EngineService::without_dll());
        let vm = AutoVM::new(VirtualFlash::new(0), 1024);

        // Order 1: free while A is still current, then switch to B.
        let h1 = svc_a.spawn_ex("cmd.exe", &["/C".into(), "exit 0".into()], "", 80, 24);
        assert_ne!(h1, 0);
        let reaper = Reaper::new(&svc_a, vec![h1]);
        vm.set_terminal_provider(svc_a.clone());
        assert_eq!(vm_cache("viewport_cols", &vm, h1), 80);
        assert_eq!(vm_cache("viewport_rows", &vm, h1), 24);
        shim_free(&vm, h1);
        vm.set_terminal_provider(svc_b.clone());
        assert_eq!(
            vm_cache("viewport_cols", &vm, h1),
            80,
            "free→switch must keep the frozen viewport"
        );
        assert_eq!(vm_cache("viewport_rows", &vm, h1), 24);
        assert_eq!(vm_cache("cursor_row", &vm, h1), 0);
        assert_eq!(vm_cache("cursor_col", &vm, h1), 0);
        shim_free(&vm, h1); // repeat free is a no-op
        assert_eq!(vm_cache("viewport_cols", &vm, h1), 80);

        // Order 2: switch to B first, then free the old handle; capture a
        // non-zero cursor from a live feed before free.
        let h2 = svc_a.spawn_ex(
            "cmd.exe",
            &["/K".into(), "echo PLAN808_CACHE".into()],
            "D:/autostack/auto-lang",
            90,
            30,
        );
        assert_ne!(h2, 0);
        let reaper2 = Reaper::new(&svc_a, vec![h2]);
        vm.set_terminal_provider(svc_a.clone());
        assert!(wait_marker(&svc_a, h2, "PLAN808_CACHE")
            .iter()
            .any(|l| l.contains("PLAN808_CACHE")));
        let expected_cursor = svc_a.cursor(h2);
        assert_eq!(vm_cache("viewport_cols", &vm, h2), 90);
        assert_eq!(vm_cache("viewport_rows", &vm, h2), 30);
        vm.set_terminal_provider(svc_b.clone());
        shim_free(&vm, h2); // resolves owner A, captures 90×30 + cursor
        assert_eq!(
            vm_cache("viewport_cols", &vm, h2),
            90,
            "switch→free must keep the frozen viewport"
        );
        assert_eq!(vm_cache("viewport_rows", &vm, h2), 30);
        assert_eq!(vm_cache("cursor_row", &vm, h2), expected_cursor.0 as i32);
        assert_eq!(vm_cache("cursor_col", &vm, h2), expected_cursor.1 as i32);

        // Unknown handle keeps the legacy 0×0 absent values.
        let unknown = i32::MAX - 7;
        assert_eq!(vm_cache("viewport_cols", &vm, unknown as i64), 0);
        assert_eq!(vm_cache("viewport_rows", &vm, unknown as i64), 0);
        assert_eq!(vm_cache("cursor_row", &vm, unknown as i64), 0);
        assert_eq!(vm_cache("cursor_col", &vm, unknown as i64), 0);

        reaper.disarm();
        reaper2.disarm();
    }

    /// F808-R1-04 permanent regression: dropping the last reference to an
    /// isolated service frees its live sessions (the real OS child exits).
    /// The child publishes its own PID; the test waits for the OS process to
    /// disappear, not merely for the private table to empty.
    #[cfg(windows)]
    #[test]
    fn service_drop_frees_live_session_and_child() {
        struct ChildGuard(Option<u32>);
        impl Drop for ChildGuard {
            fn drop(&mut self) {
                if let Some(pid) = self.0 {
                    let _ = std::process::Command::new("taskkill")
                        .args(["/PID", &pid.to_string(), "/F", "/T"])
                        .output();
                }
            }
        }

        let Some(path) = dll_path() else { return };
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let pid_file =
            std::env::temp_dir().join(format!("plan808-drop-{}-{stamp}.pid", std::process::id()));
        let _ = std::fs::remove_file(&pid_file);
        let script = format!(
            "[System.IO.File]::WriteAllText('{}', [string]$PID); Start-Sleep -Seconds 60",
            pid_file.display()
        );
        let service = service_with_dll(&path);
        let h = service.spawn_ex(
            "powershell.exe",
            &[
                "-NoProfile".into(),
                "-NonInteractive".into(),
                "-Command".into(),
                script,
            ],
            "D:/autostack/auto-lang",
            80,
            24,
        );
        assert_ne!(h, 0);
        assert!(
            wait_until(|| pid_file.is_file(), DEADLINE),
            "child must publish its PID"
        );
        let pid: u32 = std::fs::read_to_string(&pid_file)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        let mut guard = ChildGuard(Some(pid));
        assert!(pid_alive(pid), "child must be alive before drop");
        drop(service);
        let reaped = wait_until(|| !pid_alive(pid), Duration::from_secs(10));
        if reaped {
            guard.0 = None;
        }
        let _ = std::fs::remove_file(&pid_file);
        assert!(
            reaped,
            "dropping the last service reference must free the session and reap the child"
        );
    }

    /// F808-R1-04 (provider replacement last reference): the old service is
    /// kept alive by the VM owner history while its handle is routed, then
    /// released (and its sessions freed) when the handle is freed.
    #[cfg(windows)]
    #[test]
    fn provider_replacement_last_reference_releases_old_service() {
        struct ChildGuard(Option<u32>);
        impl Drop for ChildGuard {
            fn drop(&mut self) {
                if let Some(pid) = self.0 {
                    let _ = std::process::Command::new("taskkill")
                        .args(["/PID", &pid.to_string(), "/F", "/T"])
                        .output();
                }
            }
        }

        let Some(path) = dll_path() else { return };
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let pid_file = std::env::temp_dir().join(format!(
            "plan808-replace-{}-{stamp}.pid",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&pid_file);
        let script = format!(
            "[System.IO.File]::WriteAllText('{}', [string]$PID); Start-Sleep -Seconds 60",
            pid_file.display()
        );
        let service = service_with_dll(&path);
        let h = service.spawn_ex(
            "powershell.exe",
            &[
                "-NoProfile".into(),
                "-NonInteractive".into(),
                "-Command".into(),
                script,
            ],
            "D:/autostack/auto-lang",
            80,
            24,
        );
        assert_ne!(h, 0);
        assert!(
            wait_until(|| pid_file.is_file(), DEADLINE),
            "child must publish its PID"
        );
        let pid: u32 = std::fs::read_to_string(&pid_file)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        let mut guard = ChildGuard(Some(pid));

        let vm = AutoVM::new(VirtualFlash::new(0), 1024);
        vm.set_terminal_provider(service.clone());
        drop(service); // VM slot + owner history keep the service alive
        let replacement = Arc::new(EngineService::without_dll());
        vm.set_terminal_provider(replacement);
        // The old handle still routes to the original owner while alive.
        assert!(pid_alive(pid), "old service must still be alive");
        shim_free(&vm, h); // frees + prunes the last owner reference -> Drop
        let reaped = wait_until(|| !pid_alive(pid), Duration::from_secs(10));
        if reaped {
            guard.0 = None;
        }
        let _ = std::fs::remove_file(&pid_file);
        assert!(reaped, "last owner reference release must reap the child");
    }

    /// V-02/V-03: real UI adapter chain — directional input, per-pane rows,
    /// geometry, menu once-consumption, display_offset single source, and the
    /// window-store projection. Every session is reaped.
    #[cfg(feature = "ui")]
    #[test]
    fn real_ui_adapter_directional_and_scroll_projection() {
        use crate::ui::terminal::{
            terminal, terminal_dispose, terminal_history, terminal_push_input,
            terminal_queue_scroll_delta, terminal_scroll_delta_pending, terminal_scroll_offset,
            terminal_set_menu_item, terminal_take_menu_item, terminal_window_anchor,
            terminal_window_row,
        };
        use crate::ui::terminal_provider::TerminalUiProvider;

        let Some(path) = dll_path() else { return };
        let service = service_with_dll(&path);
        let provider = Arc::new(TerminalUiProvider::new(service.clone()));
        let vm = AutoVM::new(VirtualFlash::new(0), 2048);
        vm.set_terminal_provider(provider.clone());

        let key_a = "p808-pane-a";
        let key_b = "p808-pane-b";
        terminal_dispose(key_a);
        terminal_dispose(key_b);
        let core_a = terminal(key_a, 80, 24);
        let core_b = terminal(key_b, 80, 24);

        let ha = service.spawn_ex(
            "cmd.exe",
            &["/K".into(), "echo PLAN808_PANE_A".into()],
            "D:/autostack/auto-lang",
            80,
            24,
        );
        let hb = service.spawn_ex(
            "cmd.exe",
            &["/K".into(), "echo PLAN808_PANE_B".into()],
            "D:/autostack/auto-lang",
            80,
            24,
        );
        assert_ne!(ha, 0);
        assert_ne!(hb, 0);
        let reaper = Reaper::new(&service, vec![ha, hb]);

        // Directional input: queue different payloads, pump only A.
        terminal_push_input(core_a, "echo PLAN808_ONLY_A\r");
        terminal_push_input(core_b, "echo PLAN808_ONLY_B\r");
        let mut task = AutoTask::new(0, 2048, 0);
        // pump_for(handle, key)
        task.ram.push_nv(auto_val::encode_i32(ha as i32));
        let idx = vm.add_string(key_a.as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, idx);
        crate::vm::ffi::term_engine::shim_term_pump_for(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), 1, "A pump consumes one payload");
        let rows_a = wait_marker(&service, ha, "PLAN808_ONLY_A");
        assert!(rows_a.iter().any(|l| l.contains("PLAN808_ONLY_A")));
        assert_eq!(
            terminal_scroll_delta_pending(core_a),
            0,
            "pump has no pending scroll"
        );

        // B still has its payload; pumping A again drains nothing.
        let mut task = AutoTask::new(0, 2048, 0);
        task.ram.push_nv(auto_val::encode_i32(ha as i32));
        let idx = vm.add_string(key_a.as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, idx);
        crate::vm::ffi::term_engine::shim_term_pump_for(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), 0, "A queue is empty");

        // Harvest both panes through the adapter (styled cells projected).
        let mut task = AutoTask::new(0, 2048, 0);
        task.ram.push_nv(auto_val::encode_i32(ha as i32));
        let idx = vm.add_string(key_a.as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, idx);
        crate::vm::ffi::term_engine::shim_term_rows_for(&mut task, &vm).unwrap();
        let lines_a = {
            let id = task.ram.pop_i32() as u64;
            let obj = vm.get_heap_object(id).unwrap();
            let guard = obj.read().unwrap();
            let list = guard
                .as_any()
                .downcast_ref::<crate::vm::types::ListData<i32>>()
                .unwrap();
            let strings = vm.strings.read().unwrap();
            (0..list.len())
                .filter_map(|i| {
                    let v = list.get(i).copied().unwrap_or(0);
                    (v < 0)
                        .then(|| strings.get((-v - 1) as usize))
                        .flatten()
                        .map(|b| String::from_utf8_lossy(b).into_owned())
                })
                .collect::<Vec<_>>()
        };
        assert!(lines_a.iter().any(|l| l.contains("ONLY_A")));
        let mut task = AutoTask::new(0, 2048, 0);
        task.ram.push_nv(auto_val::encode_i32(hb as i32));
        let idx = vm.add_string(key_b.as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, idx);
        crate::vm::ffi::term_engine::shim_term_rows_for(&mut task, &vm).unwrap();
        let _ = task.ram.pop_nv();
        assert!(
            !core_b.lines().iter().any(|l| l.contains("ONLY_A")),
            "pane B must not receive pane A's projection"
        );
        // Styled cells arrived for A (engine row → UI cell projection).
        let styled_found = (0..core_a.rows as usize).any(|r| {
            core_a
                .line(r)
                .map(|t| t.contains("ONLY_A"))
                .unwrap_or(false)
                && core_a
                    .row_cells(r)
                    .map(|cells| !cells.is_empty())
                    .unwrap_or(false)
        });
        assert!(styled_found, "styled sideband cells must reach the pane");

        // Geometry: pending request on A is consumed only by A.
        crate::ui::terminal::terminal_request_resize(core_a, 100, 30);
        let mut task = AutoTask::new(0, 2048, 0);
        task.ram.push_nv(auto_val::encode_i32(hb as i32));
        let idx = vm.add_string(key_b.as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, idx);
        crate::vm::ffi::term_engine::shim_term_apply_resize_for(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), 0, "B must not take A's geometry");
        let mut task = AutoTask::new(0, 2048, 0);
        task.ram.push_nv(auto_val::encode_i32(ha as i32));
        let idx = vm.add_string(key_a.as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, idx);
        crate::vm::ffi::term_engine::shim_term_apply_resize_for(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), 1);
        let mut task = AutoTask::new(0, 2048, 0);
        task.ram.push_nv(auto_val::encode_i32(ha as i32));
        crate::vm::ffi::term_engine::shim_term_viewport_cols(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), 100, "viewport refresh after apply");
        let mut task = AutoTask::new(0, 2048, 0);
        task.ram.push_nv(auto_val::encode_i32(ha as i32));
        crate::vm::ffi::term_engine::shim_term_viewport_rows(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), 30);

        // Menu payload: once-consumed, key-order stable.
        terminal_set_menu_item(core_b, 2);
        assert_eq!(terminal_take_menu_item(core_b), Some(2));
        assert_eq!(terminal_take_menu_item(core_b), None, "take clears");
        terminal_set_menu_item(core_b, 3);
        let mut task = AutoTask::new(0, 2048, 0);
        crate::vm::ffi::term_engine::shim_term_menu_take(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), 3);
        let mut task = AutoTask::new(0, 2048, 0);
        crate::vm::ffi::term_engine::shim_term_menu_take(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), -1, "second take is empty");

        // Scroll single source: fill history, queue a wheel delta, harvest.
        service.write_line(ha, "for /L %i in (1,1,120) do @echo PLAN808_HIST_%i");
        let rows_hist = wait_marker(&service, ha, "PLAN808_HIST_120");
        assert!(rows_hist.iter().any(|l| l.contains("PLAN808_HIST_120")));
        terminal_queue_scroll_delta(core_a, 3);
        assert_eq!(
            terminal_scroll_delta_pending(core_a),
            3,
            "peek before drain"
        );
        let mut task = AutoTask::new(0, 2048, 0);
        task.ram.push_nv(auto_val::encode_i32(ha as i32));
        let idx = vm.add_string(key_a.as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, idx);
        crate::vm::ffi::term_engine::shim_term_rows_for(&mut task, &vm).unwrap();
        let _ = task.ram.pop_nv();
        assert_eq!(terminal_scroll_delta_pending(core_a), 0, "drain on harvest");
        let engine_offset = service.scroll_offset(ha).max(0) as usize;
        assert_eq!(
            terminal_scroll_offset(core_a),
            engine_offset,
            "UI offset is a projection of the engine display_offset"
        );
        let engine_history = service.history(ha).max(0) as usize;
        assert!(
            engine_history > 0,
            "120 output lines in a 24-row grid must create scrollback"
        );
        assert_eq!(terminal_history(core_a), engine_history);
        assert_eq!(
            terminal_window_anchor(core_a),
            engine_history as i64 - engine_offset as i64,
            "window anchor = history - display_offset"
        );
        assert!(
            terminal_window_row(core_a, engine_history as i64 - engine_offset as i64).is_some(),
            "window store carries the visible rows"
        );
        // 键入贴底: writing input returns the engine to offset 0.
        service.write_line(ha, "echo PLAN808_BOTTOM");
        let _ = wait_marker(&service, ha, "PLAN808_BOTTOM");
        let mut task = AutoTask::new(0, 2048, 0);
        task.ram.push_nv(auto_val::encode_i32(ha as i32));
        let idx = vm.add_string(key_a.as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, idx);
        crate::vm::ffi::term_engine::shim_term_scroll_pending_for(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), 0);
        let mut task = AutoTask::new(0, 2048, 0);
        task.ram.push_nv(auto_val::encode_i32(ha as i32));
        let idx = vm.add_string(key_a.as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, idx);
        crate::vm::ffi::term_engine::shim_term_rows_for(&mut task, &vm).unwrap();
        let _ = task.ram.pop_nv();
        assert_eq!(
            service.scroll_offset(ha),
            0,
            "input pins the view to bottom"
        );

        // Reap both sessions.
        service.write_line(ha, "exit");
        service.write_line(hb, "exit");
        assert!(wait_exited(&service, ha), "pane A exit");
        assert!(wait_exited(&service, hb), "pane B exit");
        service.free(ha);
        service.free(hb);
        assert_eq!(service.live_handle_count(), 0);
        terminal_dispose(key_a);
        terminal_dispose(key_b);
        reaper.disarm();
    }

    /// Test wrapper that pauses inside `live_handle_count` so a concurrent
    /// prune window can be observed deterministically (R3-F01). All other
    /// methods delegate to the real service.
    struct PausedCount {
        inner: Arc<EngineService>,
        armed: AtomicBool,
        entered: mpsc::SyncSender<()>,
        release: Mutex<mpsc::Receiver<()>>,
    }

    impl PausedCount {
        fn new(
            inner: Arc<EngineService>,
            entered: mpsc::SyncSender<()>,
            release: mpsc::Receiver<()>,
        ) -> Self {
            Self {
                inner,
                armed: AtomicBool::new(true),
                entered,
                release: Mutex::new(release),
            }
        }
    }

    impl TerminalHostProvider for PausedCount {
        fn spawn(&self, cols: i64, rows: i64) -> i32 {
            TerminalHostProvider::spawn(&*self.inner, cols, rows)
        }
        fn spawn_ex(&self, program: &str, argv: &[String], cwd: &str, cols: i64, rows: i64) -> i32 {
            TerminalHostProvider::spawn_ex(&*self.inner, program, argv, cwd, cols, rows)
        }
        fn write_line(&self, handle: i64, line: &str) {
            TerminalHostProvider::write_line(&*self.inner, handle, line)
        }
        fn write_raw(&self, handle: i64, vt: &str) {
            TerminalHostProvider::write_raw(&*self.inner, handle, vt)
        }
        fn rows(&self, handle: i64) -> Vec<String> {
            TerminalHostProvider::rows(&*self.inner, handle)
        }
        fn rows_for(&self, handle: i64, key: &str) -> Vec<String> {
            TerminalHostProvider::rows_for(&*self.inner, handle, key)
        }
        fn resize(&self, handle: i64, cols: i64, rows: i64) {
            TerminalHostProvider::resize(&*self.inner, handle, cols, rows)
        }
        fn apply_resize(&self, handle: i64) -> i32 {
            TerminalHostProvider::apply_resize(&*self.inner, handle)
        }
        fn apply_resize_for(&self, handle: i64, key: &str) -> i32 {
            TerminalHostProvider::apply_resize_for(&*self.inner, handle, key)
        }
        fn pump_input(&self, handle: i64) -> i32 {
            TerminalHostProvider::pump_input(&*self.inner, handle)
        }
        fn pump_for(&self, handle: i64, key: &str) -> i32 {
            TerminalHostProvider::pump_for(&*self.inner, handle, key)
        }
        fn scroll_pending_for(&self, handle: i64, key: &str) -> i32 {
            TerminalHostProvider::scroll_pending_for(&*self.inner, handle, key)
        }
        fn pend_resize_geom(&self, key: &str, geom: &str) -> i32 {
            TerminalHostProvider::pend_resize_geom(&*self.inner, key, geom)
        }
        fn menu_take(&self) -> i32 {
            TerminalHostProvider::menu_take(&*self.inner)
        }
        fn interrupt(&self, handle: i64) -> i32 {
            TerminalHostProvider::interrupt(&*self.inner, handle)
        }
        fn is_exited(&self, handle: i64) -> bool {
            TerminalHostProvider::is_exited(&*self.inner, handle)
        }
        fn free(&self, handle: i64) {
            TerminalHostProvider::free(&*self.inner, handle)
        }
        fn cursor_row(&self, handle: i64) -> i32 {
            TerminalHostProvider::cursor_row(&*self.inner, handle)
        }
        fn cursor_col(&self, handle: i64) -> i32 {
            TerminalHostProvider::cursor_col(&*self.inner, handle)
        }
        fn viewport_cols(&self, handle: i64) -> i32 {
            TerminalHostProvider::viewport_cols(&*self.inner, handle)
        }
        fn viewport_rows(&self, handle: i64) -> i32 {
            TerminalHostProvider::viewport_rows(&*self.inner, handle)
        }
        fn backlog_pending_mb(&self, handle: i64) -> i32 {
            TerminalHostProvider::backlog_pending_mb(&*self.inner, handle)
        }
        fn backlog_paused(&self, handle: i64) -> i32 {
            TerminalHostProvider::backlog_paused(&*self.inner, handle)
        }
        fn backlog_take_alerts(&self, handle: i64) -> i32 {
            TerminalHostProvider::backlog_take_alerts(&*self.inner, handle)
        }
        fn backlog_dropped(&self, handle: i64) -> i32 {
            TerminalHostProvider::backlog_dropped(&*self.inner, handle)
        }
        fn history(&self, handle: i64) -> i32 {
            TerminalHostProvider::history(&*self.inner, handle)
        }
        fn window_width(&self) -> i32 {
            TerminalHostProvider::window_width(&*self.inner)
        }
        fn window_height(&self) -> i32 {
            TerminalHostProvider::window_height(&*self.inner)
        }
        fn config_profiles(&self) -> Vec<String> {
            TerminalHostProvider::config_profiles(&*self.inner)
        }
        fn config_default_profile(&self) -> String {
            TerminalHostProvider::config_default_profile(&*self.inner)
        }
        fn config_profiles_full(&self) -> Vec<String> {
            TerminalHostProvider::config_profiles_full(&*self.inner)
        }
        fn config_spawn_program(&self, profile: &str) -> String {
            TerminalHostProvider::config_spawn_program(&*self.inner, profile)
        }
        fn config_spawn_argv(&self, profile: &str) -> Vec<String> {
            TerminalHostProvider::config_spawn_argv(&*self.inner, profile)
        }
        fn config_spawn_cwd(&self, profile: &str) -> String {
            TerminalHostProvider::config_spawn_cwd(&*self.inner, profile)
        }
        fn owns_handle(&self, handle: i64) -> bool {
            self.inner.owns_handle(handle)
        }
        fn live_handle_count(&self) -> usize {
            if self.armed.swap(false, Ordering::SeqCst) {
                self.entered.send(()).unwrap();
                let _ = self
                    .release
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(5));
            }
            self.inner.live_handle_count()
        }
        fn cached_cursor_viewport(&self, handle: i64) -> Option<TerminalRetiredCache> {
            self.inner.cached_cursor_viewport(handle)
        }
    }

    fn vm_viewport(vm: &AutoVM, handle: i64) -> (i32, i32) {
        (
            vm_cache("viewport_cols", vm, handle),
            vm_cache("viewport_rows", vm, handle),
        )
    }

    fn vm_cursor(vm: &AutoVM, handle: i64) -> (i32, i32) {
        (
            vm_cache("cursor_row", vm, handle),
            vm_cache("cursor_col", vm, handle),
        )
    }

    /// F808-R3-01 permanent regression: while prune evaluates
    /// `live_handle_count` outside the lock, the visible owner list must stay
    /// routable. A real VM read/free in that window must reach the original
    /// owner and release the live session (not merely restore the list later).
    #[test]
    fn prune_window_keeps_owner_routable_and_free_releases() {
        let Some(path) = dll_path() else { return };
        let svc = service_with_dll(&path);
        let h = svc.spawn_ex("cmd.exe", &["/C".into(), "exit 0".into()], "", 80, 24);
        assert_ne!(h, 0);
        let mut reaper = Reaper::new(&svc, vec![h]);
        let (entered_tx, entered_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::channel();
        let owner = Arc::new(PausedCount::new(svc.clone(), entered_tx, release_rx));
        let vm = AutoVM::new(VirtualFlash::new(0), 1024);
        vm.set_terminal_provider(owner);
        vm.set_terminal_provider(Arc::new(EngineService::without_dll()));
        let ni = vm.native_interface.clone();
        let worker = std::thread::spawn(move || ni.prune_terminal_owners());
        entered_rx
            .recv_timeout(Duration::from_secs(3))
            .expect("prune must reach live_handle_count");

        // Observed inside the pause: owner, viewport and a real free.
        assert!(
            vm.native_interface.terminal_owner_for(h).is_some(),
            "the visible owner list must stay routable during prune"
        );
        assert_eq!(vm_viewport(&vm, h), (80, 24));
        shim_free(&vm, h);
        assert_eq!(
            svc.live_handle_count(),
            0,
            "the VM free in the prune window must release the original session"
        );

        release_tx.send(()).unwrap();
        worker.join().unwrap();
        // The frozen values survive the prune/free.
        assert_eq!(vm_viewport(&vm, h), (80, 24));
        reaper.disarm();
    }

    /// F808-R3-02 permanent regression: the frozen cache is published
    /// process-wide at free time, so another VM or a direct core-free caller
    /// keeps the values even after its owner history is pruned, without a
    /// pre-read and without retaining the old service.
    #[test]
    fn retired_cache_is_shared_across_consumers_and_direct_free() {
        let Some(path) = dll_path() else { return };

        // Cross-VM: VM1 frees the shared session; VM2 had switched to B and
        // has not read the cache; its unknown-handle free triggers prune.
        let svc = service_with_dll(&path);
        let h = svc.spawn_ex(
            "cmd.exe",
            &["/K".into(), "echo PLAN808_SHARED_CACHE".into()],
            "D:/autostack/auto-lang",
            91,
            31,
        );
        assert_ne!(h, 0);
        let mut reaper = Reaper::new(&svc, vec![h]);
        assert!(wait_marker(&svc, h, "PLAN808_SHARED_CACHE")
            .iter()
            .any(|l| l.contains("PLAN808_SHARED_CACHE")));
        let expected_cursor = svc.cursor(h);
        let vm1 = AutoVM::new(VirtualFlash::new(0), 1024);
        let vm2 = AutoVM::new(VirtualFlash::new(0), 1024);
        let control = AutoVM::new(VirtualFlash::new(0), 1024);
        vm1.set_terminal_provider(svc.clone());
        vm2.set_terminal_provider(svc.clone());
        control.set_terminal_provider(svc.clone());
        let replacement = Arc::new(EngineService::without_dll());
        vm2.set_terminal_provider(replacement.clone());
        shim_free(&vm1, h);
        assert_eq!(svc.live_handle_count(), 0, "VM1 free releases A");
        control.set_terminal_provider(replacement);
        assert_eq!(
            vm_viewport(&control, h),
            (91, 31),
            "pre-read control before prune"
        );
        // Production prune path: an unknown-handle free is a no-op for the
        // old handle and must not change its frozen values.
        shim_free(&vm2, -7777);
        shim_free(&control, -7777);
        assert_eq!(vm_viewport(&vm1, h), (91, 31));
        assert_eq!(
            vm_viewport(&vm2, h),
            (91, 31),
            "another VM after prune must keep the frozen viewport"
        );
        assert_eq!(vm_viewport(&control, h), (91, 31));
        assert_eq!(
            vm_cursor(&vm2, h),
            (expected_cursor.0 as i32, expected_cursor.1 as i32),
            "cursor values are shared too"
        );
        reaper.disarm();

        // Direct core free (no shim in the consuming VM), then prune, then
        // the first read.
        let svc2 = service_with_dll(&path);
        let h2 = svc2.spawn_ex("cmd.exe", &["/C".into(), "exit 0".into()], "", 92, 32);
        assert_ne!(h2, 0);
        let mut reaper2 = Reaper::new(&svc2, vec![h2]);
        let vm = AutoVM::new(VirtualFlash::new(0), 1024);
        let control2 = AutoVM::new(VirtualFlash::new(0), 1024);
        vm.set_terminal_provider(svc2.clone());
        control2.set_terminal_provider(svc2.clone());
        vm.set_terminal_provider(Arc::new(EngineService::without_dll()));
        svc2.free(h2);
        assert_eq!(svc2.live_handle_count(), 0);
        control2.set_terminal_provider(Arc::new(EngineService::without_dll()));
        assert_eq!(
            vm_viewport(&control2, h2),
            (92, 32),
            "direct-free pre-read control"
        );
        shim_free(&vm, -7777);
        shim_free(&control2, -7777);
        assert_eq!(
            vm_viewport(&vm, h2),
            (92, 32),
            "direct core free must publish the frozen viewport"
        );
        assert_eq!(vm_viewport(&control2, h2), (92, 32));
        assert_eq!(vm_cursor(&vm, h2), (0, 0));
        reaper2.disarm();
    }
}

// ── F808-R1-02: production VmBridge constructors install the adapter ─────

#[cfg(feature = "ui")]
mod plan808_vm_bridge_assembly {
    use std::collections::HashMap;
    use std::sync::Arc;

    use auto_terminal_core::{EngineService, TerminalHostProvider};

    use crate::ui::terminal::{
        terminal, terminal_dispose, terminal_drain_inputs_for, terminal_push_input,
    };
    use crate::ui::vm_bridge::VmBridge;
    use crate::vm::host_capabilities as host;
    use crate::vm::task::AutoTask;

    fn minimal_widget() -> crate::aura::AuraWidget {
        use crate::aura::{AuraNode, AuraWidget};
        AuraWidget {
            name: "Plan808Bridge".into(),
            named_views: vec![],
            actions: None,
            state_vars: vec![],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: Default::default(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            setup: None,
            tick_interval: None,
            timers: vec![],
            handler_params: Default::default(),
            span_map: Default::default(),
            key_bindings: Default::default(),
            api_imports: vec![],
            style_css: None,
            ext_imports: vec![],
            watchers: vec![],
            exposes: vec![],
        }
    }

    fn pump_queued_input(vm: &crate::vm::engine::AutoVM, key: &str) -> i32 {
        let core = terminal(key, 80, 24);
        terminal_push_input(core, "x");
        let mut task = AutoTask::new(0, 1024, 0);
        task.ram.push_nv(auto_val::encode_i32(0));
        crate::vm::ffi::term_engine::shim_term_pump_input(&mut task, vm).unwrap();
        let pumped = task.ram.pop_i32();
        assert!(
            terminal_drain_inputs_for(core).is_empty(),
            "the component's own VM must drain the input queue"
        );
        pumped
    }

    /// F808-R1-02: both production constructors install the terminal adapter
    /// before constructing their AutoVM, so the component's own VM snapshot
    /// consumes input. Not a manual `set_terminal_provider` test.
    #[test]
    fn both_bridge_constructors_install_terminal_adapter() {
        let _slot = host::test_default_slot_lock();
        let saved = host::process_default_terminal_host_provider();
        host::clear_process_default_terminal_host_provider();
        terminal_dispose("p808-bridge-new");
        terminal_dispose("p808-bridge-decl");

        // VmBridge::new (the wrapper used by production component hosts).
        let bridge = VmBridge::new(&minimal_widget()).expect("VmBridge::new");
        assert!(
            host::process_default_terminal_host_provider().is_some(),
            "VmBridge::new must install the terminal adapter"
        );
        assert!(
            bridge.vm().terminal_provider().is_some(),
            "the bridge VM snapshot must contain the adapter"
        );
        assert_eq!(
            pump_queued_input(bridge.vm(), "p808-bridge-new"),
            1,
            "the bridge VM consumes input"
        );

        // Explicit default wins over the compatibility install.
        host::clear_process_default_terminal_host_provider();
        let explicit: Arc<dyn TerminalHostProvider> = Arc::new(EngineService::without_dll());
        host::set_process_default_terminal_host_provider(explicit.clone());
        let bridge_explicit = VmBridge::new(&minimal_widget()).expect("VmBridge::new explicit");
        let installed = host::process_default_terminal_host_provider().expect("installed");
        assert!(
            Arc::ptr_eq(&installed, &explicit),
            "compat install must not override an explicit default"
        );
        assert!(bridge_explicit.vm().terminal_provider().is_some());

        // VmBridge::new_from_decls (the decl-based production constructor).
        host::clear_process_default_terminal_host_provider();
        let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test/ui/terminal_min/src/front/app.at");
        let src = std::fs::read_to_string(&at).expect("read minimal terminal app");
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::parser::Parser::from(src.as_str()).with_session(session);
        let ast = parser.parse().expect("parse minimal terminal app");
        let decl = ast
            .stmts
            .iter()
            .find_map(|stmt| match stmt {
                crate::ast::Stmt::WidgetDecl(decl) => Some(decl),
                _ => None,
            })
            .expect("widget decl");
        let bridge_decl = VmBridge::new_from_decls(decl, &[], Vec::new(), &HashMap::new(), false)
            .expect("VmBridge::new_from_decls");
        assert!(
            host::process_default_terminal_host_provider().is_some(),
            "VmBridge::new_from_decls must install the terminal adapter"
        );
        assert!(
            bridge_decl.vm().terminal_provider().is_some(),
            "the decl bridge VM snapshot must contain the adapter"
        );
        assert_eq!(
            pump_queued_input(bridge_decl.vm(), "p808-bridge-decl"),
            1,
            "the decl bridge VM consumes input"
        );

        terminal_dispose("p808-bridge-new");
        terminal_dispose("p808-bridge-decl");
        match saved {
            Some(provider) => host::set_process_default_terminal_host_provider(provider),
            None => host::clear_process_default_terminal_host_provider(),
        }
    }
}
// ── F808-R2-01: no provider callback/destructor under NI locks ───────────

mod plan808_reentrancy {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc, Weak};
    use std::time::Duration;

    use auto_terminal_core::{EngineService, TerminalHostProvider, TerminalRetiredCache};

    use crate::vm::native::NativeInterface;

    /// Provider whose lifecycle helpers (`owns_handle`, `live_handle_count`)
    /// and `Drop` reenter the interface. The reentry is disarmed per point so
    /// each test exercises exactly one callback path.
    struct Reentrant {
        ni: Weak<NativeInterface>,
        mode: &'static str,
        armed: AtomicBool,
        live_handle: Option<i64>,
        inner: EngineService,
    }

    impl Reentrant {
        fn new(ni: Weak<NativeInterface>, mode: &'static str, live_handle: Option<i64>) -> Self {
            Self {
                ni,
                mode,
                armed: AtomicBool::new(true),
                live_handle,
                inner: EngineService::without_dll(),
            }
        }

        fn callback(&self, point: &str) {
            if self.mode == point && self.armed.swap(false, Ordering::SeqCst) {
                if let Some(ni) = self.ni.upgrade() {
                    if point == "slot-drop" {
                        ni.terminal_provider();
                    } else {
                        ni.remember_terminal_owner(Arc::new(EngineService::without_dll()));
                    }
                }
            }
        }
    }

    impl Drop for Reentrant {
        fn drop(&mut self) {
            self.callback("drop");
            self.callback("slot-drop");
        }
    }

    impl TerminalHostProvider for Reentrant {
        fn spawn(&self, cols: i64, rows: i64) -> i32 {
            TerminalHostProvider::spawn(&self.inner, cols, rows)
        }
        fn spawn_ex(&self, program: &str, argv: &[String], cwd: &str, cols: i64, rows: i64) -> i32 {
            TerminalHostProvider::spawn_ex(&self.inner, program, argv, cwd, cols, rows)
        }
        fn write_line(&self, handle: i64, line: &str) {
            TerminalHostProvider::write_line(&self.inner, handle, line)
        }
        fn write_raw(&self, handle: i64, vt: &str) {
            TerminalHostProvider::write_raw(&self.inner, handle, vt)
        }
        fn rows(&self, handle: i64) -> Vec<String> {
            TerminalHostProvider::rows(&self.inner, handle)
        }
        fn rows_for(&self, handle: i64, key: &str) -> Vec<String> {
            TerminalHostProvider::rows_for(&self.inner, handle, key)
        }
        fn resize(&self, handle: i64, cols: i64, rows: i64) {
            TerminalHostProvider::resize(&self.inner, handle, cols, rows)
        }
        fn apply_resize(&self, handle: i64) -> i32 {
            TerminalHostProvider::apply_resize(&self.inner, handle)
        }
        fn apply_resize_for(&self, handle: i64, key: &str) -> i32 {
            TerminalHostProvider::apply_resize_for(&self.inner, handle, key)
        }
        fn pump_input(&self, handle: i64) -> i32 {
            TerminalHostProvider::pump_input(&self.inner, handle)
        }
        fn pump_for(&self, handle: i64, key: &str) -> i32 {
            TerminalHostProvider::pump_for(&self.inner, handle, key)
        }
        fn scroll_pending_for(&self, handle: i64, key: &str) -> i32 {
            TerminalHostProvider::scroll_pending_for(&self.inner, handle, key)
        }
        fn pend_resize_geom(&self, key: &str, geom: &str) -> i32 {
            TerminalHostProvider::pend_resize_geom(&self.inner, key, geom)
        }
        fn menu_take(&self) -> i32 {
            TerminalHostProvider::menu_take(&self.inner)
        }
        fn interrupt(&self, handle: i64) -> i32 {
            TerminalHostProvider::interrupt(&self.inner, handle)
        }
        fn is_exited(&self, handle: i64) -> bool {
            TerminalHostProvider::is_exited(&self.inner, handle)
        }
        fn free(&self, handle: i64) {
            TerminalHostProvider::free(&self.inner, handle)
        }
        fn cursor_row(&self, handle: i64) -> i32 {
            TerminalHostProvider::cursor_row(&self.inner, handle)
        }
        fn cursor_col(&self, handle: i64) -> i32 {
            TerminalHostProvider::cursor_col(&self.inner, handle)
        }
        fn viewport_cols(&self, handle: i64) -> i32 {
            TerminalHostProvider::viewport_cols(&self.inner, handle)
        }
        fn viewport_rows(&self, handle: i64) -> i32 {
            TerminalHostProvider::viewport_rows(&self.inner, handle)
        }
        fn backlog_pending_mb(&self, handle: i64) -> i32 {
            TerminalHostProvider::backlog_pending_mb(&self.inner, handle)
        }
        fn backlog_paused(&self, handle: i64) -> i32 {
            TerminalHostProvider::backlog_paused(&self.inner, handle)
        }
        fn backlog_take_alerts(&self, handle: i64) -> i32 {
            TerminalHostProvider::backlog_take_alerts(&self.inner, handle)
        }
        fn backlog_dropped(&self, handle: i64) -> i32 {
            TerminalHostProvider::backlog_dropped(&self.inner, handle)
        }
        fn history(&self, handle: i64) -> i32 {
            TerminalHostProvider::history(&self.inner, handle)
        }
        fn window_width(&self) -> i32 {
            TerminalHostProvider::window_width(&self.inner)
        }
        fn window_height(&self) -> i32 {
            TerminalHostProvider::window_height(&self.inner)
        }
        fn config_profiles(&self) -> Vec<String> {
            TerminalHostProvider::config_profiles(&self.inner)
        }
        fn config_default_profile(&self) -> String {
            TerminalHostProvider::config_default_profile(&self.inner)
        }
        fn config_profiles_full(&self) -> Vec<String> {
            TerminalHostProvider::config_profiles_full(&self.inner)
        }
        fn config_spawn_program(&self, profile: &str) -> String {
            TerminalHostProvider::config_spawn_program(&self.inner, profile)
        }
        fn config_spawn_argv(&self, profile: &str) -> Vec<String> {
            TerminalHostProvider::config_spawn_argv(&self.inner, profile)
        }
        fn config_spawn_cwd(&self, profile: &str) -> String {
            TerminalHostProvider::config_spawn_cwd(&self.inner, profile)
        }
        fn owns_handle(&self, handle: i64) -> bool {
            self.callback("owns");
            self.live_handle == Some(handle)
        }
        fn live_handle_count(&self) -> usize {
            self.callback("count");
            usize::from(self.live_handle.is_some())
        }
        fn cached_cursor_viewport(&self, handle: i64) -> Option<TerminalRetiredCache> {
            self.inner.cached_cursor_viewport(handle)
        }
    }

    fn run_with_timeout<F>(name: &str, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        let holder = std::thread::spawn(move || {
            f();
            let _ = tx.send(());
        });
        match rx.recv_timeout(Duration::from_secs(3)) {
            Ok(()) => holder.join().expect("reentrancy test thread panicked"),
            Err(_) => panic!("{name}: provider callback/drop reentered under an NI lock"),
        }
    }

    #[test]
    fn owner_query_callback_reenters_without_deadlock() {
        let ni = Arc::new(NativeInterface::new());
        let provider = Arc::new(Reentrant::new(Arc::downgrade(&ni), "owns", None));
        ni.set_terminal_provider(provider);
        run_with_timeout("owns", move || {
            assert!(ni.terminal_owner_for(1).is_none());
        });
    }

    #[test]
    fn prune_live_count_callback_reenters_without_deadlock() {
        let ni = Arc::new(NativeInterface::new());
        let provider = Arc::new(Reentrant::new(Arc::downgrade(&ni), "count", None));
        ni.set_terminal_provider(provider);
        run_with_timeout("count", move || ni.prune_terminal_owners());
    }

    #[test]
    fn prune_removed_provider_drop_reenters_without_deadlock() {
        let ni = Arc::new(NativeInterface::new());
        let provider = Arc::new(Reentrant::new(Arc::downgrade(&ni), "drop", None));
        ni.set_terminal_provider(provider);
        run_with_timeout("drop", move || ni.prune_terminal_owners());
    }

    #[test]
    fn slot_replacement_provider_drop_reenters_without_deadlock() {
        let ni = Arc::new(NativeInterface::new());
        let provider = Arc::new(Reentrant::new(Arc::downgrade(&ni), "slot-drop", None));
        ni.set_terminal_provider(provider);
        run_with_timeout("slot-drop", move || {
            ni.set_terminal_provider(Arc::new(EngineService::without_dll()));
        });
    }

    /// Positive controls: the same callbacks/destructor are harmless when no
    /// NI lock is held.
    #[test]
    fn direct_provider_controls_complete() {
        run_with_timeout("direct-owns", || {
            let provider = Reentrant::new(Weak::new(), "owns", None);
            assert!(!provider.owns_handle(1));
        });
        run_with_timeout("direct-count", || {
            let provider = Reentrant::new(Weak::new(), "count", None);
            assert_eq!(provider.live_handle_count(), 0);
        });
        run_with_timeout("direct-drop", || {
            let provider = Reentrant::new(Weak::new(), "drop", None);
            drop(provider);
        });
        run_with_timeout("direct-slot-drop", || {
            let provider = Reentrant::new(Weak::new(), "slot-drop", None);
            drop(provider);
        });
    }

    /// Concurrent set/query/prune must keep a live owner discoverable; the
    /// generation retry and merge-back are exercised here.
    #[test]
    fn concurrent_set_query_prune_keeps_live_owner() {
        let ni = Arc::new(NativeInterface::new());
        let live = Arc::new(Reentrant::new(Arc::downgrade(&ni), "none", Some(7)));
        ni.remember_terminal_owner(live.clone());

        let mut pruners = Vec::new();
        for _ in 0..2 {
            let ni = ni.clone();
            pruners.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    ni.prune_terminal_owners();
                    std::thread::yield_now();
                }
            }));
        }
        let ni_remember = ni.clone();
        let live_remember = live.clone();
        let remember = std::thread::spawn(move || {
            for _ in 0..100 {
                ni_remember.remember_terminal_owner(live_remember.clone());
                std::thread::yield_now();
            }
        });
        let ni_query = ni.clone();
        let query = std::thread::spawn(move || {
            for _ in 0..500 {
                // R3-F01: every query in the concurrent window must resolve
                // the live owner — not merely one out of many attempts.
                assert!(
                    ni_query.terminal_owner_for(7).is_some(),
                    "live owner lost during a concurrent prune window"
                );
                std::thread::yield_now();
            }
            true
        });

        for pruner in pruners {
            pruner.join().unwrap();
        }
        remember.join().unwrap();
        assert!(
            query.join().unwrap(),
            "a live owner must not be lost under concurrent set/query/prune"
        );
        assert!(ni.terminal_owner_for(7).is_some());
    }
}
