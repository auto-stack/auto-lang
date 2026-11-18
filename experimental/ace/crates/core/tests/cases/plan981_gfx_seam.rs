//! PLAN-981 B-W1（IF-981-GFX-AV-SEAM，UI058 AV 接面）：UI058 所需 AGE 图形
//! 服务的**签名/受检投影**面——`query2d/input/tick/row/render2d/image2d/
//! frame2d` 六族经共同前门唯一 verify（真实类型/具名错误）；执行车道分类
//! （ValueCtor=AV 可执行；HostAlgorithm/HostIdentity=AV 观察车道具名拒、
//! 图形宿主车道消费）见 av 侧测试。差额表与 owner 边界见计划 §9。
//!
//! **f32 面（2026-11-18，D-981-AGE-F32-SURFACE 关闭）**：注解位（fn 参数/
//! 返回/`let`）已在 age-package 面开放（`ann-ty`/`ret-ty` 增 f32 臂）；f32
//! 同型四则/比较已在 text-record Atom 描述符（`expr.binary:*_f32`）与 shared
//! lowering 上登记；组件面经 f32 model 字段 + Tick 重放真实落值
//! （`av/tests/cases/plan981_f32_surface.rs`）。仍在面外：文本叶呈现 f32
//! （原子编码的 i32 面）、混宽隐式提升（保持具名拒）、`f64`/`double`。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn input(src: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: src.into(),
        provider: None,
    }
}

fn compile(
    src: &str,
) -> Result<
    ace_core::source::age_package::AgePackageDescription,
    Vec<ace_core::atom_text::Diagnostic>,
> {
    compile_age_package(
        &[input(src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .map(|out| out.description)
}

/// #4：`image2d.png`（宿主身份）+ `frame2d.at`（纯值构造）声明合法、类型
/// 定型（Image2D/Frame2D 端口）；类型错/元数错在执行前具名拒。
#[test]
fn p981_gfx_seam_frame_table_types_and_named_errors() {
    let ok = "node M {\n    model {\n        count int = 0\n    }\n    view {\n        text \"m\" {}\n    }\n}\n\npub fn frame_w(img Image2D) i32 {\n    let f Frame2D = frame2d.at(img, 1, 2, 32, 32)\n    return f.w\n}\n";
    let d = compile(ok).expect("the frame-table helper verifies through the shared front door");
    let _ = d;

    let arity = "node M {\n    model {\n        count int = 0\n    }\n    view {\n        text \"m\" {}\n    }\n}\n\npub fn bad(img Image2D) i32 {\n    let f Frame2D = frame2d.at(img, 1, 2, 32)\n    return f.w\n}\n";
    match compile(arity) {
        Ok(_) => panic!("a 4-argument frame2d.at must be rejected"),
        Err(ds) => {
            assert_eq!(ds[0].code, "source.age.gfx-arity", "{ds:?}");
            assert!(ds[0].message.contains("frame2d.at"), "{ds:?}");
        }
    }

    let tyerr = "node M {\n    model {\n        count int = 0\n    }\n    view {\n        text \"m\" {}\n    }\n}\n\npub fn bad(n i32) i32 {\n    let f Frame2D = frame2d.at(n, 1, 2, 32, 32)\n    return f.w\n}\n";
    match compile(tyerr) {
        Ok(_) => panic!("an i32 where Image2D is expected must be rejected"),
        Err(ds) => assert_eq!(ds[0].code, "source.age.gfx-type", "{ds:?}"),
    }
}

/// #1/#3：查询与输入服务**定型通过**（typed 结果端口/标量）；元数/类型错
/// 误具名拒——算法执行归图形宿主车道（av 侧车道拒测试）。
#[test]
fn p981_gfx_seam_query_and_input_typing() {
    let ok = "node P {\n    msg Msg {\n        Tick\n    }\n    on {\n        .Tick -> {\n            .count += 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"p\" {}\n    }\n}\n\npub fn probe(row i32) bool {\n    let q Cast2D = query2d.segment(row, true, 0.0, 0.0, 1.0, 0.0)\n    return q.hit\n}\n\npub fn held() bool {\n    return input.pressed(1)\n}\n\npub fn space() i32 {\n    return input.space()\n}\n";
    let d = compile(ok).expect("query/input service calls verify with typed results");
    let _ = d;

    let arity = "node P {\n    view {\n        text \"p\" {}\n    }\n}\n\npub fn bad(row i32) bool {\n    let q Cast2D = query2d.segment(row, true, 0.0, 0.0, 1.0)\n    return q.hit\n}\n";
    match compile(arity) {
        Ok(_) => panic!("a 5-argument segment must be rejected"),
        Err(ds) => assert_eq!(ds[0].code, "source.age.gfx-arity", "{ds:?}"),
    }

    let tyerr = "node P {\n    view {\n        text \"p\" {}\n    }\n}\n\npub fn bad(row i32, flag i32) bool {\n    let q Cast2D = query2d.segment(row, flag, 0.0, 0.0, 1.0, 0.0)\n    return q.hit\n}\n";
    match compile(tyerr) {
        Ok(_) => panic!("an i32 visible_only must be rejected"),
        Err(ds) => assert_eq!(ds[0].code, "source.age.gfx-type", "{ds:?}"),
    }

    // sweep 专属字段（origin_x）在 Cast2D 结果上读 → 记录域守卫具名拒。
    let wrong_kind = "node P {\n    view {\n        text \"p\" {}\n    }\n}\n\npub fn bad(row i32) bool {\n    let q Cast2D = query2d.segment(row, true, 0.0, 0.0, 1.0, 0.0)\n    return q.origin_x\n}\n";
    match compile(wrong_kind) {
        Ok(_) => panic!("a sweep-only field on a Cast2D result must be rejected"),
        Err(ds) => assert!(
            ds[0].message.contains("origin_x") || ds[0].message.contains("Cast2D"),
            "the wrong-kind getter is named: {ds:?}"
        ),
    }
}

/// #2/#4：行/精灵端口服务签名面定型 + 元数拒；未知服务名不静默。
/// 注：`render2d.spawn` 的 021 冻结拼写与共享文法保留词 `spawn` 冲突
/// （`.spawn` 不能作为方法名 lex）——登记为 UI058 T-02 联冻项（文法接收
/// 或服务改名二选一）；本测试以 despawn/frame 证族面。
#[test]
fn p981_gfx_seam_row_and_render_typing() {
    let ok = "node R {
    view {
        text \"r\" {}
    }
}

pub fn step(r i32) i32 {
    let placed i32 = row.place(r, 4.0, 6.0)
    return placed
}

pub fn despawn_leaf(token i32) i32 {
    return render2d.despawn(token)
}
";
    let d = compile(ok).expect("row/render2d calls verify with typed results");
    let _ = d;

    let arity = "node R {
    view {
        text \"r\" {}
    }
}

pub fn bad() i32 {
    return render2d.frame(1, \"img\", 0)
}
";
    match compile(arity) {
        Ok(_) => panic!("a 3-argument render2d.frame must be rejected"),
        Err(ds) => assert_eq!(ds[0].code, "source.age.gfx-arity", "{ds:?}"),
    }

    // 未知服务：query2d.nosuch —— 命名空间存在但无该 op → 未定义具名拒。
    let unknown = "node R {\n    view {\n        text \"r\" {}\n    }\n}\n\npub fn bad() i32 {\n    return query2d.nosuch()\n}\n";
    match compile(unknown) {
        Ok(_) => panic!("an unknown gfx service must be rejected"),
        Err(ds) => assert!(
            ds.iter().any(|d| d.message.contains("query2d")),
            "the unknown service is named (receiver resolution): {ds:?}"
        ),
    }
}

/// `render2d.spawn`（021 冻结拼写）：`spawn` 是全域保留 token，AGE 面此前连
/// 成员位都解析不到（`.spawn` 不成方法名）。本批只开**成员位**（专用臂、
/// token 表零新增、拼写不动）——正例必须命中真实服务声明（typed i32），
/// 元数错仍是 `source.age.gfx-arity` 具名拒（不是"任意方法"的泛化接受）。
#[test]
fn p981_gfx_seam_spawn_member_spelling_binds_the_service() {
    let ok = "node R {\n    view {\n        text \"r\" {}\n    }\n}\n\npub fn spawn_img() i32 {\n    return render2d.spawn(\"assets/x.png\", 4.0, 6.0)\n}\n";
    compile(ok).expect("the reserved `spawn` member spelling binds render2d.spawn");

    let arity = "node R {\n    view {\n        text \"r\" {}\n    }\n}\n\npub fn bad() i32 {\n    return render2d.spawn(\"assets/x.png\", 4.0)\n}\n";
    match compile(arity) {
        Ok(_) => panic!("a 2-argument render2d.spawn must be rejected"),
        Err(ds) => assert_eq!(ds[0].code, "source.age.gfx-arity", "{ds:?}"),
    }
}

/// query token 失效语义（差额表 #1b/#6b，021 fault 13 的语言侧缝）：同一
/// callback 内第二次 query2d.* 使先前 token 绑定失效——此后读旧 token
/// 具名拒；新鲜读（单查后即读）与重绑刷新照常合法。
///
/// **边界（如实）**：今日 age-package 文法的 handler 体只收 `.field = expr`
/// 赋值（age-view-stmt），`let`/查询调用在 handler 内不可表达，故本缝当前
/// 的可表达面是 pub fn；走查器遍历全部 Fn 体（含脱糖的 `__age_p_*` 隐藏
/// fn），文法扩展后同一走查自动覆盖 handler。
#[test]
fn p981_query_token_second_query_invalidates_the_first() {
    // 直线 stale：a 的 token 在 b 的查询后失效，读 a.hit 具名拒。
    let stale = "node P {\n    view {\n        text \"p\" {}\n    }\n}\n\npub fn two(r i32) bool {\n    let a Cast2D = query2d.segment(r, true, 0.0, 0.0, 1.0, 0.0)\n    let b Sweep2D = query2d.sweep(r, true, 1.0, 0.0)\n    return a.hit\n}\n";
    let ds = compile(stale).expect_err("reading the first token after the second query is stale");
    assert!(
        ds.iter().any(|d| d.code == "source.age.query-token-stale"
            && d.message.contains("fault 13")),
        "named staleness rejection: {ds:?}"
    );

    // 新鲜读：单查后即读，合法（编译通过）。
    let fresh = "node P {\n    view {\n        text \"p\" {}\n    }\n}\n\npub fn one(r i32) bool {\n    let a Cast2D = query2d.segment(r, true, 0.0, 0.0, 1.0, 0.0)\n    return a.hit\n}\n";
    compile(fresh).expect("a fresh token read compiles");

    // 重绑刷新：a 重绑为第二次查询的结果后读，合法。
    let rebound = "node P {\n    view {\n        text \"p\" {}\n    }\n}\n\npub fn rb(r i32) bool {\n    var a Cast2D = query2d.segment(r, true, 0.0, 0.0, 1.0, 0.0)\n    a = query2d.segment(r, false, 0.0, 0.0, 1.0, 0.0)\n    return a.hit\n}\n";
    compile(rebound).expect("rebinding refreshes the token generation");
}

/// 循环保守语义：循环体内的查询使外层旧 token 失效（二次迭代语义）——
/// 循环后读外层 token 同样具名拒。
#[test]
fn p981_query_token_loop_query_invalidates_outer_token() {
    let src = "node P {\n    view {\n        text \"p\" {}\n    }\n}\n\npub fn lp(r i32) bool {\n    let a Cast2D = query2d.segment(r, true, 0.0, 0.0, 1.0, 0.0)\n    loop {\n        let b Cast2D = query2d.segment(r, true, 0.0, 0.0, 1.0, 0.0)\n        return b.hit\n    }\n    return a.hit\n}\n";
    let ds = compile(src).expect_err("the loop's query invalidates the outer token");
    assert!(
        ds.iter().any(|d| d.code == "source.age.query-token-stale"),
        "named staleness rejection: {ds:?}"
    );
}

