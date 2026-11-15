//! PLAN-981（IF-981-ORIGINAL）：AGE **2D 资源服务**的源侧 typed 声明。
//!
//! 与 `platform_io` 同一形态：源里出现的服务调用名 → 固定符号 + 参数/返回类型
//! + 要求的能力。声明住在 core（通用接面），**领域语义**来自 AGE 既有规范
//! （auto-ui `docs/specs/auto-graphics/node2d-basic-authoring.md` §1：
//! `Material2D`/`Mesh2D` 是四类 typed 端口之一；`material2d.color(r,g,b,a)`、
//! `mesh2d.quad(w,h)`/`mesh2d.circle(d)` 是作者写法的资源构造器）。
//!
//! 本模块**只声明**：不做 GPU、不建资源系统、不解析资源文件。它让共享
//! resolver 能对"调用 → 返回什么类型"给出确定结论，从而模块级 `let` 推断
//! 与 `tint: Material2D` 参数检查都有依据。
//!
//! 资源值的**语义身份**（颜色分量、尺寸）不在这里被压平成普通 record：这里给出
//! 的是名义端口类型（`VT::Rec` 上的 AGE 端口记录），领域 owner 仍按 AGE 规范
//! 解释其字段。

use super::resolve::VT;

/// AGE 2D 资源服务的能力键（与其它平台服务同风格；进入身份/准入）。
pub const CAP_AGE2D_RESOURCE: &str = "age2d.resource";
/// 3D 资源服务的能力键（与 2D 分开：3D 资源有独立的 GLB 身份规则）。
pub const CAP_AGE3D_RESOURCE: &str = "age3d.resource";

/// 一条 AGE 资源服务声明。
#[derive(Clone, Copy, Debug)]
pub struct AgeResourceOp {
    /// 源里的**限定**写法（点分，如 `material2d.color`）。
    pub callee: &'static str,
    /// 受检产物里的稳定符号（跨后端身份）。
    pub symbol: &'static str,
    pub params: &'static [VT],
    /// 返回的 AGE 端口类型名（`Material2D` / `Mesh2D`）。
    pub result_port: &'static str,
    pub capability: &'static str,
}

const P_4F32: &[VT] = &[VT::F32, VT::F32, VT::F32, VT::F32];
const P_2F32: &[VT] = &[VT::F32, VT::F32];
const P_F32: &[VT] = &[VT::F32];
const P_STR: &[VT] = &[VT::Str];

/// AGE 资源端口类型族（AGE 规范 §1 的 typed 端口）。
///
/// 这四个是共鸣名的**端口**类型：它们不是用户 record，也不是 Core 内建，
/// 而是 AGE 领域声明的 typed 端口。resolver 按本表把它们当作名义类型登记。
pub const AGE_PORT_TYPES: &[&str] = &["Material2D", "Mesh2D", "Model3D"];

/// 每个端口携带的字段（与构造器实参一一对应；f32 分量）。
///
/// 这些不是"把资源压平成普通 record"：字段就是 AGE 构造器本身的实参
/// （颜色四分量为 alpha 合成保留、网格为宽高/直径），资源身份仍由端口名承担。
pub const AGE_PORT_FIELDS: &[(&str, &[(&str, &str)])] = &[
    (
        "Material2D",
        &[("r", "f32"), ("g", "f32"), ("b", "f32"), ("a", "f32")],
    ),
    ("Mesh2D", &[("w", "f32"), ("h", "f32")]),
    // `Model3D` 的字段是**资源身份**（GLB 内容哈希 + 字节数），不是颜色分量：
    // 规范 `model3d.glb("path.glb")` 的身份 = 完整原始字节 SHA256 + byte_length。
    // 用 `Str` 承载哈希（十六进制）与 `i32` 承载长度，避免把身份伪造为数值。
    ("Model3D", &[("sha256", "Str"), ("byte_length", "i32")]),
];

pub const AGE_RESOURCE_OPS: &[AgeResourceOp] = &[
    AgeResourceOp {
        callee: "material2d.color",
        symbol: "age.material2d.color.v1",
        params: P_4F32,
        result_port: "Material2D",
        capability: CAP_AGE2D_RESOURCE,
    },
    AgeResourceOp {
        callee: "mesh2d.quad",
        symbol: "age.mesh2d.quad.v1",
        params: P_2F32,
        result_port: "Mesh2D",
        capability: CAP_AGE2D_RESOURCE,
    },
    AgeResourceOp {
        callee: "mesh2d.circle",
        symbol: "age.mesh2d.circle.v1",
        params: P_F32,
        result_port: "Mesh2D",
        capability: CAP_AGE2D_RESOURCE,
    },
    // 3D：静态平色 GLB。参数是**文件路径**（字符串），身份由内容决定。
    AgeResourceOp {
        callee: "model3d.glb",
        symbol: "age.model3d.glb.v1",
        params: P_STR,
        result_port: "Model3D",
        capability: CAP_AGE3D_RESOURCE,
    },
];

/// 按源里的限定写法查声明。
pub fn lookup(callee: &str) -> Option<&'static AgeResourceOp> {
    AGE_RESOURCE_OPS.iter().find(|op| op.callee == callee)
}

/// 按稳定符号查声明（发射器/消费者用）。
pub fn by_symbol(symbol: &str) -> Option<&'static AgeResourceOp> {
    AGE_RESOURCE_OPS.iter().find(|op| op.symbol == symbol)
}

/// 是否为 AGE 端口类型名（`Material2D` / `Mesh2D`）。
pub fn is_port_type(name: &str) -> bool {
    AGE_PORT_TYPES.contains(&name)
}

/// `shape2d kind:` 的冻结几何符号（AGE 规范 §1：`kind: circle|quad`）。
pub const AGE_GEOMETRY_KINDS: &[&str] = &["circle", "quad"];

/// `view-3d` 的**冻结叶种类**（AGE 规范：3D 叶是 `cylinder3d` / `model3d` /
/// `group3d`，**不是** `shape3d`——2D 的 kind 符号表与 3D 的叶种类是两套东西，
/// 不能互相套用）。
pub const AGE_3D_ELEMENT_KINDS: &[&str] = &["cylinder3d", "model3d", "group3d"];

/// 属性名的**值类别**（决定该怎么校验它，而不是猜一个值类型）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttrValueKind {
    /// 冻结几何符号（`kind: circle`）——按符号表校验，不是变量。
    GeometryKind,
    /// 具名资源引用（`material: resources.red_alpha`）——按导入/声明的事实校验。
    ResourceRef,
    /// 普通 Auto 值表达式（数/元组/bool）——由共享 verify 按值类型检查。
    Value,
}

/// 一个属性名属于哪一类。`None` = 该属性名不在 AGE 的 2D 作者面内（具名拒绝）。
///
/// 这份表就是 AGE 规范 §1 的 `shape2d` 属性集：`kind`/`position`/`size`/
/// `material`/`visible`，加上组件实例的 `x`/`y`/`diameter`/`tint`。
pub fn attr_value_kind(key: &str) -> Option<AttrValueKind> {
    match key {
        "kind" => Some(AttrValueKind::GeometryKind),
        // `model3d` 的 `model:` 与 2D 的 `mesh:` 都是**具名资源引用**。
        "material" | "tint" | "model" | "mesh" => Some(AttrValueKind::ResourceRef),
        // 2D 标量/元组值 + 3D 的静态角度（yaw/pitch）。
        "position" | "size" | "x" | "y" | "diameter" | "visible" | "yaw" | "pitch"
        | "scale" => Some(AttrValueKind::Value),
        _ => None,
    }
}

/// 该 3D 叶种类拼写是否合法（`cylinder3d`/`model3d`/`group3d`）。
pub fn is_3d_element(kind: &str) -> bool {
    AGE_3D_ELEMENT_KINDS.contains(&kind)
}

/// 该拼写是否是冻结几何符号。
pub fn is_geometry_kind(spelling: &str) -> bool {
    AGE_GEOMETRY_KINDS.contains(&spelling)
}

/// 某场景行的某个资源属性**必须**绑定哪一种端口。
///
/// 只验"是某个端口"不够：`shape2d` 的 `material` 收 `Material2D`，`model3d` 的
/// `model` 收 `Model3D`——把颜色资源绑到 `model:` 上必须被拒，反之亦然。
/// `None` = 该 (行种类, 属性) 组合不是资源绑定（或不约束端口）。
pub fn attr_port(element: &str, key: &str) -> Option<&'static str> {
    match (element, key) {
        // 2D 叶
        ("shape2d", "material") => Some("Material2D"),
        ("shape2d", "mesh") => Some("Mesh2D"),
        // 3D 叶
        ("model3d", "model") => Some("Model3D"),
        _ => None,
    }
}
