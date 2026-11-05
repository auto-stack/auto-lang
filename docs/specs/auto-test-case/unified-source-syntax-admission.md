# auto-test-case 统一源码语法准入（SYN 门）

> **Status**: experimental（PLAN-929 SD-06 交付；实现=`crates/auto-lang/src/tests/syn_report_929.rs` + `test/cases/ace-m1/unified-source-syntax-929/`）
> 来源：PLAN-929 §5.4/§6；上游适用规则见本目录相邻文档。

## 定位

SYN（语法统一）门的父侧准入合同：可对语法统一签章，**不冒充 ARCH 总绿**
（SQ/INST/CT/CA-D 另有 owner）。

## 报告 schema：`ac-source-syn-report/1`

行式 `key value`；键白名单 + 四轴计数键族前缀放行（`p./m./r./io./budget.`）。

- 身份绑定：schema/plan_id/run_id/window_id/producer_commit/source_sha256/
  grammar_id/caps_version/profile/target（构建位/语法身份/源 sha 与父侧
  期望不符 = STALE 拒）。
- P 轴：阶段 → 真实 attempt 计数（失败 attempt 也在内，无 Default-0）。
- M 轴：重建/隐藏工作；R 轴：恢复深重验（合法非零）；IO 轴：字节账；
  budget.*：预算峰值引用。
- 终局枚举：ok/source-error/resource-exhausted/cancelled/infra（分态，
  不吞资源态为源码错）。

## 父 decoder 严格规则（拒绝面）

1. 缺字段/未知键/错类型/枚举外值/坏计数 → 拒。
2. **伪 0 结构拒**：whole=0 而 M/R/IO 显示整源工作且无 dump-restore R
   记录 → 结构性矛盾拒。
3. STALE：身份三面（构建位/语法/源）对账失败拒。
4. 父侧不只用自填字段：交叉核对独立结构门与真实产物（篡改矩阵为
   负控面）。

## 文件化面

`case-ace-unified-source-929` 组：九 profile parse 腿（共享前门缓存，
字节无损重建）+ profile 门负控（record@scalar 语法必拒）+ AV 源执行腿。
深恢复/无预热首编辑/清单准入的 in-process 反控由 ace-source/ace-a2x
固定组承载。
