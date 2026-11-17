//! PLAN-981 B-W1（IF-981-GFX-AV-SEAM，UI058 AV 接面）：UI058 所需 AGE 图形
//! 服务的**签名/受检投影**面——`query2d/input/tick/row/render2d/image2d/
//! frame2d` 六族经共同前门唯一 verify（真实类型/具名错误）；执行车道分类
//! （ValueCtor=AV 可执行；HostAlgorithm/HostIdentity=AV 观察车道具名拒、
//! 图形宿主车道消费）见 av 侧测试。差额表与 owner 边界见计划 §9。
//!
//! **具名边界（UI058 T-02 联冻项）**：f32 **注解位**（top-level fn 参数/
//! 返回/let 注解）不在 age-package 基础文法词表（grammar 冻结面）；f32
//! **二元运算/比较**不在 text-record atom 面（BinOp 无 f32 cases）。值面
//! （字面量/服务结果/端口字段读）已定型可流。
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
    let ok = "node P {\n    msg {\n        Tick\n    }\n    on {\n        .Tick -> {\n            .count += 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"p\" {}\n    }\n}\n\npub fn probe(row i32) bool {\n    let q Cast2D = query2d.segment(row, true, 0.0, 0.0, 1.0, 0.0)\n    return q.hit\n}\n\npub fn held() bool {\n    return input.pressed(1)\n}\n\npub fn space() i32 {\n    return input.space()\n}\n";
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
        Ok(_) => panic!("a 2-argument spawn must be rejected"),
        Err(ds) => assert_eq!(ds[0].code, "source.age.gfx-arity", "{ds:?}"),
    }

    // 未知服务：query2d.nosuch —— 命名空间存在但无该 op → 未定义具名拒。
    let unknown = "node R {\n    view {\n        text \"r\" {}\n    }\n}\n\npub fn bad() i32 {\n    return query2d.nosuch()\n}\n";
    match compile(unknown) {
        Ok(_) => panic!("an unknown gfx service must be rejected"),
        Err(ds) => assert!(!ds.is_empty(), "named rejection, not silent: {ds:?}"),
    }
}
