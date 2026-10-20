// PLAN-794: UI host scroll provider adapter（779/785/790 装配模式的
// scroll controller/子锚点批次，9900..9905 + 9922/9923）。
//
// 委托 `ui::scroll::controller` 既有注册表/锚点状态机（PLAN-656 队列 +
// PLAN-781 冻结状态机）——renderer 的 binding/measured 状态与 drain 解析
// 仍是**单一真源**：本 adapter 只做 core DTO ↔ UI 类型转换，不复制状态、
// 不新建第二个 registry、不触碰全局队列算法。core（vm/native.rs 八 shim）
// 自本批次起不再直引 ui::scroll，经
// `vm::host_capabilities::ScrollHostProvider` 装配回供值。
//
// 门控：本模块挂 `ui` 单门控（与被委托的 ui::scroll 模块一致——队列层
// 纯 Rust 无 OS 面，无 790 的 windows × native-clipboard 双门控）。no-ui
// 构建不存在本 adapter——VM 缺席 provider → T-01 冻结降级臂
//（Int 0 / false / 0.0 / "error:unsupported" / "unknown"）。
//
// 转换契约（T-01 DTO 冻结）：
// - intent：core `ScrollIntentDto` 无 source 字段（natives 恒程序化），
//   落 `ScrollSource::Programmatic`；轴 `ScrollAxisDto` → `ui::scroll::Axis`。
// - snapshot：`ControllerPaneSnapshot` 六测量直投影；progress 由
//   `ui::scroll::geometry::progress` 几何单源现算（core 不复制滚动算法，
//   与迁移前 shim 内联计算同源同值）。未绑定 handle → 注册表 default
//   全零（`controller_snapshot` 既有语义）。
// - 锚点：`enqueue_anchor_request`/`anchor_request_state` 原函数原语义
//   直通（bad_handle/empty_key/unsupported 校验与 781 状态机留在
//   controller.rs，不迁不移）。
//
// 注册点（与 779/785/790 相同的宿主入口集合，T-04）：
// - `install_ui_host_capabilities`（lib.rs 统一 installer）
// - `run_file_dynamic_ui_inner`（`auto run` UI 入口）
// - `VmBridge` 两构造（UI 组件桥）

use auto_lang_core::vm::host_capabilities::{
    ScrollAnchorStateDto, ScrollAxisDto, ScrollHostProvider, ScrollIntentDto, ScrollSnapshotDto,
};
use std::sync::Arc;

/// controller 注册表的 provider 形态（无状态——委托 lazy_static REGISTRY）。
pub struct ScrollControllerProvider;

impl ScrollHostProvider for ScrollControllerProvider {
    fn next_controller_handle(&self) -> String {
        crate::ui::scroll::next_controller_handle()
    }

    fn enqueue_intent(&self, handle: &str, intent: ScrollIntentDto) {
        use crate::ui::scroll::intent::{ScrollIntent, ScrollSource};
        let ui_intent = match intent {
            ScrollIntentDto::ToStart { axis } => ScrollIntent::ToStart {
                axis: to_ui_axis(axis),
                source: ScrollSource::Programmatic,
            },
            ScrollIntentDto::ToEnd { axis } => ScrollIntent::ToEnd {
                axis: to_ui_axis(axis),
                source: ScrollSource::Programmatic,
            },
            ScrollIntentDto::ScrollBy { axis, delta } => ScrollIntent::ScrollBy {
                axis: to_ui_axis(axis),
                delta,
                source: ScrollSource::Programmatic,
            },
            ScrollIntentDto::ScrollTo { axis, offset } => ScrollIntent::ScrollTo {
                axis: to_ui_axis(axis),
                offset,
                source: ScrollSource::Programmatic,
            },
        };
        crate::ui::scroll::controller::enqueue_intent(handle, ui_intent);
    }

    fn controller_snapshot(&self, handle: &str) -> ScrollSnapshotDto {
        let snap = crate::ui::scroll::controller::controller_snapshot(handle);
        let x = crate::ui::scroll::ScrollAxisState {
            offset: snap.offset_x,
            viewport_extent: snap.viewport_w,
            content_extent: snap.content_w,
        };
        let y = crate::ui::scroll::ScrollAxisState {
            offset: snap.offset_y,
            viewport_extent: snap.viewport_h,
            content_extent: snap.content_h,
        };
        ScrollSnapshotDto {
            offset_x: snap.offset_x,
            offset_y: snap.offset_y,
            viewport_w: snap.viewport_w,
            viewport_h: snap.viewport_h,
            content_w: snap.content_w,
            content_h: snap.content_h,
            progress_x: crate::ui::scroll::geometry::progress(&x),
            progress_y: crate::ui::scroll::geometry::progress(&y),
        }
    }

    fn enqueue_anchor_request(&self, handle: &str, key: &str) -> Result<(), &'static str> {
        crate::ui::scroll::controller::enqueue_anchor_request(handle, key)
    }

    fn anchor_request_state(&self, handle: &str, key: &str) -> Option<ScrollAnchorStateDto> {
        crate::ui::scroll::controller::anchor_request_state(handle, key).map(|s| match s {
            crate::ui::scroll::controller::AnchorRequestState::Pending => {
                ScrollAnchorStateDto::Pending
            }
            crate::ui::scroll::controller::AnchorRequestState::Applied => {
                ScrollAnchorStateDto::Applied
            }
            crate::ui::scroll::controller::AnchorRequestState::Invalid(reason) => {
                ScrollAnchorStateDto::Invalid(reason)
            }
        })
    }
}

fn to_ui_axis(axis: ScrollAxisDto) -> crate::ui::scroll::Axis {
    match axis {
        ScrollAxisDto::X => crate::ui::scroll::Axis::X,
        ScrollAxisDto::Y => crate::ui::scroll::Axis::Y,
    }
}

/// 装配 UI 兼容默认 scroll host provider = controller 注册表 adapter
///（**仅当进程默认槽为空时**安装——`set_..._if_absent` 同锁原子；优先
/// 关系与 779 F779-R2-01 / 785 / 790 逐条相同）：
/// 1. 调用方**显式**注册（`set_process_default_scroll_host_provider`）
///    恒胜——本兼容安装不覆盖；
/// 2. 空槽时补缺（ui 构建下内建宿主入口恢复迁移前 controller 接线）；
/// 3. 实例级 `AutoVM::set_scroll_host_provider` 在 1/2 之上按 VM 独立
///    生效（构造期快照 + 实例覆盖，不受后续进程默认变化影响）。
pub fn install_process_default() {
    auto_lang_core::vm::host_capabilities::set_process_default_scroll_host_provider_if_absent(
        Arc::new(ScrollControllerProvider),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F779-R2-01 优先关系镜像锚（779/785/790 同款纪律）：兼容安装不覆盖
    /// 显式默认；空槽才补缺；显式 set 恒可替换。Fake provider——不触碰
    /// 全局 controller 注册表（真注册表往返由 plan794 in-crate/fixture
    /// 与 781 状态机既有测试覆盖）。F-785-R1-01：本测试 clear/set/restore
    /// 写默认槽——须持 crate 级共享隔离锁（test_default_slot_lock），与
    /// 缺席断言/桥装配测试互斥。
    #[test]
    fn compat_install_never_overrides_explicit_default() {
        struct Fake(&'static str);
        impl ScrollHostProvider for Fake {
            fn next_controller_handle(&self) -> String {
                self.0.to_string()
            }
            fn enqueue_intent(&self, _handle: &str, _intent: ScrollIntentDto) {}
            fn controller_snapshot(&self, _handle: &str) -> ScrollSnapshotDto {
                ScrollSnapshotDto::default()
            }
            fn enqueue_anchor_request(
                &self,
                _handle: &str,
                _key: &str,
            ) -> Result<(), &'static str> {
                Ok(())
            }
            fn anchor_request_state(
                &self,
                _handle: &str,
                _key: &str,
            ) -> Option<ScrollAnchorStateDto> {
                None
            }
        }

        let _slot = auto_lang_core::vm::host_capabilities::test_default_slot_lock();
        let saved = auto_lang_core::vm::host_capabilities::process_default_scroll_host_provider();

        // 空槽 → 补缺成功（仅入槽，不触注册表）。
        auto_lang_core::vm::host_capabilities::clear_process_default_scroll_host_provider();
        install_process_default();
        assert!(
            auto_lang_core::vm::host_capabilities::process_default_scroll_host_provider().is_some(),
            "空槽时兼容默认须补缺"
        );

        // 显式默认 → 兼容安装不覆盖（哨兵句柄可区分）。
        auto_lang_core::vm::host_capabilities::set_process_default_scroll_host_provider(Arc::new(
            Fake("@plan794-explicit:哨兵-🎉"),
        ));
        install_process_default();
        let cur =
            auto_lang_core::vm::host_capabilities::process_default_scroll_host_provider().unwrap();
        assert_eq!(
            ScrollHostProvider::next_controller_handle(&*cur),
            "@plan794-explicit:哨兵-🎉",
            "兼容安装不得覆盖显式默认"
        );

        // 还原。
        match saved {
            Some(p) => {
                auto_lang_core::vm::host_capabilities::set_process_default_scroll_host_provider(p)
            }
            None => {
                auto_lang_core::vm::host_capabilities::clear_process_default_scroll_host_provider()
            }
        }
    }

    /// adapter 同源读：经 provider 接口的快照读出与直接调注册表逐值一致
    ///（progress 与迁移前 shim 内联计算同源——geometry::progress 单源）。
    #[test]
    fn adapter_snapshot_matches_registry_projection() {
        crate::ui::scroll::controller::reset_for_test();
        let p = ScrollControllerProvider;
        // 未绑定 handle → 全零默认（注册表既有语义）。
        assert_eq!(
            p.controller_snapshot("p794-未绑定-句柄"),
            ScrollSnapshotDto::default(),
            "未绑定 handle 快照=全零默认"
        );
        // 绑定 + 测量 → 六测量直投影 + progress 现算。
        let h = crate::ui::scroll::next_controller_handle();
        crate::ui::scroll::controller::bind_controller(&h, "pane_p794");
        let _ = crate::ui::scroll::controller::take_prime_requests();
        crate::ui::scroll::controller::note_controller_state(
            &h,
            (10.0, 100.0),
            (300.0, 200.0),
            (500.0, 1000.0),
        );
        let snap = p.controller_snapshot(&h);
        assert_eq!(snap.offset_x, 10.0);
        assert_eq!(snap.offset_y, 100.0);
        assert_eq!(snap.viewport_w, 300.0);
        assert_eq!(snap.viewport_h, 200.0);
        assert_eq!(snap.content_w, 500.0);
        assert_eq!(snap.content_h, 1000.0);
        assert_eq!(snap.progress_y, 100.0 / 800.0);
        crate::ui::scroll::controller::reset_for_test();
    }

    /// intent/锚点直通：DTO → UI intent 落注册表队列（轴/类型/序正确）；
    /// enqueue_anchor_request 原语义（bad_handle/empty_key/unsupported）。
    #[test]
    fn adapter_intents_and_anchor_semantics_passthrough() {
        use crate::ui::scroll::controller;
        controller::reset_for_test();
        let p = ScrollControllerProvider;
        p.enqueue_intent(
            "@scrollctl:p794",
            ScrollIntentDto::ScrollTo {
                axis: ScrollAxisDto::X,
                offset: 42.0,
            },
        );
        assert_eq!(
            controller::pending_intent_count(),
            1,
            "DTO intent 经 adapter 落注册表队列"
        );
        // 队列内容形状由 drain 侧消费验证（此处仅证入队直通 + 数量）。
        let _ = controller::drain_resolved_intents();
        assert_eq!(controller::pending_intent_count(), 0);

        // 锚点校验原语义直通（781 controller 原函数）。
        assert_eq!(
            p.enqueue_anchor_request("", "k"),
            Err("bad_handle"),
            "空句柄=bad_handle（provider 透传）"
        );
        assert_eq!(
            p.enqueue_anchor_request("@scrollctl:p794", ""),
            Err("empty_key"),
            "空 key=empty_key（provider 透传）"
        );
        p.enqueue_anchor_request("@scrollctl:p794", "锚-甲🎉")
            .unwrap();
        assert_eq!(
            p.anchor_request_state("@scrollctl:p794", "锚-甲🎉"),
            Some(ScrollAnchorStateDto::Pending),
            "入队成功 → pending（781 状态机）"
        );
        assert_eq!(
            p.anchor_request_state("@scrollctl:p794", "无此键"),
            None,
            "无记录 → None（shim 编组 unknown）"
        );
        controller::reset_for_test();
    }
}
