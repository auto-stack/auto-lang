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

/// 从属性值表达式里取一个 f32 常量（字面量或一元负号）。
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

/// 取元组的两个 f32 分量。
fn pair_of(e: &Expr) -> Option<(f32, f32)> {
    match &e.kind {
        ExprKind::TupleLit(items) if items.len() == 2 => {
            Some((f32_of(&items[0])?, f32_of(&items[1])?))
        }
        _ => None,
    }
}

fn attr<'a>(
    row: &'a ace_core::source::age_package::AgeSceneRow,
    key: &str,
) -> Option<&'a Expr> {
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

    // red circle: (-20, 10) d30
    let red = row("red_disc");
    assert_eq!(f32_of(attr(red, "x").expect("x")), Some(-20.0));
    assert_eq!(f32_of(attr(red, "y").expect("y")), Some(10.0));
    assert_eq!(f32_of(attr(red, "diameter").expect("diameter")), Some(30.0));

    // blue quad: (0,0) 30x30
    let blue = row("blue_quad");
    assert_eq!(pair_of(attr(blue, "position").expect("position")), Some((0.0, 0.0)));
    assert_eq!(pair_of(attr(blue, "size").expect("size")), Some((30.0, 30.0)));

    // green circle: (25, -15) d24
    let green = row("green_disc");
    assert_eq!(f32_of(attr(green, "x").expect("x")), Some(25.0));
    assert_eq!(f32_of(attr(green, "y").expect("y")), Some(-15.0));
    assert_eq!(f32_of(attr(green, "diameter").expect("diameter")), Some(24.0));

    // yellow quad: (-45, -25) 12x12, HIDDEN — visibility is a scene row fact,
    // not an `if` that destroys the object (per the AGE spec).
    let yellow = row("yellow_quad");
    assert_eq!(
        pair_of(attr(yellow, "position").expect("position")),
        Some((-45.0, -25.0))
    );
    assert_eq!(pair_of(attr(yellow, "size").expect("size")), Some((12.0, 12.0)));
    assert!(
        matches!(
            attr(yellow, "visible").map(|e| &e.kind),
            Some(ExprKind::BoolLit(false))
        ),
        "yellow is declared visible: false (kept, not destroyed)"
    );
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
        let red = main.scene.iter().find(|r| r.name == "red_disc").expect("red");
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

