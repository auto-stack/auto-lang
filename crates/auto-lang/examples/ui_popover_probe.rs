//! PLAN-002 N6b 最小隔离例：View::Popover 外点关闭在独立窗口是否工作。
//!
//! 背景：桌面壳 icon 右键菜单（popover+ondismiss）外点不关闭——探针实证
//! Panel::update 发布了 on_dismiss 但消息未达 daemon update（scratch/p002/
//! n6b_repro.log）。本例在**独立 OS 窗口**（无 DM 包装/单 UI/HostBackend
//! 直跑）复刻同结构：若这里外点关闭正常 → 断点在桌面宿主集成层；若同样
//! 失败 → 断点在 popover widget/iced 0.14 overlay 本体。
//!
//! 运行：
//!   cargo run --example ui_popover_probe --features ui-iced
//!
//! 预期（stderr）：点 Open menu → [probe] Toggle → open=true；
//! 点面板外任意处 → [probe] Dismissed → open=false（外点自动关闭）。

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
                eprintln!("[probe] Toggle → open={}", self.open);
            }
            Msg::Dismissed => {
                self.dismisses += 1;
                self.open = false;
                eprintln!(
                    "[probe] Dismissed #{} → open={}",
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

fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        eprintln!("[probe] standalone popover probe starting");
        return auto_lang::ui::HostBackend::Iced.run::<Probe>();
    }

    #[cfg(not(feature = "ui-iced"))]
    {
        Err("run with --features ui-iced".into())
    }
}
