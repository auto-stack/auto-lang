//! Headless local binding for the registered Gallery 013 Todo service.
//! The host supplies Realm identity. Each active generation owns a single
//! checked AV snapshot on its worker thread; requests and replies carry no
//! AV references across that boundary.

mod av;

use crate::{parse_app_uri, Registry, RegistryApp};
use av::Owner;
use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub const TODO_APP: &str = "auto.demo.todo";
pub const TODO_SERVICE: &str = "todo";
pub const TODO_VERSION: &str = "1.0.0";
pub const MAX_INSTANCES: usize = 2;
pub const MAX_QUEUED: usize = 8;
pub const MAX_TEXT_BYTES: usize = 4096;
pub const MAX_TODOS: usize = 256;
pub const MAX_REQUEST_TIME: Duration = Duration::from_secs(2);

/// A Realm already resolved by the trusted host.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RealmId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Generation(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestId {
    pub realm: RealmId,
    pub generation: Generation,
    pub lease: u64,
    pub request: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Argument {
    Int(i32),
    Text(String),
    Bool(bool),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Todo {
    pub id: i32,
    pub text: String,
    pub done: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    List(Vec<Todo>),
    Created(Todo),
    Updated(Option<Todo>),
    Flag(bool),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServiceError {
    Unauthorized,
    UnknownOperation,
    WrongArguments,
    Budget(&'static str),
    Closed,
    Closing,
    StaleGeneration,
    Cancelled { may_have_committed: bool },
    TimedOut { may_have_committed: bool },
    Initialization(String),
    Execution(String),
    Cleanup(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Completion {
    pub id: RequestId,
    pub result: Result<Reply, ServiceError>,
    /// Real AV invocation sequence, present only if this request was dispatched.
    pub execution: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inventory {
    pub leases: usize,
    pub workers: usize,
    pub owners: usize,
    pub queued: usize,
    pub active: usize,
    pub links: u64,
    pub initializations: u64,
    pub executions: u64,
    pub suppressed: u64,
    pub last_package_hash: Option<String>,
}

#[derive(Clone, Copy)]
enum Operation {
    List,
    Create,
    Update,
    Delete,
    Clear,
}

impl Operation {
    const ALL: [Self; 5] = [
        Self::List,
        Self::Create,
        Self::Update,
        Self::Delete,
        Self::Clear,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::List => "list_todos",
            Self::Create => "create_todo",
            Self::Update => "update_todo",
            Self::Delete => "delete_todo",
            Self::Clear => "remove_completed",
        }
    }

    fn admit(name: &str, args: &[Argument]) -> Result<Self, ServiceError> {
        let op = Self::ALL
            .into_iter()
            .find(|op| op.name() == name)
            .ok_or(ServiceError::UnknownOperation)?;
        use Argument::*;
        match (op, args) {
            (Self::List, []) | (Self::Delete, [Int(_)]) | (Self::Clear, [Bool(_)]) => {}
            (Self::Create, [Text(text)]) | (Self::Update, [Int(_), Text(text), Bool(_)]) => {
                if text.len() > MAX_TEXT_BYTES {
                    return Err(ServiceError::Budget("todo.text-bytes"));
                }
            }
            _ => return Err(ServiceError::WrongArguments),
        }
        Ok(op)
    }
}

/// One trusted registration and its in-process activation table.
/// The registration is fixed to (013 app, todo service, deployment version),
/// so the remaining instance-key coordinate here is the host Realm.
#[derive(Clone)]
pub struct ServiceBinding(Arc<Shared>);

impl ServiceBinding {
    pub fn new(registry: Registry, store_root: PathBuf) -> Self {
        Self(Arc::new(Shared {
            registry,
            store_root,
            changed: Condvar::new(),
            state: Mutex::new(State::default()),
            #[cfg(test)]
            controls: Controls::default(),
        }))
    }

    /// Registers a holder. Source reads, linking and initialization are lazy.
    pub fn open(
        &self,
        realm: RealmId,
        uri: &str,
        service: &str,
        version: &str,
    ) -> Result<Lease, ServiceError> {
        let parsed = parse_app_uri(uri).map_err(|_| ServiceError::Unauthorized)?;
        let Some(app) = self.0.registry.lookup(&parsed.identity) else {
            return Err(ServiceError::Unauthorized);
        };
        if parsed.identity != TODO_APP
            || service != TODO_SERVICE
            || version != TODO_VERSION
            || app.version != version
            || app.package != "todo"
            || app.entry != "api"
            || !app.capabilities.is_empty()
            || app.sources.len() != 2
            || !app.sources.iter().any(|file| file == "api.at")
            || !app.sources.iter().any(|file| file == "db.at")
        {
            return Err(ServiceError::Unauthorized);
        }
        let id = next_id();
        self.0.state.lock().unwrap().leases.insert(id, realm);
        Ok(Lease {
            shared: self.0.clone(),
            realm,
            id,
        })
    }

    pub fn inventory(&self) -> Inventory {
        let mut state = self.0.state.lock().unwrap();
        state.reap();
        state.inventory()
    }

    /// Waits for actual thread joins. An exhausted wait has a visible failure.
    pub fn wait_released(&self, timeout: Duration) -> Result<Inventory, ServiceError> {
        let start = Instant::now();
        let mut state = self.0.state.lock().unwrap();
        loop {
            state.reap();
            if let Some(error) = &state.cleanup_error {
                return Err(ServiceError::Cleanup(error.clone()));
            }
            if state.instances.is_empty() && state.leases.is_empty() {
                return Ok(state.inventory());
            }
            let remaining = timeout.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                return Err(ServiceError::TimedOut {
                    may_have_committed: false,
                });
            }
            // The worker can notify just before the OS reports it finished.
            state = self
                .0
                .changed
                .wait_timeout(state, remaining.min(Duration::from_millis(5)))
                .unwrap()
                .0;
        }
    }
}

pub struct Lease {
    shared: Arc<Shared>,
    realm: RealmId,
    id: u64,
}

impl Lease {
    /// Submit without blocking on service initialization or execution. Pass a
    /// previous generation only if the caller intends to address that exact instance.
    pub fn submit(
        &self,
        name: &str,
        arguments: Vec<Argument>,
        expected: Option<Generation>,
        timeout: Duration,
    ) -> Result<Ticket, ServiceError> {
        let operation = Operation::admit(name, &arguments)?;
        let timeout = timeout.min(MAX_REQUEST_TIME);
        if timeout.is_zero() {
            return Err(ServiceError::TimedOut {
                may_have_committed: false,
            });
        }
        let deadline = Instant::now() + timeout;
        let shared = &self.shared;
        let mut state = shared.state.lock().unwrap();
        state.reap();
        if let Some(error) = &state.cleanup_error {
            return Err(ServiceError::Cleanup(error.clone()));
        }
        if state.leases.get(&self.id) != Some(&self.realm) {
            return Err(ServiceError::Closed);
        }
        if let Some(expected) = expected {
            if state.instances.get(&self.realm).map(|i| i.generation) != Some(expected) {
                return Err(ServiceError::StaleGeneration);
            }
        }
        if !state.instances.contains_key(&self.realm) {
            if state.instances.len() == MAX_INSTANCES {
                return Err(ServiceError::Budget("todo.instances"));
            }
            let generation = Generation(next_id());
            let realm = self.realm;
            let worker_shared = shared.clone();
            let worker = thread::Builder::new()
                .name("ase-local-todo".into())
                .spawn(move || serve(worker_shared, realm, generation))
                .map_err(|error| ServiceError::Initialization(error.to_string()))?;
            state
                .instances
                .insert(realm, Instance::new(generation, worker));
        }
        let instance = state.instances.get_mut(&self.realm).expect("active Realm");
        if instance.stopping {
            return Err(ServiceError::Closing);
        }
        if instance.queue.len() == MAX_QUEUED {
            return Err(ServiceError::Budget("todo.queue"));
        }
        let id = RequestId {
            realm: self.realm,
            generation: instance.generation,
            lease: self.id,
            request: next_id(),
        };
        let (sender, receiver) = mpsc::channel();
        instance.queue.push_back((
            Job {
                id,
                operation,
                arguments,
                deadline,
            },
            sender,
        ));
        shared.changed.notify_all();
        Ok(Ticket {
            id,
            receiver: Some(receiver),
            deadline,
            shared: shared.clone(),
        })
    }

    pub fn close(self) {
        drop(self);
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        let shared = &self.shared;
        let mut state = shared.state.lock().unwrap();
        state.leases.remove(&self.id);
        let last = !state.leases.values().any(|realm| *realm == self.realm);
        if let Some(instance) = state.instances.get_mut(&self.realm) {
            instance.queue.retain(|(job, sender)| {
                if job.id.lease == self.id || last {
                    send(sender, job.id, Err(ServiceError::Closed), None);
                    false
                } else {
                    true
                }
            });
            if let Some(active) = &mut instance.active {
                if active.id.lease == self.id || last {
                    active.reject(ServiceError::Closed);
                }
            }
            if last {
                instance.stopping = true;
            }
        }
        shared.changed.notify_all();
    }
}

pub struct Ticket {
    id: RequestId,
    deadline: Instant,
    receiver: Option<Receiver<Completion>>,
    shared: Arc<Shared>,
}

impl Ticket {
    pub fn id(&self) -> RequestId {
        self.id
    }

    /// Takes the terminal result once. A pending or already drained ticket returns None.
    pub fn poll(&mut self) -> Option<Completion> {
        let receiver = self.receiver.as_ref()?;
        if Instant::now() >= self.deadline {
            self.shared.cancel(self.id, End::Timeout);
        }
        let completion = match receiver.try_recv() {
            Ok(completion) => completion,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => self.disconnected(),
        };
        self.receiver = None;
        Some(completion)
    }

    /// Host-side wait option. UI code can use poll to keep the caller nonblocking.
    pub fn wait(mut self) -> Completion {
        let Some(receiver) = self.receiver.take() else {
            return self.disconnected();
        };
        let remaining = self.deadline.saturating_duration_since(Instant::now());
        if let Ok(reply) = receiver.recv_timeout(remaining) {
            return reply;
        }
        self.shared.cancel(self.id, End::Timeout);
        receiver.try_recv().unwrap_or_else(|_| self.disconnected())
    }

    pub fn cancel(&self) {
        self.shared.cancel(self.id, End::Cancel);
    }

    fn disconnected(&self) -> Completion {
        Completion {
            id: self.id,
            result: Err(ServiceError::Execution("completion owner exited".into())),
            execution: None,
        }
    }
}

#[derive(Clone, Copy)]
enum End {
    Cancel,
    Timeout,
}

impl End {
    fn reason(self, may_have_committed: bool) -> ServiceError {
        match self {
            Self::Cancel => ServiceError::Cancelled { may_have_committed },
            Self::Timeout => ServiceError::TimedOut { may_have_committed },
        }
    }
}

struct Job {
    id: RequestId,
    operation: Operation,
    arguments: Vec<Argument>,
    deadline: Instant,
}

fn send(
    sender: &Sender<Completion>,
    id: RequestId,
    result: Result<Reply, ServiceError>,
    execution: Option<u64>,
) {
    let _ = sender.send(Completion {
        id,
        result,
        execution,
    });
}

struct Active {
    id: RequestId,
    cancel: Arc<AtomicBool>,
    sender: Option<Sender<Completion>>,
    started: bool,
}

impl Active {
    fn reject(&mut self, error: ServiceError) {
        self.cancel.store(true, Ordering::Relaxed);
        if let Some(sender) = self.sender.take() {
            send(&sender, self.id, Err(error), None);
        }
    }
}

struct Instance {
    generation: Generation,
    queue: VecDeque<(Job, Sender<Completion>)>,
    active: Option<Active>,
    stopping: bool,
    owner: bool,
    worker: JoinHandle<()>,
}

impl Instance {
    fn new(generation: Generation, worker: JoinHandle<()>) -> Self {
        Self {
            generation,
            queue: VecDeque::new(),
            active: None,
            stopping: false,
            owner: false,
            worker,
        }
    }

    fn stop(&mut self, error: ServiceError) {
        self.stopping = true;
        for (job, sender) in self.queue.drain(..) {
            send(&sender, job.id, Err(error.clone()), None);
        }
    }
}

#[derive(Default)]
struct State {
    leases: BTreeMap<u64, RealmId>,
    instances: BTreeMap<RealmId, Instance>,
    facts: Inventory,
    cleanup_error: Option<String>,
}

impl State {
    fn inventory(&self) -> Inventory {
        Inventory {
            leases: self.leases.len(),
            workers: self.instances.len(),
            owners: self
                .instances
                .values()
                .filter(|instance| instance.owner)
                .count(),
            queued: self
                .instances
                .values()
                .map(|instance| instance.queue.len())
                .sum(),
            active: self
                .instances
                .values()
                .filter(|instance| instance.active.is_some())
                .count(),
            ..self.facts.clone()
        }
    }

    fn reap(&mut self) {
        let finished: Vec<_> = self
            .instances
            .iter()
            .filter_map(|(realm, instance)| instance.worker.is_finished().then_some(*realm))
            .collect();
        for realm in finished {
            let mut instance = self.instances.remove(&realm).expect("finished Realm");
            if instance.active.is_some() || !instance.queue.is_empty() {
                let error = ServiceError::Execution(
                    "service worker exited before completing requests".into(),
                );
                instance.stop(error.clone());
                if let Some(active) = &mut instance.active {
                    active.reject(error);
                }
            }
            if instance.worker.join().is_err() {
                self.cleanup_error = Some("Todo worker panicked".into());
            }
        }
    }

    fn complete(
        &mut self,
        id: RequestId,
        result: Result<Reply, ServiceError>,
        execution: Option<u64>,
    ) {
        let active = self
            .instances
            .get_mut(&id.realm)
            .filter(|instance| instance.generation == id.generation)
            .filter(|instance| {
                instance
                    .active
                    .as_ref()
                    .is_some_and(|active| active.id == id)
            })
            .and_then(|instance| instance.active.take());
        if let Some(sender) = active.and_then(|active| active.sender) {
            if self.leases.get(&id.lease) == Some(&id.realm) {
                send(&sender, id, result, execution);
                return;
            }
        }
        self.facts.suppressed += 1;
    }
}

struct Shared {
    registry: Registry,
    store_root: PathBuf,
    changed: Condvar,
    state: Mutex<State>,
    #[cfg(test)]
    controls: Controls,
}

impl Shared {
    fn cancel(&self, id: RequestId, end: End) {
        let mut state = self.state.lock().unwrap();
        let Some(instance) = state
            .instances
            .get_mut(&id.realm)
            .filter(|instance| instance.generation == id.generation)
        else {
            return;
        };
        if let Some(index) = instance.queue.iter().position(|(job, _)| job.id == id) {
            let (_, sender) = instance.queue.remove(index).expect("queued request");
            send(&sender, id, Err(end.reason(false)), None);
        } else if let Some(active) = instance.active.as_mut().filter(|active| active.id == id) {
            active.reject(end.reason(active.started));
        }
        self.changed.notify_all();
    }

    fn take(&self, realm: RealmId) -> Option<Job> {
        let mut state = self.state.lock().unwrap();
        loop {
            let instance = state.instances.get_mut(&realm).expect("worker owns Realm");
            if instance.stopping {
                return None;
            }
            if let Some((job, sender)) = instance.queue.pop_front() {
                // Keep the reply channel in the active slot. The worker can
                // compute and commit while cancellation withdraws that slot.
                instance.active = Some(Active {
                    id: job.id,
                    cancel: Arc::new(AtomicBool::new(false)),
                    sender: Some(sender),
                    started: false,
                });
                return Some(job);
            }
            state = self.changed.wait(state).unwrap();
        }
    }
}

fn next_id() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

struct StoreDir {
    path: PathBuf,
    shared: Arc<Shared>,
}

impl StoreDir {
    fn create(shared: &Arc<Shared>, generation: Generation) -> Result<Self, ServiceError> {
        let root = &shared.store_root;
        std::fs::create_dir_all(root).map_err(store_error)?;
        let path = root.join(format!("todo-{}-{}", std::process::id(), generation.0));
        // Exclusive ownership: if this path already exists, leave it alone.
        std::fs::create_dir(&path).map_err(store_error)?;
        Ok(Self {
            path,
            shared: shared.clone(),
        })
    }
}

impl Drop for StoreDir {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.path) {
            self.shared.state.lock().unwrap().cleanup_error =
                Some(format!("{}: {error}", self.path.display()));
        }
    }
}

fn store_error(error: std::io::Error) -> ServiceError {
    ServiceError::Initialization(format!("Todo AV store: {error}"))
}

fn serve(shared: Arc<Shared>, realm: RealmId, generation: Generation) {
    let mut store = None;
    let mut owner = None;
    while let Some(job) = shared.take(realm) {
        #[cfg(test)]
        shared.controls.before(&job);
        let (cancel, withdrawn) = {
            let mut state = shared.state.lock().unwrap();
            let active = state
                .instances
                .get_mut(&realm)
                .unwrap()
                .active
                .as_mut()
                .unwrap();
            (active.cancel.clone(), active.sender.is_none())
        };
        if withdrawn || Instant::now() >= job.deadline {
            let mut state = shared.state.lock().unwrap();
            state.complete(job.id, Err(End::Timeout.reason(false)), None);
            continue;
        }
        if owner.is_none() {
            let preparation = StoreDir::create(&shared, generation).and_then(|directory| {
                let result = load(&shared, &directory.path);
                store = Some(directory);
                result
            });
            let mut state = shared.state.lock().unwrap();
            match preparation {
                Ok(prepared) => {
                    state.instances.get_mut(&realm).unwrap().owner = true;
                    state.facts.links += 1;
                    state.facts.last_package_hash = Some(prepared.source_hash.clone());
                    owner = Some(prepared);
                }
                Err(error) => {
                    state.instances.get_mut(&realm).unwrap().stop(error.clone());
                    state.complete(job.id, Err(error), None);
                    break;
                }
            }
        }
        let owner = owner.as_ref().expect("linked owner");
        let previously_initialized = owner.initialized();
        {
            let mut state = shared.state.lock().unwrap();
            let active = state
                .instances
                .get_mut(&realm)
                .unwrap()
                .active
                .as_mut()
                .unwrap();
            if active.sender.is_none() || Instant::now() >= job.deadline {
                state.complete(job.id, Err(End::Timeout.reason(false)), None);
                continue;
            }
            active.started = true;
        }
        let max_steps = av::MAX_STEPS;
        #[cfg(test)]
        let max_steps = shared.controls.steps(&job).unwrap_or(max_steps);
        let (result, execution) = owner.call(
            job.operation,
            &job.arguments,
            &cancel,
            job.deadline,
            max_steps,
        );
        #[cfg(test)]
        shared.controls.after(&job);
        let initialized = owner.initialized();
        let mut state = shared.state.lock().unwrap();
        if initialized && !previously_initialized {
            state.facts.initializations += 1;
        }
        if execution.is_some() {
            state.facts.executions += 1;
            if !initialized {
                state
                    .instances
                    .get_mut(&realm)
                    .unwrap()
                    .stop(ServiceError::Initialization(
                        "Todo module initialization failed".into(),
                    ));
            }
        }
        let result = if Instant::now() >= job.deadline {
            Err(End::Timeout.reason(result.is_ok()))
        } else {
            result
        };
        state.complete(job.id, result, execution);
    }
    drop(owner);
    drop(store);
    shared
        .state
        .lock()
        .unwrap()
        .instances
        .get_mut(&realm)
        .unwrap()
        .owner = false;
    shared.changed.notify_all();
}

fn load(shared: &Shared, path: &Path) -> Result<Owner, ServiceError> {
    let app: &RegistryApp = shared
        .registry
        .lookup(TODO_APP)
        .expect("admitted registration");
    Owner::load(&shared.registry, app, path)
}

#[cfg(test)]
#[derive(Default)]
struct Controls {
    before: Mutex<Option<Box<dyn Fn(&Job) + Send>>>,
    after: Mutex<Option<Box<dyn Fn(&Job) + Send>>>,
    steps: Mutex<Option<Box<dyn Fn(&Job) -> Option<u64> + Send>>>,
}

#[cfg(test)]
impl Controls {
    fn before(&self, job: &Job) {
        if let Some(callback) = self.before.lock().unwrap().as_ref() {
            callback(job);
        }
    }

    fn after(&self, job: &Job) {
        if let Some(callback) = self.after.lock().unwrap().as_ref() {
            callback(job);
        }
    }

    fn steps(&self, job: &Job) -> Option<u64> {
        self.steps
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|callback| callback(job))
    }
}

#[cfg(test)]
mod tests;
