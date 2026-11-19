# 按语言特性组织的共享 AutoDown 语料（feature-organized corpus）

> Status: **delivered（PLAN-1078 AVP-00，2026-11-19）**。公共语言测试按语言特性
> 集中组织：同一份 Auto 源码供 AV/AC/A2R 等后端各自追加目标检查，避免重复维护。
> 设计合同见 [docs/design/testing/ace-feature-organized-autodown-corpus.md](../../design/testing/ace-feature-organized-autodown-corpus.md)；
> 本文件记录**实际交付**的作者/选择/迁移/忽略机制与支持事实。

## 1. 作者与选择（交付）

- 唯一活跃作者根：`crates/auto-lang/test/cases/language/`（按语言行为命名，
  不以 AV/AC/A2R、M1/M2 或计划号划分）。公共总选择 `language.suite.ad`。
- 每个公共 Case 一份 `### Auto` 源码（或多文件工程闭包）；各后端在 Case 内
  声明检查。suite/固定组只引用、选择，不复制 Case 形成后端视图。
- 固定组 `case-av-language-parity`（domain=ace、显式选择）＝公共语料 **AV 视图**：
  runner 按 `check` 字段（`--check av`）只选 AV 检查；旧 AVM 对照只经显式
  avm 选择；`NOT_RUN`（未选中）不充 PASS，AV 视图门只看选中行（选中行 FAIL
  仍红）。

## 2. 检查级 `#ignore`（交付）

- 裸 `#ignore` 紧跟检查小节标题、位于属性之前 → 该检查映射 `CheckSpec.skip`
  （reason=`AV阶段暂缓非AV检查`、issue=`AV-ONLY-1078-1096`、owner=PLAN-1078）。
- 只影响该检查；不能标整 Case 或共享 Auto 源；AV required 检查不得被关闭。
- 忽略的检查不启动其绑定/编译/链接/执行/外部工具准备（FC-08：prepare/
  dispatch=0），不阻 AV 门；后端任务逐项移除本阶段 `#ignore`，保原
  oracle/平台条件/历史 ignore，按同一源真实运行。
- `#ignore` 不放松公共文档结构/schema 准入；未注册的未来 recipe 保
  Notes/义务表 pending。

## 3. 迁移映射（交付）

- `language/migration/legacy-map.tsv`：旧义务（repo 路径/Case ID/check ID/动作/
  源指纹/旧 oracle/条件）→ canonical Case/check + 原因 + owner + 状态
  （mapped / research-hold / no-such-shape / parser-only-hold）。数据映射，
  不作新注册器；旧 ID 不丢，合并不减义务。

## 4. 支持/边界

- AV-ONLY 阶段（1078–1096）：非 AV 检查加 `#ignore` 或保持未选；共享文档/源码、
  AV 必需公共编译与语义验证仍正确。非 AV required 关闭是对应后端/1068/985
  的最终门，AV 阶段的忽略不当通过。
- 平台：Windows 执行；Linux/WSL 腿归专用 Linux 环境。
