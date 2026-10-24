# PLAN-820 SD 草案（T-06 整理；review 按 D 实际 Spec 终确认，merge 才沉淀）

对应计划 §5 规范增量表 SD-01..07。各条目的 before/after 以已实现并验证的
行为为准；canonical 文件由 merge 阶段落（前序 Spec 未合入时不提前写）。

## SD-01 add docs/specs/auto-av/data-layout.md

- `av.data/1` profile：六标量（i32/bool/i64/u64/f32/f64）仅数据面（const/
  copy/call/return/entry，逐位保真，无算术）；名义 record（owner+name+字段
  表结构相等，同宽不同 owner 不可互换，图无环迭代检测，空 record 不支持，
  每类型字段≤1024）；FIELD_GET/FIELD_SET/AGGREGATE 指令；布局从受检类型图
  推导（checked 算术、逻辑描述符、物理存储=Value 枚举绝不 reinterpret、
  恢复重算）；槽+聚合共享 8MiB 活值预算（deep_size delta 记账、分配前拒、
  peak 全出口上 trace）；路径深≤64。验收：V-01..05/11/12。
- 证据：av_data_820 族 11/13/6/5/3、core layout/instr_spec 测试、
  verify 脚本 8 Scope。

## SD-02 add docs/specs/auto-mir/data-place-profile.md

- `auto.mir.av-data/1` MIR schema：MirType 增 I64/U64/F32/F64/Record；
  MirConst 增宽标量位型（to_bits）；新指令的 operand/read-def/trap/fuel/
  profile 合同见 core `instr_spec.rs`（12 opcode 表 + coverage_check 漏项
  即红）；类型等价=结构相等全消费点（Const/Copy/Bin/Un/Call/Return/
  Aggregate/FieldGet/FieldSet/entry）；重复 primitive 条目按形状归一；
  宽标量运算 mir.type-op 拒；entry 参数/返回限六标量（record 不出 entry）；
  unit 模式沿 798 r6 合同。验收：V-01/02/04/11。

## SD-03 modify docs/specs/auto-av/cache-and-conformance.md

- 新 producer `plan820/av-data` + A20 层（schema ac820/a20-data-abc/v1，
  键轴 producer/owner/entry/profile/types-fp/codec/三语义版本/source-
  snapshot）；恢复重走 envelope+decode+深验后执行（好 hash 坏载荷拒、
  类型表变化 clean miss、跨 profile 信封身份拒）；同 store 新进程 worker
  `data-restore-run` 恢复重验执行。验收：V-08/09（cache_compat 5/5 +
  store_restore_new_process）。

## SD-04 modify docs/specs/auto-av/project.md

- 登记 av.data/1 为 820 交付的独立 data profile（AVAD wire v1/profile 2、
  语义版本 1/1/1 独立轴；core-i32 的 1/2/1 与 AVAB 字节语义不变）；明确
  非目标（宽标量算术/str/借用/List/enum/闭包/Dyn/NaN-boxing）。

## SD-05 modify docs/specs/auto-test-case/architecture.md

- 新 recipe `ac-av/data-run-v1`：注册三处追加（REGISTERED_PROBE_RECIPES
  白名单、step 词表=run+edit-source、av binding）；worker case-serve data
  分支（av_core_version=av.data/1、parse/verify 诊断分级）；父侧判定同
  core 语义（成功=exit 值、diag=parse/verify 分级、infra=非语言判定）。
  **typed 面（F820-R6-01）**：`typed:<id>` 入口（av.data-fixture/1 冻结
  七样本：u64::MAX/i64::MIN/-NaN payload/-0.0/f32 1.0/bool true/i32 42）
  经真实准入链（data MIR→AVAD→AV），worker 报告实际产值的规范
  {kind,bits}（i64/u64/f64 十六位、f32 八位、i32/bool 不补零），父侧持
  **独立冻结期望表**逐项比对 tag 与精确 rawbits（worker 之言从非 oracle）；
  record 源成功同样附 typed 报告（bool/宽标量不再丢失于 i32-only 通道）。
  **family 隔离与形状校验（F820-R7-01）**：typed 存在性规则仅属 data
  family——core/i32 成功仍必须有旧 value 通道，非 data 响应携带 typed
  即准入门协议违规（infra 非绿）；data 的 typed 报告须为六 tag 之一且
  bits 符合 kind 规范宽度（畸形=typed-shape 红，非静默过）。冻结样本含
  合同 §3 的 f32 NaN payload 0x7F80_AD00（F820-R7-02，1.0 保留为额外
  合法对照）。typed 步骤修订哈希绑定入口身份。
  V-10 反控（wrong-exit/wrong-diag/未注册 recipe/worker 三故障=ERROR
  /wrong-bits/wrong-type-tag（父侧期望破坏缝 AV820_TYPED_EXPECT_SABOTAGE
  仅测试用）/未知 fixture id=检查错误非绿）。
  验收：corpus 绿（av-data-first/typed/bool.case.ad）+ ac_av_data_probe 7/7。

## SD-06 modify docs/specs/auto-hir/stage-contract.md

- record 域（auto.hir.record.draft）到 av.data/1 的实际消费支持面：
  自由函数+i32/bool 算术+RecordCtor（源序求值→声明序映射）/FieldGet/
  FieldPath 赋值（RHS 先物化）/record 按值参数返回（BorrowView 传递包装
  直通、BorrowMut 拒）/Method 含 static 显式拒（lower.method）；前端
  compile_record_source 前门复用（4MiB/128 深）。非扩大：core/scalar 域
  消费面不变。验收：V-02/05/06、semantics 族 13/13。

## SD-07 modify docs/specs/auto-av/resident-function-install.md

- data profile 接入 828 公共函数级增量接口：lower_function_data_unit
  （record 域单函数 AVAD unit）→ FunctionCode::admit_unit →
  ProgramSnapshot::link（fn: 导入解析+签名检查+owner 一致性）→ unit view
  执行。**全链增量（F820-R1-01 修复，取代旧"前门重编译+unit 层增量"边界）**：
  纯 body 编辑走 828 单体事务——字节级分段 diff 命中唯一被改函数（否则显式
  non-fast 拒绝）、等长注释掩码的单函数解析文档（解析面=1 函数，绝对偏移
  保持）、保留 CheckedEnv 的 recheck（resolve+typecheck 仅单 body，签名漂移
  受控拒）+ emit_function_body（BodyEmitReceipt=1 真 body/n−1 scaffold）+
  body_work 生产窗（未改函数零 body 工作）、单函数 lower/admit；未改函数
  HIR Arc 与 FunctionCode 指针/代际不变、生产阶段/IO=0，结构门内建于
  edit_body 生产路径（实现退化即使编辑本身红→正式脚本档非零）。类型/签名/
  声明集变化走依赖事实计算的精确失效：record mention 嵌套闭包+跨 bundle
  签名形状等价——签名内容变化→全部 unit 重 lower（unit import 表嵌入每个
  def 的签名，link 内 interstate 门拥有该依赖），body 未变函数保留 HIR Arc；
  helper body 变化不重 lower 调用者（快照按槽位绑定调用）；caps 变化在
  同一成功发布点同步三面——全 unit 重 lower/admit（HIR 身份不动）+ 驻留
  前端环境按新授权重建（grant 后使用新能力的热编辑可解析、被用撤销受控
  拒且旧值全保、成功撤销后无权热编辑被拒，F820-R5-01）；跨修订签名失配
  =link 拒且前快照存活；失败/陷阱/trap 编辑零安装（build-aside + 运行先
  于发布）；强制 whole 前端与等内容换对象（只保 hash 却替换对象）双退化
  反控均使同一结构门非零。验收：V-13（incremental 族 9/9 + 828 标量套件
  绿=共享基线）。

## touched_goals

- 空（有限实验扩面，见计划 §5 规范增量表注）。
