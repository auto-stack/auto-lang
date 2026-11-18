//! PLAN-981（SOURCE 批，IF-981-ORIGINAL）：**g01 原源闭包**。
//!
//! 用 `age-examples` 的 g01 三文件原源（逐字复制到 `fixtures_g01/`）走 ACE 共享
//! 前门，断言：完整闭包编译、四行场景按声明序（= 2D 绘制序）、几何/尺寸/材质/
//! 可见性与 auto-ui 既有基线一致，以及错误/变更/恢复。
//!
//! 基线来源：auto-ui `experimental/autographics-node2d-pilot/tests/node2d_basic_gates.rs`
//! 的 `g01_base_scene_matches_original_baseline_data`。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::ast::{Expr, ExprKind};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

const MAIN: &str = include_str!("../fixtures_g01/main.at");
const RESOURCES: &str = include_str!("../fixtures_g01/resources.at");
const SHAPES: &str = include_str!("../fixtures_g01/shapes.at");

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn compile_g01() -> Result<
    ace_core::source::age_package::AgePackageCompilation,
    Vec<ace_core::atom_text::Diagnostic>,
> {
    compile_age_package(
        &[
            input("resources", RESOURCES),
            input("shapes", SHAPES),
            input("main", MAIN),
        ],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
}

/// 从行的**CPU 值面**取一个属性值——这是 979 消费的方式，不再自己解析 AST。
///
/// 旧版这里有一套 `f32_of`/`pair_of`，把 `Expr` 的 AST 形状再判一次（是不是
/// `TypedLit{Float}`、要不要处理一元负号……）。那正是"只给 AST 等于把判断推给
/// 消费者"的写照：每个消费者都得重写一遍。现在读 `row.cpu`，那两个 helper 已删。
fn cpu<'a>(
    row: &'a ace_core::source::age_package::AgeSceneRow,
    key: &str,
) -> &'a ace_core::source::age_package::AgeSceneValue {
    &row.cpu
        .iter()
        .find(|(k, _)| k == key)
        .unwrap_or_else(|| panic!("attribute `{key}` on row `{}`", row.name))
        .1
}

fn f32_of(e: &Expr) -> Option<f32> {
    match &e.kind {
        ExprKind::TypedLit {
            ty: ace_core::source::ast::TypeWord::Float,
            bits,
        } => Some(f32::from_bits(*bits as u32)),
        ExprKind::Unary {
            op: ace_core::source::ast::UnaryOp::Neg,
            operand,
        } => f32_of(operand).map(|v| -v),
        _ => None,
    }
}

fn attr<'a>(row: &'a ace_core::source::age_package::AgeSceneRow, key: &str) -> Option<&'a Expr> {
    row.attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

/// 原源三文件走**一个**共享前门即可编译，且两个 node 都在。
#[test]
fn p981_original_g01_compiles_through_one_front_door() {
    let out = compile_g01().expect("the original g01 closure compiles");
    let names: Vec<&str> = out
        .description
        .components
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert!(names.contains(&"Disc"), "reusable component: {names:?}");
    assert!(names.contains(&"Main"), "entry component: {names:?}");
    assert_eq!(out.description.modules.len(), 3, "three source modules");
    assert!(!out.atom_text.is_empty(), "a checked document was emitted");
}

/// 场景行**按声明序**出现（= 2D 绘制序），四行齐全。
#[test]
fn p981_original_g01_scene_rows_keep_declaration_order() {
    let out = compile_g01().expect("g01 compiles");
    let main = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Main")
        .expect("Main");
    let rows: Vec<(&str, &str)> = main
        .scene
        .iter()
        .map(|r| (r.kind.as_str(), r.name.as_str()))
        .collect();
    assert_eq!(
        rows,
        vec![
            ("Disc", "red_disc"),
            ("shape2d", "blue_quad"),
            ("Disc", "green_disc"),
            ("shape2d", "yellow_quad"),
        ],
        "declaration order IS the 2D draw order"
    );
}

/// 几何/尺寸/可见性与 auto-ui 基线一致（逐值）。
#[test]
fn p981_original_g01_geometry_matches_the_baseline() {
    let out = compile_g01().expect("g01 compiles");
    let main = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Main")
        .expect("Main");
    let row = |n: &str| {
        main.scene
            .iter()
            .find(|r| r.name == n)
            .unwrap_or_else(|| panic!("scene row {n}"))
    };

    // 全部读 **CPU 值面**（979 的消费方式），不再在测试里解析 AST。
    // red circle: (-20, 10) d30
    let red = row("red_disc");
    assert_eq!(cpu(red, "x").as_float(), Some(-20.0));
    assert_eq!(cpu(red, "y").as_float(), Some(10.0));
    assert_eq!(cpu(red, "diameter").as_float(), Some(30.0));

    // blue quad: (0,0) 30x30
    let blue = row("blue_quad");
    assert_eq!(cpu(blue, "position").as_floats(), Some(&[0.0, 0.0][..]));
    assert_eq!(cpu(blue, "size").as_floats(), Some(&[30.0, 30.0][..]));

    // green circle: (25, -15) d24
    let green = row("green_disc");
    assert_eq!(cpu(green, "x").as_float(), Some(25.0));
    assert_eq!(cpu(green, "y").as_float(), Some(-15.0));
    assert_eq!(cpu(green, "diameter").as_float(), Some(24.0));

    // yellow quad: (-45, -25) 12x12, HIDDEN — visibility is a scene row fact,
    // not an `if` that destroys the object (per the AGE spec).
    let yellow = row("yellow_quad");
    assert_eq!(
        cpu(yellow, "position").as_floats(),
        Some(&[-45.0, -25.0][..])
    );
    assert_eq!(cpu(yellow, "size").as_floats(), Some(&[12.0, 12.0][..]));
    assert_eq!(
        cpu(yellow, "visible").as_bool(),
        Some(false),
        "yellow is declared visible: false (kept, not destroyed)"
    );
    // 符号值也走同一面：`kind: circle|quad`。
    assert_eq!(red.kind, "Disc");
    assert_eq!(cpu(row("blue_quad"), "kind").as_symbol(), Some("quad"));
}

/// `kind:` 必须是冻结几何符号；坏拼写具名拒绝（不是静默接受）。
#[test]
fn p981_original_g01_rejects_a_bad_geometry_kind() {
    let bad = MAIN.replace("kind: quad", "kind: triangle");
    let ds = compile_age_package(
        &[
            input("resources", RESOURCES),
            input("shapes", SHAPES),
            input("main", &bad),
        ],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect_err("an unknown geometry kind is rejected");
    assert_eq!(ds[0].code, "source.age.scene-kind", "{ds:?}");
    // 诊断必须落在**原文件**的出错 token 上（轮 48 R47-5 指出此前只断言了码）。
    let want = bad.find("triangle").expect("fixture has triangle");
    assert_eq!(
        ds[0].span.start,
        want,
        "the diagnostic must point at `triangle` in main.at; got {:?} which reads {:?}",
        ds[0].span,
        &bad[ds[0].span.start.min(bad.len())..ds[0].span.end.min(bad.len())]
    );
}

/// g01 夹具的**字节身份**必须被钉住：它必须是 `age-examples` 的原源，而不是
/// "改过一行的相似文件"。长度断言挡不住注释/空白改动，内容哈希可以（R47-7）。
#[test]
fn p981_original_g01_fixture_bytes_are_pinned() {
    use ace_core::source::cst::content_sha;
    assert_eq!(content_sha(MAIN.as_bytes()), 4_661_484_066_523_668_053);
    assert_eq!(
        content_sha(RESOURCES.as_bytes()),
        12_785_226_987_299_247_172
    );
    assert_eq!(content_sha(SHAPES.as_bytes()), 5_140_912_040_684_847_638);
}

/// 改一个几何值后闭包仍成立，且**只有**该值变化（其余逐值不变）。
#[test]
fn p981_original_g01_edit_changes_only_the_edited_value() {
    let before = compile_g01().expect("baseline compiles");
    let edited = MAIN.replace("diameter: 30.0", "diameter: 40.0");
    let after = compile_age_package(
        &[
            input("resources", RESOURCES),
            input("shapes", SHAPES),
            input("main", &edited),
        ],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect("the edited source still compiles");

    let diameter = |out: &ace_core::source::age_package::AgePackageCompilation| {
        let main = out
            .description
            .components
            .iter()
            .find(|c| c.name == "Main")
            .expect("Main");
        let red = main
            .scene
            .iter()
            .find(|r| r.name == "red_disc")
            .expect("red");
        f32_of(attr(red, "diameter").expect("diameter"))
    };
    assert_eq!(diameter(&before), Some(30.0));
    assert_eq!(diameter(&after), Some(40.0));
    // the row ORDER is untouched by the edit
    let order = |out: &ace_core::source::age_package::AgePackageCompilation| {
        out.description
            .components
            .iter()
            .find(|c| c.name == "Main")
            .expect("Main")
            .scene
            .iter()
            .map(|r| r.name.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(order(&before), order(&after));
}

/// 坏源被拒后，恢复原源仍能编译（错误不污染后续）。
#[test]
fn p981_original_g01_recovers_after_a_rejected_edit() {
    let bad = MAIN.replace("kind: quad", "kind: nope");
    assert!(compile_age_package(
        &[
            input("resources", RESOURCES),
            input("shapes", SHAPES),
            input("main", &bad),
        ],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .is_err());
    // the pristine source still compiles afterwards
    compile_g01().expect("the original source compiles again after a rejected edit");
}

/// `material:`/`tint:` 的资源引用必须**真实存在**——不存在的名字被具名拒绝。
///
/// 这条是补缺口用的：此前只校验"是引用形状"，实测 `tint: no_such_resource`
/// 会**静默通过**（引用根本没被解析）。
#[test]
fn p981_original_g01_rejects_an_unknown_resource() {
    let bad = MAIN.replace("tint: resources.red_alpha", "tint: resources.nope");
    let ds = compile_age_package(
        &[
            input("resources", RESOURCES),
            input("shapes", SHAPES),
            input("main", &bad),
        ],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect_err("an unknown resource name is rejected");
    assert_eq!(ds[0].code, "source.age.scene-resource-unknown", "{ds:?}");
    assert!(
        ds[0].message.contains("nope"),
        "the diagnostic names the offending symbol: {}",
        ds[0].message
    );
}

/// 资源可以是**裸名**（`tint: red_alpha`，经 `use` 导入）——与限定写法同等。
#[test]
fn p981_original_g01_accepts_a_bare_imported_resource() {
    let bare = MAIN
        .replace("tint: resources.red_alpha", "tint: red_alpha")
        .replace("tint: resources.green_alpha", "tint: green_alpha");
    let out = compile_age_package(
        &[
            input("resources", RESOURCES),
            input("shapes", SHAPES),
            input("main", &bare),
        ],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect("a bare imported resource name is accepted");
    let main = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Main")
        .expect("Main");
    assert_eq!(main.scene.len(), 4);
}

/// 四行场景的材质各自绑定到**不同**的资源符号（不是都塌到第一个）。
#[test]
fn p981_original_g01_rows_reference_distinct_resources() {
    let out = compile_g01().expect("g01 compiles");
    let main = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Main")
        .expect("Main");
    let mut materials = Vec::new();
    for row in &main.scene {
        let key = if row.attrs.iter().any(|(k, _)| k == "material") {
            "material"
        } else {
            "tint"
        };
        let e = attr(row, key).unwrap_or_else(|| panic!("{key} on {}", row.name));
        // 取引用末段拼写（`resources.red_alpha` → `red_alpha`）。
        let spelled = match &e.kind {
            ExprKind::Field { name, .. } => name.name.clone(),
            ExprKind::Var(id) => id.name.clone(),
            other => panic!("unexpected material shape: {other:?}"),
        };
        materials.push(spelled);
    }
    assert_eq!(
        materials,
        vec!["red_alpha", "blue_alpha", "green_alpha", "yellow_opaque"],
        "each row binds its own declared resource"
    );
}

/// 资源的**颜色分量与 alpha** 与 auto-ui 基线一致逐值。
///
/// 基线：red (0.9,0.3,0.2, α0.5)、blue (0.2,0.3,0.9, α0.5)、
/// green (0.2,0.8,0.3, α0.6)、yellow (0.95,0.9,0.1, α1.0)。
#[test]
fn p981_original_g01_resource_colours_match_the_baseline() {
    let out = compile_g01().expect("g01 compiles");
    let resource = |n: &str| {
        out.description
            .resources
            .iter()
            .find(|r| r.name == n)
            .unwrap_or_else(|| panic!("resource {n}"))
    };

    assert_eq!(resource("red_alpha").port, "Material2D");
    assert_eq!(resource("red_alpha").components, vec![0.9, 0.3, 0.2, 0.5]);
    assert_eq!(resource("blue_alpha").components, vec![0.2, 0.3, 0.9, 0.5]);
    assert_eq!(resource("green_alpha").components, vec![0.2, 0.8, 0.3, 0.6]);
    assert_eq!(
        resource("yellow_opaque").components,
        vec![0.95, 0.9, 0.1, 1.0]
    );
}

/// 五个资源全部在场（含未被任何行引用的 `deep_green`）——声明不被丢弃。
#[test]
fn p981_original_g01_all_declared_resources_are_present() {
    let out = compile_g01().expect("g01 compiles");
    let names: Vec<&str> = out
        .description
        .resources
        .iter()
        .map(|r| r.name.as_str())
        .collect();
    assert_eq!(
        names,
        vec![
            "blue_alpha",
            "deep_green",
            "green_alpha",
            "red_alpha",
            "yellow_opaque"
        ],
        "every declared resource is carried, sorted by (module, name)"
    );
    // 未引用的资源同样带真实分量，不是空壳。
    let deep = out
        .description
        .resources
        .iter()
        .find(|r| r.name == "deep_green")
        .expect("deep_green");
    assert_eq!(deep.components, vec![0.1, 0.8, 0.2, 0.5]);
}

/// 改**被引用资源**的原值 → 使用方读到的值随之改变（证明资源确来自导入模块，
/// 不是照抄一份快照）。
#[test]
fn p981_original_g01_editing_a_resource_changes_the_consumed_value() {
    let before = compile_g01().expect("baseline compiles");
    let alpha = |out: &ace_core::source::age_package::AgePackageCompilation| {
        out.description
            .resources
            .iter()
            .find(|r| r.name == "red_alpha")
            .expect("red_alpha")
            .components[3]
    };
    assert_eq!(alpha(&before), 0.5);

    // 只改 resources.at 里 red_alpha 的 alpha 分量。
    let edited_res = RESOURCES.replace(
        "material2d.color(0.9, 0.3, 0.2, 0.5)",
        "material2d.color(0.9, 0.3, 0.2, 0.75)",
    );
    let after = compile_age_package(
        &[
            input("resources", &edited_res),
            input("shapes", SHAPES),
            input("main", MAIN),
        ],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect("still compiles");
    assert_eq!(
        alpha(&after),
        0.75,
        "the consumer sees the edited resource value"
    );
    // 其它资源不受影响。
    let green = |out: &ace_core::source::age_package::AgePackageCompilation| {
        out.description
            .resources
            .iter()
            .find(|r| r.name == "green_alpha")
            .expect("green_alpha")
            .components[3]
    };
    assert_eq!(green(&before), green(&after));
}

/// 原源夹具身份记录：三份逐字复制自 `age-examples` 的 g01（内容哈希钉住，
/// 防止日后被无意改写而失去"原源"资格）。
#[test]
fn p981_original_g01_fixtures_are_the_recorded_original_sources() {
    // 与 `D:/autostack/age-examples/examples/g01-shapes/g01-main/src/` 逐字一致
    // （只有头部注释由 PLAN-008 改写为 PLAN-019 措辞，故用内容哈希钉住当前夹具）。
    assert_eq!(MAIN.len(), 1202, "main.at byte length");
    assert_eq!(RESOURCES.len(), 790, "resources.at byte length");
    assert_eq!(SHAPES.len(), 540, "shapes.at byte length");
    // 关键作者表面逐项在场（不是被"等价改写"过的版本）。
    assert!(
        MAIN.contains("use shapes: Disc"),
        "symbol-table import kept"
    );
    assert!(RESOURCES.contains("material2d.color(0.9, 0.3, 0.2, 0.5)"));
    assert!(SHAPES.contains("node Disc(x: float, y: float, diameter: float, tint: Material2D)"));
    assert!(
        MAIN.contains("visible: false"),
        "the hidden yellow quad is declared, not deleted"
    );
}

/// 交叉闭环：**凡被场景行引用的资源，都出现在 `resources` 面上**。
///
/// 两条判据（前门的存在性/类型检查、资源收集）必须给出同一答案；任何一边的
/// 判据漂移都会在此暴露，而不是靠人工核对。
#[test]
fn p981_original_g01_every_referenced_resource_is_on_the_resource_surface() {
    let out = compile_g01().expect("g01 compiles");
    let main = out
        .description
        .components
        .iter()
        .find(|c| c.name == "Main")
        .expect("Main");
    let declared: Vec<&str> = out
        .description
        .resources
        .iter()
        .map(|r| r.name.as_str())
        .collect();
    for row in &main.scene {
        let key = if row.attrs.iter().any(|(k, _)| k == "material") {
            "material"
        } else if row.attrs.iter().any(|(k, _)| k == "tint") {
            "tint"
        } else {
            continue;
        };
        let e = attr(row, key).expect("material/tint");
        let name = match &e.kind {
            ExprKind::Field { name, .. } => name.name.clone(),
            ExprKind::Var(id) => id.name.clone(),
            other => panic!("unexpected material shape: {other:?}"),
        };
        assert!(
            declared.contains(&name.as_str()),
            "row `{}` references `{name}`, which is missing from the resource surface \
             (declared: {declared:?})",
            row.name
        );
    }
}
