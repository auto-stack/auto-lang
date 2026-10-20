//! PLAN-811: 媒体宿主域抽取——core 剪断锚/native 冻结/同一 service 与类型
//! 身份/17 shim 真实调用/票据-会话生命周期（in-crate 面）。
//!
//! 外部 no-UI consumer oracle 见
//! `test/fixtures/modularity/media-core-consumer`（V-06：独立 fixture 仅依
//! 赖 auto-media-core，闭包零语言/UI/VM）。生命周期反例（shutdown 终态化、
//! close-vs-scan 竞争、满队列有界关闭、owned-vs-shared 隔离）在
//! auto-media-core 内联测试；真 VM+HTTP 实机面在 back_proxy th 档
//! （031 native-ns session + 字节保真）。本模块：
//! - core 剪断（永久回归锚）：VM/back_proxy/renderer 不再经 `ui::` 路径
//!   消费媒体机制。
//! - native 冻结（V-01）：16 固定 ID 精确、thumb 无固定 ID（按名注册）、
//!   声明文件 17 项与注册名一致。
//! - 同一 service/类型身份（V-01）：旧门面与 core 同类型；门面自由函数与
//!   core 全局 runtime 命中同一 registry 实例与票据。
//! - 17 shim 真实调用（V-01/02）：queue/open/retain/release/close/request/
//!   scan/stats 票据句柄语义 + 会话族 + thumb；request 旧 stub 语义不被
//!   升级（pop 宽高原样返回 handle）。

use auto_lang_core::vm::engine::AutoVM;
use auto_lang_core::vm::ffi::stdlib;
use auto_lang_core::vm::ffi::VMConvertible;
use auto_lang_core::vm::task::AutoTask;
use auto_lang_core::vm::virt_memory::VirtualFlash;

fn fixtures_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/ui/031-image-viewer/tests/fixtures")
}

fn task_vm() -> (AutoTask, AutoVM) {
    let flash = VirtualFlash::new(4096);
    let vm = AutoVM::new(flash, 4096);
    (AutoTask::new(0, 4096, 0), vm)
}

// ── core 剪断锚（feature 中性） ──────────────────────────────────────────

#[test]
fn media_consumers_have_no_ui_path_references() {
    let stdlib_rs = include_str!("../../../auto-lang-core/src/vm/ffi/stdlib.rs");
    assert!(
        !stdlib_rs.contains("ui::image_pipeline"),
        "vm/ffi/stdlib.rs 不得再含 ui::image_pipeline 直引形态（PLAN-811 \
         剪断；17 shim/票据表/HTTP 快路径直调 auto-media-core）"
    );
    let back_proxy_rs = include_str!("../back_proxy.rs");
    assert!(
        !back_proxy_rs.contains("ui::media_service")
            && !back_proxy_rs.contains("ui::image_pipeline"),
        "back_proxy.rs 不得再含 ui:: 媒体直引形态（PLAN-811 剪断）"
    );
    let renderer_rs = include_str!("../ui/iced/renderer.rs");
    assert!(
        !renderer_rs.contains("crate::ui::image_pipeline"),
        "renderer 媒体钩子（纹理键/代次唤醒/notify 状态）直调 core；\
         UI 只留纹理/事件投影（PLAN-811 SD-04）"
    );
}

// ── native 冻结（V-01：ID/按名注册 ABI 不变） ───────────────────────────

#[test]
fn sixteen_fixed_image_native_ids_are_frozen_and_thumb_has_no_fixed_id() {
    let map = &*auto_lang_core::vm::native_registry::NATIVE_ID_MAP;
    let frozen: &[(&str, u16)] = &[
        ("auto.image.queue", 2960),
        ("auto.image.open", 2961),
        ("auto.image.scan", 2962),
        ("auto.image.request", 2963),
        ("auto.image.retain", 2964),
        ("auto.image.release", 2965),
        ("auto.image.close", 2966),
        ("auto.image.stats", 2967),
        ("auto.image.open_session", 2968),
        ("auto.image.snapshot", 2969),
        ("auto.image.current_uri", 2970),
        ("auto.image.names", 2971),
        ("auto.image.navigate", 2972),
        ("auto.image.request_view", 2973),
        ("auto.image.close_session", 2974),
        ("auto.image.session_stats", 2975),
    ];
    for (name, id) in frozen {
        assert_eq!(
            map.get(*name),
            Some(id),
            "{name} 固定 ID 冻结于 {id}（native ABI）"
        );
    }
    // thumb 只按名注册（动态 ID），不得臆造固定 ID 或写入 catalog。
    assert!(
        !map.contains_key("auto.image.thumb"),
        "auto.image.thumb 无固定 ID——按名注册语义冻结（PLAN-811 合同基线 §4）"
    );
    // 声明文件 17 项 #[vm] 与注册名一一对应。
    let decl = include_str!("../../../../stdlib/auto/image.vm.at");
    let declared: Vec<&str> = decl
        .lines()
        .filter_map(|l| {
            l.trim()
                .strip_prefix("fn ")
                .map(|f| f.split('(').next().unwrap_or(""))
        })
        .collect();
    assert_eq!(declared.len(), 17, "image.vm.at 声明 17 项: {declared:?}");
    for name in stdlib::IMAGE_NATIVE_NAMES {
        let short = name.strip_prefix("auto.image.").unwrap();
        assert!(
            declared.contains(&short),
            "{name} 在 image.vm.at 有声明（{declared:?}）"
        );
    }
    assert_eq!(stdlib::IMAGE_NATIVE_NAMES.len(), 17);
}

// ── 同一 service/类型身份（V-01） ────────────────────────────────────────

#[test]
fn facade_types_are_core_types_with_one_shared_service() {
    use std::any::TypeId;
    assert_eq!(
        TypeId::of::<crate::ui::image_pipeline::MediaAssetId>(),
        TypeId::of::<auto_media_core::image_pipeline::MediaAssetId>()
    );
    assert_eq!(
        TypeId::of::<crate::ui::image_pipeline::MediaAssetRegistry>(),
        TypeId::of::<auto_media_core::image_pipeline::MediaAssetRegistry>()
    );
    assert_eq!(
        TypeId::of::<crate::ui::media_service::MediaIndex>(),
        TypeId::of::<auto_media_core::media_service::MediaIndex>()
    );

    // 门面自由函数与 core 全局 runtime 命中同一 registry：门面路径入队的
    // 票据，经 core 路径 lookup/HTTP 可见（同一实例、同一票据命名空间）。
    let fixture = fixtures_root().join("rgb-1x1.png");
    let ticket = crate::ui::image_pipeline::queue_media_path(
        &fixture,
        crate::ui::image_pipeline::MediaPriority::Current,
    );
    let core_registry = auto_media_core::image_pipeline::global_media_registry();
    assert!(matches!(
        core_registry.lookup(ticket.id, ticket.revision),
        auto_media_core::image_pipeline::MediaLookup::Pending
    ));
    let uri = format!("/api/__auto/media/{}/{}", ticket.id, ticket.revision);
    assert_eq!(
        auto_media_core::image_pipeline::media_http_response(core_registry, "GET", &uri, None)
            .status,
        503
    );
    core_registry.release(ticket.id);
}

// ── 17 shim 真实调用（V-01/V-02：票据句柄与会话语义） ───────────────────

#[cfg(feature = "ui-iced")]
mod shims {
    use super::*;

    #[test]
    fn ticket_handle_lifecycle_through_real_shims() {
        let fixture = fixtures_root()
            .join("rgb-1x1.png")
            .to_string_lossy()
            .into_owned();

        // queue(path) → handle（单调递增、非零）。
        let (mut task, vm) = task_vm();
        fixture.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_queue(&mut task, &vm).unwrap();
        let handle = task.ram.pop_i64();
        assert!(handle > 0, "queue 返回句柄: {handle}");

        // request 旧 stub 语义：pop 宽高、原样返回 handle（不被升级）。
        handle.push_to_stack(&mut task, &vm).unwrap();
        640i32.push_to_stack(&mut task, &vm).unwrap();
        480i32.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_request(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i64(), handle, "request stub 原样返回 handle");

        // open(known)=handle；open(unknown)=0。
        handle.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_open(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i64(), handle);
        999999i64.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_open(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i64(), 0, "未知句柄 open=0");

        // retain(known)=handle。
        handle.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_retain(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i64(), handle);
        // release 对未知句柄无害（无返回值）。
        42i64.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_release(&mut task, &vm).unwrap();

        // close 移除句柄：随后 open=0（双 close 不复活）。
        handle.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_close(&mut task, &vm).unwrap();
        handle.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_open(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i64(), 0, "close 后句柄失效");

        // scan(path)→0；stats()→completed 计数（i64）。
        let (mut task, vm) = task_vm();
        let missing = fixtures_root()
            .join("does-not-exist")
            .to_string_lossy()
            .into_owned();
        missing.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_scan(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_i64(), 0, "scan 返回 0");
        stdlib::shim_image_stats(&mut task, &vm).unwrap();
        assert!(task.ram.pop_i64() >= 0, "stats 返回 completed 计数");
    }

    #[test]
    fn session_family_through_real_shims() {
        let root = fixtures_root().to_string_lossy().into_owned();

        // open_session(root) → session id 字符串。
        let (mut task, vm) = task_vm();
        root.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_open_session(&mut task, &vm).unwrap();
        let session: String = String::pop_from_stack(&mut task, &vm).unwrap();
        assert!(session.starts_with("image-session-"), "{session}");

        // populate 是异线程：轮询 names 非空（2s 截止）。
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            let (mut probe, vm2) = task_vm();
            session.clone().push_to_stack(&mut probe, &vm2).unwrap();
            stdlib::shim_image_names(&mut probe, &vm2).unwrap();
            let names: Vec<String> = Vec::pop_from_stack(&mut probe, &vm2).unwrap();
            if !names.is_empty() {
                assert!(names.iter().any(|n| n == "rgb-1x1.png"), "{names:?}");
                break;
            }
            assert!(std::time::Instant::now() < deadline, "populate 超时");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // snapshot JSON：会话 id 在、源路径不在。
        let (mut task, vm) = task_vm();
        session.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_snapshot(&mut task, &vm).unwrap();
        let snapshot: String = String::pop_from_stack(&mut task, &vm).unwrap();
        assert!(snapshot.contains("\"count\":"), "{snapshot}");
        assert!(!snapshot.contains("fixtures"), "快照不泄源路径: {snapshot}");

        // navigate(+1) 返回真会话快照；未知会话返回 error JSON。
        let sid = snapshot_session_id(&snapshot);
        let (mut task, vm) = task_vm();
        sid.clone().push_to_stack(&mut task, &vm).unwrap();
        1i32.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_navigate(&mut task, &vm).unwrap();
        let nav: String = String::pop_from_stack(&mut task, &vm).unwrap();
        assert!(nav.contains("\"selected\":"), "navigate 返回快照: {nav}");
        let (mut task, vm) = task_vm();
        "image-session-999998"
            .to_string()
            .push_to_stack(&mut task, &vm)
            .unwrap();
        1i32.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_navigate(&mut task, &vm).unwrap();
        let unknown: String = String::pop_from_stack(&mut task, &vm).unwrap();
        assert!(
            unknown.contains("\"error\":\"unknown_session\""),
            "navigate 未知会话: {unknown}"
        );

        // request_view(session, 0, 32, 32, 90) → 快照 JSON。
        let (mut task, vm) = task_vm();
        sid.push_to_stack(&mut task, &vm).unwrap();
        0i32.push_to_stack(&mut task, &vm).unwrap();
        32i32.push_to_stack(&mut task, &vm).unwrap();
        32i32.push_to_stack(&mut task, &vm).unwrap();
        90i32.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_request_view(&mut task, &vm).unwrap();
        let view: String = String::pop_from_stack(&mut task, &vm).unwrap();
        assert!(view.contains("\"uri\":"), "request_view 返回快照: {view}");
        assert!(!view.contains("fixtures"), "request_view 不泄路径");

        // session_stats：全局 registry 计数 JSON（现行语义：忽略参数）。
        let (mut task, vm) = task_vm();
        sid.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_session_stats(&mut task, &vm).unwrap();
        let stats: String = String::pop_from_stack(&mut task, &vm).unwrap();
        assert!(stats.contains("\"completed\":"), "{stats}");

        // close_session：未知会话 false；真会话 true→false（双 close）。
        let (mut task, vm) = task_vm();
        "image-session-999999"
            .to_string()
            .push_to_stack(&mut task, &vm)
            .unwrap();
        stdlib::shim_image_close_session(&mut task, &vm).unwrap();
        assert!(
            !bool::pop_from_stack(&mut task, &vm).unwrap(),
            "未知会话 close=false"
        );
        let (mut task, vm) = task_vm();
        sid.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_close_session(&mut task, &vm).unwrap();
        assert!(bool::pop_from_stack(&mut task, &vm).unwrap());
        let (mut task, vm) = task_vm();
        sid.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_close_session(&mut task, &vm).unwrap();
        assert!(
            !bool::pop_from_stack(&mut task, &vm).unwrap(),
            "双 close=false"
        );
    }

    #[test]
    fn thumb_shim_uri_contract() {
        let fixture = fixtures_root()
            .join("rgb-1x1.png")
            .to_string_lossy()
            .into_owned();
        let (mut task, vm) = task_vm();
        fixture.push_to_stack(&mut task, &vm).unwrap();
        64i32.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_thumb(&mut task, &vm).unwrap();
        let uri: String = String::pop_from_stack(&mut task, &vm).unwrap();
        assert!(uri.starts_with("/api/__auto/media/"), "{uri}");
        assert!(!uri.contains("fixtures"), "thumb 不泄路径");

        let (mut task, vm) = task_vm();
        "not-a-file.png"
            .to_string()
            .push_to_stack(&mut task, &vm)
            .unwrap();
        64i32.push_to_stack(&mut task, &vm).unwrap();
        stdlib::shim_image_thumb(&mut task, &vm).unwrap();
        assert_eq!(
            String::pop_from_stack(&mut task, &vm).unwrap(),
            "",
            "坏路径→空串"
        );
    }

    fn snapshot_session_id(snapshot: &str) -> String {
        let value: serde_json::Value = serde_json::from_str(snapshot).expect("snapshot json");
        value["session"].as_str().unwrap().to_string()
    }
}

// shim_image_retain 的未知句柄臂在此内联（上表retain(unknown)=0 断言）。
#[cfg(feature = "ui-iced")]
#[test]
fn retain_unknown_handle_returns_zero() {
    let (mut task, vm) = task_vm();
    424242i64.push_to_stack(&mut task, &vm).unwrap();
    stdlib::shim_image_retain(&mut task, &vm).unwrap();
    assert_eq!(task.ram.pop_i64(), 0, "retain 未知句柄=0");
}
