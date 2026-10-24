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
  core 语义（成功=exit 值、diag=parse/verify 分级、infra=非语言判定）；
  V-10 反控（wrong-exit/wrong-diag/未注册 recipe/worker 三故障=ERROR）。
  验收：corpus 绿（av-data-first.case.ad）+ ac_av_data_probe 4/4。

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
  执行；body 编辑仅重建被改函数（未改函数 Arc 指针/代际不变=结构轴）、
  类型变化重建受影响函数、跨修订签名失配=link 拒且前快照存活、强制全量
  重建移动全部身份轴（退化反控）。边界：ac-syntax 驻留层（auto-scalar
  语法）为 scalar-only，record 语法属 ac-syntax owner 面；数据增量以
  前门重编译+unit/link 层增量交付。验收：V-13（incremental 族 3/3 +
  828 标量套件绿=共享基线）。

## touched_goals

- 空（有限实验扩面，见计划 §5 规范增量表注）。
