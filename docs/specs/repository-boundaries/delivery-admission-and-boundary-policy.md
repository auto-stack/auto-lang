# 发行准入与边界策略（PLAN-918）

> **Status**: active；owner为language test-governance / modularity-audit；实施与证据见
> [PLAN-918](../../plans/archive/918-cross-repository-admission-and-boundary-closure.md)。

## 1. Web 发行收据的实际执行 ID 准入

语言侧收据消费者 `scripts/test-governance/web_delivery.py::validate_distribution_receipt`
对 `independent-sdk-tests` 腿必须验证**真实唯一执行身份集**，不接受仅 summary 计数或行数：

- 冻结的正式 nextest 人读输出（非 TTY 重定向，无 ANSI 控制字符）：
  `PASS [ <dur>] ( n/m)? <binary> <test-full-name>`（PASS/FAIL 行带 8 空格缩进、Summary 行 5 空格缩进；解析须容前导空白）；binary 与 test 全名之间以空白分隔，
  test 名可含 `::` 与参数化后缀、不含空白。
- `execution_ids(raw)` 把 PASS 行解析为唯一 `binary\ttest` 身份；**重复/FAIL/未终态/畸形行、
  ANSI 控制字符、空日志**以及 **summary 计数（run==passed）与实际唯一执行数不符**均精确拒绝；
  不得以 set 去重、按行数或 summary 创造执行。合法同名跨 binary 是不同身份（同 test 名、
  不同 binary 均计入）；原 skip 条件由 summary 的 2 skipped 与调用点正则共同约束。
- 直接腿与复用腿（`reused_receipt`）同等严格：复用收据仍须核对
  `web-independent-tests/1` schema、217/2 计数、SDK 输入/config/log 哈希与文件身份。

### 1.1 独立 required 集合与 producer/consumer 差额

required 集合来源 = 现声明 SDK targets（producer 命令
`cargo nextest run --manifest-path sdk/Cargo.toml -p auto-web-contract -p auto-web --lib
--features auto-web/test-http-e2e …`）的 nextest listing。当前 `web-independent-tests/1`
收据只携带计数与输入/日志哈希、**不携带执行 ID 集**，因此 ID 对 ID 级对账（等计数缺原
ID+加伪ID）在 consumer 内不可判——该 producer/consumer 兼容差额归 Web schema 决定：
Web producer 如需 ID 级证明，须在收据 schema 增加版本化 ID 集/有效 listing 字段；语言
侧不得擅改 Web schema，也不得把 consumer 门绿当 Web 发行全绿。语言 consumer 在现 schema
内完成的校验边界即 §1 所列。

## 2. 逐边精确设施准入（PLAN-896 owned 设施）

`[[rules.facility_admission]]` 是**准入规则**而非有退出阶段的 grandfather：逐边、带设施
身份与 std-only 约束，不重标包域、不扩大 from/to 域、无目录/前缀豁免。

- 匹配语义：from/to 为精确包名（同仓 member identity）、kind 限 normal、非 optional、
  无 target；外置替身（外部同名包）、build/optional/target 形态不构成合法设施消费。
- `require_std_only_closure=true`：设施包自身 normal/build（非 optional）声明闭包必须为空
  （仅 std）；dev/bin 测试条件如实分开。闭包被破坏 → 准入失效并判
  `facility_std_only_closure_broken`；未被任何禁向规则点名且非 dev 的伪装形态判
  `facility_form_masked`；其余形态按原禁向规则判红。
- 判定输出以 `admitted_facility_edges` 具名命中的合法设施边。
- 本仓现行唯一准入边：`auto-cache → auto-tool-execution`（PLAN-896：FFI sandbox/methods
  pack 的 cargo build 与 nightly rustdoc 由 std-only auto-tool-execution 持有/捕获/等待/回收
  编译树）。上层（lang-monolith→man-ecosystem 等）、反向与同域邻居、auto-tool-execution
  新增第三方/语言/UI 正常依赖均保持禁向。

## 3. 不可解析锚的恢复纪律（baseline 维护）

- 旧基线锚不可解析（git 对象缺失）时：保留旧基线 SHA256 与“不可解析”事实，不得伪造
  旧提交差异；用保留的 baseline edge 集合、当前实际 manifest/ownership、已交付规范与来源
  逐项重建决策账。
- 新锚只允许在**干净已提交的本组来源树及干净参与仓**（含跨仓 text_scan 目标仓）上
  `--refresh-baseline` 生成；写入目标受只读守卫保护。基线文件自身落库不触陈旧；后续
  manifest/配置变更按 `staleness_paths_changed` 判 stale，由实际变更 owner 按同一流程
  重采样，不做永久豁免、不允许 `--no-cargo-check` 结果代替最后锁定 metadata 一致性。
- 未知新增边不得随重采样合法化：strict 真实完整通过且无未解释新违规；新规则而非新基线
  吞掉合法设施边。

## 4. 维护触发与各 owner 交接（SD-04）

- 031（UI 主题发布）可独立继续、不等本策略；其正式合入受管 manifest 后基线按 §3 自然
  stale，由实际变更 owner 重采样。
- Web D-906-1（vendor 暂存/锁漂移）、Man D-910-2（`-d` rust 派发）、UI 旧红、真实 OS
  事件与批回归 user-pending 均保原 owner；语言控制门的绿不替代外仓产品/发行实际执行。
- 后续无后继维护时，由变更 owner 触发本节与 §1.1/§3 流程。

## 5. PLAN-930 交接（SD-04，as-built）

- 本包**不触碰任何 Cargo manifest/依赖边**（lang 侧=driver/控制模块/规范；
  UI 侧=rust_ui.rs 投影源码）→ modularity-audit 冻结基线不因本包 stale；
  strict 复验 violations=0、self-test 16/16（V-06 负控）。
- 三具名原红（036 V04×2/V05a）已闭合并登记长期守门；UI 既有 shell-pack
  过期红（pack 改未 regen）与 jet/ark 环境门控、其他外仓债保原 owner，
  不因本包绿自动关闭、不扩大准入。
- 固定组 generated-delivery-controls 为 language 侧守门入口；跨仓真实
  构建义务仍由原 driver/UI owner 门承载。
