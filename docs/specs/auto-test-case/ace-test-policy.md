# ACE测试接入策略与慢例清单

> PLAN-893 SD-01：ACE能力接入约定与13项慢例的当前策略元数据。数据本体是
> [scripts/test-governance/ace-test-policy.json](../../../scripts/test-governance/ace-test-policy.json)
> （schema `ace-test-policy/1`），本文钉住其语义与边界；治理测试
> `scripts/tests/test_ace_test_policy.py`（固定组 `infra-ace-policy`）是守恒判定。

## 语义与边界

- **有限策略数据，不是第二目录**：JSON 只记录13项原义务的家族owner、历史叶ID→当前原生身份
  （binary+完整test名+源文件）、迁移证据、职责组/formal组导航、单项重放命令与适用条件
  （ignore/windows）。不参与日常发现/选择/调度——执行仍由固定组与原runner决定，
  `fixed-tests.py` 正常运行不加载本文件。
- **三视图（人/归因视图，不是执行器）**：
  - `affected`：计划作者与review明确受影响完整职责组及必要消费者并集；不存在`--changes`
    自动影响图。
  - `slow`：本清单13项的当前原生命名导航与显式ace-perf单项重放；重叠经固定driver同配置OR一次去重，
    缺项/未测不绿。
  - `formal`：原能力正式门仍是显式`cargo t ace-perf`及原脚本/样本/阈值/负控；
    慢例单轮不充正式P90。
- **身份冻结**：当前身份=package/target/完整Rust test名；旧888叶ID仅历史对照。改名须由
  治理owner提供一一映射证据（如K-01/02：896 commit `5d7afcf78` 迁`ac-av-probe`→ace av
  conformance cases并去`t07_`前缀，断言体随迁）。
- **13项全部处于`::performance::`命名空间**（T-01真实nextest发现冻结；静态rg/花括号解析
  曾误判4项，以发现为准），formal轴导航均适用ace-perf；K-07/11/13为
  `#[cfg(windows)]`条件。
- **守恒判定（V-01）**：策略必须恰好13项、原生身份唯一、fn真实声明于记录源文件且模块
  路径吻合、windows/ignore条件与源码一致、重放命令引用已注册固定组与冻结名；缺失/多余/
  重复/旧叶名冒当前名/不存在的函数/未注册组/formal轴错挂一律拒。

## 新增能力接入约定（未来consumer）

新增能力首例须文件化TestCase（适用source消费）、独立oracle、fresh/hit/restart/body-edit、
恢复重验、未改函数真实生产0与居民身份、内存/清理预算；不适用须具名合同依据。typed例不冒
source；A2X日常emit/golden与显式两腿compile-run认证分轴，日常不隐式启动rustc。现有真实
consumer（case-av-source-827 source轴、case-ace-m1-typed-native-891 typed轴、case-ac-av
资源轴、a2r组892 emit日常轴）是约定的活检验，不新造runner。

## pilot CLI 接入与工具形态（PLAN-1014 SD-03）

本节只补语言侧 pilot 治理的接线导航，**不改变上文13项策略分母与守恒判定**。
真实 pilot CLI 检查（原 test_case_pilot_cli.PilotCLI 8 例）由显式 ACE 域组
`infra-pilot-ace` 承接：声明工具 case-parent（pilot exe，debug target）、
batch-av（feature=test-case-batch，target-batch，供 group/combined）、
av-regular（无 feature，target/test-tools/regular，供 default-off 原路径），
source-bound 准备+运行分账。纯 Python 回合/干扰核算（2 例）留 legacy
`infra-pilot-cli`（零 ACE 准备路径）。infra-pilot-ace 为 all=false 显式选择组
（`cargo t all` 语义约束：语言 only），与 ace-host 同队。守卫/断言口径见
[fixed-regression-baseline-and-ownership](fixed-regression-baseline-and-ownership.md)。


## 日常选择与正式重放（PLAN-1016）

21个ACE常规nextest组统一排除performance；原13项owner仍是模块职责归属，重放入口统一为
`ace-perf --test-name <完整Rust名>`。13项清单不是全部PERF的目录，不用它限制新归类负载；
正式套件正向选择整个命名空间。原CLI ace-performance已更名，不保别名。
治理反控拒绝把正式重放写回普通模块组，配置守卫保护组互斥与显式触发，真实nextest list核对
三组的常规/正式互斥并集守恒。源窗口/配置身份变化使旧采集不可复用，NOT_RUN不冒绿。


`ace-perf`统一使用串行profile（test-threads=1），父截止4小时与原单测截止分离。
13项历史清单保持13项，并非所有PERF的白名单；新增归类的正式压力通过命名空间自动纳入。
源CT服务包ace-source-cte已显式加入正式套件Cargo目标，防止只依赖编译而漏跑其原规模CT见证。
