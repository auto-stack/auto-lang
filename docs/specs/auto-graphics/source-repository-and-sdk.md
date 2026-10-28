# auto-graphics — source repository & SDK（PLAN-879）

> Status: active（PLAN-879 交付）。AutoGraphics 实验源码的唯一开发权威自
> 2026-10-29 起为 **auto-ui 仓** `experimental/autographics-prototype/` 与
> `experimental/autographics-auto-driver/`（落地 commit
> `4adf2d037a1804741adc9c5a012f08cfa566439d`，基面 `154a3965`）。auto-lang 的
> `autographics-dev` 分支保留完整历史与归属指引（迁出 commit `f07b5ec8`，
> 冻结源 = `1cc68aba012f76951af2556763a5fb21be6a6c8b`，875 最终交付）；
> master 仅承载知识文档。本文记录内容映射、旧 SDK 保全、冷准备、恢复与保持面。

## 单一权威与内容映射

- 迁移分母：`1cc68aba` 下两目录共 **702 个被跟踪文件**，逐字节（git blob 恒等）
  迁至 auto-ui 同名路径；已交付树实测 **695/702 blob 恒等 + 7 个声明修改 +
  686 个新增**（681 快照 + 2 个 `.cargo/config.toml` + gen/verify 工具 +
  `research/plan879-identity.json`），零未声明差异。
- 修改仅限迁仓必需：prepare_sdk（快照准备路径）、verify_source/package_consumer
  （当前身份链选 879）、两 README（归属/冷启动）、.gitattributes（`sdk/frozen/**
  -text` 字节往返）、.gitignore（staging 目录）。算法/Auto API/scene 格式/预算/
  两把 Cargo.lock（driver `47176cc0`、prototype `bc7287b5`）恒等。
- 来源链（旧→新）机器可核：新仓 `experimental/autographics-auto-driver/
  research/plan879-identity.json`（受跟踪）逐文件记录旧 repo+commit+path+blob
  SHA 与新路径+原字节/CRLF 规范化双 SHA；`tools/verify_plan879.py --controls`
  为当前验证器（删件/篡改/未列新增/错哈希/错 aggregate 全拒、还原绿）。
- 历史 planNNN-identity.json 与 research 证据随源迁移为历史内容；历史验证器
  保留但被 879 链位屏蔽；不把旧提交伪造成新仓祖先。

## 旧 SDK 保全与冷准备

- SDK 逻辑 pin `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`（PLAN-846 r2，
  仅 nano_value.rs 非有限值修正）Git 对象不可达；不可达不回退当前语言仓 SDK。
- 保全形态：新仓 `experimental/autographics-auto-driver/sdk/frozen/` 版本化快照
  （681 文件 / 13,681,490 字节 / 最大 1,528,601 B，`-text` 属性字节往返），
  逐文件由 `research/source-lock.json` 锁定；保留副本（plan-860 消费包）实测
  681/681 一致。
- 冷准备：`python tools/prepare_sdk.py --from-snapshot`（driver 目录）——不读旧仓/
  网络/Git 对象库；逐文件校验通过后经 staging 原子发布 `.sdk-source`；拒绝已存在
  目录覆盖、链接/junction、路径逃逸、缺/多/错哈希；坏快照拒绝后无半准备目录残留，
  `--verify-only` 只认 `.sdk-source`。legacy `--repository` 路径保留供历史复现。
- 消费闭包：`verify_source.py` 断言 metadata `--locked --offline` 下 target 不逃逸、
  17 本地包/29 path 边、forbidden（auto-lang/auto-man/auto-ui-* 五生产面）=0、
  AGP 冻结注册表身份保留。

## 构建栈与workspace隔离

- 两 AG workspace 各自 `.cargo/config.toml`：Windows MSVC `/STACK:33554432`
  （32MiB PE 主线程栈）+ `RUST_MIN_STACK=16777216`（16MiB worker/test 线程栈），
  复述自旧语言仓根配置；auto-ui 根 manifest/锁/crates 零改动，默认五包
  （compiler/dialect/runtime/tools/shell-pack）不受影响；AG 不继承 auto-ui 生产
  patch/default targets（AGP 仍 stock iced，零 path 依赖生产面）。
- package_consumer 导出包自含上述配置：迁移前"导出包需手动 env 设栈"的缺口
  已修复（消费构建实测 PE reserve=33554432、26/26 无窗口测试绿、A→B 重定位
  冷准备/离线锁定构建成立）。

## 切换与恢复

- 落地顺序（已执行）：目标候选先落 auto-ui master（prepared，权威暂仍旧源）→
  确认可消费后落语言侧迁出指引 → 单一权威=auto-ui。两步均为 fast-forward，
  无 merge commit、无历史改写、无强推。
- 恢复锚点：落地前 ref `154a3965`（目标回退点）与 `1cc68aba`（旧源冻结点）
  永久保留为祖先；回退仅触碰本计划两分支，不覆盖他线；两仓不得双 active。
- 保持面：880（GY-B）自 `1cc68aba` 固定对象独立导出消费包，不依赖本计划时序；
  ag-examples（877）固定消费 861 runner，不受影响。
