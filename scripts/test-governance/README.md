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
python scripts/test-governance/measure_modules.py --selection <selection.json> --output <fresh-local-directory>
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
