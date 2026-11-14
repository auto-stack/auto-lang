# ASE M0 就绪记录（基线与公共接口准入）

> 来源：PLAN-1020（2026-11-14 交付）。本文件是 ASE M0-BASE 的现状记录：
> 现有 Gallery 真实路径/成本基线、双臂拓扑事实、M0-H/T/I 各切片的公共接口
> 准入证据与缺口。未交付能力一律 pending；本文件不宣称浏览器/新引擎产品运行成功。

## 1. 现状基线（M0-BASE，2026-11-14 实测）

环境：Windows x64（本机）；工具 auto 0.1.0+plan-890-verified-r7-t19-20-g347c29f-dirty
（2026-11-11 构建，auto-man HEAD=54fe407 前四提交）；node v25.2.1 / pnpm 11.6.0 /
playwright 1.62.1（自动注入，非人工输入）。测量树：`D:/autostack/.wt/lang-1020/`
（auto-lang@10a3dd46e + auto-os 兄弟树钉 868c80d3）；语料 auto-ui 主检出
examples/ui（HEAD a6f776c，只读）。

### 1.1 Vue 臂（`auto run`）

- 全新冷构建（C1，原始工具链）：**失败**。根因链见 §3 工具链断点 D1/D2；可复现
  命令与退出码在 PLAN-1020 §9。主检出既有 gen 为旧版生成器产物（apps/*.vue 仅
  46 行、无主题接线），不能当作当前工具链可运行的证据。
- 测量态（§3 workaround 后，D1/D2/D3 不计入对照）：外壳冷启动（清 gallery 行缓存）
  cmd→HTTP ready ≈ 78–108 s，其中 **~70 s 为 gallery 35-demo 逐个 `VueProject::
  from_workspace` 扫描分析**（与 PLAN-662 痛点陈述 ~2s/demo 一致）；vite 本体
  ready ≈ 0.5 s；back proxy 配置 **33 apps / 14 会话，`lazy_sessions: true` 按
  首访问装载**（run_vue.rs:520；启动日志的会话数为配置容量，非预装载）。
- 场景数字（S1..S6 每格 3 次中位，ms，自命令发出；全 18 样本 0 失败，自动注入）：
  S1 冷外壳 ready 109s/目录 112s；S2 冷应用 A 首交互 121.5s（目录→A 仅 ~2.6s）、
  业务(+1) 123.8s；S3 进程内 A→B 切换 2.0s、B→A 重开 1.95s；S4 重启留缓存
  ready 93.6s（方差 55–133s）；S5 A→B 顺序 B 首交互 91.8s；S6 A→B 驻留 30s
  空闲宿主树 WS 恒 27.4MB/5 线程（vite node 脱离 PPID 链未计=unknown），关闭后
  0 残留进程/listener。B（013-todo 全栈）判据 = 挂载 + `/apps/013-todo/api/todos`
  经 back-proxy 返回 200——**响应体为 `[]`（种子未生效，见 D5）**。
- 失败/降级共存事实：041-auto-edit 包装载失败（deps 内目录缺失）；若干 demo
  存在 schema 漂移告警（label/title prop 未声明）；015-notes 等子模块解析告警。
  以上均为现状 Gallery 的既有状态，非本包注入。

### 1.2 VM 臂（`auto run -r vm`）

- **当前不可运行（3/3 可复现 + 1 变体）**：启动 8–17s 后 exit 1。根因链：
  027-file-manager 处理器合成失败（`Undefined variable: auto`，24 处 SYNTH-FAIL）
  → 合并 App 链接失败（`Undefined symbol: handler_Demo027FileManager_NavTo in
  module App`）→ VM 会话退出，窗口未开、MCP 未起。剔除 027 的语料变体（1 次，
  plan-artifacts 副本）改卡 `Undefined symbol: fmt.pct`——多重 demo 级合成缺口。
  VM 臂全部测量格 pending，不估数；修复责任在 VM 合成/桥接 owner（D6/D7）。

### 1.3 进程/监听拓扑（AC-01）

- Vue 臂：`auto run`（auto.exe）单进程内完成生成/安装/编排；vite dev server
  （node）监听 pac.at front_port=3049；back proxy listener 在宿主进程内（默认
  3358，env `AUTO_GALLERY_PROXY_PORT` 覆写，冲突 +1..+10 再 OS 分配）；**每后端
  会话一条专线程**（AutoVM 线程亲和，PLAN-658）；`lazy_sessions: true` 首访问
  装载（run_vue.rs:520）。无每 tab 独立进程；关闭后 listener 与进程收敛（每
  样本 post_close 拓扑与 baseline 相等）。实测拓扑与 UI 动作对账（V-01）以
  样本 JSON 的 topology{} 快照为准（merge 时折叠摘要入 PLAN-1020 §9）。
- VM 臂：不可运行，无拓扑可录（失败前仅 auto.exe 短暂存活）。

## 2. 公共接口准入（V-03/T-04 摘要；详表在 PLAN-1020 §9）

| 切片 | 状态 | 依据 |
|---|---|---|
| 960 包模块公开执行链 | **ready（有界）** | `PackageAvTarget::cold/cold_with_entry/run_value/global_cells`（av crates，SHA16 e76d39a19c8027c5）+ source-query 生产端（compile_package_product/assemble_package_inputs 等）；spec `docs/specs/ace-migration/m2-modules.md`（SHA16 be25b0c69f2f8cc0 精确核对）；永久入口 `av::conformance::module_packages_960`（Win x64）；无 warm body-edit；A2X/native 边界 D-960-1/2 |
| F0 宿主/FFI（967） | pending（进行中） | executing **9/19** r12：T-03（生产 TestCase lane）、T-08（async completion ports）、T-09（use.rs 入口/身份冻结）已落地 master（cd5f2e568 等，2026-11-14）；本调查读表时为 0/19，执行期间上游推进（如实记录的读表竞态）；F0-D 未完成 |
| H0 Schema（979）/ E0 facade（969）/ B0 构建（978）/ HELLO AutoUI 接入（980） | pending | 均 drafting 0 |
| 会话（970）/调度（968） | pending | 均 drafting 0 |

后继进入门：1021 = go（消费 960 公开链，见上）；1022/1023 = no-go（等上表
pending 切片正式交付）。当前 Gallery 的 AppViewport（Vue/VM 两形）与 back
proxy 均为 Gallery/宿主私有接线，**不构成 ASE 公共 frame/会话接口**，不得当作
公共嵌入先例。

## 3. 工具链断点（上游缺陷，PLAN-1020 只记录不修）

| ID | 缺陷 | 位置 | 影响 | 测量 workaround（仅限测量树，不入库） |
|---|---|---|---|---|
| D1 | I-PROJECT 请求路径 `npm_deps_from_value` 不切 `name:link:path`（缺 `split_pkg_version` 的 `:link:/:file:` 分支） | auto-ui-tools vue.rs:2979（对照 :3523） | 宿主 pac.at 字符串数组 npm_deps → 非法依赖名 → pnpm 拒绝 → `auto run` 全新冷构建失败 | pac.at 改受支持对象语法 + `file:` |
| D2 | natives.ts 无条件 import `./editorBridge`，而该脚手架文件仅 `usage.code_editor` 时发射 | auto-ui-tools vue.rs:514 vs natives 模板 | 全新画廊页面装载 500（主检出同样缺失） | 手写最小 bridge stub（注册表语义） |
| D3 | demo SFC 同时发射 `function __autoThemeRoot()` 与 `const __autoThemeRoot = ref(...)` | auto-ui-tools vue 生成器 demo 发射 | 每个 demo App.vue 编译失败（demo 级全灭；vite overlay 遮挡交互） | 生成后把函数形改名 `__autoThemeRootEl()`（run 后、装载前） |
| D4 | ui-gallery pac.at npm_deps 声明的 @autodown/editor、core 目标目录已不存在 | auto-os pac.at ↔ auto-down packages/（仅剩 engine） | 旧主检出 node_modules junction 悬空；声明级悬空 | editor/core 无运行时 import，stub 包满足解析 |
| D5 | 013-todo 后端会话 `GET /apps/013-todo/api/todos` 返回 **200 `[]`**（db.at 的 4 项种子未生效） | auto-ui back_proxy VM 会话装载（SessionSpec=src/back/api.at） | 全栈 demo 视口挂载但列表空——"后端静态种子可见"不成立 | 判据改为挂载+API 200；空列表作为现状发现记录 |
| D6 | 画廊 VM 臂合并 App 链接失败：027-file-manager 处理器合成 `Undefined variable: auto` → `Undefined symbol: handler_Demo027FileManager_NavTo` | auto run -r vm 合并合成/链接（VmBridge） | VM 臂整体不可运行（3/3 复现，窗口未开） | 无——VM 臂全格 pending |
| D7 | 剔除 027 变体仍失败：`Undefined symbol: fmt.pct in module App` | 同上（另一 demo 的格式化函数缺口） | 多重合成缺口，单点剔除不可救 | 有界尝试 1 次后停止 |

D1–D3 属 auto-ui 生成器/解析器；D4 属 auto-os ↔ auto-down 声明对账。修复责任
在各原 owner，收编前 ASE M0 的 Vue 臂现状按"bounded-go（workaround 后可测）"记。

## 4. 边界

- 样本口径：每格 3 次机制基线，非 P95/正式性能认证；未执行/环境排除格记
  pending，不填零（PLAN-1020 §9 全表为准）。
- 本文件不改变任何上游 Plan 的状态/进入门；上游交付以原 Plan 为权威。
