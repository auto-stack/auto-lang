# PLAN-802 T-06 工作收据（送审冻结基线）

代码提交：`4029dd705` + R1 修复 `ad1427e33` + R2 格式 `b32d4df7d`（branch
`plan-802-dev`，base `1dd123361`，worktree
`D:/autostack/.wt/lang-802/auto-lang`，git status clean；R3 轮无 Rust 改动，
冻结代码仍为 `b32d4df7d`，实机二进制 SHA256 与证据一致）。auto-down 兄弟
`c39ac7f`（detached，只读消费）。本收据为 work 阶段证据汇总；merge-owned
门禁（裸 `cargo t`/`cargo tv`/触面档/tf 批量回归）按 2026-10-14 时序裁定
**保持 pending**，由最终 review 通过后的 merge 阶段在冻结提交上执行。

## R3 修复轮（F802-R3-01 闭合，2026-10-17）

**时序 oracle 重写（保留 ≤2s 冻结契约）**：

1. **预算 = 2.0s + u，u 仅含实测单次观察迭代耗时**（Reader tick/offset
   取同一 `scroll_state` 响应 + 端点经 press(read) 回执单跳采样——
   `state_changes` 仅含变更字段，无变更即沿用，零额外查询）。**通道刷新
   节拍不再计入**（R2 版混入 tick 间隔致 2.5s 迟到误放行）。
2. **u 受独立必验抖动门**（≤0.60s）：观察者停滞时判"本轮无法认证时序"
   （FAIL 重跑），不放宽预算。本次实测 u=0.534s。
3. **t0 取实际输入之前**（保守端，含前台化/点击/滚轮注入开销——测得值
   只会偏大）；端点一致性在计时循环**内**采样（user-wheel 端点与 Reader
   同迭代首见：均 1.273s ≤ 2.0+0.534=2.534s）。
4. **tick-progress 独立必验**：读回 tick 推进（停滞=通道冻结判 FAIL，
   本次 15→18）。
5. **五态 mock 自检先行**（`--selftest`）：0.5s 有效对照**通过**、
   2.5s 迟到对照**拒绝**（R3 负控复现修复）、tick 停滞**拒绝**、
   wrong_bound/missing_reader 维持**拒绝**——判定与实机共用同一
   `run_oracle` 路径。
6. **程序化臂分相时序**（不删除同帧验收要求）：入队回执 0.067s / Reader
   窗口首见 150 @0.078s / 读端点首见 150 @2.27s（≤采样预算 3.0s），两
   观察首变值均恰为命令值（无中间落点）；**同 drain 投影的源证据** =
   renderer.rs:23006（同一 drain 消费路径先 `note_controller_offset` 再
   安排 `__mcp_scroll`）——亚帧身份由源证据支撑，异步采样如实声明不测
   亚帧（测量边界）。

冻结基线实机重跑（`b32d4df7d` 构建，SHA256 一致）：**必验 12 项 + 辅助
1 项（共 13）全 PASS exit 0**（[证据 JSON](v06-live/v06-evidence.json.txt)）：
基线 0 / Reader 几何 8020×479.2 / 程序化分相 / clamp 偏差 0.200px /
抖动门 u=0.534s / tick 推进 / 滚轮双观察 1.273s≤2.534s / 端点一致
7241↔7241（窗内）/ 未绑定反控 / Reader 结构化必验。上轮收据"12+1"计数
有误（实为 12 必验+1 辅助），本轮已按 JSON 逐项核对。

## R2 修复轮（F802-R2-01/02，2026-10-17；时序预算缺陷由 R3 轮修正）

**F802-R2-02（格式）**：复审点名的四处新增文件（code_editor_provider.rs
118/285/298、plan802 测试 6 处、host_capabilities.rs 459、consumer fixture）
经 `rustfmt --edition 2021 --config skip_children=true` 全过（`b32d4df7d`；
renderer.rs 复审未点名，其预存格式差异不属本计划新增面，未整文件重排）。
格式化后定向回归 27/27 + fixture 7/7 PASS。

**F802-R2-01（驱动 oracle 判定缺陷）**：v06_driver.py 重写判定纪律并先跑
三态 mock 自检再上实机：

1. **clamp 上界按真实几何对拍**：上界 = Reader pane 实测 content_h −
   viewport_h（本次 8020.0 − 479.200 = 7540.800），|oy_max − 上界| ≤ 1.0px
   （实测偏差 0.200px）+ 复采稳定 + Reader 可见偏移同值对拍。错误上界
   （复审负控值 2000）必判 FAIL。
2. **ScrollStateReader 必验门禁化**：结构化解析（pane 按键命中、
   viewport/content/offset 数值齐全），失败进入非零退出；Reader↔读端点
   静止点一致性（±1.5px）为独立必验。截图降为显式辅助面（不门禁、不
   计入必验计数）。
3. **诚实描述与帧身份**：程序化收敛只声称「命令值收敛 + 采样窗内无中间
   落点」（命令后首个非零落点即 150；异步采样不证明同帧——R1 收据中
   "命令值直达"的夸大表述作废）；用户滚动计时改 **Reader pane 直读**
   （单跳 ~0.1-0.3s/轮；press+state 两跳叠加采样量化曾把通道节拍推过界，
   重跑实录）+ 观察节拍计量 + **tick 身份**：滚轮被**下一个读回拍**捕获
   （tick 22→23，无漏拍——收敛受单拍间隔约束），预算 = 通道 2s + 实测
   观察量子（1.243s），实测读回 1.336s。
4. **三态自检先行**（`--selftest`，mock 验证判定逻辑而非编辑器验收）：
   valid→PASS、wrong_bound→正确拒绝、missing_reader→正确拒绝
   （[自检证据](v06-live/v06-selftest-valid.json.txt) 等）。

冻结基线实机重跑（`b32d4df7d` 构建产物，SHA256 记录于证据 JSON 与
当前二进制逐字节一致）：必验 12 项 + 辅助 1 项全 PASS exit 0
（[证据 JSON](v06-live/v06-evidence.json.txt)）：基线 0 / Reader 几何
8020×479.2 / 程序化 150 无中间落点 / Reader 可见 150 / clamp 偏差
0.200px / Reader 可见 7541 / 观察节拍 / 滚轮下一拍捕获 1.336s /
端点一致 7241↔7241 / 未绑定反控 / Reader 结构化必验。

## R1 修复轮（F802-R1-01，2026-10-17；oracle 缺陷由 R2 轮修正）

V06 实机验证首次补齐（探针应用 + 驱动，MCP + Win32 SendInput 真滚轮，
真实 iced 窗口）。R2 轮修正判定纪律后重跑见上节；本轮历史价值：定位
**预存缺口**并修复——`render_dynamic_view` 的 code_editor 臂（VM 轨主
路径）自 PLAN-629 寄宿化以来未把 editor-scroll-{key} 绑进 scroll
controller 注册表（29480 desktop 泛型臂 701 起已绑），`code_editor_
scroll_to` 的 intent 在 VM 轨 pane 缺失被 drain 静默丢弃，写端点在 VM
实机窗口恒 no-op。**非 802 迁移引入**（旧 shim 直 enqueue 同一注册表同
队列）。修复 = 镜像 29480 的一行 `bind_controller(scroll_id, scroll_id)`
（`ad1427e33`）。修复后 701 供②b 端点合同在 VM 轨闭环。

探针资产注记（复现要点）：Init 先于首帧渲染（编辑器注册表空，set_text
no-editor raise——缺席臂逐字面值在实机再次验证）；内容装载走 value prop
数据流（`.doc` 模型 → build 期 renderer 侧 set_text，注册表保证在位）；
驱动 stdout 必须落文件（PIPE 不读灌满后阻塞 auto 进程首帧）；窗口前台化
需 ALT 键手法（直 SetForegroundWindow 被前台锁静默拒绝）。

## 一、22 native ↔ 19 provider 方法映射（T-01 矩阵实施核对）

见 [t01-frozen-matrix.md](t01-frozen-matrix.md)。实施逐项落地：cursor 三读
共用 `cursor()` 一次 DTO、scroll 双读共用 `scroll_offset()` 一次 DTO——与
迁移前 `code_editor_cursor`/`controller_snapshot` 单调用形态同构。负 offset
（9906）/1基→0基（2932）/负参钳 0（9910）均留 shim 侧（i32→usize 边界），
provider 收标准 usize。catalog/codegen/intrinsics 零改动（ID/canonical 名/
短名全部不变——fixture V02 ID 钉扎断言 22 项全对）。

## 二、验证记录（work 阶段定向面；命令与选择数）

| 命令 | 结果 |
|---|---|
| `cargo check -p auto-lang`（default=ui-iced） | PASS（411 告警=预存基线；本改动文件零新增告警） |
| `cargo check -p auto-lang --no-default-features` | PASS（no-ui 编译前提） |
| `cargo test --manifest-path test/fixtures/modularity/code-editor-provider-consumer/Cargo.toml` | **7/7 PASS**（V01 真VM全管线×2 / V02 缺席矩阵+身份+if_absent×3 / V03 delta FIFO+ownership×2） |
| `cargo t plan802` | **7/7 PASS**（core 剪断 / 装配身份 / 真注册表往返 / 文件 IO 字节保真 / provider swap 寿命 / 滚动投影链 / OS 剪贴板往返） |
| `cargo t code_editor_natives` | 3/3 PASS（既有 e2e 经新 provider 路径不变） |
| `cargo t plan701 plan670 plan794 plan785` | 25/25 PASS（含 785/794 源码扫描守卫） |
| `cargo t plan728` | 5/5 PASS（分页 editor load/edit/save 真注册表） |
| 合并终跑 `cargo t plan802 code_editor_natives plan794 plan785 plan728` | 27/27 PASS |

## 三、V06 证据（R1 修复轮后全量闭合）

~~条件缺口~~ **已闭合**：live iced 窗口消费链已实机验证（见顶部 R1 修复轮
表格——程序化命令值直达/clamp 精确上界/真滚轮 ≤2s 收敛/未绑定反控/reader
观测，8/8 PASS）。保留的组件级证据（真 controller 注册表投影链 + 真 arboard
剪贴板往返）不变，作为 headless 补充面。

## 四、V07 边界/剩余边/净删

- **777 口径**（text_scan.core_to_ui 同款 grep）：vm/native.rs `crate::ui`
  引用 **52→23**；vm/ 全目录 55→26。
- **本族生产边清零**：22 shim 零 `crate::ui` 直引（plan802_core_cut 五形态
  精确扫描守卫永久锚定）。
- **剩余边清单**（非目标，如实登记）：diff 族 9915–9917 四处（`code_editor::
  diff::envelope`，PLAN-703 面）+ native_dnd 一处（PLAN terminal/dnd 族）+
  测试模块引用（code_editor_natures e2e 布景 `ce::` 六处、794 scroll 测试
  九处——测试面非生产边）+ 注释两处。term/media/dialog/ffi 族未迁（与
  SD-02"其他族仍未迁"一致）。
- **a2r/merged 直接 UI 臂未误删**：ui_gen/rust.rs `auto_lang::ui::code_editor`
  引用 25 处原样；`git diff` 确认 ui_gen/** 零改动。
- **792 触面（闭包不变 metadata 证明）**：根 Cargo.toml 仅 exclude 登记
  （无 members/deps 变化）；crates/auto-lang/Cargo.toml 零改动；无新外部
  依赖（代码仅 std）；ui_gen/**（bundle 生成器）零改动→同输入生成物字节
  不变；792 probe（uiminimal）不消费 code_editor natives；default 与
  no-default 两特征组合编译均过。结论：792 交付闭包成员/manifest/生成器
  未变，按编排合同"闭包不变须 metadata 证明"路径处理，fresh/locked/重定位
  driver 重跑归批量回归/merge 档。

## 五、施工期发现（非本计划范围，已登记 KNOWN-DEBT）

**VM 单作用域局部变量索引 ≥16 加载失败**（预存限制）：第 17 个及以后的
`let` 局部变量经 LOAD_LOCAL 读回为垃圾值（纯 int、零 native 可复现：
17 局部 `print(v16)`→`0`）。V01 fixture 写族程序因此拆两段（每段 ≤13
局部）。与 802 迁移无关（worktree 基线复现、不涉 provider/shim 路径）。
KNOWN-DEBT 编号 P802-D1。

## 六、SD 增量素材（提案，merge 阶段发布）

按计划 §5 表：SD-01（vm/overview：editor 22 项 owned provider、缺席矩阵/
装配/寿命）、SD-02（ui/overview：adapter 单一 registry 与宿主装配，其他族
仍未迁）、SD-03（code-editor-session-endpoints：provider 后 VM/a2r 同源，
单位/IO/收敛不变）。本收据二/三/四节即 SD 证据素材。
