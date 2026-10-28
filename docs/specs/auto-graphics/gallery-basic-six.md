# AutoGraphics Gallery 基础六项（GY-A）

Status: delivered（PLAN-877）。本文件记录仓外示例内容仓的公共消费知识：目录契约、
固定引擎依赖、公开启动入口、单项包与实测边界。引擎能力本身的权威事实仍在各自
Spec（scene-document/scene-hierarchy/scene-assets/behavior-api/segment-query/
swept-aabb/game-actions-and-character2d 等），本文不扩大任何旧 profile。

## 交付形态

- 独立本地仓 `ag-examples`（无远端；master `6a223d9`，PLAN-877 分支
  `codex/plan-877-dev` 已合入）：六个逻辑 ID、十个必需子入口
  （g01/main、g02/main、g03/2d、g03/3d、g04/2d、g04/3d、g05/2d、g05/3d、
  g06/layout-a、g06/layout-b），全部 ready 且 `tools/gallery.py validate
  --all` 通过。
- 目录契约：`catalog/index.json`（ag-examples.catalog/1）与各
  `example.json` 为唯一元数据源；validate 强制集合相等、相对路径安全
  （拒绝绝对/逃逸/链接）、资产 sha256/byte_length 逐文件核对、
  `licenses/<sha256>.LICENSE` 在场、Ready 门（`previews/<slug>/
  acceptance.json`，runner SHA 绑定 + 逐 oracle pass）。未知字段/重复
  ID/缺文件/哈希漂移/坏 engine-lock 均拒绝（28 例结构负控）。
- 状态语义：`ready`=验收完成（证据在案）；`blocked` 必须带 notes；
  编辑教程类（g04）category=`editor-tutorial`（能力类 E，不冒称公共
  Auto 脚本 API）；未列平台一律未测。

## 固定引擎依赖（engine-lock.json）

autographics-dev `8b12632d7`；SDK pin `7b987ca8`（860 保留副本 681 文件
verify PASS）；两 workspace 锁 driver `47176cc0`/prototype `bc7287b5`；
runner `autographics-authoring` SHA-256 `498ce814…`。用户以 `--engine`
显式指定引擎；锁文件只记录可搬迁身份，不存生产者绝对路径。身份不符拒绝
启动。

## 公开启动与验收边界

正常启动只用冻结公共 CLI（`--open` / `--behavior`+`--behavior-api` /
`--frames`+`--report` / `--inspect`）；探针旗标（viewport/segment/sweep/
character/hierarchy-check）属验收自动化，从不进入正常启动路径。
`--inspect` 为无窗口加载+渲染 oracle（重开一致性/哈希）。

验收证据：十入口 runner 绑定记录在 `previews/*/acceptance.json`；
`tools/verify_examples.py` 66 断言（g01..g06）；`tools/verify_packages.py`
六单项包离线 A→B（validate+reopen 恒等、资产篡改拒绝/修复）；结构负控
`tests/test_gallery.py`。

## 打包

`gallery.py package <gid>` 产出自包含 ZIP：过滤目录（仅本 ID）、tools、
acceptance 证据、许可、实体化资产（`assets/<sha256>.glb`）。单项包离线
A→B 搬迁哈希恒等已实测。

## 实测边界（不声称）

- 仅 windows-native-x64 实测；其余平台未测；静态 catalog 页不是 WebGPU
  在线运行。
- 2D 合成序 = alpha 升序（不透明先画）、等 alpha 按数组序（引擎实测；
  任意绘制序不被声称）。3D 材质固定红色占位（scene-document 既有规则）。
- viewport3d 探针 save→reopen 相位按未存默认文档校准（保存前 undo 使
  打开文件回到已存态时 Save 为无操作）——引擎探针语义，非内容缺陷。
- g06 布局对照 = 场景几何文档差异 + 双探针绿；速度对照 = 探针位移
  1.333(speed4)/1.667(speed5)。探针输入深度不达两布局分歧点属预期。
- 不含：G07..G12、874/875 新 API、正式游戏、浏览器在线引擎、远端发布、
  资源商店/下载器/包管理器。

## 验收测试政策（2026-10-28 用户裁定）

- ComputerUse 式 OS 输入自动化验收（真实窗口探针/抢焦点/置顶）暂停——
  干扰日常工作；探针旗标保留但门禁不再运行，除非用户重新授权。
- 门禁测试 = 变更相关 ∩ 抽样（约 1/10 随机 case，seed 在案可复现）；
  全量回归留里程碑统一执行。
- PLAN-877 门禁落证：探针面复用内容提交 `017b959`（工具修复零内容变更）
  的全过记录；新鲜重跑 = validate + 28 负控 + 抽样 g03（12 断言）+
  六包无窗口 A→B（`tools/verify_packages.py`）。

Provenance: [PLAN-877](../../plans/archive/877-autographics-gallery-catalog-and-basic-six.md)（归档），
冻结合同 [attachments/877/work/gallery-contract.md](../../plans/attachments/877/work/gallery-contract.md)
（§9 实施修订、§10 裁定）。源：ag-examples `6a223d9`；引擎消费 autographics-dev
`8b12632d7`；SDK `7b987ca8`（860 保留副本）。G07+ 由后继计划消费。
