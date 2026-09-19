// PLAN-022 最小对照:Stack 两个绝对浮层各含一个按钮,点击打印归属。
// 运行:cargo run -p auto-lang --example p022_stack_click --features ui-iced
use auto_lang::ui::{Component, View};

#[derive(Debug, Default)]
struct StackClickApp {
    hit_a: u32,
    hit_b: u32,
}

#[derive(Clone, Debug)]
enum Msg {
    ClickA,
    ClickB,
}

impl Component for StackClickApp {
    type Msg = Msg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Msg::ClickA => {
                self.hit_a += 1;
            }
            Msg::ClickB => {
                self.hit_b += 1;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        // 与 app.at 分屏同构:relative 容器 + 两个 absolute 浮层(左右各半)。
        let left = View::col()
            .style("absolute z-10 top-0 left-0 w-[400px] h-[400px] bg-[#202020]")
            .child(
                View::button(format!("A {}", self.hit_a))
                    .style("w-full h-full")
                    .on_click(|_| Msg::ClickA)
                    .build(),
            )
            .build();
        let right = View::col()
            .style("absolute z-10 top-0 left-[400px] w-[400px] h-[400px] bg-[#303030]")
            .child(
                View::button(format!("B {}", self.hit_b))
                    .style("w-full h-full")
                    .on_click(|_| Msg::ClickB)
                    .build(),
            )
            .build();
        View::col()
            .style("relative w-full h-screen bg-background")
            .child(left)
            .child(right)
            .build()
    }
}

fn main() -> auto_lang::ui::AppResult<()> {
    eprintln!("stack-click probe up");
    auto_lang::ui::iced::run_app_devtools::<StackClickApp>()
}
