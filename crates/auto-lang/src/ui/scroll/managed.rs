//! PLAN-656 T-06: managed content 的宿主注册表（iced/Vue bridge 共用）。
//!
//! `scroll-test-content`（capability-test 专用 synthetic managed content，
//! plan r2 §11——**非正式 public widget**）的 host 实例按稳定 key 全局驻留，
//! 跨 view 重建保持 semantic offset / logical extent / viewport / intent log
//! （host 单源，pane/backend 只做投影——plan r2 §4.2）。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ui::scroll::host::SyntheticManagedContent;
use crate::ui::scroll::state::ScrollAxes;

lazy_static::lazy_static! {
    static ref MANAGED_HOSTS: Mutex<HashMap<String, Arc<Mutex<SyntheticManagedContent>>>> =
        Mutex::new(HashMap::new());
}

/// 取（或按 axes 创建）key 对应的 managed host。
pub fn managed_host(key: &str, axes: ScrollAxes) -> Arc<Mutex<SyntheticManagedContent>> {
    let mut hosts = MANAGED_HOSTS.lock().unwrap();
    hosts
        .entry(key.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(SyntheticManagedContent::new(axes))))
        .clone()
}

/// 测试面：清空注册表。
#[cfg(test)]
pub fn reset_for_test() {
    MANAGED_HOSTS.lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::scroll::host::ScrollContentHost;
    use crate::ui::scroll::state::Axis;

    #[test]
    fn host_registry_persists_across_lookups() {
        reset_for_test();
        let h1 = managed_host("cap_y", ScrollAxes::Y);
        h1.lock().unwrap().viewport_changed(crate::ui::scroll::ScrollViewportState { width: 100.0, height: 200.0 });
        // 再次取同 key → 同实例（跨重建持久）。
        let h2 = managed_host("cap_y", ScrollAxes::BOTH);
        assert_eq!(h2.lock().unwrap().latest_viewport().height, 200.0);
        // 不同 key → 独立实例。
        let h3 = managed_host("cap_x", ScrollAxes::X);
        h3.lock().unwrap().apply_scroll_intent(crate::ui::scroll::ScrollIntent::ScrollBy {
            axis: Axis::X,
            delta: 5.0,
            source: crate::ui::scroll::ScrollSource::Wheel,
        });
        assert_eq!(h1.lock().unwrap().intent_log().len(), 0);
        assert_eq!(h3.lock().unwrap().intent_log().len(), 1);
        reset_for_test();
    }
}
