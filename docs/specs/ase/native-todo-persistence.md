# ASE 原生 Todo 与本地持久化

Windows x64 的 `ase_m0_sessions_window` 通过 `--todo-project <013 根>` 和
`--todo-code-root <目录>` 注册原 Gallery 013，`--evidence <JSON>` 指定关窗收据。
交互启动等待用户关窗。宿主读取原工程 App、TodoList、
TodoStore、类型声明以及后端 `api.at`、`db.at`。受检 AV 的一个挂载持有一份
TodoStore 权威状态。TodoList 的筛选和页签草稿随原生会话保留。

`app://auto.demo.todo/`、`todo` 及 `1.0.0` 是
[本地绑定](local-service-binding.md)的注册身份。原生页签持有一份租约。有效
完成经安装的 `service:todo` 绑定进入同一挂载的受检 handler，匹配运行代次、
租约和请求。新增、编辑、完成切换、单项删除、清除和 ToggleAll 使用原 Auto
业务。写成功后在当前代次重新读取列表。不确定提交先查询列表，输入草稿仍可供
用户核对和处理。

默认模式在内存持有后端状态，最后租约释放后从种子重开。显式
`--todo-data <既存绝对目录>` 授予可信 Realm 一个私有数据根。同一宿主中活动
绑定独占该根。同 Realm 的页签共享已提交的数据，切回时主动查询。隐藏页签
仍可接收有效完成，选中车道仅显示当前会话。关闭页签撤销该租约与原生
挂载，关窗等待所有 worker 实际退出。

`todo.json` 是版本 1 UTF-8 快照，同时提交有序行和 next_id。读写限额为
256 KiB、256 行及每行文本 4096 字节。首次启动无快照时采用原种子。原子
替换成功后发布业务结果，失败后保留旧提交。坏快照、不支持的版本、拒绝授予
和超限具名返回。关闭及进程重开从最后提交恢复列表与 ID。

验收使用一个真窗、两个在途会话、显式根和原 Gallery 013。覆盖服务请求到原生
帧、编辑取消与保存、双页签、原生关闭、进程重启和 Unicode。脚本控制输入与
实际 renderer 使用同一队列。Windows 实测未观察到 demo TCP 监听或子进程。
覆盖与成本账见 [PLAN-1112](../../plans/archive/1112-ase-native-todo-and-persistence.md)。
