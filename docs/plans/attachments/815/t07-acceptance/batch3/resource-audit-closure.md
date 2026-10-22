# PLAN-815 T-07 子批3 — 交付闭包编译期/embed 资源完整盘点与实物对账

2026-10-22，work 阶段交付（F-815-T07-B2-R1-02 关闭件）。对账基线：唯一树
`D:/autostack/.wt/lang-815/auto-lang`（本批 docs 提交）+ 子批2 code
`e22fc7592`；实物 = 本批 driver 重定位交付包
`scratch-b3-r2/consumer-b`（33/33 全绿运行的实际产物，收据见
[driver-receipt.json](driver-receipt.json)）。逐文件字节对账结果：
[embed-closure-reconcile.json](embed-closure-reconcile.json)（八组全
`all_match=true`）。

## 1. 盘点方法与 oracle

- 盘点面 = 交付闭包 26 源 crate 的**生产代码**编译期嵌入（`include_bytes!`
  / `include_str!` / `#[derive(Embed)]`），逐 crate 全量扫描；`mod tests`
  内的 fixture/golden 嵌入（plan356/plan593/image.rs.at 测试钉等）不属
  发行资源面，不入表。
- 实物对账 oracle = 工作树源文件 vs 重定位交付包同相对位文件**逐字节相等**
  （非仅存在性）；组级 LICENSE/SNAPSHOT 单独确认在包内。
- 发行路径区分（纠正原子批2 §4 的混淆）：
  ① app 资产 allowlist——`assets/avatar.png` 经 delivery-manifest 逐资源
  sha256 + 随包 verify.py 校验（driver V02/V03a）；
  ② vendored crate 构建输入——下列全部源文件随包（crate 目录递归复制 +
  根级输入 DELIVERY_ROOT_INPUTS 显式随包）；
  ③ 编译后 embed——release 形态编译进二进制（RustEmbed 调试形态运行时读
  `$CARGO_MANIFEST_DIR` 相对面 = 包内 vendored 源，同字节）；
  ④ 用户工程 materialize 产物（generate 时从 embed 面落盘，本批 driver
  V05e 实测：随包工具生成的 shadcn button 物化文件 = 包内 vendored 源）。

## 2. 交付闭包编译期/embed 资源总表

| # | 组 | owner crate | 生产源（唯一） | 文件数/实测字节 | 许可 | 机制（代码位） | 消费入口 | 随包对账 |
|---|---|---|---|---|---|---|---|---|
| 1 | Inter 三字重 | auto-ui-runtime | `crates/auto-ui-runtime/src/ui/iced/assets/` | 3：Regular 411640 / Medium 417300 / SemiBold 419744 | **SIL OFL 1.1** | include_bytes（`src/ui/iced/renderer.rs:1850-1852` `INTER_FONT_*`） | iced 渲染器静态字体（vm/rust 桌面渲染面） | ✅ 字节一致（含 SHA256 复核=复审 inter-resource-audit.json.txt） |
| 2 | Inter OFL 许可全文 | auto-ui-runtime | 同上 `INTER-OFL.txt` | 1：4380 | SIL OFL 1.1 文本 | 非 include（许可文件随字体源同目录发行） | 字体许可合规（OFL 分发要求随字体携带） | ✅ 字节一致 |
| 3 | iced 壁纸 | auto-ui-runtime | 同上目录 | 2：ricepaper 263845 / inkwash 179676 | InHouse | include_bytes（`renderer.rs:1859-1860` `WALLPAPER_*`） | iced 桌面壁纸 | ✅ 字节一致 |
| 4 | 桌面壳 UI 声明 | auto-ui-runtime | `crates/auto-ui-runtime/assets/` | 5：shell 52946 / switcher 9485 / notification_center 12653 / dashboard 8593 / desktop 35360 | InHouse | include_str（`src/ui/shell.rs:15-164`） | 桌面壳/切换器/通知中心/仪表盘/桌面 UI 声明 | ✅ 字节一致 |
| 5 | gallery 资产 | auto-ui-tools | `crates/auto-ui-tools/assets/gallery/` | 1（AppViewport.vue） | InHouse | RustEmbed `GalleryAssets`（`gallery_assets.rs:15`） | `materialize` → 生成工程 `src/gallery/` | ✅ 字节一致 |
| 6 | wm 资产 | auto-ui-tools | `crates/auto-ui-tools/assets/wm/` | 9 | InHouse | RustEmbed `WmAssets`（`wm_assets.rs`） | `materialize` → 生成工程 `src/wm/` | ✅ 字节一致 |
| 7 | wm 外部构建输入 | auto-ui-tools | `crates/auto-ui-tools/drawlist-renderer/src/` | 5（codec/connect/index/messages/render.ts） | InHouse | include_str（`wm_assets.rs:53-72` `REMOTE_RENDERER_FILES`） | wm materialize 的 remote-renderer 拷贝源 | ✅ 字节一致 |
| 8 | shadcn-vue 快照 | auto-ui-tools | `crates/auto-ui-tools/assets/shadcn-ui/` | 346（含组级 LICENSE+SNAPSHOT.md，二者在包内确认） | MIT | RustEmbed `ShadcnUiAssets`（`vue_shadcn.rs`） | 按组件 materialize → `src/components/ui/<组件>/`；本批 driver V05e 实测物化字节=包内源 | ✅ 字节一致 |
| 9 | 仓根级编译输入 | （越界 include 的锚定输入） | `stdlib/assembly-providers.json`、`stdlib/auto/image.{at,rs.at,vm.at}`、`docs/schema/aura.at` | 5 | InHouse | include_str 消费方：auto-lang-core `stdlib_assembly/{loader,manifest,providers}`、auto-ui-compiler `aura/schema_loader.rs:838`；随包经 `DELIVERY_ROOT_INPUTS`（rust_ui.rs:3955）显式拷贝 | stdlib 装配快照 / aura schema 种子 | ✅ 字节一致 |
| 10 | 仓根 Cargo.lock（stdlib 装配指纹输入） | auto-lang-core | （工作区根 `Cargo.lock`） | 1 | — | include_str（`stdlib_assembly` 指纹面，解析到包根锁） | stdlib 装配指纹；交付包根锁即其随包形态 | ✅（包根锁在位；内容=包自身解析锁，语义=装配输入在包内可解析） |

唯一源纪律（不变，沿子批2 §2 证据）：
`resources_manifest.rs::manifest_groups_are_registered_once_with_licenses`（三组
current_source 互异）+ `historic_auto_man_copies_stay_removed`（旧位不复现）+
`embed_consumption_matches_registered_sources`（embed 面=登记源，5/5）。

## 3. 与子批2 审计的关系

- 子批2 [resource-audit.md](../batch2/resource-audit.md) §1 表原漏第 1–4 行
  （auto-ui-runtime 域整体缺席），§3/§4 两断言错误——已在该文档原位纠正并
  留勘误链注记；handoff.md 来源清单已恢复 Inter 行并补壁纸/壳声明行。
- 本表为**完整**交付闭包生产 embed 盘点（子批2 表只覆盖 auto-ui-tools 域
  四组）；此后新增任何 `include_bytes!/include_str!/Embed` 生产面须同步
  本表与 `resources.rs` 登记（组级）/root 输入清单（根级）。
