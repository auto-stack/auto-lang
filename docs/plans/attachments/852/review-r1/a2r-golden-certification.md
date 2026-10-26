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
4. 写证书：source/golden/actual/wrapper/oracle/reference 六 hash 轴 + rustc 身份
   与生效 flags + 依赖闭包（标量集=纯 std）+ 双侧实测输出。转译器身份仅审计，
   不作失效轴。

## 复用与失效

- 身份轴 + toolchain 全等且 verdict=certified → 复用证书，本次零编译（G03/G08）。
- 金样/oracle/参考/源码漂移 → 证书 STALE，verify snapshot scope 文件轴核查即红
  （G05）；toolchain/依赖漂移在下次 certify 时检出（日常不 spawn rustc 故不在线
  核查——记录边界）。
- 认证只证明冻结平台/依赖 profile，非所有平台证明；Rust 编译失败/缺工具/超时
  /panic 均为 infra 或编译失败，永不满足 Auto 语言负例。

## 资源治理（T-01 冻结预算，实测基线 0.12~0.19s/例）

编译墙 120s/例、运行墙 30s/例、输出 cap 1MB/流；rustc 与被认证二进制以
CREATE_SUSPENDED + Job Object（kill-on-close）contained 执行，墙到杀树
（真实挂起程序 2s 墙负控绿）；scratch 全路径退出清理。

## 范围边界

当前认证集=标量值例（10 例）。a2r 复合产物为伪 Rust（`List<int>`/`int` 不可编
译），复合行为认证是登记 gap（owner=a2r 复合 lowering）；陷阱/诊断负例按前端拒
绝通道判决，不生成可执行金样。
