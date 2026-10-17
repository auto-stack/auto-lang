---
plan_id: PLAN-805
status: archived
feature_name: Man 工具链配置与构建规则核心抽取
author: [Codex]
created_at: 2026-10-17
updated_at: 2026-10-17
plan_revision: 1
current_step: 8
total_steps: 8
supersedes_spec_components: [docs/specs/auto-man/project.md]
new_spec_components: [docs/specs/auto-toolchain-core/project.md]
touched_goals: [GOAL-003, GOAL-016]
affects: [auto-toolchain-core, auto-man/builder]
depends_on: [PLAN-791, PLAN-795, PLAN-800]
---

# [PLAN-805] Man 工具链配置与构建规则核心抽取

## 0. 变更摘要

把builder/ninja/config.rs、mapper.rs、templates.rs的工具链模型/默认规则/flag映射/命令模板抽为auto-toolchain-core单一实现；旧路径薄re-export，生产NinjaBuilder/CompilerStore/CompilerResolver真实消费新类型与规则。动态am.at、PATH/磁盘/进程观测留Man上层。

## 1. 目标

无auto-lang/auto-man/VM/UI消费者可构造完整现有工具链配置、映射flags并渲染编译/汇编/链接/归档命令；旧公共类型身份、公开路径、默认值和字符串输出保持，不是新增闲置DTO。

非目标：迁Target/Pac、构建DAG/进程runner、Cargo/Vue生成器、模板资产/解释器、改变shell quoting或编译器定位规则、修复Resolver递归旧问题。此轮不宣称Man全部无语言依赖或可独立仓交付。

## 2. 架构方案

auto-man builder/配置加载adapter → auto-toolchain-core → std/必要auto-val轻类型。核心拥有CompilerKind、ExecutableType、FlagFormat/FlagMappings、CompilerLocation、CompilerConfig、FlagMapper、CommandTemplates及现有内聚纯方法；AutoStr/AutoPath保持同型，不制造转换副本。无AutoConfig/Parser/AST/解释器/AutoGen/Pac/Target/进程调用依赖；dev-deps也不拉auto-man做集成测试。

CompilerStore::load_from_am_config/Node解码和加载失败策略、CompilerResolver环境/磁盘/版本检测、NinjaBuilder执行与AutoGen留上层。本计划只抽精确纯规则闭包，不为搬更多代码给核心塞宽泛host trait或新的配置模型。

## 3. 技术栈

Rust workspace轻crate、原auto-val类型、独立consumer与生产Ninja内存输出/专属临时目录对拍。无需安装八类商业/系统工具链，不调用编译器或网络。

## 4. 需求分析与背景调查

用户2026-10-17授权继续规划，并询问799/801前置。本轮只建立合同/设计路线，不执行Rust/建树/agent/旧仓切换，不改他线合同。取材9932962bc157be13bcaa0520011c7c78082cb2fe及[指纹](attachments/extraction-followup-20261017/inputs.json)。791/795/800已归档；无需799/801，也不消费未合入的803资产包。

config.rs/mapper.rs/templates.rs目前只需auto-val/std；配置含MSVC/GCC/Clang/GHS/IAR/Targeting/Hightec/TICLang八类，四可执行角色和三种FlagFormat。CompilerStore则引用AutoConfig读am.at；Resolver引用PATH、磁盘与版本进程，不能把整个ninja目录搬入核心。NinjaBuilder持CompilerConfig并经FlagMapper、CommandTemplates生成规则，是本计划必须证明的生产消费路径。

模板是内联命令字符串，不是803搬的assets/templates或builders/ghs/iar文件；两者不能因为“模板”同名误设依赖或重写资源。与803实现主写面分离，仅Man manifest/lib/Spec/777窄窗口串行。802/799/801不构成AST或运行时API前置。

## 5. 详细设计

T01冻结全部公开类型/字段/构造方法、八类默认配置、枚举alias/大小写行为、四角色命令与扩展名、FlagFormat和mapper边界。区分实际使用的有序flags/参数和原来无序map：不得对命令排序/去重/加引号来“改善”兼容。确认无遗漏inherent impl或私有访问跨crate导致孤儿规则问题。

搬三模块及纯测试至新crate，原builder::ninja::{config,mapper,templates}保薄重导出和ninja::*旧出口；原类型为同一身份，不留重复enum/struct/mapper/render实现。CompilerStore/Resolver/Builder继续按旧公开路径引用即应消费新实现，必要适配不得改算法。

保留map_output现有trim_end_matches及Prefix/Postfix/Both差异、includes/defines/libraries参数次序/空格、CompilerKind别名、MSVC/GCC等默认flags/扩展。CommandTemplates::render仍按原key长度降序替换，未提供占位符不擅删，不增新的escaping/递归插值规范；等长key及含占位符value的原未定序角落T01据旧实现登记，不用排序后的新golden替代已有行为。

真实生产对拍至少覆盖八类CommandTemplates和代表性NinjaBuilder生成（编译/汇编/链接/归档、有include/define/lib/自定义flag、空项/Unicode/含空格路径）。仅观测生成，不执行shell或实际工具链；CompilerStore正常加载和坏配置处置、Resolver已有定位/回退仍是上层合同。

新crate如进入792受管正常闭包，沿已交付清单/身份/staging/admission补齐并重验受影响控制；若不在该闭包须metadata证明，不能强加无关包或把漏包留给792。与803后合入者同步Cargo/lib/Spec素材并集，禁止覆盖资源迁移。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-toolchain-core/project.md | 无轻工具链包→八类配置/flag/命令规则单一定义，零语言/IO闭包 | 可独立消费的构建规则 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-man/project.md | ninja纯规则内嵌→薄门面，动态加载/定位/执行留上层 | 明确实际生产切换及未迁边 | AC-02/03/04 |

## 6. 测试设计

work→同树独立final review→merge，无额外agent/模型指定。拟唯一树D:/autostack/.wt/lang-805/auto-lang，branch codex/plan-805-dev。新fixture由T04建立，本轮不执行Rust。旧臂来自冻结提交/既有oracle，不能从新实现重造expected。

| Case / owner | AC / 不变量 | 输入/生产入口 | oracle与边界 |
|---|---|---|---|
| V01 / work | 01 独立闭包 | 新test/fixtures/modularity/toolchain-core-consumer/Cargo.toml fresh test/metadata | 全模型/mapper/templates可用，normal/build/dev闭包无auto-lang/auto-man/VM/UI/AutoConfig，无进程/文件观测 |
| V02 / work | 02 模型/规则 | 八类默认、全部alias/未知kind、四角色、三FlagFormat、空/多值/含空格与Unicode路径、重复扩展尾缀 | 旧/新字段与具体字符串逐项相同，顺序/空格/trim边界可区分；旧/新路径类型直接互传编译 |
| V03 / work | 02 命令模板 | 四命令、缺变量、相似长短key、原替换次序案例及所有compiler kind | 既有命令字节一致，不偷偷quote/重排/移除未知占位符；未定序情况只按旧承诺比较，不归一掩盖有序差异 |
| V04 / work | 03 真生产/adapter | NinjaBuilder memory输出或独占临时工程；CompilerStore真实am.at正常/坏配置与缺文件；既有Resolver定向控制 | 实际build.ninja规则/命令与旧臂一致，新类型真被生产使用；AutoConfig/定位/错误处置留上层且不漂移；不要求商业编译器执行 |
| V05 / work | 04 单一性/交付 | 删除旧实现、777精确domain、803合流、792生产metadata/包清单 | 无镜像配置/规则/隐藏语言dev依赖；必要792fresh/locked/重定位/缺输入反控触面闭合；无闭包变化给依据 |

work：`cargo check -p auto-toolchain-core`、`cargo check -p auto-man`，新consumer manifest test、新核心定向测试及`cargo test -p auto-man builder::ninja::`（T04核对实际选择数/测试隔离）。全局env控制复用既有串行约束，不能并发改真实PATH/用户am.at。final review后merge冻结裸cargo t+核心/Man定向门；未碰VM/ui_gen不跑tv/tu，tf主检出到期单实例。当前Category A不跑Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC-01 | work | 选定完整模型/规则无语言/执行依赖独立消费，V01通过 |
| AC-02 | work | 同类型身份/旧路径/八类默认/flags/命令输出保持，V02/03通过 |
| AC-03 | work | Ninja生产真实委托，上层动态配置/定位/执行合同未漂移，V04通过 |
| AC-04 | work | 旧实现删除、迁位净删/剩余边/803与792触面、SD素材完整，V05通过 |
| AC-05 | review | 同树独立AC/SD/遗漏及反控审查通过，绑定提交，merge门pending |
| AC-06 | merge | 冻结日常/必要定向门、Spec/ledger/归档与guard清理完成 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 路径/交付与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 791/795/800归档、合同确认 | ninja/config.rs/mapper.rs/templates.rs及store/resolver/builder消费者，闭包/公开面和八类旧臂冻结 | 01/02/03 |
| [x] | T-02 | work | T01 | 新crates/auto-toolchain-core，单份模型/mapper/templates/纯测试，check与V01/02；R1 F01 修复：迁入三文件已 rustfmt（scoped --check 退出 0），迁移相等证据绑定 4d1aade5a blob 复核，当前修订以 16 golden 行为对拍为证 | 01/02 |
| [x] | T-03 | work | T02 | 旧三模块/ninja出口薄re-export，真实CompilerStore/Resolver/NinjaBuilder接线，删除旧实现，定向V03/04 | 02/03 |
| [x] | T-04 | work | T03 | 新独立consumer、全八类/四命令/无效输入、真实生产与配置控制V01–04，记录实际命令/输出；R1 修复：未使用导入清零（新测试零告警）+ 替换序确定性控制×2（R1 复放集期望值）+ 等长 key 双态容忍控制（consumer 第 5 测试） | 01/02/03 |
| [x] | T-05 | work | T04 | V05 777精确边/单一性/803窄合流/792闭包，必要交付控制 | 04 |
| [x] | T-06 | work | T05 | 迁位/净删/剩余管理层与SD/SHA收据，提交送审；R1 修复：SD-01 render 语义改准+真实模块表+验证矩阵改列表、INDEX 再生列 config/mapper/templates；修复提交 589e30399 | 04 |
| [x] | T-07 | review | T06 | 同树独立AC/SD及规则/生产反控审查；R2 final pass 绑定589e30399，R1三项CLOSED，merge门pending | 05 |
| [x] | T-08 | merge | final pass | 同步master、受影响复审/冻结门、沉淀/归档/wt-guard清理 | 06 |

## 9. 复审记录

2026-10-17 / stage: new / PLAN-805 revision: 1 / outcome: pass / next: 合同确认后work。本轮无799/801/802/803硬依赖；三个纯模块和803资源根互不重叠，共享manifest/出口短窗口串行。[预检](attachments/extraction-followup-20261017/preflight.md)。

2026-10-17 / stage: work 开工 / PLAN-805 revision: 1 / worktree=D:/autostack/.wt/lang-805/auto-lang / branch=plan-805-dev（合同§6拟名 codex/plan-805-dev，按仓规 AGENTS.md L1 约定改用 plan-<NNN>-dev；其余合同不变）/ base=282e50053（master）。

T-01 冻结收据（2026-10-17，基线 282e50053 实读源码）：

- **config.rs（596行）公开面**：`CompilerKind`（8变体；`from_str` 大小写不敏感 + 别名 `gnu`→GCC/`llvm`→Clang、未知→None；`as_str` 全小写；`Display` 走 as_str）、`ExecutableType`（`from_str` **大小写敏感**精确匹配 `cc|compiler`/`ar|archiver`/`link|linker`/`as|assembler`；`as_str`=cc/ar/link/as）、`FlagFormat::{Prefix,Postfix,Both}` + `apply`（Prefix/Postfix=`template.replace("{}",value)`，Both=三段 format!）、`FlagMappings`（6 字段 include/define/library/library_path/output/compile_only + 8 个 `*_default()`）、`CompilerLocation::{Env,Dir(AutoPath),Executable(ExecutableType,AutoPath)}`、`CompilerConfig`（6 pub 字段 + `new()` + 8 个 `*_default()` + `get_executable`/`get_object_extension`（MSVC=".obj"余".o"）/`get_executable_extension`（MSVC=".exe"余""））。
- **八类默认冻结**：executables 表（MSVC cl/link/lib/ml64；GCC gcc/gcc/ar/as；Clang clang/clang/ar/clang；IAR iccarm/ilinkarm/iarchive/iasmarm；GHS ccarm/ccarm/cxarm/asarm；Targeting tcpp/tlink/tlib/tas；Hightec gcc/gcc/ar/as；TICLang tiarmclang/tiarmclang/tiarmar/tiarmclang，均 .exe 后缀）+ default_cflags（MSVC `/O2 /W3 /MD`；GCC/Clang/GHS/Targeting/Hightec `-O2 -Wall`；IAR 12 条 `--no_cse…--fpu=VFPv4_sp`；TICLang `-Oz -Wall`）+ FlagMappings 8 表（MSVC 独有 `/I{} /D{} {}.lib /LIBPATH:{} Both("/Fo",".obj") /c`；IAR 独有 compile_only=`--debug`；余同 GCC 形）。
- **mapper.rs（291行）**：`FlagMapper` 13 关联函数；`map_output` 语义冻结——Prefix/Both 用 `output.trim_end_matches(obj_ext)` 的 base_output（重复尾缀全剥），Postfix 用原始 output；`map_compile_only` Both=concat(prefix,postfix)；批量 build_* 保序、format_flags=join(" ")。
- **templates.rs（283行）**：`CommandTemplates` 4 pub String 字段；`new()` 按 kind 派发（**TICLang→clang_templates**）；8 个私有 `*_templates()`（单测同模块可访问，迁出后仍私有）；`render()` 按变量名长度降序（stable sort）替换——**等长 key 的迭代序来自 HashMap，属原未定序角落，登记不归一**（占位符为完整 `{key}` token，等长互不为子串，正常值下结果确定）；`render_compile/assemble/link/archive` 固定 7/5/5/3 参。四命令模板冻结：MSVC `/Fo{output}`+`/OUT:`、GCC/Clang/Hightec `rcs`、IAR `--debug -o`+归档 `-o`、GHS/Targeting 归档 `-r`。
- **消费者（留上层）**：compiler_store.rs（AutoConfig/am.at 解析+坏节点 warn 跳过）、resolver.rs（PATH/磁盘/进程观测 + `default_executable` 与 config 默认表镜像 + MSVC 同目录优先）、builder.rs NinjaBuilder（`load_compiler_config` 缺省 gcc_default；**`build_includes` 是本地 `-I../{p}` 格式不走 FlagMapper**；`build_defines` 走 `FlagMapper::format_defines`；`build_cflags` join default_cflags；渲染经 `CommandTemplates::new`+`quote_if_spaced`）、pac.rs:1163+（port.at compiler 节点解析）、port.rs（`Port.compiler` 字段+平台默认）、lib.rs:123（`pub use builder::ninja::config::CompilerConfig`）、builder.rs:59（BuilderKind 派发）。类型引用全部限于 auto-man 内，无仓外消费者。
- **闭包确认**：三模块依赖仅 auto-val/std，无 AutoConfig/Parser/AST/AutoGen/进程依赖；792 交付闭包 `DELIVERY_CRATES`（rust_ui.rs:3988，11 crate）不含 auto-man 系，新 crate 仅被 auto-man 依赖→不进入 792 闭包（T-05 出 metadata 证明）。
- **旧臂 golden 捕获方案**：V04 oracle 在 base 提交上以未跟踪探针测试捕获（NinjaBuilder memory 输出钉 `CompilerLocation::Dir` 不存在目录规避 PATH/磁盘观测；单 src/target 规避 `srcs: HashSet` 迭代序不定），fixture 冻结入 `crates/auto-man/tests/fixtures/plan805/`，抽取后同测试按旧公开路径（经 re-export 消费新实现）断言字节一致。

2026-10-17 / stage: work | PLAN-805 | plan_revision: 1 | outcome: pass（T-01..T-06 全部完成，冻结树全绿，送 T-07 final review） | worktree=D:/autostack/.wt/lang-805/auto-lang | branch=plan-805-dev | base=282e50053 | code=66fc9d710（4d1aade5a 实现 + 66fc9d710 777 基线重绑） | dependency_revisions=auto-down c39ac7f9（组内 detached 兄弟，未改动） | task_ids=T-01..T-06 | blockers=无 | next_responsible_role=review（T-07 同树独立 final review）

**Work 收据（证据绑定 66fc9d710）**：

- **迁移**：三模块**逐字**迁至 `crates/auto-toolchain-core/src/{config,mapper,templates}.rs`（与 base 282e50053 diff 为空，字节一致）；旧三文件改为薄 re-export 门面（仅 `pub use auto_toolchain_core::*` + 文档说明），`builder::ninja::*` 旧出口/mod.rs/lib.rs:123 根 re-export 零改动——CompilerStore/Resolver/NinjaBuilder/Port/Pac 经旧路径消费新实现（真实接线，非旁路）。净账：auto-man 侧 -1146 行实现 → +30 行门面；新 crate 1195 行（含原纯测试）。
- **V01**：`test/fixtures/modularity/toolchain-core-consumer/`（Cargo.toml 独立 workspace + 4 测试 + verify-deps.sh）——外部消费八类完整模型/别名解析/flag 映射/四命令渲染/自定义 CompilerConfig；闭包=consumer→auto-toolchain-core→auto-val+第三方，normal/build 无 auto-lang/auto-man/auto-gen/auto-ui/auto-vm/auto-ast（verify-deps PASS；本机 sccache 端口排除段故障，经 RUSTC_WRAPPER= 空跑通，脚本与 800/803 同型）。
- **V02/V03**：旧臂 golden 16 件（base 上双捕获 run1/run2 diff 为空=确定性）→ 新臂断言模式逐字节 PASS（`plan805_toolchain_parity`）：八类默认 dump/alias（含 `gnu`/`llvm`、ExecutableType 大小写敏感）/三 FlagFormat（含值含 `{}` 的全量替换）/mapper 边界（空集/多值/空格/Unicode/`main.o.o` 重复尾缀全剥/Postfix 不剥/Both compile_only 拼接）/八类×4 模板（TICLang=clang）/render（缺变量原样保留、长短 key、空 ldflags 双空格）/MSVC `/Fo{output}`。类型同一性 `plan805_type_identity`（旧路径值直传新 crate 函数+反向+跨 crate HashMap 键）编译即证。
- **V04**：NinjaBuilder memory 生产对拍 9 件（八类 windows + gcc linux 自定义 flag/含空格工具路径触发 quote_if_spaced）逐字节一致；Resolver 既有定向控制（MSVC 同目录优先/Executable 钉定栈溢出守卫/裸名守门）在 `builder::ninja::` 选择面 12 passed+1 ignored；CompilerStore am.at 三态冻结（合法文档→空 store Ok / 坏语法→Err / 缺文件→空 store Ok）。
- **V05**：① 单一性——`pub enum/struct` 全仓唯一定义于新 crate（grep 证据）；② 777——`auto-toolchain-core = man-ecosystem` 域登记，实现提交后基线重绑（declared_edges 56，**strict pass/0 violations/2 grandfathered**，无规则放宽，F-795-R1-01 同序）；③ 803 合流——root/auto-man Cargo.toml 与 auto-man-assets 条目原样保留（只追加），资源零触碰；④ 792——cargo metadata 证明新 crate 反向依赖=仅 auto-man，`DELIVERY_CRATES` 11 成员零泄漏，不进入交付闭包、无需 admission/staging 变更。
- **SD 素材**（worktree 已入库）：SD-01 `docs/specs/auto-toolchain-core/project.md`（含冻结语义 7 条+验证矩阵）；SD-02 `docs/specs/auto-man/project.md` 新增「工具链规则核心抽取分工（PLAN-805）」节+builder 行门面注记；`scripts/spec-index.py` 工具链组+INDEX 再生（41 项目）。
- **债登记**：P805-D1（预存，非 805 引入）——`CompilerStore::load_from_am_config` 以 `get_kid(Str)` 找 compilers 而解析器产位置 Int 键，合法 am.at 形态下恒空载；三态行为已冻结入 golden，修复建议 `get_nodes` 同型迭代，见 KNOWN-DEBT-AND-RISKS.md Plan 805 节。
- **预存环境/既知项**（非本计划引入）：`cargo test -p auto-man` 全 targets 因 P736-R4 预存坏 target（plan734 bin/fixture）编译失败，故 ninja 选择面经 `--lib` 跑（12+1 ignored）；本机 sccache 配置端口落 Windows 排除段（os error 10013），所有 cargo 调用经 `RUSTC_WRAPPER=` 绕过。
- **未跑（merge-owned，pending）**：冻结裸 `cargo t` + 核心/Man 定向门在 merge 阶段统一执行；未碰 VM/ui_gen/trans/book/aavm，tv/tu/tt/tb/taa 不适用；tf 到期判定归 `/auto-plan:regress`。

2026-10-17 / stage: review / review_scope: final / reviewer_identity: Codex（本次审核会话，未参与实施；无子 agent，实际模型/会话 ID 未暴露） / PLAN-805 revision: 1 / **outcome: needs_fix** / worktree_path=D:/autostack/.wt/lang-805/auto-lang / branch=plan-805-dev / reviewed_commit=66fc9d71001eab80dd33aacfe74102da565288a5 / base_commit=282e50053da41c3b65c1e424ca0d251f1126986b / dependency_revisions=auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 / next=原树 work 修复并提交，再复审。

- [R1完整报告、AC映射与修复要求](attachments/805/review-r1.md)；[基线/依赖/Spec源哈希](attachments/805/review-r1-baseline.json.txt)；[冻结规范候选增量](attachments/805/review-r1-spec-delta.patch)；[可重放顺序反例](attachments/805/review-r1-render-probe.rs)。
- **F-805-R1-01（P2，health，T02/04/06，AC05）**：新增 parity 测试 `FlagMappings` 未使用导入实际 warning；迁入三个核心模块 scoped rustfmt check exit 1。既存依赖告警与 golden 输出末尾空行不混为新增问题。
- **F-805-R1-02（P2，SD-01，T04/06，AC04/05）**：新 Spec“不递归插值”未准确描述单轮顺序替换；实测 `{longkey}`→插入`{x}`→`done`，逆序则保留`{longkey}`，等长 key 保留 HashMap 未定序。修正文案并补区分性控制，不改原算法迎合新承诺。
- **F-805-R1-03（P3，SD-01，T06）**：新项目索引把验证表头/命令解析成模块明细、空状态，建议随 SD 修复；本项不单独决定 needs_fix。
- **本轮实跑**：核心38单测+6doc、独立consumer4、16 golden对拍+类型身份2、ninja选择12 passed/1 manual ignored、两包check、777 strict pass/0 violations/2 grandfathered；metadata完整normal/build/dev图和792的11成员闭包无新核心漏包；新旧三模块Git blob逐字一致。AC01/02/03行为通过，AC04 partial，AC05 fail，AC06 merge-owned pending。未发现生产算法回归；P805-D1旧加载器空载保持并非成功加载。
- **状态**：保留T01/T03/T05，重开T02/T04/T06，T07/T08未完成，**executing 3/8**。未修改实施或主检出canonical Spec/live ledger，未合并/归档/清树。修复后重绑受影响证据与规范增量SHA。**pending_closeout_gates**：最终pass后merge冻结裸cargo t+核心/Man定向门；tv/tu/tt/tb/taa本面不适用，tf归主检出到期批量回归。

2026-10-17 / stage: work（R1 修复轮）| PLAN-805 | plan_revision: 1 | outcome: pass（R1 三 findings 全闭合，复放受影响定向门全绿，二送 T-07 final review）| worktree=D:/autostack/.wt/lang-805/auto-lang | branch=plan-805-dev | base=282e50053 | code=589e30399（4d1aade5a → 66fc9d710 → 589e30399）| dependency_revisions=auto-down c39ac7f9（未变）| task_ids=T-02/T-04/T-06 重勾 | blockers=无 | next_responsible_role=review（T-07 第二轮）

**R1 修复收据（证据绑定 589e30399）**：

- **F-805-R1-01 闭合**：① `plan805_toolchain_parity.rs` 移除未使用 `FlagMappings` 导入——修复后定向测试编译零新增告警（对照复放日志，`FlagMappings` warning 0 次命中；残余 `unused doc comment`/`Sender` 为依赖库既存，与 review check 日志同款）；② 迁入三模块 `config/mapper/templates.rs` 按 F01 指令 rustfmt（scoped `--check` 退出 0），**相等证据口径切换**：迁移逐字节相等绑定 4d1aade5a（复审 blob SHA 已核实），当前修订以 16 golden 行为对拍 + 38+6 单测为证（格式化后复跑全过）；③ golden 末尾空行保持未动。
- **F-805-R1-02 闭合**：SD-01 冻结语义第 4 条改写为准确算法描述——「按 key 字节长度降序做**一轮**顺序替换，逐 key 对当前累计结果 `str::replace`：后处理的（更短）key 会展开先前插入值中的占位符；已处理 key 不回扫（非不动点）；等长 key 处理序继承 HashMap」，删除"不递归插值"表述，算法零改动。补区分性控制（期望值取自 R1 复放集，锚定冻结核心=旧臂 blob）：parity 测试两条确定性断言（`{longkey:"{x}",x:"done"}` 渲染 `{longkey}` 得 `done`；`{longkey:"done",x:"{longkey}"}` 渲染 `{x}` 得 `{longkey}`，直接断言不入 golden 保持旧臂捕获面）+ consumer 新增第 5 测试 `external_consumer_pins_render_ordering_semantics`（同两例 + 等长 key 256 次迭代结果只落 `{bb}`/`done` 两种合法排序之内、无第三态）。
- **F-805-R1-03 闭合**：SD-01 增设真实模块表（config/mapper/templates 三行）+ 验证矩阵改 bullet 列表（脱离索引器表格正则）；`scripts/spec-index.py` 再生 INDEX——新核心模块明细现为真实三模块（模块数 4→3，伪"入口/覆盖"条目消除）。
- **复放门禁（589e30399 上实跑）**：`cargo test -p auto-toolchain-core` 38+6；parity+identity 2 pass（16 golden 逐字节 + 新顺序控制）；consumer 5 pass；`verify-deps.sh` PASS（RUSTC_WRAPPER= 绕过本机 sccache 端口排除段）；777 strict pass/0 violations/2 grandfathered（manifests/边未变，基线仍绑定 4d1aade5a 有效）；plan 迁入/新增 Rust 文件 scoped rustfmt --check 退出 0。auto-man 门面/上层消费文件本轮零改动（T-03/T-05 证据不受影响）。
- merge-owned 门禁保持 pending（裸 `cargo t` + 核心/Man 定向门在最终 pass 后的冻结提交上执行）。

2026-10-17 / stage: review / review_scope: final / reviewer_identity: Codex（本会话独立审核，未参与实现/修复；无子agent，未暴露模型/会话ID不推断） / PLAN-805 revision: 1 / **outcome: pass** / worktree_path=D:/autostack/.wt/lang-805/auto-lang / branch=plan-805-dev / reviewed_commit=589e3039962d0cc8f1cc473d8d8190d9f8083949 / base_commit=282e50053da41c3b65c1e424ca0d251f1126986b / dependency_revisions=auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 / next=merge。

- [R2完整报告与AC映射](attachments/805/review-r2.md)、[基线/依赖/Spec输入指纹](attachments/805/review-r2-baseline.json.txt)、[冻结规范增量](attachments/805/review-r2-spec-delta.patch)。规范增量SHA256=`7df10957ae593bb4a7f60e5d84f48a2a0a943427e04b1c703c5007a69f891fdb`。
- **F-805-R1-01/02/03 全部CLOSED**：新增测试导入告警消除、scoped格式检查exit0；独立证明三核心模块恰为旧blob的rustfmt输出；render规范如实描述一轮顺序替换，确定性正反顺序与等长key原例重放通过；现有索引器临时重生成全文等于已提交INDEX，仅列config/mapper/templates三个active模块。
- **本轮实跑**：核心38单测+6doc、consumer5、parity/identity2（16份原golden+顺序控制）、R1原始probe、闭包守卫、777 strict pass/0 violations/2 grandfathered、scoped rustfmt全部通过。完整locked metadata与R1深比较相等；上层消费者/manifest/验证配置未改、auto-down未变，复用R1 ninja12+1manual ignored/check/792闭包与803合流证据，具体指纹和复用边界见报告。主检出相关规范输入未漂移，无新增阻断或遗漏/延期/workaround。
- **acceptance_results**：AC01–05 pass，AC06 pending。T01–07完成，**reviewed 7/8**；T08保持未完成。先前修复未落地时的核对仅为readiness检查，不计完整final轮次。P805-D1旧加载器空载边界保留。
- **pending_closeout_gates**：最终冻结裸cargo t+核心/Man必要定向门归merge；同步如改变受审代码/依赖/配置须复核受影响内容。tv/tu/tt/tb/taa无触面，tf主检出到期单实例（805落地%5触发）。Spec/ledger/overview/goals/plans/索引沉淀、归档、wt-guard清理未执行；本次未合并或宣称交付。



2026-10-17 / stage: merge | PLAN-805:r1 | outcome: pass（delivered）| delivery=ecf53035e7410afaea445e7e18e071c33773f637 | worktree=D:/autostack/.wt/lang-805/auto-lang | branch=plan-805-dev | base=282e50053 | dependency_revisions=auto-down c39ac7f9（未变，零提交）

**合并检查点收据**：

- `prepared`：R2 final pass 绑定 589e30399（冻结规范增量 SHA256 7df10957ae59）；rebase over 801/802/804/788 合入线（映射 4d1aade5a→29c978be9 / 66fc9d710→48b4f6651 / 589e30399→8b951bb8c，range-diff 提交 1/3 全等、提交 2 仅基线 json 解决方式差异且由 merge 期重采样覆盖）；merge 预备提交 7d9cdb1dc（plans.md 建表+基线重采样 strict pass/0 violations/2 grandfathered，declared_edges 60）+ 二次 rebase over 788 簿记线（代码/测试/配置与门禁提交 diff 为空）+ 基线再采样 ecf53035e。
- `landed`：master tip == ecf53035e（`git merge --ff-only plan-805-dev`，无 merge 提交）；主检出 smoke `cargo check -p auto-toolchain-core -p auto-man` 通过；规范文件在位（project.md/plans.md/INDEX 42 项目）。
- **冻结门禁**（2026-10-14 裁定，merge-owned，跑于 4d3e9746d；二次 rebase 仅 docs/plans 簿记、crates/test/scripts/Cargo.toml diff 为空，证据显式复用于 ecf53035e）：裸 `cargo t` **5265/5265 PASS**（1 slow=plan738 p4 67.5s，1550 skipped=预配置 skip 面，零红零 flake）；定向门——核心 38+6、parity+身份 2、ninja-lib 12+1 manual ignored、consumer 5、verify-deps PASS、777 strict pass。tv/tu/tt/tb/taa 无触面不适用。
- `ledger_refreshed`：.autoos/specs.json **v43→v44 guarded offline projection**（本会话无 auto-ai store writer；备份 .autoos/specs.json.bak-v43-805；程序化 read-modify-write；范围守卫=仅 designs/tests/reviews/reports 四节各 prepend 1 条（P805-D1/P805-T/P805-R2/P805-M）+version+1，其余节逐节全等；六节 id 集合校验+读回验证通过）。
- `archived`：git mv → docs/plans/archive/805-toolchain-rules-core-extraction.md，status: archived，T-08 勾，8/8；规范/ledger/簿记随本提交入默认分支。
- `cleaned`：见下方终回执。
- **生产工件检查**：auto-lang 为库仓，无随 master 消费的后端 release 二进制/守护进程/Web bundle 重建协议（auto-musk 专属步骤不适用）；本仓生产消费面=cargo 工作区自身，裸 t 已覆盖。
- **批量回归到期判定**：805 % 5 = 0 且 805 > covered（.last-batch-regression.json last_covered_plan_id=800）→ **到期**，合入后由 /auto-plan:regress 在主检出单实例执行。

## 10. 待澄清事项

无外部决策阻塞。T01负责全部公开API/实际旧命令边界，不借“纯规则”删掉兼容方法；Target/构建DAG/命令协议/进程退出另立后续计划。auto-toolchain-core是仓内crate名，不新建Git仓。
