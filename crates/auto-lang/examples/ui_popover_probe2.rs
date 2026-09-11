//! PLAN-002 N6b 隔离矩阵 B：daemon + DM 消息包装下的 popover 外点关闭。
//!
//! 复刻桌面壳的两个结构特征：①`iced::daemon` 多窗口运行时；②视图消息经
//! `.map(|m| DM::App(app, m))` 包装。若本例外点关闭失败而 ui_popover_probe
//! （application 直跑）正常 → 断点在 daemon/消息包装层；若正常 → 断点在
//! 桌面更深的宿主结构（Stack 分层/多 app 注入等）。
//!
//! 运行：
//!   cargo run --example ui_popover_probe2 --features ui-iced

use auto_lang::ui::iced::renderer::{
    IntoIcedElement, INTER_FONT, INTER_FONT_MEDIUM, INTER_FONT_REGULAR, INTER_FONT_SEMIBOLD,
};
use auto_lang::ui::view::{PopoverAnchor, PopoverPlacement};
use auto_lang::ui::{Component, View};

#[derive(Debug, Default)]
struct Probe {
    open: bool,
    dismisses: u32,
}

#[derive(Clone, Copy, Debug)]
enum Msg {
    Toggle,
    Dismissed,
}

impl Component for Probe {
    type Msg = Msg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Msg::Toggle => {
                self.open = !self.open;
                eprintln!("[probe-b] Toggle → open={}", self.open);
            }
            Msg::Dismissed => {
                self.dismisses += 1;
                self.open = false;
                eprintln!(
                    "[probe-b] Dismissed #{} → open={}",
                    self.dismisses, self.open
                );
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(16)
            .padding(20)
            .child(
                View::Popover {
                    anchor: PopoverAnchor::Widget(Box::new(
                        View::button("Open menu").on_click(|_| Msg::Toggle).build(),
                    )),
                    content: Box::new(
                        View::col()
                            .spacing(4)
                            .padding(8)
                            .child(View::text("Menu item A"))
                            .child(View::text("Menu item B"))
                            .build(),
                    ),
                    placement: PopoverPlacement::BottomStart,
                    open: self.open,
                    on_dismiss: Some(Msg::Dismissed),
                },
            )
            .child(View::text(format!(
                "open={} dismisses={}",
                self.open, self.dismisses
            )))
            .build()
    }
}

// —— daemon + DM 包装（复刻桌面壳消息路径；HKT 教训：必须 fn 条目承载）——

#[derive(Debug, Clone)]
enum DM {
    App(u64, Msg),
    Tick,
}

fn boot() -> (Probe, iced::Task<DM>) {
    // daemon 不自动开窗（桌面先例：boot 显式 window::open）。
    let (id, task) = iced::window::open(iced::window::Settings {
        size: iced::Size::new(800.0, 600.0),
        ..Default::default()
    });
    (Probe::default(), task.map(move |_| DM::Tick).chain(iced::Task::done(DM::Tick)))
}

fn update(state: &mut Probe, msg: DM) -> iced::Task<DM> {
    match msg {
        DM::App(_, m) => {
            eprintln!("[probe-b] DM::App reached update");
            state.on(m);
        }
        DM::Tick => {}
    }
    iced::Task::none()
}

fn view(state: &Probe, _id: iced::window::Id) -> iced::Element<'_, DM> {
    // 复刻桌面：外层 .map 打 DM 包装（overlay 消息必须经此映射）。
    let el: iced::Element<'static, Msg> = state.view().into_iced();
    el.map(|m| DM::App(1, m))
}

fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        eprintln!("[probe-b] daemon+DM-wrapped popover probe starting");
        // PLAN-010 T1：补桌面同款渲染接线（对齐 renderer.rs daemon 装配
        // .font×3+.default_font+.theme），修 B 床黑屏——shadcn_theme 为
        // 私有 fn，此处以 Theme::Dark 达"内容可见"目的（主题不参与
        // overlay 消息路由，不影响本床判据）。
        iced::daemon(boot, update, view)
            .title("Popover Probe B - daemon+DM")
            .font(INTER_FONT_REGULAR)
            .font(INTER_FONT_MEDIUM)
            .font(INTER_FONT_SEMIBOLD)
            .default_font(INTER_FONT)
            .theme(|_state: &Probe, _id: iced::window::Id| iced::Theme::Dark)
            .run()?;
        return Ok(());
    }

    #[cfg(not(feature = "ui-iced"))]
    {
        Err("run with --features ui-iced".into())
    }
}
