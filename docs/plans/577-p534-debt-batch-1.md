---
plan_id: PLAN-577
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: P534 债务清偿批一期（avatar 家族 + schema 滞留 + breadcrumb 栈溢出）
author: [zhaopuming, ZCode]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径
current_step: 0
total_steps: 9
---

# [PLAN-577] P534 债务清偿批一期——avatar 家族补齐 + schema 再生成滞留清偿 + breadcrumb 栈溢出归因修复

## 变更摘要

PLAN-534 执行/复审期登记的 P534-D1..D6 中,三个可独立交付、互不阻塞的项组成
本批（债务批先例:505 一期/515 二期）:

1. **P534-D4 avatar 家族补齐**:`convert_avatar` 现仅渲染裸 avatar 标签灰圆
   占位,`avatar-image`/`avatar-fallback` 零臂（unknown fallback）,子件整体
   弃置——gallery /hovercard 页触发器 hit area 零高（534 实机 hover 只能借
   语料工程完成）、/avatar 页缺图。
2. **schema 再生成滞留清偿**:534 做最小补丁时暴露的全量再生成滞留——
   sidebar 族 7 元素+NavDestination/Swiper 未入 element_coverage 登记、
   P3 `sidebar_menu_sub_button` 档位不一致（静态 full vs schema partial）。
   谁下次跑 `SCHEMA_DRIFT_GENERATE_AT=1` 必撞围栏红。
3. **P534-D5 breadcrumb 栈溢出归因修复**:534 全站扫描三次确定性死于
   Tooltip→Breadcrumb 边界。复审期探针已定性=**页本体即崩**（新会话直达
   Breadcrumb 必现,与访问序列无关）,即 **P530-D1 已在案存量**（"导航到
   breadcrumb 页必现 overflow,疑 ForLoop/嵌套 link 链,debug 构建抓栈
   bisect"）——本批偿还之。

不在本批（维持债务）:P534-D1（VM 定时器原语/delay）、D2（拖拽手势）、
D3（vue 基线运行对照,依赖 npm 环境建设）、MCP overlay 注入通道
（P533-D2 扩展）。

## 目标

1. avatar 家族:`avatar`（有子件）渲染子件组合,`avatar-image`（src→图）、
   `avatar-fallback`（→文本）独立臂;/avatar 页观感补齐。
2. hovercard 触发器在 gallery 页**直接**可 hover（hit area 非零）,534 验收
   G3 的 gallery 形态补全,不再借语料工程。
3. schema 全量再生成入账:滞留 12 元素登记,P3 档位对齐,三围栏绿,无新增
   豁免争议。
4. breadcrumb 页可直接导航进入（不崩）,全站 68 页扫描一遍全绿。
5. KNOWN-DEBT:P534-D4/D5 结案回写,D5 关联 P530-D1 一并结算。

## 架构方案

三件独立可交付,共享 534 已验证的流程资产:

| 件 | 落点 | 复用 |
|---|---|---|
| avatar 家族 | aura_view_builder 双镜像臂+四表同步 | convert_image_or_icon（img/image/icon 先例）、534 四表流程（schema.rs+aura.at+render_support+baseline） |
| schema 滞留 | element_coverage.rs+SCHEMA_DRIFT_GENERATE_AT 全量再生成 | 534 围栏清偿流程（P533 dialog 同型） |
| breadcrumb 溢出 | 病源定位（页面内容二分+debug 构建）→修复 | P530-D1 归因留档（scratch/p530/） |

## 技术栈

Rust（auto-lang crate）+ AutoLang .at（gallery 页面）+ 既有测试基建
（headless iced_test/plan 探针/autoui-verifier MCP）。

## 需求分析与背景调查

（2026-09-07 对码+实机探针核实）

- **avatar 现状**:`convert_avatar`（aura_view_builder.rs,untracked :1842/
  tracked :3405 两臂）只产灰圆占位容器,**子件参数被丢弃**;
  `avatar-image`/`avatar-fallback` 全仓零臂。`img|image|icon` →
  `convert_image_or_icon`（src prop 图源）可直接复用;537 已实证 image
  fit cover/contain 双端。
- **schema 滞留清单**（534 重生成实测暴露）:NavDestination、Swiper、
  sidebar-group-action、sidebar-inset、sidebar-menu-action、
  sidebar-menu-badge、sidebar-menu-sub、sidebar-menu-sub-item、
  sidebar-separator、sidebarinput、sidebarmenuskeleton、sidebarrail
  （12 元素未入 element_coverage）;P3 `sidebar_menu_sub_button`
  render_support 静态=full vs schema=partial。
- **breadcrumb 溢出三事实**（534 复审期探针,scratch/p534_review/）:
  ①三次全站扫描确定性死于 Tooltip→Breadcrumb 边界;②死亡前快照指标平坦
  （无累积,同页 96 次循环零崩溃零增长）;③**新会话直达 Breadcrumb 即崩**
  （与序列无关）。P530-D1 在案先验:master@96586cca 对照构建同复现,疑
  ForLoop/嵌套 link 链。breadcrumb 家族五标签全仓零臂（全 unknown
  fallback）,页面 63 行有限树——病源疑在页内特定构造形态（breadcrumb-link
  带 href prop？unknown tag+特定 prop 组合触发的通用路径递归）,执行期
  二分定位。
- **边界**:534-D3/D1/D2/MCP 注入不在本批;avatar 圆形裁剪受 VM 无 clip
  原语限制（见待澄清③）。

## 详细设计

### D1 avatar 家族臂（双镜像）

- `avatar`（有子件）:容器（rounded-full bg-muted+尺寸类）+子件转换
  （avatar-image/avatar-fallback）；无子件保持现灰圆占位。
- `avatar-image`:复用 `convert_image_or_icon`（src prop 同 img 形态;
  网络图源加载失败/未载时的占位沿 image 既有语义）。
- `avatar-fallback`:text 臂（children text/text prop → convert_text_element,
  居中类注入）。
- 双镜像（untracked+tracked）接线两处分发区,镜像 534 sheet/drawer 臂模式。
- **四表同步**（复用 534 流程）:新拼写入 schema.rs elements+aura.at
  （avatar-image/avatar-fallback backends.iced 回填 native）+render_support
  +schema_drift_baseline。

### D2 schema 滞留清偿

- element_coverage.rs 登记 12 元素（NotConsumed"shadcn web 长尾"注释,
  对齐 drawer_close 行风格）。
- `SCHEMA_DRIFT_GENERATE_AT=1` 全量再生成→aura.at 差异复核（对照 534 期
  /tmp 重生成产物已知形态）→P3 `sidebar_menu_sub_button` 档位对齐
  （先诊断 render_support 静态表与 schema backends.iced 哪边为意图值,
  对齐另一边）→baseline 裁剪（漂移消除项）人工复核。
- 三围栏验收:schema_drift+queue_coverage+docs_gen。

### D3 breadcrumb 溢出（P530-D1 偿还）

- **T 定位**（二分法,每步留痕 scratch/p577/）:
  a. debug 构建直接导航 breadcrumb 页固定复现;
  b. 页面内容二分:preview-card 外壳/table 块/各 breadcrumb-* 子块逐块
     注释（页面 63 行,块级 ≤6 步）——锁定病源构造;
  c. 若 a/b 指向通用路径（疑:unknown tag+href prop→link/路由形态递归）,
     以最小 .at 复现（≤10 行）固定病源。
- **T 修复**（按病源二选一,预设出口）:
  - 页面构造病源（如特定 prop 组合触发通用路径环）→builder/解析路径修复
    （环检测或深度防御）+最小复现回归测试;
  - 深层运行时缺陷（修复面超出本批）→归因报告（scratch/p577/+最小复现
    入语料）+独立计划立项,KNOWN-DEBT P534-D5 标"归因在案,偿还另立"。
- 全站扫描回归:68/68 页一遍全绿（breadcrumb 可进,无崩溃尾部）。

## 测试设计

（作用域:Category B 局部 ui 改动;`cargo t iced` 局部+`cargo tv`;fold 前
pre-fold `cargo tf`;不触 aavm 路径零 taa 触发）

- avatar:builder 探针（children 渲染/无子件占位保持/fallback 文本可见/
  hit area 非零）+layout_tests 或 plan 探针文件;实机 /avatar+/hovercard
  截图（autoui-verifier MCP）。
- schema:三围栏（schema_drift_fence/queue_coverage_drift_fence/
  docs_coverage_fence）。
- breadcrumb:最小复现回归测试（病源构造 .at→build 不崩）;全站扫描脚本
  复用 scratch/p534_gallery_scan.py（68/68 全绿判据）。
- 双端:avatar vue 轨对照（shadcn Avatar 语义——图源/回退文本），537 image
  先例口径。

## 验收标准

- [ ] /avatar 页 avatar-image/avatar-fallback 渲染（截图落账 scratch/p577/）。
- [ ] /hovercard 页 avatar 触发器 hit area 非零,真 hover 进/出**直接通过**
      （不借语料工程;534 G3 gallery 形态补全）。
- [ ] schema 三围栏绿;滞留 12 元素登记;全量再生成入账无未审豁免。
- [ ] breadcrumb 页可直接导航进入不崩;全站扫描 68/68 全绿。
- [ ] KNOWN-DEBT P534-D4/D5+P530-D1 结案回写（D5 若走"归因报告+另立"出口,
      本项改判该出口产物在案）。
- [ ] `cargo t iced`+`cargo tv` 全绿（唯一红允许=charts gallery 存量）;
      pre-fold `cargo tf` 与基线一致。

## 执行步骤

（原子任务;每步完成后追加 [✅ 已完成] 一行证据。行号为 2026-09-07
master 基点,执行时以 grep 重新定位。worktree=`D:/autostack/.wt/lang-577/
auto-lang`;依赖仓不涉及。）

1. [ ] **avatar-image/fallback 臂**:aura_view_builder.rs 两处分发区
   （grep `"avatar" =>` 定位 :1842/:3405 邻域）加
   `"avatar-image" => convert_image_or_icon(props)` 与
   `"avatar-fallback"` text 臂;`convert_avatar` 改造:有子件→容器+子件
   转换（untracked+tracked 双镜像）。验证:`cargo check -p auto-lang` 零错。
2. [ ] **四表同步**:avatar-image/avatar-fallback 入 schema.rs elements
   （grep `elements.insert("sheet"` 邻域先例）+schema/aura.at
   （avatar-image/avatar-fallback 元素,backends.iced 回填）+render_support
   +schema_drift_baseline（SCHEMA_DRIFT_UPDATE_BASELINE=1 后人工复核 diff,
   只允许 avatar 拼写维度）。验证:三围栏绿。
3. [ ] **avatar 探针**:plan577 探针模块或 layout_tests 追加——avatar
   （有子件）渲染 avatar-image/avatar-fallback（fallback 文本可见,hit
   area 非零）;无子件占位保持（回归）。验证:`cargo t` 滤 plan577/新测全绿。
4. [ ] **avatar 实机**:gallery /avatar 页+/hovercard 页截图
   （scratch/p577/落账）;hovercard 触发器 hit area 非零（bounds 快照
   @rect h>0）→真 hover 进/出 state 翻转（复用 534 sweep 法）。
   验证:截图+state 证据在案。
5. [ ] **element_coverage 登记**:12 元素 NotConsumed 行（对齐 drawer_close
   风格）。验证:`cargo test -p auto-lang --test schema_drift
   queue_coverage_drift_fence` 绿。
6. [ ] **schema 全量再生成**:SCHEMA_DRIFT_GENERATE_AT=1→aura.at diff
   人工复核→P3 sidebar_menu_sub_button 档位诊断+对齐→baseline 裁剪
   复核。验证:schema_drift 2/2+docs_gen 4/4 绿。
7. [ ] **breadcrumb 病源二分**:debug 构建直达 breadcrumb 复现→页面内容
   逐块注释二分（≤6 步,scratch/p577/ 留痕）→锁定病源构造→最小 .at
   复现（≤10 行）固定。验证:最小复现 100% 复现溢出。
8. [ ] **breadcrumb 修复**:按病源修（环检测/深度防御/构造修正,预设出口
   见 D3）+最小复现回归测试。验证:最小复现不崩;直达 breadcrumb 页
   ALIVE（MCP snapshot 正常返回）。
9. [ ] **收口簿记**:KNOWN-DEBT P534-D4/D5+P530-D1 结案回写（D5 走
   "归因另立"出口时改记归因在案）;gallery README 已知边界 avatar 行更新;
   全站扫描 68/68 判据复跑;pre-fold `cargo tf` 与基线一致。
   验证:KNOWN-DEBT 三行在案+tf 唯一红=charts 存量。

## 复审记录

（待 /auto-plan:review 填写。）

## 待澄清事项

1. **P534-D3（vue 基线运行对照）与 MCP overlay 注入通道不在本批**——
   前者依赖 npm 环境建设,后者偏基础设施,均维持债务;若用户希望并入,
   本批扩为二期再议。
2. **breadcrumb 修复出口边界**:病源若在深层运行时（如 link 路由解析环
   牵动 router 全链）,浅修（页面构造规避）与根治的取舍在 T7 定位后裁定,
   超边界走"归因报告+独立计划"预设出口（验收第 5 条已含该出口形态）。
3. **avatar 圆形裁剪**:VM 无 clip 原语,圆形以 bg-muted+rounded-full+尺寸
   类近似（528 期 VM 形状能力既有口径）,不做真圆像素裁剪;图源加载失败
   占位沿 image 既有语义。
