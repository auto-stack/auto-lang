# ACE M1 架构行证据准入合同（PLAN-1005，SD-03）

> 状态：active（架构行准入的现行可复核执行合同）。

## 49 行来源与状态

- 行分母：`architecture-obligations.json`（`ace.m1.arch-obligation-contract/1`，
  PLAN-941:r1，49 行/30 entry，声明 `execution: UNOBSERVED`——声明永不授权 PASS）。
- 唯一 review 行 `ARCH-08/fresh-union-spec-review`：**凭据专属**（见下）；其余 48 行
  由逐行 recipe 准入。

## 严格封套（每 entry）

真实单组 fixed-test 运行目录（`fixed-tests.py --out`）：gate.json（schema/kind/group/
commit/binaries/originals 五键）+listing 枚举经父解析器重 derive+discovery/run 命令
收据逐 argv（父 invocations/nextest_args 推导）+冻结窗口 source-pins（before==after、
⊆合同闭包）+当前 `test-groups.toml` sha+listing 二进制字节 pin+合同 source pin。
`executed=0`/空 listing/绿布尔×失败终端/executed 计数矛盾/源-配置漂移一律拒绝；
仅 ignored:true 条件排除合法。

## 逐行 recipe 与断言映射

- 合同 `architecture-entry-recipes.json`（`ace.m1.arch-entry-recipe-contract/1`，
  SHA 由 reader 模块 pin）：48 非 review 行，每行 facts=逐 entry 的具名
  case_checks/native_tests 全集；行 PASS=全部声明 entry 绿 AND 全部具名事实实际执行
  绿。删/ignored 任一具名事实→UNOBSERVED；红 gate→FAIL；事件名以 nextest listing
  为权威（lib 目标与独立 tests 目标形状不同）。
- 四个 composer 语义行绑 composer_facts 词表（admission-peak-within-live-cap 等），
  由组合器自身在准入时验证。

## 分域同边界

- `phased_cert_observations` 白名单 10 族；observations 携 ARCH 键或 section 载荷
  为架构形 schema/键的一律拒绝。分域 entry facts（`ace.m1.arch-entry-facts/1`）每
  entry 独立预算跑完整严格链，紧凑 hash 钉定。
- 架构合并单一信任边界：review 行从任何证据族注入拒绝；重叠 verdict 冲突拒绝（不再
  覆盖优先）；测量回灌（registered 升级路径）不得触碰 review 行。

## 凭据版本与可信交接

- 凭据 schema `ace.m1.pre-signature-review-credential/1`：plan/revision、frozen
  commit（∈gate commits）、候选 bindings+报告双 sha、行规则/registry sha、audited
  48 行/1006 键/唯一排除行、reviewer 三键、AC-01..09/V-01..13/SD-01..03 全覆盖、
  findings 全 resolved、verdict=pass；无最终报告自指字段（无环）。
- 消费链（`ace_m1_review_evidence`）：候选行与当前组合逐行一致、admission_sources
  一致、签前候选恒不可签、frozen∈gate commits、行规则/registry 逐字、缺失/坏
  schema/错 H/漏审/未决发现/自指一律不可签。机械验证只担内容/身份/覆盖；独立性由
  同树独立上下文的复审流程担保并记真实身份（不建 PKI、不宣称 hash 证明人的独立性）。

## 分域 facts 的原件重导与身份门（PLAN-1013，SD-01）

F-1013-01 修复（自报 facts 绕过原件准入+虚构 commit 入审计链）后的事实合同：

- entry facts 引用必须是 `{'facts': <紧凑投影文档>, 'root': <gate 目录>}` 双键形状；
  旧 schema（无 root、仅自报 `gate_originals_sha256`）明确拒绝，无自动 fallback。
- 消费时以 gate root 内**完整原始材料**复用 raw 准入（listing/discovery/command/
  run/results/窗口 pins 逐项核验）重导投影，再与 facts 文档逐项比对
  （commit/green/executed/kept/gate_originals_sha256）——摘要自带 hash 不构成可信
  锚；篡改投影并重算全部局部 hash 不改变原件决定的判决（V-03 类）。
- raw 与 facts 共用 `_verify_commit_object`：commit 须在正确仓解析为真实 commit 对象
  （`git rev-parse --verify <c>^{commit}` 只读核查）且为冻结锚成员；40 位格式或
  仓中存在均不足。`admit_arch_entry_gates` 接受 binding 级或参数级 `frozen_anchors`，
  输出 `gate_commits` 仅含已核验身份。
- 919 资源事件族注册出口词表（resource/cancelled）按冻结合同父判为准；ptr 类语料库
  fixture 腿的证据归属为其语料库格（undotted id corpus-first），scenario 格由其专署
  套件显式映射供应（producer token 为准）。
- 同一 48MiB LiveBudget 计入分域与 raw 的原件解析/投影：组合链一次性文档解码按
  release-after-extract 纪律释放估计（knife 1..3a），判决不变量 raw/phased 等价。
- 凭据合同升级为 PLAN-1013/r1（AC-01..10/V-01..15/SD-01..03 全覆盖）；旧 PLAN-1005
  R 或错误 revision/coverage 一律拒绝——旧 R 不冒新包已审。

## 拒绝语义与 TAG 门

- 三态：缺事实 UNOBSERVED / oracle 不符 FAIL / 格式身份错拒绝（exit2 路径）。
  父 FAIL 不因子/组 PASS 晋绿。
- TAG-01..06（候选/终验/凭据/负控/落地复验/标签范围）为打 tag 前强制复核；tag
  创建与推送是复核后的单独动作（计划只交付 tag_ready）。

## 主线身份物化边界与凭据适用版本（PLAN-1025，SD-01/SD-03，2026-11-13）

- **受控文本 LF 物化（SD-01）**：`.gitattributes` 将测试用例语料整体划 `-text`
  （字节身份），准入链受控文本（expanded 根 json/tsv、governance .py、fixed-tests.py、
  test-groups.toml、.cargo/.config toml、pinned docs、parent/native-module 源码、
  588 项 producer 闭包）钉 `text eol=lf`——Git 对象与 core.autocrlf=input/true 检出
  字节一致（验证 1424/1424 相同）。故意 CRLF fixture（scalar-src 两个）由 `-text`
  保留原始字节。1013 的 CRLF 工作树写入（7 处陈旧 pin）即此边界缺口。
- **当前凭据适用版本（SD-01 续）**：`ace_m1_review_evidence.py` 现绑 PLAN-1025/r1
  （AC-01..08/V-01..10/SD-01..03）；旧 PLAN-1005/PLAN-1013 凭据不可覆盖本包，凭据
  绑当前 registry 字节——registry 漂移即废（F-1025-02 持久控制验证）。
- **C 读回先于删原件（SD-03）**：正式验收通过后、任何原件删除前，须在实际合入提交 C
  的检出上重跑正式 CLI（本包 C=b923362f4 exit 0，报告与 H 逐字节相同
  efd1822a…）；主线合同红不可记债豁免；删除后仅摘要不冒重新准入。
