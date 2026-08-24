# AAVM 系列复盘(Plan 429-434)

> 2026-08-24 | 系列收官。范围:用 Auto 语言重写 Auto 编译器自身
> (前端 → VM → 转译器),直至自举回路中没有任何 Rust 手写的编译组件。

## 一、系列脉络与成就

| 计划 | 主题 | 交付 |
|---|---|---|
| 429 | prelude/rust 清理 + 基线 | A/B/C 三阶段;临时基线 b3bd64f5 声明;四份 spike 报告(shim 盘点/perf/a2r 语法覆盖/基线) |
| 430 | shim/metadata 工具 | 三方 crate 方法 shim 包管线端到端;Vec 全链路;unwrap_ok 策略 + D3 迁移 |
| 431 | v2 移植规范 | 边界清单(porting-boundary)/文件映射/divergence 规则与登记簿/corpus 分层/测试基建全套 |
| 432 | v2 核心移植 | 六文件 lib(token/lexer/parser/typeinfo/codegen/engine,4,266 行);M1-M5 闸门全绿;宿主修复 D26/D30;30 语料执行层稳定集 |
| 433 | a2r 闭环 | AAVM 经 Rust 版 a2r 转译为纯 Rust(零 a2r_std,329→0 错);四向矩阵 30/30;242 #16 半收口;a2r 修复 12 项 + D32-D37 |
| 434 | AA2R(本计划) | **终极自举闭环**:Auto 版 a2r(a2r.at)转译含自身的七文件 lib → 纯 Rust → cargo 编译 → VM 运行 corpus 30/30 与参考一致;五方矩阵 |

**系列定性结论**:Auto 语言的自举能力在"编译器前端(432)→ 宿主转译(433)
→ 自主转译(434)"三级跳中得到完整验证。最终回路:

```text
Auto 写的 a2r.at ──(转译)──> 含 a2r.at 的 auto/lib(七文件)
      └──────────────────────┬──────────────────────────┘
                             ▼
              纯 Rust 产物(可独立 cargo build,零 a2r_std)
                             ▼
              该 VM 运行 .at 程序(corpus_m4 30/30 = ①参考)
              该 VM 亦可运行 AA2R 自身(塔可任意加层)
```

G2 演示(可复现):`helloworld.at → "hello, world!"`、`fib.at → 55`
(命令序列见 Plan 434 执行结果节)。

## 二、434 的方法学沉淀

- **token 游标直走**(D39):后端不走 S-expr dump(字符串字面量内引号
  歧义不可判定),复用 parser.at 的游标/优先级表直接走 token——与 432
  codegen.at 同一方法论,三次复用证明该模式可持续。
- **预扫描家族**:fn 签名/字段/枚举表 + 每 fn 再赋值扫描(var→let mut)
  + last-use 扫描(实参克隆决策)+ mut-参数表(D37 同族决策的 AA2R 口径)。
- **作用域栈槽位清空**(D38d):depth 计数模式(D25)的跨 fn 遮蔽隐患在
  AA2R 上实证并修复——后续任何沿用 D25 模式的代码都应带上该清空。
- **宿主缺陷协议**:VM 在新路径上的缺陷(char_at 边界 panic,D38-VM)
  按既有协议修复 + 登记;主 a2r 侧缺口按"归因三分类"挂 242,不阻塞
  系列收官(矩阵 ② 范围化处理)。

## 三、遗留与下一定位

| 项 | 状态 | 去处 |
|---|---|---|
| AA2R golden 覆盖:01/03 组大部分字节级一致;02/04/05 组部分;06+(is-match/闭包/spec/use/泛型)未移植 | 余量项 | 434 计划内 S1/S2 未尽部分,差异清单见 divergences.md D40 |
| S2(use.rust 直通 + dep/Cargo.toml + a2r_std_used 完整版) | 未做 | 仅 math 内建(max/min)+ a2r_std_used 头块已实现 |
| 主 a2r 对 a2r.at 的 45 错(`.get(i).field` 链 Option 化、&mut str 字段读无 clone) | 不修 | 242 tracker 新条目;修复后矩阵 ② 回归整目录 |
| VM 242 既有缺陷(枚举载荷跨 fn 丢标签等) | 挂账 | KNOWN-DEBT-AND-RISKS.md |
| ③ 承载 AA2R 全量转译耗时(分钟级) | 已知 | 仅一次性构建成本(⑤ 二进制内容寻址缓存);VM 性能优化不在系列范围 |
| 下一定位 | — | AA2R 扩覆盖(golden 全组 + is-match)或转入生产化评估;自举实验目标已达成,建议系列封版 |

## 四、结语

429 立项时的问题——"Auto 能不能自己编译自己"——在 434 得到了肯定且
完整的回答:**能,且产物是零依赖的纯 Rust**。系列各计划的 spike→规范→
移植→闭环→自举的推进节奏,以及 divergence 登记簿 + 多向对比矩阵的判据
基建,可作为后续语言自举实验的模板。
