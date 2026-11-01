# lang-web-e2e（PLAN-906）

语言 facade 退役 `auto-web` 依赖（PLAN-906 Web 边界收口）后，需要 Web 宿主的
HTTP e2e 家族（原 `crates/auto-lang/src/tests/` 的 plan705/707/729/730/734/
736/776/781 八文件，~235 个测试 fn，ID/条件原样保留）迁入本独立探针工程：

- **依赖**：`auto_lang`（语言 facade 公开面）+ `auto_web`（显式宿主安装，
  `install_web_http_host()` 帮助函数）；不再经 facade 的 Web 再导出边。
- **非 workspace 成员**：组内兄弟布局解析
  `../../../../../auto-lang/crates/auto-lang`、`../../../../../auto-web/crates/auto-web`
  （与 web-port-probe 等既有探针同形态）；单仓语言检出（无 auto-web 兄弟）时
  本工程不构建——这正是 L2 独立边界（语言 workspace 普通构建不含 Web）。
- **执行通道**：四仓 fixture 门（`scripts/web-four-repository.py` +
  `web-repository-fixture.py`）驱动 `cargo test`；原 `http` 固定组已同步
  收紧排除本家族（见 `scripts/test-groups.toml`）。
- **语义**：测试内容逐字迁移，仅路径改写
  （`crate::`→`auto_lang::`、`crate::http_file_service`→`auto_web::http_file_service`、
  `crate::http_service_config`→`auto_web::http_service_config`、宿主安装改显式直连）。
