//! PLAN-843 结构拆分：HTTP 服务 scope 职责（原样搬移）。
use super::*;

pub struct RequestScope {
    pub id: u64,
    pub conn_id: u64,
    /// PLAN-781 T-22（R06）：所属服务代际——排水截止按服务归属查询，
    /// 旧服务的截止不污染同进程其他/后续实例。
    pub server_gen: u64,
    pub state: std::sync::atomic::AtomicU8,
    /// PLAN-730 T-05：deadline 升级 RwLock——上传期限切换需透过 Arc 写
    ///（extend_scope_deadline 只增不减；读方短暂陈旧方向安全）。
    pub deadline: std::sync::RwLock<std::time::Instant>,
    /// 生命期许可（scope 终结时释放；SSE 流由桥侧 FrameStream Drop 代持）。
    pub permit: std::sync::Mutex<Option<tokio::sync::OwnedSemaphorePermit>>,
    /// 取消信号（桥 reply 等待 select 臂）。
    pub cancel_notify: tokio::sync::Notify,
    /// PLAN-707 T-06（D-7）：请求资源组——handler 段内打开的上游流 id。
    /// finalize_scope（完成/取消/断连/关闭）逐一流收口（abort 生产者 +
    /// 释放队列），保证组内资源不越过请求生命期。
    pub resources: std::sync::Mutex<Vec<u64>>,
    /// PLAN-727 T-05：有类型的传输资源组——与流 id 分开登记，finalize 时
    /// 取消传输并出 VM 注册表（transfer id 不与 HTTPStream id 混淆清理）。
    pub transfer_resources: std::sync::Mutex<Vec<u64>>,
    /// PLAN-729 T-04：文件响应描述符组——finalize 时移除闲置描述符
    /// （handler 未返回的构造防注册表无界增长）。
    pub file_response_resources: std::sync::Mutex<Vec<u64>>,
    /// PLAN-730 T-02：上传资源组（注入能力/会话/收据句柄混用同一 id 空间
    /// 的三张表按 id 幂等收口——take 已消费的 no-op）。finalize 时取消在途
    /// 接收、清理 staged、释放未消费 body 能力。
    pub upload_resources: std::sync::Mutex<Vec<u64>>,
    /// PLAN-730 T-05：请求起点（bridge 入队时刻）——上传总期限
    /// （started_at + total_timeout）切换的锚点。
    pub started_at: std::time::Instant,
    /// PLAN-736 T-05/AC-05：请求观测事件（恰一次终态；finalize_scope 发射）。
    pub event: std::sync::Mutex<Option<crate::http_service_observability::PendingRequestEvent>>,
    /// PLAN-730 T-05：deadline 变更通道（上传 receive 启动时延展 30s→总
    /// 期限；桥 reply 等待循环 watch 重臂——修"select 保留旧捕获值"）。
    /// 只延展（extend）不缩短；与 `deadline` 字段同步写（读方短暂陈旧
    /// 方向安全：只会更早到期判定由 watch 修正）。
    pub deadline_tx: tokio::sync::watch::Sender<std::time::Instant>,
}

impl RequestScope {
    /// 终态判定（DONE/CANCELLED）。
    pub(crate) fn terminal(&self) -> bool {
        matches!(
            self.state.load(std::sync::atomic::Ordering::SeqCst),
            SCOPE_DONE | SCOPE_CANCELLED
        )
    }
}

impl crate::vm::host_capabilities::request_context::RequestContext for RequestScope {
    fn register_resource(
        &self,
        kind: crate::vm::host_capabilities::request_context::RequestResourceKind,
        id: u64,
    ) {
        use crate::vm::host_capabilities::request_context::RequestResourceKind;
        let group = match kind {
            RequestResourceKind::Stream => &self.resources,
            RequestResourceKind::Transfer => &self.transfer_resources,
            RequestResourceKind::FileResponse => &self.file_response_resources,
            RequestResourceKind::Upload => &self.upload_resources,
        };
        if let Ok(mut resources) = group.lock() {
            if !self.terminal() {
                resources.push(id);
                return;
            }
        }
        // Terminal state is checked under the same group lock used by
        // finalization. A late native constructor cannot escape cleanup.
        VmHttpBridge::finalize_resources(kind, &[id]);
    }

    fn cancelled(&self) -> bool {
        if !scope_usable(self) {
            return true;
        }
        SERVER_DRAIN_CUTOFFS
            .lock()
            .ok()
            .and_then(|cutoffs| cutoffs.get(&self.server_gen).copied())
            .is_some_and(|deadline| std::time::Instant::now() >= deadline)
    }

    fn extend_deadline(&self, deadline: std::time::Instant) {
        extend_scope_deadline(self.id, deadline);
    }
}

pub(super) fn life_permits() -> &'static std::sync::Arc<tokio::sync::Semaphore> {
    LIFE_PERMITS.get_or_init(|| {
        let cap = TEST_PERMIT_CAPACITY
            .lock()
            .unwrap()
            .take()
            .unwrap_or_else(|| {
                super::super::http_transport::TransportConfig::from_env().queue_capacity
            });
        std::sync::Arc::new(tokio::sync::Semaphore::new(cap))
    })
}

/// 桥侧（net 线程）：入队前建 scope（许可随 scope 持有，Queued）。
/// 许可不可得（生命期总上限满）→ None（桥回 503，零分配零排队）。
pub fn create_scope(
    conn_id: u64,
    deadline: std::time::Instant,
    server_gen: u64,
) -> Option<std::sync::Arc<RequestScope>> {
    create_scope_with_permits(conn_id, deadline, server_gen, life_permits())
}

/// The serving instance supplies its lifetime budget. The legacy public
/// factory above remains for existing test/embedding callers, but network
/// services must never share its process-default permit pool.
pub(crate) fn create_scope_with_permits(
    conn_id: u64,
    deadline: std::time::Instant,
    server_gen: u64,
    permits: &std::sync::Arc<tokio::sync::Semaphore>,
) -> Option<std::sync::Arc<RequestScope>> {
    let Ok(permit) = permits.clone().try_acquire_owned() else {
        return None;
    };
    let id = SCOPE_ID_GEN.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let (deadline_tx, _deadline_rx) = tokio::sync::watch::channel(deadline);
    let deadline = std::sync::RwLock::new(deadline);
    let scope = std::sync::Arc::new(RequestScope {
        id,
        conn_id,
        server_gen,
        state: std::sync::atomic::AtomicU8::new(SCOPE_QUEUED),
        deadline,
        permit: std::sync::Mutex::new(Some(permit)),
        cancel_notify: tokio::sync::Notify::new(),
        resources: std::sync::Mutex::new(Vec::new()),
        transfer_resources: std::sync::Mutex::new(Vec::new()),
        file_response_resources: std::sync::Mutex::new(Vec::new()),
        upload_resources: std::sync::Mutex::new(Vec::new()),
        started_at: std::time::Instant::now(),
        event: std::sync::Mutex::new(None),
        deadline_tx,
    });
    if let Ok(mut map) = REQUEST_SCOPES.lock() {
        map.insert(id, scope.clone());
    }
    Some(scope)
}

/// scope 状态迁移（幂等；终态不可逆）。
fn scope_transition(scope: &RequestScope, to: u8) {
    let current = scope.state.load(std::sync::atomic::Ordering::SeqCst);
    if current < SCOPE_DONE {
        scope.state.store(to, std::sync::atomic::Ordering::SeqCst);
    }
}

/// scope 幂等终结（完成/取消共用）：释放许可 + 移除登记 + 发取消信号
/// （等它的桥 select 臂与 owner 唤醒臂消费）。
pub(super) fn finalize_scope(scope: &RequestScope, to: u8) {
    scope_transition(scope, to);
    // PLAN-707 T-06（D-7）：组收口先行——请求内打开的全部上游流终结
    //（abort 生产者 + 出表 + 通知等待者）。「等待可取消 ≠ 关闭登记」：
    // 这里是关闭登记面；生产者实际停止由 stream_cancel 的 abort 承载。
    let group: Vec<u64> = scope
        .resources
        .lock()
        .map(|mut r| std::mem::take(&mut *r))
        .unwrap_or_default();
    for stream_id in group {
        crate::vm::ffi::http_stream::stream_cancel(stream_id);
    }
    // PLAN-727 T-05：传输组收口——取消 + 出 VM 注册表（scope deadline/
    // 断连/shutdown 级联取消的传输臂）。
    let transfers: Vec<u64> = scope
        .transfer_resources
        .lock()
        .map(|mut r| std::mem::take(&mut *r))
        .unwrap_or_default();
    VmHttpBridge::finalize_transfers(&transfers);
    // PLAN-729 T-04：文件描述符组收口（幂等；编组已取出的 no-op）。
    let descriptors: Vec<u64> = scope
        .file_response_resources
        .lock()
        .map(|mut r| std::mem::take(&mut *r))
        .unwrap_or_default();
    VmHttpBridge::finalize_file_responses(&descriptors);
    // PLAN-730 T-02：上传资源组收口（幂等；取消在途接收 + 清理 staged +
    /// 释放未消费 body 能力 + 移除闲置收据）。
    let uploads: Vec<u64> = scope
        .upload_resources
        .lock()
        .map(|mut r| std::mem::take(&mut *r))
        .unwrap_or_default();
    VmHttpBridge::finalize_uploads(&uploads);
    scope.permit.lock().unwrap().take(); // 释放生命期许可（幂等）
    if let Ok(mut map) = REQUEST_SCOPES.lock() {
        map.remove(&scope.id);
    }
    scope.cancel_notify.notify_waiters();
    VmHttpBridge::notify_completion();
    // PLAN-736 T-05/AC-05：请求终态事件恰一次（完成/取消单点收敛；
    // 重复 cancel/drop 双路径由事件自身的原子闸防双计）。
    if let Ok(mut slot) = scope.event.lock() {
        if let Some(ev) = slot.take() {
            if to == SCOPE_CANCELLED {
                ev.cancel();
            } else {
                ev.complete();
            }
        }
    }
}

/// 取消（桥超时/连接终结/关闭）：幂等；首次迁移返回 true。
pub fn cancel_scope(scope: &RequestScope) -> bool {
    if scope.terminal() {
        return false;
    }
    finalize_scope(scope, SCOPE_CANCELLED);
    true
}

/// scope 是否仍可执行（owner 派发/恢复前的失效检查）：
/// 已取消 / 已终结 / 已过 deadline → false（失效请求不调用业务函数）。
pub fn scope_usable(scope: &RequestScope) -> bool {
    if scope.terminal() {
        return false;
    }
    std::time::Instant::now() <= *scope.deadline.read().unwrap()
}

/// 段执行期间绑定 scope 的 RAII 守卫（嵌套安全：保存/恢复外层值）。
pub(crate) type ScopeGuard = crate::vm::host_capabilities::request_context::RequestContextGuard;

/// 资源基线探针（测试）：存活 scope 数。
// PLAN-814: 跨 crate 测试支持面（上层测试引用）
pub fn live_scope_count() -> usize {
    REQUEST_SCOPES.lock().map(|m| m.len()).unwrap_or(0)
}

#[cfg(test)]
mod request_context_tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    struct UnreadBody(Arc<AtomicUsize>, Arc<AtomicUsize>);
    impl futures::Stream for UnreadBody {
        type Item = std::io::Result<Vec<u8>>;
        fn poll_next(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            self.0.fetch_add(1, Ordering::SeqCst);
            std::task::Poll::Pending
        }
    }
    impl Drop for UnreadBody {
        fn drop(&mut self) {
            self.1.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn host_context_restores_parent_and_reclaims_late_native_uploads() {
        let polls = Arc::new(AtomicUsize::new(0));
        let drops = Arc::new(AtomicUsize::new(0));
        let insert = || {
            VmHttpBridge::insert_upload_request(a2r_std::http::upload_request_from_parts(
                "POST",
                "/owned",
                vec![],
                Box::pin(UnreadBody(polls.clone(), drops.clone())),
            ))
        };
        let permits = Arc::new(tokio::sync::Semaphore::new(2));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let outer = create_scope_with_permits(8901, deadline, 0, &permits).unwrap();
        let inner = create_scope_with_permits(8902, deadline, 0, &permits).unwrap();
        {
            let _outer = bind_current_scope(outer.id);
            let first = insert();
            {
                let _inner = bind_current_scope(inner.id);
                let second = insert();
                assert_eq!(*inner.upload_resources.lock().unwrap(), [second]);
            }
            let third = insert();
            assert_eq!(*outer.upload_resources.lock().unwrap(), [first, third]);
            assert!(!current_request_cancelled());
            cancel_scope(&outer);
            assert!(current_request_cancelled());
            let late = insert();
            assert!(
                !crate::vm::ffi::http_upload::VM_UPLOAD_REQUESTS
                    .lock()
                    .unwrap()
                    .contains_key(&late),
                "native resource registered after terminal must close immediately"
            );
            assert_eq!(drops.load(Ordering::SeqCst), 3);
        }
        assert!(!current_request_cancelled());
        cancel_scope(&inner);
        assert_eq!(drops.load(Ordering::SeqCst), 4);
        assert_eq!(polls.load(Ordering::SeqCst), 0);
        assert_eq!(permits.available_permits(), 2);
    }
}

pub(super) fn placeholder_ctx() -> DispatchCtx {
    DispatchCtx {
        req_method: String::new(),
        req_path: String::new(),
        request_id: String::new(),
        body: String::new(),
        content_type: String::new(),
        content_type_raw: String::new(),
        cookie_header: String::new(),
        auth_header: String::new(),
        multipart_json: None,
        route: RouteMatch {
            fn_name: String::new(),
            path_params: Vec::new(),
            query_params: Vec::new(),
        },
        axum_route: None,
        started: std::time::Instant::now(),
        middleware_names: Vec::new(),
        request_info: String::new(),
        handler_task_id: None,
        scope_id: 0,
        upload_pending: None,
        legacy_parts: None,
        legacy_fields_json: String::new(),
        legacy_stored: Vec::new(),
    }
}

pub(super) fn placeholder_reply() -> ApiReply {
    ApiReply::Full {
        status: 200,
        headers: Vec::new(),
        body: ApiBody::Text(Vec::new()),
    }
}

/// 按返回 future 终态编组（Ready → 终值；Failed/缺失 → null；Pending →
/// 继续等）。
pub(super) fn probe_return_future(
    vm: &std::rc::Rc<AutoVM>,
    p: &mut ParkedRequest,
    fid: u32,
) -> ParkedResume {
    let Some(fut) = vm.futures.get(&fid) else {
        let reply = final_value_reply(vm, &p.ctx, None);
        vm.remove_task(p.task_id);
        return ParkedResume::Reply(reply);
    };
    if fut.read().unwrap().state == crate::vm::engine::FutureState::Pending {
        p.wait = ParkedWait::Future(fid);
        p.stage = ParkStage::AwaitReturnFuture;
        return ParkedResume::StillParked;
    }
    let value = fut.read().unwrap().result.clone();
    let reply = final_value_reply(vm, &p.ctx, value);
    vm.remove_task(p.task_id);
    ParkedResume::Reply(reply)
}

/// 废弃一条失效 parked 请求：live-op 回收（HttpRequest 等待）、任务清理、
/// 503 终态（reply_rx 已在桥侧超时关闭时发送静默失败——幂等）。
pub(super) fn abort_parked_request(vm: &std::rc::Rc<AutoVM>, p: &mut ParkedRequest) {
    if let ParkedWait::HttpRequest(req_id) = &p.wait {
        crate::vm::ffi::stdlib::drop_async_result(*req_id);
    }
    // PLAN-730 T-07：废弃请求的 legacy provisional 文件删除（取消/超时）；
    // LegacyStore 期的迟到落盘结果由 presence 守卫丢弃 + 结果表条目清除。
    cleanup_legacy_files(&p.ctx.legacy_stored);
    // PLAN-707 T-06（D-7）：废弃请求的任务持有的流一并取消（首段 open
    // 后 park 的形态）。
    if let Some(t) = vm.tasks.get(&p.task_id) {
        let owned = t
            .try_lock()
            .ok()
            .map(|mut t| std::mem::take(&mut t.owned_stream_ids));
        if let Some(ids) = owned {
            for stream_id in ids {
                crate::vm::ffi::http_stream::stream_cancel(stream_id);
            }
        }
    }
    vm.remove_task(p.task_id);
    if let Some(tx) = p.reply_tx.take() {
        let _ = tx.send(ApiReply::Full {
            status: 503,
            headers: cors_json_headers(&p.ctx.request_id),
            body: ApiBody::Text(br#"{"error":"request cancelled"}"#.to_vec()),
        });
    }
}

pub(super) struct SseIteratorCleanup {
    pub(super) vm: std::rc::Rc<crate::vm::engine::AutoVM>,
    pub(super) iterator_id: u32,
}

pub(super) fn cleanup_sse_iterator(vm: &crate::vm::engine::AutoVM, iterator_id: u32) {
    // PLAN-698 F-1（review #1）: AsyncHttpStream 句柄一并回收——rx drop
    // 使转发线程的 `blocking_send` 失败退出（bus.subscribe 的转发线程
    // 否则在订阅端断连后永生：broadcast 静态永活，无自然终点）。既有
    // http/io 流线程本就自然终止，此处回收同时缩小其句柄累积面。
    // Generator 仍回收其 VM task（原语义零变化）。
    match vm.iterators.remove(&iterator_id) {
        Some((_, crate::vm::engine::Iterator::Generator(state))) => {
            if let Some(task_id) = state.task_id {
                // PLAN-707 T-06（D-7）：generator 体内建立的上游流随任务
                // 回收（首次 pull 发生在 SSE serve 循环，scope 守卫已退出，
                // 资源组登记靠 task.owned_stream_ids 第二线承载）。
                if let Some(t) = vm.tasks.get(&task_id) {
                    let owned = t
                        .try_lock()
                        .ok()
                        .map(|mut t| std::mem::take(&mut t.owned_stream_ids));
                    if let Some(ids) = owned {
                        for stream_id in ids {
                            crate::vm::ffi::http_stream::stream_cancel(stream_id);
                        }
                    }
                }
                vm.remove_task(task_id);
            }
        }
        Some((_, crate::vm::engine::Iterator::AsyncHttpStream(a))) => {
            VmHttpBridge::close_legacy_stream(a.stream_id);
            // PLAN-707 T-06：流句柄出表 → 泵 tx.send 失败 → stream_cancel
            //（generator 内建立的上游连接随 SSE 输出终结一并收口）。
        }
        Some((_, crate::vm::engine::Iterator::HttpStream(h))) => {
            // PLAN-707 T-06：raw 流迭代器清理=真取消（连接关闭）。
            crate::vm::ffi::http_stream::stream_cancel(h.stream_handle);
        }
        Some((_, _)) | None => {}
    }
}

pub(super) fn sse_generator_wake_deadline(
    vm: &crate::vm::engine::AutoVM,
    iterator_id: u32,
) -> Option<std::time::Instant> {
    let task_id = match vm.iterators.get(&iterator_id) {
        Some(state) => match &*state {
            crate::vm::engine::Iterator::Generator(generator) => generator.task_id?,
            _ => return None,
        },
        None => return None,
    };
    vm.tasks
        .get(&task_id)
        .and_then(|task| task.try_lock().ok().and_then(|task| task.wake_time))
}

pub(super) fn wake_sse_generator(vm: &crate::vm::engine::AutoVM, iterator_id: u32) {
    let task_id = match vm.iterators.get(&iterator_id) {
        Some(state) => match &*state {
            crate::vm::engine::Iterator::Generator(generator) => generator.task_id,
            _ => None,
        },
        None => None,
    };
    if let Some(task_id) = task_id {
        if let Some(task) = vm.tasks.get(&task_id) {
            if let Ok(mut task) = task.try_lock() {
                if task
                    .wake_time
                    .is_some_and(|deadline| deadline <= std::time::Instant::now())
                {
                    task.wake_time = None;
                    task.status = crate::vm::task::TaskStatus::Ready;
                }
            }
        }
    }
}

/// PLAN-707 T-05（D-6）：读取 generator 任务的等待凭据（cooperative sleep
/// 之外的凭据化等待源）——外部流 id 或外部 future id。
pub(super) fn generator_wait_credential(
    vm: &crate::vm::engine::AutoVM,
    iterator_id: u32,
) -> GeneratorWait {
    use crate::vm::engine::Iterator;
    let task_id = match vm.iterators.get(&iterator_id) {
        Some(state) => match &*state {
            Iterator::Generator(generator) => generator.task_id,
            _ => None,
        },
        None => None,
    };
    let Some(task_id) = task_id else {
        return GeneratorWait::None;
    };
    let Some(task) = vm.tasks.get(&task_id) else {
        return GeneratorWait::None;
    };
    let Ok(task) = task.try_lock() else {
        return GeneratorWait::None;
    };
    if let Some(stream_id) = task.waiting_http_stream_id {
        return GeneratorWait::Stream(stream_id);
    }
    if let Some(stream_id) = task.waiting_sse_stream_id {
        return GeneratorWait::LegacyStream(stream_id);
    }
    if let Some(fid) = task.waiting_future_id {
        return GeneratorWait::Future(fid);
    }
    GeneratorWait::None
}

pub(super) enum GeneratorWait {
    None,
    Stream(u64),
    LegacyStream(u64),
    Future(u32),
}

/// 测试访问器：generator 产出值 == 期望字符串（str 堆索引解码）。
// PLAN-814: 跨 crate 测试支持面（上层测试引用）
pub fn sse_value_is(vm: &crate::vm::engine::AutoVM, nv: auto_val::NanoValue, expect: &str) -> bool {
    if auto_val::is_string(nv) {
        vm.get_string(auto_val::decode_string(nv) as u32)
            .map(|b| String::from_utf8_lossy(&b) == expect)
            .unwrap_or(false)
    } else {
        false
    }
}

impl Drop for SseIteratorCleanup {
    fn drop(&mut self) {
        cleanup_sse_iterator(&self.vm, self.iterator_id);
    }
}
