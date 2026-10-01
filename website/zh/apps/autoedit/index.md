---
title: AutoEdit — 代码与文本工作台
description: 项目浏览、多标签编辑、搜索与差异比较，以及当前运行形态。
outline: [2, 3]
editLink: false
---

<script setup>
import EvidenceImage from '../../../.vitepress/theme/components/EvidenceImage.vue'
</script>

# AutoEdit：代码与文本工作台

AutoEdit 是 Auto 生态中的轻量文本编辑器。它把浏览、比较和审阅代码放在重要位置，也支持日常编辑与保存。当前以原生桌面体验为主要方向；应用逻辑与界面由 Auto 源码组织，底层编辑和渲染能力来自 AutoLang 与 AutoUI。

[全部应用](/zh/apps) · [系统应用与示例](/zh/apps/demos/)

<EvidenceImage src="/apps/autoedit/overview-dark.png" alt="autoedit 原生主界面" caption="AutoEdit 深色原生工作台 · 用户提供的 2026-10-01 主图" :width="1924" :height="1247" :framed="false" loading="eager" zoom-label="放大图片" close-label="关闭" original-label="查看原图" />

## 从项目里的一个文件开始

打开项目后，左侧文件树用于浏览目录，文件在不同标签页中打开。编辑区提供行号、语法高亮和中文文本显示；菜单、工具栏与快捷键提供常用操作。最近文件帮助回到先前浏览的位置。

一个典型过程是：找到文件 → 打开几个相关标签 → 阅读并做小步修改 → 保存 → 比较改动。控制台可以查看操作反馈，状态栏呈现光标位置、编码和换行等信息。

## 编辑、搜索与文件保存

- **多标签编辑**：新建、打开、保存、撤销和重做，以及未保存修改的关闭确认。
- **查找与替换**：大小写、整词、正则和转义等选项，以及全部替换与跨文件搜索。
- **UTF-8 文件处理**：识别 BOM 和换行形式，提供换行转换；非法 UTF-8 的兜底路径避免静默转码保存。
- **会话与最近文件**：记录打开的文件与活动标签，重启时按需装载。打开文件的恢复与未保存正文的恢复是不同能力，不能相互替代。

大文件有单独的处理模式，用来减少不必要的高亮与全文操作。实际可处理的规模与性能取决于运行形态和具体操作；这里不将开发期测量数值作为通用性能承诺。

<!-- Screenshot slot: project search / find-and-replace using a reproducible source fixture. -->

## 比较文件、目录与编辑缓冲区

AutoEdit 的差异比较既服务于阅读，也服务于修改后的复查。

| 对象 | 可以怎样使用 |
|---|---|
| 两个文件 | 并排或内联查看差异，按差异块前后定位 |
| 两个目录 | 查看同、改、增、删等状态，再进入具体文件比较 |
| 打开的缓冲区 | 比较尚未完全落盘的编辑内容，与磁盘文件比较区分开 |

从差异视图可以跳转到对应文件侧编辑，保存后重新比较。这个回路与“直接在差异行中原位编辑”不同，后者仍需另外完善。目录同步操作有确认步骤，应先检查目标和变化范围。

<!-- Screenshot slot: side-by-side diff, followed by the edit/save/recompare operation. -->

## 它在 AI 工作流里的角色

AutoEdit 是供人阅读、审阅和精修文本的工具。它的长期方向还包括被 Agent 驱动、被其他应用嵌入，并复用公共编辑组件。Coding Agent 的任务组织由 [AutoMusk](/zh/apps/automusk/) 负责；AutoEdit 自身的介绍不把这些集成方向写成已经完整提供的 Agent 系统。

它与 [JadeEdit](/zh/apps/jadeedit/) 长期并存：前者侧重代码与文本、差异审阅，后者侧重文档之间的知识关系。两者会尽量共用组件，各自的产品用途和能力仍分别说明。

## 当前进展与运行方式

编辑、搜索、文件与目录比较已经有实现和验证资料；原生交付、性能收口与部分体验细节仍在推进，尚不将 Milestone 当作已经完成。

源码工程位于 `auto-edit` 仓的 `specs/auto-edit`。准备匹配的 Auto 工具链与工程依赖后，开发用的 VM 原生窗口入口是：

```sh
cd specs/auto-edit
auto run -r vm
```

工程也保留 Vue 生成通道；桌面与 Web 的运行条件和功能覆盖须分别确认。编译发布路径、安装和性能指标以对应版本的交付说明为准。

介绍依据项目规范与源码整理，资料时点为 **2026-10-01**。后续截图会以具体运行形态与操作过程配文。

[了解 AutoUI 桌面运行](/zh/ui-desktop) · [了解 AutoOS 虚拟桌面](/zh/autoos/)
