//! Host-owned request lifetime context for VM native operations.
//! No framework route, transport, observer or policy implementation lives here.
use std::{cell::RefCell, sync::Arc, time::Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestResourceKind {
    Stream,
    Transfer,
    FileResponse,
    Upload,
}

/// The serving host owns admission, deadlines and terminal cleanup. The VM
/// only reports created resource identities and checks cooperative cancellation.
pub trait RequestContext: Send + Sync {
    fn register_resource(&self, kind: RequestResourceKind, id: u64);
    fn cancelled(&self) -> bool;
    fn extend_deadline(&self, deadline: Instant);
}

thread_local! {
    static CURRENT: RefCell<Option<Arc<dyn RequestContext>>> = const { RefCell::new(None) };
}

pub struct RequestContextGuard {
    previous: Option<Arc<dyn RequestContext>>,
    // The Arc payload may cross threads; the TLS restoration guard may not.
    _local: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl Drop for RequestContextGuard {
    fn drop(&mut self) {
        CURRENT.with(|current| *current.borrow_mut() = self.previous.take());
    }
}

/// Nesting and unwind restore the previous local execution context. This does
/// not move a VM or bind a context across another future's polling interval.
pub fn bind_request_context(context: Option<Arc<dyn RequestContext>>) -> RequestContextGuard {
    RequestContextGuard {
        previous: CURRENT.with(|current| current.replace(context)),
        _local: std::marker::PhantomData,
    }
}

pub fn current_request_context() -> Option<Arc<dyn RequestContext>> {
    CURRENT.with(|current| current.borrow().clone())
}

pub fn register_request_resource(kind: RequestResourceKind, id: u64) {
    if let Some(context) = current_request_context() {
        context.register_resource(kind, id);
    }
}

pub fn current_request_cancelled() -> bool {
    current_request_context().is_some_and(|context| context.cancelled())
}
