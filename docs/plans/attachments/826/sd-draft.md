# PLAN-826 SD 草案（T-06 冻结，送独立 review；canonical 发布归 merge）

状态：草案 @01b3eb8cc+T-06 提交。全部为 proposed，不在本轮发布 canonical/ledger。
来源：T-01 冻结合同（contract-decision §1–10）+ T-02..T-11 实测证据。

## SD-01（add）docs/specs/auto-av/compact-abc-format.md

完整英文 spec 草案见同目录 [compact-abc-format.md](compact-abc-format.md)
（merge 可直接以其为 canonical 文件底稿）；本节为浓缩要点，两者同源
（contract-decision §1–10 + T-02..T-06 实测）。

- 新 wire 世代：magic `AVAC`、version 1、profile id 1 = `av.core-i32/1`（同一标量语义
  profile 的 wire 世代，非新语义）；逻辑 schema `auto.abc.av-compact-core-i32/1`。
- canonical ULEB128/u32 操作数（≤5B、非最短/第5字节高位/截断拒绝）；section
  offset/length/body 长度为显式 LE u32；i32 常量 4B 位型（无 zigzag）。
- 固定头 12B + 9B 目录项；5 个必需 section（MODULE/TYPES/CONSTS/IMPORTS/FUNCS）
  恰一次；可选 `0x7f SOURCE` 与未知 `0x80..0xff` 按 length 跳过；目录须连续覆盖到
  EOF，缺口/重叠/越界/尾随拒绝。
- 操作表（CONST/COPY/BIN/UN/CALL/TRACE/JUMP/BRANCH/RETURN）与类型/常量/slot/import
  标签如合同 §4；未知 tag/子操作/kind/profile 为 format 拒绝；data/owned/
  collections/closures/dynamic 不按 scalar 解码。
- 函数名/slot 名不随 wire 携带（目录序号=函数索引）；owner/名字沿信封责任在
  adapter 侧重绑定/重盖章后必须过真实 verify。
- 密度（小索引 body）：BIN=5B、COPY=3B、RETURN=2B、CONST=3B（section 差分实测）。
- PLAN-823 r8 env 合同：携带 env 布局的函数/导入在 encode 侧拒绝、解码侧 `env:None`。
- 预算与失败分类沿用已交付常量（输入 4MiB、模块/函数表 65536、decode 200万步/5s、
  decoded ≤16MiB）；分配前检查；format/resource/verify 分态。
- 独立模块与集成边界：单一生产源 `core/src/compact_abc.rs` 同源于 parallel/826 壳
  与正式注册（lib.rs `pub mod`）；T-10 壳 9/9、正式 first/targeted/compat/incremental/
  baseline/bench 六 scope 全绿。

## SD-02（modify）docs/specs/auto-av/cache-and-conformance.md

- A09 单元键 `abc-codec` 维度取 wire schema（`abc_unit_key_with_codec`）；两世代
  键空间不相交，跨世代=确定性 miss/rebuild（O(1) magic+version 世代闸门按策略判）。
- `ModuleManifest` 新增 `abc_codec`（serde 默认=旧 AVAB schema；不入 fingerprint——
  codec 差异由 payload_fp 区分）；restore 按 manifest codec 路由解码/准入，未知
  codec 拒绝；恢复不继承授权（current grants 必检）。
- 新旧 wire 均不恢复 trusted；每一层 envelope→decode→verify 重验。

## SD-03（modify）docs/specs/auto-av/project.md

- 登记实际优化面：同语义模块 raw 体积 compact/legacy 全组更小（13 组实测
  0.361–0.856；852 五源基线 917/1739=0.527；gzip 同算法同参数分列见 bench-r1）。
- 未实现面如实：短立即数/slot 复用/指令融合/常量池重写未实现（未来需新版本+独立
  反控）；不承诺公共稳定 ABI；无 lazy trust/JIT。

## SD-04（modify）docs/specs/auto-test-case/architecture.md

- 追加 recipe `ac-av/compact-run-v1`（REGISTERED_PROBE_RECIPES + avworker/1 步骤
  词汇 run|edit-source + av binding；worker case-serve 接受并 set_wire_compact）。
- fixtures `test/cases/av-abc/826/`（first-case 同 825 首例源）；父侧
  judge_compact_runtime 以 `abc_wire_compact_units/closure` + loaded 计数器判真，
  AV826_JUDGE_SABOTAGE 反控；工具身份轴 `av-wire-codec`。

## SD-05（modify）docs/specs/auto-av/abc-core-i32.md

- 同一语义 profile 的版本路由：AVAB（原 wire）与 AVAC/1（紧凑）按 magic 隔离，
  双向不互相解释/准入；旧缓存键确定性 miss；`verify_compact_abc[_bytes]` 为 AVAC
  唯一 LoadedModule 准入入口（与 verify_abc 同深准入：重盖章+deadline verify+
  executable-closure 凭证）。

## SD-06（modify）docs/specs/auto-av/runtime-benchmark-method.md

- 追加新 wire 分段/压缩/loaded 体积口径：raw（两代）+ gzip 同算法同参数（CLR
  GZip Optimal）+ loaded/执行计时；engine 行走 compact 准入路径；
  schema `av.wire.826.bench/1`（13 工作负载×2 wire size 行 + 50 分布组）。
- 编码域≠准入域：ULEB 可表达的索引号与实际表长内的合法 ref 分离断言。

## SD-07（modify）docs/specs/auto-av/resident-function-install.md

- 828 增量接口（warmup/restore/persist/edit）带显式 wire 策略
  （warmup_with_codec/restore_with_codec/warmup_persisted_with_codec）；compact
  行落 codec 维度命名空间；同策略温重启 restore_units_admitted=全量、跨代冷回退；
  未改函数零生产工作/对象恒等在 compact 下保持（V-12 实测）。

## SD-08（modify）docs/specs/auto-test-case/ace-m1-conformance.md

- 852 底座分母追加本计划实际支持起点：binding 表
  （attachments/826/852-validation/binding.md）逐行映射 B01/B04/B09/B13/B14 双 wire
  对拍与 I01/04–07/10–12 场景；`-Scope baseline` 为正式入口。
- 范围外 gap 保留 owner：复合类型/字段布局源码编辑、source/native/AJ 面。
