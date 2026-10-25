# PLAN-832 T-08 消费者切换与原件删除验证

冻结实现：Man `356f2c8186d07bc18567b18179c3cad7bda2d483`；lang `1985c73aebcd844f66e686d79004930d7260b755`；UI/down 固定版本不变。

## 1. 真实切换

- Man 目标提交先行存在并经 T-07 独立发行验证；随后语言仓 9 个 package 与 6 个已承接 fixture 实际删除，根 workspace 由 41 降为 32 个成员，`cargo metadata --locked` 通过。
- `three-repository-consumer` 改指 Man 的 auto-man-core；auto-api-gen 的 plan734 fixture 改指 Man；792 driver 分别使用 Man auto 与 fixed UI auto-ui-tool；reader/remote 默认 auto 路径改指 Man。
- Man `.cargo` 的 `t` 修正为 workspace lib + auto bin，避免 24 个 CLI 测试静默丢失。

## 2. 验证

- [t08-source-switched.json.txt](t08-source-switched.json.txt)：`--require-complete` 全 pass——原件不存在、Man DAG、真实管理+SDK+协议 consumer、单一 source、CLI/JSON、测试 ID、集成目标与 14/14 发行收据。
- `cargo check --locked -p auto-lang -p auto-api-gen` 通过；24/122/4 测试 ID 与基线逐字一致，集成目标 3/3/10。
- [t08-man-tier-faces.json.txt](t08-man-tier-faces.json.txt)：man-t/man-ti 共 323 个测试目标成功列举。Man 整档执行归 merge。
- 777 最终 strict=pass/0 violations/4 grandfathered；`man-template-assets`=20 files/410397 bytes，旧 `auto-man/assets` 重复项退役。基线刷新与源码/资产根均绑定干净提交。
- [t08-generated-ui-receipt.json.txt](t08-generated-ui-receipt.json.txt)：792 **37/37**——fresh lock、locked build、A→B、资源/坏 pin/坏 hash/外来目录、再交付恢复、dev 同源、随包 UI tool、真实 profile 和取消/VM 运行全过。首次 discovery 缺 fixed tool 路径后，显式绑定 fixed UI tool 重放通过。
- CI 入口：[ci-entrypoints.md](ci-entrypoints.md)。

## 3. 未决环境边界

V06c 的 vue/rust/vscode 构建 completed，六 profile 生成通过，VM 真运行通过；jet/ark 因本机无 Gradle/Hvigor 仅生成通过、构建未执行成功。832 没有 831 r3 的延期授权，故此项在 T-09 交接中保持 **pending**，不得以 37/37 的 driver 结果宣称全部平台支持完成。
