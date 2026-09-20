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

// ---------------------------------------------------------------------------
// PLAN-037 T-03：供给决策树（四臂）+ 按需装载/卸载
// ---------------------------------------------------------------------------

/// 一个 app 的 proxy 供给计划（纯决策产物，四臂中 ②③ 的落地面；① daemon
/// 臂是真进程不进 proxy——由既有 `ensure_daemon_if_declared` 独立消费，
/// 不在本计划产物内；④ none = 空计划）。
#[derive(Debug, Default, PartialEq)]
pub struct BackendPlan {
    /// ② capability 臂：pac `media_root:` 声明 → proxy 原生 media 路由。
    pub native_media: Option<crate::back_proxy::NativeMediaApp>,
    /// ③ session 臂：back api.at 特形谓词命中 → proxy VM session。
    pub session: Option<crate::back_proxy::SessionSpec>,
}

impl BackendPlan {
    pub fn is_empty(&self) -> bool {
        self.native_media.is_none() && self.session.is_none()
    }
}

/// PLAN-037 T-03：launch 期后端供给决策树（§5.1）。app_key = launch 名
/// （注册表 id；与 658 子 URL 段同形——前缀化 root 与 proxy 表两侧同源，
/// 自洽）。②③可叠加（一个 app 既声明 media_root 又有特形 back）。
pub fn plan_backend(spec: &crate::ui::session::LaunchSpec, app_key: &str) -> BackendPlan {
    let mut plan = BackendPlan::default();
    if spec.media_root.is_some() {
        plan.native_media = Some(crate::back_proxy::NativeMediaApp {
            app_id: app_key.to_string(),
            media_root: spec.media_root.clone(),
        });
    }
    if let Some(entry) = &spec.back_entry {
        if let Ok(source) = std::fs::read_to_string(entry) {
            if back_needs_session(&source) {
                plan.session = Some(crate::back_proxy::SessionSpec {
                    app_id: app_key.to_string(),
                    back_entry: entry.clone(),
                });
            }
        }
    }
    plan
}

impl crate::ui::session::DesktopSession {
    /// PLAN-037 T-03/T-04：launch 的显式后端供给步骤——决策树裁决 →
    /// 懒启 proxy（首个命中时；boot 零常驻）→ 首装装载（计数已存在则
    /// 共享，不重复 add）→ 计数 +1。返回前缀化 root（Some = 该 app 有
    /// proxy 供给，launch 臂需对 spec.code 做内存态前缀化；None = ④ 零
    /// 供给）。AppId 归属绑定在 allocate_app 之后（`bind_app_backend`）。
    pub fn ensure_backend(&mut self, spec: &crate::ui::session::LaunchSpec, app_key: &str) -> Option<String> {
        let plan = plan_backend(spec, app_key);
        if plan.is_empty() {
            return None;
        }
        if self.desktop.back_proxy.is_none() {
            match crate::back_proxy::start(crate::back_proxy::BackProxyConfig::default()) {
                Ok(proxy) => {
                    eprintln!(
                        "[session] back-proxy lazy-start on {:?} (port {})",
                        app_key, proxy.port
                    );
                    self.desktop.back_proxy = Some(proxy);
                }
                Err(err) => {
                    // 懒启失败降级：launch 继续（前端拿可诊断的连接错误，
                    // 与零供给面同形），下次命中重试。
                    eprintln!("[session] back-proxy start failed (degraded): {err}");
                    return None;
                }
            }
        }
        let proxy = self.desktop.back_proxy.as_ref()?;
        // 首装才装载（双窗第二次命中走共享——一份后端，计数 +1）。
        let newly = !self.desktop.back_refs.contains_key(app_key);
        if newly {
            if let Some(media) = plan.native_media {
                proxy.add_native_media(media);
            }
            if let Some(session) = plan.session {
                if let Err(err) = proxy.add_session(session) {
                    eprintln!("[session] back-proxy add_session {app_key} failed: {err}");
                }
            }
        }
        *self.desktop.back_refs.entry(app_key.to_string()).or_insert(0) += 1;
        Some(proxy.base_url_for(app_key))
    }

    /// PLAN-037 T-04：AppId → app_key 供给归属绑定（allocate_app 后、
    /// 窗关闭站点前；ensure_backend 返回 Some 的 launch 臂独占调用）。
    pub fn bind_app_backend(&mut self, app_id: crate::ui::session::AppId, app_key: &str) {
        self.desktop.app_back.insert(app_id, app_key.to_string());
    }

    /// PLAN-037 T-04：关窗卸载（三站点统一挂——inproc CloseApp 命令臂 +
    /// broker 断连 + ReclaimWindow；无归属 App 为 no-op）。计数 -1 归零 →
    /// `remove_app` 摘表（drop sender → session 线程自然退出；JoinHandle
    /// drop = 分离线程）。proxy listener 常驻（§2 裁定，不拆）。
    pub fn release_backend(&mut self, app_id: crate::ui::session::AppId) {
        let Some(app_key) = self.desktop.app_back.remove(&app_id) else {
            return;
        };
        let exhausted = match self.desktop.back_refs.get_mut(&app_key) {
            Some(count) if *count > 1 => {
                *count -= 1;
                false
            }
            _ => true,
        };
        if exhausted {
            self.desktop.back_refs.remove(&app_key);
            if let Some(proxy) = self.desktop.back_proxy.as_ref() {
                drop(proxy.remove_app(&app_key));
            }
        }
    }
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

    /// PLAN-037 T-03：供给决策树四臂各一例（§5.1）。① daemon 臂不进
    /// 决策产物（真进程，ensure_daemon_if_declared 独立消费——断言面为
    /// "daemon 声明不产生 proxy 计划"）；④ none = 空计划。
    #[test]
    fn plan_backend_four_arms() {
        use crate::ui::session::LaunchSpec;
        // ④ none：纯前端（无 media_root、无 back 入口）。
        let plain = LaunchSpec {
            daemon: Some("autoos".into()),
            ..Default::default()
        };
        assert!(plan_backend(&plain, "app-x").is_empty(), "① daemon 臂零 proxy 计划（④ 形）");

        // ② capability：media_root 声明 → 原生 media 路由，无 session。
        let media = LaunchSpec {
            media_root: Some("E:\\Music\\".into()),
            ..Default::default()
        };
        let plan = plan_backend(&media, "020-music-player");
        assert_eq!(plan.native_media.as_ref().unwrap().app_id, "020-music-player");
        assert_eq!(plan.native_media.as_ref().unwrap().media_root.as_deref(), Some("E:\\Music\\"));
        assert!(plan.session.is_none());

        // ③ session：back api.at 特形（~Stream）→ VM session。
        let dir = std::env::temp_dir().join(format!(
            "p037-plan-backend-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let back_entry = dir.join("api.at");
        std::fs::write(&back_entry, "pub fn events() ~Stream<Event> { }").unwrap();
        let spec = LaunchSpec {
            back_entry: Some(back_entry.clone()),
            ..Default::default()
        };
        let plan = plan_backend(&spec, "017-chat");
        assert!(plan.native_media.is_none());
        assert_eq!(plan.session.as_ref().unwrap().app_id, "017-chat");
        assert_eq!(plan.session.as_ref().unwrap().back_entry, back_entry);

        // 普通 #[api] CRUD back：谓词不命中 → 空计划（inproc 合并编译
        /// CALL 面已通，T-00② 实证）。
        let plain_back = dir.join("crud.at");
        std::fs::write(&plain_back, "#[api(method = \"GET\", path = \"/api/notes\")]\npub fn list() int { return 1 }").unwrap();
        let spec = LaunchSpec {
            back_entry: Some(plain_back),
            ..Default::default()
        };
        assert!(plan_backend(&spec, "013-todo").is_empty(), "CRUD back 不建 session");

        // ②③叠加：media_root + 特形 back 同存 → 双命中。
        let both = LaunchSpec {
            media_root: Some("E:\\Music\\".into()),
            back_entry: Some(back_entry),
            ..Default::default()
        };
        let plan = plan_backend(&both, "combo");
        assert!(plan.native_media.is_some() && plan.session.is_some(), "②③叠加");

        // back_entry 指向缺席文件（注册后语料被删）：读失败按零供给，
        // 不 panic。
        let ghost = LaunchSpec {
            back_entry: Some(dir.join("ghost.at")),
            ..Default::default()
        };
        assert!(plan_backend(&ghost, "ghost").is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
