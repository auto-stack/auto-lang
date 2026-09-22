# PLAN-685 T-06 双臂对拍走查归档

基线：plan-685-dev @ T-05 提交后（8b8c94c0c + computed-chain 纪律修复），
auto.exe = worktree target/debug（fix + autodown default features）。

## 网格

- `vm/`：VM iced 臂（`auto run -r vm`，MCP autoui_screenshot），3 bp × 3 节。
  - `vm_00_initial` 与已删除的 `vm_overview_spec` 字节全同（首条回退选中
    dashboard/overview 且缺省节=Spec，显式点选同条同节视图不变）——初启
    即证回退链正确。
- `vue/`：vue 臂（gen/front/vue vite dev :3000，Playwright 1280×800 dark）。
- 代表 bp：dashboard/overview（长 spec/表格 props/双变体）、
  data-display/row-list（多 gotcha）、data-display/master-detail（表格式 props）。

## 视检结论（AC-06/07/08）

- markdown 渲染件双臂真渲染：标题层级/加粗标签（wrong:/why:/right:）/
  行内码 chip/无序编号列表/表格 chrome，裸 pre 墙消失（对照 685 前截图）。
- 三节 tab 直切双臂可用；VM 臂零滚动依赖；选中档 kind-pills 语汇一致。
- 头部件双臂在位：kind 面包屑（可点击=SelectKind 等价，MCP 实测）、
  变体计数徽标（2 variants/1 variant）、prev/next（到头降灰）。
- vue 臂 Spec tab 于 Playwright 点击后呈 focus 暗态（截留帧），非样式
  缺陷——VM 臂同款样式亮紫正常、同 styles 的 All pill 亮色正常。

## 走查工具（复跑入口）

- `shoot_grid.py <port> <out> <suffix>`：VM 臂网格（MCP）。
- `shoot_one.py <port> <bp> <tab> <png>`：单格带状态验证补拍。
- `shoot_vue.mjs <url> <out>`：vue 臂网格（Playwright）。
- `probe_vue.mjs`：vue 页渲染/console 错误探针（T-06 逮 computed-chain
  白屏 bug 的工具）。
