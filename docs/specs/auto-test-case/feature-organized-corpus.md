# 按语言特性组织的共享 AutoDown 语料（feature-organized corpus）

> Status: **delivered（PLAN-1078 AVP-00 / PLAN-1079 AVP-01，2026-11-19）**。公共语言测试按语言特性
> 集中组织：同一份 Auto 源码供 AV/AC/A2R 等后端各自追加目标检查，避免重复维护。
> 设计合同见 [docs/design/testing/ace-feature-organized-autodown-corpus.md](../../design/testing/ace-feature-organized-autodown-corpus.md)；
> 本文件记录**实际交付**的作者/选择/迁移/忽略机制与支持事实。

## 1. 作者与选择（交付）

- 唯一活跃作者根：`crates/auto-lang/test/cases/language/`（按语言行为命名，
  不以 AV/AC/A2R、M1/M2 或计划号划分）。公共总选择 `language.suite.ad`。
- 每个公共 Case 一份 `### Auto` 源码（或多文件工程闭包）；各后端在 Case 内
  声明检查。suite/固定组只引用、选择，不复制 Case 形成后端视图。
- 固定组 `case-av-language-parity`（domain=ace、显式选择）＝公共语料 **AV 视图**：
  runner 按 `check` 字段（`--check av`）只选 AV 检查；旧 AVM 对照只经显式
  avm 选择；`NOT_RUN`（未选中）不充 PASS，AV 视图门只看选中行（选中行 FAIL
  仍红）。

## 2. 检查级 `#ignore`（交付）

- 裸 `#ignore` 紧跟检查小节标题、位于属性之前 → 该检查映射 `CheckSpec.skip`
  （reason=`AV阶段暂缓非AV检查`、issue=`AV-ONLY-1078-1096`、owner=PLAN-1078）。
- 只影响该检查；不能标整 Case 或共享 Auto 源；AV required 检查不得被关闭。
- 忽略的检查不启动其绑定/编译/链接/执行/外部工具准备（FC-08：prepare/
  dispatch=0），不阻 AV 门；后端任务逐项移除本阶段 `#ignore`，保原
  oracle/平台条件/历史 ignore，按同一源真实运行。
- `#ignore` 不放松公共文档结构/schema 准入；未注册的未来 recipe 保
  Notes/义务表 pending。

## 3. 迁移映射（交付）

- `language/migration/legacy-map.tsv`：旧义务（repo 路径/Case ID/check ID/动作/
  源指纹/旧 oracle/条件）→ canonical Case/check + 原因 + owner + 状态
  （mapped / research-hold / no-such-shape / parser-only-hold）。数据映射，
  不作新注册器；旧 ID 不丢，合并不减义务。

## 4. 支持/边界

- AV-ONLY 阶段（1078–1096）：非 AV 检查加 `#ignore` 或保持未选；共享文档/源码、
  AV 必需公共编译与语义验证仍正确。非 AV required 关闭是对应后端/1068/985
  的最终门，AV 阶段的忽略不当通过。
- 平台：Windows 执行；Linux/WSL 腿归专用 Linux 环境。

## 5. AVP-01（PLAN-1079）增量

- 新增唯一作者文件：`language/numbers/literals-conversions.case.ad`（3 个 Case：
  wide-exact / mixed-width-explicit / overflow-is-not-wrap）、
  `language/text/interpolation-unicode.case.ad`（2 个 Case：text-nul-and-format /
  text-literal-escapes）。`language.suite.ad` 只追加两条 canonical 引用。
- 固定组 `case-av-language-parity` 复跑 8/8（1078 三例 + 本包五例），单例
  0.45–0.54s，均在 10s 预算内；非 AV 行继续 `#ignore`/NOT_RUN，不充 PASS。
- `legacy-map.tsv` 追加 68 行（PLAN-1079 主承接的旧 VM 文档）：保留原 case ID、
  check ID、声明 oracle 与 Auto 源 SHA16；分类 mapped 22 / research-hold 34 /
  no-such-shape 12。hold 行在 reason 里写明形态、owner 与关闭条件，不缩分母。
- 门禁控制（真实执行）：oracle 变异（成功例 `exit: 41`）→ FAIL
  `exit[exit] mismatch: expected "41", actual "Some(42)"`；全部 AV 检查 `#ignore`
  的视图 → 组级 `NOT_APPLICABLE` 且单独运行 exit=1（零选中不绿）。


## PLAN-1080 增量（2026-11-19，calls 家族）

`language/calls/binding-pipeline.case.ad` 为 calls 家族唯一活跃作者
（22 例：§6 正/负例 + 10 份旧 VM 迁移 + PIPE-NEIGHBOR/CALL-SOURCE-ORDER
补充 ID）；`language.suite.ad` 增 canonical 引用。迁移判等以整份旧
`.case.ad` 文件字节 SHA256 前 16 hex 对账（10/10 零漂移）；保留词冲突的
`double`→`twice` 改名是唯一具名审计规范化（语义/oracle 不变）。旧
`test/vm` 文件零改动（具名冻结历史）；legacy-map 本包行 owner=PLAN-1080。
故意红元控制（oracle 三处变异各红、复原绿）实证语料判决器真实。
