# PLAN-754 双端统一验收档（T-04）

> 2026-10-10，worktree `@lang-754`（plan-754-dev @ 8e55088c5）。
> website 产物：worktree `vitepress build` + `vitepress preview :4273`；
> 独立版产物：worktree 后端 :3030（PLAN-746/752 全链路基线）。

## AC 逐项实证

| AC | 结果 | 证据 |
|---|---|---|
| AC-01 website 渲染 sidebar+IDE | ✅ | preview 截图（新标签页）：VitePress 页头下左 Auto Playground 侧栏（Demo 28/书籍 634/测试 629/Parity 52）+ 右侧完整 IDE（Run/Trans/Debug/Share），与 :3030 独立版同形 |
| AC-02 ?note= 深链双端 | ✅ | 双端实录：① `:4273/playground.html?note=demo/04-fibonacci` → 编辑器载斐波那契 → Run `Completed in 96ms`；② `?note=vm-bootstrap/a2r_hello` → 载笔记 → Run 返回 lib 拼接字节码（.line 179+ = lib-legacy 代码行，prepend 契约跨端生效）；③ 无效 id 回退欢迎态（e2e unify.spec 3/3） |
| AC-03 apiBase 探测 | ✅ | 后端日志：每页载 probe 命中 `/api/examples`（4173 假同源被 JSON 探针识破→fallback 127.0.0.1:3030 生效）；缺席场景组件级降级提示在案（backendDown 门控） |
| AC-04 独立版零回归 | ✅ | frontend e2e 29 条有效绿（agent-debug/debug-vs-run 初报红为并行负载 flake，隔离复跑绿）；website dedupe 契约包全入册（@lezer/highlight 补齐，highlight.js 并发会话已入册） |
| AC-05 门禁 | ⏳ 待 T-05 | — |

## 排障记录（环境/缓存类，非代码缺陷）

1. **vitepress SPA 回退假阳**：preview 对 `/api/examples` 返 404+HTML（res.ok=false 已挡），
   但旧 probe 只看 `res.ok` 时若中间层返 200 HTML 会误判——已加 content-type+JSON 结构双校验。
2. **setup 期固化**：usePlaygroundFull 在 setup 读一次 apiBase——改为先探测后渲染（v-if ready）。
3. **浏览器软导航残留**：VitePress 客户端路由使 tab.navigate 成为软导航，旧 JS 常驻；
   重建后旧资产 404 致白页——换干净端口（4273）新标签页验证通过。
4. 并发会话 npm 操作曾多次清写 node_modules（highlight.js 消失复现）——npm ci 修复。

## 构建/测试命令（复现）

```bash
cd website && npm run build && npx vitepress preview --port 4273
cd crates/auto-playground/frontend && npx playwright test   # 29 绿
```
