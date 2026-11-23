use super::*;

const TIMEOUT: Duration = MAX_REQUEST_TIME;

fn binding() -> ServiceBinding {
    let ui = std::env::var_os("AUTO_UI_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../auto-ui"));
    let root = std::env::var_os("ASE_TODO_TEST_ARTIFACTS")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    ServiceBinding::new(
        Registry {
            dir: ui.join("examples/ui/013-todo"),
            apps: vec![RegistryApp {
                identity: TODO_APP.into(),
                version: TODO_VERSION.into(),
                package: "todo".into(),
                root: "src/back".into(),
                entry: "api".into(),
                sources: vec!["api.at".into(), "db.at".into()],
                capabilities: vec![],
            }],
        },
        root,
    )
}

fn lease(binding: &ServiceBinding) -> Lease {
    binding
        .open(
            RealmId(1),
            "app://auto.demo.todo/",
            TODO_SERVICE,
            TODO_VERSION,
        )
        .unwrap()
}

fn create(lease: &Lease, text: &str) -> Ticket {
    lease
        .submit(
            "create_todo",
            vec![Argument::Text(text.into())],
            None,
            TIMEOUT,
        )
        .unwrap()
}

fn expect_list(completion: Completion) -> Vec<Todo> {
    let Ok(Reply::List(list)) = completion.result else {
        panic!("expected actual Todo list, got {completion:?}");
    };
    list
}

#[test]
fn queue_refuses_ninth_waiter_and_withdraws_pending_work() {
    let binding = binding();
    let a = lease(&binding);
    let b = lease(&binding);
    let (entered, entry) = mpsc::channel();
    let (release, paused) = mpsc::channel();
    let first = Mutex::new(Some(paused));
    *binding.0.controls.before.lock().unwrap() = Some(Box::new(move |_| {
        let mut first = first.lock().unwrap();
        if let Some(paused) = first.take() {
            entered.send(()).unwrap();
            paused.recv_timeout(TIMEOUT).unwrap();
        }
    }));
    let active = create(&a, "retired active");
    entry.recv_timeout(TIMEOUT).unwrap();
    let cancelled = create(&b, "cancelled queued");
    let expired = b
        .submit(
            "create_todo",
            vec![Argument::Text("expired queued".into())],
            None,
            Duration::from_millis(30),
        )
        .unwrap();
    let closed = create(&a, "closed queued");
    let queue = (0..MAX_QUEUED - 3)
        .map(|n| create(&b, &format!("retained {n}")))
        .collect::<Vec<_>>();
    assert_eq!(binding.inventory().active, 1);
    assert_eq!(binding.inventory().queued, MAX_QUEUED);
    assert!(matches!(
        b.submit("list_todos", vec![], None, TIMEOUT),
        Err(ServiceError::Budget("todo.queue"))
    ));
    cancelled.cancel();
    assert_eq!(
        cancelled.wait().result,
        Err(ServiceError::Cancelled {
            may_have_committed: false
        })
    );
    assert_eq!(
        expired.wait().result,
        Err(ServiceError::TimedOut {
            may_have_committed: false
        })
    );
    a.close();
    assert_eq!(active.wait().result, Err(ServiceError::Closed));
    assert_eq!(closed.wait().result, Err(ServiceError::Closed));
    assert_eq!(binding.inventory().queued, MAX_QUEUED - 3);
    release.send(()).unwrap();
    for (n, ticket) in queue.into_iter().enumerate() {
        let completion = ticket.wait();
        assert_eq!(
            completion.result,
            Ok(Reply::Created(Todo {
                id: 4 + n as i32,
                text: format!("retained {n}"),
                done: false
            }))
        );
    }
    let reply = b
        .submit("list_todos", vec![], None, TIMEOUT)
        .unwrap()
        .wait();
    assert_eq!(expect_list(reply).len(), 4 + MAX_QUEUED - 3);
    b.close();
    let result = binding.wait_released(TIMEOUT).unwrap();
    assert_eq!(
        (
            result.links,
            result.initializations,
            result.executions,
            result.suppressed
        ),
        (1, 1, 6, 1)
    );
}

#[test]
fn stale_and_late_completions_cannot_publish_but_success_stays_committed() {
    let binding = binding();
    let a = lease(&binding);
    let b = lease(&binding);
    let (entered, entry) = mpsc::channel();
    let (release, paused) = mpsc::channel();
    let once = Mutex::new(Some(paused));
    *binding.0.controls.after.lock().unwrap() = Some(Box::new(move |job| {
        if let Some(paused) = once.lock().unwrap().take() {
            entered.send(job.id).unwrap();
            paused.recv_timeout(TIMEOUT).unwrap();
        }
    }));
    let mut ticket = create(&a, "committed");
    let id = entry.recv_timeout(TIMEOUT).unwrap();
    assert_eq!(id, ticket.id());
    {
        let mut state = binding.0.state.lock().unwrap();
        for stale in [
            RequestId {
                generation: Generation(0),
                ..id
            },
            RequestId { request: 0, ..id },
            RequestId { lease: b.id, ..id },
        ] {
            state.complete(stale, Ok(Reply::Flag(true)), Some(0));
        }
    }
    assert!(ticket.poll().is_none());
    ticket.cancel();
    assert_eq!(
        ticket.wait().result,
        Err(ServiceError::Cancelled {
            may_have_committed: true
        })
    );
    a.close();
    release.send(()).unwrap();
    let list = b
        .submit("list_todos", vec![], Some(id.generation), TIMEOUT)
        .unwrap()
        .wait();
    assert_eq!(list.id.generation, id.generation);
    assert_eq!(expect_list(list)[4].text, "committed");
    b.close();
    let result = binding.wait_released(TIMEOUT).unwrap();
    assert_eq!(
        (
            result.links,
            result.initializations,
            result.executions,
            result.suppressed
        ),
        (1, 1, 2, 4)
    );
}

#[test]
fn actual_av_fuel_refusal_does_not_publish_and_explicit_retry_works() {
    let binding = binding();
    let lease = lease(&binding);
    let once = AtomicBool::new(true);
    *binding.0.controls.steps.lock().unwrap() = Some(Box::new(move |job| {
        if matches!(job.operation, Operation::Create) && once.swap(false, Ordering::Relaxed) {
            Some(10)
        } else {
            None
        }
    }));
    let first = lease
        .submit("list_todos", vec![], None, TIMEOUT)
        .unwrap()
        .wait();
    assert_eq!(expect_list(first).len(), 4);
    let rejected = create(&lease, "budgeted").wait();
    assert_eq!(
        rejected.result,
        Err(ServiceError::Budget("av.resource.instr-steps"))
    );
    assert!(rejected.execution.is_some());
    let retry = create(&lease, "retry").wait();
    assert_eq!(retry.id.generation, rejected.id.generation);
    assert_eq!(
        retry.result,
        Ok(Reply::Created(Todo {
            id: 4,
            text: "retry".into(),
            done: false
        }))
    );
    lease.close();
    let result = binding.wait_released(TIMEOUT).unwrap();
    assert_eq!(
        (result.links, result.initializations, result.executions),
        (1, 1, 3)
    );
}

#[test]
fn last_release_does_not_report_a_paused_worker_as_reaped() {
    let binding = binding();
    let lease = lease(&binding);
    let (entered, entry) = mpsc::channel();
    let (release, paused) = mpsc::channel();
    *binding.0.controls.after.lock().unwrap() = Some(Box::new(move |_| {
        entered.send(()).unwrap();
        paused.recv_timeout(TIMEOUT).unwrap();
    }));
    let pending = create(&lease, "committed then closed");
    entry.recv_timeout(TIMEOUT).unwrap();
    lease.close();
    assert_eq!(pending.wait().result, Err(ServiceError::Closed));
    let inventory = binding.inventory();
    assert_eq!(
        (inventory.leases, inventory.workers, inventory.owners),
        (0, 1, 1)
    );
    assert_eq!(
        binding.wait_released(Duration::from_millis(10)),
        Err(ServiceError::TimedOut {
            may_have_committed: false
        }),
    );
    release.send(()).unwrap();
    let settled = binding.wait_released(TIMEOUT).unwrap();
    assert_eq!(
        (
            settled.workers,
            settled.owners,
            settled.active,
            settled.suppressed
        ),
        (0, 0, 0, 1)
    );
}
