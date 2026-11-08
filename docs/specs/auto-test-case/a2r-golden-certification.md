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
绝不静默截断；墙到同样杀树（真实挂起程序墙控负控绿）。证书身份恒为**实际启动的
编译器**经 contained 有界通道实测的 `-vV`（CLI --rustc > env RUSTC > PATH 单一
规则，无"默认取构建期常量"路径）；构建期常量仅作 `default_identity_at_certify`
旁轴入证书，供日常零 spawn 漂移比对，不作认证身份。
scratch：显式 `--scratch <dir>` 或隔离 temp（PID 域），全退出路径整体清理。

## 范围边界

当前认证集=**28 例值例**（10 例标量 + PLAN-992 认证的 17 例：12 例既有复合金样
经 G04 语义重审后认证 + 5 例新冻结可编译金样 B08/B11/R01/R03/R04 + PLAN-993
认证的 K06 写捕获闭包 1 例）。PLAN-992
（2026-11-08）勘误：binding 曾注"a2r 复合产物为伪 Rust（`List<int>`/`int`）不可
编译"——该过度概括对已认证的 17 例不成立（record/enum/闭包/遮蔽/命名实参/move/
深clone 等形状经真实 rustc 编译运行 == oracle）；伪 Rust 缺口仍真实存在于其余
**20 格** required a2r 值例/场景（H-03 文本 4、H-04 集合 6、
H-06 trap/宽度裁定 4、H-07 场景 6，owner 见 gap-audit F-A2R-3；H-05 K06 已由
PLAN-993 关闭），认证缺口账
38→21（992）→20（993）。陷阱/诊断负例按前端拒绝通道判决，不生成可执行金样。

## K06 写捕获闭包认证（PLAN-993，2026-11-08）

- **H-05 双 `move` 发射修复**：写捕获闭包的 Rc::clone 块已强制 move 语义，源级显式
  `move` 不再发射第二个关键字（旧产物 `move move |..|` 为 rustc 语法错，K06 首红
  实证）。四种 capture 组合（写+move/写无move/读+move/无捕获）恰一次或零次发射，
  已正确形状字节不变。
- **自引用 cell 赋值无重叠发射**：写捕获绑定上的 `x = <rhs 引用 x>` 按 Plan 347
  全局变量先例提升为 `{ let __a2r_wc = rhs; *x.borrow_mut() = __a2r_wc; }`——直接
  形 `*x.borrow_mut() = *x.borrow() + n` 运行期 BorrowMutError（RHS Ref 临时活过
  LHS 借用）。仅 Op::Asn；复合赋值保持原形（无已知语料组合）。
- **捕获克隆序确定性**：≥2 写捕获时克隆行按名字典序发射（HashSet 迭代序曾致金样
  字节抖动，T-06 复审发现并修复）。
- **认证事实**：K06 金样重冻结（`e28247fa…`，G04 逐行审阅），actual==golden，
  wrapper 与独立 reference 双侧 `i32:42`；证书 27→28（`oracle/certs/m1_k06.cert.json`），
  旧 27 例 strict 复用不重认证；cert-gap 21→20。永久反例：
  `test/a2r/25_lifecycle/005/006/007`（写+move/写无move/多写+move 跨调用共享，
  实跑 42/42/67）。
- **边界**：嵌套闭包写捕获不获 Rc 降级（D-993-1，先存缺口另账）；本规范零gap守卫
  与历史原值不变。

## 诊断负例面与双轴（PLAN-908，G-DEC-3 2026-11-01 裁定）

- **六个 a2r 诊断负例保持 required**（corpus B17/B18/B19/T06/R09/R10）：义务面 =
  a2r 前端以**正确语义族拒绝**（B17 `type`、B18 `assign-immutable`、B19
  `missing-return`、T06 `borrow-escape`、R09 `nominal-type`、R10 `use-after-move`），
  **不生成可执行金样**——与"诊断负例不生成可执行金样"的本规范一致；也不得以
  rustc 编译失败、任意非零或统一 unsupported 冒充拒绝。
- **PLAN-916 已关闭旧 A2R 八格（2026-11-02）**：B16/B17/B18/B19/T06/R09/R10 与 E03
  的 a2r 列均为 `ok:P916`。生产 single/project 入口在 CTE/发射前返回可辨识 typed
  `NameError`/`TypeError`；14 项 adapter 负例检查零 accepted artifact，NB17/18/19 另有
  三个独立 emit reference。R07/T08 与其他后端差异不在本次关闭范围。
- **历史（908 T-04 探针，2026-11-01）**：a2r 对 B17 仍 emit 成功，六负例与 B16/E03
  名解析曾为真实缺口。G-DEC-2 裁定语义族映射（保 raw stage/code/message 及可辨识
  原因），不要求后端错误码字面相同；B17–19 ac 侧 `mn.unsupported.frontend` 为
  泛化码，不满足可辨识原因，保持 `diff:pending`（后由 PLAN-911 关闭）。
- **双轴不降格**：金样/emit 轴绿 + 认证缺 ≠ 未认证冒进，坏证书也不能降成"未认证"
  绕过 strict 防伪（REJECTED→不复用/不覆盖）；acceptance scope（`ace.m1.acceptance/2`）
  分轴报告 38 个 required a2r 格的 cert-gap，日常零编译（rustc/cargo/目标 Rust
  spawn=0），certify 恒显式。892/902/904 新 HIR A2R 证据并列，不自动顶替旧 a2r
  身份或旧负例义务。

## PLAN-914 场景证据与旧 schema 的隔离（2026-11-02，SD-05）

- 本文件的 `ace.m1.cert/1` schema、10 例标量认证集、六 hash 轴与复用/失效规则**不变**。
- PLAN-914 为 E01/E02 的旧 a2r 路径新增的只是**场景证据**：命名调用以闭包形式发射
  （`(|__a0,__a1| sub(__a1,__a0))(实参源码序)`），金样落在
  `order-layout-914/oracle/m1_e0{1,2}.rs`；显式认证对 actual 与 tracked 独立 reference
  分别编译运行，两侧 stdout 均为冻结 oracle 串 `bai32:7\n` / `bai32:103\n`。
- 该证据不写入 `oracle/certs/`、不改 `ace.m1.cert/1` 字段含义、不把新场景并入 38 格
  复合认证；日常仍是 emit + bytes，零 rustc/cargo spawn。
- 非标量形状的全命名调用仍是历史发射（登记边界）；扩到复合/字符串见后续能力线。

## 917 本域证据隔离（PLAN-917，SD-04）

PLAN-917 为 ace-m1 R07/T08 及其合法近邻建立独立认证域 `ace.m1.a2r-emit917.cert/1`：
receipt 存于 `a2r-emit-917/certs/`，与本文所述 852 标量域（`ace.m1.cert/1`，
`oracle/certs/`）严格隔离——旧消费者（按 `ok-emit` 扫描 binding.tsv 的 certifier 与
verify 脚本）不会误收本域证书。日常轴仍是零编译 emit+字节比对（固定组
`case-ace-m1-a2r-emit-917`）；认证轴经显式入口 `a2r-emit-917-cert` 执行，
receipt 结构校验为硬错误，文件漂移=诚实重认证，无 --force 覆写路径。

## 17 例值例原域认证（PLAN-992，2026-11-08）

- **认证集 10→27**：H-01 的 12 例既有复合金样（T05/R02/R05/R06/V01/V02/V04-V07/
  K01/K03，字节未动）经逐源 G04 语义重审后显式认证；H-02 的 5 例（B08/B11/R01/
  R03/R04）从当前旧 producer 现场发射提案、G04 审阅后冻结新金样并认证。全部 27
  份证书 schema `ace.m1.cert/1`、flags/std-only/资源帽不变；旧 10 份仅 strict
  复用未重认证。B11 的 914 命名实参闭包形（`(|__p914_a0,__p914_a1| sub(__p914_a1,
  __p914_a0))(3,10)`=7）经审阅确认为预期生产形态。
- **日常面 26→31**：新增 a2r-only 扩展套件 `legacy-a2r-cert-992.{case,suite}.ad`
  （仅 5 例新金样快照检查，corpus 域；原 12 例检查留 legacy-composite 不复制），
  固定组 `case-legacy-a2r-cert-992`；snapshot scope 双套件插桩运行并集守卫 31，
  认证面 spawn=0 判据不变（G02 维护链豁免精确到 cargo/sccache/rustc 祖先与纯
  `--print` 身份查询，直连 rustc/m1_* 二进制仍违规）。
- **缺口账 38→21、零 gap 守卫不变**：certifier 选择谓词（corpus∧value∧ok-emit）
  未扩大——是 17 格 binding token 经 G04+显式认证后转 `ok-emit`；`can_claim_m1`
  仍要求 cert-gap=0 且 future=45 在账，本轮后余 21（见 gap-audit F-A2R-3），M1
  不签。坏金样/坏证书/漏检查/错 expected 反控与"无半证书、--force 不覆写"行为
  保持。

## 四例文本值例原域认证（PLAN-994，2026-11-08）

- **认证集 28→32**：H-03 的 4 格（T01/T02/T03/T04）经旧 producer **lowering 修复**后
  认证关闭——声明型字符串局部（显式 `let`/`var` 声明 `str`/`Str`/`str(N)`）
  `.len()`=标量字符数（`(name.chars().count() as i64)`）、`.len_bytes()`=UTF-8 字节数
  （`(name.len() as i64)`），自包含 std 表达式（无 a2r_std 依赖，std-only 轴不变）；
  声明型 `char` 局部 `.to_codepoint()`=标量值（`(name as u32 as i64)`）；两参
  `slice`/`sub` 于字符串接收者恢复 PLAN-917 冻结的字符位守卫块（接收者 `&(recv)`
  绑定一次、双界各一次、越界 panic）。四新金样 `oracle/m1_t01..t04.rs` 经逐源 G04
  审阅冻结；4 certified（双侧编译运行 `i32:38/27/23/128512` 对 oracle），旧 28 例
  strict 复用未重认证；cert-gap **20→16**。
- **日常面 31→35**：新增 a2r-only 扩展套件 `legacy-a2r-text-994.{case,suite}.ad`
  （仅 4 例新金样快照检查，ID `ace.m1.a2rt.t01..t04` 避让既有 t-r-native 同名），
  固定组 `case-legacy-a2r-text-994`；snapshot scope 三套件（base+ext-992+ext-994）
  插桩运行并集守卫 35，认证面 spawn=0 判据不变。
- **D-993-2 随修复关闭**：PLAN-956 曾把 `emit_str_slice_chars` 换成
  `a2r_std::str::str_slice` 字节域调用且未重冻 917 金样，致 case-ace-m1-a2r-emit-917
  三红；恢复守卫块后三检查回到绿（917 金样字节未动）。公共 `a2r_std::str_slice`
  字节 API 对其显式调用方不变。
- **接收者分类边界（冻结）**：参数、记录字段、match/is 模式绑定、推断局部与调用
  结果的 `.len()`/`.length`/`byte_len` 保持既有字节/计数 remap（已批准差异，不在
  本轮统一）；`.len(n)` builder 设置器与用户 record 同名方法不受名称匹配改写。
  3 个 cookbook 冻结期望随声明型 len 投影具名修订（ASCII 值恒等，仅发射形变）。
- **缺口账 20→16、零 gap 守卫不变**：余 16 = H-04 集合 6 + H-06 trap/宽度 4 +
  H-07 场景 6（见 gap-audit F-A2R-3）；`can_claim_m1` 仍要求 cert-gap=0 且
  future=45 在账，M1 不签。坏金样/坏证书/漏检查/错 expected 反控与"无半证书、
  --force 不覆写"行为保持。

## 六例有限集合值例原域认证（PLAN-996，2026-11-08）

- **认证集 32→38**：H-04 的 6 格（L01/L02/L03/L04/L05/L07）经旧 producer **0 参
  泛型构造 lowering 修复**后认证关闭——`List<T>.new()`（`GenName("List<int>")`/
  `Bina(List Lt T)` 两解析投影）发射 typed `Vec::new()`；泛型参数经单源
  `generic_list_ctor_elem_ty`（标量+已知 struct；嵌套/非有限域→None 保持落穿）
  真实流入绑定注解（`Vec<i64>`/`Vec<Item>`/`Vec<String>`，绝不 JSON 动态桶）。
  List 接收者 Auto 语义方法面：get=索引+`.clone()` 拥有值读（433 A1 冻结约定）、
  set=索引赋值**替换**（PLAN-019，非 insert 右移）、insert/remove/clear/push 原生
  Vec 形、len=元素计数 `as i64`。六新金样 `oracle/m1_l01..l07.rs` 经逐源 G04
  审阅冻结（求值序/单求值/拥有转移/L04 字段位字节 remap 面与独立 reference 双侧
  `i32:42/42/0/42/42/42`）；6 certified，旧 32 例 strict 复用未重认证；
  cert-gap **16→10**。
- **域边界（冻结）**：1 参 `List<T>.new(expr)` 剥离（musk-022）与裸 `List.new()`
  （PLAN-010）识别面不动——GenName 泛型 1 参形的逐字发射是冻结金样载荷
  （`25_lifecycle/002`）；未解析元素型保持跳注解既有行为（不注 JSON 桶）。
- **日常面 35→41**：新增 a2r-only 扩展套件 `legacy-a2r-list-996.{case,suite}.ad`
  （仅 6 例新金样快照检查，ID `ace.m1.a2rt.l01..l07`），固定组
  `case-legacy-a2r-list-996`；snapshot scope 四套件（base+ext-992+ext-994+
  ext-996）插桩运行并集守卫 41，认证面 spawn=0 判据不变。
- **证书入库纪律（F-996-R1-01）**：`oracle/certs/*.json` 会被根 `.gitignore`
  的 `*.json` 吞没——新证书必须显式 `git add -f` 入库（852 先例）且与
  source-manifest 钉值逐字节一致；证书含 `created_at`，再生成即破 SHA，入库
  是唯一交付路径。
- **缺口账 16→10、零 gap 守卫不变**：余 10 = H-06 trap/宽度 4 + H-07 场景 6
  （997/998 域）；`can_claim_m1` 仍要求 cert-gap=0 且 future=45 在账，M1 不签。
  坏金样/坏证书/漏检查/错 expected 反控与"无半证书、--force 不覆写"行为保持。
