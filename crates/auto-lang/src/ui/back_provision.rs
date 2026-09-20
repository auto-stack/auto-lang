//! PLAN-037 T-02/T-03：桌面后端供给层（launch 决策树 + 按需 proxy 装载）。
//!
//! VM 桌面（iced/inproc 直挂形态）launch app 的后端供给此前为零（020 打开
//! 即空曲库——前端 `Http.get_json("/api/media/scan")` 无进程应答）。本模块
//! 把 PLAN-658 的 back proxy 承载层原样引进桌面宿主，生命周期自画廊的
//! "boot 期全量静态注册"翻转为 "launch 时按需装载、窗关卸载"：
//!
//! - [`back_needs_session`]：back api.at 特形谓词（658 画廊谓词迁入；
//!   auto-man 改复用——依赖方向 auto-man→auto-lang 成立）。
//! - [`prefix_api_url_literals`]：前端相对 URL 的内存态字面量前缀化
//!   （658 画廊同律迁入；桌面在 `build_dynamic_component` 前对 spec.code
//!   变换，不落盘不改语料）。
//! - [`ensure_backend`]（T-03）：launch 期四臂决策树（daemon/capability/
//!   session/none）单一裁决点。
//!
//! 门控：与 `session` 同门（ui-iced）——消费面为 iced 桌面宿主与 auto-man
//! （`features = ["ui-iced"]` 依赖）。

/// PLAN-037 T-02: back api.at 是否需要装载为 proxy VM session——特形谓词
/// （自 auto-man vue.rs `start_gallery_back_proxy` 内联谓词迁入，658 §5.2
/// 同律）：`~Stream`（SSE 流端点）/ `~Promise`（异步任务）/ 行首
/// `use auto.`（原生命名空间）任一命中。普通 `#[api]` CRUD back 不建
/// session——inproc 合并编译 CALL 面已通（PLAN-037 T-00② 实证）。
pub fn back_needs_session(back_api_source: &str) -> bool {
    back_api_source.contains("~Stream")
        || back_api_source.contains("~Promise")
        || back_api_source
            .lines()
            .any(|l| l.trim_start().starts_with("use auto."))
}

/// PLAN-037 T-02（自 auto-man vue.rs 迁入，PLAN-658 T-02 原实现）：
/// `/api/` 字面量子前缀化——仅当行内出现 `Http.`（HTTP 调用行）时，把该行
/// 的 `"/api/...` 引号字面量改写为绝对 URL `<root>/apps/<app_id>/api/...`。
///
/// 精确性依据（658 T-02 语料实证）：三 demo 前端全部相对调用点仅 020
/// player_store.at 一处（单行、同行含 `Http.`）；注释行不含 `Http.` 不受
/// 影响；back 模块的 `#[api(path = "/api/...")]` 属性行不含 `Http.`，路由
/// 收集零污染。多行调用形态（字面量与 `Http.` 不同行）不在改写面——当前
/// 语料无此形态，出现时回补（658 §5.3 注记同律）。
pub fn prefix_api_url_literals(content: &str, root: &str, app_id: &str) -> String {
    let marker = "\"/api/";
    let replacement = format!("\"{root}/apps/{app_id}/api/");
    let mut out = String::with_capacity(content.len());
    for line in content.split_inclusive('\n') {
        if line.contains("Http.") && line.contains(marker) {
            out.push_str(&line.replace(marker, &replacement));
        } else {
            out.push_str(line);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn needs_session_predicate_matrix() {
        // 三特形各自命中。
        assert!(back_needs_session("pub fn events() ~Stream<Event> { }"));
        assert!(back_needs_session("pub fn load() ~Promise<Data> { }"));
        assert!(back_needs_session("use auto.image\npub fn ok() int { return 1 }"));
        assert!(back_needs_session("    use auto.image"));
        // 普通 CRUD / 无特形提及 / 空串不命中。
        assert!(!back_needs_session(
            "#[api(method = \"GET\", path = \"/api/notes\")]\npub fn list() []Note { return notes }"
        ));
        // 注：658 谓词为朴素子串匹配——注释里出现 ~Stream 同样命中（保守
        // 侧偏置：多装载一个 session 而非漏装载），本矩阵不伪造注释豁免。
        assert!(!back_needs_session("// 谈论流端点与异步任务，但无特形词"));
        assert!(!back_needs_session(""));
    }

    #[test]
    fn prefix_rewrites_http_lines_only() {
        let src = "let data = json.to_value(Http.get_json(\"/api/media/scan\"))\n"
            .to_string()
            + "// 注释提及 \"/api/foo\" 但无调用\n"
            + "#[api(method = \"GET\", path = \"/api/notes\")]\n"
            + "let two = 1 + Http.get_json(\"/api/a\") + Http.post(\"/api/b\")\n";
        let out = prefix_api_url_literals(&src, "http://127.0.0.1:3358", "020-music-player");
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(
            lines[0],
            "let data = json.to_value(Http.get_json(\"http://127.0.0.1:3358/apps/020-music-player/api/media/scan\"))",
            "调用行内 \"/api/ 前缀化"
        );
        assert_eq!(lines[1], "// 注释提及 \"/api/foo\" 但无调用", "无调用行原样");
        assert_eq!(lines[2], "#[api(method = \"GET\", path = \"/api/notes\")]", "属性行原样");
        assert_eq!(
            lines[3],
            "let two = 1 + Http.get_json(\"http://127.0.0.1:3358/apps/020-music-player/api/a\") + Http.post(\"http://127.0.0.1:3358/apps/020-music-player/api/b\")",
            "同行多调用点全改写"
        );
        // 无 /api/ 字面量的 Http 行零改写。
        assert_eq!(
            prefix_api_url_literals("Http.get_json(\"https://example.com/x\")", "http://r", "a"),
            "Http.get_json(\"https://example.com/x\")"
        );
    }
}
