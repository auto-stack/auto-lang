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

## 拒绝语义与 TAG 门

- 三态：缺事实 UNOBSERVED / oracle 不符 FAIL / 格式身份错拒绝（exit2 路径）。
  父 FAIL 不因子/组 PASS 晋绿。
- TAG-01..06（候选/终验/凭据/负控/落地复验/标签范围）为打 tag 前强制复核；tag
  创建与推送是复核后的单独动作（计划只交付 tag_ready）。
