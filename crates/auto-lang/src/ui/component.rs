// Component abstraction - improved version aligned with Auto language

use super::view::View;
use std::fmt::Debug;

/// Core component trait - simplified and aligned with Auto's `fn on` syntax
///
/// # Example
/// ```rust
/// struct Counter { count: i64 }
///
/// #[derive(Clone)]
/// enum Msg { Inc, Dec }
///
/// impl Component for Counter {
///     type Msg = Msg;
///
///     fn on(&mut self, msg: Self::Msg) {
///         match msg {
///             Msg::Inc => self.count += 1,
///             Msg::Dec => self.count -= 1,
///         }
///     }
///
///     fn view(&self) -> View<Self::Msg> {
///         View::col()
///             .spacing(10)
///             .child(View::button("+", Msg::Inc))
///             .child(View::text(self.count.to_string()))
///             .child(View::button("-", Msg::Dec))
///     }
/// }
/// ```
pub trait Component: Sized + Debug {
    /// Message type - must be cloneable for event handling
    type Msg: Clone + Debug + 'static;

    /// Construct a Tick message from the tick interval timer.
    /// Plan 407: allows run_app to create Tick messages generically
    /// without needing to know the Msg enum's variant names.
    fn tick_msg(&self) -> Option<Self::Msg> {
        None
    }

    /// Declarative keyboard bindings emitted from AutoUI `bind` blocks.
    /// The native Rust runner uses this map together with `key_message`.
    fn key_bindings(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }

    /// Resolve a normalized key into the component's typed message.
    fn key_message(&self, _key: &str) -> Option<Self::Msg> {
        None
    }

    /// Handle messages - Auto's equivalent of `fn on(ev Msg)`
    ///
    /// This is where state mutations happen based on incoming messages.
    fn on(&mut self, msg: Self::Msg);

    /// Render the view - Auto's equivalent of `fn view() View`
    ///
    /// Returns the abstract view tree that will be adapted to specific backends.
    fn view(&self) -> View<Self::Msg>;

    /// Optional periodic tick interval in milliseconds (e.g., `.Tick` handlers).
    ///
    /// Plan 365 W1 follow-up: this replaces the former `subscription()` method
    /// which leaked `iced::Subscription` into this backend-neutral trait. The
    /// iced backend reads this value and builds `iced::time::every(...)` from it;
    /// other backends ignore it. Default: no ticking.
    fn tick_interval_ms(&self) -> Option<u32> {
        None
    }

    /// PLAN-033 T-03②（D2=B）：多 timer 声明驱动的组件（VM 轨）周期泵——
    /// 到期条目逐条派发（`when` 门控在组件内求值，与 renderer 订阅轨的
    /// fire_timer 同门）。返回 true = 本拍有派发（消费端据此前进
    /// revision）。缺省 false——a2r 编译结构体沿用单通道
    /// [`Component::tick_interval_ms`]/[`Component::tick_msg`] 配方，零影响。
    fn fire_due_timers(&mut self) -> bool {
        false
    }

    /// PLAN-033 T-03③（D3）：`__desktop_cmd` 命令读走（read_state + 清空 +
    /// `'\n'` 分行——shell_client c4 语义 child 化泛化到任意投影臂）。缺省
    /// 空——无命令面的组件零实现负担。
    fn drain_desktop_commands(&mut self) -> Vec<String> {
        Vec::new()
    }

    /// PLAN-034 T-05（D3）：位图上传读走（`FrameSource::drain_bitmap_
    /// uploads` 的组件面转发点）。缺省空——无位图生产面的组件零负担；
    /// canvas 快照（VM 轨）/合成位图生产者实现。
    ///
    /// 门控对齐 `desktop_protocol` 模块自身的 `ui-iced` cfg：返回类型
    /// `BitmapUpload` 定义在该门控模块内，实现者/调用者全在门控侧；
    /// 无门控消费者（如生成的 back server，features = ui + image-pipeline）
    /// 编译本 trait 时不得牵引 iced 类型（Plan 365 `subscription()`
    /// 泄漏前科的同类收口）。（PLAN-025 期独立同修,合并取 master 注释版）
    #[cfg(feature = "ui-iced")]
    fn drain_bitmap_uploads(
        &mut self,
    ) -> Vec<crate::ui::desktop_protocol::endpoint::BitmapUpload> {
        Vec::new()
    }

    /// PLAN-033 T-04：L3 v2a 快照注入（融合态 → child 状态迁移落点）——
    /// 逐字段写回 + 续接快照 revision。返回 false = 组件无状态写回路径
    ///（a2r typed 结构体缺省；消费端维持 not-yet 留痕）。
    fn apply_state_snapshot(
        &mut self,
        _revision: u64,
        _fields: &[(String, auto_val::Value)],
    ) -> bool {
        false
    }

    /// Snapshot of this component's scalar state fields, keyed by field name.
    ///
    /// Used by the rust-mode MCP `autoui_state` tool (Plan 371 Task 21): in
    /// rust mode there is no VM heap to read state from, so the DevTools layer
    /// calls this each frame and pushes the result into `SharedState.state`.
    ///
    /// The default returns an empty map -- VM mode never reads this (it reads
    /// state directly off the VM heap), so existing `impl Component for` sites
    /// are unaffected. The a2r generator overrides this per-struct to emit the
    /// scalar fields it knows about (`String`/`i32`/`bool`/`f64`/...), skipping
    /// collections and nested components.
    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_val::Value> {
        std::collections::HashMap::new()
    }
}
