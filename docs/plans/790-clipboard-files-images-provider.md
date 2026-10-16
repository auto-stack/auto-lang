---
plan_id: PLAN-790
status: execution_done
feature_name: 文件与图像剪贴板宿主能力依赖倒置
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-16
plan_revision: 1
current_step: 6
total_steps: 8
supersedes_spec_components: [docs/specs/auto-lang/vm/overview.md, docs/specs/auto-lang/ui/overview.md]
new_spec_components: []
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-lang/vm, auto-lang/ui]
depends_on: [PLAN-781, PLAN-785]
---

# [PLAN-790] 文件与图像剪贴板宿主能力依赖倒置

## 0. 变更摘要

将 native 2934..2937 的 files/image OS 实现依赖迁到宿主 provider，VM core 只保留列表/字符串/记录编解码和ownership。与已交付文本2925/2926统一装配优先关系，真实no-ui宿主能注入四函数。

## 1. 目标

消除四个shim对ui::clipboard_native的生产直引；原Windows OS行为、原生ID/签名、无provider时空列表/false/null降级保持。非目标：DnD、scroll/editor、term/media迁移，新的图像协议或任意格式转换，不更改既有文本provider合同。

## 2. 架构方案

沿779/785的per-VM显式provider→进程默认快照→兼容默认补空槽规则，provider只使用标准库owned数据（文件路径列表、图像path/width/height），不暴露VM堆句柄、UI类型或Win32 HANDLE。OS/PNG/DIB实现仍在ui::clipboard_native；新UI adapter仅委托。无feature也可注入不依赖UI的fake provider；无人注入时保持既有降级值。

## 3. 技术栈

既有host_capabilities、AutoVM/CompileSession宿主快照、StakeGuard、native_catalog、Windows剪贴板桥与命名互斥；独立Cargo consumer。

## 4. 需求分析与背景调查

取材基线 auto-lang@`3e65540cbe6ac019d210318dc93d17accf4ebcc4`；[输入指纹](attachments/extraction-batch-20261016/inputs.json) 记录现行 Specs、代码和 781/784/785/786 归档合同的 SHA256。overview 是导航快照，以相关模块 Spec 和实际源码为准。
用户 2026-10-16 授权“建立下一批实施计划”；本轮只建立合同及更新设计/路线，不实施 Rust、不创建执行树、不更改旧仓 refs、不发布/推送。之后按确认的合同进入 work；不能把本次建计划算作实施授权或完成证据。
781/784/785/786 均已归档；旧仓主分支和所有恢复载荷继续保留。787/788 为 AC/AutoDown 实验与集成线，不是本批全局前置，实施时按实际 diff 复核共用 manifest/数据类型热点。
native.rs 的 files_get/set 为 List<str>/Bool；image_get 为 {path,width,height}|null，image_set 接收 PNG路径。并非RGBA缓冲直接跨VM传递。2934..2937 当前在 windows+native-clipboard下直引UI，其他分支仍消费参数；文件列表有两种现有ListData形态。785已处理字符串编码/Clone和优先级；不能把其成功外推到列表/记录/临时文件寿命。
实际热点 native.rs、host_capabilities.rs、vm/engine.rs、session.rs、UI installer、CLI UI入口与785相同；785已归档，执行时不得与别的native族同时写这些文件。789/791可独立推进；792等待本批合流后冻结runtime。

## 5. 详细设计

T01 列明四ID的参数解码、错误/空值、非Windows降级与图像临时PNG生命周期；冻结既有Windows和非feature分支输出。provider接口名称由work选择，但不能用Result吞错把既有false/null改为异常。临时PNG仍由既有OS实现策略管理，返回路径在消费期可读；不能provider返回后立即删文件。
所有输入引用在成功、降级、provider错误和提前返回路径均配平；文件列表read锁不得持有到provider回调，返回 owned Vec/记录，跨VM不可带heap id。Unicode/空路径、既有支持列表类型、图片尺寸/路径编码按现行合同。
补空槽装配走785同一生命周期；显式provider永不被UI安装覆盖。provider可重入到不冲突的宿主调用，不得在默认注册锁内执行用户回调。非Windows不把UI默认伪造为可用OS。

### 规范增量

| ID | 操作 | target | before/after rule | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/overview.md | 增四函数provider、编码/缺席/ownership规则 | core去反向实现依赖 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-lang/ui/overview.md | OS adapter与宿主接线，更新未迁族清单 | 明确Win能力与后续边界 | AC-02/04 |

SD 素材（work 冻结稿，review 校验后由 merge 沉淀）：

**SD-01（vm/overview 增补节）**：`clipboard_files_get/files_set/image_get/image_set`（2934..2937，canonical `auto.clipboard.*`）供值经 per-VM `ClipboardMediaProvider`（trait：`files_get()->Vec<PathBuf>`、`files_set(&[PathBuf])->bool`、`image_get()->Option<ClipboardImage{path,width,height}>`、`image_set(&Path)->bool`；全标准库 owned 值域，零 VM 堆句柄/字符串索引/UI 类型/Win32 HANDLE）。装配=779/785 同构三级：实例注入（`AutoVM::set_clipboard_media_provider`）> 构造期进程默认快照（`AutoVM::new`）> 兼容默认 if-absent 补空槽（UI 宿主）；注入不 bump binding_revision/不撤 proof；Clone/merge 保 provider。缺席/失败语义=G3 值降级（空表/false/null，**非 VMError**——与文本族 2925/2926 的 feature-disabled 错误形构成族差异，冻结于 T-01 矩阵）：任何 provider 不得用 Result 吞错改异常、不得伪装成功写入。编解码契约：files_set 收两种 ListData 形（`<Value>` 仅 Str 元素、`<String>` 全收），read 锁不越 provider 回调；image_get 记录字段 path(lossy Str)/width/height(Int i32)；临时 PNG 由实现方落 temp 不即删、消费期可读。

**SD-02（ui/overview 增补节+未迁清单收窄）**：`NativeClipboardMediaProvider`（无状态）委托 `ui::clipboard_native` 既有 CF_HDROP/DIBV5+PNG 桥，挂 `windows × native-clipboard` 双门控（非 Windows 无 adapter——宿主入口不装配，VM 缺席 provider=G3 值降级，不把 UI 默认伪造为可用 OS）。宿主接线四点与 779/785 同集：`install_ui_host_capabilities`（run 家族/测试/调试/ABT/配置/REPL/解释器/back-proxy 漏斗+autovm_persistent）、`run_file_dynamic_ui_inner`、VmBridge 两构造、auto CLI main。未迁族清单更新：core→ui 直引剩余=code_editor（2919..2924）、dialog（2927/2928）、scroll controller（9918 族后 scroll 族）、dnd_start（2938）、term/media 等按 785 版清单减去 files/image 族。

## 6. 测试设计

实施、独立 final review、merge 分别交接，未指定外部模型/额外 agent。所有人使用本计划同一棵实施树；本轮只给拟定路径，不创建它。V 用例由 work 建成可执行 fixture；下表是合同而非已通过结果。
开发只运行受影响定向案例及必要 cargo check；先修完全部 findings、提交并完成 final review，merge 再在冻结提交执行裸 cargo t 和触面档。cargo tf 仅主检出批量到期回归，不能在每棵树运行。无相关 Rust/Schema 改动不跑 docs_gen；UI codegen 触面用 tu，不用改金样消红。

| V / owner | AC | 输入/入口 | oracle 与覆盖界限 |
|---|---|---|---|
| V01 / work | 01/02 | 新test/fixtures/modularity/clipboard-media-provider-consumer，真实源码→编译→VM四函数；可区分路径/宽高fake | no-default-features独立manifest实际输出；Unicode文件列表、png路径及记录字段值正确，不只直接调shim |
| V02 / work | 02/03 | 空/正常列表、两种既有ListData表示、字符串/图像record；重复调用后清栈+reap | 类型/值/缺席降级保持；heap/string引用回到基线，无所有权悬空和持续增长 |
| V03 / work | 03/04 | 两VM、Clone/merge、显式provider后运行UI安装、默认替换及旧VM快照 | 哨兵值证明不串线、不覆写显式值；provider失败/缺席仍消费参数且不死锁 |
| V04 / work | 02/04 | windows+native-clipboard真OS四函数；ui/no-ui开关、无OS/非Windows静态与可运行控制 | 在现有机器锁下OS循环读写；跳过不算OS验收；若本机不可用记录缺失并待真实环境补齐，不能只用fake宣称全部通过 |

拟实施树 D:/autostack/.wt/lang-790/auto-lang，branch codex/plan-790-dev。merge-owned 裸cargo t+tv，相关Windows OS定向档；tf仍批量单实例。未改视觉布局不要求重跑整画廊；改变UI交互时按autoui-verifier补对应双端验证。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 四个core shim无UI实现直引，no-ui实际VM外部注入V01通过，ID/签名不变 |
| AC-02 | work | 文件/图像数据与缺席值、临时路径行为兼容，V02/04通过 |
| AC-03 | work | 成功/降级/失败的引用和堆对象寿命配平，串线/优先级反控成立 |
| AC-04 | work | 生产宿主真实装配，OS桥保持可用；未迁能力清单明确，V03/04通过 |
| AC-05 | review | 独立重放真实VM/ownership/OS证据及SD，绑定冻结提交 |
| AC-06 | merge | 冻结门禁、规范/ledger/归档、guard清理完成 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 实施/产出/验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 781/785交付、合同确认 | 冻结native.rs/UI桥数据、降级、临时文件与ownership矩阵，标出全部宿主接线和OS环境 | 02/03/04 |

T-01 冻结矩阵（基线 master@e4ac72a62 勘察实录，native.rs:2123-2227 / ui/clipboard_native.rs / native_catalog.rs:86-89）：

| ID | 函数 | 参数/栈协议 | 现Windows+native-clipboard行为 | 现降级臂（非win/无feature） | provider化后统一体 |
|---|---|---|---|---|---|
| 2934 | `clipboard_files_get() -> List<str>` | 无参；压 heap id（ListData<Value>，Str元素） | `clipboard_native::clipboard_files_get()->Vec<String>`，逐个 `Value::Str`；Win32失败=空Vec→空表 | 空表（新 ListData + heap id） | `provider.files_get()->Vec<PathBuf>`；缺席/空→空表 |
| 2935 | `clipboard_files_set(paths)->Bool` | `pop_arg_i32` list id + StakeGuard | 解码两种 ListData（`<Value>` 仅Str元素／`<String>` 全收）→`Vec<PathBuf>`；**read锁限collect域，OS调用在锁外**；空输入=false | 弹参（`pop_arg_i32`），false | 同解码纪律；缺席→false |
| 2936 | `clipboard_image_get()->{path,width,height}\|null` | 无参；压 ObjectData record id 或 null NV | `clipboard_image_get()->Option<TempImage>`；字段 path=Str(lossy)/width,height=Int(i32)；**临时PNG落`%TEMP%`不删（消费期可读，provider返回后不得删）** | null | `provider.image_get()->Option<ClipboardImage>`；缺席→null |
| 2937 | `clipboard_image_set(png_path)->Bool` | `pop_string_arg`（owned String+StakeGuard.nv） | `clipboard_image_set(&Path)`；不可读/超64MP/Win32败=false | 弹参，false | `provider.image_set(&Path)`；缺席→false |

- ID/签名/错误面：catalog 注册无条件（native_catalog.rs:86-89 两 cfg 臂同名同ID）；短名 intrinsic 在 codegen.rs:694-712；缺席=**值降级**（空/false/null），与文本族 2925/2926 的 VMError 形**不同**——降级契约 G3 保持，非 raise。
- 宿主接线点（785 同集 4 处）：`install_ui_host_capabilities`(lib.rs:454)、`run_file_dynamic_ui_inner`(lib.rs:5594)、VmBridge 两构造(vm_bridge.rs:642/767)、auto CLI main(main.rs:971)。
- OS 环境：adapter 门控 `all(windows, feature="native-clipboard")`（`native-clipboard` 不蕴含 `ui`；非Windows不装=缺席降级，不伪造可用OS）；OS 验收共享 `GlobalClipboardTestLock`（命名互斥 `auto-lang-clipboard-tests`，与418/485族互斥）。
- ownership 纪律（全部路径配平）：成功/降级/缺席均恰好消费实参+压一返回值；files_set read 锁不越 provider 回调；image_set 经 pop_string_arg 物化 owned；跨VM边界零 heap id/VM索引（provider 值域=PathBuf/Vec<PathBuf>/ClipboardImage）。

| [x] | T-02 | work | T01 | host_capabilities及engine/session新增owned provider与快照；定向注册优先/Clone/实例隔离控制 | 01/03 |

T-02 证据：`ClipboardMediaProvider` trait（`files_get()->Vec<PathBuf>`/`files_set(&[PathBuf])->bool`/`image_get()->Option<ClipboardImage>`/`image_set(&Path)->bool`，全标准库 owned 值域+`ClipboardImage{path,width,height}` 元数据结构）+ 进程默认四 API（set/if_absent 同锁原子/get/clear）入 host_capabilities.rs（零 `ui::` 文本，785 锚纪律）；NativeInterface `clipboard_media_provider` 槽（Clone 保留/new 初始化/set+get 访问器）；engine.rs `new_inner` 构造期快照 + `AutoVM::set/clipboard_media_provider` 实例 API（781/785 同形）。session.rs 零改动（快照经 AutoVM::new，785 文件集同形）。定向控制：in-crate `process_default_snapshot_and_instance_isolation`（快照 A→换默认 B→实例覆盖 C 三层隔离哨兵）+ `identity_and_contract_rules_preserved`（注入零 revision bump/Clone 保 provider）绿。

| [x] | T-03 | work | T02 | 四shim统一能力调用与编解码；清除UI直引；补所有返回路径引用配平；cargo check+定向泄漏反控 | 01/02/03 |

T-03 证据：四 shim 单体 feature/平台中性体（8 臂 cfg → 4 统一体）：files_get=provider 缺席→`Vec::new()`→空表（同旧降级臂）；files_set=两种 ListData 解码（read 锁限 collect 域，provider 回调在锁外收 owned `Vec<PathBuf>`）+StakeGuard 配平；image_get=`and_then(image_get)` 锁外回调→ObjectData 记录/null；image_set=pop_string_arg 物化+StakeGuard.nv→`&Path`。native.rs 零 `ui::clipboard_native::` 直引（in-crate 永久锚 `core_media_shims_have_no_ui_clipboard_native_reference`，针含尾随 `::` 防注释误伤）。存量 plan485 OS 测试 `tests_clipboard_natives` 补实例级真实 adapter 注入（否则 provider 缺席静默走 headless guard 空转——已实测发现并修复）。`cargo check -p auto-lang/-p auto` 零新增警告；新增 no-ui+native-clipboard 组合可建（旧 Windows 臂要求 ui 在场，此组合迁移前不可编译——改善非回归）。

| [x] | T-04 | work | T03 | ui新adapter委托clipboard_native，统一installer/CLI等实际宿主接线；显式默认优先测试 | 04 |

T-04 证据：`ui/clipboard_media_provider.rs`（`NativeClipboardMediaProvider` 无状态委托 clipboard_native 四函数 + `install_process_default` if-absent）挂 `ui/mod.rs` 双门控 `windows × native-clipboard`（与被委托 Win32 桥同门控；非 Windows 无 adapter 不伪造可用 OS）。四宿主接线点与 785 同集：`install_ui_host_capabilities`(lib.rs)、`run_file_dynamic_ui_inner`(lib.rs)、VmBridge 两构造(vm_bridge.rs)、auto CLI main(crates/auto/main.rs)——各带同形 cfg。优先测试：adapter 内 `compat_install_never_overrides_explicit_default`（空槽补缺/显式不覆盖/哨兵路径可区分）+ fixture `v03_explicit_default_survives_entry_install` 两构建同绿。

| [x] | T-05 | work | T04 | 新独立consumer及V01..04；系统命名锁下OS验收，no-ui/feature矩阵及777检查 | 01–04 |

T-05 证据：
- 外部 consumer `test/fixtures/modularity/clipboard-media-provider-consumer`（独立空 workspace、default-features=false）：no-ui 形 8/8（V01 全管线四函数实值 stdout 可区分 Unicode 路径/640×480 三元组/set 精确 PathBuf/双实例不串线；V02 两种 ListData+空列表+相邻周期+空/None/失败值语义；V03 缺席=G3 值降级逐字面值 `0`/`false`/`none`+裸 VM 零装配锚+显式默认不被兼容安装覆盖+Clone/merge/ID 钉扎 2934..2937）；ui-host 形 10/10（同套+V04）。
- V04 真 OS（windows×ui-host）：命名互斥锁（`GlobalClipboardTestLock`）下 files set→get Unicode 即时往返 + image set→get（5×4 合成 PNG→DIBV5→临时 PNG IHDR 尺寸正确+消费期可读+清理）；`v04_ui_host_entry_installs_real_os_adapter` 空槽经 run 入口补缺真 adapter（读侧）；`v04_non_windows_does_not_fake_os_default`（cfg(not(windows)) 静态控制：槽保持 None+值降级）。一次性探针实证本机 OS 往返真实执行（"OS ROUNDTRIP REALLY EXECUTED"），headless guard 未空转。
- in-crate `plan790_clipboard_media_provider` 7 测试（core 剪断锚/降级+委托/两 ListData 形/身份规则/快照隔离/reap/merge 保 provider）。
- 回归：clipboard 族 30/30（含 plan785 6 锚+plan485 VM 面真 OS+adapter 优先测试）、plan779 4/4。
- feature 矩阵：no-ui（fixture 默认）绿；no-ui+native-clipboard 单开 `cargo check` 通过（新可建）；ui-host（ui-iced 超集）绿；日常默认面绿。
- 777 检查：`modularity-audit.py --mode strict` 判红=预存 `stale_baseline`（Cargo.toml×2 自基线 base_revision 变更——master 干净树同判红，非本批引入；本批零 manifest 依赖边新增，仅 exclude 列表追加）。基线重采样绑定干净树 HEAD，归 merge 侧处理。

| [x] | T-06 | work | T05 | 剩余宿主族/依赖删边收据、SD素材；提交并确认work验收全部闭合 | 01–04 |

T-06 收据：core→ui 删边扫描——`vm/native.rs` 零 `ui::clipboard_native::` 调用形引用（仅 PLAN-790 说明注释一行，无尾随 `::`）；`host_capabilities.rs` 零 `ui::` 文本；残留 ui 引用=cfg(test) 测试注入（`tests_clipboard_natives` 的 GlobalClipboardTestLock+实例级 adapter 注入，785 前已在案的测试面形态）。SD 素材冻结稿见 §5 规范增量节下。实施提交 `be5dfea0c`（worktree `D:/autostack/.wt/lang-790/auto-lang`，branch `codex/plan-790-dev`，base `e4ac72a62`，13 文件：3 新增 + 10 修改；触面文件单独 rustfmt，fmt 递归波及的 48 个非触面文件已全部回滚）。merge-owned 门禁（裸 `cargo t` + `tv` + Windows OS 定向档复验）**pending** 于冻结提交。
| [ ] | T-07 | review | T06 | 独立重放反控、临时路径与OS证据、源码去边和SD冻结 | 05 |
| [ ] | T-08 | merge | final pass | 裸t+tv及OS触面门，沉淀/归档/guard清理 | 06 |

## 9. 复审记录

2026-10-16 stage=new | plan_revision=1 | outcome=pass（草案preflight） | next=合同确认并释放native热点后work | 0/8；本轮未运行OS测试或修改provider。

2026-10-16 stage=work | PLAN-790 | plan_revision=1 | outcome=pass（work 完成，6/8；T-07 review / T-08 merge 待接） | worktree=D:/autostack/.wt/lang-790/auto-lang | branch=codex/plan-790-dev | base=e4ac72a62 | code=be5dfea0c | deps=auto-down sibling worktree detached@c39ac7f（组内解析，只读；789/791/792 热点不相交本轮未推进） | tasks=T-01..T-06 | evidence=AC-01（四shim零UI直引+ID/签名不变，no-ui真VM注入 fixture 8/8）；AC-02（G3 降级/编解码/临时PNG 消费期可读——V02/04 与 in-crate 降级锚）；AC-03（引用/堆配平=reap 回基线+两 ListData 形+锁外回调；串线/优先反控=三层哨兵隔离+显式默认幸存）；AC-04（四宿主接线+OS 桥 30/30 含真 OS 往返探针实证；未迁清单入 SD-02 素材）。验证：`cargo check -p auto-lang/-p auto` 零新增警告；`cargo t clipboard` 30/30、`plan790` 7/7、`plan779` 4/4；fixture no-ui 8/8 + ui-host 10/10；no-ui+native-clipboard 组合新可建 | blockers=无合同内未闭项。merge-owned pending：裸 `cargo t` + `cargo tv` + Windows OS 定向档于冻结提交；`cargo tf` 批量到期回归（主检出单实例）。观察项：777 strict `stale_baseline` 为 master 预存（本批零 manifest 边新增；基线重采样归 merge） | next_responsible_role=review（T-07：独立重放反控/临时路径/OS 证据/源码去边/SD 冻结校验）

## 10. 待澄清事项

T01须冻结原图像临时文件寿命和平台实际支持，不新增跨平台OS承诺。若需要修改.at签名/值编码/清理时机，先给具体不兼容点修订合同；不能通过禁用native-clipboard/改空值expected规避验收。

- **[work 观察项，非阻断]** 777 模块边界 strict 检查在 master 干净树即判红 `stale_baseline`（`Cargo.toml`+`crates/auto-lang/Cargo.toml` 自基线 base_revision 变更——785 合入后的既有漂移，非本批引入；本批仅向根 exclude 列表追加 fixture 项，零依赖边新增）。基线重采样须绑定干净树 HEAD，归 merge 阶段或独立簿记处理。
- **[work 观察项，非阻断]** no-ui+`native-clipboard` 单开组合自本批**新可建**（旧 Windows 臂直引 `ui::clipboard_native` 使该组合不可编译）；可建后语义=provider 恒缺席→G3 值降级。此为依赖倒置的改善性副产物，非兼容性承诺。
