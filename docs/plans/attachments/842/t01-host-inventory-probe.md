# PLAN-842 T-01 宿主登记探针

基线 lang `ecc78701124fe6e0ab6f0e262148a1d853ba2af5`、UI `82217b7592855cef033fc34c86b070b74ce7ce57`、Man `03e82527946761c01d4ecf379c6b169ae4bcc94b`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。执行入口为外部 `ui-dialect-consumer`（默认 feature 全 UI）真实 VM，而非直接调用某计数器。

在 `test/fixtures/modularity/ui-dialect-consumer/tests/plan842_inventory_probe.rs` 临时加入以下测试，执行 `cargo test --locked --offline --manifest-path test/fixtures/modularity/ui-dialect-consumer/Cargo.toml --test plan842_inventory_probe -- --test-threads=1`：

```rust
use auto_lang::vm::host_capabilities::{clear_process_default_frame_provider, process_default_frame_provider};
#[test]
fn direct_sdk_reexport_installs_ui_default() {
    clear_process_default_frame_provider();
    let (_, out, _) = auto_lang::run_with_capture_and_bytecode("print(frame.begin_ms())\n").expect("direct SDK execution");
    assert_eq!(out.lines().collect::<Vec<_>>(), vec!["0"]);
    assert!(process_default_frame_provider().is_some(), "UI default not installed on direct SDK path");
}
```

原样 1/1 成功；将 `crates/auto-lang/src/lib.rs` 仅有的 `inventory::submit!(auto_lang_core::UiCompatInstaller(plan814_ui_compat_install));` 临时注释后，同一命令 exit 101、失败断言 `UI default not installed on direct SDK path`。原始文件逐字节恢复并删除临时测试；此探针证明该无包装的 SDK 再导出当前依赖 inventory，不能凭普通 `run_with_capture` 包装的成功判断兜底无消费者。正式实现必须迁移该类直达路径并用永久用例复测。

独立 no-default `native-provider-consumer` 的 `provider_vm` 6/6：高层缺席和独立 VM 缺席均 0/0，显式 A/B 分别 17/31 和 43/59，两活实例保持各自 provider 快照；锁 SHA `3f7d4c5c8c0a0dd0df26312a3292f504d344dff302ae2585262f0a6548c01fc8`。另一个 UI 全 feature 临时 probe 验证 `run_with_capture` 空槽安装 provider，显式 A/B 输出同上。尚未覆盖所有宿主能力、线程及关闭路径。
