# PLAN-807 work → review 交接（handoff）

2026-10-18。work 树：`D:/autostack/.wt/lang-807/auto-lang`，分支
`codex/plan-807-dev`，base `2046a025b`。本文绑定送审冻结 SHA（T-07 提交）
与全部验证结果；review 在**同一工作树**独立复核。

## 1. 代码身份

- base commit：`2046a025b`（master）
- 送审冻结提交：`1f5e05141`（HEAD；§8 清单末行）
- 依赖修订：workspace +`crates/auto-text-core`（imara-diff 0.2 唯一三方；
  dev-deps serde_json）；auto-lang +path dep auto-text-core、−直接 imara-diff
- 组内兄弟：`D:/autostack/.wt/lang-807/auto-down`（detached @c39ac7f，
  autodown-core 路径解析用，零改动）

## 2. 交付形态

- `crates/auto-text-core`：rope.rs + rope/file_backing.rs + diff/{mod,envelope,dirs}.rs
  逐字迁移（git mv 保历史）+ 冻结最小公开面（见 contract-baseline §3）。
- 预取所有权闭环清偿：worker 改持 Weak；`PageStore::worker_exited_handle()`
  每店退出观测；crate 内永久回归 + auto-lang 层 Windows delete-pending 硬观测。
- 旧路径薄门面（类型/函数身份不变）：`ui/code_editor/core/rope.rs`（7 行）、
  `ui/code_editor/diff/{mod(17),dirs(5)}.rs`；`diff/envelope.rs`（31 行）= UI
  adapter：key→snapshot 查找（`code_editor_with`）+ core 纯形调用。
- 快照端口：`vm/host_capabilities.rs` 第 8 槽 `EditorSnapshotProvider`
  （独立 trait，802 trait 零改动）+ NI/AutoVM set_/accessor + `new_inner`
  构造期快照 + Clone/merge 保留；UI adapter
  `EditorSnapshotRegistryProvider` 挂 `code_editor_provider::install_process_default`
  同四点位 if-absent。
- 四 diff shim（9915/16/17/21）与 a2r_std::diff 四函数 on 臂零 `crate::ui`；
  off 臂 raise/panic 文案逐字保留；`ui_gen/rust.rs` 发射串零改动。
- 777：domains +`auto-text-core = lang-core`；基线重采（base_revision 见
  §6）；792：`DELIVERY_CRATES` 12→13。

## 3. work-owned 验证收据（V-01..V-06）

| 项 | 命令 | 结果 |
|---|---|---|
| V-01 身份 | `cargo test -p auto-lang --lib plan807`（TypeId 同型/跨 API 传递/剪断锚/真编辑器门面==纯形逐字节） | **7/7 PASS** |
| V-01/V-06 独立消费 | `cargo test --manifest-path test/fixtures/modularity/text-core-consumer/Cargo.toml` + `bash …/verify-deps.sh` | **3/3 PASS；闭包=auto-text-core→imara-diff，零 auto-lang/man/ui/vm** |
| V-02 字节/持久性 | `cargo test -p auto-text-core`（rope 29+分页 12+diff 24 族随源迁移零漂移，含 1GB 谱 3 个 #[ignore] 显式位） | **65 PASS / 3 ignored（显式谱位）** |
| V-03 资源/保存/退出 | `AUTO_LANG_P728_BENCH=1g cargo test -p auto-lang --lib plan728_bench -- --ignored --nocapture` | **PASS：1,073,741,834B/23,280,107 行；结构驻留 3,033,703B ≤ 12MB 合同；answer 4.15ms；edit_50 8.4ms/远端 max 0.247ms；save 1,464ms；typing-paged-1g p50 1.04ms/max 5.73ms；谱行入 docs/reports/p728-bench.jsonl** |
| V-03 退出观测 | plan807 `paged_base_handle_released_after_last_drop`（Windows）+ crate 内 `prefetch_worker_exits_after_last_external_release` | **双 PASS（snapshot 存活期不退出/可读；最后释放后 worker 受控退出、同名重建成功=句柄关闭硬观测）** |
| V-04 四 native 语义 | `cargo test -p auto-lang --lib plan703|plan710|plan716`（VM catalog 真调用、缺 key、feature 开关臂、窗口/错误值形） | **7/7 + 22/22 + 14/14 PASS（冻结矩阵零漂移）** |
| V-04 端口/隔离 | plan807（构造式缺席/快照隔离/实例覆盖/双 VM/Clone·merge/不动 revision/缺 provider=missing key_a） | **含于 7/7** |
| V-05 diff 语义 | `cargo test -p auto-text-core`（golden 八形、704 D-1/D-2、716 窗口语义、dirs 五态族随源迁移）+ 探针族 | **随 65 PASS + plan716 14/14** |
| V-06 外部发行 | 上行 consumer fixture + 777 strict + 792 driver | **见 §6/§7** |
| 旧臂零漂移复跑 | plan728 supply probes（经门面） | **5 PASS/2 ignored（谱位）** |
| feature-off 编译 | `cargo check -p auto-lang --no-default-features` | **绿（off 臂矩阵完整）** |
| 新 crate 卫生 | `cargo check -p auto-text-core` | **零警告** |

## 4. 去重审计（旧算法零残留）

- 算法符号（imara/poly_digest/histogram_changes/anchor_partition/intern_lines/
  split_lines_universal/group_hunks）在 auto-lang src **零命中**；
- 四门面文件共 60 行（7+17+5+31），唯一算法邻接代码 = envelope adapter 的
  key 查找（UI 状态，设计归属）；
- vm/native.rs、a2r_std.rs 对 `ui::code_editor::diff` **零引用**（剪断锚
  永久回归钉住）；
- ui_gen/rust.rs 发射路径经门面续存（下游金样零扰动）。

## 5. 剩余 UI 边（诚实清单）

1. `ui/code_editor/diff/envelope.rs`——key→snapshot 查找（注册表=UI 状态；
   算法零残留）。
2. `ui/code_editor/core/mod.rs`——编辑器内核消费 rope（分页装载阈值/布局
   窗口/预取触发留 UI，合同 §7 冻结）。
3. `ui/code_editor_provider.rs`——快照 adapter 安装（四点位）。
4. `PageStore::prefetch_byte_range/file_len` 为 UI 消费点升 pub（冻结 API）。

## 6. 777 收据

- domains 登记 + 基线重采：`base_revision=3717b2164`，declared_edges=61
  （新增 auto-lang→auto-text-core，lang-monolith→lang-core 合法抽取边）。
- strict：`python scripts/modularity-audit.py --mode strict --no-cargo-check`
  → **verdict=pass violations=0 grandfathered=2**（既有窄豁免，未放宽）。

## 7. 792 收据

- 登记：`DELIVERY_CRATES` 12→13（`+auto-text-core`，rust_ui.rs:3988）。
- driver：`python test/fixtures/modularity/generated-ui-consumer/driver.py
  --scratch D:/p807-scratch-792`（正式入口、真实生成/fresh 解析/locked
  构建/重放/负控全档；cargo 1.98.0）。
- 结果：**25/25 PASS**——manifest = 13 源 crate（含 auto-text-core
  229,843B/44 文件）+ 1 patch 条目；V01 fresh resolve（5s，patch 单源 +
  pin 全命中）+ 真实 build --locked（214s，产物在位）；V02 重定位 locked
  build（226s）+ metadata 单源/越界 path=无 + 资源字节一致；V03a-j 全部
  负控确定性红（tamper/missing/corrupt/unlisted/missing-patch/bad-pin/
  foreign-dir/redelivery 保旧/sidecar 外部 path/根相对/staging 兄弟保留/
  raw TOML/他应用交付存活）；V04 dev 模式与交付输出逐字节同源。
- 收据：`attachments/807/receipt-792-driver.json`（driver 原始输出）。

## 8. 提交清单（work 树）

| SHA | 内容 |
|---|---|
| `bc2fc322c` | T-02 rope+file_backing 迁入 + 闭环修复 + 门面 |
| `75f492fc8` | T-03 diff 三件迁入 + 纯快照形 + 门面/adapter |
| `6a931eb20` | T-03 卫生（envelope mut 警告清零） |
| `d340bd0a8` | T-04 快照端口 + 四 shim 接线 + UI adapter |
| `7cb1239b4` | T-05 a2r_std 四函数接线 |
| `43c077e0d` | T-06a/b consumer fixture + plan807 上层回归 |
| `3717b2164` | T-06c/d/e 1GB 资源测 + 777/792 登记 |
| `1f5e05141` | 777 基线重采（**送审冻结提交**，本 handoff 绑定） |

## 8b. SD 草案（merge 沉淀用；canonical 编辑归 merge）

- SD-01（add `docs/specs/auto-text-core/project.md`）：crate 边界/公开面
  （rope/分页/快照/diff/envelope）、支持面与诚实边界、依赖纪律
  （imara-diff+std）、闭环修复的寿命合同——取材 contract-baseline §3/§7。
- SD-02（modify `paged-rope.md`）：实现路径迁移注记（ui/…/rope →
  auto-text-core::rope，门面身份不变）；全部分页不变式原样保留；
  `worker_exited_handle` 观测面新增；50MB 阈值/2MB 窗口仍留 UI。
- SD-03（modify `vm/overview.md`）：diff 四项 9915..9921 消费面
  （auto-text-core 直调 + EditorSnapshotProvider 端口）；缺席/feature 矩阵
  （缺 provider=无法找到 editor→err envelope 值形；off 臂 raise 逐字）。
- SD-04（modify `ui/overview.md`）：editor/Rust 消费新 core 单源；注册表
  adapter（envelope key 查找 + 快照 provider 安装点）；旧路径薄门面兼容
  （ui_gen 发射零扰动）。

## 9. review 注意事项

- review 入口：同树 `D:/autostack/.wt/lang-807/auto-lang`（独立上下文，
  不新建同步树）；AC-05 复核清单见计划 §7。
- merge-owned 门（**pending，不由 work/review 执行**）：冻结提交上裸
  `cargo t` + `cargo tv`（VM 触面）+ `cargo tu`（Rust 生成接线触面）；
  V-03 资源测绑定条件（源/依赖/配置/输入/编译形态未变）已在 §3 记录。
- 806 会话并行在途（另一工作树）；公共写面本计划只增自己的片段（根
  Cargo.toml members+exclude、tests.rs 一行、host_capabilities 追加节、
  777 基线——后入者取并集）。
