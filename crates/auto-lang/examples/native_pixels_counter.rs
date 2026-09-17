// Plan 020 T-03 —— native 像素臂 e2e 子进程载体（计数器示例）。
//
// 为什么是示例二进制而不是测试二进制 re-exec：winit（Windows）拒绝在
// 非主线程初始化 EventLoop，而 libtest 恒在工作线程跑测试——窗口化
// 像素臂的 e2e 载体必须是自带 main 的生产形态二进制（stage3 t3 的
// `--autodesk-render=independent` 用真 `auto run` 同理；本示例是
// native Component 形态的最小等价物）。
//
// 运行：env `AUTO_020_NATIVE_PIXELS_PIPE=<pipe>`（直连 per-app 管道）。
// 由 pixels.rs 测试 `native_pixels_child_two_process`（AUTO_DESKTOP_E2E
// 实机档）build + spawn。

use auto_lang::ui::component::Component;
use auto_lang::ui::desktop_protocol::pixels;
use auto_lang::ui::desktop_protocol::transport;
use auto_lang::ui::view::View;

const PIPE_ENV: &str = "AUTO_020_NATIVE_PIXELS_PIPE";

/// 计数器 native 组件（a2r 生成物最小同构：typed Msg + 状态结构体 +
/// 无 VM——Plan 020 §5.1 策略 B 的投影/驱动对象形态）。
#[derive(Debug)]
struct NativeCounter {
    count: i64,
}

#[derive(Debug, Clone)]
enum NativeCounterMsg {
    Inc,
}

impl Component for NativeCounter {
    type Msg = NativeCounterMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            NativeCounterMsg::Inc => self.count += 1,
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .child(
                View::button(format!("count: {}", self.count))
                    .on_click(|_| NativeCounterMsg::Inc)
                    .build(),
            )
            .build()
    }
}

fn main() {
    let pipe = std::env::var(PIPE_ENV).expect("env AUTO_020_NATIVE_PIXELS_PIPE");
    let end = transport::connect(&pipe, 10_000).expect("connect");
    let exit = pixels::run_independent_native_child(
        Box::new(end),
        NativeCounter { count: 0 },
        "native-counter",
        "native-counter",
        64.0,
        32.0,
    );
    eprintln!("AUTO020-PIXELS-EXIT={exit:?}");
    if let Err(e) = exit {
        std::process::exit(1);
    }
}
