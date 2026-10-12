# PLAN-738 SD-01..07 沉淀稿（merge 阶段消费）

日期：2026-10-09（Phase 3 / T-13 终稿）。绑定 Phase 3 提交链
`2c1b4a763`（R2 基线）→ `c1579ed71`（T-10）→ `84bec29ef`（T-11）→
`b6cfc6df1`（T-12）；差异范围机械分类见 [738-phase3-scope.md](738-phase3-scope.md)。
本文是 canonical 沉淀前的正文草案；最终落稿以 merge 时 canonical Specs 现状
对齐（736 合同措辞、既有章节结构）。规则来源=PLAN-738 §5 设计 + R1/R2/Phase 3 实测。
merge 前若 T-14 后续提交改变行为，以最终提交复锚。

## SD-01 add `docs/specs/stdlib/design/assembly-manifest.md`

**装配合同（AssemblyManifest v1，schema_version=4 载荷，双重身份）**

1. 两类清单分离：**inventory**（stdlib/auto/** 全量扫描：模块/层文件/公开
   fn/method/type/field 分母、解析结果、provider 候选；失败层记诊断不入
   分母删除）与 **AssemblyManifest**（某次编译/请求的真实选择快照）。目录
   扫描不得冒充 manifest。
2. manifest 必含：schema/provider 目录版本、execution target（vm/rust/c）、
   environment（native/browser）、consumer、编译 features、按序 selected
   sources（公共在前、选定目标层在后，含层字节边界与内容指纹）、依赖闭包
   指纹、实际引用证明（reference proofs）、providers 输入身份（含本地
   producer/compiler 源与 Cargo 版本）。
3. **双重身份（schema 4）**：`fingerprint`=共同装配身份（consumer 名置空、
   `consumer_input` 角色源剔除的中立投影哈希）——同 fixture/同 target/同
   features/同来源闭包下 CLI actual、编译会话、生成收据三角相等；
   `consumer_fingerprint`=消费者收据身份（全量 payload，旧 schema 3 单指纹
   语义）。业务输入只改收据身份不改装配身份；target/provider/来源闭包变化
   改变共同身份。未知/旧不完整快照=陈旧或拒绝（保守再生），null 指纹不能
   相互证明新鲜。
4. 验证等级：declared → resolved → bound → signature_checked → executed。
   文件存在/名称登记最高到 resolved/bound；producer 签名核对后才
   signature_checked；真实执行才有 executed。未知不得升格；**Resolved-only
   不是引用核心调用成功的依据**（strict 发射/生成入口只接受充分证明）。
5. 快照不可变：freeze 后同输入重算逐字节一致；便携源 ID（stdlib 相对）与
   本地诊断路径分离；同内容异绝对根共同身份稳定。
6. 指纹族：manifest 双指纹（上文）；`stdlib_assembly_fingerprint` 另为内容
   级身份（inventory+目录 schema+目标+全部层内容+宿主实现 build 输入），
   用途不同、都目标敏感；read 失败闭合拒绝（fail-closed）。
7. 核心六模块（io/net/async/http/json/sse）被引用闭包 strict：缺实现/
   冲突/未证明=拒绝；未引用的 unsupported 声明不拒绝模块导入。

## SD-02 modify `docs/specs/stdlib/design/backend-assembly.md`

- 覆盖表语义改为"实际 provider 索引"：VM=名称/ID+真实 shim 绑定双面；
  Rust=producer 文件（含 `pub use` 转发链解析到定义文件，producer 身份=
  定义处）+ 独立逻辑签名；C=c.* 声明模块 + libc（`stdlib/c/*.c.at`，
  README 记载的 `use c.stdio` 合同）。
- Rust producer 面按运行形态分家：Standalone=crates/a2r-std crate、
  Embedded=crates/auto-lang/src/a2r_std.rs 内嵌镜像；3 参 post 认证面等
  producer 只在内嵌镜像存在——Standalone 下诚实拒绝，不冒称 Both。
- **适配契约注册表**：发射侧适配（状态侧信道包装/数值 cast/三参元数分派/
  async 流 facade 含参数 facade 视图）必须命中独立声明的契约表
  （ADAPTER_RULES），发射结构/producer 实形/公共投影四方一致才
  signature_checked；类型名 pattern 与公共 AST 复制不构成证明。纯适配面
  符号无裸名拼写（具名导入拒绝并指明限定拼写）。
- 签名证据显式含 mode：async producer 与调用点 `.await` 双向核对为证据，
  错配=漂移；`impl AsRef<str>` 视图适配与不透明资源类型（FileResponse/
  Upload*）按名身份，适配变体写入证明单独记录，不冒称同形。
- 旧 HTTP 矩阵按 729/730/734 增量重写：host-mapped Rust 能力不以缺
  .rs.at 文件为由拒绝；C 无 HTTP provider=Unsupported 不造实现。
- stdlib auto.* 的 ext 面（File.open 等）在 C 目标无发射：构建期
  `STDASSEMBLY.TARGET_UNSUPPORTED` 诊断指引直连 c.* provider；不产坏 C。
- 已证漂移面如实上报：如 json.is_valid 公共 int vs Rust producer bool
  （VM shim=int 在案）——跨拼写一致拒绝，不以拼写切换放行。

## SD-03 modify `docs/specs/auto-lang/frontend/design/module-resolution.md`

- 解析产物统一进装配计划：`AssemblyPlan::from_resolved`（公共 .at 在前+
  目标选定层）为 VM/持久/Rust/C 共同选源；`AssemblyTarget::context_
  extension()` 驱动层后缀，不再硬编码 .vm.at。
- 目标层声明命名空间：`c.*` 模块（`stdlib/c/<name>.c.at`）仅 C 目标会话
  可见（load_module_inner 2b 解析臂）；VM/Rust 会话维持 not-found 语义。
- 源段边界（context_byte_boundary）保留：公共段可独立解析时，合并源错
  误归因目标层真实文件行。
- use 语义不变：545 bare/named/wildcard、635 门控、循环、同名异 root 按
  resolver 身份区分；用户同名模块/同名函数干净遮蔽 stdlib（fn 遮蔽优先）。
- **核心导入闭包**：具名 `use auto.<core>: f` 逐项验证绑定+公共签名；
  wildcard 只记台账不因未引用 unsupported 拒绝，其裸名调用按公共面唯一
  归属验证（多模块同名=歧义拒绝）；裸名调用重建限定文本携带真实实参面。
- 缓存边界：段级指纹（公共+选定层各自 FNV+absent 台账+provider schema+
  依赖闭包）；跨 target 条目共存不串；root/目标热换=SessionTargetMismatch
  须重建会话。

## SD-04 modify `docs/specs/auto-lang/trans/overview.md`

- Rust/C 发射先声明装配目标再装载/发射（trans_rust/trans_c 入口）；
  VM 专用层（.vm.at）不进入 Rust/C 类型上下文，反之亦然。
- 项目 Rust 模块发射消费实际选定 `.rs.at` 层；C 发射消费完整 AST（声明
  锚+有 body 定义）；stdlib auto.* 模块不转译为 C 产物（provider=c.*+libc）。
- **六核心引用闭包全路径**：限定 Dot（含 Json 别名）、具名/裸名/通配导入、
  `auto.<core>.<method>` 三段形状、for-in 流反糖、`fn expr` Await 臂、
  **公共方法 receiver 调用**（`Owner.method` 分母归属：模块限定/扁平
  value_* helper/json 值绑定直发/未接管发射臂四形态——前三验证，
  第四（发射原样保留、实参丢失）诚实拒绝）——全部经
  `verify_rust_reference`（契约表驱动）；provider claim 无真实 callee
  =PROVIDER_CLAIM_NO_CALLEE；漂移=SIGNATURE_DRIFT；未证明
  =SIGNATURE_UNVERIFIED，不产出成功产物。方法符号的 receiver/is_static
  证据来自公共声明事实；序列面 Vec<T>→[]T 按名身份；适配契约族含 bool→int if 壳
  （has_key）、宽度归一 cast 壳（len/last_status）、Null 哨兵返回族
  （parse/get/get_at 的 JsonValue? 面）；receiver 带参方法（get/get_at/
  has_key）为 parser 不支持拼写（语言层拒绝）；模块拼写的无臂方法
  （type/as_number/as_array 等）经未接管发射臂诚实拒绝。既有真漂移面
  （JsonValue.is_null/is_valid/as_bool 公共 int vs Rust producer bool）
  双拼写一致拒绝，不擅改公开 ABI。
- 面外边界（记档不删）：类型名渲染、`json!` 宏复合、legacy 平面符号
  （http_post）、无公共声明的 legacy 别名（json.get/get_u64 等，P738-D2
  在案）。
- 非六核心裸名映射（diff/frame/fs/env 等）不在 strict 面，不收集证明。

## SD-05 modify `docs/specs/auto-man/design/api-generation-integrity.md`

- generation.json 收据 assembly 块（schema 4 双指纹）：业务 source_hashes
  与 assembly 指纹各自记录；收据/烘焙常量/在途核对/新鲜度门统一使用
  **consumer_fingerprint**（同消费者跨时语义与 schema 3 兼容）。
- **生成装配引用闭包**：endpoint 内联体 + db.at + 伴生 `src/back/*.at`
  back 模块全部按 Embedded 运行形态转译收集证明并入 manifest——back 产物
  经 qualify_a2r_std 链接内嵌镜像，不把 endpoint 面冒称全部装配。
- **workspace lock 收据身份（一次绑定）**：ready 记录 `workspace_lock`
  （生成产物运行时依赖输入的 FNV 身份；未建 lock 记 absent）——与生成器
  自身 Cargo 输入（manifest provider 面）分开记录。复用门为**一次绑定
  状态机**：absent→首次出现实际 lock 时判新鲜并把实际身份绑定写回收据
  （一次性收敛）；此后严格比较——依赖版本漂移/lock 删除/读取失败/旧
  收据缺字段均陈旧走再生臂（R5-01 反例冻结：纯 absent 豁免会永久放行
  后续漂移）。
- 新鲜度门：`backend_generation_is_fresh` 比对当前 consumer_fingerprint vs
  收据指纹（仅双已知相等判新鲜）+ workspace lock 对拍；漂移或单侧缺失=陈
  旧。runtime serviceconfig/config_hash 不是 assembly 指纹，不混入。
- 生成开始撤销旧收据；结束重核冻结输入，中途变化/失败不写成功收据。
- 生成服务二进制内嵌 `__assembly_fingerprint()`（收据身份）；`/__auto/
  health/ready` 报告并与收据一致；改 stdlib 不改 api.at 必须触发再生。

## SD-06 modify `docs/specs/auto-cli/project.md`

- `auto stdlib inspect [--module auto.http] --target vm|rust|c --environment
  native|browser --format json [--check] [--actual <file.at>]`：
  inventory/actual 模式区分；--actual 真实 dry 装配（resolve_uses，不执行
  main/网络/文件业务）。
- **退出码（以代码为准，纠正旧稿 2/3 写反）：0=pass；1=check 违规（被引用
  闭包 missing/conflict/unverified/unsupported）；2=错误（EXIT_ERROR）；
  3=inventory partial（解析失败非零不吞）。**
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
