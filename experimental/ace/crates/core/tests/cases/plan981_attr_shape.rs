//! PLAN-981（SOURCE 批）：**场景属性的形状纪律**。
//!
//! 三个此前**静默接受**的构造，实测都通过了编译：
//!
//! 1. 同一行内**重复属性键**（`kind: quad` + `kind: circle`）——后值覆盖前值、零诊断；
//! 2. **属性不适用于该元素**（`yaw:` 写在 `shape2d` 上、`size:` 写在 `cylinder3d` 上）；
//! 3. 非 bool 的 `visible:`（**未修**，见下）。
//!
//! 本文件同时守住一条**反向**不变量：组件实例的属性名由作者自定义，**不受**白名单
//! 约束。我第一版把实例也套进 `(element, attribute)` 白名单，结果
//! `Badge(label: int)` 的 `label:` 被拒——那是真实的过度收紧，已由
//! `p981_component_instance_accepts_author_defined_parameter_names` 钉住。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn compile(src: &str) -> Result<
    ace_core::source::age_package::AgePackageCompilation,
    Vec<ace_core::atom_text::Diagnostic>,
> {
    compile_age_package(
        &[input("main", src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
}

/// 同一行重复写同一个属性名 → 具名拒绝（不是后值静默胜出）。
#[test]
fn p981_duplicate_attribute_key_is_rejected() {
    let ds = compile(
        "node M {\n    view-2d {\n        shape2d q {\n            kind: quad\n            kind: circle\n        }\n    }\n}\n",
    )
    .expect_err("a duplicated attribute key is rejected");
    assert_eq!(ds[0].code, "source.age.scene-attr-duplicate", "{ds:?}");
    assert!(
        ds[0].message.contains("twice"),
        "the diagnostic must say it was written twice: {}",
        ds[0].message
    );
}

/// 3D 专属属性写到 2D 叶上 → 具名拒绝。
#[test]
fn p981_3d_attribute_on_a_2d_leaf_is_rejected() {
    let ds = compile(
        "node M {\n    view-2d {\n        shape2d q {\n            kind: quad\n            yaw: 1.0\n        }\n    }\n}\n",
    )
    .expect_err("`yaw` does not apply to shape2d");
    assert_eq!(ds[0].code, "source.age.scene-attr", "{ds:?}");
    assert!(ds[0].message.contains("does not apply"), "{}", ds[0].message);
}

/// 2D 专属属性写到 3D 叶上 → 具名拒绝。
#[test]
fn p981_2d_attribute_on_a_3d_leaf_is_rejected() {
    let ds = compile(
        "node M {\n    view-3d {\n        cylinder3d c {\n            position: (0.0, 0.0, 0.0)\n            size: (1.0, 1.0)\n        }\n    }\n}\n",
    )
    .expect_err("`size` does not apply to cylinder3d");
    assert_eq!(ds[0].code, "source.age.scene-attr", "{ds:?}");
}

/// **组件实例的自定义参数名不受白名单限制**——对过度收紧的反控。
///
/// 我第一版 `attr_applies_to` 用 `(element, attribute)` 白名单并把组件实例也套进去，
/// 结果 `Badge(label: int)` 的 `label:` 被拒（"`label` does not apply to `Badge`"）。
/// 实例的元素名是作者自定义的 node 名、属性名由该 node 自己声明，**无法穷举**；
/// 用白名单管它必然误伤。
#[test]
fn p981_component_instance_accepts_author_defined_parameter_names() {
    compile(
        "node Badge(label: int) {\n    view { text \"b\" {} }\n}\n\
node M {\n    view-2d {\n        Badge inst {\n            label: 3\n        }\n    }\n}\n",
    )
    .expect("an author-defined parameter name on a component instance is legal");
}

/// 但 `shape2d` 与 3D 元素**仍受**白名单约束（放宽不能变成全放开）。
#[test]
fn p981_leaf_and_3d_elements_are_still_whitelisted() {
    let ds = compile(
        "node M {\n    view-2d {\n        shape2d q {\n            kind: quad\n            bogus: 1.0\n        }\n    }\n}\n",
    )
    .expect_err("an unknown attribute on shape2d is still rejected");
    assert_eq!(ds[0].code, "source.age.scene-attr", "{ds:?}");

    let ds2 = compile(
        "node M {\n    view-3d {\n        cylinder3d c {\n            position: (0.0, 0.0, 0.0)\n            bogus: 1.0\n        }\n    }\n}\n",
    )
    .expect_err("an unknown attribute on a 3D element is still rejected");
    assert_eq!(ds2[0].code, "source.age.scene-attr", "{ds2:?}");
}

/// 正确的元素/属性搭配仍然合法（收紧不能破坏正常源）。
#[test]
fn p981_valid_element_attribute_pairs_still_compile() {
    compile(
        "node M {\n    view-3d {\n        cylinder3d c {\n            position: (0.0, 0.0, 0.0)\n            yaw: 1.5\n        }\n    }\n}\n",
    )
    .expect("yaw is legal on cylinder3d");
    compile(
        "node M {\n    view-2d {\n        shape2d q {\n            kind: circle\n            position: (1.0, 2.0)\n            size: (3.0, 3.0)\n            visible: false\n        }\n    }\n}\n",
    )
    .expect("the normal shape2d form is legal");
    compile(
        "let red = material2d.color(0.9, 0.3, 0.2, 0.5)\n\
node Disc(tint: Material2D) {\n    view-2d {\n        shape2d d {\n            kind: circle\n            material: tint\n        }\n    }\n}\n\
node M {\n    view-2d {\n        Disc inst {\n            tint: red\n        }\n    }\n}\n",
    )
    .expect("a component instance bound to a real resource is legal");
}

/// 精灵叶属性面（差额表 #4b，PLAN-981）：`image:`/`frame:` 是**具名资源引用**
/// （Image2D / Frame2D 端口），`flip:` 是 bool 值——三键在 `shape2d` 精灵行上
/// 合法，且 `image:`/`frame:` 落进**资源引用事实**面（消费者沿同一通道复核）。
#[test]
fn p981_sprite_leaf_attrs_bind_and_reference() {
    let out = compile(
        "let img = image2d.png(\"assets/x.png\")\n\
let fr = frame2d.at(img, 0, 0, 16, 16)\n\
node M {\n    view-2d {\n        shape2d s {\n            kind: quad\n            image: img\n            frame: fr\n            flip: true\n        }\n    }\n}\n",
    )
    .expect("the sprite leaf attribute face compiles");
    let comp = out
        .description
        .components
        .iter()
        .find(|c| c.name == "M")
        .expect("component M");
    // 引用事实：image→Image2D 资源、frame→Frame2D 资源，原拼写保留。
    let img_ref = comp
        .resource_refs
        .iter()
        .find(|r| r.row == "s" && r.attr == "image")
        .expect("the image: binding becomes a resource reference fact");
    assert!(
        matches!(
            &img_ref.target,
            ace_core::source::age_package::AgeResourceTarget::Resource { port, .. }
                if port == "Image2D"
        ),
        "image: resolves to an Image2D resource: {:?}",
        img_ref.target
    );
    let frame_ref = comp
        .resource_refs
        .iter()
        .find(|r| r.row == "s" && r.attr == "frame")
        .expect("the frame: binding becomes a resource reference fact");
    assert!(
        matches!(
            &frame_ref.target,
            ace_core::source::age_package::AgeResourceTarget::Resource { port, .. }
                if port == "Frame2D"
        ),
        "frame: resolves to a Frame2D resource: {:?}",
        frame_ref.target
    );
    assert_eq!(img_ref.spelling, "img");
    assert_eq!(frame_ref.spelling, "fr");
}

/// `image:` 绑到 Material2D 资源 → 具名拒（端口不匹配，不静默换轨）。
#[test]
fn p981_sprite_image_attr_rejects_wrong_port() {
    let ds = compile(
        "let red = material2d.color(0.9, 0.3, 0.2, 0.5)\n\
node M {\n    view-2d {\n        shape2d s {\n            kind: quad\n            image: red\n        }\n    }\n}\n",
    )
    .expect_err("image: must bind an Image2D, not a Material2D");
    assert!(
        ds.iter()
            .any(|d| d.code == "source.age.scene-resource-type"
                || d.code == "source.age.scene-resource"),
        "named port rejection: {ds:?}"
    );
}

/// `frame:` 绑 Image2D 资源 → 同样具名拒（Frame2D ≠ Image2D）。
#[test]
fn p981_sprite_frame_attr_rejects_wrong_port() {
    let ds = compile(
        "let img = image2d.png(\"assets/x.png\")\n\
node M {\n    view-2d {\n        shape2d s {\n            kind: quad\n            frame: img\n        }\n    }\n}\n",
    )
    .expect_err("frame: must bind a Frame2D, not an Image2D");
    assert!(
        ds.iter()
            .any(|d| d.code == "source.age.scene-resource-type"
                || d.code == "source.age.scene-resource"),
        "named port rejection: {ds:?}"
    );
}

/// 精灵键不进 3D 叶：`image:` 写在 `cylinder3d` 上仍按"不适用"具名拒。
#[test]
fn p981_sprite_keys_do_not_leak_into_3d_leaves() {
    let ds = compile(
        "let img = image2d.png(\"assets/x.png\")\n\
node M {\n    view-3d {\n        cylinder3d c {\n            position: (0.0, 0.0, 0.0)\n            image: img\n        }\n    }\n}\n",
    )
    .expect_err("sprite keys are 2D-only");
    assert_eq!(ds[0].code, "source.age.scene-attr", "{ds:?}");
}
