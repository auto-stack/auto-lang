# Auto 源码 text profile

> **Status**: experimental；`auto.source.text.draft` revision 1（PLAN-770）。
> 实现：`experimental/ac-core/src/source/`（与 core/scalar 共用前端骨架，profile
> 显式选择）；消费端见 [project.md](project.md)；姊妹 profile 见
> [source-core-i32.md](source-core-i32.md) 与 [source-scalar-i32.md](source-scalar-i32.md)。

本 profile 在 scalar-i32 之上交付 **str/Str/char 首批有限值语义**：文本/字符
字面量、拥有/借用转换、len/char_at/slice/concat/eq/ne 与显式码点转换。资源
语境（拥有 vs 借用、`.move` 转移、LIFO 出口清理）以 774 冻结合同
（[data-resource-contract.md](data-resource-contract.md)）为唯一接口来源；本
文件只定义 text 源码域的语法/语义/诊断合同。**不宣称**完整 token/lexer 编译、
容器/enum/record 文本字段、字素索引/normalization、f-string/raw/regex、生产
print/File/process、公开 ABI 或 ACC Gen1。

## 身份

- source `auto.source.text.draft` → HIR `auto.hir.text.draft / 1 /
  text-i32-draft`（774 interface-contract §7.2 唯一身份表；descriptor
  `experimental/ac-core/schema/text.atom`，scalar 的超集，只增不删）。
- 身份映射为**精确可失败映射**：未注册三元组整体拒绝；canonical writer 对
  未注册/错域一律拒绝（`canonical.identity-conflict`）；旧 core/scalar 不
  自动获得文本能力。
- verify 分域入口 `verify_text` 限本域；core/scalar/record 入口拒绝本域文档
  （`verify.domain`），本域对其他域面防御（`verify.profile-op` 双向生效，
  含 record 面——三域+core 支持集合互不扩大；`move` 仅 text、`mut` 仅
  text/record）。

## 类型与字面量

- `str`（借用 UTF-8 切片）/`Str`（拥有缓冲）/`char`（独立 Unicode 标量）；
  无注解 `let` 的文本字面量默认 `str`（借用语境）；拥有语境（`let s Str =`）
  字面量提升为 owned 构造（运行期 AP-4 复制）。
- 字面量形态：普通/跨行（行终结符原样入值，CRLF 保真）/白名单转义
  （`\n` `\t` `\\` `\"` `\'` `\0`）/`\u{1-6hex}`；非法转义
  （`text.invalid-escape`）、非法标量（`text.invalid-scalar`）、多字符 char
  （`text.char-multi`）、空 char（`text.char-empty`）在**词法层**拒绝——
  未执行分支同样做静态合法性检查。
- 无隐式 char/int 互转。

## 索引与比较

- `len`/`char_at`/`slice` 按 **Unicode 标量单位**（非字节、非字素）：
  `A中😀` len=3、0/1/2 对应三字符；组合 e+U+0301 len=2。
- `len_bytes` 是显式 **UTF-8 字节长度**（`A中😀` len_bytes=8）——与标量数
  不可混称。
- `slice [start,end)`：空范围合法；负数/反向/end>len 为运行期 RT-FAIL
  （-9/-10）+trap70，不静默裁剪。
- `==`/`!=` 字节精确比较（NUL 不截断、无 normalization）；文本排序不在本批。

## 资源与语境（774 D-03/04 消费）

- 逐语境 str/Str：`.to(Str)` 显式提升、`.to(str)` 借用化不转移；`+` 双文本
  →拥有 concat；`.clone()`；`=` 对 owned 目标拒绝
  （`verify.resource-assign`）。
- owned 局部**块词法作用域**：owner 出作用域（块自然结束、break/continue
  的 loop 作用域 frees、Drop）后经 backing 的读取为
  `verify.owned-scope-escape`（构造代际：循环重入后 fresh `let` 重定义资源
  身份，旧构造的 view 读作陈旧）。
- 借用不外逃：str 局部存拥有临时视图（`verify.borrow-escape`）、返回借用
  读 owned/mut 来源拒绝；借用参数源身份进别名集。
- `.move` 调用点转移：参数须声明 `move Str`（`verify.ownership-required`）；
  转移后使用 `verify.use-after-move`/`verify.use-after-free`；同语句
  view+move 重叠 `verify.mut-conflict`；二次转移 `verify.double-move`；
  短路条件 move（未知极性）`verify.ownership-required` 拒。
- **清理责任不变式**（r4..r10 迭代冻结）：
  - join（if/循环出口）只合并**真正进入**该出口的状态（常量可达性剪枝；
    终止臂不污染 continuation；不可达 break/continue 不产生可执行出口）；
  - 循环重入按 **RE-ENTRY 状态 fixpoint** 重走（自然完 + continue 出口；
    break/return 终止体不伪造回边；dead/id 投影 backing 单调收敛）；
  - **after-loop 清理义务按 incoming 三态**：真实自然完（仅有回边时）+
    全部 break 出口逐状态判定——全 live caller 保持义务、全 dead 移交
    callee、**混合 `verify.ownership-required` 拒**（零次迭代/移交前
    break 绝不 LIT 缺 FREE，也不双释放）；
  - **参数对排他约束跨函数传播**（fixpoint）：`mut` 参数与任意其他参数的
    源身份成对约束，包装函数把自己的形参对再出口到调用边界
    （`verify.mut-conflict`；move 参数非别名载体）；
  - 语句尾寿命：拥有临时活到**完整语句尾**（后续 mark 先于 FREE）；短路
    RHS 临时按实际执行路径经 join 块参数携带（skip 边 null），跳过侧零
    分配零 FREE、执行侧一次配对；LIFO 出口清理由 verifier 的
    TextResourcePlan 冻结、native 只消费（帧恒小 + 内联探测帧，4096 链
    安全）。

## 方法解析

- 内建方法按（接收者类型,方法名）绑定：`len`/`len_bytes`/`char_at`/`slice`/
  `clone`/`to`/`to_codepoint`/`from_codepoint`/`move`；用户同名函数按裸名
  绑定，两命名空间不冲突（遮蔽优先）；`.move`/`.to` 括号形态固定。

## 能力与运行期合同门

- 文本模块（真实 runtime 调用**或**清理义务）须声明并显式提供能力
  `ac.runtime.text.v1`（`capability.missing` 前门；需求**派生**自实际
  内容，caller 声明不可绕过）。
- `--support-lib` 携带伴随 `<lib>.contract.json`（api/revision/target/
  lib_sha256 对真实库校验）；失败码 `runtime.contract-missing/-hash/
  -target/-api` 在发布前拒绝；收据记录库+合同指纹。
- 运行期出口策略：负状态 → runtime 先写 `RT-FAIL code=<n>` stderr 行 →
  生成代码 ExitProcess(70)（70 为测试 profile trap 码；char_at 越界冻结
  ABI 为 -10）。

## 边界

- 无 List/Map/enum/record 文本字段、字素索引/normalization/f-string/raw/
  正则、生产 print/File/process、公开 ABI、ACC 桥/迁移/自举。
- 预算/故障注入无语言入口（runtime 控制面，归 runtime 测试族）。
- trace 支持库与文本运行时两个 no_std staticlib 不能共链一个 PE
  （duplicate `rust_begin_unwind`）；mark×text 共链面由
  `test-support/marks.rs` 纯 COFF 对象承载。
- text-record 组合 profile 未立项：770/771 各自 pass 不构成组合验收。

## 验证状态

- 测试族：text_binding/text_hir/text_source/text_native/text_resources +
  review 重放集 text_review_r2..r10（每轮冻结反例/正控/组合 oracle 永久
  化，合法 run 附 alloc=free 通用断言）+ runtime/text 独立 workspace 三档；
  联合门禁 `scripts/verify-ac-text-770.ps1`（14 步，含 741/745/767/771
  record gate）。收据见
  [../../reports/770-text/verification.md](../../reports/770-text/verification.md)。
