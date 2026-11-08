# auto-abi / resource-and-callback-bridge

> **Status**: candidate（`autoabi.win-x64-v0.1-candidate` 资源扩展，PLAN-939 r3 delivered）
> 路径：`experimental/ace/crates/abi`（worker/cside/resource_provider/worker_source）+
> `crates/auto-lang-core`（父判定）+ `crates/auto-test-case`（契约）

有限 foreign 资源、句柄与同步回调桥的已交付现状合同：在 937 中立契约与
938 受检 source 调用之上，把 buffer view、record const/mut/out、opaque
handle 与同步 per-call callback 接到双真实消费者（native/A2C）。资源分析
唯一权威在 core verifier；后端只消费中立投影，不复制 ownership 推导。

## 支持矩阵（方向 × 形态）

| 公开形态 | NI/CI（source import） | CE（source export） | TP（typed peer） |
|---|---|---|---|
| BufViewBytes/Utf8 | 支持（`t939_byte_length/scalar_count` 族）；NUL 为数据、len=字节 | 支持（`ExportLane::ViewText`：公共 `(data,len)` 对→`a2x_str_view`） | S07 字节/Utf8 |
| record const/mut/out | 支持（纯 record 闭包=自然对齐标量、指针 view/out；mut 排他、shadow/commit） | mut 支持（公共闭包指针+回写；const 需 `view` 参数语法=冻结面边界） | S07 RecMix |
| opaque Handle | 支持（provider 代际/唯一 destroy/声明式 clone hook；owned 移动/lease） | 不承诺（typed peer 另证） | S07 |
| multiout/BufOut | 支持（事务：全部准备→成功一次 commit；失败 caller 字节逐字节不变，半树原方回收） | — | S05 原列表 |
| CallbackFn+ctx | 支持（同步 per-call、callback-by-name 静态 trampoline、错误 latch 非成功哨兵、noescape 可观测） | 不承诺（typed peer 另证） | S07 宽回调 |
| i64/u64/f64/char 宽形态 | source 域具名拒（D-960-1 词表） | 同左 | 仅 TP（S07） |

- 声明侧 mode/layout/clone/deleter 的**逐轴**失效案例需新受审 provider
  manifest（机制由 provider 代际轴与 digest 键精确失效佐证；
  见 KNOWN-DEBT D-939-AX-1）。
- certify 路径的 str/record **导出函数**签名事实仍属 934 冻结词表
  （`produce_fn_sig` PortableTy={Unit,I32,Bool}）；扩展= D-960-1 关闭条件
  （KNOWN-DEBT D-939-CE-1 certify 臂）。

## 唯一资源事实与凭证计划

- core 在受检 signature/effect 上产出**唯一** per-call 凭证计划
  （`ForeignCallCredential`：view borrow 携派生 owner、exclusive Mut/Out
  车道+sig atomic 承诺、handles_in+provider tag、owned_out 凭证、
  callback 具名目标、ctx symbol）；`ForeignResourcePlan` 由声明+调用点计算，
  经 CheckedModule/CheckedFunction/SemanticView 双路携带。
- a2x lower_rt **严格消费**：资源 foreign 调用缺凭证行=响亮拒；后端不
  重推导 ownership（详见
  [auto-a2x/checked-resource-facts.md](../auto-a2x/checked-resource-facts.md)）。

## 失败原子性与生命周期

- mut/out 事务：shadow/snapshot 先行，成功与值域验证后一次 commit；失败
  保持 caller 字节与 provider 半树由原方回收（t939_txn/txn3+全分配域
  fault sweep 佐证）。外部 IO 副作用不承诺回滚。
- provider 代际：静态绑定、binding generation 单调递增；真 lease 钉住在用
  旧代，末次释放后 GC 真回收；错 provider/旧代/伪造句柄受控拒。
- **对象确定性**：C 编译旗标含 `-Brepro`（COFF TimeDateStamp 归零）——
  provider 对象 digest 是绑定身份，GC 回收 chunk 后的重编译必须复现同
  digest（否则 restore 身份断裂、会话砖化；实测无 -Brepro 时同源两编译
  12 字节漂移）。

## 认证与预算

- recipe `abi/resource-foreign-certify-v1`；deck `abi.resource-import-deck/1`
  /manifest `ace.foreign.manual/2`（resource 函数经 FnFact 可选 `resource`
  字段携带 `autoabi.contract.resource-sig/1`，None 时 wire/digest 不变）；
  report `abi.resource-import-report/1`；逐 deck 受审 reference oracle。
- 超限取消：typed `foreign.resource` 拒（reserve-before-allocation）、接受
  根保留、清理在配额耗尽下完成；预算恢复后同编辑正常发布。
- 恢复：缺 required 会话页=typed `foreign.restore.*` 拒（provider 为可复产
  工具产物、确定性重编译；**不暗冷启动**）；伪 pointer decode 拒且不覆写。
- 正式采样：资源 lane 六场景×30（fresh/warm-hit/body-edit 硬门/
  provider-transition/callback-warm/recovery-edit），五暖门 P90≤2000ms
  实测 776–1025ms；原 797 六场景继续同 crate 绿。

## 历史与证据

- 计划过程与逐 finding 证据：`docs/plans/archive/`（PLAN-939 归档件，
  §9 rounds 1-22）。
- 独立复审：r3 冻结提交 `c9f60314f` needs_fix→修复 `ea3eb327c` final
  pass（AC-01..08）。
