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
pub const AGE_PORT_TYPES: &[&str] = &[
    "Material2D",
    "Mesh2D",
    "Model3D",
    // PLAN-981（IF-981-GFX-AV-SEAM）：UI058 查询结果/精灵资源端口（声明序
    // 保证被引端口先注册——Frame2D.image 引 Image2D）。
    "Cast2D",
    "Sweep2D",
    "Image2D",
    "Frame2D",
];

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
    // PLAN-981 B-W1（IF-981-GFX-AV-SEAM，UI058 AV 接面）：查询结果与精灵
    // 资源的 typed 承载。Cast2D/Sweep2D 按 021 getter 面（axis 参数的
    // point/normal 在值面展开为分量字段——最终拼写由 UI058 T-02 联冻）；
    // Image2D 身份=内容 SHA256+byte_length+IHDR（022 语义）；Frame2D 引用
    // Image2D 端口 + rect。
    (
        "Cast2D",
        &[
            ("hit", "bool"),
            ("node", "i32"),
            ("t", "f32"),
            ("point_x", "f32"),
            ("point_y", "f32"),
            ("normal_x", "f32"),
            ("normal_y", "f32"),
        ],
    ),
    (
        "Sweep2D",
        &[
            ("hit", "bool"),
            ("node", "i32"),
            ("t", "f32"),
            ("origin_x", "f32"),
            ("origin_y", "f32"),
            ("normal_x", "f32"),
            ("normal_y", "f32"),
            ("start_overlap", "bool"),
        ],
    ),
    (
        "Image2D",
        &[
            ("sha256", "Str"),
            ("byte_length", "i32"),
            ("width", "i32"),
            ("height", "i32"),
        ],
    ),
    (
        "Frame2D",
        &[
            ("image", "Image2D"),
            ("x", "i32"),
            ("y", "i32"),
            ("w", "i32"),
            ("h", "i32"),
        ],
    ),
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

/// PLAN-981 B-W1（IF-981-GFX-AV-SEAM，UI058 AV 接面）：AGE 图形**服务**的
/// 执行车道分类——算法/身份归图形宿主（UI058 中立单元，PLAN-938 host
/// authority 车道消费同一受检产物），语言侧只交签名/受检投影/接线合同。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgeGfxLane {
    /// 纯值构造（`frame2d.at`）：AV 观察车道 lower 成端口 record Aggregate，
    /// 可独立执行/观察。
    ValueCtor,
    /// 宿主算法（query2d/input/tick/row/render2d）：AV 观察车道在调用点
    /// 具名拒（fail-closed，零效果）；由图形宿主车道执行。
    HostAlgorithm,
    /// 宿主身份（`image2d.png`：文件内容身份由宿主计算注入；语言侧不做 IO）
    /// ——AV 观察车道具名拒（同 `model3d.glb` 先例）。
    HostIdentity,
}

/// gfx 服务的结果形态。
#[derive(Clone, Copy, Debug)]
pub enum AgeGfxResult {
    Port(&'static str),
    Scalar(VT),
}

/// gfx 服务的形参形态（标量或端口引用——`frame2d.at` 的 Image2D 实参）。
#[derive(Clone, Copy, Debug)]
pub enum AgeGfxParam {
    Scalar(VT),
    Port(&'static str),
}

/// 一条 AGE 图形服务声明（UI058 所需六覆盖项的服务签名面；语义/预算映射
/// 由 UI058 T-02 与 981 联冻，本表是**受检投影**的唯一事实源）。
#[derive(Clone, Copy, Debug)]
pub struct AgeGfxServiceOp {
    /// 源里的限定写法（`query2d.segment`）。
    pub callee: &'static str,
    /// 受检产物里的稳定符号（跨后端身份）。
    pub symbol: &'static str,
    pub params: &'static [AgeGfxParam],
    pub result: AgeGfxResult,
    pub capability: &'static str,
    pub lane: AgeGfxLane,
}

pub const CAP_AGE2D_QUERY: &str = "age2d.query";
pub const CAP_AGE2D_INPUT: &str = "age2d.input";
pub const CAP_AGE2D_SCENE: &str = "age2d.scene";
pub const CAP_AGE2D_SPRITE: &str = "age2d.sprite";

const GP_I32: AgeGfxParam = AgeGfxParam::Scalar(VT::I32);
const GP_BOOL: AgeGfxParam = AgeGfxParam::Scalar(VT::Bool);
const GP_F32: AgeGfxParam = AgeGfxParam::Scalar(VT::F32);
const GP_STR: AgeGfxParam = AgeGfxParam::Scalar(VT::Str);
const GP_IMG: AgeGfxParam = AgeGfxParam::Port("Image2D");
const R_I32: AgeGfxResult = AgeGfxResult::Scalar(VT::I32);
const R_F32: AgeGfxResult = AgeGfxResult::Scalar(VT::F32);
const R_BOOL: AgeGfxResult = AgeGfxResult::Scalar(VT::Bool);

/// UI058 六覆盖项的服务签名（差额表 #1/#2/#3/#4/#6 的 981 侧接面）。
pub const AGE_GFX_SERVICE_OPS: &[AgeGfxServiceOp] = &[
    // #1 查询（算法=021 v3/v4 中立单元，宿主车道执行）
    AgeGfxServiceOp {
        callee: "query2d.segment",
        symbol: "age.query2d.segment.v1",
        params: &[GP_I32, GP_BOOL, GP_F32, GP_F32, GP_F32, GP_F32],
        result: AgeGfxResult::Port("Cast2D"),
        capability: CAP_AGE2D_QUERY,
        lane: AgeGfxLane::HostAlgorithm,
    },
    AgeGfxServiceOp {
        callee: "query2d.sweep",
        symbol: "age.query2d.sweep.v1",
        params: &[GP_I32, GP_BOOL, GP_F32, GP_F32],
        result: AgeGfxResult::Port("Sweep2D"),
        capability: CAP_AGE2D_QUERY,
        lane: AgeGfxLane::HostAlgorithm,
    },
    // #3 输入/时钟（宿主逐 Tick 喂值；轴归一/pressed 语义由 UI058 冻结）
    AgeGfxServiceOp {
        callee: "input.space",
        symbol: "age.input.space.v1",
        params: &[],
        result: R_I32,
        capability: CAP_AGE2D_INPUT,
        lane: AgeGfxLane::HostAlgorithm,
    },
    AgeGfxServiceOp {
        callee: "input.x",
        symbol: "age.input.x.v1",
        params: &[],
        result: R_F32,
        capability: CAP_AGE2D_INPUT,
        lane: AgeGfxLane::HostAlgorithm,
    },
    AgeGfxServiceOp {
        callee: "input.y",
        symbol: "age.input.y.v1",
        params: &[],
        result: R_F32,
        capability: CAP_AGE2D_INPUT,
        lane: AgeGfxLane::HostAlgorithm,
    },
    AgeGfxServiceOp {
        callee: "input.pressed",
        symbol: "age.input.pressed.v1",
        params: &[GP_I32],
        result: R_BOOL,
        capability: CAP_AGE2D_INPUT,
        lane: AgeGfxLane::HostAlgorithm,
    },
    AgeGfxServiceOp {
        callee: "tick.dt",
        symbol: "age.tick.dt.v1",
        params: &[],
        result: R_F32,
        capability: CAP_AGE2D_INPUT,
        lane: AgeGfxLane::HostAlgorithm,
    },
    // #2 行位置/移动（行状态=图形宿主 SoA；character 行位置写拒为宿主语义）
    AgeGfxServiceOp {
        callee: "row.place",
        symbol: "age.row.place.v1",
        params: &[GP_I32, GP_F32, GP_F32],
        result: R_I32,
        capability: CAP_AGE2D_SCENE,
        lane: AgeGfxLane::HostAlgorithm,
    },
    AgeGfxServiceOp {
        callee: "row.shift",
        symbol: "age.row.shift.v1",
        params: &[GP_I32, GP_F32, GP_F32],
        result: R_I32,
        capability: CAP_AGE2D_SCENE,
        lane: AgeGfxLane::HostAlgorithm,
    },
    AgeGfxServiceOp {
        callee: "row.show",
        symbol: "age.row.show.v1",
        params: &[GP_I32, GP_BOOL],
        result: R_I32,
        capability: CAP_AGE2D_SCENE,
        lane: AgeGfxLane::HostAlgorithm,
    },
    AgeGfxServiceOp {
        callee: "row.world",
        symbol: "age.row.world.v1",
        params: &[GP_I32, GP_I32],
        result: R_F32,
        capability: CAP_AGE2D_SCENE,
        lane: AgeGfxLane::HostAlgorithm,
    },
    AgeGfxServiceOp {
        callee: "row.local",
        symbol: "age.row.local.v1",
        params: &[GP_I32, GP_I32],
        result: R_F32,
        capability: CAP_AGE2D_SCENE,
        lane: AgeGfxLane::HostAlgorithm,
    },
    // #4 精灵端口（022 render2d 语义：代次 token、旧代写拒）
    AgeGfxServiceOp {
        callee: "render2d.spawn",
        symbol: "age.render2d.spawn.v1",
        params: &[GP_STR, GP_F32, GP_F32],
        result: R_I32,
        capability: CAP_AGE2D_SPRITE,
        lane: AgeGfxLane::HostAlgorithm,
    },
    AgeGfxServiceOp {
        callee: "render2d.despawn",
        symbol: "age.render2d.despawn.v1",
        params: &[GP_I32],
        result: R_I32,
        capability: CAP_AGE2D_SPRITE,
        lane: AgeGfxLane::HostAlgorithm,
    },
    AgeGfxServiceOp {
        callee: "render2d.frame",
        symbol: "age.render2d.frame.v1",
        params: &[GP_I32, GP_STR, GP_I32, GP_I32],
        result: R_I32,
        capability: CAP_AGE2D_SPRITE,
        lane: AgeGfxLane::HostAlgorithm,
    },
    // #4 资源：图像身份（宿主计算注入）与帧表（纯值构造，AV 可执行）
    AgeGfxServiceOp {
        callee: "image2d.png",
        symbol: "age.image2d.png.v1",
        params: &[GP_STR],
        result: AgeGfxResult::Port("Image2D"),
        capability: CAP_AGE2D_SPRITE,
        lane: AgeGfxLane::HostIdentity,
    },
    AgeGfxServiceOp {
        callee: "frame2d.at",
        symbol: "age.frame2d.at.v1",
        params: &[GP_IMG, GP_I32, GP_I32, GP_I32, GP_I32],
        result: AgeGfxResult::Port("Frame2D"),
        capability: CAP_AGE2D_SPRITE,
        lane: AgeGfxLane::ValueCtor,
    },
];

impl AgeGfxServiceOp {
    /// 端口结果的端口名（Scalar 结果为 None）。
    pub fn result_port_name(&self) -> Option<&'static str> {
        match self.result {
            AgeGfxResult::Port(name) => Some(name),
            AgeGfxResult::Scalar(_) => None,
        }
    }
}

/// 按限定写法查 gfx 服务。
pub fn gfx_lookup(callee: &str) -> Option<&'static AgeGfxServiceOp> {
    AGE_GFX_SERVICE_OPS.iter().find(|op| op.callee == callee)
}

/// 服务级信用计量的**每回调预算**（差额表 #6a，021 原额保持：32
/// credits/callback）。费率/映射与 UI058 T-02 **联冻**——本值是原语义
/// 冻结，不单方面改额。
pub const GFX_CALLBACK_CREDIT_BUDGET: u32 = 32;

/// 一次 gfx 服务调用的信用费（021 原额：cast 8、sweep 16；getter 是端口
/// 字段读、不构成服务调用；其余服务基价 1）。宿主车道（UI058）逐调用
/// 动态计费 + 第 33 次 sticky 拒时消费**同一张表**；语言侧在 gfx 派发点
/// 以每函数静态调用点合计做保守准入界（`lower.age-gfx-credits` 具名拒、
/// 构造性零效果）。
pub fn gfx_credit_cost(callee: &str) -> u32 {
    match callee {
        "query2d.segment" => 8,
        "query2d.sweep" => 16,
        _ => 1,
    }
}

/// 按稳定符号查 gfx 服务。
pub fn gfx_by_symbol(symbol: &str) -> Option<&'static AgeGfxServiceOp> {
    AGE_GFX_SERVICE_OPS.iter().find(|op| op.symbol == symbol)
}

/// 是否为 AGE 图形服务的**宿主车道** symbol（HostAlgorithm/HostIdentity——
/// 派发到宿主的 16 op；`frame2d.at` 是纯值构造，不在此列）。
///
/// 语言侧受检表的事实源：A2X 视图的 gfx 符号面（PLAN-981 A2C 宿主面）按
/// 本谓词收符号。AV 宿主派发（av-core `is_host_symbol`）是独立 VM 里的
/// **镜像谓词**（同集合；av-core 不依赖编译器 crate），两表变更须同步。
pub fn is_gfx_host_symbol(symbol: &str) -> bool {
    gfx_by_symbol(symbol).is_some_and(|op| op.lane != AgeGfxLane::ValueCtor)
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
    /// PLAN-981（G1，2026-11-18 裁定）：`layer: render-only` 冻结符号行
    /// ——可见不碰撞层，仅 `shape2d`。按冻结拼写校验（单值），进
    /// `AgeSceneRow.layer` typed 事实；其它元素/其它值具名拒。
    Layer,
}

/// PLAN-981（G1）：`layer:` 的唯一冻结值拼写。
pub const AGE_ROW_LAYER_RENDER_ONLY: &str = "render-only";

/// 一个属性名属于哪一类。`None` = 该属性名不在 AGE 的 2D 作者面内（具名拒绝）。
///
/// 这份表就是 AGE 规范 §1 的 `shape2d` 属性集：`kind`/`position`/`size`/
/// `material`/`visible`，加上组件实例的 `x`/`y`/`diameter`/`tint`，以及
/// 精灵叶属性面（差额表 #4b，PLAN-981）：`image:`/`frame:` 是**具名资源引用**
/// （Image2D / Frame2D），`flip:` 是 bool 值；G1 增 `layer:`（冻结符号
/// `render-only`，仅 `shape2d`）。
pub fn attr_value_kind(key: &str) -> Option<AttrValueKind> {
    match key {
        "kind" => Some(AttrValueKind::GeometryKind),
        // PLAN-981（G1）：render-only 层（可见不碰撞）。
        "layer" => Some(AttrValueKind::Layer),
        // `model3d` 的 `model:` 与 2D 的 `mesh:` 都是**具名资源引用**；
        // 精灵叶的 `image:`（Image2D）/`frame:`（Frame2D）同属一类。
        "material" | "tint" | "model" | "mesh" | "image" | "frame" => {
            Some(AttrValueKind::ResourceRef)
        }
        // 2D 标量/元组值 + 3D 的静态角度（yaw/pitch）+ 精灵翻转标志。
        "position" | "size" | "x" | "y" | "diameter" | "visible" | "yaw" | "pitch" | "scale"
        | "flip" => Some(AttrValueKind::Value),
        _ => None,
    }
}

/// 该 3D 叶种类拼写是否合法（`cylinder3d`/`model3d`/`group3d`）。
pub fn is_3d_element(kind: &str) -> bool {
    AGE_3D_ELEMENT_KINDS.contains(&kind)
}

/// 该属性名是否**适用于**这种场景元素。
///
/// `attr_value_kind` 是一张不带元素轴的扁平表，于是 `yaw:`（3D 静态角度）写在
/// `shape2d` 叶上、`size:`（2D 尺寸）写在 `cylinder3d` 上，都会被当作合法值接受。
///
/// **组件实例不受本表约束。** 组件实例的元素名是作者自定义的 node 名，它的属性名
/// 就是那个 node 声明的**参数名**——无法在 `(element, attribute)` 表里穷举。
/// 早期版本用 `is_component` 兜底把 14 个已知拼写放行，结果 `Badge(label: int)` 的
/// `label:` 被拒（"`label` does not apply to `Badge`"）——那是**真实的过度收紧**，
/// 实测确认。故现在只有 `shape2d` 与冻结 3D 元素走本表；组件实例一律放行，
/// 其参数类型由 `check_scene_resource_refs` 按**端口**校验（`tint` 等）。
pub fn attr_applies_to(element: &str, key: &str) -> bool {
    let is_2d_leaf = element == "shape2d";
    let is_3d = is_3d_element(element);
    if !is_2d_leaf && !is_3d {
        // 组件实例：属性名由作者在该 node 的声明里定义，本表无从判断。
        return true;
    }
    match key {
        "kind" => is_2d_leaf,
        // 3D 专属
        "yaw" | "pitch" | "model" | "scale" => is_3d,
        // 2D 专属（`image`/`frame`/`flip`/`layer` 是 2D 作者面属性，不进 3D）
        "size" | "material" | "mesh" | "image" | "frame" | "flip" | "layer" => is_2d_leaf,
        // 两类都可用
        "position" | "visible" | "tint" | "x" | "y" | "diameter" => true,
        _ => false,
    }
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
        // 2D 精灵叶（差额表 #4b）：`image:` 收 Image2D、`frame:` 收 Frame2D。
        ("shape2d", "image") => Some("Image2D"),
        ("shape2d", "frame") => Some("Frame2D"),
        // 3D 叶
        ("model3d", "model") => Some("Model3D"),
        _ => {
            // 组件实例（`Disc red_disc { tint: … }`）的元素名是**组件名**，无法在
            // 本表里逐一列举，所以按属性名兜底：`tint:` 无论出现在哪种元素上，
            // 都必须绑 `Material2D`；精灵键同理（`image:`/`frame:` 绑定端口
            // 与元素无关——组件参数 `img: Image2D` 可喂给精灵行）。
            //
            // 复审指出：此前本表只有 `("shape2d","material")` 等条目，**没有 `tint`**，
            // 于是 `node Disc(tint: int) { … tint: tint }` 仍能编译——而那正是
            // g01 组件实例实际使用的属性，B4 的不变量恰好漏掉了真实路径。
            match key {
                "tint" => Some("Material2D"),
                "image" => Some("Image2D"),
                "frame" => Some("Frame2D"),
                _ => None,
            }
        }
    }
}
