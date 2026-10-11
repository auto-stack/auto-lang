# Website application introductions

> PLAN-722 / PLAN-723 · 2026-10-01 · VitePress / Vue 3

## Product structure

/apps (EN/ZH) distinguishes four main products—AutoEdit, AutoShell, AutoMusk, AutoJade—from system apps and demos. AutoDown and AutoUI retain related resource pages; neither replaces one of the four product entries. AutoEdit and AutoJade remain separate products with shared foundations.

AppsOverview.vue and data/applications.ts provide SSR cards, a two-column desktop grid, a single-column narrow layout, the inline six-category 28-demo catalog, and ecosystem/resource links. No business API or runtime gallery is loaded by the introduction. Global navigation is a separate change.

## Reading topics and truthful progress

Four EN/ZH product topics use the default reading layout with a single H1, an outline, locale-correct internal links, and a material date. Explain purpose, a typical operation, current capabilities, runtime prerequisites, related projects, and work still in progress. Current claims come from the sibling source/spec snapshot recorded in docs/reports/p722-source-baseline.json.

Do not claim all-Auto, full runtime parity, unattended coding, completed Milestones, universal performance budgets, or collaboration from project direction alone. AutoMusk distinguishes skill contracts from runtime Relay, and provider/daemon communication from application tool execution. AutoJade distinguishes single-document draft/reload protection from in-progress multi-document operations.

## Approved product screenshot contract

The initial writing phase was image-free. PLAN-723 introduces the approved AutoEdit PixPin_2026-10-01_15-14-27.png capture and AutoShell ash-01 native colored ls capture in the four-product overview and their EN/ZH details. Use EvidenceImage for keyboard enlargement and return focus. Never render empty frames, broken images, visible TODOs, or fake screenshots. Musk remains text-only until its sample-project walkthrough is ready; Jade remains text-only until its Milestone. Further operation images require their own real evidence.

AutoShell's existing evidence and interactive examples remain at /apps/autoshell/guide/ and the ZH mirror, using unchanged AutoShellLanding.vue. The introduction links the guide; the guide links back. Retain the native ls overview, mode records and their provenance, script tabs, keyboard navigation, copy, downloads, and version/platform boundaries. Release AutoShellPreview remains unchanged.

PLAN-756 user direction permits explicit text-only pending-capture slots for this writing phase. Home and v0.5 use the same four-product applicationCopy source, with pending AutoEdit/Musk/Jade images; approved AutoEdit detail/overview and Shell captures remain. Product operation slots and release comparison/Playground slots bind to SHOT-01..12 in docs/reports/website-v05-capture-guide.md. Slots have no image request and are not delivery evidence. Musk Kanban/Canvas and Jade milestone imagery require their own candidate acceptance; old Kanban or Jade Garden material cannot fill them.

## Verification

Build and verify SSR, routes, local links, outline/locale behavior, approved image hashes and loading, absence of unready-product images/iframes/backend requests, original guide interactions, and EN/ZH at five widths in both themes. Demo introductions stay in the overview, with native disclosures and shareable anchors; only the four main products retain standalone detail topics. Legacy demo URLs redirect to the overview. Demo images follow [the capture catalog contract](demo-capture-catalog.md). Prepared Spec changes belong to the plan worktree until landing. Category A does not trigger Cargo tests or docs_gen.

PLAN-758：产品现名 AutoJade，工程/专题标识 auto-jade；双语规范路径 /apps/auto-jade/ 与 /zh/apps/auto-jade/，旧 jadeedit 路由保留同语言兼容跳转。源码仓库实际地址尚未改名，GitHub jade-edit 地址继续可用。用户提供的六张深色第一方手册示例图按实际场景展示，SHOT-07 为主图，SHOT-08 为反链/双链/快速打开/建页确认/文件树画廊，SHOT-09 草稿恢复仍待拍；P4 标签文件名与实际画面不符不入库。图片保留原始字节、原图与键盘放大，来源/宽高/hash/准备态及未独立确认的深色工具链身份在 docs/reports/p758-autojade-capture-catalog.json。快开不替代全文搜索，准备图不代表最终 RC 已通过。
