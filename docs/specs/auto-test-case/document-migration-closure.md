# 文档式迁移收口（Document Migration Closure）

> 状态：delivered（PLAN-1062 全量收尾；本文为该迁移的权威收口口径）。

## 已交付事实

- 全部 75 个固定 case 组活跃作者文件为文档式：176 份旧围栏 case → `<stem>.document.case.ad`，
  1062 唯一检查对 + 95 既有文档位 = 1174 位置全 document；旧围栏行=0（逐组重投影核验）。
- B 层逐条目文档 1227+（vm 506/a2r 250/cookbook 133 双面/a2c 118/a2ts 85/a2p 99/
  conformance 33/ffi_dual 19）：原消费者（VM cache/TestRunner/各 helper）直读文档载荷，
  逐族 oracle 语义（bytes/trim/error-presence/no-crash）与 native ID、ignore/未注册
  原态保持；缺文档=硬错误。
- 冻结清单工具链：`document_case_migration.py`（manifest/pair/verify/controls）+
  `auto-case migration-inventory --schema-version 2 / helper-inventory`。

## 阶段门与证据复用

- 每迁移阶段：语义门（v2 投影 vs 冻结 manifest 全等）→ 运行门（新旧双侧实跑，
  expected-red 元 oracle：故意红控制组两侧同轴）→ 活入口核验 → 受影响固定组。
- 证据按身份复用（同代码/输入/配置）；表征白名单（origin 表征键、artifacts 载体、
  聚合 oracle_sha256 仅内容等价时可异）；expected-red/环境归因/原条件排除分列。

## 边界与后续准入

- 旧围栏/`.at`/`.expected.*` 为冻结历史：显式回放可用，活跃路径零回读（缺文档硬错误；
  doc-only 树仍可发现并执行）。
- 具名债务：D-P3-A2X-DRIFT（54 对 a2x 金样落后于现行 worker——AutoTrap 新增 LangError；
  关闭条件=金样按现行 worker 重制并独立审）。
- 环境归因：CWD 并发竞争（cb_file_png 等，同类已标 CWD-dependent）、mn 温存间歇
  infra ERROR（切换前同现）。
- 新增语料准入：新 suite/case 一律文档式；新 file-corpus 读者须登记（T-16 扫描口径：
  include_str/include_bytes/字面 test/ 路径），support fixture（单文件解析器/库资产）
  具名不迁；外仓（book/UI/Man/auto-term）归原 owner。
- conformance_bootstrap 生成器仅显式 proposal 模式可写 oracle（30 例闭集具名）。
