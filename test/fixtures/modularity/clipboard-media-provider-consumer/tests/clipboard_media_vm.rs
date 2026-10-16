//! PLAN-790 V-01..V-04：外部 clipboard media provider 注入真 VM 断言。
//!
//! 覆盖（计划 §6 矩阵 + T-01 冻结矩阵）：
//! - V-01 真注入（no-ui 全管线）：fake provider 实值经 VM 内建
//!   2934..2937 到达 stdout（可区分 Unicode 路径列表 + 记录三元组
//!   path/width/height）；set 收精确 PathBuf 参数；两实例不串线。
//! - V-02 ownership/值域：空/正常列表、两种 ListData 表示、图像记录、
//!   重复调用后相邻数据仍可用（owned 数据与 provider 调用窗口解耦）；
//!   显式失败 set→false 不计成功。
//! - V-03 缺席/优先矩阵：no-ui 无装配=裸 VM 缺席 → G3 值降级（空表/
//!   false/null——与文本族 VMError 形不同，逐字面值断言）；显式默认
//!   不被 UI 兼容安装覆盖（ui-host 形）；实例覆盖独立；Clone/merge 保
//!   provider；provider 失败/缺席仍消费实参且不死锁。
//! - V-04 真 OS 往返（仅 windows × ui-host 形编译）：命名互斥锁下
//!   files set→get 即时往返 + image set→get（临时 PNG 消费期可读）；
//!   ui-host 高层入口经兼容安装恢复 OS 接线（读侧成功）；非 Windows
//!   静态控制（cfg(not(windows))：装配不伪造可用 OS，值降级保持）。
//!
//! 进程默认槽是进程级全局——所有触槽测试经 `serialized()` 互斥串行并做
//! save/restore。红线（计划 §5）：no-ui 形态全 fake 零 OS 触碰；ui-host
//! 形态真 OS 写仅 V-04 命名互斥锁下 set→get 即时往返。

use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use auto_lang::vm::engine::AutoVM;
use auto_lang::vm::host_capabilities::{
    clear_process_default_clipboard_media_provider, process_default_clipboard_media_provider,
    set_process_default_clipboard_media_provider, ClipboardImage, ClipboardMediaProvider,
};
use auto_lang::vm::native::NativeInterface;
use auto_lang::vm::task::AutoTask;
use auto_lang::vm::virt_memory::VirtualFlash;

// ── 测试序列化（进程默认槽互斥） ─────────────────────────────────────────

fn serialized() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let m = LOCK.get_or_init(|| Mutex::new(()));
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// 进程默认槽 save/restore RAII（测试结束还原进入前状态）。
struct DefaultGuard(Option<Arc<dyn ClipboardMediaProvider>>);

impl DefaultGuard {
    fn new() -> Self {
        Self(process_default_clipboard_media_provider())
    }
}

impl Drop for DefaultGuard {
    fn drop(&mut self) {
        match self.0.take() {
            Some(p) => set_process_default_clipboard_media_provider(p),
            None => clear_process_default_clipboard_media_provider(),
        }
    }
}

// ── fake provider（可区分哨兵） ──────────────────────────────────────────

struct FakeClipboardMedia {
    /// files_get 返回值（空 = 缺席/不可用值语义）。
    files: Mutex<Vec<std::path::PathBuf>>,
    /// image_get 返回值；None = 无图像格式（→ null）。
    image: Option<ClipboardImage>,
    /// files_set 收到的参数记录（精确 PathBuf 断言用）。
    received_files: Mutex<Vec<Vec<std::path::PathBuf>>>,
    /// image_set 收到的参数记录。
    received_image: Mutex<Vec<std::path::PathBuf>>,
    /// files_set 返回值（false = 显式失败）。
    files_set_result: bool,
}

impl FakeClipboardMedia {
    fn with_files(paths: &[&str]) -> Arc<Self> {
        Arc::new(Self {
            files: Mutex::new(paths.iter().map(std::path::PathBuf::from).collect()),
            image: Some(ClipboardImage {
                path: std::path::PathBuf::from("C:/plan790-fx-temp/图像-β🎉.png"),
                width: 640,
                height: 480,
            }),
            received_files: Mutex::new(Vec::new()),
            received_image: Mutex::new(Vec::new()),
            files_set_result: true,
        })
    }

    fn empty() -> Arc<Self> {
        Arc::new(Self {
            files: Mutex::new(Vec::new()),
            image: None,
            received_files: Mutex::new(Vec::new()),
            received_image: Mutex::new(Vec::new()),
            files_set_result: true,
        })
    }

    fn failing_set() -> Arc<Self> {
        Arc::new(Self {
            files: Mutex::new(vec![std::path::PathBuf::from("never-read")]),
            image: None,
            received_files: Mutex::new(Vec::new()),
            received_image: Mutex::new(Vec::new()),
            files_set_result: false,
        })
    }

    fn received_files_snapshot(&self) -> Vec<Vec<std::path::PathBuf>> {
        self.received_files.lock().unwrap().clone()
    }

    fn received_image_snapshot(&self) -> Vec<std::path::PathBuf> {
        self.received_image.lock().unwrap().clone()
    }
}

impl ClipboardMediaProvider for FakeClipboardMedia {
    fn files_get(&self) -> Vec<std::path::PathBuf> {
        self.files.lock().unwrap().clone()
    }
    fn files_set(&self, paths: &[std::path::PathBuf]) -> bool {
        self.received_files.lock().unwrap().push(paths.to_vec());
        self.files_set_result
    }
    fn image_get(&self) -> Option<ClipboardImage> {
        self.image.as_ref().map(|i| ClipboardImage {
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
        true
    }
}

// ── .at 程序（短名 intrinsic 经 codegen 表解析到 2934..2937） ────────────

const GET_FILES_PROGRAM: &str = concat!(
    "let fs = clipboard_files_get()\n",
    "print(fs.len())\n",
    "if fs.len() > 0 {\n",
    "    print(fs[0])\n",
    "}\n",
);

const GET_IMAGE_PROGRAM: &str = concat!(
    "let img = clipboard_image_get()\n",
    "if img != nil {\n",
    "    print(img.path)\n",
    "    print(img.width)\n",
    "    print(img.height)\n",
    "} else {\n",
    "    print(\"none\")\n",
    "}\n",
);

const SET_FILES_PROGRAM: &str = concat!(
    "let ok = clipboard_files_set([\"C:/plan790-fx-set/参数-甲🎉.txt\", \"C:/plan790-fx-set/second.png\"])\n",
    "print(ok)\n",
);

const SET_IMAGE_PROGRAM: &str = concat!(
    "let ok = clipboard_image_set(\"C:/plan790-fx-set/写入图-γ🎉.png\")\n",
    "print(ok)\n",
);

/// 相邻周期 + 原列表复用（V-02 ownership：owned 数据跨 provider 调用窗口）。
const CYCLE_PROGRAM: &str = concat!(
    "let fs = clipboard_files_get()\n",
    "print(fs.len())\n",
    "let ok = clipboard_files_set([\"C:/plan790-fx-cycle/循环-一.png\"])\n",
    "print(ok)\n",
    "let fs2 = clipboard_files_get()\n",
    "print(fs2.len())\n",
    "print(fs.len())\n",
);

fn stdout_lines(s: &str) -> Vec<&str> {
    s.lines().collect()
}

// ── V-01 真注入（no-ui 全管线 + 低层双实例） ────────────────────────────

/// V-01(a) 全管线：进程默认 fake → 四函数实值经真 VM stdout 到达。
#[test]
fn v01_process_default_reaches_real_vm_stdout() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    let a = FakeClipboardMedia::with_files(&[
        "C:/plan790-fx/示例 目录/文件一 🎉.txt",
        "C:/plan790-fx/second-file.png",
    ]);
    set_process_default_clipboard_media_provider(a.clone());
    let (_, out_files) = auto_lang::run_with_capture(GET_FILES_PROGRAM).expect("run files");
    assert_eq!(
        stdout_lines(&out_files),
        vec!["2", "C:/plan790-fx/示例 目录/文件一 🎉.txt"],
        "files_get 实值到达 stdout（got {out_files:?}）"
    );

    let (_, out_img) = auto_lang::run_with_capture(GET_IMAGE_PROGRAM).expect("run image");
    assert_eq!(
        stdout_lines(&out_img),
        vec!["C:/plan790-fx-temp/图像-β🎉.png", "640", "480"],
        "image_get 记录三元组精确（path/width/height；got {out_img:?}）"
    );

    // set 侧：精确 PathBuf 参数 + Bool true 出口。
    let (_, out_set) = auto_lang::run_with_capture(SET_FILES_PROGRAM).expect("run set files");
    assert_eq!(stdout_lines(&out_set), vec!["true"], "set 成功=Bool true");
    assert_eq!(
        a.received_files_snapshot(),
        vec![vec![
            std::path::PathBuf::from("C:/plan790-fx-set/参数-甲🎉.txt"),
            std::path::PathBuf::from("C:/plan790-fx-set/second.png"),
        ]],
        "files_set 收到精确 PathBuf 参数（got {:?}）",
        a.received_files_snapshot()
    );

    let (_, out_iset) = auto_lang::run_with_capture(SET_IMAGE_PROGRAM).expect("run set image");
    assert_eq!(stdout_lines(&out_iset), vec!["true"]);
    assert_eq!(
        a.received_image_snapshot(),
        vec![std::path::PathBuf::from("C:/plan790-fx-set/写入图-γ🎉.png")],
        "image_set 收到精确路径参数"
    );
}

/// V-01(b) 低层双活实例：两 VM 同时存活、各持不同 provider；构造期快照
/// ——换进程默认不影响已建实例；实例级覆盖生效且不污染其他实例/默认。
#[test]
fn v01_two_live_instances_do_not_cross_talk() {
    use auto_lang::vm::native::{shim_clipboard_files_get, shim_clipboard_files_set};

    let _g = serialized();
    let _guard = DefaultGuard::new();

    let a = FakeClipboardMedia::with_files(&["C:/plan790-fx/A-快照.txt"]);
    let b = FakeClipboardMedia::with_files(&["C:/plan790-fx/B-默认.txt"]);
    let c = FakeClipboardMedia::with_files(&["C:/plan790-fx/C-覆盖.txt"]);

    set_process_default_clipboard_media_provider(a.clone() as Arc<dyn ClipboardMediaProvider>);
    let vm1 = AutoVM::new(VirtualFlash::new(0), 64); // 快照 A
    set_process_default_clipboard_media_provider(b.clone() as Arc<dyn ClipboardMediaProvider>);
    let vm2 = AutoVM::new(VirtualFlash::new(0), 64); // 快照 B

    let first_path = |vm: &AutoVM| -> String {
        let mut task = AutoTask::new(0, 256, 0);
        shim_clipboard_files_get(&mut task, vm).expect("provider 在");
        let nv = task.ram.pop_nv();
        let id = auto_lang::vm::rc::heap_ref_id(nv).expect("heap ref") as u64;
        let first = {
            let obj = vm.get_heap_object(id).expect("对象");
            let guard = obj.read().unwrap();
            let list = guard
                .as_any()
                .downcast_ref::<auto_lang::vm::types::ListData<auto_val::Value>>()
                .expect("ListData<Value>");
            match &list.elems[0] {
                auto_val::Value::Str(s) => s.as_str().to_string(),
                other => panic!("{other:?}"),
            }
        };
        vm.rc_release_id(id);
        first
    };
    assert_eq!(
        first_path(&vm1),
        "C:/plan790-fx/A-快照.txt",
        "vm1 保持快照 A"
    );
    assert_eq!(
        first_path(&vm2),
        "C:/plan790-fx/B-默认.txt",
        "vm2 取新默认 B"
    );

    // 实例覆盖 C：只影响 vm2。
    vm2.set_clipboard_media_provider(c.clone());
    assert_eq!(first_path(&vm2), "C:/plan790-fx/C-覆盖.txt");
    assert_eq!(
        first_path(&vm1),
        "C:/plan790-fx/A-快照.txt",
        "vm2 覆盖不影响 vm1"
    );
    let cur = process_default_clipboard_media_provider().expect("默认仍在");
    assert_eq!(
        ClipboardMediaProvider::files_get(&*cur),
        vec![std::path::PathBuf::from("C:/plan790-fx/B-默认.txt")],
        "实例覆盖不得改写进程默认"
    );

    // set 侧不串线：vm1（快照 A）收参数，B/C 收不到。
    let mut task = AutoTask::new(0, 256, 0);
    let mut list = auto_lang::vm::types::ListData::<auto_val::Value>::new();
    list.push(auto_val::Value::Str(auto_val::AutoStr::from(
        "C:/plan790-fx-set/set-参数.txt",
    )));
    let list_id = vm1.insert_heap_object(list);
    vm1.rc_push_id(&mut task, list_id);
    shim_clipboard_files_set(&mut task, &vm1).expect("set");
    assert!(auto_val::decode_bool(task.ram.pop_nv()));
    assert_eq!(
        a.received_files_snapshot(),
        vec![vec![std::path::PathBuf::from(
            "C:/plan790-fx-set/set-参数.txt"
        )]],
        "set 精确落到快照 provider A"
    );
    assert!(
        b.received_files_snapshot().is_empty() && c.received_files_snapshot().is_empty(),
        "其他实例/覆盖 provider 不得串线收参"
    );
}

// ── V-02 ownership / 值域（两种 ListData 形态 / 空值 / 相邻周期） ────────

/// 两种既有 ListData 表示直调 shim：`ListData<Value>`（Str 收、非 Str 跳）
/// 与 `ListData<String>`（全收）；空列表参数照常到达 provider。
#[test]
fn v02_both_listdata_forms_and_empty_list() {
    use auto_lang::vm::native::shim_clipboard_files_set;

    let vm = AutoVM::new(VirtualFlash::new(0), 64);
    let fake = FakeClipboardMedia::empty();
    vm.set_clipboard_media_provider(fake.clone());
    let mut task = AutoTask::new(0, 256, 0);

    // 形一：ListData<Value>（含非 Str 元素——跳过）。
    let mut list = auto_lang::vm::types::ListData::<auto_val::Value>::new();
    list.push(auto_val::Value::Str(auto_val::AutoStr::from(
        "C:/p790/一.txt",
    )));
    list.push(auto_val::Value::Int(7)); // 非 Str：跳过
    let id = vm.insert_heap_object(list);
    vm.rc_push_id(&mut task, id);
    shim_clipboard_files_set(&mut task, &vm).unwrap();
    assert!(auto_val::decode_bool(task.ram.pop_nv()));

    // 形二：ListData<String>（全收）。
    let mut list = auto_lang::vm::types::ListData::<String>::new();
    list.push("C:/p790/二 🎉.png".to_string());
    let id = vm.insert_heap_object(list);
    vm.rc_push_id(&mut task, id);
    shim_clipboard_files_set(&mut task, &vm).unwrap();
    assert!(auto_val::decode_bool(task.ram.pop_nv()));

    // 空列表：参数照常消费。
    let empty = auto_lang::vm::types::ListData::<auto_val::Value>::new();
    let id = vm.insert_heap_object(empty);
    vm.rc_push_id(&mut task, id);
    shim_clipboard_files_set(&mut task, &vm).unwrap();
    assert!(auto_val::decode_bool(task.ram.pop_nv()));

    assert_eq!(
        fake.received_files_snapshot(),
        vec![
            vec![std::path::PathBuf::from("C:/p790/一.txt")],
            vec![std::path::PathBuf::from("C:/p790/二 🎉.png")],
            Vec::<std::path::PathBuf>::new(),
        ],
        "两形态解码正确 + 空列表照常到达（got {:?}）",
        fake.received_files_snapshot()
    );
}

/// 相邻周期：set→get→复用原列表后原数据仍正确（owned 值与 provider 调用
/// 窗口解耦）；重复调用无串扰。
#[test]
fn v02_owned_data_survives_adjacent_cycles() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    let p = FakeClipboardMedia::with_files(&["C:/plan790-fx/中文🎉-keep.txt"]);
    set_process_default_clipboard_media_provider(p.clone());
    let (_, out) = auto_lang::run_with_capture(CYCLE_PROGRAM).expect("run cycles");
    assert_eq!(
        stdout_lines(&out),
        vec!["1", "true", "1", "1"],
        "相邻读写后原列表 fs 仍可用（got {out:?}）"
    );
    assert_eq!(
        p.received_files_snapshot(),
        vec![vec![std::path::PathBuf::from(
            "C:/plan790-fx-cycle/循环-一.png"
        )]],
        "相邻 set 收到精确参数"
    );
}

/// 空列表/无图像/失败写：files_get 空=空表（len 0）；image_get None=null
/// （"none" 分支）；files_set 显式失败=false 不计成功。
#[test]
fn v02_empty_none_and_failure_value_semantics() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    set_process_default_clipboard_media_provider(FakeClipboardMedia::empty());
    let (_, out_len) = auto_lang::run_with_capture(GET_FILES_PROGRAM).expect("run empty");
    assert_eq!(stdout_lines(&out_len), vec!["0"], "空能力=空表 len 0");

    let (_, out_none) = auto_lang::run_with_capture(GET_IMAGE_PROGRAM).expect("run none");
    assert_eq!(stdout_lines(&out_none), vec!["none"], "无图像=None→null");

    set_process_default_clipboard_media_provider(FakeClipboardMedia::failing_set());
    let (_, out_fail) = auto_lang::run_with_capture(SET_FILES_PROGRAM).expect("run fail");
    assert_eq!(
        stdout_lines(&out_fail),
        vec!["false"],
        "显式失败=Bool false（不得伪装成功；got {out_fail:?}）"
    );
}

// ── V-03 缺席/优先矩阵 ──────────────────────────────────────────────────

/// 裸 VM 缺席（构造式无快照路径在外部 crate 不可用——改用全局槽缺席形：
/// no-ui 构建无任何装配入口，清槽后新建 VM 即缺席）：files_get=空表、
/// files_set/image_set=false（仍消费实参——程序正常打印不挂起）、
/// image_get=null。**G3 值降级非 VMError**（与文本族差异的逐字面值锚）。
#[test]
fn v03_absent_capability_is_value_degradation() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    // no-ui 构建下 run 族入口不安装任何 media adapter（install 为 no-op）；
    // 清槽即全局缺席。ui-host 构建下 run 入口会兼容补缺真 OS——差分期望
    // 用 cfg 分臂（内容不定不作断言，仅成功）。
    clear_process_default_clipboard_media_provider();

    #[cfg(not(feature = "ui-host"))]
    {
        let (_, out_len) = auto_lang::run_with_capture(GET_FILES_PROGRAM).expect("run");
        assert_eq!(
            stdout_lines(&out_len),
            vec!["0"],
            "缺席 files_get=G3 空表（got {out_len:?}）"
        );
        let (_, out_none) = auto_lang::run_with_capture(GET_IMAGE_PROGRAM).expect("run");
        assert_eq!(
            stdout_lines(&out_none),
            vec!["none"],
            "缺席 image_get=G3 null（got {out_none:?}）"
        );
        // set 侧缺席=消费实参 + false（程序正常完成不死锁）。
        let (_, out_setf) = auto_lang::run_with_capture(SET_FILES_PROGRAM).expect("run");
        assert_eq!(stdout_lines(&out_setf), vec!["false"]);
        let (_, out_seti) = auto_lang::run_with_capture(SET_IMAGE_PROGRAM).expect("run");
        assert_eq!(stdout_lines(&out_seti), vec!["false"]);

        // 裸 VM 零装配缺席锚（不经 run 入口）。
        let vm = AutoVM::new(VirtualFlash::new(0), 64);
        assert!(
            vm.clipboard_media_provider().is_none(),
            "no-ui 裸 VM 无 provider（两构建同形——裸 VM 不经宿主入口）"
        );
    }

    #[cfg(feature = "ui-host")]
    {
        // ui-host：run 入口兼容补缺真 OS adapter（读侧成功，内容不定不
        // 断言——红线：本臂不写真实剪贴板）。
        let (_, _out) = auto_lang::run_with_capture(GET_FILES_PROGRAM)
            .expect("ui-host 高层入口兼容补缺后读侧成功");
        let vm = AutoVM::new(VirtualFlash::new(0), 64);
        assert!(
            vm.clipboard_media_provider().is_some(),
            "ui-host run 入口装配后新 VM 快照到 media provider"
        );
    }
}

/// 显式默认经真实高层入口供值：兼容安装（ui-host）if-absent 不覆盖显式
/// 默认——两构建同一期望（fake 哨兵到达 stdout）。
#[test]
fn v03_explicit_default_survives_entry_install() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    set_process_default_clipboard_media_provider(FakeClipboardMedia::with_files(&[
        "C:/plan790-fx/explicit-wins.png",
    ]));
    let (_, out) = auto_lang::run_with_capture(GET_FILES_PROGRAM).expect("run");
    assert_eq!(
        stdout_lines(&out),
        vec!["1", "C:/plan790-fx/explicit-wins.png"],
        "显式默认不被兼容安装覆盖（got {out:?}）"
    );
}

/// Clone/merge 保 provider + 注入不 bump binding_revision + ID 钉扎
/// （2934..2937；族间不撞）。
#[test]
fn v03_identity_clone_merge_rules_preserved() {
    use auto_lang::vm::native::{
        NATIVE_CLIPBOARD_FILES_GET, NATIVE_CLIPBOARD_FILES_SET, NATIVE_CLIPBOARD_IMAGE_GET,
        NATIVE_CLIPBOARD_IMAGE_SET,
    };

    let ni = NativeInterface::production();
    assert_eq!(ni.resolve("auto.clipboard.files_get"), Some(2934));
    assert_eq!(ni.resolve("auto.clipboard.files_set"), Some(2935));
    assert_eq!(ni.resolve("auto.clipboard.image_get"), Some(2936));
    assert_eq!(ni.resolve("auto.clipboard.image_set"), Some(2937));
    assert_eq!(NATIVE_CLIPBOARD_FILES_GET, 2934);
    assert_eq!(NATIVE_CLIPBOARD_FILES_SET, 2935);
    assert_eq!(NATIVE_CLIPBOARD_IMAGE_GET, 2936);
    assert_eq!(NATIVE_CLIPBOARD_IMAGE_SET, 2937);

    let rev0 = ni.binding_revision();
    ni.set_clipboard_media_provider(FakeClipboardMedia::empty());
    assert_eq!(
        ni.binding_revision(),
        rev0,
        "provider 注入不得撤销绑定身份/失效 proof"
    );
    let clone = ni.clone();
    assert!(
        clone.clipboard_media_provider().is_some(),
        "NI Clone 不丢 clipboard media provider"
    );

    // VM 级 merge：provider 保留。
    let mut vm = AutoVM::new(VirtualFlash::new(0), 64);
    vm.set_clipboard_media_provider(FakeClipboardMedia::with_files(&["C:/p790/m.png"]));
    let mut carrier = NativeInterface::new();
    carrier.register_dynamic(|_t: &mut AutoTask, _v: &AutoVM| Ok(()));
    vm.merge_native_interface(&carrier);
    assert!(
        vm.clipboard_media_provider().is_some(),
        "VM merge 路径不丢 provider"
    );
}

// ── V-04 真 OS 往返（windows × ui-host 形编译） ─────────────────────────

/// 真 OS 四函数往返（命名互斥锁下 set→get 即时往返；headless 打不开
/// 剪贴板时 set 返回 false 整组静默跳过——418/485 同款 guard）。实例级
/// 注入真实 Win32 adapter（不触全局默认槽）。
#[cfg(all(windows, feature = "ui-host"))]
#[test]
fn v04_real_os_files_and_image_roundtrip() {
    use auto_lang::vm::native::{
        shim_clipboard_files_get, shim_clipboard_files_set, shim_clipboard_image_get,
        shim_clipboard_image_set,
    };

    let _global = match auto_lang::ui::clipboard_native::GlobalClipboardTestLock::acquire() {
        Some(g) => g,
        None => return, // 拿锁超时视同 headless 跳过
    };

    let vm = AutoVM::new(VirtualFlash::new(0), 64);
    vm.set_clipboard_media_provider(Arc::new(
        auto_lang::ui::clipboard_media_provider::NativeClipboardMediaProvider,
    ));
    let mut task = AutoTask::new(0, 1024, 0);

    // ── files set→get（CF_HDROP 纯路径列表，文件不必存在） ──
    let paths = vec![
        std::path::PathBuf::from("C:\\plan790-fx-os\\示例 目录\\文件一.txt"),
        std::path::PathBuf::from("C:\\plan790-fx-os\\second-file.png"),
    ];
    let mut list = auto_lang::vm::types::ListData::<auto_val::Value>::new();
    for p in &paths {
        list.push(auto_val::Value::Str(auto_val::AutoStr::from(
            p.to_string_lossy().into_owned(),
        )));
    }
    let list_id = vm.insert_heap_object(list);
    vm.rc_push_id(&mut task, list_id);
    shim_clipboard_files_set(&mut task, &vm).unwrap();
    if !auto_val::decode_bool(task.ram.pop_nv()) {
        return; // headless CI guard：剪贴板不可用，整组跳过
    }

    shim_clipboard_files_get(&mut task, &vm).unwrap();
    let nv = task.ram.pop_nv();
    let got_id = auto_lang::vm::rc::heap_ref_id(nv).expect("heap ref") as u64;
    let got = {
        let obj = vm.get_heap_object(got_id).expect("列表对象");
        let guard = obj.read().unwrap();
        let list = guard
            .as_any()
            .downcast_ref::<auto_lang::vm::types::ListData<auto_val::Value>>()
            .expect("ListData<Value>");
        list.elems
            .iter()
            .map(|v| match v {
                auto_val::Value::Str(s) => s.as_str().to_string(),
                other => panic!("non-str: {other:?}"),
            })
            .collect::<Vec<_>>()
    };
    vm.rc_release_id(got_id);
    assert_eq!(
        got,
        vec![
            "C:\\plan790-fx-os\\示例 目录\\文件一.txt".to_string(),
            "C:\\plan790-fx-os\\second-file.png".to_string(),
        ],
        "真 OS files set→get 即时往返（Unicode 路径无损；got {got:?}）"
    );

    // ── image set→get（合成 PNG→DIBV5→临时 PNG） ──
    let (w, h) = (5u32, 4u32);
    let rgba: Vec<u8> = (0..w * h * 4).map(|i| (i * 9 + 3) as u8).collect();
    let src_png = std::env::temp_dir().join("plan790-fx-src.png");
    image_helper_save_png(&src_png, w, h, &rgba);
    // F790-R1-01 修复：有效性控制独立于被测侧解码器——喂 OS 前先严格
    // 自校验合成 PNG（chunk CRC/zlib 结构/Adler32/扫描线内容），夹具
    // 不得依赖 image 解码器容错（旧版坏校验流曾被宽容接受）。
    image_helper_assert_strict_png(&src_png, w, h, &rgba);

    let idx = vm.add_string(src_png.to_string_lossy().into_owned().into_bytes());
    vm.rc_push_str_idx(&mut task, idx);
    shim_clipboard_image_set(&mut task, &vm).unwrap();
    if !auto_val::decode_bool(task.ram.pop_nv()) {
        let _ = std::fs::remove_file(&src_png);
        return; // headless CI guard
    }

    shim_clipboard_image_get(&mut task, &vm).unwrap();
    let nv = task.ram.pop_nv();
    let rec_id = auto_lang::vm::rc::heap_ref_id(nv).expect("heap ref") as u64;
    let (path_str, got_w, got_h) = {
        let obj = vm.get_heap_object(rec_id).expect("记录对象");
        let guard = obj.read().unwrap();
        let rec = guard
            .as_any()
            .downcast_ref::<auto_lang::vm::types::ObjectData>()
            .expect("ObjectData");
        let field = |k: &str| {
            rec.get(&auto_val::ValueKey::Str(auto_val::AutoStr::from(k)))
                .cloned()
        };
        match field("path") {
            Some(auto_val::Value::Str(p)) => {
                (p.as_str().to_string(), field("width"), field("height"))
            }
            other => panic!("path field wrong: {other:?}"),
        }
    };
    vm.rc_release_id(rec_id);
    assert_eq!(got_w, Some(auto_val::Value::Int(w as i32)));
    assert_eq!(got_h, Some(auto_val::Value::Int(h as i32)));
    // 临时 PNG 消费期可读（provider 返回后不即删——T-01 冻结）：魔数 +
    // IHDR 尺寸正确 + 字节非空（像素级无损往返由 in-crate plan485 既有
    // 测试承担，本外部长程锚证明跨 crate 消费面成立）。
    assert!(path_str.ends_with(".png"), "临时路径 .png 形（{path_str}）");
    let (dec_w, dec_h, bytes) = image_helper_read_png_dims(&path_str);
    assert_eq!((dec_w, dec_h), (w, h), "临时 PNG IHDR 尺寸正确");
    assert!(!bytes.is_empty(), "临时 PNG 字节非空");

    let _ = std::fs::remove_file(&path_str);
    let _ = std::fs::remove_file(&src_png);
}

/// PNG chunk CRC32（RFC 1952 多项式 0xEDB88320——chunk 覆盖 kind+data）。
#[cfg(all(windows, feature = "ui-host"))]
fn image_helper_crc32(data: &[u8]) -> u32 {
    let mut c: u32 = 0xFFFF_FFFF;
    for &b in data {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
    }
    !c
}

/// 最小 PNG 落盘（无压缩存储块 + CRC32/Adler32 手写——fixture 保持零
/// 额外依赖矩阵；真 image 解码由被测侧 auto-lang 承担）。
/// Adler32 只覆盖未压缩数据（RFC 1950——F790-R1-01 修复：旧版误把
/// DEFLATE 存储块 5 字节头算入，严格解压 `incorrect data check`）。
#[cfg(all(windows, feature = "ui-host"))]
fn image_helper_save_png(path: &std::path::Path, w: u32, h: u32, rgba: &[u8]) {
    fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8], crc32: fn(&[u8]) -> u32) {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        out.extend_from_slice(kind);
        out.extend_from_slice(data);
        let mut crc_input = Vec::with_capacity(4 + data.len());
        crc_input.extend_from_slice(kind);
        crc_input.extend_from_slice(data);
        out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
    }
    // zlib 流：单存储块（未压缩）——RFC 1950/1951。
    let raw_len = (w as usize) * (h as usize) * 4 + h as usize; // +filter bytes
    let mut z = vec![0u8; 2 + 5 + raw_len + 4];
    z[0] = 0x78;
    z[1] = 0x01;
    z[2] = 0x01; // BFINAL=1, BTYPE=00
    z[3] = (raw_len & 0xFF) as u8;
    z[4] = (raw_len >> 8) as u8;
    z[5] = (!raw_len & 0xFF) as u8;
    z[6] = (!(raw_len >> 8) & 0xFF) as u8;
    let mut at = 7;
    for row in 0..h as usize {
        z[at] = 0; // filter: None
        at += 1;
        let stride = w as usize * 4;
        z[at..at + stride].copy_from_slice(&rgba[row * stride..(row + 1) * stride]);
        at += stride;
    }
    // Adler32 范围 = 未压缩扫描线数据 z[7..]（不含 zlib 头与存储块头）。
    let adler_start = z.len() - 4;
    let mut adler1: u32 = 1;
    let mut adler2: u32 = 0;
    for &b in &z[7..adler_start] {
        adler1 = (adler1 + b as u32) % 65521;
        adler2 = (adler2 + adler1) % 65521;
    }
    let adler = (adler2 << 16) | adler1;
    z[adler_start..].copy_from_slice(&adler.to_be_bytes());

    let mut png = Vec::new();
    png.extend_from_slice(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8bit RGBA
    chunk(&mut png, b"IHDR", &ihdr, image_helper_crc32);
    chunk(&mut png, b"IDAT", &z, image_helper_crc32);
    chunk(&mut png, b"IEND", &[], image_helper_crc32);
    std::fs::write(path, png).expect("write png");
}

/// 严格 PNG 自校验（F790-R1-01 有效性控制——不依赖被测侧 image 解码
/// 器的容错）：签名 + 逐 chunk 重算 CRC32 + IHDR 形状 + IDAT zlib 结构
///（存储块 LEN/NLEN 一致、长度=扫描线量）+ Adler32 重算比对 + 未压缩
/// 数据逐字节等于期望扫描线。任一不符即 panic——合成输入喂 OS 前，
/// 夹具自身先证明其严格有效。
#[cfg(all(windows, feature = "ui-host"))]
fn image_helper_assert_strict_png(path: &std::path::Path, w: u32, h: u32, rgba: &[u8]) {
    let bytes = std::fs::read(path).expect("read back generated png");
    assert_eq!(
        &bytes[..8],
        &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A],
        "PNG 签名"
    );
    // 逐 chunk：长度+kind+data+CRC 重算。
    let mut at = 8;
    let mut idat: Option<&[u8]> = None;
    let mut saw_ihdr = false;
    while at < bytes.len() {
        let len = u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
        let kind = &bytes[at + 4..at + 8];
        let data = &bytes[at + 8..at + 8 + len];
        let stored_crc = u32::from_be_bytes(bytes[at + 8 + len..at + 12 + len].try_into().unwrap());
        let mut crc_input = Vec::with_capacity(4 + len);
        crc_input.extend_from_slice(kind);
        crc_input.extend_from_slice(data);
        assert_eq!(
            image_helper_crc32(&crc_input),
            stored_crc,
            "chunk {:?} CRC32 必须重算一致",
            String::from_utf8_lossy(kind)
        );
        if kind == b"IHDR" {
            assert_eq!(len, 13, "IHDR 定长 13");
            assert_eq!(
                u32::from_be_bytes(data[0..4].try_into().unwrap()),
                w,
                "IHDR 宽"
            );
            assert_eq!(
                u32::from_be_bytes(data[4..8].try_into().unwrap()),
                h,
                "IHDR 高"
            );
            assert_eq!(&data[8..13], &[8, 6, 0, 0, 0], "8bit RGBA 无交错");
            saw_ihdr = true;
        } else if kind == b"IDAT" {
            assert!(idat.is_none(), "本夹具单 IDAT");
            idat = Some(data);
        }
        at += 12 + len;
    }
    assert!(saw_ihdr, "IHDR 存在");
    let idat = idat.expect("IDAT 存在");
    // zlib 结构：头 + 单存储块 + Adler32。
    assert_eq!(&idat[..2], &[0x78, 0x01], "zlib CMF/FLG（存储级）");
    assert_eq!(idat[2], 0x01, "BFINAL=1, BTYPE=00（存储块）");
    let raw_len = (w as usize) * (h as usize) * 4 + h as usize;
    let len = u16::from_le_bytes([idat[3], idat[4]]) as usize;
    let nlen = u16::from_le_bytes([idat[5], idat[6]]);
    assert_eq!(len, raw_len, "存储块 LEN=扫描线字节数");
    assert_eq!(!len as u16, nlen, "NLEN=LEN 补");
    // Adler32 重算（范围=未压缩数据，修复后的正确口径）。
    let raw = &idat[7..7 + raw_len];
    let mut adler1: u32 = 1;
    let mut adler2: u32 = 0;
    for &b in raw {
        adler1 = (adler1 + b as u32) % 65521;
        adler2 = (adler2 + adler1) % 65521;
    }
    let expect_adler = (adler2 << 16) | adler1;
    assert_eq!(
        u32::from_be_bytes(idat[7 + raw_len..11 + raw_len].try_into().unwrap()),
        expect_adler,
        "Adler32 严格一致（不含 DEFLATE 块头）"
    );
    // 内容逐字节：filter=0 行 + RGBA 扫描线。
    let stride = w as usize * 4;
    for row in 0..h as usize {
        let line = &raw[row * (stride + 1)..(row + 1) * (stride + 1)];
        assert_eq!(line[0], 0, "filter=None");
        assert_eq!(
            &line[1..],
            &rgba[row * stride..(row + 1) * stride],
            "扫描线 {row} 逐字节一致"
        );
    }
}

/// IHDR 尺寸解析（只读 oracle：魔数 + 尺寸 + 全字节返回供非空校验）。
#[cfg(all(windows, feature = "ui-host"))]
fn image_helper_read_png_dims(path: &str) -> (u32, u32, Vec<u8>) {
    let bytes = std::fs::read(path).expect("temp png readable");
    assert_eq!(
        &bytes[..8],
        &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
    );
    let w = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let h = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    (w, h, bytes)
}

/// ui-host 兼容安装真 OS 接线（读侧）：空槽 → run 入口 → 新 VM 快照到
/// provider（windows+native-clipboard）；读侧成功（内容不定不作内容
/// 断言——真实剪贴板可为任意内容）。非 Windows 见下方静态控制测试。
#[cfg(all(windows, feature = "ui-host"))]
#[test]
fn v04_ui_host_entry_installs_real_os_adapter() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    clear_process_default_clipboard_media_provider();
    let (_, _out) =
        auto_lang::run_with_capture(GET_FILES_PROGRAM).expect("ui-host run 入口兼容补缺后读侧成功");
    let cur = process_default_clipboard_media_provider()
        .expect("windows ui-host 空槽经 run 入口补缺 Win32 adapter");
    // 读侧走真 OS（值可空——剪贴板无 CF_HDROP 时空表，同 OS 语义）。
    let _ = ClipboardMediaProvider::files_get(&*cur);
}

/// 非 Windows 静态控制：ui-host 形态下装配**不**伪造可用 OS——media
/// 默认槽保持 None（无 adapter 模块），程序得 G3 值降级（0/false/none）。
/// 本测试在非 Windows 主机运行（Windows 开发机 cfg 排除——静态控制面）。
#[cfg(all(not(windows), feature = "ui-host"))]
#[test]
fn v04_non_windows_does_not_fake_os_default() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    clear_process_default_clipboard_media_provider();
    let (_, _out) = auto_lang::run_with_capture(GET_FILES_PROGRAM).expect("run");
    assert!(
        process_default_clipboard_media_provider().is_none(),
        "非 Windows 不安装 media adapter（不把 UI 默认伪造为可用 OS）"
    );
}
