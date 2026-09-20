# PLAN-668 T-01 基线勘定（baseline.md）

- 采集环境：worktree `D:/autostack/.wt/lang-668/auto-lang`（plan-668-dev），HEAD `27c196c04`（==master，2026-09-20）。
- 采集命令（全部 `--no-fail-fast`，本仓 nextest 0.9.138 默认 fail-fast 会截停——方法论注记）：
  `cargo t` / `cargo tv` / `cargo tt` / `cargo nextest run -p auto-lang --lib --features ui-iced`
  / `cargo taa test_aavm2_goldens_check` / `cargo nextest run -p auto-lang --test desktop_behavior --features ui-iced,ui-interpreter`
  / `cargo nextest run -p auto-lang --test ui_snapshots` + 7 项隔离复测（滤串级单跑）。
  原始日志：`.wt/lang-668/logs/{t,tv,tt,uiiced,taa_goldens,desktop_behavior,ui_snapshots,iso_*}.log`（组目录，不随仓提交）。

## 1. 各档红签名全集（实跑）

| 档 | 跑量 | 红数 | 签名 |
|---|---|---|---|
| `cargo t`（daily，ui-iced lib+3 集成） | 5320 | 37 | 见 §2 逐条（含 docs_gen/schema_drift **全绿**） |
| `cargo tv`（test-vm-files） | 3816 | 2 | R-01/R-02（ui_gen 两件；display_family **绿**） |
| `cargo tt`（test-trans） | 4038 | 6 | R-01/R-02 + R-12 四件（007_shared_var/27_c_abi 003/004/rustc 实编门） |
| ui-iced 裸 lib | 5320-3 集成 | 37 | 与 daily 36 重合 + `clipboard_files_set_get_roundtrip`（隔离绿→环境红） |
| `cargo taa test_aavm2_goldens_check` | 1 | 1（4 金样） | b13/b32/g25/b42 expected.rs 漂移（台账在册 2 件→实跑 4 件，同族括号漂移） |
| desktop_behavior（ui-iced,ui-interpreter） | 11 | 1 | plan488_dnd_bridge_app_handlers（R-17） |
| ui_snapshots | 3 | 3 | snapshot_app/editor/sidebar（台账在册 2 件→实跑 3 件，app 同款过期） |

## 2. R-01..R-26 逐条四态分类（26/26 覆盖）

四态：**修复**（并入 T-xx）｜**核销-已不红**｜**环境红豁免**｜**域外转介**。

| # | 实跑状态与签名 | 分类 |
|---|---|---|
| R-01 | tv/tt/daily 三档红：`mouse_area_emits_events_and_logical_extent` 断言 `PointerMoveHandler::new(...)` 未发射（rust.rs:10256） | 修复 T-02 |
| R-02 | tv/tt/daily 红：`test_autodown_panel_heading_codegen`（rust.rs:8155 静态样式串断言）；同族 aura 面 `test_autodown_panel_heading`（aura_view_builder.rs:14146）多 `Tracking(-0.025)` token——heading 样式发射已带 tracking，两断言过期 | 修复 T-02（断言跟语义） |
| R-03 | daily 红：`lucide_icon_coverage_manifest_all_hit`——`crates/auto-lang/assets/shell.at` 8 个 `iconfile:` scheme 位图钮（2026-09-15 用户裁定）被 lucide 清单走查收进命中断言；iconfile 是位图通道非 lucide 命名空间 | 修复 T-02（走查过滤 `iconfile:` 前缀） |
| R-04 | daily 红：`conditional_style_with_comments_in_branches_resolves_hover`——**测试体内联源码**解析失败（duplicate `view` block + 20 RBrace 错，offset 489）：注释进条件分支的语法面测试源过期 | 修复 T-02（测试源跟现行语法） |
| R-05 | daily 红：`p010_popover_ondismiss_extracted_from_events`——5 枚 popover 第 3 枚落 `__popover_close` 兜底 `[MenuClose,MenuClose,__popover_close,BlankClose,PickerDismiss]`（=027 在册拖拽幽灵 popover 判 widget 形态） | 修复 T-05（convert_popover 坐标锚判定） |
| R-06 | daily 红 + **隔离单跑仍红**（P667"单跑绿"口径失效）：`external_config_poll_hot_apply_loopsafe` ③防回环断言 dark_theme left(false)/right(true)。机制：②外写应用后 theme_source 归一化回 `system`，③ save→load() 从 **OS 主题**派生 dark_theme——OS=light 机器必红（P667 期观察机器 OS=dark 恰绿）。测试意图=防回环非主题跟随，OS 依赖是测试缺陷 | 修复 T-05（③保存前显式 manual 源；非豁免——可确定性根修） |
| R-07 | **不在红集**：d8_toggle_dark_mode 日常档绿（015 默认翻 true 断言已被顺修） | 核销-已不红 |
| R-08 | daily 红：`c2_param_msg_declaration_both_tracks_alive`——vue SFC 未发射 pointer-move 归一坐标 arrow（plan492_m4_tests.rs:130）；与 R-01 同族嫌疑（mouse-area 事件发射，vue/rust 两臂） | 修复 T-03（与 R-01 同勘） |
| R-09 | daily 红：`strips_tags_and_decodes_entities` 双空格 `" @x  你好"` vs `" @x 你好"`。裁定：strip_html_tags 职责=去标签+实体解码，不重排空白（改动运行时行为波及 aura 全消费面）；期望跟现行为 | 修复 T-03（断言跟语义，裁定注记） |
| R-10 | 与 R-03 同一测试（`lucide manifest` 564-Q6 与 P661-D7 两次发现同物） | 合并入 R-03 |
| R-11 | daily 红 15 件：ui::layout 全族（grid×7/master_stack×4/snap×2/usable_rect/apply_layout_filters），全部 panic 于 layout.rs:371 同一断言（本机显示几何）。576 在案"数量随会话 6↔9↔14 浮动"，本次 15 | 环境红豁免（CI 为准；本机浮动记录在案） |
| R-12 | tt 红四件：①`007_shared_var` 仅括号漂移 `(*G.lock().unwrap())` vs `*G.lock().unwrap()`——trans/rust.rs:2754 **PLAN-018 有意修复**（括号绑定解引用范围防 E0614），金样陈旧→bless；②③`27_c_abi_003/004` 发射丢 S/D-form 包装面（use.c 清单回退成裸 extern，rustc 门 E0425 同根）→根因修；④`a2r_rustc_real_compile_gate` 3 unexpected=003（同②）+`024_nested_async_await` 解析错（`.await` 点后 Await token，parser 缺口） | 修复 T-04（bless+根因修） |
| R-13 | taa 红：4 金样 b13/b32/g25/b42 expected.rs——与 R-12① 同根（PLAN-018 括号化后金样未再生） | 修复 T-04（A2R_BLESS=1 再生+人工核验） |
| R-14 | daily 红：`covered_elements_within_target_set`——element_coverage.rs:43 登记 imagesurface=Covered 但 `Coverage::target_set()` kinds 无 imagesurface 键（normalize 不折叠） | 修复 T-05（投影器臂或登记降级，T-05 内勘定） |
| R-15 | **双态皆绿**：tv（无 ui-iced）与 daily（有 ui-iced）均 PASS `test_display_family_codegen_arm_fixture`——P649 期特性面差异已被后续 feature 演化消解 | 核销-已不红（终验双态复核） |
| R-16 | daily 红：`test_029_photo_gallery_thumbnails...` data-URL 断言（P642-D11：语料已演进为路径引用+渲染端内嵌，断言过时） | 修复 T-05（断言跟现契约） |
| R-17 | desktop_behavior 红（其余 10 绿）：`plan488_dnd_bridge_app_handlers`——**根因勘定**：dnd-bridge app.at `use stylekit.styles: caption_text` 导入名未经 `prepare_style_recipe_imports` 预注册即裸 parse（load_inline 直连 Parser），`style: caption_text` → UndefinedVariable + 恢复雪崩 20 RBrace 错。真实管线（auto run/build）走预注册不受影响——测试装载器缺陷，非解析器回归 | 修复 T-05（测试装载器走真实管线） |
| R-18 | ui_snapshots **3** 红（app/editor/sidebar，台账 2 件+app 同款）：015-notes SFC 字节漂移未随改动刷新 | 修复 T-04（insta accept+diff 人工核验） |
| R-19 | daily 档 docs_gen `kitchen_sink_page_in_sync` + schema_drift `schema_drift_fence` **双绿**（P528-D6 期 tf 档口径） | 核销-已不红（T-09 tf 全量终验确认） |
| R-20 | 待复跑（playwright）：根因已勘定——`website/public/ui/gallery/index.html` title=`widgets-gallery`（auto-os/widgets-gallery 应用名，PLAN-590 迁移后快照），其余三资产 title 已是 `Auto Language - X`；spec 期望 `Auto Language - Components` 是迁移前旧资产名 | 修复 T-05（spec 断言跟现资产名；auto-os 域不动） |
| R-21 | 构建链（本批 T-06 复跑勘定）：`generate_tsconfig()`（auto-man vue.rs:834）无 `types`；gen/ 产物不入 git、build 自愈再生 → 模板修即全域绿 | 修复 T-06（SD-01） |
| R-22 | 构建链：`write_auto_sources_ts` 仅 run 路径（vue.rs:5595/5622）调用，build_vue_project 缺位 → overlay.ts TS2307 | 修复 T-06（build 前补写；SD-02） |
| R-23 | 构建链：038 store 名未解析（P666-D2，codegen 演化滞后）——T-06 实跑勘定 | 修复 T-06 |
| R-24 | 构建链：017 db.at 种子字面量 &str→String 槽 E0308（api_gen.rs）；025 29 错——T-07 实跑分族 | 修复 T-07 |
| R-25 | daily 红 4 件复测在案：p053_1×2/p053_4/p053_6（==P028-D4 家族，master 签名一致）；另有 p054 新观测 2 件（见 §3 A-08） | 域外转介（musk；R-25 只呈报不修） |
| R-26 | `cargo metadata` 清单解析通过（worktree+组内 auto-down 兄弟位）；master 主检出 `crates/auto-lang/Cargo.toml:132` 已指 `packages/engine/rust`（P551-D2 期 path 失效已被随行修复） | 核销-已不红 |

## 3. 计划外新增观测（T-01 实跑发现，随批处置入册）

| # | 档/状态 | 签名 | 分类 |
|---|---|---|---|
| A-01 | daily 红（隔离未复测——与 R-17 同签名高置信） | `plan339 test_016_calendar_app_compiles_with_namespace`：caption_text UndefinedVariable offset 8532 + 20 错——**R-17 同根**（stylekit 导入预注册缺位；该测试在 daily 档，装载器同病） | 修复 T-05（随 R-17） |
| A-02 | daily 红，隔离仍红 | `plan370 d10_edit_fills_draft_fields`：EditTitle 后 draft_title 停留 "Welcome"——015 EditorPanel 事件链断（与 R-06 同为"既往闪测现确定性红"族） | 修复 T-05 |
| A-03 | daily 红，隔离仍红 | `plan632 f2_embedded_dep_component_expands_with_props`：open=true 面板内容未展开（空 Column） | 修复 T-05 |
| A-04 | daily 红，隔离仍红 | `plan640 t04_vm_track_empty_state_reference`：VM 链接失败 `Undefined symbol: on_primary` | 修复 T-05 |
| A-05 | daily 红，隔离仍红 | `plan050_imported_component_icon_maps_to_lucide_image`：期望 View::Image 得非 Image 视图（icon 映射漂移，R-03 邻族） | 修复 T-05 |
| A-06 | daily 红，隔离仍红 | `plan626_literal_interpolation_multi_segment_dot_path`：`${.store.line}:${.store.col}` 多段插值未解析停留字面量 | 修复 T-05 |
| A-07 | ui-iced 裸档红，**隔离绿** | `clipboard_files_set_get_roundtrip`（剪贴板原生面，并行争用） | 环境红豁免（CI 为准） |
| A-08 | daily 红 2 件 | musk p054_t1（lucide:plus 进 button 内容子树）/p054_t4（icon image class prop）——musk 域 | 域外转介（musk，随 R-25 呈报） |

## 4. 分类汇总

- 修复并入 T-xx：R-01..R-06、R-08、R-09、R-12、R-13、R-14、R-16..R-18、R-20..R-24、A-01..A-06（其中 R-07/R-10/R-15/R-19/R-26 五件核销，R-03=R-10 合并）
- 核销-已不红：R-07（d8 断言已顺修）、R-10（并入 R-03）、R-15（display_family 双态绿）、R-19（docs_gen/schema_drift 双绿，tf 终验确认）、R-26（路径依赖已修）
- 环境红豁免：R-11（ui::layout 15 件，本机几何，CI 为准）、A-07（clipboard，隔离绿）
- 域外转介：R-25（musk p053 ×4 呈报）+ A-08（musk p054 ×2 呈报）

## 5. 方法论注记（复现口径）

1. 本仓 nextest 0.9.138 **默认 fail-fast**——任何基线采集/门禁跑必须显式 `--no-fail-fast`，否则首个已知红截停全档（本次 t 档曾被 musk p053 截停在 820/5320）。
2. desktop_behavior 集成档需 `--features ui-iced,ui-interpreter`（文件级 `#![cfg(feature = "ui-interpreter")]`），裸跑报 "no tests to run"。
3. R-06 的 OS 主题依赖机制：外写应用后 theme_source 归一化回 system，load() 从 OS 派生 dark_theme——同型测试在 OS=dark 机器绿、OS=light 机器必红（P667"单跑绿"观察的环境真相）。
4. aavm 金样再生：`A2R_BLESS=1 cargo taa test_aavm2_goldens_check`（bless 后 git diff 人工核验）；a2r text golden：失配自动落 `.wrong.rs`，人工核验后覆写 expected.rs。
