# 固定Cargo包与Rust模块测试（PLAN-896 / PLAN-898）

当前入口是`cargo t <组> [组...]`。分组配置在`../test-groups.toml`。

- `cargo t`：原日常固定配置，ACE宿主显式启用。
- `cargo t parser` / `cargo t parser vm`：完整Rust模块子树及配置中明确的集成测试，兼容配置一次nextest并行。
- `cargo t all`：显式语言仓全部固定组，仍遵守ignore/平台/里程碑条件；不包括外仓UI/Man所有权腿。
- `cargo t --list`：只读固定组名，不编译。
- `cargo t parser --select-only`：只输出调用计划，不实际发现或执行。
- `cargo t all --discover-only`：明确的迁移/覆盖审计，会构建所选测试目标并列实际身份，执行0测试；不是普通运行前置。
- `cargo t aavm --ignored --test-name <名字>`：手动重放自举尝试，保原T3条件。

PLAN-898（2026-10-31）：`tm`公共入口、旧档别名（tv/tt/tb/th/tf/taa/ta/t3）、tm引擎
（module_command/modules.json/recipes.json/legacy-contract 等冻结目录）与 tier 选择基线
（full-regression-*-ids.txt、tier-selection-minimums）已整体删除；历史语义见归档计划
888/895/896/898。周期批量回归=主检出单实例 `cargo t all`（触发条件不变：计划号%5、
或>48h且有合并、缺收据即到期），UI/Man仍走各自owner入口。

所有实际命令产生原生Job资源、真实逐项终态及source/config/tool身份收据。缺项、额外执行、重复终态、
首红/崩溃/超时/清理错误不会变PASS，条件排除/未执行分列。VM/语料包装保持原source/oracle inventory与child check。

批量性能见 `measure_pilot.py` 与冻结 `measurement-contract.json`。`test-case-pilot` 的固定
15例及单例重放均默认A；B须显式--mode group，C须显式D3观察。新候选收益矩阵后置，未证明前不自动推广。
不要将本批改善推断为全生态改善。详见[规范](../../docs/specs/auto-test-case/module-selection.md)。

准备、发现（包括Git/Cargo元数据）和实际执行都使用有界的Windows Job；父进程退出0而后代仍在运行属于基础设施错误，记录实际PID/映像并回收整树，不能据此发布工具就绪或测试通过。临时目录相关TEMP/TMP/TMPDIR参与有效环境指纹。

TestCase在系统短临时目录运行，避免SQLite等原生后端的长输出路径限制；命令仍保留真实runtime argv。退出后先逐文件SHA256校验复制到请求的输出目录，再按目录所有权标记回收临时目录，复制/清理计入模块墙钟。`case_artifacts`直接提供可读报告与复制清单，原始报告/收据字节及其原runtime路径不改。reparse point在遍历前拒绝，不向worktree复制链接，复制失败保留原目录供排查。

原计划的历史写入范围审计绑定其实际可达实施基面和归档交付SHA，不能审查当前计划相对master的无关改动；原命名空间禁区断言保持，当前产品依赖边界继续由实时结构检查负责。

编译准备/发现有一项明确例外：`compiler-auxiliaries.json`钉住当前参考机Microsoft签名MSVC遥测辅助程序的完整路径和SHA256；仅在编译器成功退出、所有残留映像逐一匹配且Job实际回收后active=0时接受准备。原meter错误、PID、映像和回收记录不改。未知映像/错hash/回收未完成仍失败，实际测试负载不启用例外；更换工具版本须重新核验该源码配置。Cargo launcher注入的包标签不算有效配置变更；Cargo home及仓内env默认按实际生效值绑定，显式不同覆盖仍使证据失效。

Windows受计量命令明确禁用RUSTC_WRAPPER/RUSTC_WORKSPACE_WRAPPER，直接运行原rustc；既有外部缓存daemon不在本Job内，不能让它派生编译器而绕过CPU/超时回收。源码/编译选项不变，实际生效环境指纹按该策略绑定；准备与冷编译仍另列，不将缓存收益混入warm运行收益。不更改系统环境或停止共享daemon。

仓内Cargo env默认同时用于直接启动的子命令与环境指纹，避免同一测试经Cargo/Python入口得到不同的默认线程栈。889新增集合suite及mn-collections-worker独立准备接线，标量mn-worker保持原端口。

正式矩阵的外部负载检测区分编译/测试与常驻服务：构建、测试、未知Cargo命令及其不明子进程仍阻止采样；只有显式cargo run的绝对*-back manifest、唯一同名后端、仅loopback监听、连续两次服务树身份和命令稳定，才进入背景CPU核算。观测间隔须在0到90秒之间，所有服务实际PID去重后的CPU增量上限固定为单核的1%，最小记账容差为Windows一次1/64秒刻度；超过上限、身份/命令变化、监听丢失、新编译/测试均为干扰，整轮作废。每次CPU前后值、间隔、上限、身份、监听与busy原因保留在campaign；这只定义计量背景，不改变原性能门值、样本数或测试oracle。不停止其他服务，不把一次观测当空闲证明。

单轮结果保留发现/执行Job的CPU和内存：CPU加计Python runner且不重复加meter CPU；内存报告顺序子进程树Job峰值最大值，不含共享Python runner内存。长例测量队列须逐个单原子、源绑定及理由/历史证据，标AWAITING_HUMAN；原测试保持启用，普通模块门不能据此通过，条件不适用另列。
