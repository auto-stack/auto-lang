// Plan 485 T3: VM natives 层四件套——直接驱动 shim 验证栈协议
// （pop 实参 / push 返回值的 NV 形态）。
// Windows 实剪贴板环境一次串行跑通四步（剪贴板是进程全局资源，单测试
// 函数串行消除 cargo test 同进程并行的竞态；nextest 进程隔离下同样成立）；
// 打不开剪贴板（CI 服务会话）时第一步 set 返回 false 即整组静默跳过
//（418 headless guard 同款语义）。非 Windows / 未开 native-clipboard 的
// 降级臂由 cfg 对称排除——本测试仅覆盖真实臂，降级臂的返回值约定在
// `ui/clipboard_native.rs` 模块头与 catalog 注释中文档化（G3）。

use crate::vm::engine::AutoVM;
use crate::vm::native::{
    shim_clipboard_files_get, shim_clipboard_files_set, shim_clipboard_image_get,
    shim_clipboard_image_set,
};
use crate::vm::task::AutoTask;
use crate::vm::virt_memory::VirtualFlash;

fn make_vm() -> AutoVM {
    AutoVM::new(VirtualFlash::new(1024), 1024)
}

/// 弹出 shim 压入的堆引用 NV 并归还 RC 计数（测试内的栈纪律）。
fn pop_heap_id(vm: &AutoVM, task: &mut AutoTask) -> u64 {
    let nv = task.ram.pop_nv();
    let id = crate::vm::rc::heap_ref_id(nv).expect("heap ref NV") as u64;
    vm.rc_retain_id(id); // pop 消费一次属主
    vm.rc_release_id(id);
    id
}

#[test]
fn clipboard_files_and_image_via_vm_shims() {
    let vm = make_vm();
    let mut task = AutoTask::new(0, 1024, 0);

    // ── 1. files_set(["C:\...\一.txt", "C:\...\二.png"]) -> true ──
    let mut list = crate::vm::types::ListData::<auto_val::Value>::new();
    list.push(auto_val::Value::Str(auto_val::AutoStr::from("C:\\plan485-vm\\示例 一.txt")));
    list.push(auto_val::Value::Str(auto_val::AutoStr::from("C:\\plan485-vm\\second.png")));
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
        .downcast_ref::<crate::vm::types::ObjectData>()
        .expect("ObjectData record");
    let field = |k: &str| rec.get(&auto_val::ValueKey::Str(auto_val::AutoStr::from(k))).cloned();
    match field("path") {
        Some(auto_val::Value::Str(p)) => {
            assert!(p.as_str().ends_with(".png"));
            assert!(std::path::Path::new(p.as_str()).is_file(), "temp png exists");
            let _ = std::fs::remove_file(p.as_str());
        }
        other => panic!("path field wrong: {other:?}"),
    }
    assert_eq!(field("width"), Some(auto_val::Value::Int(3)));
    assert_eq!(field("height"), Some(auto_val::Value::Int(2)));
    drop(guard);
    let _ = std::fs::remove_file(&png_path);
}
