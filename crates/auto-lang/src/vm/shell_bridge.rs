// Plan 060(api.at 契约归一):merged 模式 shell 执行请求队列 —— 传输件。
//
// 链路:shell.at 的 run_command 经 native `auto.shell.exec_submit` 把请求压到
// 这里;iced renderer 的 merged_exec_loop 从这里取走执行(spawn + 流式,
// 零 shell 语义)。取代旧的 __pending_command renderer 桥 —— 那条链绕过
// api.at 契约、语义长在 renderer,已随 Plan 060 退役。
//
// 队列放 vm 模块(不 feature-gate):native shim(vm/native.rs,总是编译)与
// renderer(ui-iced)都能引用,依赖方向合法(ui → vm)。HTTP 模式不触达:
// AUTO_BACKEND 非空时 codegen 把 api 调用编译为 fetch,本队列无人生产。

/// 请求类型:进程执行(外部命令)或直发结果(.at 侧已算好语义的 builtin,
/// 如 ls/cd/pwd —— Plan 060 M2:语义在 .at 提交侧完成,传输层零语义)。
#[derive(Debug, Clone)]
pub enum ShellExecKind {
    Process,
    Result,
}

/// 一条待执行命令(merged 传输的请求单元)。
#[derive(Debug, Clone)]
pub struct ShellExecRequest {
    pub kind: ShellExecKind,
    pub block_id: i64,
    pub cmd: String,
    pub cwd: String,
    /// Result 变体:完整的 command_result payload JSON(执行线程直发)。
    pub result_json: String,
}

lazy_static::lazy_static! {
    static ref QUEUE: std::sync::Mutex<std::collections::VecDeque<ShellExecRequest>> =
        std::sync::Mutex::new(std::collections::VecDeque::new());
}

/// 压入一条执行请求(shell_exec_submit native 调用,来自 shell.at)。
pub fn submit(req: ShellExecRequest) {
    QUEUE.lock().unwrap().push_back(req);
}

/// 取出一条执行请求(merged_exec_loop 轮询);队列空返回 None。
pub fn pop() -> Option<ShellExecRequest> {
    QUEUE.lock().unwrap().pop_front()
}
