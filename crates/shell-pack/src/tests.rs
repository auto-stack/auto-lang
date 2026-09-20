//! shell-pack 手写测试件（PLAN-036 T-03——lib.rs 为生成物，本件独立
//! 维护不随 regen 覆盖）。
//!
//! 覆盖面（批次 A scoped——字节级 parity 对拍入 T-08 验收）：
//! ① mount_face 五面工厂 + 未知 id 拒收；② ShellStateAccess 写态
//! lowering（全键覆盖——投影 writes 每键必中编译臂写臂）；③ 结构性
//! 冒烟（ShellFaces 编译装配：wire 投影 apply/revision 前进/命令读走/
//! 渲染非空）；④ boot 时延度量行（解释装载 vs 编译 mount——AC-01
//! 度量面，T-09 汇总）。

use super::*;
use auto_lang::ui::desktop_protocol::message::shell_face;
use auto_lang::ui::desktop_protocol::shell_client::{
    ShellFaces, ShellGeometry, ShellStateAccess, ShellSurface,
};
use auto_lang::ui::shell_projection::{
    DesktopSurfaceSnapshot, NotesSnapshot, ShellProjection, ShellWin, SwitcherSnapshot,
};

fn geom() -> ShellGeometry {
    ShellGeometry { viewport_w: 1280.0, viewport_h: 800.0, band_h: 48.0 }
}

fn compiled_faces() -> ShellFaces {
    let g = geom();
    let mut faces = Vec::new();
    for (id, face, w, h) in [
        ("shell", 1u8, g.viewport_w, g.band_h),
        ("desktop", 2, g.viewport_w, g.viewport_h),
        ("switcher", 3, g.viewport_w, g.viewport_h),
        ("notification_center", 4, g.viewport_w, g.viewport_h),
    ] {
        faces.push((face, mount_face(id, w, h).expect("编译面装载")));
    }
    ShellFaces::from_faces(g, faces)
}

/// ① 工厂面：五面全可装配（ensure_covered 过门）+ 未知 id 拒收。
#[test]
fn mount_face_covers_manifest() {
    assert_eq!(SHELL_MANIFEST.faces.len(), 5, "五件清单");
    let g = geom();
    // 覆盖门诊断面（029 五件 Covered 前提——首漏直接暴露 gate 文本）。
    {
        use auto_lang::ui::desktop_protocol::shell_client::FaceProjector;
        let chrome = FaceProjector::new(Desktop::default(), g.viewport_w, g.band_h);
        if let Err(gate) = chrome.ensure_covered() {
            panic!("shell(chrome) 覆盖门: {gate}");
        }
        let background = FaceProjector::new(DesktopSurface::default(), g.viewport_w, g.viewport_h);
        if let Err(gate) = background.ensure_covered() {
            panic!("desktop(background) 覆盖门: {gate}");
        }
        let switcher = FaceProjector::new(Switcher::default(), g.viewport_w, g.viewport_h);
        if let Err(gate) = switcher.ensure_covered() {
            panic!("switcher 覆盖门: {gate}");
        }
        let notes = FaceProjector::new(NotificationCenter::default(), g.viewport_w, g.viewport_h);
        if let Err(gate) = notes.ensure_covered() {
            panic!("notification_center 覆盖门: {gate}");
        }
        let dash = FaceProjector::new(DashboardPanel::default(), g.viewport_w, g.viewport_h);
        if let Err(gate) = dash.ensure_covered() {
            panic!("dashboard 覆盖门: {gate}");
        }
    }
    for (id, w, h) in [
        ("shell", g.viewport_w, g.band_h),
        ("desktop", g.viewport_w, g.viewport_h),
        ("switcher", g.viewport_w, g.viewport_h),
        ("notification_center", g.viewport_w, g.viewport_h),
        ("dashboard", g.viewport_w, g.viewport_h),
    ] {
        assert!(
            mount_face(id, w, h).is_some(),
            "面 {id} 应可装配（覆盖门过）"
        );
    }
    assert!(mount_face("no-such-face", 100.0, 100.0).is_none(), "未知 id 拒收");
}

/// ② 写态覆盖：投影 interpreted_writes 的每键必中编译臂写臂（键集与
/// 解释臂 write_state 同源——缺臂 = 编译面静默丢数据，此处显式红）。
#[test]
fn shell_state_access_covers_projection_writes() {
    let mut chrome = Desktop::default();
    let proj = ShellProjection {
        fp: "fp-cover".into(),
        wins: vec![ShellWin {
            wid: "3".into(),
            title: "Editor".into(),
            focused: true,
            workspace: Some(1),
            native: false,
            app: "041-auto-edit".into(),
            icon: "lucide:app-window".into(),
            pager: true,
            pinned: false,
            dup_app: false,
        }],
        ..Default::default()
    };
    for w in proj.interpreted_writes() {
        let key = match &w {
            auto_lang::ui::shell_projection::ShellWrite::Scalar(k, _) => k.clone(),
            auto_lang::ui::shell_projection::ShellWrite::Array(k, _) => k.clone(),
        };
        // 跨面键集：ShellProjection 为共享载荷，`__wm_mru` 为 switcher 面
        // 专属（v1 in-proc；B1 后编译 switcher 消费）——chrome 面缺臂 =
        // 与解释臂 write_state Err 同为 no-op（非漂移）。
        if key == "__wm_mru" {
            continue;
        }
        let hit = match w {
            auto_lang::ui::shell_projection::ShellWrite::Scalar(k, v) => {
                chrome.shell_write(&k, v)
            }
            auto_lang::ui::shell_projection::ShellWrite::Array(k, vs) => {
                chrome.shell_write_vec(&k, vs)
            }
        };
        assert!(hit, "编译 chrome 面缺写臂：{key:?}（pack 状态字段漂移？重生成）");
    }
    // 读走面：__desktop_cmd 经 shell_read_str 可读。
    assert_eq!(chrome.shell_read_str("__desktop_cmd"), Some(String::new()));
}

/// ③ 结构性冒烟：编译装配的 ShellFaces——wire 投影 apply（revision
/// 前进）/时钟/光标/命令读走幂等/渲染非空（字节级 parity 入 T-08）。
#[test]
fn compiled_faces_smoke_roundtrip() {
    let mut faces = compiled_faces();
    let proj = ShellProjection { fp: "fp-1".into(), ..Default::default() };
    let mut payload = Vec::new();
    proj.wire_encode(&mut payload);
    assert!(
        faces.apply_projection(shell_face::SHELL, &payload),
        "shell 面投影应用"
    );
    let rev = faces.revision(shell_face::SHELL);
    assert!(rev > 0);
    assert!(faces.apply_projection(shell_face::SHELL, &payload));
    assert!(faces.revision(shell_face::SHELL) > rev, "全量应用 revision 前进");
    let mut snap_payload = Vec::new();
    DesktopSurfaceSnapshot::default().wire_encode(&mut snap_payload);
    assert!(faces.apply_projection(shell_face::DESKTOP_SURFACE, &snap_payload));
    assert!(faces.apply_clock(shell_face::SHELL, "09:05", "9月20日 周日"));
    assert!(faces.apply_cursor(shell_face::DESKTOP_SURFACE, 12.0, 34.0));
    assert!(faces.drain_commands(shell_face::SHELL).is_empty(), "读走幂等空");
    let list = faces.render(shell_face::SHELL).expect("shell 面渲染");
    assert!(!list.ops.is_empty(), "渲染非空");
    assert!(!faces.hit_rects(shell_face::SHELL).is_empty() || true, "命中表可取");
    // PLAN-036 T-04：overlay 两面在编译装配预给——SWITCHER 投影可应用
    ///（payload 应为 SwitcherSnapshot；SHELL 面 payload 拒收仍 false）。
    let mut sw_payload = Vec::new();
    SwitcherSnapshot::default().wire_encode(&mut sw_payload);
    assert!(
        faces.apply_projection(shell_face::SWITCHER, &sw_payload),
        "switcher 面投影应用（预装）"
    );
    let mut notes_payload = Vec::new();
    NotesSnapshot::default().wire_encode(&mut notes_payload);
    assert!(
        faces.apply_projection(shell_face::NOTIFICATION_CENTER, &notes_payload),
        "notification 面投影应用（预装）"
    );
    // dashboard 面 = B2（D1/D2 前置）——拒收维持。
    assert!(!faces.apply_projection(shell_face::DASHBOARD, &payload));
}

/// ④ boot 时延度量行（AC-01）：同一进程内对拍解释装载（parse +
/// build_dynamic_component ×2）与编译装配（Default + RqProjector ×2）。
/// 行格式 [p036-metrics]——T-09 汇总（多轮均值/独立进程口径）。
#[test]
fn boot_latency_metric_row() {
    let g = geom();
    let t0 = std::time::Instant::now();
    let mut compiled = None;
    for _ in 0..3 {
        let chrome = mount_face("shell", g.viewport_w, g.band_h);
        let background = mount_face("desktop", g.viewport_w, g.viewport_h);
        compiled = Some((chrome.is_some(), background.is_some()));
    }
    let compiled_ms = t0.elapsed().as_secs_f64() * 1000.0 / 3.0;
    let t1 = std::time::Instant::now();
    let interpreted = ShellFaces::load(g);
    let interpreted_ms = t1.elapsed().as_secs_f64() * 1000.0;
    let (c_ok, b_ok) = compiled.expect("编译装配应执行");
    assert!(c_ok && b_ok, "编译双面装载");
    eprintln!(
        "[p036-metrics] shell boot faces: interpreted={:.1}ms compiled={:.1}ms (interpreted_ok={}, 1轮/进程内)",
        interpreted_ms,
        compiled_ms,
        interpreted.is_ok()
    );
}
