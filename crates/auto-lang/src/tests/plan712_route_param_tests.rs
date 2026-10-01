//! PLAN-712 T-17 第二层：`router.param("id")` 在 VM 侧的取值链回归。
//!
//! 实机实证（2026-10-01，桌面 + 独立 VM 双轨）：018 点书 → 详情页 fetch
//! `GET /api/books/0`（book_id 落 0）——前缀层修复后请求已到 proxy，但
//! `router.param("id")` 取到空。本测用最小 routes fixture 驱动
//! push → sync_route_params → 页 Init 读 param 的完整链，钉住断点。

use crate::build_dynamic_component;

fn write_fixture(dir: &tempfile::TempDir) -> std::path::PathBuf {
    let pages = dir.path().join("pages");
    std::fs::create_dir_all(&pages).expect("pages dir");
    std::fs::write(
        dir.path().join("app.at"),
        r#"
widget App {
    routes {
        "/" -> use home
        "/book/:id" -> use detail
    }
    view {
        outlet
    }
}
"#,
    )
    .expect("write app.at");
    std::fs::write(
        pages.join("home.at"),
        r#"
widget home {
    msg { Go }

    model {
        var clicked str = "no"
    }

    view {
        text .clicked {}
    }

    on {
        .Go -> {
            .clicked = "yes"
            router.push("/book/7")
        }
    }
}
"#,
    )
    .expect("write pages/home.at");
    std::fs::write(
        pages.join("detail.at"),
        r#"
widget detail {
    msg { Init }

    model {
        var got str = "none"
        var got_int int = 0
    }

    view {
        text .got {}
    }

    on {
        .Init -> {
            .got = router.param("id")
        }
    }
}
"#,
    )
    .expect("write pages/detail.at");
    dir.path().join("app.at")
}

/// push("/book/7") 后 `__route_params` 应含 id="7"，页 Init 的
/// `router.param("id")` 应取到 "7"。
#[test]
#[cfg(feature = "ui-iced")]
fn route_param_survives_push_and_reaches_page_init() {
    let dir = tempfile::tempdir().expect("tempdir");
    let app_path = write_fixture(&dir);
    let code = std::fs::read_to_string(&app_path).expect("read app.at");
    let mut dc = build_dynamic_component(&code, app_path.to_str())
        .expect("routes app 必须能编译");

    // boot：home 页挂载（outlet 首路由）。
    dc.fire_init();
    let _ = dc.view_with_debug_gated(true);
    dc.drive_scheduler_to_quiescence(10_000);

    // 导航：home.Go → router.push("/book/7")。
    dc.on_with_input_for("home", "Go", None);
    let _ = dc.view_with_debug_gated(true);
    dc.drive_scheduler_to_quiescence(10_000);

    eprintln!(
        "T17L2-DBG __current_route={:?} __route_params={:?}",
        dc.read_state("__current_route").ok(),
        dc.read_state("__route_params").ok()
    );

    // 路由参数面：push 落地后 params 必须已同步——形态 = 堆 ObjectData
    // 引用（状态字段持有约定；PLAN-712 T-17 第二层修复：裸 Value::Obj 会被
    // GET_FIELD 实例臂兜底吞成 0，router.param 恒 0 → book_id=0 → 0 entries）。
    let params = dc.read_state("__route_params").ok();
    match params {
        Some(auto_val::Value::VmRef(_)) => {}
        other => panic!(
            "push 后 __route_params 应为堆对象引用（VmRef），实际 {other:?}"
        ),
    }

    // detail 页 Init 已被 outlet 挂载 demand 驱动：got 应为 "7"。
    let got = dc.read_state("got").ok();
    assert_eq!(
        got,
        Some(auto_val::Value::Str("7".into())),
        "页 Init 的 router.param(\"id\") 应取到路由段；实际 {got:?}"
    );
}

/// PLAN-712 T-17 第三层探针：`for it in .items { button onclick: .Go(it.id) }`
/// 的**逐实例实参烘焙**——018 书架三卡同内容（fetch 恒 /api/books/1）的
/// 候选根因：循环变量字段实参若烘焙失败（全塌首项/0），点击哪张卡都开
/// 第一本。015 先例只覆盖裸循环变量实参（`k`），字段实参（`it.id`）在案
/// 面未见。本测检查构建产物里两只按钮各自烘焙的实参。
#[test]
#[cfg(feature = "ui-iced")]
fn loop_item_field_event_args_bake_per_instance() {
    let dir = tempfile::tempdir().expect("tempdir");
    let pages = dir.path().join("pages");
    std::fs::create_dir_all(&pages).expect("pages dir");
    std::fs::write(
        dir.path().join("app.at"),
        r#"
widget App {
    routes {
        "/" -> use home
    }
    view {
        outlet
    }
}
"#,
    )
    .expect("write app.at");
    std::fs::write(
        pages.join("home.at"),
        r#"
widget home {
    msg { Go(int) }

    model {
        var items = []
        var seeded str = "no"
    }

    view {
        col {
            for it in .items {
                button "open" {
                    onclick: .Go(it.id)
                }
            }
        }
    }

    on {
        .Init -> {
            .items.push({ id: 7, t: "a" })
            .items.push({ id: 8, t: "b" })
            .seeded = "yes"
        }
    }
}
"#,
    )
    .expect("write pages/home.at");

    let app_path = dir.path().join("app.at");
    let code = std::fs::read_to_string(&app_path).expect("read app.at");
    let mut dc = build_dynamic_component(&code, app_path.to_str()).expect("编译");
    dc.fire_init();
    let _ = dc.view_with_debug_gated(true);
    dc.drive_scheduler_to_quiescence(10_000);
    let _ = dc.view_with_debug_gated(true);
    assert_eq!(
        dc.read_state("seeded").ok(),
        Some(auto_val::Value::Str("yes".into())),
        "Init 种子必须已跑"
    );

    // 构建产物的 Debug 串里检查逐实例烘焙实参。
    let (view, _, _) = dc.view_with_debug_gated(true);
    let dump = format!("{view:?}");
    eprintln!("T17L3-DBG view dump（截 800）: {}", &dump[..dump.len().min(800)]);
    let has_go7 = dump.contains("Go") && dump.contains("Int(7)");
    let has_go8 = dump.contains("Go") && dump.contains("Int(8)");
    assert!(
        has_go7 && has_go8,
        "逐卡实参应分别为 7 与 8（全塌首项/0 = 烘焙断链）；dump 头：{}",
        &dump[..dump.len().min(600)]
    );
}

/// PLAN-712 T-17 第三层（实锤复现）：**重复导航不重跑页 Init**——018 实机
/// （独立 VM + 桌面双轨）：点第 2/3 张卡详情恒第一本；standalone 日志仅
/// 一次 `book_detail_Init`。机理：outlet 页 Init 身份 = key prop 或裸
/// widget 名（`child_init_identity`），与路由参数无关 → `/book/1` 与
/// `/book/2` 同代际 → `register_init_demand` AlreadyKnown 不重派 → 页面
/// 状态滞留首次装载。语义应为「导航 = 新装载」：路由路径进身份，参数
/// 变化 = 新代际 = Init 重跑（与书架回退重取数的既有行为一致）。
#[test]
#[cfg(feature = "ui-iced")]
fn page_init_reruns_when_route_param_changes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let app_path = write_fixture(&dir);
    let code = std::fs::read_to_string(&app_path).expect("read app.at");
    let mut dc = build_dynamic_component(&code, app_path.to_str()).expect("编译");

    // 首次装载：/book/7。
    dc.fire_init();
    let _ = dc.view_with_debug_gated(true);
    dc.drive_scheduler_to_quiescence(10_000);
    dc.on_with_input_for("home", "Go", None);
    let _ = dc.view_with_debug_gated(true);
    dc.drive_scheduler_to_quiescence(10_000);
    assert_eq!(
        dc.read_state("got").ok(),
        Some(auto_val::Value::Str("7".into())),
        "首次装载应取到 7"
    );

    // 二次导航：/book/8 —— fixture 的 home.Go 固定 push 7，这里直写路由
    // 通道（与 navigate 等价面：__current_route 写入 + sync + 重建）。
    dc.set_route("/book/8");
    let _ = dc.view_with_debug_gated(true);
    dc.drive_scheduler_to_quiescence(10_000);
    assert_eq!(
        dc.read_state("got").ok(),
        Some(auto_val::Value::Str("8".into())),
        "参数变化必须重跑页 Init（导航=新装载）；滞留 7 = 同代际不重派复现"
    );
}
