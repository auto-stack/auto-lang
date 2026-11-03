# a2c-hir-scalar 认证合同（PLAN-921 CA-A 首例）

> recipe `ac-a2c/hir-scalar-emit-v1` | cert `ace.a2c-hir.scalar.cert/1`
> | fileset `auto.a2x.c.fileset/1` | 层 AXC1/AXP1

## 日常通道（spawn 零）

- 首例套件 `a2c-hir-scalar-921`（固定组同名，tools=[case-parent, a2x]；
  recipe 前缀映射 `("ac-a2c/", "a2x")`）：worker 单进程
  `daily-emit-c`，父适配器（`ac_a2c_hir.rs`）断言协议/recipe 回显、
  `spawned_rustc==0 && spawned_cl==0`、manifest 重哈希、路径域审查后
  4 文件 golden 字节判 + 未声明多余产物拒绝。emit 对 nonce 确定
  （不同 nonce 字节同一，测试断言）。
- golden manifests 的 .gitignore 豁免已登记
  （`!.../a2c-hir-scalar-921/golden-*/manifest.json`）。

## 认证通道（真 cl 双腿，strict 证书）

- `certify-c`：actual（生成集）与 caller 供**手写** C reference
  （独立平移单元，同一受管执行器/冻结 flags）各自真实编译运行，均须
  等于同一闭式 oracle（value:n / trap:Name 字节协议；reference 亦须
  二进制模式 stdout）。reference 不匹配=错误（不可充当 oracle）。
- **证书身份**（`ace.a2c-hir.scalar.cert/1`）：schema/recipe/case/
  profile/hir_domain/entry/verdict/oracle_kind/value/input_hash/
  source_sha256/actual_files[](path+sha256)/golden_sha256?/
  wrapper_sha256/reference_sha256/backend=auto-a2x-c/backend_version/
  standard=c11/shared_view_version/protocol/toolchain{kind=cl,
  version(banner),version_sha256(cl 二进制 hash),vcvars_sha256,flags}/
  capabilities。字节稳定序列化（同身份同字节，测试断言）。
- **strict 拒绝面**（测试为证）：旧 schema/recipe 冒充
  （ace.m1.cert/1、ace.a2r-hir.*、ac-a2r/*）、裁剪 actual_files、
  verdict 非 PASS/REJECTED、oracle_kind 非 value/trap、backend/
  standard 错配、工具身份不全（空 flags/坏 hash）、文件绑定畸形——
  全部 REJECTED；错 oracle 在双腿被 `a2x.oracle` 拒绝。

## 账本分离

- 旧 M1 acceptance（binding.tsv/applicability.json/required313）不因
  C 通道改变；C 首例独立登记（套件+固定组+本合同），CA-E 后续才定义
  AND 旧门+新 C 门的总签章——当前总 M1 不可因 C 而签。
