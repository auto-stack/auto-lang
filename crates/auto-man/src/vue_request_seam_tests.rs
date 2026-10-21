//! PLAN-815 T-04 随迁：I-PROJECT 请求缝对拍测试（原 vue.rs 内联）。
//!
//! 构造侧（Pac → GenerationRequest 的动态臂）是 auto-man
//! generation_request_adapter 的职责；消费侧经门面调
//! auto-ui-tools::vue 的双路径（legacy 自扫 vs request）。缝的对拍留在
//! adapter 所在侧，随 T-04 主体迁移析出。

#[cfg(test)]
mod plan809_request_seam {
    use crate::vue::{VueProject, VueReads};

    /// PLAN-809 T-06c V-02：请求路径与 legacy 自扫的读点对拍——同输入
    /// pac.at 下 VueReads 双源字段逐一相等 + 完整 from_workspace 双路径
    /// 项目字段一致（生成器项目读取改轻合同的等价性证据）。
    #[test]
    fn plan809_v02_reads_request_vs_legacy_parity() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let front = root.join("src").join("front");
        std::fs::create_dir_all(&front).unwrap();
        std::fs::write(front.join("app.at"), "widget W { view { text \"hi\" {} } }").unwrap();
        std::fs::write(
            root.join("pac.at"),
            "name: \"parity-app\"
title: \"Parity\"
title_zh: \"对拍\"
render: \"vue\"
shadcn: off
default_classes: on
front: \"src/front\"
npm_deps: [\"vue\", \"axios@^1.0.0\"]
styles: [\"theme.css\"]
i18n: true
window: \"800x600\"
",
        )
        .unwrap();

        let pac_path = root.join("pac.at");
        let pac_content = std::fs::read_to_string(&pac_path).unwrap();
        let legacy = VueReads::scan(root, &pac_path, &pac_content);

        // 生产请求构造路径（Pac::new → request_of）
        let config =
            auto_lang::config::AutoConfig::from_file(&pac_path, &auto_val::Obj::new()).unwrap();
        let pac = crate::pac::Pac::new(config);
        let req = crate::generation_request_adapter::request_of(&pac, &pac_path);
        let from_req = VueReads::from_request(&req);

        assert_eq!(from_req.name, legacy.name, "name");
        assert_eq!(from_req.shadcn, legacy.shadcn, "shadcn 决策");
        assert_eq!(
            from_req.default_classes, legacy.default_classes,
            "default_classes 决策"
        );
        assert_eq!(from_req.front_rel_path, legacy.front_rel_path, "front 声明");
        assert_eq!(from_req.index_title, legacy.index_title, "locale 链标题");
        assert_eq!(
            from_req.npm_deps, legacy.npm_deps,
            "npm_deps（数组+@ver 形态）"
        );
        assert_eq!(from_req.style_files, legacy.style_files, "styles");
        assert_eq!(from_req.i18n.enabled, legacy.i18n.enabled, "i18n.enabled");
        assert_eq!(
            from_req.theme.is_some(),
            legacy.theme.is_some(),
            "theme 合成存在性"
        );

        // 完整双路径：from_workspace（自扫）vs from_workspace_with_request
        let a = VueProject::from_workspace(root).unwrap();
        let b = VueProject::from_workspace_with_request(root, &req).unwrap();
        assert_eq!(a.name, b.name);
        assert_eq!(a.index_title, b.index_title);
        assert_eq!(a.npm_deps, b.npm_deps);
        assert_eq!(a.style_files, b.style_files);
        assert_eq!(a.i18n.enabled, b.i18n.enabled);
        assert_eq!(a.theme.is_some(), b.theme.is_some());
        assert_eq!(a.output_dir, b.output_dir);
    }

    /// R2-F03 对拍补例：仅 title_zh + app(front) 节点形态 + 主声明不变反控。
    #[test]
    fn plan809_r2_parity_zh_only_and_front_node() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let front = root.join("front");
        let src_front = root.join("src").join("front");
        std::fs::create_dir_all(&front).unwrap();
        std::fs::create_dir_all(&src_front).unwrap();
        std::fs::write(
            front.join("app.at"),
            "widget W { view { text \"node\" {} } }",
        )
        .unwrap();
        std::fs::write(
            src_front.join("app.at"),
            "widget S { view { text \"src\" {} } }",
        )
        .unwrap();
        std::fs::write(
            root.join("pac.at"),
            "name: \"fp\"
title_zh: \"Only Chinese\"
render: \"vue\"
app(\"front\") { }
",
        )
        .unwrap();
        let pac_path = root.join("pac.at");
        let config =
            auto_lang::config::AutoConfig::from_file(&pac_path, &auto_val::Obj::new()).unwrap();
        let pac = crate::pac::Pac::new(auto_lang::config::AutoConfig {
            code: config.code.clone(),
            root: config.root.clone(),
            args: config.args.clone(),
        });
        let req = crate::generation_request_adapter::request_of(&pac, &pac_path);
        assert_eq!(req.front_node_form.as_deref(), Some("./front"));
        let a = VueProject::from_workspace(root).unwrap();
        let b = VueProject::from_workspace_with_request(root, &req).unwrap();
        assert_eq!(a.front_dir, b.front_dir, "app(front) 节点形态目录一致");
        assert!(
            b.front_dir.ends_with("front"),
            "选 ./front 而非 src/front：{:?}",
            b.front_dir
        );
        assert_eq!(a.index_title.as_deref(), Some("Only Chinese"));
        assert_eq!(
            b.index_title.as_deref(),
            Some("Only Chinese"),
            "仅 title_zh 不得 None"
        );

        // 主声明不变反控：请求不变，磁盘后加 dep 不得改变请求路径产物集
        let req2 = crate::generation_request_adapter::request_of(&pac, &pac_path);
        assert_eq!(req2.dep_roots.len(), 0);
        std::fs::write(
            root.join("pac.at"),
            "name: \"fp\"
title_zh: \"Only Chinese\"
render: \"vue\"
app(\"front\") { }
dep \"late\" { path: \"../late\" }
",
        )
        .unwrap();
        let c = VueProject::from_workspace_with_request(root, &req2).unwrap();
        assert!(c.npm_deps.is_empty(), "磁盘后改的声明不得进入请求路径产物");
    }
}
