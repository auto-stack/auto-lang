# ace-migration: m2-os-apps — 通用嵌入、非桌面 consumer 与 OS 正式收据

> Status: **M2-A 切片（984/EMBED）已交付**（PLAN-984，T-01 冻结 + B-EMBED-I/V/R，
> 见 [PLAN-984](../../plans/984-ace-m2-os-apps.md) §5/§8/§9）。本文件是该包的规范
> 落点（SD-01/SD-PHASE/SD-DETAIL/SD-OS-WAVES 的 A 切片段；B 批次交付后补齐）。
> OS 桌面/28 App/双 Host 验收正文只在
> [auto-os/PLAN-064](../../../../auto-os/docs/plans/064-ace-desktop-and-app-migration-batches.md)——
> 本文件不复制 OS 产品规范，只登记本包覆盖核对与指针。

## M2-A 已验：通用嵌入 consumer（984/EMBED）

**身份**：仓外独立 consumer（`plan-artifacts/plan-984/embed-consumer`，path deps 仅
`auto-lang-core` + `auto-lang-ace-host` 两公开 crate，scope 内冷构建）+ 仓内永久
fixture（`crates/auto-lang-ace-host/tests/p984_embed.rs`，固定组 `ace-host`）+
978 候选 SDK 腿（`ac-build build|sdk|run|verify|restore`，脚本
`plan-artifacts/plan-984/run-sdk-legs.sh` 驱动）。平台 Windows x64
（ace-host crate Windows-only）。

| 面 | 已验事实 | 来源 |
|---|---|---|
| 宿主 boot 组合 | 单进程安装三公开面：969 桥（`install()` 幂等单例）、970 会话宿主（`install_session_host`，guard 持有/释放）、971-A 工程执行器（`install_ace_project_executor(project_host())` 同指针幂等）——真实嵌入序，非单面测试拼接 | p984_embed host-boot + consumer `boot` |
| 原多模块脚本 | **原** `stdlib/ace/std/{math,text}.at` 模块源（文件原样读取，SHA256 前16 math=`0fe2cfc94f75785d`/text=`9b86f97c9c639abb`）作 `std` 包模块，经 `run_ace_project` 跨包跑 entry；独立 oracle：square(7)×100+byte_len("Auto")=**4904**；原模块槽坏源诊断携带原身份 `std::math`，修复后同门复跑 | p984_embed stdlib 腿 + consumer `run-project` |
| 会话隔离 | 两会话各自携带表独立（x=11/x=22 无串）；Close 后同 id op=具名拒（`session/unknown`），幸存会话不受影响 | p984_embed session 腿 + consumer `sessions` |
| 权限/兼容门 | 具名拒+零 dispatch：`ace.grant/no-provider`（无 provider 授 grant）、`ace.capability/unsupported`（foreign 车道未冻结 cap）、`ace.provider/revoked`（revoke 后；readmit 为精确修复同门重开）、`ace.args|input/unsupported`（argv/stdin 车道）；teardown 后晚到调用=`HostFailure ace.provider/unknown`；每次拒后合法近邻仍跑 | p984_embed permission 腿 + consumer `deny`/`closed` |
| SDK 分发腿 | 隔离异根重定位运行（av-value=49，原 math 合同 oracle）；错 SDK 三负控：异目录=`m2build.recipe.missing`、单字节篡改=`m2build.recipe.integrity`、未声明夹带=`m2build.recipe.extra-file`；权限=错 per-run grant →`m2build.selection.authorization`；坏更新=工件更新+声明哈希更新而 pin 未重钉 →`m2build.selection.pin`（完好 required 子集**不**自动升级 pin）；全部负控后完体 SDK 复跑仍 49 | run-sdk-legs.sh 9 腿（plan §8 B-EMBED-V 收据） |

**边界（A 切片，非全包）**：原多模块材料=已交付域内的原 stdlib 模块（int/text-record
面）；产品级多模块工程（coder/sse/runtime 等含 str/env/fs/模板域）归 B 批次
（CODER/CODERS/SHELL/AI/EDIT）与 OS064，不在 A 切片宣称。SDK 工程模型=978 冻结的
单公共源（多源=具名拒 `m2build.project.multi-source`/`m2build.usage`，其 owner 为
978 域，本包不扩）。978 真工件 provider 调用腿（V-ID-RUN）仍按 D-978 债归
985 消费点；本切片 provider 走 967 F0 真构建（rustc cdylib）。969/978 既有收据
（e0_facade/t970/sdk_pack 等）按原身份复用，本切片新增的只是**实际仓外 consumer**
与组合合同。

## B 批次（未交付，合同见 PLAN-984 §5）

SHELL/AI/CODER/CODERS/EDIT/CLOSURE 及 T-05..07 属 M2-B：开工前置 M2A-08
（985 A批）正式收据；OS 实际产品正文只在 OS064。本节交付后按 SD-PHASE/SD-DETAIL
补已验段落。
