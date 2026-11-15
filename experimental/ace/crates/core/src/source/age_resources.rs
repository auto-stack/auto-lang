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

/// AGE 资源端口类型族（AGE 规范 §1 的 typed 端口）。
///
/// 这四个是共鸣名的**端口**类型：它们不是用户 record，也不是 Core 内建，
/// 而是 AGE 领域声明的 typed 端口。resolver 按本表把它们当作名义类型登记。
pub const AGE_PORT_TYPES: &[&str] = &["Material2D", "Mesh2D"];

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
];

/// 按源里的限定写法查声明。
pub fn lookup(callee: &str) -> Option<&'static AgeResourceOp> {
    AGE_RESOURCE_OPS.iter().find(|op| op.callee == callee)
}

/// 是否为 AGE 端口类型名（`Material2D` / `Mesh2D`）。
pub fn is_port_type(name: &str) -> bool {
    AGE_PORT_TYPES.contains(&name)
}
