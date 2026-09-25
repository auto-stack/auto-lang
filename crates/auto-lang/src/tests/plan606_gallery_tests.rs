//! Plan 606: Photo Gallery thumbnail and fit verification tests.

#[cfg(test)]
mod plan606_gallery_tests {
    use crate::ui::view::View;
    use crate::ui::component::Component;
    use crate::ui::interpreter::DynamicMessage;
    use crate::ui::style::StyleClass;

    fn collect_images(view: &View<DynamicMessage>, out: &mut Vec<(String, Option<crate::ui::Style>)>) {
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

    #[test]
    fn test_029_photo_gallery_thumbnails_render_with_resolved_src_and_cover_fit() {
        let mut dc = match crate::plan370_test_support::build_example_component("029-photo-gallery") {
            Some(dc) => dc,
            None => {
                eprintln!("[SKIP] 029-photo-gallery app.at not located");
                return;
            }
        };

        // PLAN-702 段驱动适配：挂载自发 Init 内含 api 调用即 park——驱动
        // 恢复泵至缩略图源就绪再渲染断言（生产 = __parked_resume_tick 泵）。
        crate::plan370_test_support::drive_parked_segments(&mut dc, |dc| {
            let mut imgs = Vec::new();
            collect_images(&dc.view(), &mut imgs);
            imgs.iter().filter(|(src, _)| !src.starts_with("lucide:")).count() >= 24
        }, "029 thumbnails");

        // Render initial grid view
        let view = dc.view();
        let mut all_images = Vec::new();
        collect_images(&view, &mut all_images);

        // Filter out lucide icons (e.g. moon/sun)
        let thumbnails: Vec<_> = all_images.into_iter()
            .filter(|(src, _)| !src.starts_with("lucide:"))
            .collect();

        // All 24 photos in the initial grid view must be rendered as thumbnail images
        assert_eq!(thumbnails.len(), 24, "Expected 24 thumbnail images in photo gallery grid, got {}", thumbnails.len());

        for (i, (src, style)) in thumbnails.iter().enumerate() {
            assert!(!src.is_empty(), "Thumbnail {} src should NOT be empty (Plan 606 bugfix)", i);
            // PLAN-668 R-16 (P642-D11 contract update): corpus has evolved to
            // "path reference + render-side inlining" (Plan 617/628) -- view-layer
            // src = resolved thumbnail file path (data-URL inlining is the
            // renderer's job). Path may target the main checkout's corpus when
            // run from a worktree (P642-D5), so no root anchor.
            assert!(
                src.contains("thumbnails") && src.ends_with(".jpg"),
                "Thumbnail {} src should be resolved thumbnail path (path-ref contract): {}",
                i, src
            );

            let s = style.as_ref().unwrap_or_else(|| panic!("Thumbnail {} should have style", i));
            let has_cover = s.classes.iter().any(|c| matches!(c, StyleClass::ObjectFit(crate::ui::style::ObjectFit::Cover)));
            assert!(has_cover, "Thumbnail {} should have StyleClass::ObjectFit(Cover) from fit: 'cover'", i);
        }

        // Test opening a photo viewer: OpenPhoto(1)
        dc.on(DynamicMessage::Typed {
            widget_name: "App".to_string(),
            event_name: "OpenPhoto".to_string(),
            args: vec![auto_val::Value::Int(1)],
        });

        let view_mode = dc.view();
        let mut all_viewer_images = Vec::new();
        collect_images(&view_mode, &mut all_viewer_images);

        let viewer_photos: Vec<_> = all_viewer_images.into_iter()
            .filter(|(src, _)| !src.starts_with("lucide:"))
            .collect();

        assert_eq!(viewer_photos.len(), 1, "Viewer mode should have exactly 1 full image, got {}", viewer_photos.len());
        let (full_src, full_style) = &viewer_photos[0];
        // PLAN-668 R-16：全图同"path 引用 + 渲染端内嵌"契约——src = 解析后
        // 的原图路径（语料引用用户图片目录，环境相关不锚根），区别于缩略
        // 图（thumbnails/ 段）即可。
        assert!(
            !full_src.is_empty() && !full_src.contains("thumbnails"),
            "Full image should be the resolved original photo path: {}", full_src
        );
        let s = full_style.as_ref().expect("Full image should have style");
        let has_contain = s.classes.iter().any(|c| matches!(c, StyleClass::ObjectFit(crate::ui::style::ObjectFit::Contain)));
        assert!(has_contain, "Full image should have StyleClass::ObjectFit(Contain) from fit: 'contain'");
    }
}
