//! Plan 606: Photo Gallery thumbnail and fit verification tests.
//! PLAN-759 T-06 重构：029 语料自 PLAN-043（231cca2fd）起 Init 经
//! `Http.get_json("/api/photos/scan")` 拉后端 photo_service——本测试内起
//! in-process photo stub（std TcpListener，back_proxy.rs 响应 schema 同形）
//! 并经 `AUTO_HTTP_BASE` 锚定，断言精确 URL 映射（origin/app/id），不再
//! 依赖机器后端在场的旧「thumbnails/ 路径 + OpenPhoto(Int)」契约。

#[cfg(test)]
mod plan606_gallery_tests {
    use crate::ui::component::Component;
    use crate::ui::interpreter::DynamicMessage;
    use crate::ui::style::StyleClass;
    use crate::ui::view::View;

    fn collect_images(
        view: &View<DynamicMessage>,
        out: &mut Vec<(String, Option<crate::ui::Style>)>,
    ) {
        match view {
            View::Image { src, style } => {
                out.push((src.clone(), style.clone()));
            }
            View::Column { children, .. } | View::Row { children, .. } => {
                for c in children {
                    collect_images(c, out);
                }
            }
            View::Grid { cells, .. } => {
                for c in cells {
                    collect_images(c, out);
                }
            }
            View::Container { child, .. } => {
                collect_images(child, out);
            }
            View::Button { content, .. } => {
                if let Some(c) = content {
                    collect_images(c, out);
                }
            }
            View::Scrollable { child, .. } => {
                collect_images(child, out);
            }
            _ => {}
        }
    }

    /// in-process photo_service stub（std-only；back_proxy.rs:1020-1060 响应
    /// schema 同形）。OS 分配端口；accept 非阻塞轮询 + read 超时有界；
    /// stop 旗标翻转后线程自行退出（join 不悬挂）。请求计数供断言「stub
    /// 侧真实收到 scan」。
    struct PhotoStub {
        port: u16,
        scan_hits: std::sync::Arc<std::sync::atomic::AtomicUsize>,
        stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
        handle: Option<std::thread::JoinHandle<()>>,
    }

    impl PhotoStub {
        fn start() -> PhotoStub {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("stub bind");
            let port = listener.local_addr().unwrap().port();
            listener
                .set_nonblocking(true)
                .expect("stub nonblocking listener");
            let scan_hits = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let (scan_hits2, stop2) = (scan_hits.clone(), stop.clone());
            let handle = std::thread::spawn(move || {
                let ids: Vec<String> = (1..=24).map(|i| format!("p{i:02}")).collect();
                let base = format!("http://127.0.0.1:{port}");
                let entries: Vec<String> = ids
                    .iter()
                    .map(|id| {
                        format!(
                            "{{\"id\":\"{id}\",\"index\":0,\"title\":\"photo {id}\",\"name\":\"{id}.jpg\",\
                             \"rel_dir\":\"\",\"relative_path\":\"{id}.jpg\",\"extension\":\"jpg\",\
                             \"bytes\":1024,\"size_str\":\"1.0 KB\",\"width\":400,\"height\":300,\
                             \"date\":\"2026-01-01\",\"sort_key\":0,\
                             \"thumb_url\":\"{base}/apps/029-photo-gallery/api/photos/thumb/{id}\",\
                             \"full_url\":\"{base}/apps/029-photo-gallery/api/photos/full/{id}\"}}"
                        )
                    })
                    .collect();
                let scan_body = format!(
                    "{{\"root_missing\":false,\"entries\":[{}],\"dirs\":[]}}",
                    entries.join(",")
                );
                let png = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
                while !stop2.load(std::sync::atomic::Ordering::Relaxed) {
                    let (mut stream, _) = match listener.accept() {
                        Ok(s) => s,
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            std::thread::sleep(std::time::Duration::from_millis(2));
                            continue;
                        }
                        Err(_) => break,
                    };
                    stream.set_nonblocking(false).ok();
                    stream
                        .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                        .ok();
                    let mut buf = [0u8; 4096];
                    let _ = std::io::Read::read(&mut stream, &mut buf);
                    let req = String::from_utf8_lossy(&buf).to_string();
                    let path = req.split_whitespace().nth(1).unwrap_or("").to_string();
                    let (body, ctype) = if path.starts_with("/api/photos/scan") {
                        scan_hits2.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        (scan_body.clone().into_bytes(), "application/json")
                    } else if path.contains("/api/photos/thumb/")
                        || path.contains("/api/photos/full/")
                    {
                        (png.clone(), "image/png")
                    } else {
                        (b"{\"error\":\"not found\"}".to_vec(), "application/json")
                    };
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
                    let _ = std::io::Write::write_all(&mut stream, &body);
                }
            });
            PhotoStub {
                port,
                scan_hits,
                stop,
                handle: Some(handle),
            }
        }

        fn base(&self) -> String {
            format!("http://127.0.0.1:{}", self.port)
        }

        fn thumb_url(&self, id: &str) -> String {
            format!(
                "{}/apps/029-photo-gallery/api/photos/thumb/{id}",
                self.base()
            )
        }

        fn full_url(&self, id: &str) -> String {
            format!(
                "{}/apps/029-photo-gallery/api/photos/full/{id}",
                self.base()
            )
        }
    }

    impl Drop for PhotoStub {
        fn drop(&mut self) {
            self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
            if let Some(h) = self.handle.take() {
                let _ = h.join();
            }
        }
    }

    #[test]
    fn test_029_photo_gallery_thumbnails_render_with_resolved_src_and_cover_fit() {
        // AUTO_HTTP_BASE 锚定 stub（请求期解析，stdlib resolve_http_base_url）。
        // 须在 build_example_component **之前**：fire_init 同步派发 Init，
        // Http.get_json 首调即提交 job 并读 env——晚置 env 时请求带未解析
        // 相对 URL 立败（load_error 形态，PLAN-759 T-06 实测定谳）。
        // 隔离形态（F-759-R2）：锁=与 stdlib http_base_url_resolution 共用的
        // 串行锁（裸 libtest 并行不再互踩）；守卫=HttpBaseEnvGuard panic-safe
        // 恢复（中途 build/驱动/断言失败不再污染进程 env）。
        let _ser = auto_lang_core::vm::ffi::stdlib::lock_http_base_for_test();
        let stub = PhotoStub::start();
        let _env = auto_lang_core::vm::ffi::stdlib::HttpBaseEnvGuard::set(&stub.base());

        let mut dc = match crate::plan370_test_support::build_example_component("029-photo-gallery")
        {
            Some(dc) => dc,
            None => {
                eprintln!("[SKIP] 029-photo-gallery app.at not located");
                return;
            }
        };

        // PLAN-702 段驱动适配：挂载自发 Init 内含 api 调用即 park——驱动
        // 恢复泵至缩略图源就绪再渲染断言（生产 = __parked_resume_tick 泵）。
        crate::plan370_test_support::drive_parked_segments(
            &mut dc,
            |dc| {
                let mut imgs = Vec::new();
                collect_images(&dc.view(), &mut imgs);
                imgs.iter()
                    .filter(|(src, _)| !src.starts_with("lucide:"))
                    .count()
                    >= 24
            },
            "029 thumbnails",
        );

        // stub 侧真实收到 scan 请求（非缓存/非空态巧合）。
        assert!(
            stub.scan_hits.load(std::sync::atomic::Ordering::Relaxed) >= 1,
            "photo stub 须收到 /api/photos/scan 请求"
        );

        // Render initial grid view
        let view = dc.view();
        let mut all_images = Vec::new();
        collect_images(&view, &mut all_images);

        // Filter out lucide icons (e.g. moon/sun)
        let thumbnails: Vec<_> = all_images
            .into_iter()
            .filter(|(src, _)| !src.starts_with("lucide:"))
            .collect();

        // All 24 photos in the initial grid view must be rendered as thumbnail images
        assert_eq!(
            thumbnails.len(),
            24,
            "Expected 24 thumbnail images in photo gallery grid, got {}",
            thumbnails.len()
        );

        // 精确 URL 映射（PLAN-759 T-06/F-06）：每张 src 逐字等于
        // {base}/apps/029-photo-gallery/api/photos/thumb/{id}，id ∈ stub 集，
        // 且 24 个 id 各出现一次（非子串包含）。
        let mut got_urls: std::collections::BTreeSet<String> =
            thumbnails.iter().map(|(src, _)| src.clone()).collect();
        for (i, (src, style)) in thumbnails.iter().enumerate() {
            let expected: Vec<String> = (1..=24)
                .map(|n| stub.thumb_url(&format!("p{n:02}")))
                .collect();
            assert!(
                expected.contains(src),
                "Thumbnail {i} src must be the exact stub thumb URL ({src})"
            );
            let s = style
                .as_ref()
                .unwrap_or_else(|| panic!("Thumbnail {} should have style", i));
            let has_cover = s
                .classes
                .iter()
                .any(|c| matches!(c, StyleClass::ObjectFit(crate::ui::style::ObjectFit::Cover)));
            assert!(
                has_cover,
                "Thumbnail {} should have StyleClass::ObjectFit(Cover) from fit: 'cover'",
                i
            );
        }
        assert_eq!(got_urls.len(), 24, "24 个缩略图 URL 须互不相同");
        for n in 1..=24 {
            assert!(
                got_urls.remove(&stub.thumb_url(&format!("p{n:02}"))),
                "thumb URL p{n:02} 缺席"
            );
        }
        assert!(got_urls.is_empty(), "不得有 stub 集外的 thumb URL");

        // Test opening a photo viewer: OpenPhoto(<str id>) —— PLAN-043 起签名
        // 为 str（旧 Int(1) 形态已过时）。
        dc.on(DynamicMessage::Typed {
            widget_name: "App".to_string(),
            event_name: "OpenPhoto".to_string(),
            args: vec![auto_val::Value::str("p07")],
        });
        crate::plan370_test_support::drive_parked_segments(
            &mut dc,
            |dc| {
                let mut imgs = Vec::new();
                collect_images(&dc.view(), &mut imgs);
                imgs.iter()
                    .filter(|(src, _)| !src.starts_with("lucide:"))
                    .count()
                    == 1
            },
            "029 viewer",
        );

        let view_mode = dc.view();
        let mut all_viewer_images = Vec::new();
        collect_images(&view_mode, &mut all_viewer_images);

        let viewer_photos: Vec<_> = all_viewer_images
            .into_iter()
            .filter(|(src, _)| !src.starts_with("lucide:"))
            .collect();

        assert_eq!(
            viewer_photos.len(),
            1,
            "Viewer mode should have exactly 1 full image, got {}",
            viewer_photos.len()
        );
        let (full_src, full_style) = &viewer_photos[0];
        // 全图 URL 与所选同一 id 精确一致（origin/app/id 映射）。
        assert_eq!(
            full_src,
            &stub.full_url("p07"),
            "Full image src must be the exact stub full URL for the selected id"
        );
        let s = full_style.as_ref().expect("Full image should have style");
        let has_contain = s.classes.iter().any(|c| {
            matches!(
                c,
                StyleClass::ObjectFit(crate::ui::style::ObjectFit::Contain)
            )
        });
        assert!(
            has_contain,
            "Full image should have StyleClass::ObjectFit(Contain) from fit: 'contain'"
        );

        // env 恢复由 _env Drop 完成（正常/panic/early-return 全覆盖，
        // F-759-R2）。
        drop(stub);
    }
}
