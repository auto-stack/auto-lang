//! PLAN-037：launch 期作用域化**模块源前缀化 overlay**（un-gated）。
//!
//! 桌面后端供给（`ui::back_provision`）的前端 URL 前缀化有两层：
//! ① 入口源（`spec.code`）在 launch 臂直接变换；② **模块文件**
//! （`use player_store:` 等装载的 `.at`）——Http 调用点多在 store 模块
//! 而非入口（020 player_store.at:93 实证），模块装载分散在 lib.rs 多个
//! 读点且无统一源注入缝，故以进程级 overlay 承载：launch 臂在
//! `build_dynamic_component` 前置入 [`PrefixSpec`]（RAII guard，build 返回
//! 即清），模块读点经 [`apply`] 过滤——仅该 app 前端目录下的文件做行级
//! 改写，其余（含 `src/back/api.at` 的 `#[api(path="/api/...")]` 属性行
//! ——本就不含 `Http.`，双保险）原样透传。
//!
//! async HTTP 的 base 解析在**独立线程**做（`resolve_http_base_url` 进程
//! env 架构），请求期 per-app base 不可行（§2 拒绝面之外的架构实证）——
//! 源级变换是唯一能同时满足 per-app 隔离 + async 线程的机制。
//!
//! un-gated 原因：消费读点在 lib.rs 核心装载链（feature 无关）；`ui::
//! back_provision` re-export [`prefix_api_url_literals`] 维持 auto-man 引用
//! 路径稳定。

use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// 一个 launch 的前缀化规格。
#[derive(Debug, Clone)]
pub struct PrefixSpec {
    /// app 前端目录（入口 `app.at` 的父目录；其下 `.at` 全在改写面——
    /// 行级 `Http.` 规则保证 back 属性行零污染）。
    pub front_dir: PathBuf,
    /// proxy 根（`http://127.0.0.1:<port>`）。
    pub root: String,
    /// app_key（子 URL 段）。
    pub app_key: String,
}

static ACTIVE: Mutex<Option<PrefixSpec>> = Mutex::new(None);

/// 置入 overlay 并返回 RAII guard（drop 即清）。launch 臂 build 期专用。
pub fn set_guard(spec: PrefixSpec) -> PrefixGuard {
    *ACTIVE.lock().unwrap() = Some(spec);
    PrefixGuard
}

/// guard：drop 清 overlay（编译失败路径同样回收）。
#[must_use]
pub struct PrefixGuard;

impl Drop for PrefixGuard {
    fn drop(&mut self) {
        *ACTIVE.lock().unwrap() = None;
    }
}

/// 模块读点过滤：active 且文件在 `front_dir` 之下 → 行级前缀化；否则
/// 原样返回。路径按 canonicalize 归一比对（读点路径可能为相对形态），
/// 归一失败退原值比对。
pub fn apply(path: &Path, code: String) -> String {
    let Some(spec) = ACTIVE.lock().unwrap().as_ref().cloned() else {
        return code;
    };
    let under = path_under(path, &spec.front_dir);
    if code.contains("/api/media/scan") {
        eprintln!(
            "[P041-DBG] prefix apply: under={under} root={} key={} path={:?}",
            spec.root, spec.app_key, path
        );
    }
    if under {
        prefix_api_url_literals(&code, &spec.root, &spec.app_key)
    } else {
        code
    }
}

fn path_under(path: &Path, dir: &Path) -> bool {
    let p = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let d = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    p.starts_with(&d)
}

/// PLAN-658 T-02 原实现（自 auto-man vue.rs 迁入，PLAN-037 再迁本模块供
/// 桌面与 auto-man 双消费）：`/api/` 字面量子前缀化——仅当行内出现
/// `Http.`（HTTP 调用行）时，把该行的 `"/api/...` 引号字面量改写为绝对
/// URL `<root>/apps/<app_id>/api/...`。
///
/// 精确性依据（658 T-02 语料实证）：三 demo 前端全部相对调用点仅 020
/// player_store.at 一处（单行、同行含 `Http.`）；注释行不含 `Http.` 不受
/// 影响；back 模块的 `#[api(path = "/api/...")]` 属性行不含 `Http.`，
/// 路由收集零污染。多行调用形态（字面量与 `Http.` 不同行）不在改写面
/// ——当前语料无此形态，出现时回补（658 §5.3 注记同律）。
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
    fn guard_scopes_overlay_and_dir_predicate_filters() {
        let dir = std::env::temp_dir().join(format!(
            "p037-back-prefix-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let front = dir.join("src").join("front");
        std::fs::create_dir_all(&front).unwrap();
        let store = front.join("player_store.at");
        std::fs::write(&store, "let d = Http.get_json(\"/api/media/scan\")\n").unwrap();
        let back = dir.join("src").join("back").join("api.at");
        std::fs::create_dir_all(back.parent().unwrap()).unwrap();
        std::fs::write(
            &back,
            "#[api(method = \"GET\", path = \"/api/notes\")]\npub fn list() int { return 1 }\n",
        )
        .unwrap();

        // 未激活：全透传。
        assert_eq!(
            apply(&store, "let d = Http.get_json(\"/api/media/scan\")\n".to_string()),
            "let d = Http.get_json(\"/api/media/scan\")\n"
        );

        // 激活：front 下改写，front 外（back）透传。
        {
            let _g = set_guard(PrefixSpec {
                front_dir: front.clone(),
                root: "http://127.0.0.1:3358".to_string(),
                app_key: "020-music-player".to_string(),
            });
            assert_eq!(
                apply(&store, "let d = Http.get_json(\"/api/media/scan\")\n".to_string()),
                "let d = Http.get_json(\"http://127.0.0.1:3358/apps/020-music-player/api/media/scan\")\n",
                "front 目录下模块源改写"
            );
            let back_code =
                "#[api(method = \"GET\", path = \"/api/notes\")]\npub fn list() int { return 1 }\n".to_string();
            assert_eq!(apply(&back, back_code.clone()), back_code, "back 目录透传");
        }

        // guard drop 后：回归透传。
        assert_eq!(
            apply(&store, "let d = Http.get_json(\"/api/media/scan\")\n".to_string()),
            "let d = Http.get_json(\"/api/media/scan\")\n",
            "guard 生命周期外零改写"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 子目录模块（`pages/*.at`）与顶层模块同在改写面——`path_under` 对
    /// front_dir 的**递归**包含是行级改写的前提（018 的 routes 页在
    /// `pages/` 下）。
    #[test]
    fn subdir_modules_under_front_dir_are_transformed() {
        let dir = std::env::temp_dir().join(format!(
            "p712-back-prefix-sub-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let front = dir.join("src").join("front");
        let page = front.join("pages").join("book_detail.at");
        std::fs::create_dir_all(page.parent().unwrap()).unwrap();
        // 文件必须在盘（path_under 走 canonicalize——真实装载读点传入的
        // 就是已存在的文件路径）。
        std::fs::write(&page, "").unwrap();

        let code = ".chapters = Http.get_json(\"/api/books/\" + id)\n".to_string();
        {
            let _g = set_guard(PrefixSpec {
                front_dir: front.clone(),
                root: "http://127.0.0.1:3358".to_string(),
                app_key: "018-book-reader".to_string(),
            });
            assert_eq!(
                apply(&page, code.clone()),
                ".chapters = Http.get_json(\"http://127.0.0.1:3358/apps/018-book-reader/api/books/\" + id)\n",
                "pages/ 子目录模块必须与顶层模块同被改写"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// PLAN-712 T-17（r3，端到端红→绿锁）：**routes 路由页（pages/*.at）的
    /// 模块读点必须过前缀化 overlay**。此前 lib.rs 的路由页装载臂裸读文件
    /// （唯一未包 `apply` 的模块读点）——018-book-reader 实机实证：顶层
    /// `book_store.at` 前缀生效（书架取数 2 REQ 到 proxy），`pages/
    /// book_detail.at` 留相对路径 → VM 解析到 app 自身 back origin（死端口）
    /// → `Http.get_json` 吞错返空 → 详情页 0 entries。
    ///
    /// 本测用真实 TCP 监听充当 proxy 根：前缀化生效 ⇒ 页 Init 的 fetch 打到
    /// 监听口（路径带 `/apps/{key}/api/` 前缀）⇒ 数据回灌状态；未生效 ⇒
    /// 相对路径解析不到本监听口（红相形态：零请求 + 状态滞留初值）。
    #[test]
    #[cfg(feature = "ui-iced")]
    fn route_page_fetch_reaches_proxied_root_when_guard_active() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::{Arc, Mutex};

        // ① 真 TCP stub：accept 循环，逐连接读请求行、回固定 JSON。
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().unwrap().port();
        let seen_paths: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let served = Arc::new(AtomicUsize::new(0));
        {
            let seen_paths = Arc::clone(&seen_paths);
            let served = Arc::clone(&served);
            std::thread::spawn(move || {
                for stream in listener.incoming().flatten() {
                    let mut s = stream;
                    let mut buf = [0u8; 4096];
                    let _ = s.read(&mut buf);
                    // 请求行形如 `GET /apps/<key>/api/books/1 HTTP/1.1`。
                    let head = String::from_utf8_lossy(&buf).to_string();
                    if let Some(first) = head.lines().next() {
                        seen_paths.lock().unwrap().push(first.to_string());
                    }
                    let body = br#"{"title": "The Source"}"#;
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = s.write_all(resp.as_bytes());
                    let _ = s.write_all(body);
                    served.fetch_add(1, Ordering::Relaxed);
                }
            });
        }

        // ② fixture：routes 页 + 页内相对 `/api/` 取数（018 book_detail 同构）。
        let dir = tempfile::tempdir().expect("tempdir");
        let pages = dir.path().join("pages");
        std::fs::create_dir_all(&pages).expect("pages dir");
        std::fs::write(
            dir.path().join("app.at"),
            r#"
widget App {
    routes {
        "/" -> use detail
    }
    view {
        outlet
    }
}
"#,
        )
        .expect("write app.at");
        std::fs::write(
            pages.join("detail.at"),
            r#"
widget detail {
    msg { Init }

    model {
        var title str = "none"
        var ok bool = false
    }

    view {
        text .title {}
    }

    on {
        .Init -> {
            try {
                let d = Http.get_json("/api/books/1")
                .title = d.title
                .ok = true
            } catch {
                .ok = false
            }
        }
    }
}
"#,
        )
        .expect("write pages/detail.at");

        // ③ guard 激活 + 生产构建路径（路由页装载臂在其内）。
        let app_path = dir.path().join("app.at");
        let code = std::fs::read_to_string(&app_path).expect("read app.at");
        let mut dc = {
            let _g = set_guard(PrefixSpec {
                front_dir: dir.path().to_path_buf(),
                root: format!("http://127.0.0.1:{port}"),
                app_key: "018-book-reader".to_string(),
            });
            crate::build_dynamic_component(&code, app_path.to_str())
                .expect("routes app 必须能编译")
        };

        // ④ 驱动 Init（页 Init 是 demand 制——**先渲染**：outlet 挂载路由页
        //    时才登记页 Init demand（437 同款），再泵驱动；parked fetch 由
        //    async HTTP 执行器就绪，外层轮询等待）。
        dc.fire_init();
        let _ = dc.view_with_debug_gated(true);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut ok = false;
        while std::time::Instant::now() < deadline {
            dc.drive_scheduler_to_quiescence(10_000);
            let _ = dc.view_with_debug_gated(true);
            if matches!(dc.read_state("ok"), Ok(auto_val::Value::Bool(true))) {
                ok = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        // ⑤ 断言：fetch 到达 stub 且带前缀；数据回灌状态。
        let paths = seen_paths.lock().unwrap().clone();
        eprintln!(
            "T17-DBG ok={ok} title={:?} parked_io={} init_work={} cpu={} served={} paths={:?}",
            dc.read_state("title").ok(),
            dc.has_parked_io_tasks(),
            dc.has_pending_init_work(),
            dc.has_cpu_continuations(),
            served.load(Ordering::Relaxed),
            paths
        );
        assert!(
            ok,
            "页 Init 的取数应成功回灌（前缀化生效 ⇒ 打到本监听口）；\
             stub 收到的请求行：{paths:?}"
        );
        assert!(
            !paths.is_empty(),
            "stub 未收到任何请求——路由页取数未到达 proxy 根（前缀化未覆盖 pages/ 装载臂）"
        );
        assert!(
            paths.iter().any(|p| p.contains(&format!("/apps/018-book-reader/api/books/1"))),
            "请求行应带 /apps/<key>/api 前缀，实际：{paths:?}"
        );
        assert_eq!(
            dc.read_state("title").ok(),
            Some(auto_val::Value::Str("The Source".into())),
            "响应体应经 get_json 回灌到页状态"
        );
    }
}
