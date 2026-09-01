//! Plan 502 M1: diagram 标签发射对照——svg `<text>` 直通(vue 上下文分流 +
//! VM svgdoc 内容序列化)与 overlay 动态 arbitrary 值(`left-[${x}px]`)双轨。
//!
//! M1 决策输入(设计文档 diagram-components.md §6.1/§10.1):
//! - 轨 A(svg text): 双端序列化/发射到位即胜出——随 viewBox 缩放、零 DOM
//!   膨胀、与几何同源;
//! - 轨 B(overlay): 499 M3 已实证动态 arbitrary 值可用(line_chart tooltip
//!   LeftOffset 断言),但 CSS px 定位与 svg viewBox 响应式缩放脱钩,
//!   节点标签需固定宽容器才能对位——大图/响应式场景失配。
//!
//! 本测试锚定轨 A 的双端发射事实 + 轨 B 的可用性(对照完整)。

/// 与探针同源的源码(vm 轨从内联源码构建,不依赖 examples 树)。
const PROBE_SRC: &str = r##"
widget App {
    msg { Init }

    model {
        nodes List = []
        ovX int = 46
    }

    on {
        .Init -> {
            .nodes = [
                { x: "46", y: "64", label: "开始 start" },
                { x: "246", y: "64", label: "判定 check?" }
            ]
        }
    }

    view {
        col (style: "p-6 gap-2 font-sans") {
            svg (viewBox: "0 0 400 120", style: "w-full h-auto max-w-md border rounded") {
                rect (x: "30", y: "40", width: "130", height: "44", rx: "8", fill: "#dbeafe") {}
                for n in .nodes {
                    text n.label { x: n.x, y: n.y, fill: "#1e293b", font-size: "13" }
                }
            }
            col (style: "relative") {
                svg (viewBox: "0 0 400 120", style: "w-full h-auto max-w-md border rounded") {
                    line (x1: "30", y1: "62", x2: "230", y2: "62", stroke: "#64748b") {}
                }
                col (class: f"absolute left-[${.ovX}px] top-[58px]") {
                    text "overlay 标签" { style: "text-[13px]" }
                }
            }
        }
    }
}
"##;

/// VM 轨:svgdoc 序列化含 `<text>` 元素(内容 = 位置参数文本,x/y 进属性),
/// for 循环绑定下 record 字段(Dot)取值正确;overlay 动态 arbitrary 值
/// 解析为 LeftOffset(对照轨完整)。
#[cfg(feature = "ui-iced")]
#[test]
fn plan502_m1_vm_svgdoc_text_and_overlay() {
    let mut dc = crate::build_dynamic_component(PROBE_SRC, Some("plan502_m1_probe.at"))
        .expect("probe must build");
    dc.fire_init();
    dc.set_route("/");
    let (view, _, _) = dc.view_with_debug_gated(true);
    let dump = format!("{:?}", view);

    // 轨 A: svg text 直通——元素 + 属性 + 内容断言。VM 属性序为 HashMap
    // 迭代序(不确定),断言不锁顺序。
    assert!(
        dump.contains(r#"<text "#),
        "svg text 元素必须进 svgdoc(位置参数文本为内容、x 为属性)"
    );
    assert!(
        dump.contains(r#"x=\"46\""#),
        "text 的 x 属性必须序列化(字面量)"
    );
    assert!(
        dump.contains("开始 start"),
        "位置参数文本必须成为 text 元素内容"
    );
    assert!(
        dump.contains(r##"font-size=\"13\""##) && dump.contains(r##"fill=\"#1e293b\""##),
        "text 的 svg 表现属性(fill/font-size)必须序列化"
    );
    // for 循环内第二条(动态字段取值)
    assert!(
        dump.contains(r#"x=\"246\""#),
        "for 循环绑定的 record Dot 字段(n.x)必须逐项解析"
    );
    assert!(dump.contains("判定 check?"), "第二条循环标签内容");

    // 轨 B: overlay 动态 arbitrary 值(对照可用性)
    assert!(
        dump.contains("LeftOffset(46.0)"),
        "overlay 轨动态 left-[${{x}}px] 必须解析为 LeftOffset(499 M3 同源能力)"
    );
}

/// vue 轨:svg 子树内 text 直通为 `<text>`(字面量属性静态化),子树外
/// text→span 行为不变;overlay 动态类串保留。
#[test]
fn plan502_m1_vue_svg_text_passthrough() {
    let session = crate::session::CompilerSession::ui();
    let mut parser = crate::parser::Parser::from(PROBE_SRC).with_session(session);
    let ast = parser.parse().expect("probe must parse");
    let decl = ast
        .stmts
        .iter()
        .find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        })
        .expect("widget decl");
    let widget = crate::aura::extract_widget_from_decl(decl).expect("extract widget");
    let mut gen = crate::ui_gen::VueGenerator::new_shadcn();
    use crate::ui_gen::BackendGenerator;
    let sfc = gen.generate(&widget).expect("generate SFC");
    eprintln!("=== PLAN502 SFC ===\n{sfc}\n=== END ===");

    // 轨 A: svg 子树内 text → <text>(不是 span)。属性按字母序发射
    // (fill 在 x 前),断言不锁顺序。
    assert!(
        sfc.contains("<text "),
        "svg 子树内 text 必须直通为 SVG <text>,got:\n{sfc}"
    );
    assert!(
        sfc.contains("</text>"),
        "text 内容(位置参数)必须为元素内容而非属性"
    );
    assert!(
        sfc.contains(">{{ n.label }}</text>"),
        "位置参数文本必须是 text 元素内容(插值形态),且不退化为 span"
    );
    // 字面量 fill 静态化;动态 x(:x 绑定,v-for 作用域);内容插值
    assert!(
        sfc.contains("fill=\"#1e293b\""),
        "svg text 字面量表现属性静态化"
    );
    assert!(
        sfc.contains(":x=\"n.x\"") && sfc.contains(":y=\"n.y\""),
        "动态 x/y 经 v-bind(循环变量字段)"
    );
    assert!(
        sfc.contains(">{{ n.label }}</text>"),
        "位置参数文本必须是 text 元素内容(插值形态)"
    );
    // 子树外 text → span 不回归
    assert!(
        sfc.contains("<span"),
        "svg 子树外的 text→span 行为不变"
    );
    // 轨 B: overlay 动态类串保留(col 容器化,text 直挂 class f-string
    // 在 shadcn 路径退化 :style —— M1 实证注记)
    assert!(
        sfc.contains("left-[${ovX}px]"),
        "overlay 轨 f-string arbitrary 值必须保留在类串"
    );
}
