# 语言仓独立构建边界（PLAN-906）

> **Status**: active；owner为language；Web/UI/Man按各自组合点参与。900-N7 关闭依据。

## L1/L2 两层验收

- **L1（Core SDK 独立消费）**：外部消费者仅凭语言仓来源即可消费 `auto-lang-core`
  公开面（含 `http_service_config` 合同再导出与类型化槽、VM、a2r 投影、HTTP host
  port 协议）。无需 Web/UI/Man 检出、无 `AUTO_*_SOURCE_ROOT`/delivery override、
  无 ambient patch。
- **L2（完整语言 workspace）**：根 `cargo metadata --locked` 与
  `cargo build --workspace --locked` 及受影响固定组在无 Web/UI/Man 兄弟检出下完成。
  不以删成员/隐藏 optional/dev 依赖/复制实现/删 links 守卫/借旧编译物实现。

## 单源与来源规则

- `auto-web-contract`（严格解析/hash/CORS/Host/可信代理纯策略；serde+serde_json；
  无运行时）单源在语言仓 `crates/auto-web-contract`（PLAN-906 自 Web 仓逐字迁入，
  `links = "auto_web_contract"` 与 build.rs 指纹机制随包）。Web 仓
  `auto-web` 经 path 回指语言仓消费（方向=既有 web→lang path 依赖族）；同名双
  source/可编辑副本被禁止，`RateLimiter`/`ServiceRuntime`/`RateDecision` 等有状态
  运行时留 Web。
- facade `auto-lang` 与 `auto-api-gen` 零 `auto-web` 依赖：Web 生成器/文件/上传/
  观测/`vm::ffi` Web 执行面由消费方直连 `auto_web`（Man/UI/生成工程既有形态）；
  `auto_lang::http_service_config` 指 core 再导出（合同+槽，不含 Web 运行时件）；
  api-gen 非 tauri 入口具名拒绝并指路 `auto_web::generate_api`。
- 需要真 Web 宿主的 http e2e 家族（plan705/707/729/730/734/736/776/781 与 707_wait/
  349 CORS）在独立探针工程 `test/fixtures/modularity/lang-web-e2e/`（非 workspace
  成员，显式装配 `install_http_service_host_if_absent`），由四仓 fixture 门驱动
  （`four_repository` 固定腿）；原测试 ID/断言保留，语言 `http` 固定组仅保客户端面。

## 渠道与恢复

- 固定来源=语言仓自身（含仓内 `vendor/autodown-test-parser`）；registry 依赖按
  `Cargo.lock` 锁定（fresh 需网络/缓存，warm `--offline` 已锁可离线——不冒充冷源无网）。
- 缺源/错源具名失败（manifest 级红于具体缺失路径），无主检出 fallback；交付闭包
  receipt owner：`auto-web-contract`=`auto-lang`（`source_owner` lang 类别）。
- 回退：跨仓提交组合逆序（lang manifest/源码/测试迁移 + web workspace/依赖行 +
  ui 投影类别），锁再生即回旧形态；无远端动作。

## PLAN-930 独立边界复验（SD-03，as-built）

- 隔离快照=仅语言来源的冻结副本（无 UI/Web/Man/Down 兄弟检出、快照初始无
  target/ 无 .git、清除 AUTO_* 覆盖），落 plan-artifacts 暂存区不移动用户
  真实仓；L2 构建产物落快照内属允许范围（不借任何旧编译物）。
- L1：`cargo test --manifest-path test/fixtures/modularity/language-sdk-consumer/
  Cargo.toml` 真实 API/VM/trans 消费 5/5；缺源负控（抽走 `auto-lang-core`
  源码模块→具名红、恢复→绿）。
- L2：根 `cargo metadata --locked` 与 `cargo build --workspace --locked`
  在隔离快照内真实完成（exit 0）；不删成员/optional/dev 依赖、不复制
  UI/Web 实现、不借旧编译物。
- 复用边界：输入与历史有效证据身份未变时按 PLAN-896 规则复用，否则重验；
  L1 不冒 L2，metadata 不冒 build；registry 源可用条件如实记录。
