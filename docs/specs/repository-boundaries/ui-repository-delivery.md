# UI 仓发行与交付合同（SD-02，as-built）

> **Status**: active（PLAN-831）

## 渠道形态

- 开发：三仓同级 path 组（显式根/env 覆盖 → 组内兄弟 → 主检出，禁隐式主检出回退进入发行链）。
- 源码发行：本地 Git 固定 commit clone 渠道（fresh/locked/整组重定位 A→B 实测在案）。
- 外部生成工程：受管生成 bundle（792 合同：保旧字节/owned 清理/误删保护/陌生目录不删）。

## profile 与支持矩阵

交付 profile 含 compiler/runtime、真实 auto-ui-tool 二进制与运行必需资产；生成应用 Rust canonical 双直依赖、Vue 与各现行生成后端支持保持。全 profile 真实生成为合同内必需面；构建必需面=vue/rust/vscode+vm 真实运行（r3 用户裁定：jet/ark 真实构建转后续环境门控渠道，缺环境逐项记录【环境门控·转后续】，在场执行后的真实编译/协议失败传播红——分类由失败类别语义位置决定，项目路径字节不可翻转）。

## 多仓切换事务

prepared → ui-landed → lang-switched → verified 四状态收据链（append-only JSONL；scripts/ui-repository-switch.py）。单仓仅 ff-only 且核旧锚（第三方推进零覆盖）；失败保留已完成状态与恢复指令，不假称跨 Git 原子。取消/超时/交换失败按 792 受管保旧合同验证。U831 完整收尾前不放行后继消费者（832）。

## 收据身份

发行/安装/切换/验证收据绑定 producer/工具 hash/三仓 sources；admission 拒绝源身份缺失/漂移/同 HEAD tracked 脏树收据（untracked 可再生产物不判脏）。
