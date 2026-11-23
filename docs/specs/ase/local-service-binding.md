# ASE 本地 Todo 服务绑定

PLAN-1111 为 Windows x64 的可信宿主提供无窗口的 013 Todo 本地服务。宿主从
`Registry` 注册 `app://auto.demo.todo/`、服务 `todo`、部署版本 `1.0.0` 和真实
AutoUI `examples/ui/013-todo/src/back/{api,db}.at`。注册项的能力表为空，源码沿
[ASE 源根规则](m0-headless-loading.md)读取。宿主提供已解析的 `RealmId`。

## 请求与回值

在 `ase_m0::todo_service::ServiceBinding` 上 `open` 获取独立 `Lease`。请求沿受检的
`compile_package_product_auto → PackageAvTarget → execute_program_entry_args` 派发。
每次激活检查公开声明、参数和回值类型、Todo 字段、全局列表 cell 与宿主导入。
文本以 UTF-8 借用 `str` 实参进入当前 AV arena，回值转换成可独立持有的 Rust 值。

| 已注册操作 | 实参 | 成功结果 |
|---|---|---|
| `list_todos` | 无 | `List(Vec<Todo>)`，含空列表 |
| `create_todo` | `str text` | `Created(Todo)` |
| `update_todo` | `int id, str text, bool done` | `Updated(Option<Todo>)` |
| `delete_todo` | `int id` | `Flag(bool)` |
| `remove_completed` | `bool done` | `Flag(bool)` |
| `set_all_todos`（可选） | `bool done` | `Flag(bool)` |

`Todo` 持有 `id: i32`、`text: String` 和 `done: bool`。缺失记录仍由原业务
返回 `Updated(None)` 或 `Flag(false)`。参数错型、未知操作、未授权、超限、
关闭、旧代次、初始化、执行以及清理错误保留独立状态。

## 激活、限额与提交

打开租约只登记 holder。首次有效请求激活一条工作线程和一个受检 AV 根。
同 Realm、app、服务和版本的租约共享这个代次。其它 Realm 有独立状态。
同一代次链接和模块初始化各一次，后续每个请求在同一根派发一次目标函数。
最多两个实例，每实例一个在途任务及八个待执任务。文本上限为 4096 个
UTF-8 字节，活列表最多 256 条。请求期限最多 2 秒，AV 使用最多 100 万
步和最多 500 毫秒的 wall 配额，同时执行 AV 固有帧和 slot 额度。

每份 `Ticket` 带 Realm、租约、请求号和代次，可轮询终态。传入旧代次会被拒绝。
待执取消和过期撤下请求。执行中的取消将信号送到既有 AV 检查点。
业务更新在受检全局状态事务中完成。载荷解码、预算或语言执行失败时回滚，
新增操作使用 ACE 的列表缓冲隔离，其他写入构建新的业务列表。成功提交可先于取消或关闭到达，过期或失去租约的
完成不会再投递。带 `may_have_committed: true` 的取消或超时要求调用方刷新
列表再决定操作。

关闭一个租约仅撤销该租约的投递，剩余租约继续使用当前根。最后租约释放
后撤待执、通知在途任务、销毁根并清理该代制品目录。`wait_released` 以
已 join 线程为终态。初始化失败回收该代，后续显式请求可重新激活。
默认内存模式最后释放后从原 Auto 种子开始，磁盘仅暂存受检代码制品。
显式耐久模式按受信 Realm 授予专用数据根，从最后一次原子提交恢复。
`set_all_todos` 在声明该受检入口的部署中用一次业务提交更新全部完成状态。

## 验证边界

永久验证通过真实 013 源检查五个业务操作、源变化、燃料拒绝、队列和重开。
20 轮开关后工作线程、根、队列与租约归零。无窗口本地执行不分配 demo
HTTP listener 或 demo 子进程。历史 HTTP 对照、Windows 进程和监听账、
输入及提交身份见 [PLAN-1111](../../plans/archive/1111-ase-local-todo-service.md)。
原生前端消费和显式数据根见 [ASE 原生 Todo 与本地持久化](native-todo-persistence.md)。
