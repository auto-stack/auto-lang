---
plan_id: PLAN-767
status: archived
feature_name: os-overview-inline-cards
author: [Codex]
created_at: 2026-10-12
updated_at: 2026-10-12
plan_revision: 2
supersedes_spec_components: [docs/specs/website/design/os-ai-introduction.md, docs/specs/website/project.md]
new_spec_components: []
touched_goals: []
affects: [website]
current_step: 3
total_steps: 3
---

# [PLAN-767] OS 主页面融合与单列截图卡片

## 0. 变更摘要
用户要求 /autoos/ 当前桌面内容直接并入 /os，多个 OS 截图逐张显示为单列图文卡片，图片在上、介绍在下；不依赖 tab 或跨页查看。

## 1. 目标
双语 OS 主页面完整融合桌面、窗口/工作区、应用/Kanban、共享设置/配置图片、宿主边界和既有理念/架构/进展/长期路线；六图全部直接展示。保留完整原图及放大，更新入口和旧地址兼容。
非目标：新系统能力、重拍/重绘截图、更改图片字节、其他产品图集改版。

## 2. 架构方案
作者源 /os.md 与 /zh/os.md 是 canonical 阅读页。旧 AutoOS 地址变为同语言兼容入口，保留 hash 跳到融合页；主页面不需要离页。共用 DesktopShowcase 改为逐图单列卡片（v0.5 同步）；ScreenshotGallery 其他使用者保全。

## 3. 技术栈
VitePress/Vue/EvidenceImage/Playwright。Category A：网站/内容，禁止 Cargo/docs_gen。

## 4. 需求分析与背景调查
授权：2026-10-12 用户明确提供 OS/AutoOS/apps 页和两截图指定上述融合与布局；本会话既有本地提交及服务器部署授权持续覆盖网站更新。
依据：docs/specs/overview.md、website/project.md、website/design/os-ai-introduction.md（PLAN-719/720/758）和现行 /os /autoos 双语源码、DesktopShowcase、desktop-showcase.ts、EvidenceImage、introCopy、ReleaseLanding/DemoCatalog 入口及 desktop-showcase/os-ai-introduction/v05-copy/site-ui 测试。
当前 /os preview 仅两图，/autoos 六图 tab 且正文分离。六图 2560×1600 /desktop-showcase 保留 2026-10-01 演示来源；Kanban/配置四图是历史项目截图，不能冒充同批桌面或全平台验收。
在线页面 web 工具不可访问，使用本地 canonical 源及部署服务器 HTTP/HTTPS 验证。

## 5. 详细设计
DesktopShowcase 的每张图独立 article：EvidenceImage + 捕获来源说明 + 编号/标题/场景介绍；宽屏/窄屏均一列，六图为 SSR 正文、lazy 加载且不隐藏。取消 preview/desktop-more 与独立的四步骤列表，逐图描述完整承接原步骤。
将 AutoOS 窗口/应用/设置/框架正文与四张历史图迁入 OS 主页面，保持默认阅读、目录和 LaOS/OS over OS/AI+Lang+OS 层次；单一 H1。旧 /autoos/ canonical/noindex + 轻量 OSOverviewRedirect（新文件），正常访问跳同语言 /os#desktop-showcase，已有 hash 保留；无 JS 时保留清晰主页面链接。
当前站内入口改为 /os 或 /os#desktop-showcase，v0.5 使用相同六卡布局。其他历史链接仍由兼容入口承接。

### 规范增量
| delta_id | operation | target | before/after | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/website/design/os-ai-introduction.md | 两页职责/预览跳专题 → /os 完整融合 + 旧地址兼容；OS/v0.5 六图单列可见卡片 | 用户指定直接阅读流程 | AC-01/02/03 |
| SD-02 | modify | docs/specs/website/project.md | 当前 OS 页面、共用桌面展示/入口口径同步 | current-state 唯源 | AC-01/03 |

## 6. 测试设计
website 工作树 bun install --frozen-lockfile；AUTO_BOOK_ROOT=D:/autostack/book npm run build；更新 desktop-showcase / os-ai-introduction / v05-copy，定向运行上述及 site-ui 桌面序列/Launcher 回归。验证 SSR 新正文/十张来源、图片原始 hash 不变、单列顺序无 tabs、双语目录/链接/搜索/兼容 hash、Enter/原图/Esc 返回焦点。五宽度 360/390/768/1024/1440×双主题；390/1440 双语 OS/v0.5 截图审读。部署备份/切换/回滚，线上验证两主页面/兼容入口及卡片。

## 7. 验收标准
- AC-01：双语 /os 直接包含原 AutoOS 桌面/窗口/应用/Kanban/配置/宿主说明和既有架构/进展/长期路线；单一 H1/有效目录/SSR，可不跳转看完整内容。
- AC-02：OS 与共用 v0.5 六张图按既定顺序全展示于单列独立卡片，图片在上、标题/介绍在下、无 tab；原图字节/日期/平台边界保留，键盘放大及原图可用，五宽度双主题无溢出。
- AC-03：现行入口直达 /os 相应内容；旧 /autoos/ 同语言/hash 兼容，其他图集不变；构建/定向回归/线上校验通过，本地提交与部署有凭据。

## 8. 执行步骤
- [x] T-01：lang-767 专用工作树修改双语 OS/AutoOS、DesktopShowcase/desktop-showcase data、新 OSOverviewRedirect 与主题入口；AC-01/02/03。
- [x] T-02（依赖 T-01）：更新既有相关测试/构建/矩阵视觉验证，准备 canonical SD-01/02、来源 hash 和冻结规范增量/证据；AC-01/02/03。
- [x] T-03（依赖 T-02）：已验证源码提交并静态部署，备份/回滚与线上卡片/路由校验；AC-03。随后按 review/merge 记录合入、索引/兼容账本、归档及守卫清理。

T-01 证据：lang-767 自 d541c456a 创建；双语正文/六卡/入口/兼容组件完成，10 原图与基面字节一致。构建 EXIT0 137.27s；测试端口 4767 被 Windows 拒绝监听，改由 OS 分配已可绑定端口 7899，功能回归继续。

## 9. 复审记录
stage: new | PLAN-767 r1 | outcome: pass | next: work；现有用户授权覆盖合同，不增加待确认事项。

r2 等价验证调整：现场确认 site-ui 的旧 gallery tab 两测试实际指向 DesktopShowcase；相应断言改为六卡常驻/Launcher 可见。ScreenshotGallery 组件本身保持，需求/AC/授权不变。

## 10. 待澄清事项
无。具体渲染/演示证据边界按既有 dated captures；旧链接兼容是历史入口维护，不是主页面阅读所需跳转。

T-02 证据：构建 137.27s EXIT0；34 项主要回归 + 2 项 site-ui 定向回归全绿。四路由×五宽度×双主题 40 次布局校验；16 张完整图集及 16 张首卡捕获，人工审读桌面深色/手机浅色首卡。10 张原图不变。durable: docs/reports/p767-verification.json、p767-os-source-catalog.json、p767-spec-delta.json。

T-03 证据：源码 964dd64bb；release 20261012-964dd64bb；IP HTTP + 域名 HTTPS 字节一致；10 原图上传 hash 全等，备份 /home/visus/auto-website-backup-20261012-964dd64bb。线上双语主页面与兼容入口 4/4，另行完成双语原图 decode 与捕获 2/2。规范哈希改用 Git 已提交 UTF-8/LF 字节冻结（仅 CRLF 规范化）。

review PLAN-767:r2 | pass | reviewed_commit: dff0a8c0b4040504fe93940500d153570b57fed9 | implementation: 964dd64bb | dependencies: p767-verification.json book/lock | frozen_delta: p767-spec-delta.json | next: merge。复审同执行会话，以已提交 diff/原图/hash/运行证据重建，非另一个独立上下文；详见 docs/reports/p767-review.md。AC-01/02/03 全 pass；零未批准遗漏/新债，Category A。

## 11. spec-sync 回写与交付收据
PLAN-767:r2 completion_kind: delivered。
- prepared：review pass dff0a8c0b；SD-01/02 UTF-8/LF hash冻结；准备提交 7dc50b29b，源码964dd64bb后仅报告/规范/账本元数据改变。
- landed：master --ff-only d541c456a→7dc50b29b；rebase up-to-date 无改写；主检出源码与部署源码网站树零差异、规范 blob hash与账本文件hash核对通过。
- ledger_refreshed：外部 Codex B 轨 guarded offline projection，localhost8080 不可达；字节备份及复原对账通过。P719-1/P720-1 两个受影响契约更新，P767-T/P767-R/P767-Report 新增；849 原条目除两受影响契约外零扰动，六段未知字段保持，现852项。旧投影无 version，保留根形状；p767-deposit.json 记录前/后hash。其他章节/原条目字节保持，首次全局ID闸门遇跨段历史P014-1重复，自动恢复备份；改用段内ID唯一闸门后通过（无产品改动）。
- module回写：website/design/os-ai-introduction.md 与 project.md；website/plans.md追加767，spec-index.py26projects再生无语义diff。
- archived：docs/plans/archive/767-os-overview-inline-cards.md；本收据同Plan进入Git，交付完成。
- batch_due：767%5!=0，收据last_covered766/2026-10-12T04:08Z，<48h；本轮未到期且Category A零Rust，未跑Cargo/docs_gen。
- deployment：20261012-964dd64bb，archive50a079bc3c7a251794f20d1860d36a3c88887658bf309e30974858daf4670b0b；HTTP/HTTPS与原图字节验证；/home/visus/auto-website-backup-20261012-964dd64bb保留。
- cleaned：wt-guard clean；lang-767 工作树与 plan-767-dev 分支已移除；核对绝对组路径且空目录后移除组。归档搬迁先由 fc4dd1109 提交，状态/复审/收据随后由 921240568 落盘（首次 git add 同时引用已移除旧路径而被拒，现已纠正）；最终主检出 clean。本次只清理自有工作树，fix-widget-media-key 等他席工作树保留。
