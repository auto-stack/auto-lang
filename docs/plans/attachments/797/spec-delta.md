# PLAN-797 规范增量提案（spec-delta）

> 状态：**R4 修复后重置稿**。绑定 R4 修复提交 `6294d4bea`（真实 semantic
> contract 绑定 / 正式 AutoDown 执行 / 性能复测已按实际通路重写）与
> T-07 材料提交；canonical Spec 输入 hash 为下方当前实测值（R2 曾警示
> goals.md 漂移，已按当前树重核）。§5 五行意向表是计划契约，本文件是
> 可审阅的规范正文；**不得**在 review/merge 前把本文发布为 canonical
> Spec 或改 ledger；本文自身 hash 由 freeze-proposal 与复审记录外置绑定。

| 项 | 值 |
|---|---|
| plan_id | PLAN-797 |
| plan_revision | 1 |
| draft_for | T-07 / F-797-R1-02 |
| candidate_profile | `autoabi.win-x64.v0.1-candidate` |
| implementation_tree | `D:/autostack/.wt/lang-797/auto-lang` @ `plan-797-dev` |
| freeze_state | **candidate**，不是 “ABI v0.1 已发布” |

## Spec inputs（canonical，worktree 终态实测 SHA256，2026-10-19）

| path | SHA256 | 角色 |
|---|---|---|
| docs/specs/auto-ac/project.md | 008D039E7C2897619407C2E3281907847542921750EC40D53397934A72240A9F | SD-04 导航补丁的目标 |
| docs/specs/auto-ac/data-resource-contract.md | 85B8074B6045F8F1A7DB391D50347182E05E601EC8BEB7D2EB88DAD48161B1C1 | SD-05 分域声明的目标 |
| docs/specs/auto-ac/artifact-contract.md | 2C5305E7FEE33595CEBFE1CB8399F4318984D38D45A8278A0513C841C2049DFE | 产物/链接语境输入 |
| docs/specs/auto-cache/project.md | 7A48BA94FDAE6B2A80CF51CBFF9E49A101942C28217D782FAFBB9A81AA4FB30C | cache key/失效语境输入 |
| docs/specs/auto-test-case/project.md | F2993EC85AA9DF0FD7FFA41D7E159FEA053E592C18D940565A4CD616A8B9AD82 | .case.ad / AutoDown / probe seam 语境输入 |
| docs/specs/auto-semantics/semantic-contract.md | 8E27FCDB6592B7F44A86A85F6BE1619022C72A88A0EA67B1CBD10AF0B897AC2D | 真实语义绑定依据（T-06 消费面） |
| docs/specs/goals.md | EC5170B6B769E33F24735E35541A071EE6E499F09D5BD328F47BF64B6F1E2D7D | touched_goals 依据（当前 `[]`） |

---

## SD-01 | add | docs/specs/auto-abi/project.md

### before

无 `docs/specs/auto-abi/` 公共有限 profile 规范。Design41/42 与
ABI RFC 是目标文档，不是已发布 ABI；774 `probes/data-resources` 的
ABI-A–E 是内部冻结面，不能被读成公共 AutoABI。

### after（规则）

1. **域**：公共边界属于 ABI 域 `autoabi`，与 774 内部 ABI-A–E
   **无兼容关系**；禁止任何“两者可互换”表述。
2. **候选/冻结状态机**：`v0.1-candidate` →（独立 review + merge 门禁）
   → 本 target 有限 profile 冻结。未过门禁不得登记“已冻结”。
3. **支持边界**：仅 Windows x86_64-pc-windows-msvc / Windows x64 调用
   约定；声明源为受限 declaration IR，**不是** Auto 源码 parser 支持面。
4. **工具**：真实 C 侧用可定位 MSVC cl.exe（或 clang-cl）独立编译；
   缺工具 = infra 阻断，禁止 Rust repr(C) 伪通过。
5. **拒绝面**（有限 profile 支持表，不是永久删去战略）：按值聚合、
   variadic、bitfield、任意 union、C++ ABI、异常/unwind 穿界、
   跨线程持久 callback、热重载/DLL 事务、跨域自动转发。
6. **版本五元组**：ABI 域、target/profile、接口 revision（descriptor
   指纹）、runtime 服务表版本、语义 schema 版本分开存在；未知/错配在
   **调用前**拒绝，native 被调次数 0。

### rationale

防止把候选探针冒称为完整 ABI / 产品源码支持面。

### acceptance IDs

AC-01–06

---

## SD-02 | add | docs/specs/auto-abi/windows-x64-v01.md

### before

无首版 Windows x64 合同正文。

### after（规则，经实证）

1. **调用约定**：Windows x64；公开 AutoABI 函数 = 规范 C 标识符符号 +
   显式 context（fallible）+ `i32 status` + 显式 out 指针。
2. **纯形态**：`ret(params...)`，标量直传直返；参数/返回 ∈
   `{i32,u32,i64,u64,f64,bool(u8 仅 0/1),char(u32 合法标量)}`。
3. **可失败形态**：`i32 status f(ctx*, params..., out*)`；status 0=成功；
   **成功才提交 out**；失败清理本次部分分配，caller 旧输出不变。
4. **布局**：声明序 + 自然对齐；checked 算术；nominal 身份（同布局不
   自动互换）；与独立 C `sizeof/_Alignof/offsetof` 对照为 oracle。
5. **buffer**：`{i64 ptr, u64 len}`；`len=0` 允许 null；文本另标 UTF-8；
   越界/溢出/非法 bool/char 受控拒绝。
6. **handle**：opaque owned handle + 原方 destroy + 可校验身份/代际；
   double destroy / 错误 provider / 借用逃逸受控拒绝。
7. **callback**：仅同步 per-call + 显式 ctx；不保存、不跨线程/代际；
   保存/异步能力 = 明确不支持并拒绝（不是 loader 承诺）。
8. **descriptor**：`autoabi.descriptor/1`，canonical JSON + blake3
   接口指纹；含 domain/target/profile/packing/semref/records/symbols/
   runtime services；未知 format 拒绝。
9. **资源上限**（probe 运行策略）：build≤60s、run≤5s、total≤70s；
   stdout/stderr≤1MiB；IPC≤4MiB；单产物≤64MiB/请求合计≤128MiB；
   受管 cache 热点≤8MiB + 暂存/索引另≤8MiB。超限 = resource 状态，
   与 ABI 不兼容拒绝分开。

### rationale

可被独立 consumer 按合同编写 callee/caller 并对拍。

### acceptance IDs

AC-01–03

---

## SD-03 | add | docs/specs/auto-abi/conformance-cache.md

### before

无 AutoABI 互调基准 / 缓存规范。

### after（规则）

1. **三腿**（每个 producer 独立命令/obj/lib + descriptor；consumer 独立
   构建并先 admission）：
   - A AutoABI caller → AutoABI callee
   - B AutoABI caller → 独立 C callee（cl.exe）
   - C 独立 C caller → AutoABI callee（可嵌套 C→AutoABI→C）
2. **独立 oracle**：C 侧 sizeof/offsetof + 运行结果；禁止双方同源错误
   layout 生成器互证；禁止以 “call ok” 日志当成功。
3. **负控**：swap 参数 / 错 offset / 漏调用 / tampered descriptor 必须
   可区分失败（exit 恰 1 或调用前拒绝），且已知不兼容签名不制造 UB。
4. **cache**：复用 783 ManagedStore，probe 自有层（ABI-L0/ABI-O1…），
   不改 783/787 注册/GC。key 含语义 ref、canonical decl-set（含 body
   identity）、target/profile、emitter id、C 对象 content hash + cl
   指纹 + flags。命中**从不**调 producer，**从不**缓存 PASS；每次仍
   admission + link + run + 重比较 oracle。
5. **失效**：body/符号集/链接模式/C 内容/profile/toolchain 任一变化按层
   失效；tampered block = 受控 CAS 拒绝；缺块 = miss 重算。
6. **生命周期**：512 次接口/实现修订后 live 集收敛；rooted+leased 存活；
   用户 source/exe 哨兵不变；失败/取消释放 lease 且不删交付物。
7. **性能**：六场景×30；硬门 P95 body-edit <1000ms、descriptor-reject
   <1000ms；样本 JSONL 全保留（含失败）；工具准备单列。
8. **正式通路**：D-796/D-788 后经正式 AutoDown/probe adapter 复测；
   当前 Rust 原生测试是临时等价入口，**不能**充作 AC-05 正式复测。

### rationale

可重验的互调与增量证据，防止缓存假绿。

### acceptance IDs

AC-02–05

---

## SD-04 | modify | docs/specs/auto-ac/project.md

### before

AC project 未导航到公共 AutoABI probe。

### after

补一节 **ABI probe 导航**：指向 `docs/specs/auto-abi/*`（候选/冻结
状态、支持边界）；明确生产 Auto 前端 / 旧内部 profile **仍原状**，本
导航不扩大源码支持面。

### rationale

不扩大源码支持声明。

### acceptance IDs

AC-06

---

## SD-05 | modify | docs/specs/auto-ac/data-resource-contract.md

### before

774 数据/资源合同与未来公共 profile 边界未分域。

### after

补 **分域声明**：774 内部 ABI-A–E 与 `autoabi` 公共候选 profile 分属
不同兼容域；禁止用内部 ABI 的兼容承诺去描述公共 profile（或反向）。
资源 representation（buffer/handle/callback）在公共 profile 中按 SD-02
执行，不复用旧 text 内部 12/8 布局。

### rationale

旧承诺保持，防止误用兼容声明。

### acceptance IDs

AC-01、06

---

## touched_goals

当前 `touched_goals: []` 保持：goals 账本未列独立公共 AutoABI probe 目标；
探针尚未接入产品消费面，不能提前写成 GOAL 产品能力完成。T-07 终稿若
796/产品接线改变目标归属，再提案修订元数据。

## 未决（T-06 后必须闭合）

- 临时 `autoabi-semref-snapshot/0` 退出收据 → 切换 796 真实 semantic
  contract 后写入本文件与 freeze proposal。
- 正式 AutoDown 执行与性能复测数据绑定。
- 本草稿与最终实现 diff 再对账后才能冻结 hash。
