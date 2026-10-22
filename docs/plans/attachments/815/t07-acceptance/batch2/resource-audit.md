# PLAN-815 T-07 子批2 — 资源 owner/许可/唯一源与实际发行内容审计

> **子批3 勘误修正（2026-10-22，F-815-T07-B2-R1-02）**：本文档原 §1 表缺
> Inter 字体行、原 §3 "handoff 勘误"段与 §4 "资产组不直接进 792 交付包"
> 的断言**与实物不符**（复审 inter-resource-audit.json.txt /
> delivery-coverage.json.txt 证实）。以下已按实物纠正；完整交付闭包
> 编译期/embed 资源盘点（含 runtime Inter/壁纸/shell 壳与根级输入）见
> [batch3/resource-audit-closure.md](../batch3/resource-audit-closure.md)。

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
| Inter 字体（三字重） | **auto-ui-runtime** | `crates/auto-ui-runtime/src/ui/iced/assets/`（子批2 原表漏记，子批3 补正） | 3（Regular 411640B / Medium 417300B / SemiBold 419744B） | **SIL OFL 1.1**（全文 `INTER-OFL.txt` 4380B 同目录；renderer.rs:1848 注释在案） | include_bytes（`renderer.rs:1850-1852` `INTER_FONT_*` 常量） | iced 渲染器静态三字重加载；源文件随包（792 包同相对位、字节一致，复审 inter-resource-audit.json.txt 四文件全对） |
| iced 壁纸 | auto-ui-runtime | 同上目录 | 2（wallpaper-ricepaper.jpg 263845B / wallpaper-inkwash.jpg 179676B） | InHouse（本仓自著） | include_bytes（`renderer.rs:1859-1860`） | iced 桌面壁纸；源随包 |
| 桌面壳 UI 声明 | auto-ui-runtime | `crates/auto-ui-runtime/assets/` | 5（shell/switcher/notification_center/dashboard/desktop.at） | InHouse（本仓自著） | include_str（`ui/shell.rs:15-164`） | 桌面壳（shell/切换器/通知中心/仪表盘/桌面）UI 声明；源随包 |

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
- **Inter 字体为 SIL OFL 1.1**（非 InHouse）：三字重源文件 + 许可全文
  `INTER-OFL.txt` 位于 `crates/auto-ui-runtime/src/ui/iced/assets/`，
  随 crate 源进 792 交付包（复审逐文件字节核对通过）。
- **handoff.md 来源清单勘误链（如实记录）**：原稿 Inter 行误写位置
  "vue.rs 内嵌"（vue.rs 无字体消费，实际 owner=auto-ui-runtime renderer）；
  子批2 勘误时又以"全仓无 Inter/OFL"为由**错误删除该行**——该结论源于
  只搜了 UI 工具域目录与忽略规则误判（SILENT/EPSILON 误匹配分析据此
  排除了真实命中）。子批3 按 F-815-T07-B2-R1-02 纠正：Inter 行已按
  正确 owner/位置/许可恢复（见 handoff.md 修正注记）。生成工程字体
  形态不变：vue 生成物 index.html/CSS 不内嵌字体文件，沿用浏览器
  系统字体栈——Inter 仅属 iced（VM/rust 桌面）渲染面，与 vue 工程无关
  （这也是"vue.rs 内嵌"原稿位置错误的根因）。

## 4. 实际发行内容（792 交付包）对账

- 792 交付包资产面由交付清单 `delivery-manifest.json` 逐资源 sha256 钉住，
  随包 `verify.py` 独立校验（driver V02-resource-identity + V03a 负控四态：
  等长篡改/坏 hash/缺失/未登记）。本批 driver 重跑收据见同目录
  `driver-run3.log`（含 fresh/重定位 locked 真实构建）。
- **资产组的源码随包（子批3 纠正原 §4 断言）**：交付拷贝
  `delivery_copy_dir` 递归复制 UI 工具 crate 源——
  `crates/auto-ui-tools/assets/{gallery,wm,shadcn-ui}` 三组生产源
  **逐字节进入 792 交付包**（复审 delivery-coverage.json.txt：gallery 1 /
  wm 9 / shadcn 346 全对，含组级 LICENSE+SNAPSHOT.md），既作 vendored
  构建输入（RustEmbed 调试形态运行时读、release 形态编译期嵌入），也
  经 `auto-ui-tools` 的 RustEmbed 随**二进制**发行，materialize 时落
  生成工程。四条路径须区分：① app 资产 allowlist（`assets/avatar.png`
  经 delivery-manifest 逐资源校验）② vendored crate 构建输入（三组源 +
  runtime 字体/壁纸/壳声明 + 根级输入 stdlib/、docs/schema/aura.at，
  随包不逐资源入 manifest——crate 目录完整性由构建本身保证）③ 编译后
  的 embed（随二进制）④ 用户工程 materialize 产物。原"资产组不直接进
  792 交付包"的说法混淆了 ① 与 ②，已纠正。
- 本批发现的交付包真实缺陷（canonical 双直依赖缺失致 fresh build E0433/
  E0252）及修复见 `batch2.md` §792——发行矩阵的 producer/source/embed/
  重定位/失败保留恢复/独立消费结论以修复后 driver 全程收据为准。
