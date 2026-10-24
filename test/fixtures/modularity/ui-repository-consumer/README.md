# PLAN-831 ui-repository-consumer — 三仓验证设施（T-03 接口冻结）

本目录是 PLAN-831「auto-ui 独立仓承接与完整发行交付」的 fixture 与 driver 集：
五族 driver（source/type/test-id/distribution/failure）的具体接口在 T-03 冻结，
预迁移态的有效控制即刻执行；T-05..T-09 按 `migration-phases.json` 的
successor_execution 以最终来源完善执行。本目录不包含 cargo 消费 crate——
消费面（独立 clone 定向 `cargo check/test --manifest-path`、真实工程 build/run）
由 driver 在受管 scratch 生成/驱动，不因缺源偷偷读取主检出。

## 入口

```bash
# 有界三仓入口（推荐；解析序见下）
python scripts/ui-repository-fixture.py [--lang-repo P] [--ui-repo P] [--down-repo P] \
    [--phase auto|pre-migration|source-switched] [--output receipt.json] [--family ...]

# 单族直调
python test/fixtures/modularity/ui-repository-consumer/driver.py <source|type|test-id|distribution|failure|all> \
    [--lang-repo P] [--ui-repo P] [--down-repo P] [--phase auto|...] [--output receipt.json] [--compile-probe]

# 冻结断言区分性自检（合成仓受控突变 + 真仓合法对照；T-06 收紧态含在内）
python test/fixtures/modularity/ui-repository-consumer/driver.py selftest
```

三仓解析序（AGENTS 约定）：CLI 参数 → env（`AUTO_LANG_ROOT`/`AUTO_UI_ROOT`/
`AUTO_DOWN_ROOT`）→ 语言仓组内兄弟 `../auto-ui`、`../auto-down` → `D:/autostack`
主检出。收据记录各仓解析路径与 HEAD。有界性：默认路径零 cargo 调用（manifest
直解 + 三次 `git rev-parse`）；发行/失败族的重负载执行全部 phase 门控。

## 阶段模型（migration-phases.json，冻结）

| phase | 判别 | 五包位置 | 断言口径 |
|---|---|---|---|
| `pre-migration` | lang 含全部五包且 ui 无任何迁移包名 | 语言仓 | 五包闭包**无 auto-man 大库/无 lang-monolith**（auto-man/auto-man-runtime/auto-lang）；Man-core 边按冻结台账记档（现值 2 条：auto-ui-tools/auto-api-gen → auto-man-core） |
| `import-window` | lang 与 ui 均含全部五包（T-04 具名接口修订新增） | 双侧（权威=lang，候选=ui） | 语言仓权威面仍按 pre 口径断言；UI 仓候选闭包零大库/monolith + Man-core 仅限窗口台账（冻结台账 from∈迁移包子集，现值 {auto-ui-tools→auto-man-core}）+ 候选布线必须解析（SRC-P1b）；SRC-P6=设计态保留记档 |
| `source-switched` | ui 含全部五包且 lang 无任何迁移包名 | UI 仓 | 同一断言**收紧到零 auto-man-core**；UI 仓全成员零禁断边（含 legacy 清理）；语言仓成员零五包名（原件删除）；autodown 边必须可解析 |

部分导入/部分删除（判别均不匹配）= 中间态，入口拒绝作为门禁基线（exit 3）。
预迁移 legacy 归属（老仓 crates 面与 examples/）在 SRC-P4b 记档
`legacy_observed`（现值：legacy `auto-ui → auto-lang` optional 跨根边 1 条），
非 legacy 成员新增禁断边立即红（T-04 走私守卫）；source-switched 后一律红。

## 五族 driver 冻结接口

| 族 | V/AC | 断言 ID | 预迁移执行 | 完成点 |
|---|---|---|---|---|
| source | V-01/AC-01 | SRC-P1..P6 | **全量执行**（manifest 直解闭包；P2/P4a 判红，P3/P4b/P5/P6 记档） | P2/P4/P5/P6 收紧 @T-06 |
| type | V-02/AC-02 | TYP-C01..C16、TYP-D01..D04、TYP-F1、TYP-PROBE | 锚点钉住全量执行；D04/F1 记档；编译探针 pending | 探针+声明语义正反控 @T-05；切源后 D04 断言 @T-06 |
| test-id | V-06/AC-06 | TID-A1..A3 | 源码枚举清点（dialect=86/compiler=1101/runtime=2245/tools=229/shell-pack=4 @基线）+ 档位别名表 8/8 | 正典法 `cargo -- --list` 重采与对拍 @T-06..09 |
| distribution | V-03/AC-03 | DST-1..5 | pending（接口冻结；现行同构控制=generated-ui-consumer/driver.py，815-A） | 独立 clone/UI 仓 producer 实测 @T-07 |
| failure | V-04/AC-04 | FLR-1..6 | pending（接口冻结；现行同构控制=792 driver V03 + legacy-preservation.py） | 资源/锁/producer 拒绝与三仓切换失败控制 @T-07/T-08 |

- **source**：闭包=五包声明的 normal/build/dev（含 optional）path 边仓内递归 +
  跨根边记名；禁断判定按包名（对齐 777 manifest 直解主源口径）。
- **type**：锚点=今日实测源事实（touchpoint 表携带词边界锚文本）；C2 声明符号
  （sidecar_decl_from_root/SidecarDecl/pac_from_decl）在 {auto-man-core,
  auto-project-model} **恰居其一**——双在场=复制红、缺席=红；编译探针 use 路径
  由 touchpoint 表派生（单一事实源）。
- **test-id**：预迁移用源码枚举法（盲区显式：宏生成/include!/cfg 外属性不可见），
  正典基线归 T-06+；档位别名表快照自 `.cargo/config.toml`。

## 收据（ui-repository-fixture-receipt/1 / -run/1）

- 稳定性契约：同输入两次运行字节一致；无时间戳、无检出绝对路径（仓根以
  `<lang-root>`/`<ui-root>`/`<down-root>` 占位）；排序稳定。
- 断言四态：`pass`（判绿）/ `fail`（判红，exit 2）/ `recorded`（事实记档，
  阶段收紧后转判）/ `pending`（接口冻结未到执行期）。
- 退出码：0 全通过（recorded/pending 不判红）；2 断言红；3 用法/配置/阶段错配。

## 与既有控制的分工（复用，不重建）

- `scripts/modularity-audit.py --mode strict --no-cargo-check`：语言仓 777 边界
  基线（T-06 迁出后重采样，见 contract-baseline §5）；本 driver 不替代它。
- `scripts/legacy-preservation.py`：旧仓保全/恢复（V-05，T-02 已执行）。
- `test/fixtures/modularity/generated-ui-consumer/driver.py`：792/815-A 发行与
  失败控制的现行权威入口；DST/FLR 在 T-07 起按 UI 仓 producer 挂接同构执行。
- `test/fixtures/modularity/language-sdk-consumer`：纯 SDK 反向控制（DST-4 指针）。

## 接口变更协议

`migration-phases.json` / `compat-touchpoints.toml` / 本 README 构成冻结合同。
断言集、阶段语义、判别或锚点的任何变化都必须在 Plan 831 记录具名修订
（原因 + 受影响 V/AC + 修订时点的实测事实），不得静默放宽；收紧不受限。
T-05 切移声明符号、T-06 切源时，touchpoint homes 与台账预期值随记录更新。
