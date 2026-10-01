// PLAN-716 供料包 probes (plan710 shape: VM bridge calling the native +
// a2r generator-output form asserts + channel semantics at unit level).
//
// 组B (供② frame timestamps): 9918/9919 readback via VM .at call + a2r
// bare-name arm + gate semantics (AUTO_FRAME_BENCH) + two-state overhead
// measurement (T-09). Live-fire frame ordering belongs to the downstream
// bench tier (supply §2 验收形态原文: 上游探针到位+下游 bench 档建立).

#[cfg(test)]
mod plan716_supply {
    use crate::parser::Parser;

    /// Minimal VM bridge (plan703/plan710 build_probe_bridge 同款骨架).
    fn build_probe_bridge(src: &str) -> crate::ui::vm_bridge::VmBridge {
        let session = crate::session::CompilerSession::ui();
        let mut parser = Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|st| match st {
                crate::ast::Stmt::WidgetDecl(d) => Some(d.clone()),
                _ => None,
            })
            .expect("decl");
        let fns: Vec<crate::ast::Stmt> = ast
            .stmts
            .iter()
            .filter(|st| matches!(st, crate::ast::Stmt::Fn(_)))
            .cloned()
            .collect();
        let widget = crate::aura::extract_widget_from_decl(&decl).expect("extract");
        crate::ui::vm_bridge::VmBridge::new_with_imports(&widget, fns).expect("bridge")
    }

    fn probe_bridge_with(body: &str) -> crate::ui::vm_bridge::VmBridge {
        let src = format!(
            concat!(
                "widget T701Probe {{\n",
                "    view {{\n",
                "        col {{\n",
                "            text \"probe\"\n",
                "        }}\n",
                "    }}\n",
                "}}\n",
                "{}\n"
            ),
            body
        );
        build_probe_bridge(&src)
    }

    fn call_i64(bridge: &crate::ui::vm_bridge::VmBridge, name: &str) -> i64 {
        as_int_i64(bridge.call_vm_fn(name, &[]).expect("call ok"))
    }

    /// a2r 轨：内联 widget 源 → rust 发射产物（plan627/634/710 范式）。
    fn gen_rust(handler_body: &str) -> String {
        let src = format!(
            r#"
widget App {{
    msg {{ Init }}

    on {{
        {handler_body}
    }}

    view {{
        col {{
            text "term" {{}}
        }}
    }}
}}
"#,
        );
        let session = crate::session::CompilerSession::ui().with_backend("rust");
        let mut parser = crate::Parser::from(src.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let mut widget = crate::aura::extract::extract_widget_from_decl(decl).expect("extract");
        let mut gen = crate::ui_gen::rust::RustGenerator::new();
        gen.generate_rust(&widget).expect("generate rust")
    }

    /// Bridge-side Int|I64 acceptor（701 同款——i64 走 i64 lane，tag 定 variant）。
    fn as_int_i64(v: auto_val::Value) -> i64 {
        match v {
            auto_val::Value::Int(i) => i as i64,
            auto_val::Value::I64(i) => i,
            other => panic!("expected integer value, got {other:?}"),
        }
    }

    fn call_str(bridge: &crate::ui::vm_bridge::VmBridge, name: &str) -> String {
        match bridge.call_vm_fn(name, &[]) {
            Ok(auto_val::Value::Str(s)) => s.to_string(),
            other => panic!("{name} must return Str, got {other:?}"),
        }
    }

    /// plan703 同款临时文件对（T-14 裸名探针的 diff 输入）。
    struct TempFixture(std::path::PathBuf);
    impl TempFixture {
        fn new(tag: &str) -> Self {
            let p = std::env::temp_dir().join(format!("p716_probe_{tag}_{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&p);
            std::fs::create_dir_all(&p).unwrap();
            TempFixture(p)
        }
        fn write(&self, name: &str, content: &str) -> String {
            let path = self.0.join(name);
            std::fs::write(&path, content).unwrap();
            // Forward slashes — no .at escape hazards on Windows paths.
            path.to_string_lossy().replace('\\', "/")
        }
    }
    impl Drop for TempFixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    // ── AC-B2: .at 可达双轨（VM 9918/9919 + a2r 臂同源）──────────────────

    /// VM 轨：.at 直调 frame.begin_ms/present_ms（两段名→canonical 归一；
    /// 编译器签名面=stdlib/auto/frame.vm.at #[vm] 声明——id 经 NATIVE_ID_MAP
    /// 钉 9918/9919）→ i64 读回（未捕获态=0 形；门态数字由门开探针驱动）。
    #[test]
    fn frame_timestamps_vm_readback() {
        // concat 逐行形（701 PROBE_SRC 同构——规避行续接转义差异）。
        // frame 值=单调进程毫秒（测试进程内 <2^31，Int lane 无截断风险；
        // epoch 系全宽值才需字符串出口——701 注记口径）。
        let src = concat!(
            "widget T716Probe {\n",
            "    view {\n",
            "        col {\n",
            "            text \"probe\"\n",
            "        }\n",
            "    }\n",
            "}\n",
            "fn probe_frame_begin() i64 {\n",
            "    return frame.begin_ms()\n",
            "}\n",
            "fn probe_frame_present() i64 {\n",
            "    return frame.present_ms()\n",
            "}\n",
        );
        let bridge = build_probe_bridge(src);
        let b = call_i64(&bridge, "probe_frame_begin");
        let p = call_i64(&bridge, "probe_frame_present");
        assert!(b >= 0 && p >= 0, "时间戳读回非负形: begin={b} present={p}");
        // 未捕获态（门关进程）双零；门开态 present ≥ begin——两形皆合法。
        assert!(p == 0 || b == 0 || p >= b, "到达序: begin={b} present={p}");
    }

    /// a2r 轨：裸名 frame_begin_ms/frame_present_ms → a2r_std::frame::*
    /// 限定发射（grep 锚，703/710 形）。
    #[test]
    fn frame_timestamps_a2r_arm() {
        let rs = gen_rust(".Init -> { let _b = frame_begin_ms(); let _p = frame_present_ms() }");
        // handler 体直调纪律（710 code_editor_delta 同款——实现体单源
        // ui::frame_bench，不绕 VM shim）；模块 fn 体 a2r 臂=trans/rust.rs
        // 裸名映射（frame_timestamps_a2r_std_parity 锚实现体同源）。
        assert!(
            rs.contains("auto_lang::ui::frame_bench::frame_begin_ms()"),
            "handler 体 frame_begin_ms 直调臂缺失"
        );
        assert!(
            rs.contains("auto_lang::ui::frame_bench::frame_present_ms()"),
            "handler 体 frame_present_ms 直调臂缺失"
        );
    }

    /// a2r_std::frame 模块语义：读回 i64 非负（与 VM 轨同源同形）。
    #[test]
    fn frame_timestamps_a2r_std_parity() {
        let b = crate::a2r_std::frame::begin_ms();
        let p = crate::a2r_std::frame::present_ms();
        assert!(b >= 0 && p >= 0);
        assert_eq!(b, crate::ui::frame_bench::frame_begin_ms(), "双轨同源");
        assert_eq!(p, crate::ui::frame_bench::frame_present_ms(), "双轨同源");
    }

    // ── AC-B1: 门控语义 + 到达序（门开态）───────────────────────────────

    /// 门开探针（AUTO_FRAME_BENCH=1 显式驱动——nextest 每测进程隔离安全；
    /// 裸 cargo test 下其余门关测试自跳不冲突）：note 序写入+读回单调。
    #[test]
    fn frame_capture_gate_on_ordering() {
        if std::env::var("AUTO_FRAME_BENCH").ok().as_deref() != Some("1") {
            eprintln!("[P716] 门开探针需 AUTO_FRAME_BENCH=1（跳过——门关语义已由 gate_off 覆盖）");
            return;
        }
        // 模拟帧序：begin → present（真实序由桌面泵在实机产生）。
        crate::ui::frame_bench::note_frame_begin();
        std::thread::sleep(std::time::Duration::from_millis(2));
        crate::ui::frame_bench::note_frame_present();
        let b = crate::ui::frame_bench::frame_begin_ms();
        let p = crate::ui::frame_bench::frame_present_ms();
        eprintln!("[P716 门开] begin={b}ms present={p}ms");
        assert!(b > 0, "门开态 begin 必须被捕获");
        assert!(p >= b, "到达序: present {p} ≥ begin {b}");
    }

    // ── T-09: 零开销两态对照（观测面开销 ≤ 噪声带）──────────────────────

    /// 两态开销微基准：门关（note 的真实常驻成本=门检查+分支）vs 门开
    /// （OnceLock 读+elapsed+双 store）。数字 --nocapture 在档；断言 CI
    /// 安全上界。帧泵消息频率量级 ≤1000/s——每条消息 <100ns 即 ≤0.01%
    /// 帧预算（16.7ms@60fps），噪声带内。
    #[test]
    fn frame_capture_overhead_two_state() {
        const ITERS: u32 = 1_000_000;
        // 门关路径（本进程门态如实测量——不预设）。
        let t0 = std::time::Instant::now();
        for _ in 0..ITERS {
            crate::ui::frame_bench::note_frame_begin();
        }
        let per_call = t0.elapsed().as_nanos() as f64 / ITERS as f64;
        eprintln!("[P716 开销] note_frame_begin per-call={per_call:.1}ns（门态如实）");
        assert!(per_call < 200.0, "门检查路径超上界 {per_call:.1}ns/op");

        // elapsed+store 开销上界（门开路径的成本构成实测）。
        let t1 = std::time::Instant::now();
        let mut acc: i64 = 0;
        for _ in 0..ITERS {
            acc += std::time::Instant::now().elapsed().as_millis() as i64;
        }
        let per_timed = t1.elapsed().as_nanos() as f64 / ITERS as f64;
        eprintln!("[P716 开销] elapsed+add per-call={per_timed:.1}ns（门开成本构成）acc={acc}");
        assert!(per_timed < 500.0, "门开路径构成超上界 {per_timed:.1}ns/op");

        // 帧预算占比（60fps 参照）：万条消息/帧量级仍 <1%。
        let budget_ns = 16_666_666.0;
        eprintln!(
            "[P716 开销] 占 60fps 帧预算：门关 {:.4}%/帧@1000msg",
            per_call * 1000.0 / budget_ns * 100.0
        );
    }

    // ── T-11 (组C): 100MB 散点对 census——窗口 vs 全量（021 FAIL 清偿弹药）──

    /// census 探针（710 形）：散点对全量 envelope vs 窗口投影的体量/墙钟
    /// 对照。默认档=~2MB 散点对（CI 安全，语义+量级对照）；100MB 全尺寸
    /// 由 AUTO_P716_CENSUS=1 显式驱动（每进程一次性成本，数字入 SD-C/
    /// 计划簿记——下游切换件的依据数字）。
    #[test]
    #[cfg(feature = "code-editor")]
    fn diff_window_census() {
        let full_scale = std::env::var("AUTO_P716_CENSUS").ok().as_deref() == Some("1");
        // 散点对生成：每 40 行一处单行改（021 形态微缩/全尺寸按行数放大）。
        let hunks = if full_scale { 400_000 } else { 12_000 };
        let lines = hunks * 40; // ~19B/行 → 全尺寸≈300MB? 控制在 ~100MB：~5.2M 行
        let lines = if full_scale { 5_200_000 } else { 480_000 };
        let mut a = String::with_capacity(lines * 20);
        let mut b = String::with_capacity(lines * 20);
        for i in 0..lines {
            a.push_str("L");
            a.push_str(&i.to_string());
            a.push('\n');
            if i % 40 == 20 {
                b.push_str("L");
                b.push_str(&i.to_string());
                b.push_str("-changed\n");
            } else {
                b.push_str("L");
                b.push_str(&i.to_string());
                b.push('\n');
            }
        }
        let mb = a.len() as f64 / (1024.0 * 1024.0);
        let _ = lines;

        // 全量 envelope（021 形——rows 全投影+双层 JSON）。
        let t0 = std::time::Instant::now();
        let full = crate::ui::code_editor::diff::envelope::diff_files_envelope(&a, &b, 3);
        let full_ms = t0.elapsed().as_millis();

        // 窗口投影（下游 600 行窗——016 渲染窗形态）。
        let t1 = std::time::Instant::now();
        let win = crate::ui::code_editor::diff::envelope::diff_files_envelope_window(&a, &b, 3, 0, 600);
        let win_ms = t1.elapsed().as_millis();

        // 越头窗（分页形）对照。
        let t2 = std::time::Instant::now();
        let win2 = crate::ui::code_editor::diff::envelope::diff_files_envelope_window(&a, &b, 3, 600, 600);
        let win2_ms = t2.elapsed().as_millis();

        eprintln!(
            "[P716 census] {mb:.1}MB 散点对：full={}B/{full_ms}ms window(0,600)={}B/{win_ms}ms window(600,600)={}B/{win2_ms}ms",
            full.len(),
            win.len(),
            win2.len()
        );
        // 语义断言：窗口体量数万倍级收缩（600 行 vs 全量）；首窗含 rows_total。
        assert!(win.len() < full.len() / 10, "窗口体量应显著小于全量");
        assert!(win.contains("rows_total"));
        assert!(win.contains("\"truncated\":true"));
        // 页窗时间量级：不劣于全量（走线相同+行物化大幅减少）。
        assert!(
            win_ms <= full_ms.max(1),
            "窗口墙钟不应劣于全量: {win_ms}ms vs {full_ms}ms"
        );
    }

    // ── T-14 (Phase 2 供⑧): 9921 VM 轨裸名臂 ────────────────────────────

    /// VM 轨 .at 裸名直调 diff_files_window——Phase 1 注册面有一处双缺：
    /// ① codegen intrinsics 裸名表漏登记（计划勘定面）；② 9920 号位与
    /// PLAN-095 ui.focus 撞号（T-00 槽位复核漏扫 :419 段——shim 表按清单
    /// 序后注册者覆盖，VM 轨窗口调用恒派发 ui_focus shim 静默 no-op，
    /// 下游 plan-022 挂死实测根因）。修复=补臂+改签 9921；本探针走完整
    /// 调用链（parse→codegen→VM→shim→envelope）锁真值形。
    #[test]
    #[cfg(feature = "code-editor")]
    fn diff_files_window_vm_bare_name() {
        let fx = TempFixture::new("win");
        let pa = fx.write("a.txt", "alpha\nbeta\ngamma\n");
        let pb = fx.write("b.txt", "alpha\nbeta2\ngamma\n");
        // limit=2 < rows_total=3 → truncated 置位形（截断语义随链路验证）。
        let bridge = probe_bridge_with(&format!(
            "fn probe_diffwin() str {{\n    return diff_files_window(\"{pa}\", \"{pb}\", 3, 0, 2)\n}}\n"
        ));
        let json = call_str(&bridge, "probe_diffwin");
        assert!(json.contains("rows_total"), "窗口 envelope rows_total 在场: {json}");
        assert!(json.contains("\"rows_total\":3"), "rows_total 真值: {json}");
        assert!(json.contains("\"truncated\":true"), "窗口截断置位: {json}");
        assert!(json.contains("\"adds\":1"), "行内容真值（非 err 净形）: {json}");
        assert!(json.contains("\"hunks\":[{\"a1\":0,\"a2\":3,\"b1\":0,\"b2\":3}]"), "hunks 全量保持: {json}");
        assert!(!json.contains("gamma"), "窗口外行不物化: {json}");
    }

    /// 四面在册零扰动锚（供⑧ 修复不回改 Phase 1 注册面）：catalog 条目、
    /// trans 裸名臂、ui_gen handler 直调臂的 grep 锚（plan703/710 形）。
    /// 9921=供⑧ 改签位；ui.focus 独占 9920 断言防回撞。
    #[test]
    fn diff_files_window_faces_registered() {
        // catalog: 9921 canonical 归一（native_catalog.rs for_each_native 表）。
        let src = include_str!("../vm/native_catalog.rs");
        assert!(
            src.contains("(9921, NATIVE_DIFF_FILES_WINDOW, shim_diff_files_window, \"auto.diff_files_window\")"),
            "catalog 9921 条目在册（供⑧ 改签位）"
        );
        assert!(
            !src.contains("(9920, NATIVE_DIFF_FILES_WINDOW"),
            "9920 撞号残留（ui.focus 独占位）"
        );
        // trans 裸名臂（a2r 轨）。
        let tr = include_str!("../trans/rust.rs");
        assert!(tr.contains("diff_files_window"), "trans 裸名臂在册");
        // ui_gen handler 直调臂。
        let ug = include_str!("../ui_gen/rust.rs");
        assert!(ug.contains("diff_files_window"), "ui_gen 直调臂在册");
    }
}
