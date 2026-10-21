//! PLAN-790: clipboard media natives 2934..2937 host provider 注入——
//! 结构性剪断与装配不变量（in-crate 面；外部 consumer oracle 见
//! test/fixtures/modularity/clipboard-media-provider-consumer，V-01..04）。
//!
//! 锚点对应：
//! - core 剪断：vm/native.rs（shim 实现体）不再含 `ui::clipboard_native::`
//!   直引（本批次的剪断目标；文本族 `ui::clipboard::` 由 plan785 锚守）。
//! - 降级契约（G3，与文本族 VMError 形**不同**）：无 provider → 空表 /
//!   false / null 值语义（迁移前非 Windows / 未开 native-clipboard 臂
//!   同形），且 set 侧仍消费实参保持栈纪律。
//! - 委托契约：实例注入 fake → files/image 四 lane 值与参数精确到达；
//!   Unicode 路径无损；显式失败 set → false 不计成功。
//! - 两种 ListData 表示：files_set 同时收 `ListData<Value>`（仅 Str 元素，
//!   非 Str 跳过）与 `ListData<String>`（全收）。
//! - 装配/身份：注入不 bump binding_revision；Clone/merge 保 provider；
//!   进程默认构造期快照 + 两实例不串线 + 实例覆盖不污染默认。
//! - ownership：重复调用后任务栈释放 + reap，heap 回基线（无悬空/增长）。

mod plan790_media {
    use std::sync::{Arc, Mutex};

    /// core 剪断（永久回归锚）：shim 实现文件零 `ui::clipboard_native::`
    /// 直引（files/image 族 OS 实现经 host_capabilities provider 装配；
    /// 针含尾随 `::`——注释提及不误伤，785 `ui::clipboard::` 同款纪律）；
    /// host_capabilities 零 `ui::` 引用（能力接口与装配层分离）。
    #[test]
    fn core_media_shims_have_no_ui_clipboard_native_reference() {
        let native_rs = include_str!("../../../auto-lang-core/src/vm/native.rs");
        assert!(
            !native_rs.contains("ui::clipboard_native::"),
            "vm/native.rs 不得直引 ui::clipboard_native（PLAN-790 剪断；供值经 host_capabilities provider）"
        );
        let host_caps_rs = include_str!("../../../auto-lang-core/src/vm/host_capabilities.rs");
        assert!(
            !host_caps_rs.contains("ui::"),
            "vm/host_capabilities.rs 是 core 能力接口，不得引用 ui"
        );
    }

    /// 可区分哨兵 fake：files_get 返回固定 Unicode 路径列表；files_set 记录
    /// 收到的精确 PathBuf；image_get 返回固定三元组（None 形用 switch）；
    /// image_set 记录参数并返回可配结果。
    struct FakeMedia {
        files: Vec<std::path::PathBuf>,
        image: Option<crate::vm::host_capabilities::ClipboardImage>,
        received_files: Mutex<Vec<Vec<std::path::PathBuf>>>,
        received_image: Mutex<Vec<std::path::PathBuf>>,
        set_files_ok: bool,
        set_image_ok: bool,
    }

    impl FakeMedia {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                files: vec![
                    std::path::PathBuf::from("C:\\plan790\\示例 目录\\文件一 🎉.txt"),
                    std::path::PathBuf::from("C:\\plan790\\second-file.png"),
                ],
                image: Some(crate::vm::host_capabilities::ClipboardImage {
                    path: std::path::PathBuf::from("C:\\plan790-temp\\图像-β🎉.png"),
                    width: 320,
                    height: 240,
                }),
                received_files: Mutex::new(Vec::new()),
                received_image: Mutex::new(Vec::new()),
                set_files_ok: true,
                set_image_ok: true,
            })
        }

        fn received_files_snapshot(&self) -> Vec<Vec<std::path::PathBuf>> {
            self.received_files.lock().unwrap().clone()
        }

        fn received_image_snapshot(&self) -> Vec<std::path::PathBuf> {
            self.received_image.lock().unwrap().clone()
        }
    }

    impl crate::vm::host_capabilities::ClipboardMediaProvider for FakeMedia {
        fn files_get(&self) -> Vec<std::path::PathBuf> {
            self.files.clone()
        }
        fn files_set(&self, paths: &[std::path::PathBuf]) -> bool {
            self.received_files.lock().unwrap().push(paths.to_vec());
            self.set_files_ok
        }
        fn image_get(&self) -> Option<crate::vm::host_capabilities::ClipboardImage> {
            self.image
                .as_ref()
                .map(|i| crate::vm::host_capabilities::ClipboardImage {
                    path: i.path.clone(),
                    width: i.width,
                    height: i.height,
                })
        }
        fn image_set(&self, png_path: &std::path::Path) -> bool {
            self.received_image
                .lock()
                .unwrap()
                .push(png_path.to_path_buf());
            self.set_image_ok
        }
    }

    /// 弹出 shim 压入的堆引用 NV（既有 tests_clipboard_natives 同款栈纪律）。
    fn pop_heap_id(vm: &crate::vm::engine::AutoVM, task: &mut crate::vm::task::AutoTask) -> u64 {
        let nv = task.ram.pop_nv();
        let id = crate::vm::rc::heap_ref_id(nv).expect("heap ref NV") as u64;
        vm.rc_retain_id(id); // pop 消费一次属主
        vm.rc_release_id(id);
        id
    }

    /// 降级 + 委托契约（直调 shim，无程序编译）：
    /// - 缺席（构造式 `new_without_host_snapshot`）：files_get → **空表**
    ///   （heap 对象存在但零元素）；files_set/image_set → 消费实参后
    ///   **false**；image_get → **null**——G3 值语义非 raise，与迁移前
    ///   非 Windows 臂同形。
    /// - 注入 fake：四 lane 委托供值，Unicode 路径/记录字段精确；set 收
    ///   精确 PathBuf 参数。
    #[test]
    fn shim_degradation_and_provider_delegation() {
        use crate::vm::engine::AutoVM;
        use crate::vm::native::{
            shim_clipboard_files_get, shim_clipboard_files_set, shim_clipboard_image_get,
            shim_clipboard_image_set,
        };
        use crate::vm::task::AutoTask;
        use crate::vm::virt_memory::VirtualFlash;

        // ── 缺席（构造式——不依赖全局槽时序） ──
        let vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 64);
        assert!(
            vm.clipboard_media_provider().is_none(),
            "构造式缺席：跳过快照=无 provider"
        );
        let mut task = AutoTask::new(0, 256, 0);

        // files_get → 空表（ListData<Value> 零元素）。
        shim_clipboard_files_get(&mut task, &vm).expect("缺席=降级非错误");
        let empty_id = pop_heap_id(&vm, &mut task);
        let obj = vm.get_heap_object(empty_id).expect("空表 heap 对象");
        let guard = obj.read().unwrap();
        let empty_list = guard
            .as_any()
            .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
            .expect("ListData<Value>");
        assert!(empty_list.elems.is_empty(), "缺席 files_get=空表");
        drop(guard);

        // files_set → 消费实参（压 list 后弹出）+ false。
        let mut list = crate::vm::types::ListData::<auto_val::Value>::new();
        list.push(auto_val::Value::Str(auto_val::AutoStr::from("C:\\x.png")));
        let list_id = vm.insert_heap_object(list);
        vm.rc_push_id(&mut task, list_id);
        shim_clipboard_files_set(&mut task, &vm).expect("缺席=降级非错误");
        assert!(
            !auto_val::decode_bool(task.ram.pop_nv()),
            "缺席 files_set=false"
        );

        // image_get → null NV。
        shim_clipboard_image_get(&mut task, &vm).expect("缺席=降级非错误");
        assert!(auto_val::is_null(task.ram.pop_nv()), "缺席 image_get=null");

        // image_set → 消费实参 + false。
        let idx = vm.add_string(b"C:\\y.png".to_vec());
        vm.rc_push_str_idx(&mut task, idx);
        shim_clipboard_image_set(&mut task, &vm).expect("缺席=降级非错误");
        assert!(
            !auto_val::decode_bool(task.ram.pop_nv()),
            "缺席 image_set=false"
        );

        // ── 委托：实例注入 fake → 四 lane 供值 ──
        let fake = FakeMedia::new();
        vm.set_clipboard_media_provider(fake.clone());
        assert!(vm.clipboard_media_provider().is_some(), "实例注入在位");

        // files_get → Unicode 路径列表（String lane 元素精确）。
        shim_clipboard_files_get(&mut task, &vm).expect("provider 在=成功");
        let got_id = pop_heap_id(&vm, &mut task);
        let obj = vm.get_heap_object(got_id).expect("列表 heap 对象");
        let guard = obj.read().unwrap();
        let got = guard
            .as_any()
            .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
            .expect("ListData<Value>");
        let strs: Vec<String> = got
            .elems
            .iter()
            .map(|v| match v {
                auto_val::Value::Str(s) => s.as_str().to_string(),
                other => panic!("non-str entry: {other:?}"),
            })
            .collect();
        drop(guard);
        assert_eq!(
            strs,
            vec![
                "C:\\plan790\\示例 目录\\文件一 🎉.txt".to_string(),
                "C:\\plan790\\second-file.png".to_string(),
            ],
            "Unicode 文件列表精确到达（got {strs:?}）"
        );

        // files_set → provider 收精确 PathBuf 列表 + true 出口。
        let mut list = crate::vm::types::ListData::<auto_val::Value>::new();
        list.push(auto_val::Value::Str(auto_val::AutoStr::from(
            "C:\\plan790-set\\参数-甲🎉.txt",
        )));
        let list_id = vm.insert_heap_object(list);
        vm.rc_push_id(&mut task, list_id);
        shim_clipboard_files_set(&mut task, &vm).expect("set 成功");
        assert!(
            auto_val::decode_bool(task.ram.pop_nv()),
            "set 成功=Bool true"
        );
        assert_eq!(
            fake.received_files_snapshot(),
            vec![vec![std::path::PathBuf::from(
                "C:\\plan790-set\\参数-甲🎉.txt"
            )]],
            "files_set 收到精确 PathBuf 参数"
        );

        // image_get → {path,width,height} 记录字段精确（i32 出口）。
        shim_clipboard_image_get(&mut task, &vm).expect("provider 在=成功");
        let rec_id = pop_heap_id(&vm, &mut task);
        let obj = vm.get_heap_object(rec_id).expect("记录 heap 对象");
        let guard = obj.read().unwrap();
        let rec = guard
            .as_any()
            .downcast_ref::<crate::vm::types::ObjectData>()
            .expect("ObjectData record");
        let field = |k: &str| {
            rec.get(&auto_val::ValueKey::Str(auto_val::AutoStr::from(k)))
                .cloned()
        };
        match field("path") {
            Some(auto_val::Value::Str(p)) => {
                assert_eq!(p.as_str(), "C:\\plan790-temp\\图像-β🎉.png")
            }
            other => panic!("path field wrong: {other:?}"),
        }
        assert_eq!(field("width"), Some(auto_val::Value::Int(320)));
        assert_eq!(field("height"), Some(auto_val::Value::Int(240)));
        drop(guard);

        // image_set → 精确路径参数 + true 出口。
        let idx = vm.add_string("C:\\plan790-set\\写入图-γ🎉.png".as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, idx);
        shim_clipboard_image_set(&mut task, &vm).expect("image set 成功");
        assert!(auto_val::decode_bool(task.ram.pop_nv()));
        assert_eq!(
            fake.received_image_snapshot(),
            vec![std::path::PathBuf::from("C:\\plan790-set\\写入图-γ🎉.png")],
            "image_set 收到精确路径参数"
        );

        // image_get=None 形 → null（provider 在但无图像，值语义同缺席）。
        let none_fake = Arc::new(FakeNoneImage);
        vm.set_clipboard_media_provider(none_fake);
        shim_clipboard_image_get(&mut task, &vm).expect("None=降级非错误");
        assert!(auto_val::is_null(task.ram.pop_nv()), "image None → null");
    }

    struct FakeNoneImage;
    impl crate::vm::host_capabilities::ClipboardMediaProvider for FakeNoneImage {
        fn files_get(&self) -> Vec<std::path::PathBuf> {
            Vec::new()
        }
        fn files_set(&self, _paths: &[std::path::PathBuf]) -> bool {
            false
        }
        fn image_get(&self) -> Option<crate::vm::host_capabilities::ClipboardImage> {
            None
        }
        fn image_set(&self, _png_path: &std::path::Path) -> bool {
            false
        }
    }

    /// 两种既有 ListData 表示（V02）：`ListData<Value>`（Str 元素收、非 Str
    /// 元素跳过——迁移前行为）与 `ListData<String>`（全收）。provider 收到
    /// 的路径列表两形一致。
    #[test]
    fn files_set_decodes_both_listdata_forms() {
        use crate::vm::engine::AutoVM;
        use crate::vm::native::shim_clipboard_files_set;
        use crate::vm::task::AutoTask;
        use crate::vm::virt_memory::VirtualFlash;

        // 形一：ListData<Value>（含一个非 Str 元素——被跳过）。
        let vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 64);
        let fake = FakeMedia::new();
        vm.set_clipboard_media_provider(fake.clone());
        let mut task = AutoTask::new(0, 256, 0);
        let mut list = crate::vm::types::ListData::<auto_val::Value>::new();
        list.push(auto_val::Value::Str(auto_val::AutoStr::from(
            "C:\\a\\一.txt",
        )));
        list.push(auto_val::Value::Int(42)); // 非 Str：跳过
        list.push(auto_val::Value::Str(auto_val::AutoStr::from(
            "C:\\b\\二.png",
        )));
        let list_id = vm.insert_heap_object(list);
        vm.rc_push_id(&mut task, list_id);
        shim_clipboard_files_set(&mut task, &vm).unwrap();
        assert!(auto_val::decode_bool(task.ram.pop_nv()));
        assert_eq!(
            fake.received_files_snapshot(),
            vec![vec![
                std::path::PathBuf::from("C:\\a\\一.txt"),
                std::path::PathBuf::from("C:\\b\\二.png"),
            ]],
            "Value 形只收 Str 元素（非 Str 跳过）"
        );

        // 形二：ListData<String>（全收）。
        let mut list = crate::vm::types::ListData::<String>::new();
        list.push("C:\\c\\三 🎉.md".to_string());
        let list_id = vm.insert_heap_object(list);
        vm.rc_push_id(&mut task, list_id);
        shim_clipboard_files_set(&mut task, &vm).unwrap();
        assert!(auto_val::decode_bool(task.ram.pop_nv()));
        assert_eq!(fake.received_files_snapshot().len(), 2, "两次调用均记录");
        assert_eq!(
            fake.received_files_snapshot()[1],
            vec![std::path::PathBuf::from("C:\\c\\三 🎉.md")],
            "String 形全收"
        );

        // 空列表输入：参数照常消费（provider 收空 vec）。
        let empty = crate::vm::types::ListData::<auto_val::Value>::new();
        let list_id = vm.insert_heap_object(empty);
        vm.rc_push_id(&mut task, list_id);
        shim_clipboard_files_set(&mut task, &vm).unwrap();
        assert!(auto_val::decode_bool(task.ram.pop_nv()), "fake 返回 true");
        assert_eq!(
            fake.received_files_snapshot()[2],
            Vec::<std::path::PathBuf>::new(),
            "空列表参数照常到达 provider"
        );

        // 非 list 实参（i32）：不 panic，provider 收空 vec（迁移前
        // downcast 失败静默空集行为保持）。
        task.ram.push_i32(12345);
        shim_clipboard_files_set(&mut task, &vm).unwrap();
        assert!(auto_val::decode_bool(task.ram.pop_nv()));
        assert_eq!(
            fake.received_files_snapshot()[3],
            Vec::<std::path::PathBuf>::new(),
            "非 list 实参=空集消费（不 panic）"
        );
    }

    /// 身份/合同规则：canonical 名→钉扎 ID（2934..2937）；族间不撞；
    /// provider 注入不 bump binding_revision；Clone/merge 保 provider。
    #[test]
    fn identity_and_contract_rules_preserved() {
        use crate::vm::native::{
            shim_clipboard_files_get, NativeInterface, NATIVE_CLIPBOARD_FILES_GET,
            NATIVE_CLIPBOARD_FILES_SET, NATIVE_CLIPBOARD_IMAGE_GET, NATIVE_CLIPBOARD_IMAGE_SET,
            NATIVE_CLIPBOARD_TEXT,
        };

        let ni = NativeInterface::production();
        assert_eq!(
            ni.resolve("auto.clipboard.files_get"),
            Some(NATIVE_CLIPBOARD_FILES_GET),
            "canonical files_get 解析到钉扎 ID"
        );
        assert_eq!(
            ni.resolve("auto.clipboard.files_set"),
            Some(NATIVE_CLIPBOARD_FILES_SET)
        );
        assert_eq!(
            ni.resolve("auto.clipboard.image_get"),
            Some(NATIVE_CLIPBOARD_IMAGE_GET)
        );
        assert_eq!(
            ni.resolve("auto.clipboard.image_set"),
            Some(NATIVE_CLIPBOARD_IMAGE_SET)
        );
        assert_eq!(NATIVE_CLIPBOARD_FILES_GET, 2934);
        assert_eq!(NATIVE_CLIPBOARD_FILES_SET, 2935);
        assert_eq!(NATIVE_CLIPBOARD_IMAGE_GET, 2936);
        assert_eq!(NATIVE_CLIPBOARD_IMAGE_SET, 2937);
        // 族间不撞（邻段实锚：文本 2925）。
        assert_ne!(NATIVE_CLIPBOARD_FILES_GET, NATIVE_CLIPBOARD_TEXT);
        assert_ne!(NATIVE_CLIPBOARD_IMAGE_SET, NATIVE_CLIPBOARD_TEXT);

        // 注入不 bump revision（绑定身份不变，proof 保持）。
        let rev0 = ni.binding_revision();
        ni.set_clipboard_media_provider(FakeMedia::new());
        assert_eq!(
            ni.binding_revision(),
            rev0,
            "provider 注入不得撤销绑定身份/失效 proof"
        );

        // Clone 路径保留 provider（merge_native_interface 经 Clone+merge）。
        let clone = ni.clone();
        assert!(
            clone.clipboard_media_provider().is_some(),
            "NI Clone 不丢 clipboard media provider"
        );

        // 既有规则保持：register_static 重注册 bump revision。
        let mut ni2 = NativeInterface::production();
        let rev2 = ni2.binding_revision();
        ni2.register_static(NATIVE_CLIPBOARD_FILES_GET, shim_clipboard_files_get);
        assert!(ni2.binding_revision() > rev2, "重注册须 bump revision");
    }

    /// 进程默认快照 + 两实例不串线 + 实例覆盖不污染默认（构造式缺席锚
    /// 之上的装配面；写全局槽须持共享隔离锁——F-785-R1-01 纪律）。
    #[test]
    fn process_default_snapshot_and_instance_isolation() {
        use crate::vm::engine::AutoVM;
        use crate::vm::host_capabilities::{
            clear_process_default_clipboard_media_provider,
            process_default_clipboard_media_provider, set_process_default_clipboard_media_provider,
            ClipboardMediaProvider,
        };
        use crate::vm::native::shim_clipboard_files_get;
        use crate::vm::task::AutoTask;
        use crate::vm::virt_memory::VirtualFlash;

        let _slot = crate::vm::host_capabilities::test_default_slot_lock();
        let saved = process_default_clipboard_media_provider();

        // 快照语义：默认 A → vm1 快照 A；换默认 B → vm2 取 B；vm1 不变。
        let a = FakeMedia::new();
        set_process_default_clipboard_media_provider(a.clone());
        let vm1 = AutoVM::new(VirtualFlash::new(0), 64);
        let b = Arc::new(FakeSwapped);
        set_process_default_clipboard_media_provider(b.clone());
        let vm2 = AutoVM::new(VirtualFlash::new(0), 64);

        let read_first_path = |vm: &AutoVM| -> String {
            let mut task = AutoTask::new(0, 256, 0);
            shim_clipboard_files_get(&mut task, vm).expect("provider 在");
            let id = {
                let nv = task.ram.pop_nv();
                crate::vm::rc::heap_ref_id(nv).expect("heap ref") as u64
            };
            // 先读后放（release 至零可能在 reap 前摘除对象）。
            let first = {
                let obj = vm.get_heap_object(id).expect("列表对象");
                let guard = obj.read().unwrap();
                let list = guard
                    .as_any()
                    .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                    .expect("ListData");
                match &list.elems[0] {
                    auto_val::Value::Str(s) => s.as_str().to_string(),
                    other => panic!("{other:?}"),
                }
            };
            vm.rc_release_id(id);
            first
        };
        assert_eq!(
            read_first_path(&vm1),
            "C:\\plan790\\示例 目录\\文件一 🎉.txt",
            "vm1 保持构造期快照 A"
        );
        assert_eq!(
            read_first_path(&vm2),
            "C:\\plan790-swapped\\B-默认.png",
            "vm2 取新默认 B"
        );

        // 实例覆盖 C：只影响本实例，不污染进程默认。
        let c = Arc::new(FakeOverridden);
        vm2.set_clipboard_media_provider(c.clone());
        assert_eq!(read_first_path(&vm2), "C:\\plan790-override\\C-覆盖.txt");
        assert_eq!(
            read_first_path(&vm1),
            "C:\\plan790\\示例 目录\\文件一 🎉.txt",
            "vm2 覆盖不影响 vm1"
        );
        let cur = process_default_clipboard_media_provider().expect("默认仍在");
        assert_eq!(
            ClipboardMediaProvider::files_get(&*cur),
            vec![std::path::PathBuf::from("C:\\plan790-swapped\\B-默认.png")],
            "实例覆盖不得改写进程默认"
        );

        // 还原。
        match saved {
            Some(p) => set_process_default_clipboard_media_provider(p),
            None => clear_process_default_clipboard_media_provider(),
        }
    }

    struct FakeSwapped;
    impl crate::vm::host_capabilities::ClipboardMediaProvider for FakeSwapped {
        fn files_get(&self) -> Vec<std::path::PathBuf> {
            vec![std::path::PathBuf::from("C:\\plan790-swapped\\B-默认.png")]
        }
        fn files_set(&self, _paths: &[std::path::PathBuf]) -> bool {
            true
        }
        fn image_get(&self) -> Option<crate::vm::host_capabilities::ClipboardImage> {
            None
        }
        fn image_set(&self, _png_path: &std::path::Path) -> bool {
            true
        }
    }

    struct FakeOverridden;
    impl crate::vm::host_capabilities::ClipboardMediaProvider for FakeOverridden {
        fn files_get(&self) -> Vec<std::path::PathBuf> {
            vec![std::path::PathBuf::from("C:\\plan790-override\\C-覆盖.txt")]
        }
        fn files_set(&self, _paths: &[std::path::PathBuf]) -> bool {
            true
        }
        fn image_get(&self) -> Option<crate::vm::host_capabilities::ClipboardImage> {
            None
        }
        fn image_set(&self, _png_path: &std::path::Path) -> bool {
            true
        }
    }

    /// ownership/reap（V02）：files_get + image_get 重复调用（各 50 轮），
    /// 每轮 pop 后任务栈释放 + 全量收割——heap 回基线，无持续增长/悬空。
    /// 同时覆盖"provider 返回的临时数据在后续调用后仍可用"（相邻周期）。
    #[test]
    fn repeated_calls_reap_heap_objects() {
        use crate::vm::engine::AutoVM;
        use crate::vm::native::{shim_clipboard_files_get, shim_clipboard_image_get};
        use crate::vm::task::AutoTask;
        use crate::vm::virt_memory::VirtualFlash;

        let vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 64);
        vm.set_clipboard_media_provider(FakeMedia::new());

        let mut task = AutoTask::new(0, 1024, 0);
        // 预热一轮建立基线（heap 无残留后计 len）。
        let cycle = |task: &mut AutoTask| {
            shim_clipboard_files_get(task, &vm).unwrap();
            let nv = task.ram.pop_nv();
            let id = crate::vm::rc::heap_ref_id(nv).expect("list ref") as u64;
            vm.rc_release_id(id);
            shim_clipboard_image_get(task, &vm).unwrap();
            let nv = task.ram.pop_nv();
            if !auto_val::is_null(nv) {
                let id = crate::vm::rc::heap_ref_id(nv).expect("record ref") as u64;
                vm.rc_release_id(id);
            }
        };
        cycle(&mut task);
        vm.rc_release_task_stack(&mut task);
        vm.reap_all();
        let heap_base = vm.heap_objects.len();

        for _ in 0..50 {
            cycle(&mut task);
        }
        vm.rc_release_task_stack(&mut task);
        vm.reap_all();
        assert_eq!(
            vm.heap_objects.len(),
            heap_base,
            "50 轮 files_get+image_get 后 heap 回基线（got {} -> {}）",
            heap_base,
            vm.heap_objects.len()
        );

        // 相邻周期后 provider 数据仍一致可用（无悬空串扰）。
        let mut task2 = AutoTask::new(0, 256, 0);
        shim_clipboard_files_get(&mut task2, &vm).unwrap();
        let nv = task2.ram.pop_nv();
        let id = crate::vm::rc::heap_ref_id(nv).expect("list ref") as u64;
        let obj = vm.get_heap_object(id).expect("对象仍可读");
        let guard = obj.read().unwrap();
        let list = guard
            .as_any()
            .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
            .expect("ListData");
        assert_eq!(list.elems.len(), 2, "相邻周期后列表仍完整");
        drop(guard);
        vm.rc_release_id(id);
    }

    /// merge 路径保 provider（VM 级）：merge_native_interface 后 provider
    /// 仍在且供值不串线。
    #[test]
    fn vm_merge_preserves_media_provider() {
        use crate::vm::engine::AutoVM;
        use crate::vm::native::{shim_clipboard_files_get, NativeInterface};
        use crate::vm::task::AutoTask;
        use crate::vm::virt_memory::VirtualFlash;

        let mut vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 64);
        vm.set_clipboard_media_provider(FakeMedia::new());

        // merge 携带无关动态 shim 的接口（触发 Clone+merge 路径）。
        let mut carrier = NativeInterface::new();
        carrier.register_dynamic(
            |_task: &mut crate::vm::task::AutoTask,
             _vm: &crate::vm::engine::AutoVM|
             -> Result<(), crate::vm::engine::VMError> { Ok(()) },
        );
        vm.merge_native_interface(&carrier);
        assert!(
            vm.clipboard_media_provider().is_some(),
            "merge 路径不丢 clipboard media provider"
        );

        // merge 后供值仍正确。
        let mut task = AutoTask::new(0, 256, 0);
        shim_clipboard_files_get(&mut task, &vm).expect("merge 后供值");
        let nv = task.ram.pop_nv();
        let id = crate::vm::rc::heap_ref_id(nv).expect("list ref") as u64;
        // 先读后放（release 至零可能在 reap 前摘除对象）。
        let len = {
            let obj = vm.get_heap_object(id).expect("对象");
            let guard = obj.read().unwrap();
            let list = guard
                .as_any()
                .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                .expect("ListData");
            list.elems.len()
        };
        vm.rc_release_id(id);
        assert_eq!(len, 2, "merge 后 fake 供值不变");
    }
}
