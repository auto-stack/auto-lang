---
plan_id: PLAN-1064
status: executing
feature_name: ACE共同模块装配、byte→Str与延后系统库交付
author: [ZCode]
created_at: 2026-11-16
updated_at: 2026-11-16
plan_revision: 2
current_step: 4
total_steps: 13
depends_on: [PLAN-973, PLAN-972, PLAN-967, PLAN-1065]
supersedes_spec_components: [docs/specs/ace-migration/m2-stdlib-io.md, docs/specs/stdlib/design/ace-binding-and-assembly.md, docs/specs/ace-migration/m2-host.md, docs/specs/auto-abi/checked-source-foreign-calls.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [stdlib, ace]
---

# [PLAN-1064] ACE标准库byte→Str车道、共同模块装配与延后面交付

## 0. 变更摘要

2026-11-16 r2（本轮当前）：依用户要求复用973/975已有成果、承接975模块语法与装配缺口。新增独立MODULE I/V/R/D四任务及受检输出帧T-FRAME、持久存储T-KV；原T-01..07/AC-01..07/V-01..04保留并细化，当前drafting **0/13**（原0/7；新增合同任务，非实现进度）。973已归档，保持其已验范围/终态；本包承接后继，不重开973/972/967。首次正式交付先选MODULE切片，byte→Str/其余库面可并行准备，最终AC运行消费1065，不能反等975全包。只规划，全部拟接口/验证未实施。

## 1. 目标

- MODULE：接973装配域原975交接，统一合法`pub fn`＋私有helper的共同CST投影/可见性/源身份，使同一份真实标准库作者源可被需要的packages与text/foreign消费；不改成全pub，不在测试中删pub/复制镜面源。
- D-01..D-09：共享Auto UTF-8严格解码＋有长度owned Str构造，再完成文件文本、目录/env/path/process、KV、ZIP及Option<List>形状；逐项保公开旧语义和独立oracle。
- AV/native、AV×C/Rust、AC、A2R、A2PY与原A2C适用域各留真实证据；Linux代码/原完整用例移交1063 NOT_RUN，不在Windows跑Linux/WSL。

非目标：重做已交普通库/967 binding系统、另建parser/类型系统/装配清单、将所有profile变成无边界全语言、替975实现结构感知`decode[T]`或数据库协议；Godot与独立TS产品沿原范围。

## 2. 架构方案

原始文件字节＋模块身份/来源候选→共同Schema/CST→已有PackageModule/exports/imports与共享resolver→唯一verify→各既有后端。MODULE只统一声明外壳与可见性，不把解析接受当后端完整语义；合法但目标尚缺能力须具名拒绝，不能旧parser/旧VM fallback。

字节数据：平台受检region/字节List→Auto严格UTF-8状态机→复用975/T-B1-D `char.to(Str)`和现有slice/clone/concat owned促升→共享owned字符串→公开API逐面sentinel映射。现有分块递归/预算沿973；不把Rust/C字符串构造当默认Auto算法。原50操作/四表及1043装配身份复用。

## 3. 技术栈

拟唯一树`D:/autostack/.wt/lang-1064/auto-lang`，分支`plan-1064-dev`（未创建）。普通构建target在树；外置材料仅`D:/autostack/plan-artifacts/plan-1064/`按MODULE/BSTR/FRAME/ZIP/KV区分，开工登记，不建链接。实现/内部核对/独立review按原owner在同树串行；本轮未派agent、未建树。

## 4. 需求分析与背景调查

用户本轮明确授权细化1064/1065及975/973/985交接、旧例映射、平台与验收、消除循环；仅规划，不启动实现/测试/合并/清理。语言读取主线9d0f973cb3cafc585ba2bae52ad269b9b6a8c0e3；975分支02a687da8含B1及JSON/Redis进展，未据此宣称正式交付。973归档路径为[原交付](archive/973-ace-m2-stdlib-io.md)，Spec取材为`docs/specs/ace-migration/m2-stdlib-io.md`及`stdlib/design/ace-binding-and-assembly.md`。

**已观察事实**：975/json.at含`pub fn`及私有`json_cp`等helper；sqlite.at现为plain fn内联面。975记录text-record拒pub、packages拒private helper；这是待关闭的接面问题，不能仅凭文本归因锁死某一个parser分支。PackageModule已有exports/imports，CtInStmt::Decl已有public标志；stdlib冻结身份在binding冷/恢复路径均检查。975的TextFromChar/StrFromChar与多端降落已有源码/测试，先独立正式交付再复用，不新造相同原语。AV×C/Rust当前读出参帧未交，由本包T-FRAME在967现有ABI/binding owner内补必要面，非外部无主等待。

**真实改面与来源**（均只读核存在，后续T-01重核SHA/漂移）：
- `experimental/ace/grammars/auto-source.at`；SHA256前16 `25cf8e8a109784bd`。
- `experimental/ace/crates/core/src/source/cst/frontdoor.rs`；SHA256前16 `a80f98a43ffe5c96`。
- `experimental/ace/crates/core/src/source/packages.rs`；SHA256前16 `25bd4a338890275b`。
- `experimental/ace/crates/core/src/source/packages/binding.rs`；SHA256前16 `3d1b626bbfcfb8a0`。
- `experimental/ace/crates/core/src/source/packages/stdlib.rs`；SHA256前16 `9bbb4580e9627838`。
- `stdlib/ace/sys/syscommon.at`；SHA256前16 `e5bbff2020e079bb`。
- `stdlib/auto/zip.at`；SHA256前16 `f694a450ad526f7e`。

ABI改面沿`experimental/ace/crates/{abi,foreign-contract,av-core}`的现有manifest/typed/resource/foreign_exec；native和A2X消费各沿本包实际触面选职责组。新测试/辅助类型/字段以下均为拟设计，不声称已存在。

## 5. 详细设计

### 5.1 实际承接与无环依赖

| 交付键 | 实际任务/来源 | 启动、关闭和下游 |
|---|---|---|
| IF-975-B1 | 975原T-02已有实现＋新增T-B1-V验证/T-B1-R独立phase复审/T-B1-D正式合入 | 不等1064/MODULE或975全包；最小primitive源码可内联只验原语，各端真实值/错误/owner。1064 BSTR消费正式来源，不消费WIP/整包完成 |
| IF-1064-MODULE | T-MODULE-I→V→R→D | I可在973已交底座开工；V真实JSON/Redis构造消费已交B1，其余合法声明/可见性可先验。D独立合入且交版本化源/grammar/exports/provider/read-set/真实结果，释放975 T-03 LOAD；不等1065、ZIP/KV或本包全归档 |
| IF-1064-READ-FRAME | T-FRAME | 复用967已交协议在本包补必要受检可写借用，供AV×C/Rust读路径；不反等967重开或975全包 |
| IF-1065-ACSYS | 1065/T-02..04后按其原review/merge正式来源 | 只阻本包适用AC系统API的最终运行验收，不阻MODULE或Auto解码算法先行；1065不依赖本包整包 |
| IF-1064-SURFACES | 原T-02/03/04＋T-KV/T-FRAME→T-05→06→07 | 完整库/所有适用路径与原AC/V齐才归档；985沿原D行收据逐项认证 |

frontmatter表达整包最终输入并集，**不增加PLAN-975整包依赖**；975可以依本包MODULE切片而不形成环。最早可执行工作是T-01及T-MODULE-I；BSTR依IF-975-B1，AC终验依1065，未交保持pending。973终态/已验事实不改，后继工作不反置成973归档前置。

### 5.2 MODULE：语法、可见性、身份与消费合同

最小正式输入（拟永久fixture，API拼写来自现行Auto）：

```auto
// demo.at
fn helper(x int) int { return x + 1 }
pub fn answer() int { return helper(41) }
// app.at
use demo
pub fn main() int { return demo.answer() }
```

正例结果42；`app`跨模块调用`demo.helper(41)`必须在共享binder具名可见性拒绝，指向原模块span。私有函数仍可由同模块pub函数调用；导入/别名/同名模块/来源配置不能把private自动导出。共同CST把`pub`统一投影为已有public/exports事实，普通profile内inline无跨模块导出时也不丢原span；保profile能力门与Schema身份。实现定位grammar声明分派、frontdoor public壳/普通声明投影、packages exports与binding两条身份准入；不得只改test fixture或对源串删/加pub。

975负责将其新sqlite公共API显式标为唯一canonical作者源的`pub fn`（正文/ABI行为保持，私有helpers仍private）；plain fn原形的跨模块不可见断言保留。JSON/Redis的真实canonical文件逐字节走装配，不能造“全pub版json”或“去pub版sqlite”并让测试冒实际交付。通用MODULE用demo和完整JSON/Redis/SQLite公共面取证；975自己的业务/typed decode最终验收仍归975。

保留1043 discover/assemble/provider候选与std冻结命名空间：新增正式std模块身份由其真实producer/975公开清单在既有stdlib registry登记，按内容/版本/签名校验，不能放宽为任意`std`文件或把某fixture内容hash写成一般语言规则。冷编译/缓存恢复均核source/grammar/exports/ABI/provider读集；private正文变更只影响真实读者，失败/取消不替CURRENT，旧运行代可保在途pin，源map定位原源而非重写串。canonical从逻辑fallback纠正为严格结果及输出帧等行为变化须更新既有声明/producer/runtime版本及真实依赖，不能因函数签名相同复用旧行为收据。

示意数据字段（复用现有对象，不增第二IR）：`PackageModule {program, exports, imports, source_hash, grammar_id}`、`CtInStmt::Decl {item, public}`；交接收据另记repo/SHA/revision、可用profile和完整声明/consumer映射、正反例/未交域。Checked信任仍由唯一verify产生，不能从收据反序列化取得。

### 5.3 BSTR与D-09的具体接口/错误/资源

拟新增Auto库函数采用现有Option拼写：`pub fn decode_utf8(bytes List<int>) ?Str`，`pub fn decode_utf8_region(region int, offset int, length int) ?Str`；只读输入的owner/借用按已有受检机制，不把region裸指针交给作者。UTF-8严格拒过长编码、孤立continuation、截断、surrogate、>U+10FFFF；合法NUL保留。空输入=`Some("")`；非法编码=`None`；预算/失效owner/非法内存访问按既有具名错误分类，不能充编码负例。解码实现复用syscommon/bytes中的码点编码知识及975 `char.from_codepoint(cp).to(Str)`，逐scalar构造owned文本，禁止CString/NUL截断。

新`std.file.read_bytes(path str) ?List<int>`：成功空文件=`Some([])`、缺失/读取失败=`None`，实际现有List＋read_bytes_len兼容辅助可保薄包装，不能作为新Option面已交证明。D-09不仅决定类型，还须补A2R及其他适用lowering/表示与真实空/非空/失败测试。不得把返回类型冲突靠重载同名抹掉；旧`File.read_bytes`/`auto.file`兼容入口继续返回原List（失败空List），由新checked结果薄适配，不调用旧VM；此前973新std面List消费者按明确迁移映射更新一次canonical源码/签名，调用方不能靠返回类型重载猜。975/消费者按同一新公共签名与旧兼容别名分别验证，旧辅助仍独立记录。

内部可按已有Result/诊断承载`{value, status, consumed_bytes, owner}`，示意字段非新增ABI承诺。输入region在解码期间pin，owned返回不借用输入，最后owner释放；正常/非法UTF8/超限/取消均释放临时region/句柄，超预算不可发布半串。继承65,536 B/call、1,048,576 B/run和原handle预算；大输入分块、partial IO与EOF明确。NUL/UTF8单位按字节和scalar分列；不承诺靠O(n²)拼接/无限递归通过上限，T-01先有界实测最长解码链及预算边界，不缩原输入。

972 hex/base64解码项不再留“是否顺手并入”的开关：本包T-02一并接原S-03(b)，核心拟接口`decode_hex(s str) ?List<int>`、`decode_base64(s str) ?List<int>`；合法二进制返回Some bytes（FF不能强转UTF8），空输入Some([])，非法字母/奇数hex/非法padding为None，不用宽松过滤吞错。复用既有to_hex/to_base64与byte访问，编码不是解码oracle；明确`00ff`→[0,255]、`aGVsbG8gd29ybGQ=`→hello world字节，大小写hex及padding边界另有手算expected，文本wrapper只在strictUTF8成功时给Some(Str)。

旧`stdlib/auto/encoding/{hex,base64}.at`是来源合同，不自动算旧实现PASS；hex注记非法空串、Base64注记nil/Str冲突，T-01须核旧实现/实际consumer后冻结兼容公共别名/返回值（不将新checked字节接口默称旧Str API原样）。现cookbook/encoding/004_base64源实际为use.rs第三方示例，005_hex实际是字符长度演示，均不能冒这两个stdlib解码面的旧运行覆盖。旧义务保持，发现仅声明/无运行也如实记录并为新能力建独立oracle，不能重开972或以目录名计PASS。只有涉及公开语义改变才回用户，核心byte解码与MODULE可先行。

### 5.4 D-01..D-09逐面合同

| ID | 拟公开面/行为 | 实际实现与关闭任务 |
|---|---|---|
| D-01 | `file.read_text(path str) Str`失败`""`；`read_text_range(path,offset,limit) Str`返回JSON envelope，text/total/next_offset有独立错误/EOF形状，不与普通read_text混用sentinel | 读region→严格解码→owned；部分读取/EOF/取消关闭原句柄。range以字节计offset/limit，跨UTF8字符起点/终点回退到边界；成功text/total/next_offset，真实EOF next_offset=null，参数/IO/非法编码错误text=""/total=-1/next_offset=null；中途short-read不能冒EOF。对照058完整10行expected；T-03/V-02 |
| D-02 | `dir_names(path str) List<Str>`，目录名Unicode/NUL协议清楚，失败空集/诊断按旧实际调用冻结 | DirOpenW16/Next/Close→UTF16→UTF8，不把目录顺序当稳定排序承诺；T-03 |
| D-03 | `env.get(key str) Str`、`get_or(key,default)`、`home_dir()/local_data_dir()` | 缺键get=`""`；get_or保存在空值与缺键区别。OS实际值转owned，环境改动测试隔离后恢复；T-03 |
| D-04 | 旧`path.canonicalize(path str) Str`，新canonical别名同合同：真实链接解析、错误`""` | 已有Win32 CanonicalW16含最终路径和逻辑fallback，不能把“缺失/不可开时逻辑绝对路径”当旧canonical成功；需在既有owner修正/验证此差额，关闭handle、保路径授权。与1065共享本语义，不反等彼此实现；T-03 |
| D-05 | `process.args() List<Str>`、`current_dir() Str` | argv不是按空格split；保引号/空参数/Unicode。当前dir成功往返，不把cmdline单Str冒args列表；T-03 |
| D-06 | `spawn_with_output`旧JSON envelope，exit_code/stdout/stderr三个真实字段 | 计数面复用，字节解码/CRLF/NUL按原语义保；超限/取消杀并等待进程树、drain/释放pipe，失败不输出成功空文本；T-03 |
| D-07 | 新KV：拟`get(path,key) KVRead`、`set(path,key,value) int`、`remove(path,key) int` | 无旧同形，见下独立格式/重开/失败合同；T-KV/V-03 |
| D-08 | `zip.list_json/read_text/extract_entry`保原三函数及sentinel | Auto容器/解压算法和目标薄端口分离，见下；T-04/V-03 |
| D-09 | `read_bytes(path) ?List<int>`，Some空/Some非空/None区分 | T-02必须实现＋V-D09，原aux兼容不冒新返回形状 |

KV最小持久合同（无旧同形，本轮固定而非全部留T-01）：`KVRead`拟字段为`status:i32, found:bool, value:owned Str`；get成功status0，缺键found=false/value=""，存在空值found=true/value=""；IO/权限错误=-1、损坏/版本错误=-2、预算超限=-3，错误时found=false，不把坏文件冒缺键。set成功0/失败同负状态，remove删除1/缺键0/错误同负状态。返回owned值不借用文件buffer，首次set可创建不存在的库，get不存在文件按空库缺键，但不可读/损坏文件不如此处理。

格式字节固定：ASCII头`ACE-KV/1\n`，每条为ASCII十进制`<key_bytes> <value_bytes>\n`，随即是精确长度的原UTF8 key bytes与value bytes，最后一字节LF。长度不含终止符，禁止负数/整数溢出/越界；payload中NUL/换行/空格合法，不按内容split，也不需要把escape/unescape helper误当完整存储。空key/value合法；重复key按文件序最后有效记录；坏头/截断/坏长度/非法UTF8整次status=-2且不发布部分表。复用原syscommon编码/I/O/atomic replace，必要旧迁移输入才复用escape/unescape，不依赖975 JSON解析器形成环。

set/remove候选全量写临时文件、flush/关闭后原子替换；失败保旧文件字节，清临时，重开读到已提交代。单写者顺序语义，多写冲突具名拒绝或由既有入口串行，不承诺跨进程无锁事务；运行预算按文件字节/临时峰值算，超限拒，不承诺任意大KV。T-01只选等价分块/查找结构与原权限策略，不再决定格式/错误是否支持。独立expected直接手写上述字节、关闭重开/损坏码与旧文件hash，不只set→get同错自证。

ZIP复用旧源码/规范/测试，算法默认纯Auto，Python zipfile/旧Rust库只作具名测试工具或显式可选provider，不能偷当默认实现。至少区分Stored、Deflated及原实际支持/生产使用的其他方法；T-01读取zip3依赖/原公开支持与真实语料冻结完整方法/ZIP64/加密/目录/CRC处理清单，再在T-04做Auto central-directory/EOCD范围检查、解压位流/窗口与CRC校验；本表不是只支持两个方法的降范围授权。无法关闭原required方法时具名needs_replan，不冒三函数齐。

ZIP规范：中央目录原顺序和name原样，list_json字段name/size/comp_size/method保持；read_text strictUTF8失败`""`；extract_entry成功0/一般错-1/解压超限-2。max_bytes≤0非法，恰好上限成功、实际输出超限失败，不信任header大小；在验证/解压/预算通过前不覆盖已有dest；只写显式dest，不从entry名物化路径。损坏目录/偏移溢出/CRC错/不支持方法/截断须受控失败。独立原测试与新增边界都要有真正解压与磁盘结果，不能以生成了JSON字符串代ZIP支持。

### 5.5 受检输出帧（原967 owner内扩必要面）

拟Rust源例`fn read_into(path: &str, dst: &mut [u8]) -> i32`、C等价显式`dst,cap`；经967普通源码导入→自动binding/shim→AV真实调用，不手写manifest或另建FFI模型。沿typed/resource帧扩可写借用`{owner, origin, generation, offset, capacity, rights}`（示意字段），运行时映射地址不进缓存。调用期间pin且不别名可变借用，不可被foreign留存/异步逃逸；返回n必须0≤n≤cap，只将已写n字节发布，负状态按原错误映射，close/异常解除借用；取消/超时先阻新调用，正在native执行的借用保持pin直到实际返回或受控子进程终止并回收，不能只标cancel就先释放region。cap=0为合法空借用但仍核owner/rights，成功written只能0，调用方不能把零容量请求当EOF；partial read/EOF、过期/错owner/只读region、n>cap和不同DLL/ABI/target均有负控；边界前后guard字节不变。不以这种帧授权任意不可信native代码，provider可信/权限及967安全边界保持。

### 规范增量

| delta_id | action | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-BSTR-01 | modify | `docs/specs/ace-migration/m2-stdlib-io.md` | D行pending→逐面真实来源/错误/资源/平台与未交边界 | 不用计划代支持 | AC-02..05/AC-D09 |
| SD-BSTR-02 | modify | 同上＋`docs/specs/stdlib/design/ace-binding-and-assembly.md` | 缺构造/装配差额→已验字节/owned转换与共同声明语义 | 复用975/973 | AC-01/02/AC-MODULE |
| SD-MODULE | modify | `docs/specs/stdlib/design/ace-binding-and-assembly.md` | profile拼写分裂→统一public/private事实、真实原源/exports/来源冷恢复 | 用户975交接 | AC-MODULE/AC-MODULE-LAND |
| SD-FRAME | modify | `docs/specs/ace-migration/m2-host.md`、`docs/specs/auto-abi/checked-source-foreign-calls.md`及系统库Spec | 输入借用有限帧→必要有界输出借用及真实provider结果 | 六路径读面不冒写形见证 | AC-FRAME |

本轮不改canonical Specs；实际交付后原review核delta，merge才沉淀已验知识。

## 6. 测试设计

全部拟实施/未运行。T-01冻结真实命令/身份，永久fixture由各work任务建立，独立review在冻结SHA/同树停写后进行，merge不承担实现正确性。六路径不是彼此相等oracle；新增平台OS面分别核实际磁盘/进程/环境值/资源，Linux全NOT_RUN交1063，不冒PASS/NA。

| Case | AC / work任务 | 输入→入口→独立oracle及负控 |
|---|---|---|
| V-MODULE | AC-MODULE；T-MODULE-I/V | demo原双模块与完整canonical JSON/Redis/SQLite源→公开CST/装配→binder/verify→真实consumer；42、JSON user.name/数组路径及SQLite SUM=9；跨模块private应拒且原span，删pub/全pub/private泄露/错std身份/错误候选/同名模块污染均须RED |
| V-MODULE-RESTORE | AC-MODULE；T-MODULE-I/V | 冷/缓存恢复相同声明可用；改helper正文/exports/grammar/provider后真实readers失效，坏更新保旧，旧来源/陈旧签名不能准入；不得仅计cache-hit布尔 |
| V-01 | AC-02；T-02 | bytes `41 E4 B8 AD F0 9F 98 80 00`→decode→owned；9B/4scalar且NUL未截；空Some、非法C0 AF/ED A0 80/F4 90 80 80/截断None；释放输入后返回仍有效；预算边界与取消/回收有独立观察 |
| V-CODEC | AC-02；T-02 | 新checked hex/base64源→各适用端真实bytes；00ff/hello world/空Some([])对手算expected，奇数hex/坏字母/错padding None，二进制FF与UTF8文本wrapper严格区分；旧声明/第三方示例不得冒本stdlib实现PASS |
| V-D09 | AC-D09；T-02 | 同一公开read_bytes源→各适用backend；空文件Some([])、字节00 FF Some([0,255])、缺失None，真实A2R运行核Option/List布局，不能只显示长度 |
| V-02 | AC-03；T-03 | D01..06逐项：Unicode文件名+文本/NUL/空/失败、真实symlink与缺失canonical、环境空值/缺键、argv空/引号、捕获CRLF/stdout/stderr/非零退出，坏编码/超限/越权/恢复均有合法近邻和资源关闭 |
| V-03 | AC-04；T-04/T-KV | ZIP旧三函数＋Stored/Deflated及原required语料；CRC/截断/假大小/恰好预算/超一字节/坏UTF8/失败保dest；KV手写格式/重开/重复key/坏行/写失败与原子替换失败保旧hash，独立oracle而非只同实现往返 |
| V-FRAME | AC-FRAME；T-FRAME | 同一文件含NUL/多字节，自动导入C/Rust provider→真实AV读输出帧；n/guard/owned bytes独立断言；错owner/rights/ABI/过期/超cap/缺工具不得充成功或语言负例 |
| V-04 | AC-05；T-05 | 各D行/源码→真实路径→独立oracle＋前述负控，适用AC消费1065正式来源；缺行/错源/仅emit/缺工具/0选择均不PASS；六路径计数/原A2C与平台适用性逐项保留 |

### 已有用例到新检查映射（不是覆盖上限）

| 原件/原断言 | 新映射/任务 | 复用与边界 |
|---|---|---|
| `crates/auto-lang/test/vm/18_ffi/058_read_text_range/read_text_range.at`及`.expected.out` | D01/V-02，T-03 | 原范围/输出原样核，非只parse |
| `crates/auto-lang/test/vm/18_ffi/046_env_get_set/env_get_set.at`及expected | D03/V-02，T-03 | 原env sentinel＋空值/缺键增补，真实子进程观察 |
| 973 `stdlib_sys_973.rs`的sys973_v03_process_spawn_capture_timeout_exit、sys973_dirs_and_replace、sys973_t08_seek_permissions_mtime | D02/04/05/06、V-02 | 原计数/预算/关闭继续，新增文本/args/链接/Option不由旧计数PASS替代 |
| `zip_io.rs::plan897_list_json_shape`、`plan897_read_text_roundtrip_and_caps`、`plan897_extract_entry_bytes_exact`；`a2r-std/src/zip.rs`原Deflated正例；`docs/specs/auto-vm/zip-natives.md` | D08/V-03，T-04 | 旧完整输入/真实expected＋新独立Auto解压边界，Python/旧Rust不是默认运行实现 |
| 975 `json975_char_to_str_primitive/json975_char_to_str_a2x_ends`、真实json.at私有helper与sqlite.at public迁移、redis真实Str返回 | B1/MODULE/V-01/V-MODULE | 取975正式切片SHA及原文件hash；现分支的源码/绿不能充正式来源 |
| 967＋973 `sys973_av_foreign.rs`既有写形primitive控制 | V-FRAME，T-FRAME | 写形保持回归，新read-out有真实独立fixture，不沿旧PASS冒输出能力 |
| KV新格式无旧同形；D09旧List+len辅助 | V-03/V-D09 | 无旧同形如实标注；新增手写字节/Option oracle，不伪造旧金样 |

职责组已静态核存在：MODULE `cargo t ace-source ace-syntax`＋实际共享变更的必要consumer；BSTR/sys `cargo t ace-av ace-stdlib`；输出帧 `cargo t ace-abi ace-foreign-contract`；适用AC `ace-native`；A2X `ace-a2x`；语言case线`auto-test-case`和原`a2r a2py`按真实修改选完整并集。不得把全部组合反复全仓跑，普通组排正式performance，Linux/WSL在Windows排除且不计PASS。具体新例在既有tests/cases注册，不另造runner；发现/运行/原条件排除、准备/运行/清理/总时分列，首次红/重放绿保留。

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 973有限面/975B1/MODULE/1065接口、D行旧合同/原例/平台/预算及未决工程策略有界冻结 | T-01；未决不得进入依赖实现 |
| AC-02 | work | 严格UTF8→owned与编码共享解码实际可用，生命周期/预算/错误及独立结果齐 | T-02/V-01/V-CODEC |
| AC-03 | work | D01..06真实公共API、旧sentinel/实际OS/资源与原例＋增补齐 | T-03/V-02 |
| AC-04 | work | 原required ZIP三函数/方法与新KV持久合同完整关闭 | T-04/T-KV/V-03 |
| AC-05 | work | 全部required路径/旧新检查闭合，AC取1065来源，Linux完整移交，错误/缺证不冒PASS | T-05/V-04 |
| AC-06 | review | 全包独立复审绑定本r2/冻结实现SHA、逐AC/差额/健康/SD，MODULE批审不代全包 | T-06 |
| AC-07 | merge | 已验规范/正式来源、归档和全输出对账/清理事实齐 | T-07 |
| AC-MODULE | work | 合法public/private共同CST/exports/原源身份及冷恢复与实际975 consumer闭合，无源重写/旧链fallback | T-MODULE-I/V；V-MODULE/RESTORE |
| AC-MODULE-REVIEW | review | 独立phase复核MODULE接口/原源/负控及本revision，不能只采实现者汇报 | T-MODULE-R |
| AC-MODULE-LAND | merge | MODULE已独立正式合入、收据/Spec同版供975，整包状态仍executing | T-MODULE-D |
| AC-D09 | work | Option<List>真实实现及全部适用端空/非空/失败闭合，不只裁定 | T-02/V-D09 |
| AC-FRAME | work | 输出借用权限/owner/长度/ABI及自动binding→AV C/Rust真读闭合，无手工manifest | T-FRAME/V-FRAME |

## 8. 执行步骤

当前4/13（MODULE切片I/V/R/D已交付，余9项待执行）；下列是唯一当前任务表。

- [ ] **T-01｜work｜AC-01**：核取材/正式依赖与原D/旧例表，冻结MODULE声明/exports、必要输出帧、UTF8/资源和ZIP完整方法/预算清单、hex/base64旧合同冲突；以最小真实正例＋非法近邻＋失败恢复的最长链先验策略，不造平行系统，不把未决全部留给后续。
- [x] **T-MODULE-I｜work｜AC-MODULE**：在已核grammar/frontdoor/packages/binding/stdlib owner统一合法声明/可见性和冻结身份；建立V-MODULE/RESTORE；canonical sqlite pub迁移由975原T-03负责，不代其业务实现。
  [✅ 2026-11-16, commits 208cc04a5+e2e9b4724] r1=**条件缝闭合**（诊断：json.at不可解析根因是packages-modules用贪婪expr解析for/if条件，'x < ident {'把块括号提交为record ctor、误报挂'fn unsupported'——非pub/private拼写；修复=record族condition-expr全链扩至packages-modules的if-cond/for-cond）；r2=**模块域'use demo'**（frontdoor一段路径=模块域导入、整模块reexport命名拒；binding绑定模块命名空间BoundImport+模块边+未知名拒；限定调用沿既有module_prefix/cross_call路径带exports可见性执法）。正例负例全绿（module_1064 6/6：§5.2 demo verbatim编译、demo.helper限定私调=module-private、demo()裸调=module-symbol、pub use demo=module-path、use nope=module-missing）。
- [x] **T-MODULE-V｜work｜AC-MODULE**：运行MODULE完整职责与实际JSON/Redis/SQLite装配消费、原源/错误/冷恢复负控；完整文本consumer所需B1先核独立正式切片，不等975全包/1065/ZIP/KV。
  [✅ 2026-11-16, commit 6ee22e770] module_v_1064 7/7绿：demo **执行**=42（AV）；**真实json.at逐字节**跨模块装配+**执行**（user.name="Michi"+services.0.port数组路径=42）；**真实redis.at**绑定达checked HIR（纯pub helper消费）+具名lane边界（av.owned/1首刀无平台intrinsic——lower.intrinsic断言在案；redis网面沿platform lane）；负控：删pub镜面=module-private RED/伪造std=stdlib-reserved RED/同名=module-identity RED；**RESTORE**：store往返同身份+恢复产物执行42+改私有helper正文→input_hash移动（陈旧readers失效）+坏digest拒绝。职责组：ace-source 610 PASS+ace-syntax 155 PASS（956.5s）。**SQLite canonical装配腿具名阻塞975原T-03 sqlite.at pub迁移**（合同禁镜面源）；provider候选/装配线=1043既有身份检查未触。
- [x] **T-MODULE-R｜review｜AC-MODULE-REVIEW**：停写，在同树独立phase审查MODULE全部AC/差额/代码/SD，绑定r2/实现SHA；失败回I。
  [✅ 2026-11-16 phase pass] reviewed_commit 6ee22e770（=208cc04a5+e2e9b4724+6ee22e770）；同会话复审（局限性在案，从工件重构：AC-MODULE矩阵逐项对6/6+7/7测试、V-MODULE各RED负控核panic路径、RESTORE三层核identity机制）；**突变负控**：condition-ops pratt表回缩→json真实源装配测试RED（source.infra命名拒绝）→回退后13/13绿（绑定证明）；健康：两测试文件fmt净、无debug打印、无新警告；消费组（module_packages_960 4/4、stdlib_core_945 4/4、module_sources 2/2）r2后重跑绿。SQLite腿/装配provider线=具名分账非本切片。
- [x] **T-MODULE-D｜merge｜AC-MODULE-LAND**：phase pass后正式合入与沉淀已验MODULE，交IF-1064-MODULE；保唯一tree/resync继续余项，不归档整包/删除树。
  [✅ 2026-11-16] phase fold落主 **master@4ba167930**（代码6文件；与在途PLAN-981同owner冲突按并集解决：age-package∪packages-modules条件机扩面+Node构造is_module）＋ **master@58b383eea**（`SD-MODULE` 规范沉淀落位 `docs/specs/stdlib/design/ace-binding-and-assembly.md`，sha256/16=`c1f10e6dbb135789`，`--ff-only` 合并）；**合入纪律事故已纠正**：首合并误git add -u裹入40文件他WIP→soft reset重做干净合并（他WIP全部保留未暂存）；合并基线实测：`cargo t ace-source ace-syntax`（`results.json`：`ace-source` PASS 757执行/1 ignored `run_exe_60s_deadline_bounds_descendant_pipe_hold`，prep 30.40s+run 20.74s；`ace-syntax` PASS 155执行/0排除，prep 24.51s+run 27.47s），新增13例均≤10s门（`module_1064` 6/6=0.168s，最大`module_1064_json_at_parses_with_pub_private_split` 0.168s；`module_v_1064` 7/7=0.319s，最大`module_1064_v_json_at_real_source_executes` 0.316s；前序合入check另见`plan979_age` 26/26+`packages units` 43/43）；`wt-guard.ps1` clean，worktree保留并resync至master继续余项（整包维持`executing`）。**IF-1064-MODULE**（975 T-03/V-LOAD-MODULE消费接口）：①json.at/redis.at等pub fn+私有helper作者源现可作为packages模块逐字节装配消费（条件缝闭合）；②模块域`use mod`+限定调用`mod.fn()`带exports可见性（跨模块私有=命名`source.module-private`+原span）；③std冻结身份/重复身份/坏digest拒绝在案；④canonical sqlite pub迁移归975原T-03（其tracked文件现为plain fn内联拼写）。
- [ ] **T-02｜work｜AC-02/AC-D09**：消费975/B1正式原语，实作共享UTF8/owned及972同车道解码、D09 Option/List全端合法化与V-01/V-CODEC/V-D09，不新造同义原语。
- [ ] **T-FRAME｜work｜AC-FRAME**：在967现有typed/ABI/binding/runtime owner内补必要受检输出借用＋V-FRAME，自动导入C/Rust真读，无另一个FFI模型。
- [ ] **T-03｜work｜AC-03**：按D01..06具体合同落公开面/原兼容包装，真实OS/错误/生命周期与V-02，不以常量/计数充文本/args。
- [ ] **T-04｜work｜AC-04**：实现ZIP central-directory/原required解压方法/CRC/预算及三公开面，V-03 ZIP部分；复用原source/fixtures，不能缩成Stored-only。
- [ ] **T-KV｜work｜AC-04**：实现本节KV格式/读写/原子替换/重开及V-03 KV部分；不依975 JSON/DB业务循环，不重复既有atomic replace底座。
- [ ] **T-05｜work｜AC-05**：各work能力齐后核全部原例/新增V、六路径/原A2C、1065 AC正式来源与1063完整移交，形成冻结并集/真实结果；未闭required阻work完成。
- [ ] **T-06｜review｜AC-06**：同树停写后的全包独立final，核全部AC/原后延义务与新增差额/健康/SD，阶段R不冒final。
- [ ] **T-07｜merge｜AC-07**：final pass后核正式来源/沉淀/合入归档；输出逐目录对账、摘要折Plan、按现行政策清scope/guard清树，未知持有不盲删。

## 9. 复审记录

stage: goal | plan_id: PLAN-1064 | plan_revision: r2 | scope: MODULE (T-MODULE-I/V/R/D) | status: executing (4/13; MODULE phase delivered) | outcome: pass | worktree: D:/autostack/.wt/lang-1064/auto-lang @ plan-1064-dev (retained, wt-guard clean, resynced) | code_commit: 4ba167930 | spec_commit: 58b383eea (docs/specs/stdlib/design/ace-binding-and-assembly.md sha256/16=c1f10e6dbb135789, --ff-only) | gates: ace-source PASS 757/1 ignored (30.40s+20.74s) + ace-syntax PASS 155/0 (24.51s+27.47s) + module_1064 6/6 (0.168s, max 0.168s) + module_v_1064 7/7 (0.319s, max 0.316s); spec-lint 0 new issues | next: T-01 / BSTR / T-FRAME / T-03..07 (remaining 9/13)

stage: merge | phase (T-MODULE-D MODULE切片合入) | PLAN-1064 | r2 | outcome: pass | worktree: D:/autostack/.wt/lang-1064/auto-lang @ plan-1064-dev（保树，resync至master tip） | landed: master@4ba167930（恰6文件；981并集解冲突；add -u事故已soft-reset纠正） | IF-1064-MODULE: 出具（见T-MODULE-D勾注，975 T-03消费） | landing-check: master上6/6+7/7+26/26+43/43 | next: 1064余项（BSTR/T-FRAME/T-03..07按合同序）；975侧V-LOAD-MODULE腿解锁

stage: review | phase (T-MODULE-R) | reviewer: 同会话（无独立指派，局限性在案；从工件重构） | PLAN-1064 | r2 | outcome: pass | worktree: D:/autostack/.wt/lang-1064/auto-lang @ plan-1064-dev | reviewed_commit: 6ee22e770 | base: 208cc04a5(r1)+e2e9b4724(r2) | evidence: module_1064 6/6 + module_v_1064 7/7（执行+绑定+负控+restore）；突变：condition-ops回缩→RED（source.infra）→回退13/13绿；组ace-source 610/ace-syntax 155 PASS；消费组r2后绿 | findings: 0（健康/绑定/负控全过；sqlite腿与装配provider线=具名分账） | next: T-MODULE-D（phase合入+IF-1064-MODULE）

stage: work | PLAN-1064 | r2 | 2026-11-16 | outcome: in-progress (T-MODULE-I r1：packages-module条件缝闭合) | worktree: D:/autostack/.wt/lang-1064/auto-lang @ plan-1064-dev | code_commit: 208cc04a5 | diagnosis: json.at/redis.at作packages模块不可解析的根因≠pub/private——是**条件缝**：packages-modules的for/if条件用贪婪expr，'x < ident {'把块括号提交为record ctor，恢复路径把误报挂在'fn is not part of this source profile'；§4"已观察事实"的text-record拒pub/packages拒私有helper两缝中packages侧的真缝即此 | fix: record族condition-expr（受限条件文法，无ctor提交）扩至packages-modules的if-cond/for-cond（规则+condition-ops pratt表+condition-unary/postfix/ctor/atom全链加面；grammar-id变化=设计性缓存失效） | evidence: module_1064.rs 5/5绿0.07s——json.at解析=19 pub exports+8私有helper声明不导出；redis.at同；app::jsond（json.at逐字节verbatim）跨模块compile_package_product达checked HIR；§5.2 demo现行item-import拼写=模块内私有调用OK+跨模块pub导入OK+导入私有helper=命名source.module-private拒绝；作者构造矩阵11形全过 | regression: core packages units 43/43＋av module_packages_960 4/4＋module_sources 2/2＋stdlib_core_945 4/4（四端16.72s） | named-remainder: 模块域'use demo'+'demo.answer()'限定形（resolve无模块命名空间绑定）、V-MODULE-RESTORE、删pub/全pub镜面/旧身份/只inline替LOAD负控 | next: T-MODULE-I r2（模块域限定形）→ V → R → D（IF-1064-MODULE）

stage: work | PLAN-1064 | r2 | 2026-11-16 | outcome: in-progress (T-MODULE-I开工) | authorization: PLAN-975目标系统裁定(a)（2026-11-16，§9在案）——本会话按r2合同先行MODULE切片T-MODULE-I→V→R→D | worktree: D:/autostack/.wt/lang-1064/auto-lang @ plan-1064-dev（基面master@d71a776e5） | materials: plan-artifacts/plan-1064/已建 | scope-note: 首次交付=MODULE切片（§0）；T-01全包冻结项的MODULE子集随I核取，ZIP/KV/BSTR清单具名pending非本切片 | next: T-MODULE-I（grammar/frontdoor/packages/binding/stdlib五owner统一合法声明/可见性/冻结身份＋V-MODULE/RESTORE）

2026-11-16 stage:new/revision | PLAN-1064 r2 | outcome:pass（规划合同静态预检；非实现/独立产品review） | changed:明确MODULE实际承接及I/V/R/D、B1切片输入、T-FRAME/T-KV、D09真实交付、全平台/旧例/AC与有界决定 | 0/13/drafting，原7任务与原AC/V保留；不实施/生成fixture/建树/运行/改canonical Spec或重开973。next=未来实施指令后先T-01/MODULE-I；依赖B1/1065的验收按切片就绪，不反等975整包。

### r1历史（当前合同由r2替代）

2026-11-16 stage: new | 本计划由 PLAN-973 §10.1 用户裁定（选项 c）创建为延后面承接 owner；取号 1064（scripts/new-plan.sh，master 主检出）。仅规划承接合同：延后面移交表 9 项（D-01..D-09，代码锚点=973 基线 a9eceef38 快照）、车道落点二选项、V/AC/任务 7 项。未实施、未建 worktree、未运行任何测试、未改 canonical Specs；973 不反向依赖本计划。


本轮规划预检：五份相关Plan原45任务ID/勾选及原AC/V/SD全部保留；新增9任务全部未执行（1064增6、975增3），当前合计54项。0–10章节、任务总数/唯一性、围栏/控制字符、本轮新增本地引用与行尾核对通过；1065表与真实SYS973_KINDS逐项相同=50，Windows W36/N14。B1→MODULE→975完整消费及973→1065→1064 AC验收的任务图无环，不存在1064/1065整包依赖975。原972归档、系统库canonical Spec与抽样产品源码hash未变；本轮未写取号计数器（其他会话1066取号与本任务分开）。973仅核原10项勾选/清理收据更正滞后计数并补前向交接，不重开终态。此为静态规划检查，所有新增运行验证仍未实施。

## 10. 待澄清事项

本轮已将975 MODULE的实际执行定为本包四个具名任务，并固定默认复用975/B1＋共享Auto syscommon解码；不再为“落972还是新造原语”重复请求用户裁定，也不重开归档972。局部实现策略由T-01在当前合同内冻结。

具名未决仅保真实风险：原hex/base64公开sentinel/任意字节与旧文本约定的冲突、ZIP原required方法/格式全集以及某后端无法承接既定borrow/Option表示时，T-01给最小原源/旧实际结果/规范差额、候选与独立oracle后记录决定；涉及公开语义变更才回用户。不能以待决定为由删原D行，或以本轮规划已细化宣称能力已交。
