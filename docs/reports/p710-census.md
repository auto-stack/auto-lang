# PLAN-710 T-00 普查报告：a2r 生成缺口全量分类与穷尽性判定

- 日期：2026-09-30（执行日）
- 工具链：本仓 worktree `plan-710-dev` 构建 `auto` release（`v0.4.2-2243-gb310acafd`；判据前核构建源——016/018 纪律）
- corpus：auto-edit **main@`70c5c603dcafdb1d4f7776dc5af2ce0bb2bcb72c`** 钉版快照（`git archive` 提取，落 `.wt/lang-710/p710-corpus`——组内兄弟布局使 pac.at 的 `../../../auto-lang/blueprints` dep 与 `compute_auto_lang_rel_path` 兄弟漫步自然解析到本 worktree；**零 junction**——deps/stylekit、deps/blueprints 预置真实拷贝，`copy_local` 见在即跳过 materialize）
- 判定命令：`auto build -r rust`（cwd=corpus 的 `specs/auto-edit`，`AUTO_LANG_CRATE` 钉本 worktree crate）
- 证据文件：`p710-run/regen-{baseline,r1..r22}.log`、`census-*.txt`（组目录 `.wt/lang-710/p710-run/`）

## 1. 基线普查（未修改工具链，供① 承接前）

`regen-baseline.log`：rustc **133 errors**（+30 warnings）——与 018/019 两轮探针
的 133 错口径**逐数吻合**（018 日志未随 fold 存续，本普查即 fresh 重跑基线）：

| 类别 | 计数 | 说明 |
|---|---|---|
| `/* expr */` 占位（G-A） | **17 行** | 与 018 §①-b 表一致 |
| `/* unhandled stmt */`（G-B） | **4 处** | 与 018 一致 |
| 裸 `code_editor_delta`（G-C） | **3 调用点** | 与 018 一致；唯一 E0425 |
| E 码分布 | E0308×96 / E0599×11 / E0061×4 / E0605×2 / E0277×2 / E0425×1 | 018 口径同形（±2 归工具链 2205→2243） |
| `error: a2r codegen: prop controller … PLAN-027 拒绝门` | 1 | **基线在册**（scroll-pane controller，见 D-5） |

## 2. 生成链路定界（T-00②）

`auto build -r rust` → auto-man `rust_ui.rs` → **`ui_gen/rust.rs` RustGenerator
单链**（rust_ui.rs:19 `use auto_lang::ui_gen::rust::RustGenerator`）。
**`trans/r2a.rs` 不在此链**（旧 trans 面零触及——计划非目标兑现）。
G-A 单侧落位 ui_gen/rust.rs 判定成立。

## 3. 逐类归因与回补判定（T-00①③——穷尽性枚举）

三类清偿后（G-C/G-B/G-A，commit 5c1d43b52）迭代 regen r1→r22，逐步暴露
的**非三类残余**全数枚举如下（018 普查将其统归「占位符类型推断塌方」——
本普查实证为**独立生成器缺面**，误并档在此更正）；按计划 §5 T-00
「非三类残余枚举（若有=§10 记录回补本件范围）」全部回补进本件：

| ID | 残面 | 根因（生成器落点） | 株（corpus 实例） | 修形 |
|---|---|---|---|---|
| D-4 | `scroll_to` E0425 | ui_gen `vm_builtin_host_call` 无臂（基线解析期被 G-A 语法占位吞没——占位注释使 rustc 停在 parse，名字解析未达） | app.at DiffScrollToHunk `scroll_to(.diff_scroll,"y",off)` | 表增臂——shim_scroll_to 轴形同源（intent enqueue） |
| D-5 | scroll-pane `controller` prop PLAN-027 拒绝门 | ui_gen view 发射 `scroll`/`scrollable` 臂未收 `scroll-pane` 别名（schema.rs:492/499 在册别名） | app.at:609 `scroll-pane (controller: .store.diff_scroll,…)` | 别名入臂 + controller 透传 `ScrollControllerBinding(Arc<str>)`（VM 轨 PLAN-656 Scrollable 同源） |
| D-6 | 条件位 Value 裸读 E0308（`expected bool, found String` ×22 基线在册） | convert_condition/var.field 重写与 stmt If/While 条件发射按访问器启发式猜型（envelope 字段名表外落 as_str） | diff 视图 `r.ix_del`/`e_same`/`sx_*` 族条件 | 条件位 truthy 投影（`a2r_std::json::truthy`——VM `Value::is_true` value.rs:682 同源）五挂点（convert_condition×2/cond_expr_rust→If/While/if-expr）+ bool 声明/赋值强转臂 + declared-str/local 拷贝 clone 语义（E0507/E0382 株族：path/diff_a/src/diff_rows/out/body） |
| D-7 | merged back-api GET/POST 桩契约错位（E0061×4/E0599×11 及级联） | auto-man rust_ui.rs merged api 客户端 GET 分支恒发 by-id 模板 `fn f(id: i32) -> Option<Value>`、POST 恒发 `-> Value`——与契约（`env_str(name str) str` 等 query 端点）错位 | env_str/read_text/exists/file_size/search_files/read_text_range/tree/diff_buffers/sync_copy/sync_delete | GET 桩按契约参数/返回收口（str→""/bool→false 缺省，请求记录架构保形）；POST bool→bool、str→String（记录项序列化回传——`json.to_value(regex_replace(…))` 消费链按契约 parse）。route-A 深修（api_impl 委派真实现）不在本件——fsys.at 转译链 `.ok()?` 静默失败致 route A 未启用，登记 §10 为后续件边界 |
| D-8 | `char_at`/`substr`/`to_int` E0599/E0605 | ui_gen 内建方法翻译表（try_builtin_method_call）缺员 | BOM 探测 `w.char_at(0)`/`w.substr(0,3)`、`.diff_buf_ia.to_int()` | 字符语义臂（VM 串按字符纪律——len 同款）：`chars().nth`/`chars().skip.take`/`trim().parse::<i32>().unwrap_or(0)` |

## 4. 终态判定（T-04 三重判据——r22）

| 判据 | 结果 |
|---|---|
| `auto build -r rust` exit | **0**（r22，工具链同上） |
| 生成物 grep | `/* expr */`=0、`/* unhandled stmt */`=0、裸 `code_editor_delta`=0（直调臂×3 在位）、`compile_error!`=0、`unimplemented`=0 |
| 生成 workspace cargo check | **过**（`Finished dev profile in 56.55s`，0 error，171 warnings——警告面为既warning 惯性，非错误） |

**普查对照：133 → 0（穷尽性实证成立）**——三类+回补五面即全量；非上表
残余为零。

## 5. 双轨语义同源对拍（T-05，AC-01..03 单测面）

`plan710_supply_probes` 11 测试全绿：G-C VM 轨 envelope/destructive
watermark/未注册错误消息 vs core 直读；G-B try 成功臂零扰动+错误臂
catch(e) 绑定消息串；G-A envelope 五段投影 VM 轨 vs a2r_std helper 同
fixture + truthy↔`Value::is_true` 逐格；发射形七断言（catch_unwind 形、
delta 直调+shim 消息、coalesce 七族含负缺省/嵌套切片/局部接收者、
scroll_to、char_at/substr/to_int）。

## 6. pristine 断言

auto-edit 检出（D:/autostack/auto-edit）零触碰：corpus 全程组内 tmp 拷贝
（git archive 快照）；`git -C D:/autostack/auto-edit status` 执行前后一致
（仅其自有 WIP：specs/stylekit 两删除在录，非本件触碰）。组目录内零
junction（reparse 扫描过；deps 两 dep 预置真实拷贝规避 materialize 的
mklink /J 路径）。

## 7. R2 增补判定集（PLAN-714 r2——盲区堵截三行）

> 2026-09-30 r2 实录：本 census §4 的「cargo check 过」为**陈旧基面
> 掩蔽假绿**——tmp 拷贝携带旧代 fsys.rs 满足导入（021 fresh 复验
> E0432 实证）。以下三行入判定集，堵截三类掩蔽。

| # | 判据 | 形态 | r2 收据（2026-09-30，工具链 v0.4.2-2330-g99405c675-dirty） |
|---|---|---|---|
| ① | **skip 警告 grep 零命中** | regen 日志 grep「⚠ … transpile failed (module skipped)」=0——缺模块洞机读暴露（710 实录该警告在案但不在判定集） | **0 命中**（fsys.at:208 try 臂落地后整模块恢复发射——a2r-run.log grep 实证） |
| ② | **fresh-copy 卫生** | regen 前清生成区（`rm -rf rust-workspace`）或生成文件清单 diff——堵陈旧基面掩蔽 | 本轮 fresh（快照+清区后首跑）：fsys.rs 全新生成（try 闭包体在册），暴露掩蔽层=26 错（§8） |
| ③ | **api 客户端桩形检测** | 生成物 grep `String::new()` 恒返形（D-7 桩）=0——编译绿假阴性堵截 | **实体形在位**：`pub fn env_str/read_text/search_files…` 实体委托 `fsys::env_lookup/read_text_range…`（非 D-7 恒返桩）——route-A 伴生嵌入恢复实证（AC-R2-2 实质达成） |

**r2 判定结论**：①③ 绿、② fresh 成立；**exit 0/cargo check 未达**——
fresh fsys.rs 暴露 710 §10 显式延后的 route-A 深修层（26 错/8 类）——
路由 r3 深修阶段（同件收纳）。

**r3 收口判定（2026-09-30，PLAN-714 r3 R3-T4/T5——深修落地后同判据复跑）**：
**① skip 警告=0 ② fresh 卫生成立 ③ 实体形在位** 全绿维持；**`auto build
-r rust` exit 0**（r3f 轮）+ **fresh workspace cargo check 过**（1m07s）；
下游真仓 perf.py a2r 三重判据全绿（exit 0+三占位 0/0/0+fresh check
59.82s）——**AC-R2-1 补全+AC-R2-4 交付，PLAN-021 解阻**。深修面=
stdlib（fs::copy_recursive/remove_dir/read_bytes_list/write_bytes_list、
re::test/replace、json::from_value/parse_str_list、diff 三件套[code-editor
门双轨]、list=Vec<Value> 别名）+表臂三处分发器齐装+Try return 传播形
（Option<Ret>——710 边界条款实实例收口）+借位迭代变量 clone 窄门；
附带收口=cookbook file 003/004 夹具随 VM fs.metadata→size 现行语义
同步（.len()/.modified() 死形退役）。工具链 v0.4.2-2330-g99405c675-dirty
→r3 提交链（96e848ea5+3574b47c8）。

