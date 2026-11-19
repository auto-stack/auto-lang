//! PLAN-980（B-DIALECT-I/V 完整批）：传统 widget 原源经同一 AGE Schema
//! legacy-widget 别名的**全量投影 + 共享 verify + 字段级覆盖**永久 case。
//!
//! 输入：真实 002-counter 原源（frozen sha256/16 = 04937c35325faa74，与
//! auto-ui examples/ui/002-counter/src/front/app.at 一致——本仓不引 UI 仓文件，
//! 以冻结副本 + hash 锚定身份）。链路：原源 → widget f-string desugar →
//! age-component 前门 → 未受信候选（全成员结构 + 原 span）→ 场景编译（唯一
//! verify；2 实例 + 定向事件）→ 字段级覆盖（清单 vocab 全槽位）。
//! `node` 规范头副本同链路等价（同 ID 集/同结构）；别名关闭 = 声明头 span
//! 具名拒绝；deferred 槽位（style/setup/actions）= 成员 span 具名拒绝（见
//! plan980_field_manifest 的 reject case）。

use ace_core::atom_text::Span;
use ace_core::source::age_component::{
    compile_component_scenario_observing, project_component_candidates, ComponentScenario,
    ScenarioOp,
};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::domain::DomainVersion;
use ace_core::source::field_manifest::{
    component_field_ids, field_coverage, validate_field_manifest, FieldApplicability,
    FieldManifest, FieldPresented, RequiredFieldRow,
};

/// 002-counter 原源冻结副本（sha256 前 16 hex = 04937c35325faa74）。
const SRC_002: &str = r#"// PLAN-007 T-06：002-counter 接入官方主题族（C-13/14）。
// 业务保持：加/减/重置计数器（原 Plan 506 形态）；新增主题/模式切换，
// theme/mode 经 design 块数据绑定（同帧校验发布）。语义类（bg-primary
// 等）随官方树状主题双端联动；独立运行默认 Auto/light。
use themes.themes: Auto, Solarized

widget App {
    msg { Inc, Dec, Reset, UseAuto, UseSolarized, ToggleMode }

    model {
        var count int = 0
        var theme_name str = "Auto"
        var dark_mode bool = false
    }

    // 主题/模式选择器（C-06/07：数据绑定，同帧校验发布）
    design {
        theme: .theme_name
        mode: if .dark_mode { "dark" } else { "light" }
    }

    // Plan 506: 外壳改"内容即页面"（无 center 包裹，循 011 样板），配合
    // pac `window: "fit"` 窗口随内容收缩。
    view {
        col {
            text `Counter: ${.count}`
            row {
                button "-" { onclick: () => {.count -= 1} }
                button "Reset" { onclick: () => {.count = 0} }
                button "+" { onclick: () => {.count += 1} }
            }
            row {
                button "Auto" { onclick: .UseAuto, style: "px-3 py-1.5 rounded-lg bg-primary text-primary-foreground" }
                button "Solarized" { onclick: .UseSolarized, style: "px-3 py-1.5 rounded-lg bg-secondary text-secondary-foreground" }
                button "切换明暗" { onclick: .ToggleMode, style: "px-3 py-1.5 rounded-lg bg-accent text-accent-foreground" }
            }
            style: "items-center gap-4 p-6"
        }
    }

    on {
        .Inc -> { .count += 1 }
        .Dec -> { .count -= 1 }
        .Reset -> { .count = 0 }
        .UseAuto -> { .theme_name = "Auto" }
        .UseSolarized -> { .theme_name = "Solarized" }
        .ToggleMode -> { .dark_mode = !.dark_mode }
    }
}


// 对照：同一计数器的显式 msg/on 写法（与上面的内联 lambda 简写等价）。
// 简单回调直接用内联 lambda；当事件需要携带参数、或一处逻辑要被
// 多处/多事件复用时，改用下面的显式声明更合适。
//
// widget App {
//     msg { Inc, Dec, Reset }
//
//     model {
//         var count int = 0
//     }
//
//     view {
//         center {
//             text `Counter: ${.count}`
//             row {
//                 button "-" { onclick: .Dec }
//                 button "Reset" { onclick: .Reset }
//                 button "+" { onclick: .Inc }
//             }
//         }
//     }
//
//     on {
//         .Inc -> { .count = .count + 1 }
//         .Dec -> { .count = .count - 1 }
//         .Reset -> { .count = 0 }
//     }
// }
"#;

/// 002 锚定必需面清单（UI 侧权威清单 ui.widget-decl/v1 的 002 子集演示行；
/// ID 与 auto-ui p980_fields 一致）。
fn manifest_002() -> FieldManifest {
    let row = |id: &str, applicability: FieldApplicability| RequiredFieldRow {
        id: id.to_string(),
        origin: format!("plan980_dialect_full 002 subset ({id})"),
        source_span: Span { start: 0, end: 0 },
        parent_role: id
            .rsplit_once('.')
            .map(|(p, _)| p.to_string())
            .unwrap_or_default(),
        type_rule: "002-counter anchor".to_string(),
        applicability,
        projection_path: "ComponentNode candidates".to_string(),
        check_id: "p980_dialect_full_002".to_string(),
        consumer_owner: "980/DIALECT".to_string(),
    };
    let required = |m: &str| FieldApplicability::Required {
        mode: m.to_string(),
    };
    FieldManifest {
        domain: "ui.widget-decl".to_string(),
        version: DomainVersion { major: 1, minor: 0 },
        rows: vec![
            row("widget.decl.head", required("every decl")),
            row("widget.decl.name", required("every decl")),
            row("widget.msg", required("002-counter")),
            row("widget.msg.variant", required("002-counter")),
            row("widget.model", required("002-counter")),
            row("widget.model.field", required("002-counter")),
            row("widget.model.field.var", required("002-counter")),
            row("widget.design", required("002-counter")),
            row("widget.design.entry", required("002-counter")),
            row("widget.view", required("every decl")),
            row("widget.on", required("002-counter")),
            row("widget.on.handler", required("002-counter")),
            row("view.node.element", required("002-counter")),
            row("view.node.text", required("002-counter")),
            row("view.prop", required("002-counter")),
            row("view.event", required("002-counter")),
            row("view.event.inline", required("002-counter")),
        ],
    }
}

/// 全槽位 vocab（ID 与 auto-ui p980_fields 的 widget.*/view.* 行一致）。
fn vocab_full() -> ace_core::source::field_manifest::ComponentFieldVocab {
    let s = |v: &str| v.to_string();
    ace_core::source::field_manifest::ComponentFieldVocab {
        node_head: s("widget.decl.head"),
        node_name: s("widget.decl.name"),
        model_block: s("widget.model"),
        model_field: s("widget.model.field"),
        model_field_var: s("widget.model.field.var"),
        on_block: s("widget.on"),
        on_handler: s("widget.on.handler"),
        view_block: s("widget.view"),
        text_leaf: s("view.node.text"),
        msg_block: s("widget.msg"),
        msg_variant: s("widget.msg.variant"),
        msg_variant_payload: s("widget.msg.variant.payload"),
        design_block: s("widget.design"),
        design_entry: s("widget.design.entry"),
        view_element: s("view.node.element"),
        view_prop_entry: s("view.prop"),
        view_event_entry: s("view.event"),
        view_event_inline: s("view.event.inline"),
        bind_key: s("widget.bind.key"),
        route_def: s("widget.routes.def"),
        expose_name: s("widget.expose.name"),
        lifecycle_method: s("widget.lifecycle.method"),
        computed_property: s("widget.computed.property"),
        watch_decl: s("widget.watch.decl"),
        timer_entry: s("widget.timer.entry"),
        named_view: s("widget.view.named"),
    }
}

// ---- 内嵌 sha256（frozen 身份锚定；避免测试内新依赖） ----

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256_16(bytes: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut msg = bytes.to_vec();
    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for block in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let hex: String = h
        .iter()
        .flat_map(|x| format!("{x:08x}").chars().collect::<Vec<_>>())
        .collect();
    hex[..16].to_string()
}

// ---- 永久 case ----

#[test]
fn p980_dialect_full_002_projection_structure() {
    assert_eq!(
        sha256_16(SRC_002.as_bytes()),
        "04937c35325faa74",
        "frozen 002 identity"
    );
    let candidates =
        project_component_candidates(SRC_002).expect("002 projects through the AGE face");
    assert_eq!(candidates.len(), 1);
    let n = &candidates[0];
    // legacy 别名拼写经同一 Schema 准入（head 行同一 role）。
    assert!(n.legacy_widget);
    assert_eq!(n.head_spelling, "widget");
    assert_eq!(n.name.name, "App");
    // msg：6 声明变体（不含 UI 面 parse 期铸造的 __evt——AGE 面无该铸造）。
    let msgs: Vec<&str> = n.messages.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(
        msgs,
        vec![
            "Inc",
            "Dec",
            "Reset",
            "UseAuto",
            "UseSolarized",
            "ToggleMode"
        ]
    );
    // model：3 var 字段（int/str/bool）。
    assert_eq!(n.fields.len(), 3);
    assert_eq!(n.fields[0].name, "count");
    assert_eq!(n.fields[1].name, "theme_name");
    assert_eq!(n.fields[2].name, "dark_mode");
    // design：theme/mode 两行；mode=if/else 形结构性保留（value=None，span 锚定）。
    assert_eq!(n.design.len(), 2);
    assert_eq!(n.design[0].key, "theme");
    assert!(n.design[0].value.is_some());
    assert_eq!(n.design[1].key, "mode");
    assert!(
        n.design[1].value.is_none(),
        "if/else design values are structural in this slice"
    );
    // on：widget 形 6 handler（复合赋值脱糖为一元二元形）。
    assert_eq!(n.handlers.len(), 6);
    assert!(n.handlers.iter().all(|h| h.assigns.len() == 1));
    // view 树：col > [text(fstring), row(3 inline buttons), row(3 msg+style buttons), style]
    assert_eq!(n.view.texts.len(), 1);
    let (col_tag, col_items) = match &n.view.tree[0] {
        ace_core::source::age_component::ComponentViewItem::Element(e) => {
            (e.tag.clone(), e.items.len())
        }
        other => panic!("root must be col, got {other:?}"),
    };
    assert_eq!(col_tag, "col");
    assert_eq!(col_items, 4);
}

#[test]
fn p980_dialect_full_002_shared_verify_via_legacy_alias() {
    let scenario = ComponentScenario {
        instances: 2,
        ops: vec![
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Dec".to_string(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "UseSolarized".to_string(),
            },
            ScenarioOp::Send {
                instance: 1,
                msg: "Reset".to_string(),
            },
            ScenarioOp::Send {
                instance: 1,
                msg: "ToggleMode".to_string(),
            },
        ],
    };
    // 别名开启：全源（含 fstring 文本绑定 `.count`、str 状态字段、复合赋值、
    // `!` 一元、design 绑定）经唯一 verify 接受。
    let out = compile_component_scenario_observing(
        SRC_002,
        "plan980_dialect_full_002.at",
        LegacyWidgetAlias::Accept,
        &[],
        &scenario,
        Some("count"),
    )
    .expect("the full 002 source passes the shared scenario verify");
    assert!(out.description.nodes[0].legacy_widget);
    // 驱动 oracle：观测 count；两实例各有独立状态函数族（含 str/bool 字段）。
    assert!(out.atom_text.contains("__age_state_0_count"));
    assert!(out.atom_text.contains("__age_state_1_count"));
    assert!(out.atom_text.contains("__age_state_0_theme_name"));
    assert!(out.atom_text.contains("__age_state_1_dark_mode"));
    // fstring 文本叶的实例绑定函数在位（插值 `.count` 经 __fstr 链 verify）。
    assert!(out.atom_text.contains("__age_c_text_0"));

    // 别名关闭：legacy 头在原 span 具名拒绝（不回落旧引擎）。
    let ds = compile_component_scenario_observing(
        SRC_002,
        "plan980_dialect_full_002.at",
        LegacyWidgetAlias::Reject,
        &[],
        &scenario,
        Some("count"),
    )
    .expect_err("alias-off must reject the legacy head");
    assert_eq!(ds[0].code, "source.age.legacy-widget");
}

#[test]
fn p980_dialect_full_002_field_coverage_and_node_copy_equivalence() {
    let manifest = manifest_002();
    validate_field_manifest(&manifest).expect("subset manifest is executable");
    let vocab = vocab_full();
    let widget_candidates =
        project_component_candidates(SRC_002).expect("widget head projects via the alias");
    let presented = component_field_ids(&widget_candidates, &vocab);
    let cov = field_coverage(&presented, &manifest);
    assert!(
        cov.is_complete(),
        "the full 002 source closes the field coverage: {cov:?}"
    );
    // 计数独立 oracle（AGE 面；UI 面计数见 ui_domain_validation）：
    // 元素 10（col/text/2×row/6×button）、事件行 6（3 内联）、属性行 4、
    // 文本叶 1（fstring）、msg 变体 6、on handler 6、model 字段 3。
    let cnt = |id: &str| presented.iter().filter(|p| p.id == id).count();
    for (id, expected) in [
        // AGE 面：text 行是文本叶（非元素）——元素 9 = col + 2×row + 6×button
        //（UI 面把 text 计为元素 10；两面计数差已注记，行 ID 语义一致）。
        ("view.node.element", 9),
        ("view.event", 6),
        ("view.event.inline", 3),
        ("view.prop", 4),
        ("view.node.text", 1),
        ("widget.msg.variant", 6),
        ("widget.on.handler", 6),
        ("widget.model.field", 3),
        ("widget.model.field.var", 3),
    ] {
        assert_eq!(cnt(id), expected, "AGE-face occurrence count for {id}");
    }

    // 规范 Node 副本（只改声明头）：同 ID 集、同结构、legacy 旗翻转。
    let node_src = SRC_002.replacen("widget App", "node App", 1);
    let node_candidates = project_component_candidates(&node_src).expect("node head projects");
    assert!(!node_candidates[0].legacy_widget);
    let node_presented = component_field_ids(&node_candidates, &vocab);
    assert_eq!(
        presented.iter().map(|p| p.id.clone()).collect::<Vec<_>>(),
        node_presented
            .iter()
            .map(|p| p.id.clone())
            .collect::<Vec<_>>(),
        "head spelling does not change the field-id set"
    );
    // 破坏反控：删一条必需清单行 → 该 ID 进 unknown（红）；恢复 → 绿。
    let mut without = manifest.clone();
    without.rows.retain(|r| r.id != "view.event.inline");
    let cov = field_coverage(&presented, &without);
    assert_eq!(cov.unknown, vec!["view.event.inline".to_string()]);
    assert!(!cov.is_complete());
    assert!(
        field_coverage(&presented, &manifest).is_complete(),
        "restored green"
    );
}

// ---- 001-helloworld 腿（现行字段用例之二；frozen sha256/16 = b2763be5178fa040）----

const SRC_001: &str = r#"// PLAN-033 T-02：接入官方主题包（Auto/Solarized × light/dark）。
// 主题/模式经 design 块数据绑定（C-14 形态，同 010/002 先例）：仅新增主题
// 选择状态（theme_name/dark_mode——非业务模型），原问候文本/结构/可选文案
// 面逐字节保持；无新增菜单或业务事件。独立运行默认 Auto/light（原外观）。
use themes.themes: Auto, Solarized

widget App {
    model {
        var theme_name str = "Auto"
        var dark_mode bool = false
    }

    // 主题/模式选择器（C-06/07：数据绑定，同帧校验发布）
    design {
        theme: .theme_name
        mode: if .dark_mode { "dark" } else { "light" }
    }

    view {
        col {
            // Plan 481: selectable text —— 标题样式显式化(text-4xl 对齐 h1
            // 默认),正文可选可复制(VM 拖选/双击/Ctrl+C,vue 显式 user-select)。
            text "Hello, World!" {
                style: "text-4xl font-bold text-primary",
                selectable: true
            }
            // Plan 512: fit 迁移——center 居中外壳退役（Fill 会把内容测量
            // 钉回窗口尺寸），根 col Shrink 包裹 + 内边距。
            // PLAN-033 T-02：公共角色接入——根面 bg-background/text-foreground
            // （包角色；family/mode 有效值可见面。text-primary 保持既有 accent
            // 驱动语义——VM `Primary` 由运行时 accent 驱动是 031 交付契约）。
            style: "p-8 items-center bg-background text-foreground"
        }
    }
}
"#;

/// 001：全量结构投影（带属性的 text 元素经元素形承载——tag=text + label +
/// entries；绑定表达式形与 002 相同并已在 002 腿经共享 verify）。001 的
/// model 无 int 观测字段，场景驱动腿不适用（边界注记；非静默省略）。
#[test]
fn p980_dialect_full_001_projection() {
    assert_eq!(
        sha256_16(SRC_001.as_bytes()),
        "b2763be5178fa040",
        "frozen 001 identity"
    );
    let candidates =
        project_component_candidates(SRC_001).expect("001 projects through the AGE face");
    assert_eq!(candidates.len(), 1);
    let n = &candidates[0];
    assert!(n.legacy_widget);
    assert_eq!(n.fields.len(), 2);
    assert!(n.fields.iter().all(|f| f.mutable));
    assert_eq!(n.design.len(), 2);
    // view > col > [text 元素（label + style/selectable 两个属性行), style 属性行]
    let col = match &n.view.tree[0] {
        ace_core::source::age_component::ComponentViewItem::Element(e) => e,
        other => panic!("root must be col, got {other:?}"),
    };
    assert_eq!(col.tag, "col");
    assert_eq!(col.items.len(), 2);
    let text_el = match &col.items[0] {
        ace_core::source::age_component::ComponentViewItem::Element(e) => e,
        other => panic!("braced text with entries projects as an element, got {other:?}"),
    };
    assert_eq!(text_el.tag, "text");
    assert!(text_el.label.is_some());
    assert_eq!(text_el.items.len(), 2);
    assert!(text_el.items.iter().all(|it| match it {
        ace_core::source::age_component::ComponentViewItem::Entry(v) => !v.is_event,
        _ => false,
    }));
    // 覆盖：001 的必需面（无 msg/on/事件——design/model/view 元素/属性）。
    let vocab = vocab_full();
    let presented = component_field_ids(&candidates, &vocab);
    let cov = field_coverage(&presented, &manifest_002());
    // 001 缺 msg/on/事件/内联（002 锚定的 Required 行）——按锚定 fixture 口径，
    // 001 以结构断言核（上行），覆盖闭合以 002 为锚（本测试不冒充 002 口径）。
    assert!(cov.unknown.is_empty(), "001 has no unknown ids: {cov:?}");
}

/// AC-B-DIALECT 第一句：**普通标识符上下文保持**——`msg`/`design`/`bind` 等
/// 槽位拼写只在成员槽位有结构含义；在字段名/局部名/函数名位仍是普通标识符。
#[test]
fn p980_dialect_full_identifier_contexts_preserved() {
    // 字段名 = msg/design/bind（成员头拼写作为 model 字段名）。
    let src = "node C {\n    model {\n        msg int = 0\n        design int = 1\n        bind int = 2\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";
    let candidates = project_component_candidates(src)
        .expect("member-head spellings stay ordinary identifiers in field position");
    let n = &candidates[0];
    let names: Vec<&str> = n.fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, vec!["msg", "design", "bind"]);
    assert!(n.messages.is_empty() && n.design.is_empty() && n.members.is_empty());
    // 普通函数体内的局部名同名（slot 拼写不进词法保留字表）。
    let fn_src = "fn probe() int {\n    let design = 7\n    return design\n}\nnode C {\n    view {\n        text \"x\" {}\n    }\n}\n";
    let _ = project_component_candidates(fn_src)
        .expect_or_else_named("slot spellings are not reserved words", || String::new());
}

/// 内部辅助：Ok 断言（避免 unwrap 无消息）。
trait ExpectOrElseNamed<T> {
    fn expect_or_else_named(self, label: &str, f: impl FnOnce() -> String) -> T;
}
impl<T> ExpectOrElseNamed<T> for Result<T, Vec<ace_core::atom_text::Diagnostic>> {
    fn expect_or_else_named(self, label: &str, f: impl FnOnce() -> String) -> T {
        match self {
            Ok(v) => v,
            Err(_) => panic!("{label}: {}", f()),
        }
    }
}

/// 新成员面的坏形状：值缺冒号/handler 缺箭头/悬空逗号/未闭合元素括号——
/// 各自在原位具名拒绝（不是静默略过或整文件兜底）。
#[test]
fn p980_dialect_full_malformed_member_shapes_reject_at_span() {
    for (label, src, anchor) in [
        (
            "design-no-colon",
            "node C {\n    design { theme }\n    view {\n        text \"x\" {}\n    }\n}\n",
            "design",
        ),
        (
            "on-no-arrow",
            "node C {\n    on {\n        .Inc { .c = 1 }\n    }\n    view {\n        text \"x\" {}\n    }\n}\n",
            "on",
        ),
        (
            // PLAN-105 T-02：末项尾随逗号（`msg { A, B, }`）已成为合法书写形
            // （016 真语料逐行一成员、行尾逗号）；这里改用**双逗号**作坏形状。
            "msg-double-comma",
            "node C {\n    msg { A,, B }\n    view {\n        text \"x\" {}\n    }\n}\n",
            "msg",
        ),
        (
            "element-unclosed",
            "node C {\n    view {\n        col {\n            text \"x\" {}\n    }\n}\n",
            "col",
        ),
    ] {
        match project_component_candidates(src) {
            Ok(ns) => panic!("{label}: expected rejection, got {} nodes", ns.len()),
            Err(ds) => {
                assert!(!ds.is_empty(), "{label}: empty diags");
                let at = src.find(anchor).unwrap_or_else(|| panic!("{label} anchor"));
                assert!(
                    ds.iter().any(|d| d.span.end > at && d.span.start <= at + 40),
                    "{label}: diagnostic must point at the malformed member: {ds:?}"
                );
            }
        }
    }
}
