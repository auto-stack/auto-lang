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


/// Plan 502 顺带修复的预存债回归:`link "label" {}` 位置参数文本形态。
/// 此前 parse_view_link 仅认 (props) 形态,kitchen-sink(docs_gen 产物)
/// 的 link "sample" {} 整体解析失败 → vue 轨路由 import 断链整站白屏
/// (435 起预存,VM 轨仅丢页未暴露)。
#[test]
fn plan502_link_positional_text_parses() {
    let code = r#"
widget W {
    view {
        row {
            link "sample" {}
            link (href: "sample") {}
        }
    }
}
"#;
    let session = crate::session::CompilerSession::ui();
    let mut parser = crate::parser::Parser::from(code).with_session(session);
    let ast = parser.parse().expect("link positional form must parse");
    assert!(ast.stmts.iter().any(|s| matches!(s, crate::ast::Stmt::WidgetDecl(_))));
}


/// Plan 502 M3:Sugiyama-lite 布局几何对拍(端到端,真实组件)。
/// gallery /flow-diagram 页为双卡(td+lr)演示——P320 单态下同名组件
/// 双实例共享根状态且 Init 序非树序,末写者不确定;故 e2e 自建单实例
/// 临时工程(组件源码取自 gallery components,消费页按方向参数生成),
/// td/lr 各一跑,断言手算几何。算法值域由镜像对拍
/// (plan502_m3_layout_core_parity)覆盖,两者互为漂移围栏。
#[cfg(feature = "ui-iced")]
#[test]
fn plan502_m3_layout_geometry_e2e() {
    use std::path::PathBuf;
    let comp_src = {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/widgets-gallery/src/front/components/flow_diagram.at");
        match std::fs::read_to_string(&p) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("plan502 M3: SKIPPED — flow_diagram.at not found");
                return;
            }
        }
    };
    let demo = r##"
widget App {
    use { package: official from "./components" }
    model {
        flowNodes = [
            { id: "start", label: "开始", shape: "round" },
            { id: "check", label: "就绪?", shape: "diamond" },
            { id: "run",   label: "执行任务" },
            { id: "retry", label: "重试", shape: "round" },
            { id: "done",  label: "结束", shape: "round" }
        ]
        flowEdges = [
            { from: "start", to: "check" },
            { from: "check", to: "run" },
            { from: "run",   to: "done" },
            { from: "check", to: "retry" },
            { from: "retry", to: "check" }
        ]
    }
    view {
        col (style: "p-6") {
            flow-diagram (direction: "{DIR}") {
                nodes: .flowNodes
                edges: .flowEdges
            }
        }
    }
}
"##;

    let run_dir = |dir: &str| -> String {
        let root = std::env::temp_dir().join(format!("plan502_e2e_{}", dir));
        let front = root.join("src/front");
        let comps = front.join("components");
        std::fs::create_dir_all(&comps).unwrap();
        std::fs::write(comps.join("flow_diagram.at"), &comp_src).unwrap();
        std::fs::write(
            comps.join("package.at"),
            "name: \"official\"
version: \"0.1.0\"
namespace: \"auto\"
description: \"e2e fixture\"
",
        )
        .unwrap();
        let app = demo.replace("{DIR}", dir);
        let app_path = front.join("app.at");
        std::fs::write(&app_path, app).unwrap();
        let mut dc = crate::build_dynamic_component(
            &std::fs::read_to_string(&app_path).unwrap(),
            Some(app_path.to_str().unwrap()),
        )
        .expect("fixture must build");
        dc.fire_init();
        dc.set_route("/");
        let (view, _, _) = dc.view_with_debug_gated(true);
        format!("{:?}", view)
    };

    // ---- td:DFS 回边 = retry→check;分层 st=0/ck=1/[rn,rt]=2/dn=3;
    //      父居中 st/ck/dn 中轴 168,rn/rt 分居 84/252 ----
    let td = run_dir("td");
    // 属性序为 HashMap 迭代序(不确定),断言按属性独立落值
    assert!(td.contains(r##"x=\"108\""##), "td st/ck 中轴 x=108");
    assert!(td.contains(r##"y=\"24\""##) && td.contains(r##"y=\"132\""##), "td st(y24)/ck(y132) 分层");
    assert!(td.contains(r##"x=\"192\""##) && td.contains(r##"x=\"24\""##), "td rt(192)/rn(24) 同层分居");
    assert!(td.contains(r##"y=\"240\""##), "td rn/rt 同层 y=240");
    assert!(td.contains(r##"y=\"348\""##), "td dn 第 3 层 y=348");
    assert!(td.contains(r##"viewBox=\"0 0 336 416\""##), "td viewBox 包络");
    assert!(td.contains(r##"M 168 46 L 168 154"##), "td 主干垂直直线");
    assert!(td.contains(r##"M 252 262 L 168 154"##), "td 回环边向上(DFS 回边)");
    assert!(td.contains(r##"M 84 262 L 84 370"##), "td rn→dn 垂直(叶居中于父)");

    // ---- lr:转置,层沿 x 展开;st/ck/dn 中轴 y=100,rn/rt 分居 y=46/154 ----
    let lrd = run_dir("lr");
    assert!(lrd.contains(r##"y=\"78\""##) && lrd.contains(r##"x=\"192\""##), "lr st/ck 水平主干(y=78)");
    assert!(lrd.contains(r##"x=\"360\""##) && lrd.contains(r##"y=\"24\""##), "lr rn(360,y24)");
    assert!(lrd.contains(r##"y=\"132\""##) && lrd.contains(r##"x=\"528\""##), "lr rt(y132)/dn(528)");
    assert!(lrd.contains(r##"viewBox=\"0 0 672 200\""##), "lr viewBox 转置包络");
    assert!(lrd.contains(r##"M 84 100 L 252 100"##), "lr 主干水平直线");
}

// Plan 502 M3 追加:Sugiyama-lite 布局核几何对拍(镜像管线,script 上下文)。
// 覆盖:链式 rank、菱形分层、环回边剥离、barycenter 降交叉、父居中坐标、
// td/lr 转置与确定性(双跑一致)。
// 纪律:与 components/flow_diagram.at 的 Init 管线同构(chart 几何对拍先例);
// e2e(plan502_m3_layout_geometry_e2e)锚定真实组件,镜像锚定算法值域,
// 两者互为漂移围栏。
#[test]
fn plan502_m3_layout_core_parity() {
    let algo = r##"
var nw float = 120.0
var nh float = 44.0
var gx float = 48.0
var gy float = 64.0
var mg float = 24.0
var ranks List = []
var i int = 0
var ei int = 0
while i < n { ranks.push(0); i = i + 1 }
var color List = []
var cursor List = []
var stack List = []
var efdb List = []
i = 0
while i < n { color.push(0); cursor.push(0); efdb.push(0); i = i + 1 }
var sv int = 0
while sv < n {
    if color[sv] == 0 {
        color[sv] = 1
        stack.push(sv)
        while stack.len() > 0 {
            var v = stack[stack.len() - 1]
            var found int = -1
            var eIdx int = cursor[v]
            var brk bool = false
            while eIdx < m {
                if brk {
                    eIdx = m
                } else {
                    if ef[eIdx] == v {
                        var w = et[eIdx]
                        if color[w] == 1 {
                            efdb[eIdx] = 1
                        } else {
                            if color[w] == 0 {
                                found = w
                                cursor[v] = eIdx + 1
                                brk = true
                            }
                        }
                    }
                    if !brk { eIdx = eIdx + 1 }
                }
            }
            if found >= 0 {
                color[found] = 1
                stack.push(found)
            } else {
                cursor[v] = m
                color[v] = 2
                stack.pop()
            }
        }
    }
    sv = sv + 1
}
var round int = 0
while round < n + 2 {
    ei = 0
    while ei < m {
        if efdb[ei] == 0 {
            var rf2 = ranks[ef[ei]]
            if ranks[et[ei]] <= rf2 { ranks[et[ei]] = rf2 + 1 }
        }
        ei = ei + 1
    }
    round = round + 1
}
var maxRank int = 0
for r in ranks { if r > maxRank { maxRank = r } }
var cnt List = []
i = 0
while i <= maxRank { cnt.push(0); i = i + 1 }
for r in ranks { cnt[r] = cnt[r] + 1 }
var pos List = []
var cur List = []
i = 0
while i <= maxRank { cur.push(0); i = i + 1 }
i = 0
while i < n { pos.push(0); i = i + 1 }
i = 0
while i < n {
    var rl = ranks[i]
    pos[i] = cur[rl]
    cur[rl] = cur[rl] + 1
    i = i + 1
}
var mark List = []
i = 0
while i < n { mark.push(0); i = i + 1 }
var stamp int = 0
round = 0
while round < 2 {
    var pass int = 0
    while pass < 2 {
        var down bool = true
        if pass == 1 { down = false }
        var lr int = 0
        if !down { lr = maxRank }
        var moving bool = true
        if down { moving = lr <= maxRank } else { moving = lr >= 0 }
        while moving {
            var mem List = []
            var mp List = []
            var p int = 0
            while p < cnt[lr] {
                var kk int = 0
                while kk < n {
                    if ranks[kk] == lr {
                        if pos[kk] == p { mem.push(kk) }
                    }
                    kk = kk + 1
                }
                p = p + 1
            }
            var mi int = 0
            while mi < mem.len() {
                var vv = mem[mi]
                var sum float = 0.0
                var c int = 0
                stamp = stamp + 1
                var ei2 int = 0
                while ei2 < m {
                    var u int = -1
                    if ef[ei2] == vv { u = et[ei2] }
                    if et[ei2] == vv {
                        if ef[ei2] != vv { u = ef[ei2] }
                    }
                    if u >= 0 {
                        var take bool = false
                        if down {
                            if ranks[u] < lr { take = true }
                        } else {
                            if ranks[u] > lr { take = true }
                        }
                        if take {
                            if mark[u] != stamp {
                                mark[u] = stamp
                                var pu float = pos[u]
                                sum = sum + pu
                                c = c + 1
                            }
                        }
                    }
                    ei2 = ei2 + 1
                }
                var bc float = 0.0
                if c > 0 {
                    var cf float = c
                    bc = sum / cf
                } else {
                    var mf float = mi
                    bc = mf
                }
                mp.push(bc)
                mi = mi + 1
            }
            var a int = 1
            while a < mem.len() {
                var km = mem[a]
                var kbc = mp[a]
                var b int = a - 1
                while b >= 0 && mp[b] > kbc {
                    mem[b + 1] = mem[b]
                    mp[b + 1] = mp[b]
                    b = b - 1
                }
                mem[b + 1] = km
                mp[b + 1] = kbc
                a = a + 1
            }
            a = 0
            while a < mem.len() {
                pos[mem[a]] = a
                a = a + 1
            }
            if down { lr = lr + 1 } else { lr = lr - 1 }
            if down { moving = lr <= maxRank } else { moving = lr >= 0 }
        }
        pass = pass + 1
    }
    round = round + 1
}
var inStep float = nw + gx
var rankStep float = nh + gy
var inOff float = mg + nw / 2.0
var rankOff float = mg + nh / 2.0
if direction == "lr" {
    inStep = nh + gy
    rankStep = nw + gx
    inOff = mg + nh / 2.0
    rankOff = mg + nw / 2.0
}
var xs List = []
i = 0
while i < n {
    var pf float = pos[i]
    xs.push(inOff + pf * inStep)
    i = i + 1
}
var lr2 int = maxRank - 1
while lr2 >= 0 {
    var tgt List = []
    i = 0
    while i < n {
        if ranks[i] == lr2 {
            var sum2 float = 0.0
            var c2 int = 0
            stamp = stamp + 1
            var ei3 int = 0
            while ei3 < m {
                var u2 int = -1
                if ef[ei3] == i { u2 = et[ei3] }
                if et[ei3] == i {
                    if ef[ei3] != i { u2 = ef[ei3] }
                }
                if u2 >= 0 {
                    if ranks[u2] > lr2 {
                        if mark[u2] != stamp {
                            mark[u2] = stamp
                            var xu = xs[u2]
                            sum2 = sum2 + xu
                            c2 = c2 + 1
                        }
                    }
                }
                ei3 = ei3 + 1
            }
            var t = xs[i]
            if c2 > 0 {
                var cf2 float = c2
                t = sum2 / cf2
            }
            tgt.push(t)
        } else {
            tgt.push(0.0)
        }
        i = i + 1
    }
    var p3 int = 0
    var prev float = -1000000.0
    while p3 < cnt[lr2] {
        var kk2 int = 0
        while kk2 < n {
            if ranks[kk2] == lr2 {
                if pos[kk2] == p3 {
                    var xv = tgt[kk2]
                    if xv < prev + inStep {
                        xv = prev + inStep
                    }
                    xs[kk2] = xv
                    prev = xv
                }
            }
            kk2 = kk2 + 1
        }
        p3 = p3 + 1
    }
    lr2 = lr2 - 1
}
var outR = ""
var outF = ""
var outC = ""
for r in ranks { outR = outR + f"${r}," }
for f in efdb { outF = outF + f"${f}," }
i = 0
while i < n {
    var rk float = ranks[i]
    var inC float = xs[i]
    var rankC float = rankOff + rk * rankStep
    var cx float = inC
    var cy float = rankC
    if direction == "lr" {
        cx = rankC
        cy = inC
    }
    outC = outC + f"(${cx},${cy})"
    i = i + 1
}
f"r:${outR} fb:${outF} c:${outC}"
"##;

    let run_case = |n: &str, ef: &str, et: &str, dir: &str| -> String {
        let code = format!(
            "var n int = {n}\nvar ef List = {ef}\nvar et List = {et}\nvar m int = ef.len()\nvar direction = \"{dir}\"\n{algo}"
        );
        crate::run(&code).unwrap()
    };

    // ① 链:a→b→c。rank 0,1,2;无回边;td 中心 (84,46)(84,154)(84,262)。
    let chain = run_case("3", "[0, 1]", "[1, 2]", "td");
    assert_eq!(chain, "r:0,1,2, fb:0,0,0, c:(84,46)(84,154)(84,262)", "链式分层");

    // ② 菱形:a→{b,c}→d。b/c 同层,父居中 a/d 对齐中轴。
    let diamond = run_case("4", "[0, 0, 1, 2]", "[1, 2, 3, 3]", "td");
    assert_eq!(
        diamond,
        "r:0,1,1,2, fb:0,0,0,0, c:(168,46)(84,154)(252,154)(84,262)",
        "菱形分层 + 父居中"
    );

    // ③ 环:a→b→c→a。DFS 剥 c→a;DAG 分层 b=0,c=1,a=2。
    let cycle = run_case("3", "[0, 1, 2]", "[1, 2, 0]", "td");
    assert_eq!(cycle, "r:0,1,2, fb:0,0,1, c:(84,46)(84,154)(84,262)", "环回边剥离(c→a 回边,主干 a→b→c)");

    // ④ barycenter 降交叉:u1→v2, u2→v1 初始序 1 交叉;下扫后 v2 前置
    //    (v 层重心 = u 位均值)→ 0 交叉。
    let cross = run_case("4", "[0, 1]", "[3, 2]", "td");
    assert_eq!(
        cross,
        "r:0,0,1,1, fb:0,0,0,0, c:(84,46)(252,46)(252,154)(84,154)",
        "barycenter 降交叉(v2 pos0 < v1 pos1)"
    );

    // ⑤ gallery demo(5 节点回环)td 中心(与 e2e 手算一致)。
    let demo_td = run_case("5", "[0, 1, 2, 1, 3]", "[1, 2, 4, 3, 1]", "td");
    assert_eq!(
        demo_td,
        "r:0,1,2,2,3, fb:0,0,0,0,1, c:(168,46)(168,154)(84,262)(252,262)(84,370)",
        "demo td 主干中轴 + rn/rt 分居"
    );

    // ⑥ 同图 lr 转置:层沿 x 展开。
    let demo_lr = run_case("5", "[0, 1, 2, 1, 3]", "[1, 2, 4, 3, 1]", "lr");
    assert_eq!(
        demo_lr,
        "r:0,1,2,2,3, fb:0,0,0,0,1, c:(84,100)(252,100)(420,46)(420,154)(588,46)",
        "demo lr 转置"
    );

    // ⑦ 确定性:双跑一致。
    let again = run_case("5", "[0, 1, 2, 1, 3]", "[1, 2, 4, 3, 1]", "td");
    assert_eq!(demo_td, again, "布局确定性(同输入同输出)");
}
