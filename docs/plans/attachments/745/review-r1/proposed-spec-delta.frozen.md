# PLAN-745 proposed Spec delta（work 阶段提案，review 定稿，merge 沉淀）

> 执行只写本提案；canonical Specs（docs/specs/**）、模块 plans/index、
> `.autoos/specs.json` 与 ledger 在 T-09 /auto-plan:merge 才写入。
> 每条 delta 绑定 r1 合同 + 实测证据（[verification.md](verification.md)、
> [numeric-boundary.md](numeric-boundary.md)），不重开 GOAL-017，不把有限
> adapter 算 Gen1/2/3。

## SD-01（add）docs/specs/auto-ac/source-core-i32.md

新增模块 Spec：`auto.source.core-i32.draft` 源码 profile 合同。要点（全部
以 745 r1 §5 为合同、以 74 例实测为证据）：

1. **数值映射**：profile 内 int/i32 同义 = 精确有符号 32 位；
   无后缀十进制字面量域 [-2147483648, 2147483647]（宽临时解析 + 双侧域检查，
   越界 `text/source.literal-range`）；`-2147483648` 与 `- 2147483648` 是单个
   有符号字面量，负号仅允许直接后随数字；add/mul 运行期 signed 溢出 trap
   （ExitProcess 70），常量表达式不做编译期求值/拒绝；lt 为 signed 比较；
   Bool/I32 无隐式互转；char/float/i64/u32/u64/uint/usize/后缀/进制一律拒绝。
   全语言 int 宽度不因此冻结（MD-409 unknown 维持，见 SD-04）。
2. **支持面**：单模块函数型；参数/返回必须注解（int/i32/bool）；let/var 可省
   注解（推导 I32/Bool）、始终初始化；显式有值 return；if/else、while、loop、
   无标签 break/continue、局部赋值；`*` > `+` > `<` 左结合；括号；positional/
   named 实参（混用拒绝）；直接/互/前向递归。终结符后语句、分支/循环局部
   逃逸、同块重复声明等按 741 既有 verify 保守规则拒绝。
3. **来源与可信边界**：生成 Atom 的每个节点经条目序 span 再锚投影回原 .at
   （token 碎片映射为回退；post 检查 = 范围/UTF8 边界/无生成位置残留；缺映射
   → 受控 `source.internal` 诊断，不伪造 source 错误、不 panic）；合成 bool
   （0<1/0<0）与 while 降低（loop{if cond{body}else{break}}）指回 literal/
   while 条件 token；诊断 code 集 =
   text/{source.lex,source.syntax,source.unsupported,source.literal-range}、
   bind/{source.undefined,source.duplicate,source.argument,source.not-callable}、
   verify/source.type-mismatch + 既有 verify.*/bind.*。
4. **CLI**：check-source/build-source 必须显式 `--source-profile
   auto.source.core-i32.draft`（漏/未知 = 2）；成功/流水线拒绝/用法错 = 0/1/2；
   build 入口是源码函数名（零参数、int/i32 返回）；源码 profile + 输入 hash
   写入 .ac-link.txt 同一发布事务；后端只接本进程 verify 产生的 CheckedModule，
   无 VM/A2R/旧 AST 回退，模式不按扩展名猜测。
5. **trace intrinsics**：mark_a/mark_b 仅由 `hir.test.trace.v1` 激活封闭
   nullary→int 表；未提供能力时 mark 调用是 source.undefined；用户函数/局部
   同名优先，不能名字劫持；无值调用语句（N41）在 text 层拒绝。

## SD-02（modify）docs/specs/auto-hir/stage-contract.md

S0 从“未实现”改为：**仅本 profile** 定义 S0（源码 → span 化 token/AST →
词法/类型解析）→ S2（adapter 生成 Atom 文本 + bind_source）；HIR 三重身份
`auto.hir.core.draft / 1 / core-i32-draft`、Schema/运算面/pass 状态不变，
source 构造与 Checked/pass 分界保持（verify 仍是 Checked 唯一构造路径）。

## SD-03（modify）docs/specs/auto-ac/project.md

在“HIR→native→PE”证据之上追加“A1–A3 源码形态”证据：Auto 源码子集经
显式 profile → Checked HIR → Windows PE 原生执行（21 例退出码/stderr oracle、
含算术/循环/递归/异构命名/求值序/trap/分支语义），收据见 verification.md。

## SD-04（modify）docs/specs/auto-acc/project.md

MD-408/409 相关边界收敛：int/char 从“全 unknown”改为“仅 source profile
int 有精确 32 位证据；完整宽度与 char↔int 映射仍 open”。提案 note 追加文本：

- **MD-408**（cap:char）追加：
  `〔745 T-01/probe-char-int：source profile auto.source.core-i32.draft 拒绝 char（字面量/注解/转换均 source.unsupported，N20）；三宿主表示并存取证（VM builtin 单字符 str：string.rs:863-891；auto-lib char_at→int 码点：lexer.at:44-52；Value Char(char)：value.rs:82-97），char↔int unknown 维持，完整 char 语义保留 open。证据：docs/reports/745-source-core-i32/numeric-boundary.md §5/§6.2。〕`
- **MD-409**（cap:int-ops）追加：
  `〔745 T-01/probe-int-width：source profile 内 int/i32=精确有符号 32 位（[-2147483648,2147483647]，越界 source.literal-range；运行期溢出 trap=70；负号仅直接后随数字）。全语言宽度仍 unknown：VM 栈 i32、值存储 i32|i64、旧 AST 按幅值 i32|i64|u64、trans 目标 i64 并存。本 profile 仅 add/mul/lt（候选②未变）。证据：docs/reports/745-source-core-i32/numeric-boundary.md §2-§4/§6.1。〕`

两处均不改 conclusion/证据绑定（required 维持；bound hash 未过期——
auto/lib/lexer.at 复核一致）。

## 盘点工具检查（T-06 实测）

```
$ python scripts/acc_inventory.py --root . --output docs/reports/743-acc-hir-contract/inventory.md --check --require-decisions
ERROR[artifacts-absent] 先运行 --write 生成 source-manifest.json/scan-summary.md
ERROR[decisions-required] 完成态要求 manual-decisions.json 存在
CHECK-FAIL 2 项
```

解读与策略（不自动重 hash 掩漂移）：两项失败均为 743 扫描器的工作区态——
source-manifest.json 本就不入库（743 收据 scan-summary.md 已入库）、
`--require-decisions` 完成态门在无 manifest 时先行失败；**非 745 改动引入的
内容漂移**。盘点刷新（--write 重扫 + MD note 绑定刷新）是 merge 阶段动作，
由 ACC 线 owner 在 T-09 沉淀时执行；本提案不代跑、不改写 743 产物。

## 新鲜度绑定策略

- 本提案绑定：r1 合同（plan_revision 1）+ worktree 提交链（c535b8fd6 →
  addcf4d5a，见 verification.md §1）+ 74 例实测收据（脚本 context.txt 的
  HEAD/binary sha256）。
- review（T-08）按同树同 HEAD 复核；若 HEAD 移动或证据过期，受影响条目
  重验后才能定稿。
- merge（T-09）落 canonical 时重跑 `verify-ac-source-745.ps1` 并以当日收据
  替换指纹；MD-408/409 note 追加按落地时 hash 重绑（741/743 先例格式）。
