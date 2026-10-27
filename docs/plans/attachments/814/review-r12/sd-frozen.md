# PLAN-814 B 阶段规范提案（R8-F02 修复：可审定正文）

2026-10-21。stage: work（T-09 补齐——R8 needs_fix 后一次修复）。绑定实现
`780415083`+本修复批终端 SHA（见 §6）。每节：**before**（A 已发布版本
git-hash，主检出 master）/ **修改与新增规则** / **理由** / **AC**。
T-10 审定通过后由 T-11/T-12 受管落库（非本批直写 canonical）。

---

## SD-03 全文（新增 `docs/specs/auto-ui-runtime/project.md`）

**before**：不存在（A 期仅骨架 crate，无 spec）。**新增全文如下**（拟
落库正文，含 Status 头）：

> # auto-ui-runtime
>
> > **Status**: active（PLAN-814 B/T-04 交付；A 期 @ f6e07acee 为骨架）
> > 路径：`crates/auto-ui-runtime` | 技术栈：Rust
> >
> > PLAN-814 T-04：UI **运行/renderer/宿主域**独立成包——原 auto-lang
> > `src/ui/` 全树（iced renderer、desktop_protocol、code_editor、
> > terminal、mpv、native_dock/scroll/dynamic/session/shell/mcp_server、
> > interpreter、headless 等 ~20.4 万行）+ host_adapters + back_proxy +
> > `assets/*.at` 五件 + UI 动态构建管线（原 lib.rs Plan 015 glue 区=
> > `dynamic_build.rs`）单源迁入；旧路径经上层再导出全保持。
>
> ## 职责与边界
>
> - 依赖：auto-lang-core（SDK）+ auto-ui-compiler（UI 编译域）+
>   auto-val/auto-text-core/auto-terminal-core/auto-media-core/a2r-std
>   轻核；**不反依赖**旧 auto-lang 兼容包，不被 SDK/compiler 反依赖
>   （DAG：Compat→{SDK,UIC,UIR}、UIC→SDK、UIR→{SDK,UIC}；777 域=ui）。
> - 旧 auto-lang 对本包的依赖边 **default-features=false**（T-08 捕获
>   语义）：no-UI 消费者经该边不得拖入 ui-iced；本包自身 default=
>   ["ui-iced"]（独立消费=旧默认重后端语义保持）。
> - 777 `ui_to_language` 文本扫描以本包+compiler 为源根，基线 **0**=
>   零命中不变量围栏（禁 `auto_lang::` 引用；SDK 引用形态
>   `auto_lang_core::` 不在扫描内）。
>
> ## 公开能力与 feature 面
>
> - `ui` 门下 UI 核心抽象（Component/View/VTree 族）；`ui-interpreter`
>   （event_router/hot_reload/interpreter）；`ui-iced`（iced 后端+
>   renderer）；`ui-headless`；`image-pipeline`；`code-editor`（双
>   owner：SDK a2r diff 转发+本包 editor registry/provider）；
>   `highlight-treesitter`；`ui-clipboard`/`ui-dialog`/`native-dock`/
>   `native-clipboard`/`native-dnd`；`iced-layout-tests`；
>   `mpv-native/gpu/widget/spike`；`autodown`（跨仓 autodown-core，
>   optional）；`test-native-dock`。旧 auto-lang 同名 feature 全部
>   转发本包（组合面逐项保持）。**R9-F03 声明面组合全绿**：
>   no-default/bare-`ui`/`ui-headless`/`ui-dialog`/`code-editor`/
>   default 六形态 0 错（iced 专属面经 cfg 隔离——style_iced_adapter/
>   anchor_slot/sched_diag::trace/vm_bridge/i18n 钩子随实门；
>   PAYLOAD_SEP/encode_payload/ScrollStateEntry 纯项上移 mcp_types
>   单源+iced 侧再导出；log 依赖显式钉 std）。
> - 动态构建管线公开面（crate 根）：`build_dynamic_component`/
>   `has_ui_keywords`/`run_file_dynamic_ui{,_with_scenario}`/
>   `ui_module_parse_failures`（cfg 门=原 auto-lang 根项形态）。
> - 外部消费门：`test/fixtures/modularity/ui-runtime-consumer/`
>   （canonical 双直依赖=生成工程镜像形态，5/5+fresh+locked，闭包零
>   旧包）。
>
> ## 装配语义（单点+r3 登记权）
>
> - `install_ui_host_capabilities()`（pub，any(ui,ui-dialog) 门）=
>   provider 全族装配单点：frame/clipboard×2/scroll/terminal/
>   code-editor/dnd/dialog + autodown 引擎 + i18n lookup 钩子。
>   no-op 边界精确为 `not(any(ui, ui-dialog))`——bare ui/headless/
>   dialog-only 形态仍装配其 feature 内能力（R10 审定口径澄清，
>   非无宿主化）。
> - **不登记 UI 文法**：auto-ui-dialect 依赖与 `UiGrammarSource`/
>   `UiCompatInstaller` submit 权属旧 auto-lang 兼容包与
>   auto-ui-compiler（r3 §5.3）；本包零 inventory submit——纯 SDK
>   缺席/显式拒绝语义不受本包链接影响（T-08 缺席臂回归即违反本条）。
>
> ## 单源状态与生命周期
>
> - 进程级：native_widget GLOBAL 工厂、native_widget registry、mpv
>   RUNTIMES（UI 线程亲和）、mem_guard、UI console——随迁单源，不复制。
> - thread_local：theme 活动主题真源（style::theme）、`STORE_EXTRA_
>   FILES`/`USE_DIAGS` 带外通道（SDK 单源，本包经 SDK 面消费）。
> - FontSystem 多处 OnceLock 为预存局部双源（继承现状，不新增——
>   PLAN-814 合同"不改现状不新增"）。
> - 浮层几何提取（aura_view_builder）：直接消费 compiler StyleClass
>   IR 的纯扫描（headless 同轨），**position 族后继覆盖仲裁**与
>   IcedStyle::from_style 同语义（`absolute relative` 终态=relative
>   不 hoist；R10-F01 修复，overlay 探针五组合 mismatches=0）。
> - 退出/寿命：terminal/media/editor 生命周期、provider 缺席=冻结降级
>   臂、store drain、use 诊断跨包单源（V-04 族 91/91 回归面）。
>
> ## 资产与测试归属
>
> - `assets/{shell,desktop,switcher,notification_center,dashboard}.at`
>   单源本包（include_str 相对深度保持；compiler 测试经
>   `../auto-ui-runtime` 单源读）；Inter 字体/壁纸随 iced 资产。
> - 语料 `test/ui/*` 单源留 crates/auto-lang/test/ui（多消费者），
>   `test/parity/native` 同形；61+6 个 UI 域测试在 `src/ui_tests/`
>   （挂载门=原 tests.rs 逐项）；plan446 批一在 dynamic_build。
> - 测试选择面：t/tf/tt/tb/taa/ta/t3 七档含 `-p auto-ui-runtime`
>   （九档逐 ID 核销见 t07-closeout）。
>
> ## canonical import 与生成工程合同
>
> - Rust 生成物 canonical import：UI 面 `auto_ui_runtime::ui::*`、
>   语言/VM 面 `auto_lang_core::*`（策略单源=compiler
>   `ui_gen::import_policy`，新旧映射表在档；负控 pin：生成物零
>   `auto_lang::`）。
> - 生成工程 manifest 双直依赖：`auto-ui-runtime`（含 ui-iced）+
>   `auto-lang-core`；e2e 真编译门=compiler 两 harness（2/2）。
>
> ## 发行与兼容
>
> - 交付闭包 DELIVERY_CRATES 含本包；792 投影
>   `PROJECTION_AUTO_UI_RUNTIME`（跨仓 autodown 边剥除 owner 随迁）。
> - 旧兼容：`auto_lang::ui::*`/`auto_lang::back_proxy`/
>   `auto_lang::host_adapters` 全路径再导出（V-02 pin 锁面）；旧
>   feature 组合经转发保持；**旧样式路径** `ui::style::IcedStyle`/
>   `ui::style::iced_adapter::*`（R9-F02 恢复——style 门面模块内
>   compiler DSL glob+iced 适配器旧路径，同一类型单源=
>   style_iced_adapter，reviewer pin 编译过）。

**理由**：B 阶段 runtime 能力首次真实发布，需独立可审定规则；A 期
SD-03 明确留 B。**AC**：AC-03/04。

---

## SD-01 增量（`docs/specs/language-sdk/project.md`）

**before**：`1a313353484e`（A 发布）。**保留**：全部 A 规则（公开面/
闭包/UiGrammar 端口/缓存身份语义/inventory 具名例外——原文不动）。
**新增**：

1. `use_diag`/`take_use_diags` 下沉 SDK 单源（与 STORE_EXTRA_FILES
   同族 thread_local 带外通道；auto-lang 旧路径再导出）。
2. `test_code`/`test_code_with_deadline_and_input`/
   `test_code_with_path_deadline_and_input`/`test_file` 执行族下沉
   （纯 Core 面；宿主装配经本 crate 钩子分发器——无 UI 实现时 no-op）。
3. 非_UI 消费者基线升至四枚：auto-vm（A）+ auto-gen/auto-playground/
   a2r-actor-tests（B；闭包零 UI 零旧包）。

**理由**：T-06 下沉与消费者切换的规范事实。**AC**：AC-01。

## SD-02 增量（`docs/specs/auto-ui-compiler/project.md`）

**before**：`8695c3a9059b`。**保留**：A 全部规则。**新增**：

1. **import 路径策略单源**（`ui_gen::import_policy`）：canonical=
   runtime(UI)+SDK(Core) 双前缀；新旧映射表（view/code_editor/scroll/
   style/auto_val/action_config/frame_bench/dialog/iced→runtime::ui；
   vm/a2r_std→core）；负控 pin=生成物零 `auto_lang::`；vue/jet/ark 不
   在射面；**a2r trans 泳道发射明确不在本策略内**（消费者仍依赖旧包，
   后续批次）。
2. **生成器行为修复两处（具名反例）**：
   - truthy 过投射（PLAN-710 D-6 扩展；R9-F01/R10/R11 复修为完整类型感知）：
     dot 访问处于**值比较位**（前导/后随 `==`/`!=`/`<`/`>`）时不得
     truthy 包裹，且投影按真实类型分流——**形状已知走 shaped 访问器**
     （bool→as_bool/int→as_i64 as i32，循环形状语义保持）；形状未知按
     另一操作数分流：bool 字面量→as_bool/**整数（含负号）→as_i64**/
     **浮点→as_f64**（R10：AST 下形一元负号与数字可含空白 `- 1`，
     扫描跨空收数字；浮点不截断成整数）/默认 str；**R11（R9-01 收口）：
     已声明状态引用（`.limit`/`.allowed`/`.threshold` 形）按生成器既有
     state_types 声明环境分流**——int(i32)/f64/bool 与同名字面量域一致，
     **非默认声明宽度（语言 `float`=f32、u32/i64/u64 显式声明）投影跟随
     声明宽度**（`as_f64() as f32`/`as_i64() as <宽度>`，否则与声明侧
     E0308）；**透明分组括号不改变操作数类型**（两侧 `( 0.5 )`/
     `(.limit)` 嵌套与内部空白跨过；分组不得作为回落 str 的依据）；
     String/Value/未声明引用不命中声明分流，保持默认 str 语义（反例
     `p.running == .wanted` 字符串状态比较不变）。反例集（r9+r10+r11
     全集）：`p.running == "1"`（str）、`p.dirty == true`（bool）、
     `p.rank > 3`/`p.rank > -1`（int 含负）、`p.score > 0.5`（float）、
     RHS 位 `true == p.dirty`/`-1 < p.rank`/`0.5 < p.score` 同形、
     Value prop 形（`note.rank/note.score`）同分流、声明操作数
     `p.rank > .limit`（int）/`.limit < p.rank`/`p.dirty == .allowed`
     （bool 两侧）/`p.score > .threshold`（float=f32）/i64 宽度、分组
     `p.score > (0.5)`/`(0.5) < p.score`/`p.rank > (.limit)`/
     `(.threshold) < p.score` 及 prop 分组形。loop/prop×左右位×
     字面量/声明/分组代表组合经 r11 探针 26 feature+正控真实编译
     全 exit0（[r11-closeout](r11-closeout/)）。
   - on_click 单元兜底：点击族消息承载事件（onclick/oncontextmenu
     [.prevent]→on_right_click/ondoubleclick）在场时跳过 `on_click
     (|_| ())` 兜底。反例：shell 语料按钮带 `oncontextmenu.prevent`，
     原发射追加单元兜底 → E0308（expected DesktopMsg, found ()）。
   两者均为**预存生成器缺陷**（shell-pack 再生暴露，非迁移引入）；tu
   870/870 全量金样保持。
3. e2e harness manifest 合同：双直依赖（runtime ui-iced+SDK）真编译
   门（2/2，#[ignore] 档）。

**理由**：T-05 canonical 化与 T-08 修复的可审定语义。**AC**：AC-03。

## SD-04 增量（`docs/specs/auto-lang/project.md`）

**before**：`7416ca83b4e9`。**保留**：A 全部（兼容/组合层定位、
inventory 具名例外）。**新增**：

1. →auto-ui-runtime 依赖边 **default-features=false**（T-08 捕获）：
   no-UI 消费者（default-features=false 形态）闭包禁拖 ui-iced；六
   provider fixture=缺席臂哨兵（43 测）。
2. 测试期 UI 依赖清零（T-04 临时 dev-deps 兜住形态收口；残留 UI 测试
   仅经再导出消费=V-02 兼容面）。
3. `extract_autovm_result` 再导出恢复（T-02 丢失公开路径勘误）。

**AC**：AC-01/02。

## SD-08 增量（`docs/specs/repository-boundaries/generated-ui-distribution.md`）

**before**：`3e1beae817cc`。**保留**：A 全部（19 包闭包/分 spec 投影/
指纹路径）。**新增**：

1. 生成工程（app/成员/workspace 三 manifest 模板）canonical 双直依赖
   （runtime ui-iced+SDK；feature 转发随 runtime）。
2. 投影 spec 随包图重构：PROJECTION_AUTO_LANG 收窄（test-case 两项）；
   新增 PROJECTION_AUTO_UI_RUNTIME（autodown 边剥除 owner=runtime；
   receipt 增 projection_edits_auto_ui_runtime 台账）。
3. 792 driver V04 断言随 canonical；24/24 EXIT=0 为 B 基线。

**AC**：AC-05。

（SD-05/06/07：B 无结构变化——frontend/vm/ui overview 的 A 版
`81375f38ec3f`/`f270359de0bd`/`fe811ba53e33` 增补 B 事实注记即可，
规则无 before/after 差异，随 T-11 沉淀时附注。）

---

## 6. 修复批身份

- R8 三项修复批：健康（R8-03）＝本批提交 A；SD 正文＝本文件；
  V-03 见 handoff 更新版 §3。终端 SHA 以提交链为准（handoff §1 追加）。
- frontmatter 影响元数据（supersedes/new/touched）无需变更——B 未
  改 SD 目标集合本身，仅充实正文。
