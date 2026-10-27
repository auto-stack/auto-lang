# PLAN-828 规范增量具体文本（SD-01..06，T-09 落文；proposed，merge 沉淀）

日期：2026-10-22 ｜ 状态：proposed（canonical/ledger 由 merge 按本文本沉淀；当前不写
已实现状态到 docs/specs）｜ 依据：r3 合同 + 真实代码（本批冻结提交）。

本文件给出每个 SD 的**具体落文文本**：add=新 spec 文件全文；modify=对现行文件的
精确增改段。语义与 `successor-interface-contract.md`（r3 回写）一致。

---

## SD-01（add）`docs/specs/auto-ac/function-incremental-source.md`

```markdown
# auto-ac 函数级增量源前端（PLAN-828）

> Status: active（scalar-i32/bool profile；实现=`experimental/ac-core/src/source/*`
> + `experimental/ac-av-probe/probe/src/session.rs` 热路径）。

## 受检接口环境与单 body 检查

- 冷初始化走完整前端（`compile_scalar_source`：parse→resolve→typecheck→Atom
  render/bind→verify），产 `CheckedEnv`（驻留受检接口环境：每函数签名/owner 域/
  generation）——整模块入口原样保留，作为 truth 源与冷启动路径。
- 编辑热路径**不再构造全部 stub Program**：ac-syntax 增量重扫（`incremental_reparse`）
  →单声明严格投影（`project_decl_strict`）→`prepare_candidate_document`（真单 body
  resolve/typecheck/emit 事务：接口目录脚手架、恰一 body 发射、返回前恢复 current）
  →FnEnv 接口门→**内容版本判 hit/miss**——hit 复用已验对象（零重验证）；miss 才
  `verify_function` 凭证门→全链（lower/mir_verify/abc/decode/admit）。会话生产
  consumer 不经 `prepare_function_candidate`（那是 ac-core 的组合公开接缝=document
  +verify；B3-C05 更正早期把两者顺序写反的表述）。
- 结构门：生产者窗口内改函恰 +1×4 阶段（project/resolve/typecheck/hir_build）、
  其余函数全零、整模块轴零；强制走旧整 Program 管线时同门必红（永久负控）。

## 函数凭证与上下文身份

- `verify_function(bundle, def_index) -> CheckedFunction`：仅验证器构造；绑定 body
  内容、owner/域、实际读取事实与 `context_revision`（内容规范化指纹，类型按形状
  非表下标）。
- 候选事务（`FnCandidate`）：候选自持新条目（move）+env_id（进程唯一）+
  base_generation；commit 单次安装（与 root/source/revision 同发布点）；
  cancel/drop 零 env 引用；公开 recheck 成功同属发布（generation+1，旧候选经
  base_generation 失效）。
- canonical 内容身份（can4）覆盖 hir PartialEq 全部语义字段（绑定含 moved、
  Binary overflow；cosmetic 仅 span/mutable_explicit 不进键）；格式标记换键即
  版本隔离旧行。

## 来源与失效轴

- 跨修订函数身份=def id 文本（`d_<name>`）；body 长度/行号变化不使未改函数凭证
  按全局 source revision 失效（目录/词法边界/来源映射单列）。
- 签名/声明集/配置/授权变化、跨函数编辑、词法不收敛=显式非快路径（记录
  `non_fast_path_reason`），不静默降级。
- body 事实查询核心（`FnEnv::{set_body_fact,body_fact_of,body_fact_current}`）：
  凭据属具体受检产物（history 条目自带冻结 deps）；已读事实变化→消费者失效；
  重读返回新值不刷新旧凭据；未读不失效；只读接口 caller 零重编。
```

## SD-02（modify）`docs/specs/auto-hir/stage-contract.md`、`docs/specs/auto-syntax/snapshot-cst.md`

stage-contract.md 在"阶段表"节后追加：

```markdown
## 函数级私有凭证（PLAN-828）

S1–S4 完整模块入口保持；828 增设函数级私有通道：`verify_function` 对单个 def 产出
`CheckedFunction`（body 检查+全 def 签名上下文，缺 body 的混合/未调用 def 在整模块
`check_module` 走查被拒——`verify.missing-body`；函数级脚手架成对合法）。凭证绑定
`context_revision`（内容规范化）与实际依赖，不从旧 CheckedModule 切片盖章；恢复走
A07 五门重验（见 [../auto-ac/function-incremental-source.md]），凭证不落盘。
```

snapshot-cst.md 在"增量"相关节后追加：

```markdown
## 会话驻留消费（PLAN-828）

增量编辑/重扫（`incremental_reparse`+`project_decl_strict` 单声明严格投影）被 828
会话作为唯一编辑热路径消费：未改声明不重新投影（计数 `syntax_reparsed_decls` 恰一
为证据轴）；树/快照/页合同不变。scalar 表面由 `grammars/auto-scalar.at` 版本化承载，
与 ac-core scalar parser 的 AST+span 全等由 `scalar_projection_` 差分钉死——投影差分
是准入面，不支持面不减。
```

## SD-03（add）`docs/specs/auto-mir/function-incremental.md`

```markdown
# auto-mir 函数级增量（PLAN-828）

> Status: active（实现=`experimental/ac-av-probe/probe/src/hir_adapter.rs` +
> `core/src/{mir,mir_verify}.rs`）。

- 单函数 lower：`lower_function_unit` 从 CheckedFunction（经单函数 bundle）直接
  lower/verify 该函数；**不调用 whole lower 再 split**；未改函数零 lower/emit/
  decode/verify/load（结构门 0 断言）。
- 单元局部性：函数局部常量/类型/槽/PC 独立于其他函数长度/顺序；跨函数引用=稳定
  def id 文本（`fn:<name>` import + 链接期绑定表），不是模块数组序号/物理地址 ABI。
- 首版函数为唯一根区域（Design43 §8.1）：身份/依赖/代码引用保持可分解接缝；
  block 级增量/活帧迁移/无语义占位指令不在本合同。
- 内容命中：同内容版本复用现存不可变 MIR/ABC/代码对象与各自 mint 来源/代际；
  仅变更单元产生新产物（计数只计新产物）。
```

## SD-04（add）`docs/specs/auto-av/resident-function-install.md`

```markdown
# auto-av 常驻函数安装与事务快照（PLAN-828）

> Status: active（实现=`core/src/{function_code,program_snapshot,av}.rs` +
> `probe/src/session.rs`）。

## 单元派发（无 merged 载荷）

- `ProgramSnapshot::link_with_maps`：单一 owner 上下文门（schema+module 全一致）+
  实际调用 import 闭包门（每个被执行的 Call 必须有链接期绑定）+ `fn:<name>` 签名
  等值绑定表（scratch interned 类型表仅作内容等值校验）。**不复制未改 blocks/
  instrs/slots/spans**——执行视图是单元本身（`UnitView`/`ModuleView`）。
- 三公开入口（`execute_program`/`execute_program_steps`/`resume_program`）共用同一
  单元派发；嵌套/递归沿捕获 snapshot 选 callee；分段 resume 预算（段内增量比较，
  累计硬限额跨段）。
- 来源投影：payload span 经帧执行槽投影映射（payload func 号不参与键控）；强制
  复制未改执行 payload 且实际使用时结构门必红（副本负控）。

## 安装事务与寿命

- prepare 仅构造候选（PreparedEdit 自持候选 HIR/code/来源/绑定）；成功 commit 在
  同一发布点原子更新 root/source/revision/env 槽位；失败/取消/迟到/foreign 不覆盖
  当前可用版（base_generation/env_id 守卫）。
- 新顶层调用取新 snapshot；在途调用链（PausedRun/pin）固定旧 snapshot 至退出；
  旧代码在 snapshot/frame/lease 引用期间保活，最后 owner 释放即回收（Weak oracle）；
  close 幂等、释放 env/store 拥有引用，外 pin 存活、vacant 执行受控 Infra。
- 每函数内容历史有界（HISTORY_CAP=32，真实淘汰；驻留当前条目永不逐出）。
```

## SD-05（modify）`docs/specs/auto-av/cache-and-conformance.md`

"真实粒度与后继"节的 828 段替换为：

```markdown
828 已交付 source→函数代码安装完整切片：常驻会话内未改函数 HIR/MIR/代码 identity/
代际不变、未改生产阶段/产物 IO 为 0；结构与时间双门（正式战役=常驻 worker+IPC+父侧
judge，计时窗=编辑入队→父侧全部校验完成为终点，n≥30，P95<1000ms 且全对且结构绿——
结构含修订进度/确切安装目标/目录完整（functions 交叉+entry 锚）/未改函数派发载荷身份稳定（地址+代际+内容，观测由执行器采进 trace）/驻留对象绑定（resident 活地址对比，run 内真实副本执行必红）/执行序配对（record seq==current seq，冗余副本执行必红）；逐样本阶段 wall（真实生产接缝 µs）与真实 IO 字节（无 store=结构零盘 IO）随观测上报；完整观测与逐样本 raw 经 write_evidence **先落盘后断言**留存，成功失败同留）。
资源按 798 §5.9 双线预算硬门：A 线前端编译瞬态隔离测量 ≤224MiB；B 线探针残差
（总峰值−A 归因）≤128MiB，缺归因整峰计 B。存储轴：manifest format=2 双指纹键
（body+规范化 required 闭包）；单元 ABC/sidecar store-internal 挂 required 边，
superseded 根发布点退役；A07 HIR soft root 有界（每函数至多一个当前 cv root，
supersession 退役/回访重挂/restore 按行存在置归属追踪器——与 untrusted HIR 可采纳性
分轴）；512 真实修订+周期 GC 存储水位有界（不随编辑数增长），恢复重验、旧 pin
保活/释放回收、用户哨兵不动。
```

## SD-06（modify）`docs/specs/auto-test-case/architecture.md`

"probe 通道"节 ac-av/core-run-v1 段后追加：

```markdown
PLAN-828 正式持续会话入口：`av_probe_worker session-serve`（一进程一会话：init/ready
恒定身份+初始派发身份基线、编辑流发布已验证当前值+分相计时+计数快照+派发载荷身份
（对象地址+代际）+worker 线内存峰值、close），父侧 judge 拥有测量窗（编辑入队→
父侧**全部校验完成后**为终点，IPC 与逐请求校验在窗内）与独立期望；每发布绑定期望
修订（revision==req_id）与确切安装目标（`expected_install`），未改函数派发身份跨修订
稳定；双门=P95<gate ∧ 全对 ∧ 结构（未改零阶段/零整模块/恰一确切安装 ∧ 修订/目标/身份
门）∧ 完整性，gate=0 必红；worker 中途死→缺样本红完整性，不静默缩战役；初始化排除
单列（init_ms）、ready 先判；完整观测（逐样本计数差分/诊断/身份/worker 相位与
输入/二进制身份）经 `JudgeReport::write_evidence` 落盘留存（成功与失败同留）；
`frontend-attribution` 子命令为双线预算 A 线隔离测量口。one-shot run/recipe 保留不变。
```
