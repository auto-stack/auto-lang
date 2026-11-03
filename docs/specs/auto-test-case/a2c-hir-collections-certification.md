# a2c-hir collections 认证合同（PLAN-926）

> 依据：PLAN-926 §5.5；实现：worker `certify-c`/`daily-emit-c`（collections
> profile）、套件 `crates/auto-lang/test/cases/a2c-hir-collections-926/`、固定组
> `case-a2c-hir-collections-926`。沿
> [a2c-hir-scalar-certification](a2c-hir-scalar-certification.md) 与
> [a2c-hir-record-text-certification](a2c-hir-record-text-certification.md)
> 的双侧/严格身份纪律；本合同只记 collections 增量。

## 双侧认证（certify-c，collections profile）

- **ACTUAL 腿**：真实 MSVC cl（/std:c11 /TC，vcvars 派生环境）编译发射的完整 C
  文件集并运行，观察 `value:<i32>`/`trap:<Name>`（二进制模式 stdout，LF 终止）。
- **REFERENCE 腿**：逐案例**手写 std-only C 参考**（套件内 `reference-<case>.c`，
  独立实现——不翻译 actual、不用生成 runtime），独立翻译单元同 cl 编译运行。
- 两腿各自等于冻结 oracle（hand-computed）才 PASS；任一腿不符=受控失败（error
  status，退出码非零）——绝不产出错误 verdict。cl 恰 2 次、rustc 0。

## 证书（ace.a2c-hir.collections.cert/1，严格）

- 通道三元配对：schema×recipe×fileset 严格绑定（标量/record-text/集合互不相容，
  跨通道冒充=validate 拒）；**execution_config 为必需键材**：canonical 渲染
  `{"budget_work_units":…,"cancel":…,"events":…,"fault_alloc":…,"mode":…}`
  （与 904 Rust 集合证书字节同构），mode∈debug/release 为键——不同配置身份互斥。
- 绑定：source_sha256（全源）、input_hash、实际文件逐 path+sha256、wrapper/
  reference sha、工具链身份（cl banner+二进制 hash+vcvars hash+冻结 flags）、
  backend/standard/协议。缺字段/空身份/裁剪文件集=拒。
- 20 冻结源 oracle 表：L01/02/04/05/07/V01/02/04..07/FIN864A/N889×3=42、L03=0、
  V03=3、LSTR864=51、FIN864B=53、L06=trap:ListIndex。

## 日常门（spawn-zero）

- `daily-emit-c`（collections profile）：仅现场发射+字节校验，cl/link/rustc/cargo
  实际 spawn=0；manifest 携带 `auto.a2x.c.collections.fileset/1` 与全源 sha256。
- 固定组 `case-a2c-hir-collections-926`：20 案例四文件金样（main.c/
  a2x_c_coll_runtime.h/generated.c/manifest.json）字节比对；金样自真实 worker
  发射冻结（双 nonce 确定性）。认证轴（真实 cl 双侧）由 worker 一致性测试
  `a2c_coll_cert` 持有，非日常门。

## 正式采样（V-12）

- 冻结 904 完整负载（共享 `coll_formal_load`：≥40 函数含 16 data helper、32 未改
  caller、非注释≥16KiB、字符串数据≥4KiB）四态（fresh/hit/restart/body-edit）
  ×≥30 样本；512 真实不同修订（256 left_pad + 256 right_pad 变长体编辑）逐修订
  真实 cl 预认证对闭式 oracle；暖 P90≤2000ms（P50/95/99 单列）；新进程恢复见证
  （P0 计数 + 恢复后下一编辑双侧认证）；未改单元计数（编辑态 lower/print 恰 +1）。
