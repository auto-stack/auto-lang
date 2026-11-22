//! PLAN-106（014 真语料）B2：handler 重写无类型 view-data 记录列表的
//! **惰性收据**写面——`.hourly = [ { … } ]` 可 bind、可编译；展示仍读
//! view_data 初值（不落 state cell、不提升为 model 字段）；真正未知字段
//! 仍具名拒。
//!
//! 红相（刀前）：`compile_component_event_program` 5085 标量门 + bind 层
//! 三层拒绝任何列表字段写——014 原样源无法 `--source` 装载。
use ace_core::source::age_component::{
    compile_component_event_program, project_component_candidates,
};
use ace_core::source::age_h0::LegacyWidgetAlias;

/// 014 形最小源：Init 全量重写（静态列表）、SelectCity 按参数分支
/// （else-if 链 + 动态字段引用进列表）、Refresh 标量时间收据。
const WEATHER: &str = r#"
widget App {
    msg { SelectCity(str), Refresh, Init }
    model {
        var city_id str = "beijing"
        var city_zh str = "北京"
        var icon str = "⛅"
        var temp str = "23°"
        var cond_zh str = "多云"
        var updated_at str = "14:30"
        var hourly = [
            { time: "现在", icon: "⛅", temp: "23°", now: true },
            { time: "15:00", icon: "☁️", temp: "22°", now: false },
        ]
        var daily = [
            { day: "今天", icon: "⛅", cond: "多云", tmin: "18°", tmax: "25°" },
        ]
    }
    view {
        col {
            for h in .hourly {
                if h.time == "现在" {
                    text "现在" {}
                } else {
                    text "稍后" {}
                }
            }
        }
    }
    on {
        .Init -> {
            .hourly = [
                { time: "现在", icon: "⛅", temp: "23°", now: true },
                { time: "16:00", icon: "☁️", temp: "22°", now: false },
            ]
            .daily = [
                { day: "今天", icon: "⛅", cond: "多云", tmin: "18°", tmax: "25°" },
            ]
        }
        .SelectCity(id) -> {
            .city_id = id
            if id == "shanghai" {
                .city_zh = "上海"
            } else if id == "guangzhou" {
                .city_zh = "广州"
            } else {
                .city_zh = "北京"
            }
            .hourly = [
                { time: "现在", icon: .icon, temp: .temp, now: true },
                { time: "17:00", icon: .icon, temp: .temp, now: false },
            ]
        }
        .Refresh -> {
            .updated_at = "14:31"
        }
    }
}
"#;

/// ①AV 事件车道：014 形整源装载——列表重写 + else-if 链 + 标量写
/// （刀前 5085 标量门 + bind 双层拒绝，`--source` 无法装载）。
#[test]
fn p106_viewdata_list_writes_load_in_event_lane() {
    let caps = vec!["ac.runtime.text.v1".to_string()];
    compile_component_event_program(
        WEATHER,
        "p106-weather.at",
        LegacyWidgetAlias::Accept,
        &caps,
        None,
        true,
    )
    .expect("the 014-shaped source loads in the AV event lane");
}

/// ②inert 语义：重写目标仍是 view_data（展示源），不提升为 state 字段。
#[test]
fn p106_viewdata_writes_stay_viewdata_not_state() {
    let nodes = project_component_candidates(WEATHER).expect("the source projects");
    let node = &nodes[0];
    for name in ["hourly", "daily"] {
        assert!(
            node.view_data.iter().any(|f| f.name == name),
            "`{name}` stays a view-data display field"
        );
        assert!(
            !node.fields.iter().any(|f| f.name == name),
            "`{name}` gets no state slot (inert receipt)"
        );
    }
    assert!(
        node.fields.iter().any(|f| f.name == "city_id"),
        "scalar state fields still admit normally"
    );
}

/// ③正反边界：真正未知字段的列表重写仍具名拒——不因收据面静默吞。
#[test]
fn p106_unknown_field_list_write_still_rejects() {
    let bad = r#"
widget App {
    msg { M }
    model { var n int = 0 }
    view { text "x" {} }
    on {
        .M -> {
            .nosuch = [
                { time: "现在", icon: "⛅" },
            ]
        }
    }
}
"#;
    let caps = vec!["ac.runtime.text.v1".to_string()];
    let err = compile_component_event_program(
        bad,
        "p106-bad.at",
        LegacyWidgetAlias::Accept,
        &caps,
        None,
        true,
    )
    .expect_err("an unknown field list write must be named");
    assert!(
        err.iter().any(|d| d.message.contains("`nosuch`")),
        "named reject: {err:?}"
    );
}
