# PLAN-815 T-07 子批2 — 资源 owner/许可/唯一源与实际发行内容审计

2026-10-22，work 阶段交付。审计对象 = T-02 登记的资产清单
（`crates/auto-ui-tools/src/resources.rs` `ASSET_GROUPS`）+ T-08 交接稿
`handoff.md` 来源清单，对账实际仓内内容与真实发行/消费面。逐项核对基于
当前树（worktree f6fd0ce7c + 本批修复提交），文件计数为实测 `find -type f`。

## 1. 逐源对账表

| 组 | owner | 生产源（唯一） | 实测文件 | 许可 | embed 形态 | 实际消费/发行入口 |
|---|---|---|---|---|---|---|
| gallery | auto-ui-tools | `crates/auto-ui-tools/assets/gallery/` | 1（AppViewport.vue） | InHouse（本仓自著） | RustEmbed（`gallery_assets::GalleryAssets`） | `gallery_assets::materialize` → 生成工程 `src/gallery/`（画廊宿主壳） |
| wm | auto-ui-tools | `crates/auto-ui-tools/assets/wm/` | 9 | InHouse（本仓自著） | RustEmbed（`wm_assets::WmAssets`） | `wm_assets::materialize` → 生成工程 `src/wm/`（窗口管理运行时） |
| wm 外部输入 | auto-ui-tools | `crates/auto-ui-tools/drawlist-renderer/src/` | 5 | InHouse（本仓自著） | include_str（随组 T-04 迁移；`packages/drawlist-renderer` 旧位已删） | wm materialize 的 remote-renderer 拷贝源；wm-test 测试位双向钉住与包源漂移 |
| shadcn-ui | auto-ui-tools | `crates/auto-ui-tools/assets/shadcn-ui/` | 346（含组级 LICENSE+SNAPSHOT.md） | MIT（上游 shadcn-vue registry 快照 2026-08-27 + 本仓 PLAN-571 补丁，SNAPSHOT.md 记录） | RustEmbed（`vue_shadcn::ShadcnUiAssets`） | `vue_shadcn::materialize` 按组件 write-if-missing → 生成工程 `src/components/ui/<组件>/`；缺组件走 CLI 回退 |

## 2. 唯一源与历史副本

- 生产源唯一性：`resources_manifest.rs::manifest_groups_are_registered_once_with_licenses`
  钉住三组 `current_source` 互异；`historic_auto_man_copies_stay_removed`
  钉住 auto-man 旧位与仓根 `packages/drawlist-renderer` 不复现（T-04 一次性
  迁移纪律）。
- embed 面=登记源：本批新增
  `resources_manifest.rs::embed_consumption_matches_registered_sources`
  ——三组 embed iter 非空、锚文件（AppViewport.vue / VirtualWindow.vue）在
  embed 面、shadcn 组件面 ≥10、组级 LICENSE+SNAPSHOT.md 进入 embed 面。
- 777 边（跨仓依赖闭包）分支 strict 本轮零变化（无 manifest/边改动）；
  master 并集按分工归 T-10。

## 3. 许可旅行边界（如实记录，不新造义务）

- shadcn 组级 `LICENSE`（MIT）+ `SNAPSHOT.md`（来源快照证明）随组在仓内
  且进入 embed 面（随 `auto-ui-tool` 二进制发行）。**按组件 materialize
  只拷贝 `<组件>/` 下的文件，组级许可文件不随组件拷入用户工程**——上游
  shadcn-vue registry 本身也不随组件附逐文件许可头；本审计如实记录该
  边界，T-09 终审如需按组件携带许可再裁。
- gallery/wm/drawlist 为本仓自著（InHouse），无外部许可义务。
- **handoff.md 来源清单勘误（本批已修）**：原稿列有 "Inter 字体 | vue.rs
  内嵌（三字重）| SIL OFL 1.1"——全仓核实无任何 Inter 字体资产、无
  @font-face/woff/ttf/base64 字体内嵌、无 OFL 许可文本（`grep -rln
  "SIL\|OFL"` 全为 SILENT/EPSILON 类误命中）。该行与实际内容不符，已从
  handoff.md 删除并在原位留勘误注记。生成工程字体形态：index.html/CSS
  不内嵌字体文件，沿用浏览器系统字体栈。

## 4. 实际发行内容（792 交付包）对账

- 792 交付包资产面由交付清单 `delivery-manifest.json` 逐资源 sha256 钉住，
  随包 `verify.py` 独立校验（driver V02-resource-identity + V03a 负控四态：
  等长篡改/坏 hash/缺失/未登记）。本批 driver 重跑收据见同目录
  `driver-run3.log`（含 fresh/重定位 locked 真实构建）。
- 资产组不直接进 792 交付包：交付闭包=源 crate（DELIVERY_CRATES 26+app）；
  资产经 `auto-ui-tools` 的 RustEmbed 随**二进制**发行，materialize 时落
  生成工程。embed 面许可旅行见 §3。
- 本批发现的交付包真实缺陷（canonical 双直依赖缺失致 fresh build E0433/
  E0252）及修复见 `batch2.md` §792——发行矩阵的 producer/source/embed/
  重定位/失败保留恢复/独立消费结论以修复后 driver 全程收据为准。
