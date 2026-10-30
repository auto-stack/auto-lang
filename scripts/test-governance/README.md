# 按模块运行测试

先用 `cargo tm --list` 找到叶模块；每条义务仅属于一叶，命令总是执行完整叶及实际消费者、
集成闭包的去重并集。多个模块直接放在同一命令中，无需叠加旧档位。

```powershell
cargo tm --list
cargo tm infra-meter/unittest/test-native-meter/NativeMeter/direct/001
cargo tm --changes target/changed-paths.txt --select-only --out target/test-selection
cargo tm --changes target/changed-paths.txt --out target/test-execution
cargo tm --test-name test_legacy_navigation --select-only
```

变更文件清单每行一个仓内路径，支持删除的路径。无映射时保守扩大；不以零选择冒绿。
`--test-name`/`--test-regex` 只是从原测试名导航到完整模块，不能筛掉叶内其余用例。
`--select-only` 只产选择收据；执行结果在 `execution/results.json`，保留原始命令、条件、
实际原子比较和失败。PowerShell调用方复用 `invoke-modules.ps1`，检查收据及必需原身份，
不要再用控制台PASS字样推断完成。原case的单项诊断重放另走报告内实际replay argv。

t/tv/tt/tb/th为截至2026-11-30的兼容导航，仍有旧包含关系，不是叶模块。
新计划/新脚本使用tm；兼容滤串选择完整叶，不能视为单case执行。
`tf --batch` 为显式周期调度，消费 `periodic-module-schedule.json` 的recipe清单生成真实
叶选择并去重执行一次。原UI/Man owner腿、手工/里程碑条件与单实例纪律保留。
没有必需的“全部测试”名称。

完整模块成本测量消费显式源绑定选择文件：

```powershell
python scripts/test-governance/measure_modules.py --catalog <selection.json内的catalog_path> --selection <selection.json> --output <fresh-local-directory>
```

至少三次完整warm，P90目标60秒；预构建/冷态/整命令分列。可拆超标继续拆，不可拆原正式
采样/规模/生命周期oracle具名登记慢模块，原断言和样本不减。原始日志/JSON留计划本地附件，
只提交审查摘要及长期工具/配置。目录由真实运行时发现产生；过期时tm自动刷新，不能手改
分片或通过重生 `legacy-contract.json` / `legacy-wrapper-contract.json` 接受漏项。
旧金样和VM缓存的原包装/输入关联有独立冻结守卫，执行须保留每项原始终态与源/期望身份；
原包装退出成功或刷新目录都不能抹去缺失的子项。

批量性能见 `measure_pilot.py` 与冻结 `measurement-contract.json`。`test-case-pilot` 的固定
15例白名单默认B k=10，原单例重放默认A；其他recipe继续独立，C仅显式D3观察。
不要将本批改善推断为全生态改善。详见[规范](../../docs/specs/auto-test-case/module-selection.md)。

已测慢聚合的细分规则在partition-policy.json中，所有原用例继续归属唯一叶；规则失效或消费端重新聚合会失败。正式成本入口要求原AV/MN 30样本与2000ms门，并拒oracle再生开关。相关环境参数属于绑定内容，不可通过改环境获得假达标。

准备、发现（包括Git/Cargo元数据）和实际执行都使用有界的Windows Job；父进程退出0而后代仍在运行属于基础设施错误，记录实际PID/映像并回收整树，不能据此发布工具就绪或测试通过。临时目录相关TEMP/TMP/TMPDIR参与有效环境指纹。

TestCase在系统短临时目录运行，避免SQLite等原生后端的长输出路径限制；命令仍保留真实runtime argv。退出后先逐文件SHA256校验复制到请求的输出目录，再按目录所有权标记回收临时目录，复制/清理计入模块墙钟。`case_artifacts`直接提供可读报告与复制清单，原始报告/收据字节及其原runtime路径不改。reparse point在遍历前拒绝，不向worktree复制链接，复制失败保留原目录供排查。

原计划的历史写入范围审计绑定其实际可达实施基面和归档交付SHA，不能审查当前计划相对master的无关改动；原命名空间禁区断言保持，当前产品依赖边界继续由实时结构检查负责。

`--refresh`和过期自动刷新写入本次`target/modules-*/runtime-catalog`，不会改写已跟踪目录；选择收据的`catalog_path`指向本次实际目录。发布目录只由计划worktree中的显式发现/分区步骤更新。`--out`必须不存在或为空，防止覆盖历史结果；拒绝reparse祖先。选择涉及TestCase时，先调用原`auto-case audit-output`审查最终输出目录，再写选择收据，原suite/fixture/oracle保护规则保持单源。短运行目录不能绕过最终证据目录的同一审查。

编译准备/发现有一项明确例外：`compiler-auxiliaries.json`钉住当前参考机Microsoft签名MSVC遥测辅助程序的完整路径和SHA256；仅在编译器成功退出、所有残留映像逐一匹配且Job实际回收后active=0时接受准备。原meter错误、PID、映像和回收记录不改。未知映像/错hash/回收未完成仍失败，实际测试负载不启用例外；更换工具版本须重新核验该源码配置。Cargo launcher注入的包标签不算有效配置变更；Cargo home及仓内env默认按实际生效值绑定，显式不同覆盖仍使证据失效。

Windows受计量命令明确禁用RUSTC_WRAPPER/RUSTC_WORKSPACE_WRAPPER，直接运行原rustc；既有外部缓存daemon不在本Job内，不能让它派生编译器而绕过CPU/超时回收。源码/编译选项不变，实际生效环境指纹按该策略绑定；准备与冷编译仍另列，不将缓存收益混入warm运行收益。不更改系统环境或停止共享daemon。
