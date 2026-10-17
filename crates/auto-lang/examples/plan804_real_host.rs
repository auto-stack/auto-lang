//! PLAN-804 V05 真宿主链验收 bin（**仅显式构建**：
//! `cargo build -p auto-lang --example plan804_real_host --features native-dnd`
//! ——examples 不进日常档）。
//!
//! 真 main 线程上运行生产 Desktop 模式入口 `run_dynamic_desktop`（单宿主
//! OS 窗口 + 双虚拟窗；`session::dnd_finished_subscription` 仅在此模式
//! 挂载——standalone 不含拖出完成泵）。用法：`plan804_real_host <marker_dir>`。
//! 事件链全程生产接线：按钮 onclick（测试经 Tab+Enter 键盘触发——免 UI
//! 位置猜测；实录：鼠标网格扫描误点虚拟窗 chrome 关闭钮、Init 自启则
//! 窗口出生即挂阻塞探询）→ handler 内 dnd_start（UI 线程内联模态
//! DoDragDrop）→ 真订阅 → 生产 update 臂
//! DndFinished → dnd_finished_target（生产锚定）→ inject_native_event →
//! 发起方 on_dnd_finished → file.write_text 落盘 + Process.exit(0)。

fn main() {
    let marker_dir = std::env::args()
        .nth(1)
        .expect("usage: plan804_real_host <marker_dir>");
    let started = format!("{}/started.txt", marker_dir.replace('\\', "/"));
    let finish = format!("{}/finish.txt", marker_dir.replace('\\', "/"));
    let other = format!("{}/other-finish.txt", marker_dir.replace('\\', "/"));

    let initiator_at = format!(
        r#"widget InitiatorApp {{
    msg Msg {{
        StartDrag,
        on_dnd_finished(str),
    }}
    model {{ var unused str = "" }}
    view {{
        column {{
            style: "w-full h-full"
            button {{
                onclick: .StartDrag
                style: "w-full h-full"
                text "拖出🎉"
            }}
        }}
    }}
    on {{
        .StartDrag -> {{
            let ok = dnd_start("{{\"text\":\"plan804 真宿主链🎉\"}}")
            file.write_text("{started}", "started=" + ok.str())
        }}
        .on_dnd_finished(eff) -> {{
            file.write_text("{finish}", "finish=" + eff)
            Process.exit(0)
        }}
    }}
}}
"#
    );
    let other_at = format!(
        r#"widget OtherApp {{
    msg Msg {{
        on_dnd_finished(str),
    }}
    model {{ var unused str = "" }}
    view {{
        column {{
            style: "w-full h-full"
            text "干扰方β（无按钮）"
        }}
    }}
    on {{
        .on_dnd_finished(eff) -> {{
            file.write_text("{other}", "other-got:" + eff)
        }}
    }}
}}
"#
    );

    let initiator =
        auto_lang::build_dynamic_component(&initiator_at, None).expect("build initiator");
    let other = auto_lang::build_dynamic_component(&other_at, None).expect("build other");
    let out = auto_lang::ui::iced::renderer::run_dynamic_desktop(vec![initiator, other]);
    eprintln!("[plan804-real-host] run out: {out:?}");
    std::process::exit(match out {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("[plan804-real-host] FAILED: {e}");
            1
        }
    });
}
