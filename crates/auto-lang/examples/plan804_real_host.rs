//! PLAN-804 V05 真宿主链验收 bin（**仅显式构建**：
//! `cargo build -p auto-lang --example plan804_real_host --features native-dnd`
//! ——examples 不进日常档）。
//!
//! 真 main 线程上运行生产 Desktop 模式入口 `run_dynamic_desktop`（单宿主
//! OS 窗口 + 双虚拟窗；`session::dnd_finished_subscription` 仅在此模式
//! 挂载——standalone 不含拖出完成泵）。用法：`plan804_real_host <marker_dir>`。
//! 事件链全程生产接线：按钮 onclick（测试以真实鼠标 SendInput 点击触发
//! ——desktop 模式 Tab/Enter 焦点停在虚拟窗层不达 onclick，实录键盘零
//! 命中；发起方窗置于级联顶层保证按钮可点，见下）→ handler 内 dnd_start
//! （UI 线程内联模态 DoDragDrop）→ 真订阅 → 生产 update 臂
//! DndFinished → dnd_finished_target（生产锚定）→ inject_native_event →
//! 发起方 on_dnd_finished → file.write_text 落盘 + Process.exit(0)。

fn main() {
    auto_lang_core::set_ui_host_install_hook(auto_ui_runtime::install_ui_host_capabilities);
    auto_ui_runtime::install_ui_host_capabilities();
    // 反控钩子（仅验收设施，生产语义零改动）：STALL_MS=run 前停滞
    // （启动超时反控——测试窗口等待应超时并回收本进程）；NO_DRAG=
    // StartDrag handler 空转不发起会话（交付失败反控——测试 finish
    // 断言应失败并回收本进程）。
    if let Ok(ms) = std::env::var("PLAN804_HOST_STALL_MS") {
        let ms: u64 = ms.parse().unwrap_or(60_000);
        eprintln!("[plan804-real-host] stall hook: {ms}ms");
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
    let no_drag = std::env::var("PLAN804_HOST_NO_DRAG").is_ok();

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

    // NO_DRAG 反控：把 AT 模板中的真拖调用替换为空转（started 仍写 true，
    // finish 永不出现——交付失败路径）。
    let initiator_at = if no_drag {
        initiator_at.replace(
            r#"let ok = dnd_start("{\"text\":\"plan804 真宿主链🎉\"}")"#,
            "let ok = true",
        )
    } else {
        initiator_at
    };

    let initiator =
        auto_lang::build_dynamic_component(&initiator_at, None).expect("build initiator");
    let other = auto_lang::build_dynamic_component(&other_at, None).expect("build other");
    // 组件序 = 虚拟窗级联序（renderer.rs boot：第 i 窗 (80+48i,80+48i)、
    // 60% 宿主尺寸，后挂者顶层）。**发起方置 component[1]=顶层**——实录
    // （截图+零 CALL_HANDLER）：component[0] 被 60% 尺寸+48px 级联的后窗
    // 几乎全遮挡，充满窗的按钮零命中。完成事件锚定按发起者身份
    // （dispatch 环置位），与槽序/z 序无关——交换仅为可点性。
    let out = auto_ui_runtime::ui::iced::renderer::run_dynamic_desktop(vec![other, initiator]);
    eprintln!("[plan804-real-host] run out: {out:?}");
    std::process::exit(match out {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("[plan804-real-host] FAILED: {e}");
            1
        }
    });
}
