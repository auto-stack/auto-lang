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

    use auto_terminal_core::{EngineService, TerminalHostProvider};

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
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use auto_terminal_core::EngineService;

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

    /// V-04: provider replacement routing — a handle belongs to the service
    /// that created it; another service returns absent values and its free
    /// must not release the original session.
    #[test]
    fn provider_replacement_does_not_reroute_handles() {
        let Some(path) = dll_path() else { return };
        let vm = AutoVM::new(VirtualFlash::new(0), 1024);
        let svc_a = service_with_dll(&path);
        let svc_b = Arc::new(EngineService::without_dll());
        let handle = svc_a.spawn_ex(
            "cmd.exe",
            &["/C".into(), "echo PLAN808_ROUTE".into()],
            "D:/autostack/auto-lang",
            80,
            24,
        );
        assert_ne!(handle, 0);
        let reaper = Reaper::new(&svc_a, vec![handle]);
        let mut task = AutoTask::new(0, 1024, 0);

        vm.set_terminal_provider(svc_a.clone());
        task.ram.push_nv(auto_val::encode_i32(handle as i32));
        crate::vm::ffi::term_engine::shim_term_rows(&mut task, &vm).unwrap();
        let _ = task.ram.pop_nv();
        assert_eq!(svc_a.live_handle_count(), 1);

        // Route the same handle through B: absent values, no free of A.
        vm.set_terminal_provider(svc_b.clone());
        let mut task = AutoTask::new(0, 1024, 0);
        task.ram.push_nv(auto_val::encode_i32(handle as i32));
        crate::vm::ffi::term_engine::shim_term_rows(&mut task, &vm).unwrap();
        let _list_id = task.ram.pop_i32();
        let mut task = AutoTask::new(0, 1024, 0);
        task.ram.push_nv(auto_val::encode_i32(handle as i32));
        crate::vm::ffi::term_engine::shim_term_free(&mut task, &vm).unwrap();
        assert_eq!(
            svc_a.live_handle_count(),
            1,
            "free through another service must not release A's session"
        );

        // Back to A: the session is still alive and readable.
        vm.set_terminal_provider(svc_a.clone());
        let rows = wait_marker(&svc_a, handle, "PLAN808_ROUTE");
        assert!(rows.iter().any(|l| l.contains("PLAN808_ROUTE")));
        assert!(wait_exited(&svc_a, handle));
        svc_a.free(handle);
        assert_eq!(svc_a.live_handle_count(), 0);
        reaper.disarm();
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
        let mut reaper = Reaper::new(&service, vec![ha, hb]);

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
}
