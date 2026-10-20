//! PLAN-802: code editor 22 native host provider——结构性剪断/装配身份/
//! 真实 adapter/滚动投影/OS 剪贴板回归锚（in-crate 面）。
//!
//! 外部 no-ui consumer oracle 见
//! `test/fixtures/modularity/code-editor-provider-consumer`（V-01..V-03：
//! fake 注入真 VM 全管线、缺席矩阵逐字面值、delta/ownership）。本模块：
//! - core 剪断（永久回归锚）：vm/native.rs 不再含 code_editor 注册表
//!   直引形态（`code_editor::code_editor_*` / `editor_scroll_handle` /
//!   `ClipboardOp` / `ui::scroll` intent 直入——diff 族 9915..9917 为
//!   非目标保留臂，不匹配这些精确形态）。
//! - 装配身份：构造期快照/实例覆盖/NI Clone/VM merge/注入不动
//!   binding_revision（785/794 同款 fake 面）。
//! - 真实 adapter（V05/V04 寿命）：真 CodeEditorCore 注册表经
//!   CodeEditorRegistryProvider 往返——多字节 edit/光标单位/undo/fold/
//!   delta 水位/临时文件 load-save 字节保真/失败值形；provider 替换与
//!   VM drop 不清空 UI 文档。
//! - 滚动投影链（V06）：editor 寄宿 scroller 经 handle 约定绑定
//!   controller 注册表——offset 读出/双轴 intent 入队/drain 折叠/clamp。
//! - OS 剪贴板（V06）：cut/copy/paste 经 arboard 桥真往返（保存恢复
//!   剪贴板；headless guard 静默跳过）；readonly 编辑器组合臂。

// ── core 剪断锚（feature 中性） ──────────────────────────────────────────

mod plan802_core_cut {
    /// core 剪断（永久回归锚）：22 shim 的直引形态全部消失。按精确调用
    /// 形态扫描（非节区段——code_editor shim 与非目标 diff 族交错）；
    /// `ce::` 别名仅存在于测试模块（不匹配）；`code_editor::diff::` 为
    /// 非目标保留臂（不匹配）。
    #[test]
    fn code_editor_shims_have_no_ui_direct_reference() {
        let native_rs = include_str!("../../../auto-lang-core/src/vm/native.rs");
        for pattern in [
            "ui::code_editor::code_editor_",
            "ui::code_editor::editor_scroll_handle",
            "ui::code_editor::ClipboardOp",
            "ui::scroll::enqueue_intent",
            "ui::scroll::controller::controller_snapshot",
        ] {
            assert!(
                !native_rs.contains(pattern),
                "vm/native.rs 不得再含直引形态 {pattern:?}（PLAN-802 剪断；供值经 \
                 host_capabilities::CodeEditorHostProvider 装配）"
            );
        }
    }
}

// ── 装配身份（feature 中性 fake 面） ─────────────────────────────────────

mod plan802_assembly {
    use std::sync::Arc;

    use auto_lang_core::vm::engine::AutoVM;
    use auto_lang_core::vm::host_capabilities::{
        CodeEditorCursorDto, CodeEditorHostProvider, CodeEditorScrollOffsetDto,
    };
    use auto_lang_core::vm::native::NativeInterface;
    use auto_lang_core::vm::task::AutoTask;
    use auto_lang_core::vm::virt_memory::VirtualFlash;

    /// 无状态哨兵 fake（text 哨兵可区分实例）。
    struct Fake(&'static str);
    impl CodeEditorHostProvider for Fake {
        fn text(&self, _k: &str) -> Option<String> {
            Some(self.0.to_string())
        }
        fn delta(&self, _k: &str) -> Option<String> {
            None
        }
        fn edit(&self, _k: &str, _s: usize, _e: usize, _r: &str) -> bool {
            false
        }
        fn cursor(&self, _k: &str) -> Option<CodeEditorCursorDto> {
            None
        }
        fn find(&self, _k: &str) -> bool {
            false
        }
        fn set_text(&self, _k: &str, _t: &str) -> bool {
            false
        }
        fn load_file(&self, _k: &str, _p: &str) -> Option<i64> {
            None
        }
        fn save(&self, _k: &str, _p: &str) -> bool {
            false
        }
        fn set_cursor(&self, _k: &str, _l: usize, _c: usize) -> bool {
            false
        }
        fn scroll_offset(&self, _k: &str) -> CodeEditorScrollOffsetDto {
            CodeEditorScrollOffsetDto::default()
        }
        fn scroll_to(&self, _k: &str, _x: f64, _y: f64) -> bool {
            false
        }
        fn fold_toggle(&self, _k: &str, _l: usize) -> bool {
            false
        }
        fn fold_hidden_count(&self, _k: &str) -> usize {
            0
        }
        fn undo(&self, _k: &str) -> bool {
            false
        }
        fn redo(&self, _k: &str) -> bool {
            false
        }
        fn select_all(&self, _k: &str) -> bool {
            false
        }
        fn cut(&self, _k: &str) -> Option<bool> {
            None
        }
        fn copy(&self, _k: &str) -> Option<bool> {
            None
        }
        fn paste(&self, _k: &str) -> Option<bool> {
            None
        }
    }

    fn read_text_lane(vm: &AutoVM, task: &mut AutoTask) -> String {
        use auto_lang_core::vm::native::shim_code_editor_text;
        let idx = vm.add_string("k".as_bytes().to_vec());
        vm.rc_push_str_idx(task, idx);
        shim_code_editor_text(task, vm).unwrap();
        let nv = task.ram.pop_nv();
        assert!(auto_val::is_string(nv), "text 走 String lane");
        let i = auto_val::decode_string(nv);
        vm.get_string(i)
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default()
    }

    /// 构造期快照 + 实例覆盖 + Clone/merge + 注入不动 binding_revision
    ///（785/794 同款面；构造式缺席用 `AutoVM::new_without_host_snapshot`）。
    #[test]
    fn snapshot_instance_override_clone_merge_identity() {
        let _slot = auto_lang_core::vm::host_capabilities::test_default_slot_lock();
        let saved =
            auto_lang_core::vm::host_capabilities::process_default_code_editor_host_provider();

        // 构造式缺席：跳过快照=无 provider（不依赖全局槽时序）。
        let bare = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 64);
        assert!(bare.code_editor_provider().is_none(), "构造式缺席");

        // 快照隔离：vm1 快照 A；换默认 B 后 vm1 仍 A，vm2 快照 B。
        auto_lang_core::vm::host_capabilities::set_process_default_code_editor_host_provider(
            Arc::new(Fake("@A")),
        );
        let vm1 = AutoVM::new(VirtualFlash::new(0), 256);
        auto_lang_core::vm::host_capabilities::set_process_default_code_editor_host_provider(
            Arc::new(Fake("@B")),
        );
        let vm2 = AutoVM::new(VirtualFlash::new(0), 256);
        let mut t1 = AutoTask::new(0, 256, 0);
        assert_eq!(read_text_lane(&vm1, &mut t1), "@A", "vm1 保持构造期快照 A");
        let mut t2 = AutoTask::new(0, 256, 0);
        assert_eq!(read_text_lane(&vm2, &mut t2), "@B", "vm2 快照替换后默认 B");

        // 实例覆盖只影响本 VM。
        vm1.set_code_editor_provider(Arc::new(Fake("@C")));
        let mut t1 = AutoTask::new(0, 256, 0);
        assert_eq!(read_text_lane(&vm1, &mut t1), "@C", "实例覆盖生效");
        let mut t2 = AutoTask::new(0, 256, 0);
        assert_eq!(read_text_lane(&vm2, &mut t2), "@B", "兄弟 VM 不受覆盖影响");
        // 后改进程默认不影响已建实例（vm2 换默认 D 后仍 B）。
        auto_lang_core::vm::host_capabilities::set_process_default_code_editor_host_provider(
            Arc::new(Fake("@D")),
        );
        let mut t2 = AutoTask::new(0, 256, 0);
        assert_eq!(read_text_lane(&vm2, &mut t2), "@B", "后改默认不影响旧 VM");

        // 注入不 bump binding_revision；Clone/merge 保 provider。
        let ni = NativeInterface::production();
        let rev0 = ni.binding_revision();
        ni.set_code_editor_provider(Arc::new(Fake("@NI")));
        assert_eq!(ni.binding_revision(), rev0, "注入不 bump revision");
        assert!(
            ni.clone().code_editor_provider().is_some(),
            "NI Clone 保 provider"
        );
        let mut vm = AutoVM::new(VirtualFlash::new(0), 64);
        vm.set_code_editor_provider(Arc::new(Fake("@NI")));
        let mut carrier = NativeInterface::new();
        carrier.register_dynamic(|_t: &mut AutoTask, _v: &AutoVM| Ok(()));
        vm.merge_native_interface(&carrier);
        assert!(vm.code_editor_provider().is_some(), "VM merge 保 provider");

        // 还原全局槽。
        match saved {
            Some(p) => {
                auto_lang_core::vm::host_capabilities::set_process_default_code_editor_host_provider(p)
            }
            None => auto_lang_core::vm::host_capabilities::clear_process_default_code_editor_host_provider(),
        }
    }
}

// ── 真实 adapter（V05 真注册表往返 + V04 寿命） ──────────────────────────

#[cfg(feature = "code-editor")]
mod plan802_real_adapter {
    use std::sync::Arc;

    use crate::ui::code_editor_provider::CodeEditorRegistryProvider;
    use auto_lang_core::vm::engine::AutoVM;
    use auto_lang_core::vm::host_capabilities::CodeEditorHostProvider;
    use auto_lang_core::vm::task::AutoTask;
    use auto_lang_core::vm::virt_memory::VirtualFlash;

    /// 与既有 code_editor_natives 测试同款 font system 装配（headless）。
    fn install_fs(with: &mut dyn FnMut(&mut cosmic_text::FontSystem)) {
        use std::sync::{OnceLock, RwLock};
        static FS: OnceLock<RwLock<cosmic_text::FontSystem>> = OnceLock::new();
        let fs = FS.get_or_init(|| RwLock::new(cosmic_text::FontSystem::new()));
        let mut guard = fs.write().unwrap();
        with(&mut guard);
    }

    /// 注册表互斥 + font system 装配 + 键编辑器建立（返回的锁守卫持至
    /// 测试结束——REGISTRY_TEST_LOCK 串行）。
    fn setup(key: &str) -> std::sync::MutexGuard<'static, ()> {
        let guard = crate::ui::code_editor::core::REGISTRY_TEST_LOCK
            .lock()
            .unwrap();
        crate::ui::code_editor::set_font_system_call(install_fs);
        crate::ui::code_editor::code_editor_dispose(key);
        let config = crate::ui::code_editor::CodeEditorConfig::default();
        crate::ui::code_editor::code_editor(key, &config);
        guard
    }

    /// 实例级装配的 VM（不动全局默认槽——790 同款纪律）。
    fn vm_with_registry() -> AutoVM {
        let vm = AutoVM::new(VirtualFlash::new(0), 1024);
        vm.set_code_editor_provider(Arc::new(CodeEditorRegistryProvider));
        vm
    }

    /// V05 核心：真 CodeEditorCore 经 adapter/VM shim 往返——多字节 edit、
    /// 光标 0基行/char 列、selection bytes、undo/redo、fold、delta 水位。
    #[test]
    fn real_adapter_registry_roundtrip() {
        let key = crate::ui::code_editor::storage_key("p802-roundtrip");
        let _g = setup(&key);
        let vm = vm_with_registry();
        let p: Arc<dyn CodeEditorHostProvider> = vm.code_editor_provider().unwrap();

        // 全文写 + 读（多字节）。
        assert!(p.set_text(&key, "héllo中文xyz"));
        assert_eq!(p.text(&key).as_deref(), Some("héllo中文xyz"));
        // adapter 同源：与直调注册表逐字一致（V05"与原 UI 函数同源"锚）。
        assert_eq!(
            crate::ui::code_editor::code_editor_text(&key).as_deref(),
            Some("héllo中文xyz")
        );

        // edit：合法 char 边界替换（"hé"→"HÉ"；é 为 2 字节，边界=3）。
        assert!(p.edit(&key, 0, 3, "HÉ"));
        assert_eq!(p.text(&key).as_deref(), Some("HÉllo中文xyz"));
        // "HÉllo中文xyz"：H(1)+É(2)+llo(3)=6 字节；中=6..9、文=9..12。
        // 取中的内部偏移 7（非 char 边界）。
        assert!(!p.edit(&key, 7, 9, "！"), "多字节中点非边界 edit=false");
        // 合法：整字替换 中→华（bytes 6..9）。
        assert!(p.edit(&key, 6, 9, "华"));
        assert_eq!(p.text(&key).as_deref(), Some("HÉllo华文xyz"));

        // 反区间/越界 → false。
        assert!(!p.edit(&key, 5, 2, "x"), "反区间 false");
        assert!(!p.edit(&key, 0, 9999, "x"), "越界 false");

        // 光标：0 基行 / char 列（多字节行内区分字节/char 单位）。
        assert!(p.set_text(&key, "l1\nl2中文\nl3"));
        assert!(p.set_cursor(&key, 1, 2)); // 行"l2中文"第 2 char 后（l,2 之后）
        let c = p.cursor(&key).expect("cursor");
        assert_eq!((c.line, c.char_col), (1, 2), "0基行/char 列");
        // 越界钳位：行→末行、列→行尾 char 数。
        assert!(p.set_cursor(&key, 99, 99));
        let c = p.cursor(&key).expect("cursor");
        assert_eq!((c.line, c.char_col), (2, 2), "钳位入文档（l3=2 chars）");

        // selection_len = 字节单位（cursor_info 实现形：选中行内容字节和，
        // 不含行分隔符）："l1"2+"l2中文"8+末行差 2 = 12；char 计数=8——
        // 12≠8 钉字节锚（docstring"bytes"的实现口径，迁移保形不改）。
        assert!(p.select_all(&key));
        let c = p.cursor(&key).expect("cursor");
        assert_eq!(c.selection_bytes, 12, "selection_len 按字节（12≠8 char）");

        // undo/redo（真引擎 undo 栈——programmatic set_text/edit 走 rewrite
        // 不入引擎历史[2775 注释]，种一个引擎侧键入条目后经 provider 撤销/
        // 重做——三来源同流既有 core 测试同款形态）。
        assert!(p.set_text(&key, "alpha"));
        let _ = p.delta(&key); // 清 set_text 入队
        assert!(p.set_cursor(&key, 0, 5)); // 光标至行尾
        crate::ui::code_editor::code_editor_with(&key, |core| {
            core.set_focused(true);
            crate::ui::code_editor::with_font_system(|fs| {
                core.handle_input(
                    fs,
                    crate::ui::code_editor::EditorInput::KeyPressed {
                        key: crate::ui::code_editor::EditorKey::Char('!'),
                        text: Some("!".to_owned()),
                        modifiers: crate::ui::code_editor::EditorModifiers::none(),
                    },
                    &mut crate::ui::code_editor::NullClipboard,
                );
            });
        })
        .expect("editor in registry");
        assert_eq!(p.text(&key).as_deref(), Some("alpha!"), "引擎键入 '!' 生效");
        assert!(p.undo(&key));
        assert_eq!(p.text(&key).as_deref(), Some("alpha"), "undo 撤销键入");
        assert!(p.redo(&key));
        assert_eq!(p.text(&key).as_deref(), Some("alpha!"), "redo 复应用");

        // delta：undo/redo 与键入同流入队——先排空（三源同流锚），再断言
        // 空水位形态 + 连续空读同形（真编辑器形；fixture V03 为 fake 形）。
        let d_drain = p.delta(&key).expect("delta");
        assert!(
            d_drain.contains("\"revision\":"),
            "delta JSON 形（got {d_drain}）"
        );
        assert!(
            d_drain.contains("\"replacement\":\"!\""),
            "键入/undo/redo 三源同流在队（got {d_drain}）"
        );
        let d1 = p.delta(&key).expect("delta");
        assert!(
            d1.contains("\"deltas\":[]"),
            "排空后空队列保水位（got {d1}）"
        );
        let d2 = p.delta(&key).expect("delta");
        assert_eq!(d1, d2, "连续空读同水位同形（destructive read 不改水位）");

        // fold：1基→0基在 shim——直调 provider 收 0 基（e2e 程序轨由
        // code_editor_natives 既有测试覆盖）。
        assert!(p.set_text(&key, "fn a() {\n    body()\n}\n"));
        assert!(p.fold_toggle(&key, 0), "折叠第 1 行（0 基 0）");
        assert_eq!(p.fold_hidden_count(&key), 2, "隐藏 2 行");
        assert!(!p.fold_toggle(&key, 0), "再切换=展开");

        crate::ui::code_editor::code_editor_dispose(&key);
    }

    /// V05 文件 IO：临时文件 load/save 字节保真（CRLF 保持、无 BOM 添加、
    /// rope 直写）；失败=值形（load None→shim -1 / save false）。
    #[test]
    fn load_save_file_io_byte_fidelity() {
        use auto_lang_core::vm::native::{shim_code_editor_load_file, shim_code_editor_save};

        let key = crate::ui::code_editor::storage_key("p802-fileio");
        let _g = setup(&key);
        let vm = vm_with_registry();

        let dir = std::env::temp_dir();
        let src = dir.join("plan802-src-🎉.txt");
        let dst = dir.join("plan802-dst-🎉.txt");
        let body = "行一\r\nline two\r\nmixed \u{feff}no-bom-keep\r\n";
        std::fs::write(&src, body.as_bytes()).unwrap();

        // load：shim 级（返回 bytes 值；CRLF/中文名路径保真）。
        let mut task = AutoTask::new(0, 512, 0);
        let k = vm.add_string(key.as_bytes().to_vec());
        let path = vm.add_string(src.to_string_lossy().as_bytes().to_vec());
        // 弹序：path 先、key 后（codegen 左→右压）。
        vm.rc_push_str_idx(&mut task, k);
        vm.rc_push_str_idx(&mut task, path);
        shim_code_editor_load_file(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), body.len() as i32, "load 返回总 bytes");
        // rope 保真：读出与磁盘字节一致（CRLF 不改写）。
        assert_eq!(p_text(&vm, &key), body);

        // save：rope→文件直写；dst 字节与 src 逐字节一致（无 BOM/EOL 改写）。
        let _ = std::fs::remove_file(&dst);
        let mut task = AutoTask::new(0, 512, 0);
        let k = vm.add_string(key.as_bytes().to_vec());
        let path = vm.add_string(dst.to_string_lossy().as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, k);
        vm.rc_push_str_idx(&mut task, path);
        shim_code_editor_save(&mut task, &vm).unwrap();
        assert!(auto_val::decode_bool(task.ram.pop_nv()), "save 成功");
        assert_eq!(
            std::fs::read(&dst).unwrap(),
            body.as_bytes(),
            "落盘字节保真（CRLF/无 BOM 添加）"
        );

        // 失败=值形：不存在路径 load → -1（非 raise）；坏目录 save → false。
        let mut task = AutoTask::new(0, 512, 0);
        let k = vm.add_string(key.as_bytes().to_vec());
        let bad = vm.add_string(
            dir.join("plan802-不存在的目录-🎉")
                .join("x.txt")
                .to_string_lossy()
                .as_bytes()
                .to_vec(),
        );
        vm.rc_push_str_idx(&mut task, k);
        vm.rc_push_str_idx(&mut task, bad.clone());
        shim_code_editor_load_file(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i32(), -1, "load 失败=-1 值（非 raise）");
        vm.rc_push_str_idx(&mut task, k);
        vm.rc_push_str_idx(&mut task, bad);
        shim_code_editor_save(&mut task, &vm).unwrap();
        assert!(
            !auto_val::decode_bool(task.ram.pop_nv()),
            "save 失败=false 值"
        );

        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&dst);
        crate::ui::code_editor::code_editor_dispose(&key);
    }

    /// V04 寿命：provider 替换/VM drop 不清空 UI 文档（注册表=单一真源，
    /// provider 是无状态透传）；无此键编辑器 → text None（shim raise 形）。
    #[test]
    fn provider_swap_and_vm_drop_keep_ui_documents() {
        let key = crate::ui::code_editor::storage_key("p802-lifetime");
        let _g = setup(&key);
        {
            let vm1 = vm_with_registry();
            vm1.code_editor_provider()
                .unwrap()
                .set_text(&key, "长寿文档-🎯");
        } // vm1 drop
        assert_eq!(
            crate::ui::code_editor::code_editor_text(&key).as_deref(),
            Some("长寿文档-🎯"),
            "VM drop 不清空 UI 文档"
        );
        // 新 VM + 新 provider 实例读同一文档（注册表单一真源）。
        let vm2 = vm_with_registry();
        assert_eq!(
            vm2.code_editor_provider().unwrap().text(&key).as_deref(),
            Some("长寿文档-🎯")
        );
        // 无此键 → None（shim 侧 raise no-editor 形）。
        assert!(vm2
            .code_editor_provider()
            .unwrap()
            .text("__p802-无此键")
            .is_none());
        crate::ui::code_editor::code_editor_dispose(&key);
    }

    fn p_text(vm: &AutoVM, key: &str) -> String {
        vm.code_editor_provider().unwrap().text(key).unwrap()
    }
}

// ── 滚动投影链（V06） ────────────────────────────────────────────────────

#[cfg(feature = "code-editor")]
mod plan802_scroll {
    use crate::ui::code_editor_provider::CodeEditorRegistryProvider;
    use crate::ui::scroll::controller;
    use auto_lang_core::vm::host_capabilities::CodeEditorHostProvider;

    /// editor 寄宿 scroller 三项：handle 约定绑定 → offset 投影读出；
    /// scroll_to 双轴 intent 入队（X 先 Y 后）→ drain 折叠 → clamp；未绑定
    /// handle 静默丢弃（恒 true=已入队）。
    #[test]
    fn editor_scroll_projection_chain() {
        controller::reset_for_test();
        let p = CodeEditorRegistryProvider;
        let key = "p802-scroll";

        // 未绑定 → 全零（701 会话恢复中性值）。
        assert_eq!(
            p.scroll_offset(&format!("__{key}-未绑定")),
            Default::default(),
            "未绑定 handle 投影=全零"
        );

        // 绑定 + 测量（renderer 构建期形态模拟：bind → prime → measure）。
        let handle = crate::ui::code_editor::editor_scroll_handle(key);
        controller::bind_controller(&handle, "w_p802");
        let _ = controller::take_prime_requests();
        controller::note_controller_state(&handle, (3.0, 4.0), (300.0, 200.0), (500.0, 1000.0));
        let off = p.scroll_offset(key);
        assert_eq!((off.x, off.y), (3.0, 4.0), "offset=注册表投影单源");

        // scroll_to：双轴入队（X 先 Y 后）、恒 true。
        assert!(p.scroll_to(key, 40.5, 250.0));
        assert_eq!(controller::pending_intent_count(), 2, "双轴两条 intent");
        let (finals, prime) = controller::drain_resolved_intents();
        assert!(prime.is_empty(), "已预热不回队");
        assert_eq!(
            finals,
            vec![(handle.clone(), "w_p802".to_string(), 40.5, 250.0)],
            "drain 折叠终态=命令值（X/Y 各一条折叠）"
        );
        // renderer 消费侧读回（模拟 note-back）→ 投影更新。
        controller::note_controller_state(&handle, (40.5, 250.0), (300.0, 200.0), (500.0, 1000.0));
        let off = p.scroll_offset(key);
        assert_eq!((off.x, off.y), (40.5, 250.0), "读回后投影=新偏移");

        // clamp：越界值钳入 content-viewport 范围（X≤200 / Y≥0）。
        assert!(p.scroll_to(key, 999.0, -5.0));
        let (finals, _) = controller::drain_resolved_intents();
        assert_eq!(
            finals,
            vec![(handle.clone(), "w_p802".to_string(), 200.0, 0.0)],
            "clamp：(999,-5)→(200,0)"
        );

        // 未绑定句柄：true=已入队（双轴 2 条），drain 静默丢弃（unprimed 规则）。
        assert!(p.scroll_to(&format!("__{key}-未绑定-2"), 1.0, 2.0));
        assert_eq!(controller::pending_intent_count(), 2);
        let (finals, _) = controller::drain_resolved_intents();
        assert!(finals.is_empty(), "未绑定 intent 静默丢弃（恒 true 语义）");
        assert_eq!(controller::pending_intent_count(), 0);

        controller::reset_for_test();
    }
}

// ── OS 剪贴板（V06：真 arboard 桥往返） ─────────────────────────────────

#[cfg(all(feature = "code-editor", feature = "ui-clipboard"))]
mod plan802_clipboard {
    use crate::ui::code_editor_provider::CodeEditorRegistryProvider;
    use auto_lang_core::vm::host_capabilities::CodeEditorHostProvider;

    /// cut/copy/paste 经 OS 剪贴板真往返：copy 不删、cut 删、paste 插入；
    /// readonly 编辑器 cut/paste=false、copy=true（PLAN-089 面）。剪贴板
    /// 保存恢复；headless（arboard 打不开）静默跳过——不算 OS 验收缺口，
    /// 实机交互面由 V06 记录（autoui-verifier 未跑时按条件缺口登记）。
    #[test]
    fn os_clipboard_cut_copy_paste_roundtrip() {
        // 跨进程命名互斥（nextest 多进程并行互清剪贴板——485 T9 形）。
        #[cfg(all(windows, feature = "native-clipboard"))]
        let _global = match crate::ui::clipboard_native::GlobalClipboardTestLock::acquire() {
            Some(g) => g,
            None => return, // 拿锁超时视同 headless 跳过
        };
        if arboard::Clipboard::new().is_err() {
            return; // headless CI：arboard 无法初始化
        }
        let saved = crate::ui::clipboard::clipboard_get();

        let key = crate::ui::code_editor::storage_key("p802-clip");
        let _guard = crate::ui::code_editor::core::REGISTRY_TEST_LOCK
            .lock()
            .unwrap();
        fn install_fs(with: &mut dyn FnMut(&mut cosmic_text::FontSystem)) {
            use std::sync::{OnceLock, RwLock};
            static FS: OnceLock<RwLock<cosmic_text::FontSystem>> = OnceLock::new();
            let fs = FS.get_or_init(|| RwLock::new(cosmic_text::FontSystem::new()));
            let mut guard = fs.write().unwrap();
            with(&mut guard);
        }
        crate::ui::code_editor::set_font_system_call(install_fs);
        crate::ui::code_editor::code_editor_dispose(&key);
        let config = crate::ui::code_editor::CodeEditorConfig::default();
        crate::ui::code_editor::code_editor(&key, &config);

        let p = CodeEditorRegistryProvider;
        p.set_text(&key, "甲乙丙-🎯");

        // copy：全选→复制；剪贴板得内容；编辑器文本不变。
        assert!(p.select_all(&key));
        assert_eq!(p.copy(&key), Some(true), "copy=Some(true)（OS 面可用）");
        assert_eq!(
            crate::ui::clipboard::clipboard_get().as_deref(),
            Some("甲乙丙-🎯"),
            "OS 剪贴板得选区内容"
        );
        assert_eq!(
            p.text(&key).as_deref(),
            Some("甲乙丙-🎯"),
            "copy 不删除编辑器文本"
        );

        // cut：全选→剪切；剪贴板得内容；文本清空。
        assert!(p.select_all(&key));
        assert_eq!(p.cut(&key), Some(true));
        assert_eq!(
            crate::ui::clipboard::clipboard_get().as_deref(),
            Some("甲乙丙-🎯")
        );
        assert_eq!(p.text(&key).as_deref(), Some(""), "cut 删除选区");

        // paste：光标处插入剪贴板内容。
        assert_eq!(p.paste(&key), Some(true));
        assert_eq!(
            p.text(&key).as_deref(),
            Some("甲乙丙-🎯"),
            "paste 插入回内容"
        );

        // readonly 编辑器（PLAN-089）：cut/paste 拒绝（false）、copy 允许。
        crate::ui::code_editor::code_editor_dispose(&key);
        let ro_config = crate::ui::code_editor::CodeEditorConfig {
            readonly: true,
            ..Default::default()
        };
        crate::ui::code_editor::code_editor(&key, &ro_config);
        p.set_text(&key, "只读内容");
        assert!(p.select_all(&key));
        assert_eq!(p.copy(&key), Some(true), "readonly copy 允许");
        assert_eq!(p.cut(&key), Some(false), "readonly cut 拒绝（非 None）");
        assert_eq!(p.paste(&key), Some(false), "readonly paste 拒绝（非 None）");
        assert_eq!(
            p.text(&key).as_deref(),
            Some("只读内容"),
            "readonly 文本不变"
        );

        // 恢复剪贴板（原 None 时置空串——arboard 无清空 API 的近似形）。
        match saved {
            Some(t) => {
                crate::ui::clipboard::clipboard_set(&t);
            }
            None => {
                crate::ui::clipboard::clipboard_set("");
            }
        }
        crate::ui::code_editor::code_editor_dispose(&key);
    }
}
