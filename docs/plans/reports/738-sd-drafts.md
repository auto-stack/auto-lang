# PLAN-738 SD-01..07 沉淀稿（merge 阶段消费）

日期：2026-10-09。绑定 R2 稳定树（worktree `b838f5f16` + 本轮 C 收口提交）。
本文是 canonical 沉淀前的正文草案；最终落稿以 merge 时 canonical Specs 现状
对齐（736 合同措辞、既有章节结构）。规则来源=PLAN-738 §5 设计 + R1/R2 实测。

## SD-01 add `docs/specs/stdlib/design/assembly-manifest.md`

**装配合同（AssemblyManifest v1，schema_version=3 载荷）**

1. 两类清单分离：**inventory**（stdlib/auto/** 全量扫描：模块/层文件/公开
   fn/method/type/field 分母、解析结果、provider 候选；失败层记诊断不入
   分母删除）与 **AssemblyManifest**（某次编译/请求的真实选择快照）。目录
   扫描不得冒充 manifest。
2. manifest 必含：schema/provider 目录版本、execution target（vm/rust/c）、
   environment（native/browser）、consumer、编译 features、按序 selected
   sources（公共在前、选定目标层在后，含层字节边界与内容指纹）、依赖闭包
   指纹、实际引用证明（reference proofs）、providers 输入身份（含本地
   producer/compiler 源与 Cargo 版本）。
3. 验证等级：declared → resolved → bound → signature_checked → executed。
   文件存在/名称登记最高到 resolved/bound；producer 签名核对后才
   signature_checked；真实执行才有 executed。未知不得升格。
4. 快照不可变：freeze 后同输入重算逐字节一致（`captured_sources_are_
   immutable_and_portable`）；便携源 ID（stdlib 相对）与本地诊断路径分离。
5. 指纹：payload 全量 JSON 序列化的 FNV-1a 64。`stdlib_assembly_fingerprint`
   另为内容级身份（inventory+目录 schema+目标+全部层内容+宿主实现 build
   输入），两者用途不同、都目标敏感；read 失败闭合拒绝（fail-closed）。
6. 核心六模块（io/net/async/http/json/sse）被引用闭包 strict：缺实现/
   冲突/未证明=拒绝；未引用的 unsupported 声明不拒绝模块导入。

## SD-02 modify `docs/specs/stdlib/design/backend-assembly.md`

- 覆盖表语义改为"实际 provider 索引"：VM=名称/ID+真实 shim 绑定双面；
  Rust=producer 文件（含 `pub use` 转发链解析到定义文件，producer 身份=
  定义处）+ 独立逻辑签名；C=c.* 声明模块 + libc（`stdlib/c/*.c.at`，
  README 记载的 `use c.stdio` 合同）。
- 签名证据显式含 mode：async producer 与调用点 `.await` 双向核对为证据，
  错配=漂移；`impl AsRef<str>` 视图适配与不透明资源类型（FileResponse/
  Upload*）按名身份，适配变体写入证明单独记录，不冒称同形。
- 旧 HTTP 矩阵按 729/730/734 增量重写：host-mapped Rust 能力不以缺
  .rs.at 文件为由拒绝；C 无 HTTP provider=Unsupported 不造实现。
- stdlib auto.* 的 ext 面（File.open 等）在 C 目标无发射：构建期
  `STDASSEMBLY.TARGET_UNSUPPORTED` 诊断指引直连 c.* provider；不产坏 C。

## SD-03 modify `docs/specs/auto-lang/frontend/design/module-resolution.md`

- 解析产物统一进装配计划：`AssemblyPlan::from_resolved`（公共 .at 在前+
  目标选定层）为 VM/持久/Rust/C 共同选源；`AssemblyTarget::context_
  extension()` 驱动层后缀，不再硬编码 .vm.at。
- 目标层声明命名空间：`c.*` 模块（`stdlib/c/<name>.c.at`）仅 C 目标会话
  可见（load_module_inner 2b 解析臂）；VM/Rust 会话维持 not-found 语义。
- 源段边界（context_byte_boundary）保留：公共段可独立解析时，合并源错
  误归因目标层真实文件行。
- use 语义不变：545 bare/named/wildcard、635 门控、循环、同名异 root 按
  resolver 身份区分；用户同名模块干净遮蔽 stdlib。
- 缓存边界：段级指纹（公共+选定层各自 FNV+absent 台账+provider schema+
  依赖闭包）；跨 target 条目共存不串；root/目标热换=SessionTargetMismatch
  须重建会话。

## SD-04 modify `docs/specs/auto-lang/trans/overview.md`

- Rust/C 发射先声明装配目标再装载/发射（trans_rust/trans_c 入口）；
  VM 专用层（.vm.at）不进入 Rust/C 类型上下文，反之亦然。
- 项目 Rust 模块发射消费实际选定 `.rs.at` 层；C 发射消费完整 AST（声明
  锚+有 body 定义）；stdlib auto.* 模块不转译为 C 产物（provider=c.*+libc）。
- 六核心引用闭包经 `verify_rust_reference`：provider claim 无真实 callee
  （含转发链解析失败）=PROVIDER_CLAIM_NO_CALLEE；签名漂移/arity/模式错配
  =SIGNATURE_DRIFT；未证明=SIGNATURE_UNVERIFIED。裸名映射（diff/frame 等
  非六核心）不在 strict 面，不收集证明（记档边界）。

## SD-05 modify `docs/specs/auto-man/design/api-generation-integrity.md`

- generation.json 收据 assembly 块（schema 3）：业务 source_hashes 与
  assembly 指纹各自记录（读写形状对齐——数组/对象失配曾致源内容漏检）。
- 新鲜度门：`backend_generation_is_fresh` 比对当前 manifest 指纹 vs 收据
  指纹；仅双已知相等判新鲜；漂移或单侧缺失=陈旧走再生臂。runtime
  serviceconfig/config_hash 不是 assembly 指纹，不混入。
- 生成开始撤销旧收据；结束重核冻结输入，中途变化/失败不写成功收据。
- 生成服务二进制内嵌 `__assembly_fingerprint()`；`/__auto/health/ready`
  报告并与收据一致；改 stdlib 不改 api.at 必须触发再生（服务验收冻结）。

## SD-06 modify `docs/specs/auto-cli/project.md`

- `auto stdlib inspect [--module auto.http] --target vm|rust|c --environment
  native|browser --format json [--check] [--actual <file.at>]`：
  inventory/actual 模式区分；--actual 真实 dry 装配（resolve_uses，不执行
  main/网络/文件业务）。
- 退出码：0=pass；1=check 违规（被引用闭包 missing/conflict/unverified/
  unsupported）；2=inventory partial（解析失败非零不吞）；3=错误。
- JSON 稳定排序、便携源 ID、无绝对路径泄漏；provider_claims 仅列当前
  target 且实际装载的模块。

## SD-07 modify `docs/specs/stdlib/project.md`、`networking-stdlib.md`

- stdlib 能力矩阵以 manifest 实测为准：支持集须 signature_checked+必要
  exec 证据；stub（sse parse_sse）/第四绑定面（io 方法 VmModule 表）/
  扫描名 id 相撞 13 处=诚实 Unverified/冲突上报，不冒称 Supported。
- TCP read 公开缓冲参数/单返回与实际 buf_size/双栈输出差异保持 Unverified
  拒绝，不擅改公开 ABI。
- D3a/D3b 边界：本期交付来源真实性与错误可见性；跨目标语义/取消/错误
  ABI/资源 parity 属 D3b，不得由 signature_checked 推定。
