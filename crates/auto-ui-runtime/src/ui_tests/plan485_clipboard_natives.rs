// Plan 485 T3: VM natives 层四件套——直接驱动 shim 验证栈协议
// （pop 实参 / push 返回值的 NV 形态）。
// PLAN-814 T-02 迁移上移：本测试消费上层 UI 面（ui::clipboard_media_
// provider 真实 Win32 adapter / GlobalClipboardTestLock），原居
// auto-lang-core vm/ 的 `native-clipboard` 门在 SDK 无定义——迁移后死门、
// 选择丢失（V-05）；按真实所有者上移本包恢复选择（auto_lang_core::vm::* 经 SDK
// 再导出，路径不变）。

use auto_lang_core::vm::engine::AutoVM;
use auto_lang_core::vm::native::{
    shim_clipboard_files_get, shim_clipboard_files_set, shim_clipboard_image_get,
    shim_clipboard_image_set,
};
use auto_lang_core::vm::task::AutoTask;
use auto_lang_core::vm::virt_memory::VirtualFlash;

fn make_vm() -> AutoVM {
    let vm = AutoVM::new(VirtualFlash::new(1024), 1024);
    // PLAN-790：真实臂注入 OS adapter（缺席注入 = G3 值降级，OS 往返
    // 断言会走 headless guard 静默跳过——不算 OS 验收，见计划 V04）。
    #[cfg(all(windows, feature = "native-clipboard"))]
    vm.set_clipboard_media_provider(std::sync::Arc::new(
        crate::ui::clipboard_media_provider::NativeClipboardMediaProvider,
    ));
    vm
}

/// 弹出 shim 压入的堆引用 NV 并归还 RC 计数（测试内的栈纪律）。
fn pop_heap_id(vm: &AutoVM, task: &mut AutoTask) -> u64 {
    let nv = task.ram.pop_nv();
    let id = auto_lang_core::vm::rc::heap_ref_id(nv).expect("heap ref NV") as u64;
    vm.rc_retain_id(id); // pop 消费一次属主
    vm.rc_release_id(id);
    id
}

#[test]
fn clipboard_files_and_image_via_vm_shims() {
    // 跨进程命名互斥：与 ui::clipboard_native / ui::clipboard 的剪贴板
    // 集成测试互斥（nextest 多进程并行会互清剪贴板，见 T9 修复注）。
    #[cfg(all(windows, feature = "native-clipboard"))]
    let _global = match crate::ui::clipboard_native::GlobalClipboardTestLock::acquire() {
        Some(g) => g,
        None => return,
    };
    let vm = make_vm();
    let mut task = AutoTask::new(0, 1024, 0);

    // ── 1. files_set(["C:\...\一.txt", "C:\...\二.png"]) -> true ──
    let mut list = auto_lang_core::vm::types::ListData::<auto_val::Value>::new();
    list.push(auto_val::Value::Str(auto_val::AutoStr::from(
        "C:\\plan485-vm\\示例 一.txt",
    )));
    list.push(auto_val::Value::Str(auto_val::AutoStr::from(
        "C:\\plan485-vm\\second.png",
    )));
    let list_id = vm.insert_heap_object(list);
    vm.rc_push_id(&mut task, list_id as u64);
    shim_clipboard_files_set(&mut task, &vm).unwrap();
    let ok = auto_val::decode_bool(task.ram.pop_nv());
    if !ok {
        return; // headless CI guard：剪贴板不可用，整组跳过
    }

    // ── 2. files_get() -> List<str>（同路径往返） ──
    shim_clipboard_files_get(&mut task, &vm).unwrap();
    let got_id = pop_heap_id(&vm, &mut task);
    let obj = vm.get_heap_object(got_id).unwrap();
    let guard = obj.read().unwrap();
    let got = guard
        .as_any()
        .downcast_ref::<auto_lang_core::vm::types::ListData<auto_val::Value>>()
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
            "C:\\plan485-vm\\示例 一.txt".to_string(),
            "C:\\plan485-vm\\second.png".to_string()
        ]
    );

    // ── 3. image_get() -> None（仅 CF_HDROP，无图像格式 → null NV） ──
    shim_clipboard_image_get(&mut task, &vm).unwrap();
    let nv = task.ram.pop_nv();
    assert!(auto_val::is_null(nv), "expected null, got tag");

    // ── 4. image_set(png) -> true; image_get() -> {path,width,height} ──
    let (w, h) = (3u32, 2u32);
    let rgba: Vec<u8> = (0..w * h * 4).map(|i| (i * 13 + 7) as u8).collect();
    let png_path = std::env::temp_dir().join("plan485-vm-src.png");
    image::save_buffer(&png_path, &rgba, w, h, image::ColorType::Rgba8).unwrap();
    let idx = vm.add_string(png_path.to_string_lossy().into_owned().into_bytes());
    vm.rc_push_str_idx(&mut task, idx as usize);
    shim_clipboard_image_set(&mut task, &vm).unwrap();
    assert!(auto_val::decode_bool(task.ram.pop_nv()));

    shim_clipboard_image_get(&mut task, &vm).unwrap();
    let rec_id = pop_heap_id(&vm, &mut task);
    let obj = vm.get_heap_object(rec_id).unwrap();
    let guard = obj.read().unwrap();
    let rec = guard
        .as_any()
        .downcast_ref::<auto_lang_core::vm::types::ObjectData>()
        .expect("ObjectData record");
    let field = |k: &str| {
        rec.get(&auto_val::ValueKey::Str(auto_val::AutoStr::from(k)))
            .cloned()
    };
    match field("path") {
        Some(auto_val::Value::Str(p)) => {
            assert!(p.as_str().ends_with(".png"));
            assert!(
                std::path::Path::new(p.as_str()).is_file(),
                "temp png exists"
            );
            let _ = std::fs::remove_file(p.as_str());
        }
        other => panic!("path field wrong: {other:?}"),
    }
    assert_eq!(field("width"), Some(auto_val::Value::Int(3)));
    assert_eq!(field("height"), Some(auto_val::Value::Int(2)));
    drop(guard);
    let _ = std::fs::remove_file(&png_path);
}
