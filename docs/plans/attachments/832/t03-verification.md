# PLAN-832 T-03 三仓验证设施记录

## 交付物

- `test/fixtures/modularity/three-repository-consumer/`：独立 workspace，真实消费管理规则、语言 SDK、I-PROJECT/I-TOOL 协议。
- `scripts/three-repository-fixture.py`：pre-migration 真实控制、后续阶段 pending 门、测试 ID 检查和自变负控。
- 根 `Cargo.toml` 仅将新 fixture 排除出主 workspace，使其成为独立解析单元；未改生产热点。

## 实际结果

`python scripts/three-repository-fixture.py run --phase pre-migration`：**10 pass / 0 fail / 3 pending**。

- SRC-01/02：现行包、lang 中立源、fixed UI 源和零 Man 复制。
- DAG-01：CLI 基座、runtime 无 UI、compat UI facet 的区分性成立。
- CORE-01：外部 consumer 3/3——管理端口规则、SDK 实际 VM 输出 42、协议 framed request 编解码往返。
- CORE-02：normal/build 闭包无 UI/旧兼容/第二实现，四个关键 package 各一个 source；独立 `verify-deps.sh` 同样 PASS。
- CMD-01：实际 `auto --version` 和 30 个可见命令面。
- CMD-02：`stdlib inspect --format json` 为合法机器 JSON；当前 inventory partial 按合同 exit 3。
- TID-01/02：24/122/4 lib 测试 ID 无重复/缺失，集成目标 3/3/10。
- RELEASE-01：UI/down 固定提交与 U831 archive 读回。

通过收据：[t03-pre-migration.json.txt](t03-pre-migration.json.txt)。

## 区分性与 pending 门

`python scripts/three-repository-fixture.py selftest` PASS：缺命令、重复/缺失测试 ID、runtime 非法 UI 依赖、UI crate/第二管理源均被判红；合法控制及 cargo tree 同 source 重复打印/`(*)` 标记正确去重。

`python scripts/three-repository-fixture.py run --phase pre-migration --require-complete` 以 **exit 1** 拒绝 INSTALL/FAILURE/SWITCHED 三项 pending，收据：[t03-require-complete.json.txt](t03-require-complete.json.txt)。因此 pre-migration 通过不能冒充最终安装/故障/切换验收。

## 调试归因（最终代码已覆盖）

- 合法协议请求 1073B 初超测试自设 1024B 帧预算：调为 8192B，非协议产品缺陷。
- Windows Python 的 `bash` 命中 WSL：driver 改直接 `cargo tree`，不依赖宿主 shell 路径。
- `cargo tree --prefix none` 同 source 多行及 `(*)` 标记：按去除标记后的完整包行去重；不同 source 行仍由负控判红。

本轮仅运行新 fixture/脚本定向检查，未运行整档。安装、故障恢复、迁后原件删除仍分别归 T-05..T-08；merge 统一门 pending。
