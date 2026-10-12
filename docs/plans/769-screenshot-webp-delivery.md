---
plan_id: PLAN-769
status: drafting
feature_name: screenshot-webp-delivery
author: [Codex]
created_at: 2026-10-12
updated_at: 2026-10-12
plan_revision: 1
current_step: 0
total_steps: 3
supersedes_spec_components: [docs/specs/website/project.md, docs/specs/website/design/os-ai-introduction.md]
new_spec_components: []
touched_goals: []
affects: [website]
---

# [PLAN-769] 截图压缩与按需图片加载

## 0. 变更摘要
用户要求压缩网站较大截图，尤其虚拟桌面，大图低于2MB、小图低于1MB，减轻网页加载。

## 1. 目标
以真实截图生成网页轻量 WebP，保持场景、比例及清晰文字；默认展示/放大使用压缩图，原始PNG仅显式原图下载。修正隐藏dialog提前加载原图。原始资产与证据hash不改。

## 2. 架构方案
工作树生成截图WebP派生物与静态src映射；EvidenceImage用映射选择展示图，dialog图片仅打开后挂载且使用压缩图，原图href沿用原PNG。大型直接img相关入口也使用映射或派生路径。无远程图片服务。

## 3. 技术栈
VitePress/Vue、Pillow WebP 编码和Playwright网络/交互验证，Category A禁止Cargo/docs_gen。

## 4. 需求分析与背景调查
用户2026-10-12明确授权截图降分辨率/压缩和尺寸预算，覆盖派生图片编码；本会话本地提交部署授权持续。读 docs/specs/overview.md、website/project.md、os-ai-introduction.md，EvidenceImage/DesktopShowcase/ScreenshotGallery/ReleaseLanding，以及相关测试。最大desktop01为5.57MiB，desktop02为4.95MiB，launcher03为3.83MiB；现有hidden dialog eager img提前加载原图，缩略lazy不足以避免。原图为既有授权截图，派生物保留来源，不生成/改画面。源码原始资产不改，用户最新压缩要求更新网页展示契约，历史来源hash仍有效。

## 5. 详细设计
盘点public截图目录（desktop-showcase/apps/v05，排除未用candidate；其他目录大图补查），>=200KB编码WebP，优先原尺寸质量95，超预算逐级90/85/80；所有实际派生物目标<1,000,000 bytes，比用户大图2MB预算严格。若某图质量不够，按比例缩小且记录真实输出尺寸，不冒充原图分辨率。
新脚本 website/scripts/optimize-screenshots.py 可重现派生物/映射/来源hash目录；新增 theme/data/screenshot-variants.json 和同目录image文件。EvidenceImage computed展示src，original href留旧值；响应尺寸保留原比例；dialog用反映open/close/cancel的状态按需挂载，tab切换须按Vue当前props更新。直接img的有用大图进入同一展示策略，文字/日期不变。

### 规范增量
| delta_id | operation | target | before/after | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/website/project.md | 原PNG默认加载 → 本地轻量WebP/按需dialog/原图下载预算 | 用户加载速度与尺寸要求 | AC-01/02 |
| SD-02 | modify | docs/specs/website/design/os-ai-introduction.md | 六图保留原图展示 → 原始来源不变、WebP展示/放大、下载原PNG | 避免原图说明混淆 | AC-01/02 |

## 6. 测试设计
字节预算/维度/原图hash/重复生成确定性校验；原图与压缩图对照审读（桌面壁纸/Launcher/编辑器小字）。冻结依赖构建；更新desktop-showcase/gallery/assets现有src断言；专门网络用例验证默认和放大无大PNG请求、预算内WebP成功、无隐藏dialog图片、显式原图href和Enter/Esc焦点。中英OS/v05/apps/应用及五宽度双主题回归，按改动触面选测试不重复全部。备份部署HTTP/HTTPS图和页面核对，线上中英OS网络与原图放大。

## 7. 验收标准
- AC-01：六桌面及被使用的其他大截图实际浏览使用派生压缩图；派生图<2,000,000 bytes、小图<1,000,000（优先全部<1MB）；原图hash不变，文字/比例/日期如实，来源和减小幅度记录。
- AC-02：隐藏dialog不下载图片，展示lazy/放大按需；Enter/关闭/Esc/焦点/原图链接/其他图集仍可用；中英/五宽度/双主题不溢出，构建和按触面回归通过。
- AC-03：本地提交/合入、可回滚部署及线上实际下载预算核验，review/spec/索引/归档守卫清理有收据。

## 8. 执行步骤
- [ ] T-01：lang-769专用工作树编码/目录/图片映射，组件/直接图片与规范调整；AC-01/02。
- [ ] T-02（依赖T-01）：字节/原始hash/图像审读、构建、现有相关回归和网络预算检查；AC-01/02。
- [ ] T-03（依赖T-02）：提交部署线上验证；完成review/merge投影和归档清理；AC-03。

## 9. 复审记录
stage:new | PLAN-769 r1 | outcome:pass | next:work；用户范围授权已明确，具体压缩/按需实现属于该范围。

## 10. 待澄清事项
无。默认网页图压缩，原图不被页面提前下载，仅用户明确打开时获取。
