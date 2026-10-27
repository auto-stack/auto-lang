# PLAN-828 第三批 R3：C03 CLOSED，C04 余两处计量/收据缺口

2026-10-22；`review_scope: phase`（第三批T-07/T-08/T-09受影响面，非T-10）。**needs_fix；仅B3-C04/P2保持OPEN**，含握手失败报告和存储字节重复计量两处。B3-C03/P1本轮关闭，C01/C02/C05原具体错误保持关闭。不是再要求重造执行观测或重开前两批。

## 基线与审核身份

- 同树`D:/autostack/.wt/lang-828/auto-lang`，`plan-828-dev@d5a4f3147e336095e0e9d9e6207b67aea1c51d1c`；base=`f4d7a9da6fb6b58a15a3149ee86583a7555daa86`。主检出交接9576b4dad5ceb8b201adedd46f9689f25c2dd5f9（包含用户所列簿记）。
- 入口/结束实现clean、HEAD与依赖未变；baseline/final-state的520项源码/配置/Specs/Design43/冻结主Plan接口/技能hash一致。auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8未消费。三交付文档及Spec delta冻结在本目录，hash见document-inputs/evidence-index。
- 本Codex审核上下文未实施、未启代理，不编造未暴露身份。审核源码投影/控制和收据只写主检出附件或自己的临时目录；没有改WT源码、canonical Specs/ledger、合入或执行整档。

## C03 关闭证据

执行器的ProgramSnapshot四入口采集实际传入程序的地址/代际/内容摘要与执行序；session从同次返回的out/trace生成last_run_publication，worker值/身份同源；resident由仍存活的当前驻留对象比较得出；父侧目录、resident、执行序与内容身份都进入正式总门。原错误的reuse_hit全局键也已改成生产者per-fn键。

| 受影响路径 / 原反例 | 本轮独立结果 |
|---|---|
| 原judge六控 | 6/6，stale/install/copy/whole/wrongvalue各自红、clean绿 |
| run内唯一真实AV调用改为深副本，其他函数体/新观测未动 | consumer-copy 2/2，副本结果仍正确但resident/identity使正式总门红，原run绿 |
| 官方worker合法`7→7 `当前内容命中 / omitmain / copyafterrun | identity 3/3：合法hit绿、缺目录红、冗余副本执行seq不配对红 |
| 真实8→9→8回访 / 仅回访时实际复制执行 | hit 2/2；clean绿，copyhit发布旧record的值/序号被拒 |
| 四个Program执行入口与旧pin | trace控制1/1：普通、units、steps/resume的目录/指针/代际/内容对应各自快照；seq递进；新结果10与旧pin结果9分别保持；未改main的完整证据不变 |
| AV/core原行为 | 22/22，未改三入口/旧调用链核心语义 |

consumer控制只机械适配新run的唯一`let(out,trace)=execute_program(...)`针点，替换为真实copy执行返回(out,trace)；值、trace及resident计算原样，不手填布尔字段。目录/执行序反控沿用冻结生产worker循环。记录原件/投影hash与全部源，见consumer-source/adjacent-source/worker-*.json。

旧copyhit控制的“发布值仍正确”前提，因本次值/身份同源修复不再成立：副本在旁路执行，但publication取上一run记录，故值和seq共同使总门红。控制保留实际copy mutation及“同一正式门必须红”oracle，新增核对seq/structural红，不将其已不可达的混源前提当产品失败。trace控制初版审核装配误用steps/resume签名，已依真实API机械更正，初始编译日志保留，不计实现finding。

因此C03原例、合法兄弟出口和实际消费不变量均有区分证据，可以关闭。边界仍是受支持ProgramSnapshot入口的明确故障区分力，不声称抵御恶意worker将值/身份/序号/resident全部一致伪造；也不因此追加密码学协议。

## B3-C04/P2 残项A：ready前EOF/坏JSON仍丢失报告

**归属T-08/T-09、AC-07、V-08/09、原§5.5全部raw/失败留存。原finding的失败出口残项，不是新增场景性能承诺。** 本轮work明确声称“中止握手也出报告”。

formal_worker.rs的Init发送仍以`?`传播错误，随后仍是`match link.recv()?`。新增`other => failures...`仅捕获已成功反序列化的错误消息类型；EOF/JSON解析错误在进入match前就Err返回。正式战役的`run_formal_campaign(...).expect(...)`因此在write_evidence前退出。错值运行先落盘已修好，但不能代表握手中断已修好。

`handshake-controls.rs`经真实父侧judge调用生产循环投影：真实warmup/初次run完成后，仅在ready发送点注入故障。无虚构成功回包、无store/目录权限干扰。

| 故障或合法对照 | 预期 | 实际 |
|---|---|---|
| ready前EOF | 红report，0样本，带输入/worker身份与握手失败原因，可落盘 | 直接Err("worker closed the protocol stream")，没有可保存report |
| ready前坏JSON | 同上，保留坏回包原因 | 直接Err("bad reply `{broken`...")，没有可保存report |
| 合法JSON但意外Closed类型 | 失败report可落盘 | 通过：0样本、失败原因及缺样本可落盘 |
| 第二编辑坏JSON，第一编辑已正常 | 保留第一样本+失败report | 通过：1样本、原错误及missing 1/2可落盘 |

handshake-controls.log为2过2败、退出101；两个可落盘正控报告已复制到handshake-positive-*，不只留在WT target。源与生成器/hash见handshake-source.json。已确认根因是握手Result被`?`提早传播，不是笼统推测。

**关闭oracle**：将握手阶段可控发送/接收/解析失败汇入同一失败收据路径，保留0样本、init/输入/二进制身份及原因，正式调用方能在判决/退出前保存；上述两红例转绿，类型异常和编辑中断正控保持。也核同类Init发送失败处理，避免仅挪一个match分支；不要求无法启动进程时伪造一个成功会话。

## B3-C04/P2 残项B：新IO观测将HIR载荷多算一次

**归属T-07/T-08观测、AC-07/08及§5.5真实IO计量。新观测修复中的计量回归，沿用C04。** 不影响此次程序执行结果或实际store行的正确性，但当前证据数值失真。

session.rs:1607在调用A07 put前加`hbytes.len()`，1623又在Ok分支再加同一个长度。一次真实新HIR行成功写入却统计两次。其余Conflict/修复/错误出口的“写入载荷”口径也须明确；本反例故意排除这些歧义。

`generate-io-control.py`包含完整冻结session源码（仅doc注释转换，函数体tokens未改），在crate::cache::put_enveloped外围挂一个旁路记录器；每次仍只委托一次真实put，记录layer、原payload长度及成功状态。fresh store预热后清空审核记录，再helper7→8：四次put全部Ok，**没有Conflict**：

| 实际调用 | 逻辑payload字节 |
|---|---:|
| A09 ABC | 173 |
| A07 HIR | 252 |
| A09 sidecar | 72 |
| A09 manifest | 406 |
| 一次一份合计 | **903** |
| session.phase_io[store_write_bytes] | **1155** |

多出的**252恰等于HIR载荷**。io-controls.log为1过1败、退出101；无store真实编辑的结构零盘IO正控通过。此oracle比的是同一轮逻辑载荷字节，不混入envelope/SQLite/WAL物理开销，也不要求新增物理磁盘监视器。

**关闭oracle**：每份真实提交/成功载荷按所声明口径只计一次，明确attempted/accepted、幂等/Conflict/修复/失败含义；fresh四个Ok的反例必须903，不能靠改名字保留重复加数。无store正控保持；相关失败出口不丢已发生计量。无需改存储身份/回收策略或新增预算阈值。

## C04 已实际关闭的部分

- 正式永久测试现已先write_evidence再assert门；原30轮wrongvalue控制转绿，失败report/raw确已保存（本目录formal-failed-*），不是只调用写文件helper。
- 正常编辑后的错误类型/流中断保留先前样本；上表正控已验证。
- phase_walls真实挂在syntax/project/frontend组合/verify/lower/emit/decode/load/link等生产入口，本次正式miss样本含11项；无store=false口径对应仅21B rescan观测。接受其按真实组合入口计frontend_body，不要求捏造独立计时；µs分辨率下个别0不等同缺事件。存储计量重复仍需修。
- 计时终点仍在父侧检查后，init单列；输入/worker身份和全成功样本保留。

## 定向验证与证据边界

| 面 | 本次结果 |
|---|---|
| 官方T08（永久组件+空白命中e2e+正式战役） | 5/5；成功report/raw在formal-official |
| 正式30样本 | P50/P95/P99=401.0208/473.3761/490.3966ms；init1276.4192ms；全值正确、现有门绿 |
| 原798双线预算硬断言 | 通过；日志A=201/B=0（整数MiB），worker峰值209436672B、父侧22429696B；不因B=0改变既定归因口径 |
| 原六judge / consumer / identity / hit | 6/6、2/2、3/3、2/2 |
| core / 新四入口trace | 22/22、1/1 |
| permanent mig/t07/b2/b1 | 9/9、8/8、14/14、11/11 |
| session/contract/t01r/r1 | 18/18（显式skip三512）、22/22、4/4、86/86 |
| 原root / 六轮恢复回访GC | 2/2、1/1；root始终3、采纳门保持 |
| close继承控制 | 96/96，include链与r1有重叠，不相加宣传唯一覆盖数 |
| 六改动Rust文件fmt / diff-check | 0/0；生产定向构建零告警 |

已知C04还未闭，本轮没有再追加512长负载，也未重跑798全套；work的321.5s不是本次独立结果。R2已测512只覆盖旧冻结，不能拿它给新执行器观测自动授予长负载PASS；最终修复冻结/T-10按实际变动确认必要重放。本轮root存储机制的未变部分由原控/六轮当前代码控制保持。最终整份AC09/10、SD05/06整体通过和merge-owned裸t+tv/必要consumer触面档、tf到期主检单实例/沉淀/归档/guard均pending。

SD01/原C05接缝更正保持；SD05/06、interface中“中止握手也出报告/真实IO”仍受C04限制，保持proposed，canonical未改。覆盖清单旧消费者表的_t08=4应更新为当前5，作为同批簿记校准，不另立阻塞finding。Plan r3/executing0/11保留。

## 复现和下一责任

重建冻结Cargo定向目标后，在本目录：

```powershell
python ./generate-worker-variants.py
powershell -NoProfile -File ./build-worker.ps1 -AllLibraries
python ./generate-handshake-workers.py
powershell -NoProfile -File ./build-handshake.ps1 -AllLibraries
powershell -NoProfile -File ./run-controls.ps1 -Source handshake-controls.rs -AllLibraries
python ./generate-io-control.py
powershell -NoProfile -File ./run-controls.ps1 -Source io-controls.rs -Filter measurement_ -AllLibraries
```

两个残项控制当前均退出101；脚本后续命令exit0不能覆盖该失败。Cargo库/exe/hash、源和变换、log均独立保留，不依赖实施者TEMP验证台。C03重放脚本也在本目录；原输入/oracle与机械适配说明已记录。

**只交work窄修C04这两处**，保留C03及其他已闭机制。小反例先修完并转绿，一批冻结再窄复审；无需重造数据流或再次整批架构调整。全部work-owned通过后才T-10整份最终，之后merge-owned门仍pending。
