# auto-package-lock

> **Status**: active
> 路径：`crates/auto-package-lock`  | 技术栈：Rust（serde / serde_json / toml 0.8 / auto-val）

pac.lock 核心（PLAN-795 从 auto-man `lock.rs` 拆出）：锁数据模型、TOML/JSON 编解码与排序/查找/校验规则的单一定义。可被无 auto-lang/auto-man/auto-gen/auto-vm/UI 闭包的外部 consumer 独立消费（外部消费方探针 `test/fixtures/modularity/package-lock-consumer`，闭包断言 `verify-deps.sh`）。

## 目标与范围

- 模型：`LockEntry{name,version,url,commit,path}`、`LockFile{automan_version,dependencies}`（serde 形状与历史 auto-man 公开 API 逐字段一致）。
- 编解码：`.json` 后缀（对 `AutoPath` 统一化字符串判定，大小写敏感）选 JSON，其余默认 TOML；pretty 序列化；`save` 先序列化后建目录再写盘（序列化失败不写新数据，写盘为普通 `fs::write`，无原子事务承诺）。
- 规则：`from_dependencies` 构建时按 name 稳定排序；`find_entry`/`get` 首个同名命中（重名保插入序、首条语义）；`verify_dependencies` 校验缺项/版本错/PLAN-635 D6 无 commit 本地路径的物化存在性，错误按 resolved 迭代序聚合并以 `\n` 连接。
- 版本戳：`automan_version` 恒由调用方提供（auto-man 传其 `AUTOMAN_VERSION`），core 永不代填自身包版本。
- 视图投影：`LockedEntryView`（name/version/commit/path 只读视图）让 auto-man 兼容门面类型零 DTO 往返接入共享规则；`ResolvedDependency{name,version}` 为 Target-free 的已解析依赖投影。
- 不做：pac.at 解析、依赖下载/物化、版本解析、构建调度、Git 内容完整性校验——这些留在 auto-man（adapter）或后续计划。

## 依赖边界（红线）

- 依赖仅 `auto-val`（AutoPath——保持 codec 路径语义与历史行为逐字节一致）+ serde/serde_json/toml + std。
- **禁止**依赖 auto-lang / auto-man / auto-gen / auto-vm / 任何 renderer/UI/构建栈；core 不见 `Target`/`TargetKind`/Git。闭包由 package-lock-consumer fixture + module-boundaries strict 基线钉住（man-ecosystem 域，`man-ecosystem-no-lang-monolith` 禁向规则适用）。
- Git commit 观测（`git rev-parse HEAD`）属 auto-man adapter（`lock.rs::get_git_commit`），不进 core。

## 冻结语义（PLAN-795 T-01 旧臂，579f6dec0 探针冻结；改动=行为变更须显式决策）

1. **格式字节**：TOML pretty（`automan_version` 先、`[[dependencies]]` 块、反斜杠路径取字面量单引号形态、尾换行）与 JSON pretty（2 空格缩进、无尾换行、空锁 `dependencies = []`/`[]`）的 golden 由 core/门面/consumer 三面测试钉死；Unicode 与 Windows 路径原样保序。
2. **错误阶段与文本**：读失败 `Failed to read lock file '<unified-path>': <io>`（路径为 AutoPath 统一化形态，反斜杠→`/`）→ JSON/TOML 解析各自阶段文本 → 序列化文本 → 建目录 → 写失败文本；verify 三臂错误文本与聚合并序冻结。
3. **首条命中**：重名条目查找/校验恒取首条；稳定排序保重名插入序。
4. **空 path 空.commit**：不作物化检查（历史行为，不借抽取顺手硬化/放宽）。
5. **路径存在 ≠ commit 校验**：有 commit 条目跳过路径存在性检查；verify 通过不是 Git 内容完整性或供给链证明。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| model | LockEntry/LockFile/LockedEntryView/ResolvedDependency + sort/find 单一规则 | active |
| verify | verify_dependencies 单一校验规则（错误文本/聚合序冻结） | active |
| codec | load/save/is_json_path 单一编解码（AutoPath 路径语义） | active |

## 验证

- core 单测+合同测试（`crates/auto-package-lock/tests/lock_contract.rs`）：格式冻结 golden（TOML/JSON/空锁）、格式选择（统一化/大小写）、错误阶段、路径存在性正负控（真目录/文件/缺失/有 commit 跳过）、聚合并序、重名首条、save 失败不写新数据（目录阻塞反控）。
- 外部消费方探针：`cargo test --manifest-path test/fixtures/modularity/package-lock-consumer/Cargo.toml`（4 例，含冻结 golden 与错误阶段）+ `bash test/fixtures/modularity/package-lock-consumer/verify-deps.sh`（normal/build 闭包无 auto-lang/auto-man/auto-gen/auto-ui/auto-vm）。
- auto-man 门面级端到端（Target 投影/kind 过滤/真 Git 观测/旧 API 编译面）由 `crates/auto-man/src/lock.rs` 测试钉住；该 pac.lock 与 PLAN-792 的 Cargo.lock 发行锁为不同合同。

## 已知债与未迁项（诚实登记）

- auto-man 的 resolver/Target/build/exporter/可执行模板等仍携语言栈，未随本抽取迁移；`auto-man→auto-lang` 边在 module-boundaries 基线 grandfather（exit R5）。
- 旧 `verify` 只覆盖缺项/版本/本地路径存在性——完整 Git 内容校验与供给链完整性不在本 core 声明面内；发现的锁校验弱点只登记后续行为修复，不借抽取改格式。
