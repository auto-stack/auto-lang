//! PLAN-029 T-03（D2）：Windows `SendInput` FFI——OS 级真机合成键入。
//!
//! 候选 A 基建：合成事件进真实 OS 输入队列 → 焦点 OS 窗（真桌面壳）
//! 的 iced 事件流 → `desktop_window_events` live 臂（T-02 接线）→
//! `route_live_input` → child。**真桌面 SendInput e2e 腿本计划 not-yet
//! 随注**（依赖 B 程序启动序基建 + child 观察 channel；启用前置 =
//! [`foreground_window`] 目标窗前置断言——键盘无坐标问题，P020-D4 阻断
//! 面只伤点击）。本模块交付组装层 + 单测（结构尺寸/字段级），后续真机
//! 腿直用。手写 FFI 先例同型：[`crate::ui::desktop_protocol::shm`]。
//!
//! 全模块 `#[cfg(windows)]`（宿主侧 OS 工具，非跨平台协议面）。

#![cfg(windows)]

// winuser.h 常量（keybdinput 族）。
const INPUT_KEYBOARD: u32 = 1;
const KEYEVENTF_KEYUP: u32 = 0x0002;
const KEYEVENTF_UNICODE: u32 = 0x0004;

/// winuser.h MOUSEINPUT（x64：4×u32 + usize 对齐 = 32 字节）。
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
struct MouseInput {
    dx: i32,
    dy: i32,
    mouse_data: u32,
    dw_flags: u32,
    time: u32,
    dw_extra_info: usize,
}

/// winuser.h KEYBDINPUT（x64：2×u16 + 2×u32 + usize 对齐 = 24 字节）。
#[repr(C)]
#[derive(Clone, Copy)]
struct KeybdInput {
    w_vk: u16,
    w_scan: u16,
    dw_flags: u32,
    time: u32,
    dw_extra_info: usize,
}

/// INPUT 联合体（x64：MOUSEINPUT 最大 32 字节；pad 保尺寸防布局漂移）。
#[repr(C)]
union InputUnion {
    mi: MouseInput,
    ki: KeybdInput,
    pad: [u8; 32],
}

/// winuser.h INPUT（x64：type + 对齐垫 + 联合体 = 40 字节）。
#[repr(C)]
struct Input {
    kind: u32,
    u: InputUnion,
}

#[link(name = "user32")]
extern "system" {
    fn SendInput(c_inputs: u32, p_inputs: *const Input, cb_size: i32) -> u32;
    /// winuser.h `GetForegroundWindow`——真机腿前置断言用（焦点窗 =
    /// 目标窗才发射，防合成事件打进无关窗）。
    fn GetForegroundWindow() -> isize;
}

/// 前台窗口 HWND（0 = 无前台；调用方比对目标窗）。
pub fn foreground_window() -> isize {
    unsafe { GetForegroundWindow() }
}

fn keybd(w_vk: u16, w_scan: u16, dw_flags: u32) -> Input {
    Input {
        kind: INPUT_KEYBOARD,
        u: InputUnion {
            ki: KeybdInput {
                w_vk,
                w_scan,
                dw_flags,
                time: 0,
                dw_extra_info: 0,
            },
        },
    }
}

/// 可打印文本 → UNICODE 事件序列（每字符 keydown+keyup；BMP 外字符按
/// UTF-16 代理对码元展开——SendInput UNICODE 语义）。
pub fn unicode_inputs(text: &str) -> Vec<Input> {
    let mut out = Vec::new();
    for unit in text.encode_utf16() {
        out.push(keybd(0, unit, KEYEVENTF_UNICODE));
        out.push(keybd(0, unit, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP));
    }
    out
}

/// VK 键 → keydown+keyup 事件对（修饰位不在此层——需修饰时先发修饰
/// VK down … up 序列，e2e 腿按需组合）。
pub fn vk_inputs(vk: u32) -> Vec<Input> {
    vec![keybd(vk as u16, 0, 0), keybd(vk as u16, 0, KEYEVENTF_KEYUP)]
}

/// 发射（返回成功注入条数；0 = 被输入桌面/UIPI 拒绝——真机腿须断言
/// = inputs.len()）。
pub fn send(inputs: &[Input]) -> u32 {
    if inputs.is_empty() {
        return 0;
    }
    unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<Input>() as i32,
        )
    }
}

/// PLAN-031 R2：可打印文本一键发射（`Input` 私有——pub 便捷口给 e2e
/// 真机腿；返回成功注入条数 = 2×UTF-16 码元数）。
pub fn send_unicode(text: &str) -> u32 {
    send(&unicode_inputs(text))
}

// —— 组装级单测（零真实发射：send 不触，只验结构/字段）——
#[cfg(test)]
mod tests {
    use super::*;

    /// x64 布局钉死：INPUT = 40 / KEYBDINPUT = 24 / 联合体 = 32——布局
    /// 漂移 = FFI UB，红即拦。
    #[test]
    fn win64_input_layout_pinned() {
        assert_eq!(std::mem::size_of::<KeybdInput>(), 24);
        assert_eq!(std::mem::size_of::<MouseInput>(), 32);
        assert_eq!(std::mem::size_of::<InputUnion>(), 32);
        assert_eq!(std::mem::size_of::<Input>(), 40);
    }

    /// UNICODE 组装：每字符 down+up 两事件，字段与 flag 位钉死。
    #[test]
    fn unicode_inputs_field_roundtrip() {
        let ev = unicode_inputs("a");
        assert_eq!(ev.len(), 2);
        for (i, e) in ev.iter().enumerate() {
            assert_eq!(e.kind, INPUT_KEYBOARD);
            unsafe {
                assert_eq!(e.u.ki.w_vk, 0);
                assert_eq!(e.u.ki.w_scan, 'a' as u32 as u16);
                let want = if i == 0 {
                    KEYEVENTF_UNICODE
                } else {
                    KEYEVENTF_UNICODE | KEYEVENTF_KEYUP
                };
                assert_eq!(e.u.ki.dw_flags, want);
            }
        }
        // BMP 外字符 → 代理对 = 4 事件（2 码元 × down+up）。
        assert_eq!(unicode_inputs("文").len(), 2);
        assert_eq!(unicode_inputs("\u{1F600}").len(), 4);
    }

    /// VK 组装：VK_RETURN(13) down+up 对。
    #[test]
    fn vk_inputs_pair() {
        let ev = vk_inputs(13);
        assert_eq!(ev.len(), 2);
        unsafe {
            assert_eq!(ev[0].u.ki.w_vk, 13);
            assert_eq!(ev[0].u.ki.dw_flags, 0);
            assert_eq!(ev[1].u.ki.w_vk, 13);
            assert_eq!(ev[1].u.ki.dw_flags, KEYEVENTF_KEYUP);
        }
    }

    /// 空输入零发射（不触 FFI）。
    #[test]
    fn send_empty_is_zero() {
        assert_eq!(send(&[]), 0);
    }
}
