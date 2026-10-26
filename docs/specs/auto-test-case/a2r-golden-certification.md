# a2r 金样独立认证（一次认证、日常快照）

> 状态：delivered（PLAN-852 T-03/T-04；入口 `crates/auto-lang/src/bin/ace-certify.rs`
> → `auto-lang-core/src/test_cases/ace_certify.rs`；证书
> `crates/auto-lang/test/cases/ace-m1/oracle/certs/*.cert.json`，schema
> `ace.m1.cert/1`，随库版本化、可跨检出审计）。

## 用户速度裁定

日常 a2r 只 **emit + 逐字节比较已认证金样 + 证书文件轴核查**；rustc/cargo/目标
Rust 进程 spawn=0（verify snapshot scope 的结构守卫 + 认证器代码边界双保险）。
真实 Rust 行为核验是**显式独立操作**（ace-certify），任何日常路径不得隐式调用。

## 认证流程（certify）

对每个 `ok-emit` 值例（binding.tsv 驱动）：

1. 由生产转译器对源码**现场生成 actual**（绝不把金样当 actual）；
2. actual 全文件集必须等于冻结金样（漂移即 FAIL；金样只经显式审阅更新，无自动
   更新——G04）；
3. actual 以 byte-identical `generated.rs` + 冻结 wrapper（`mod generated {
   include!(...); pub fn entry() -> i64 { main() } }`，全文 hash 计入证书身份）
   经真实 rustc 编译运行；手写 `reference/*.rs` 同样编译运行；**两者各自**等于
   固定行为 oracle `i32:<十进制>\n`；
4. 写证书：source/golden/actual/wrapper/oracle/reference 六 hash 轴（wrapper 单源
   =语料库版本化文件 `oracle/wrapper-main-v1.rs`，代码 include_str 引用）+ rustc
   `-vV` 完整身份与冻结 flags + 依赖闭包（标量集=纯 std）+ 双侧实测输出。转译器
   身份仅审计，不作失效轴。

## 复用与失效（R1 修复后）

- 复用仅在**严格收据验证**通过后发生：schema=`ace.m1.cert/1`、case/module/verdict/
  oracle 文本（绑定 binding.tsv 冻结登记值）/双侧 observed 输出与 exit/冻结 flags
  `["--edition=2021","-O"]`/deps=std-only 且 external 空/actual==golden 自洽——任一
  结构破损即 **REJECTED**（疑似篡改）：不复用、不被覆盖（--force 也不行），须人工
  删除/修复后再认证（F-852-R1-01）。
- 文件轴（source/golden/wrapper/oracle/reference）漂移 → STALE，显式 certify 重认证；
  日常 snapshot scope 对全部 ok-emit 行做同套严格验证（零 spawn），缺证书=显式缺口
  红而非静默。
- **身份绑定实际启动的编译器（F-852-R2-A）**：证书 `toolchain.identity` = 认证当次
  实际启动的 rustc 的完整 `-vV`（经 contained 有界通道捕获；CLI --rustc > env RUSTC
  > PATH 单一规则，`identity_source` 记录来源）——绝不以构建期常量冒充认证证据。
- **toolchain 漂移日常即检出（零 spawn）**：证书另冻结 `default_identity_at_certify`
  （认证时构建期默认）；suite 报告携带 `build-rustc-identity` 轴（build.rs 捕获），
  snapshot 将两者比对——默认工具链漂移即 STALE（G05，无延迟核查弱化）；显式覆盖
  编译的身份属冻结证据，日常消费不重算。
- 认证只证明冻结平台/依赖 profile，非所有平台证明；Rust 编译失败/缺工具/超时
  /panic 均为 infra 或编译失败，永不满足 Auto 语言负例。

## 资源治理（T-01 冻结预算，实测基线 0.12~0.19s/例；R1 修复后完整覆盖）

编译墙 120s/例、运行墙 30s/例、输出 cap 1MB/流。全部工具进程（rustc 与被认证
二进制）以 CREATE_SUSPENDED + Job Object（kill-on-close）contained 执行；stdout 与
stderr 由**独立**有界读线程并发排水（一管满不阻塞另一管，F-852-R1-03 复现例
stderr-first 128KiB 1.3s 通过）；任一流超 cap 即**杀全树**并向父侧报溢出错误，
绝不静默截断；墙到同样杀树（真实挂起程序墙控负控绿）。工具身份默认取 build.rs
构建期捕获（零 spawn）；显式 `--rustc` 覆盖时经同一 contained 有界通道捕获。
scratch：显式 `--scratch <dir>` 或隔离 temp（PID 域），全退出路径整体清理。

## 范围边界

当前认证集=标量值例（10 例）。a2r 复合产物为伪 Rust（`List<int>`/`int` 不可编
译），复合行为认证是登记 gap（owner=a2r 复合 lowering）；陷阱/诊断负例按前端拒
绝通道判决，不生成可执行金样。
