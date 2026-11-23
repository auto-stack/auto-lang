use ase_m0::todo_service::*;
use ase_m0::{Registry, RegistryApp};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

const TIMEOUT: Duration = MAX_REQUEST_TIME;
const URI: &str = "app://auto.demo.todo/";

struct Fixture {
    work: PathBuf,
    registry: Registry,
}

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::var_os("ASE_TODO_TEST_ARTIFACTS")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let work = path.join(format!(
            "ase-todo-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir(&work).unwrap();
        let backend = auto_ui().join("examples/ui/013-todo/src/back");
        let project = work.join("back");
        std::fs::create_dir(&project).unwrap();
        let sources = ["api.at", "db.at"];
        for file in sources {
            std::fs::copy(backend.join(file), project.join(file)).unwrap_or_else(|error| {
                panic!(
                    "read actual Gallery backend {}: {error}",
                    backend.join(file).display()
                )
            });
        }
        let registry = Registry {
            dir: work.clone(),
            apps: vec![RegistryApp {
                identity: TODO_APP.into(),
                version: TODO_VERSION.into(),
                package: "todo".into(),
                root: PathBuf::from("back"),
                entry: "api".into(),
                sources: sources.map(str::to_owned).into(),
                capabilities: vec![],
            }],
        };
        Self { work, registry }
    }

    fn binding(&self) -> ServiceBinding {
        ServiceBinding::new(self.registry.clone(), self.work.join("av"))
    }

    #[cfg(all(windows, target_arch = "x86_64"))]
    fn durable(&self, data: &Path) -> ServiceBinding {
        ServiceBinding::with_persistence(
            self.registry.clone(),
            self.work.join("av"),
            RealmId(1),
            data,
        )
        .unwrap()
    }

    fn change(&self, name: &str, f: impl FnOnce(String) -> String) {
        let file = self.work.join("back").join(name);
        let before = std::fs::read_to_string(&file).unwrap();
        let after = f(before.clone());
        assert_ne!(before, after, "the control must change real source");
        std::fs::write(file, after).unwrap();
    }

    fn released(&self, binding: &ServiceBinding) -> Inventory {
        let report = binding.wait_released(TIMEOUT).unwrap();
        assert_eq!(
            (
                report.leases,
                report.owners,
                report.workers,
                report.queued,
                report.active
            ),
            (0, 0, 0, 0, 0)
        );
        assert_eq!(std::fs::read_dir(self.work.join("av")).unwrap().count(), 0);
        report
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.work);
    }
}

fn auto_ui() -> PathBuf {
    std::env::var_os("AUTO_UI_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../auto-ui"))
}

fn lease(binding: &ServiceBinding, realm: u64) -> Lease {
    binding
        .open(RealmId(realm), URI, TODO_SERVICE, TODO_VERSION)
        .unwrap()
}

fn call(lease: &Lease, name: &str, args: Vec<Argument>) -> Completion {
    let ticket = lease.submit(name, args, None, TIMEOUT).unwrap();
    let id = ticket.id();
    let result = ticket.wait();
    assert_eq!(result.id, id);
    result
}

fn list(lease: &Lease) -> (Vec<Todo>, Completion) {
    let result = call(lease, "list_todos", vec![]);
    let Ok(Reply::List(items)) = &result.result else {
        panic!("expected list, got {result:?}");
    };
    (items.clone(), result)
}

fn row(id: i32, text: &str, done: bool) -> Todo {
    Todo {
        id,
        text: text.into(),
        done,
    }
}

#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn durable_snapshot_reopens_with_exact_rows_and_next_id() {
    let fixture = Fixture::new();
    let data = fixture.work.join("private realm");
    std::fs::create_dir(&data).unwrap();
    let binding = fixture.durable(&data);
    assert!(matches!(
        binding.open(RealmId(2), URI, TODO_SERVICE, TODO_VERSION),
        Err(ServiceError::Unauthorized)
    ));
    let a = lease(&binding, 1);
    let b = lease(&binding, 1);
    assert_eq!(list(&a).0.len(), 4);
    assert_eq!(std::fs::read_dir(&data).unwrap().count(), 0);
    let competing = ServiceBinding::with_persistence(
        fixture.registry.clone(),
        fixture.work.join("other-code"),
        RealmId(2),
        &data,
    )
    .unwrap();
    let other = lease(&competing, 2);
    assert_eq!(
        call(&other, "list_todos", vec![]).result,
        Err(ServiceError::Unauthorized)
    );
    other.close();
    assert_eq!(competing.wait_released(TIMEOUT).unwrap().workers, 0);
    let text = "计划 🌱\n\0";
    assert_eq!(
        call(&a, "create_todo", vec![Argument::Text(text.into())]).result,
        Ok(Reply::Created(row(4, text, false)))
    );
    let edit = "编辑后 🌼\n\0";
    assert_eq!(
        call(
            &b,
            "update_todo",
            vec![
                Argument::Int(4),
                Argument::Text(edit.into()),
                Argument::Bool(true)
            ],
        )
        .result,
        Ok(Reply::Updated(Some(row(4, edit, true))))
    );
    assert_eq!(
        call(&a, "delete_todo", vec![Argument::Int(1)]).result,
        Ok(Reply::Flag(true))
    );
    let expected = list(&b).0;
    assert_eq!(expected.last(), Some(&row(4, edit, true)));
    a.close();
    b.close();
    assert_eq!(fixture.released(&binding).durable_writes, 3);
    assert!(data.join("todo.json").is_file());
    assert!(!data.join("todo.json.tmp").exists());

    let reopened = lease(&binding, 1);
    assert_eq!(list(&reopened).0, expected);
    assert_eq!(
        call(&reopened, "remove_completed", vec![Argument::Bool(true)]).result,
        Ok(Reply::Flag(true))
    );
    assert_eq!(
        list(&reopened)
            .0
            .iter()
            .map(|todo| todo.id)
            .collect::<Vec<_>>(),
        [2, 3]
    );
    assert_eq!(
        call(
            &reopened,
            "create_todo",
            vec![Argument::Text("next".into())]
        )
        .result,
        Ok(Reply::Created(row(5, "next", false)))
    );
    reopened.close();
    fixture.released(&binding);
}

#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn durable_snapshot_failed_replace_preserves_disk_rows_and_next_id() {
    use std::os::windows::fs::OpenOptionsExt;

    let fixture = Fixture::new();
    let data = fixture.work.join("private");
    std::fs::create_dir(&data).unwrap();
    let binding = fixture.durable(&data);
    let client = lease(&binding, 1);
    let create =
        |text: &str| call(&client, "create_todo", vec![Argument::Text(text.into())]).result;
    assert_eq!(
        create("retained"),
        Ok(Reply::Created(row(4, "retained", false)))
    );
    let committed = list(&client).0;
    let path = data.join("todo.json");
    let original = std::fs::read(&path).unwrap();

    // Permit existing readers/writers, but deny deletion/replacement.
    let hold = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0x1 | 0x2)
        .open(&path)
        .unwrap();
    assert!(matches!(create("rejected"), Err(ServiceError::Storage(_))));
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert!(!data.join("todo.json.tmp").exists());
    assert_eq!(list(&client).0, committed);
    assert_eq!(binding.inventory().durable_writes, 1);
    drop(hold);

    assert_eq!(
        create("after failure"),
        Ok(Reply::Created(row(5, "after failure", false)))
    );
    client.close();
    fixture.released(&binding);
}

#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn durable_snapshot_rejects_corruption_and_versions_then_recovers() {
    let fixture = Fixture::new();
    let data = fixture.work.join("private");
    std::fs::create_dir(&data).unwrap();
    let binding = fixture.durable(&data);
    let path = data.join("todo.json");
    let version = br#"{"version":2,"next_id":4,"todos":[]}"#;
    let too_large = vec![0u8; 256 * 1024 + 1];

    for (bytes, expected) in [
        (
            b"invalid JSON".as_slice(),
            ServiceError::StorageCorrupt("expected value at line 1 column 1".into()),
        ),
        (version.as_slice(), ServiceError::StorageVersion(2)),
        (
            too_large.as_slice(),
            ServiceError::Budget("todo.snapshot-bytes"),
        ),
    ] {
        std::fs::write(&path, bytes).unwrap();
        let client = lease(&binding, 1);
        assert_eq!(call(&client, "list_todos", vec![]).result, Err(expected));
        client.close();
        fixture.released(&binding);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }

    std::fs::remove_file(&path).unwrap();
    let client = lease(&binding, 1);
    assert_eq!(list(&client).0.len(), 4);
    client.close();
    assert_eq!(fixture.released(&binding).durable_writes, 0);
}

#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn durable_snapshot_expansion_refusal_leaves_a_readable_commit() {
    let fixture = Fixture::new();
    let data = fixture.work.join("private");
    std::fs::create_dir(&data).unwrap();
    let binding = fixture.durable(&data);
    let mut rows: Vec<_> = (0..63)
        .map(|id| row(id, &"a".repeat(4096), false))
        .collect();
    rows.push(row(63, &"b".repeat(500), false));
    let json = serde_json::json!({ "version": 1, "next_id": 64, "todos": rows });
    let original = serde_json::to_vec(&json).unwrap();
    let path = data.join("todo.json");
    std::fs::write(&path, &original).unwrap();
    let client = lease(&binding, 1);
    assert_eq!(list(&client).0, rows);
    rows[63] = row(63, &"c".repeat(500), true);
    assert_eq!(
        call(
            &client,
            "update_todo",
            vec![
                Argument::Int(63),
                Argument::Text(rows[63].text.clone()),
                Argument::Bool(true),
            ],
        )
        .result,
        Ok(Reply::Updated(Some(rows[63].clone())))
    );
    let original = std::fs::read(&path).unwrap();
    let update = call(
        &client,
        "update_todo",
        vec![
            Argument::Int(63),
            Argument::Text("b".repeat(4096)),
            Argument::Bool(true),
        ],
    );
    assert_eq!(
        update.result,
        Err(ServiceError::Budget("todo.snapshot-bytes"))
    );
    assert_eq!(list(&client).0, rows);
    assert_eq!(std::fs::read(&path).unwrap(), original);
    client.close();
    assert_eq!(fixture.released(&binding).durable_writes, 1);
}

#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn durable_snapshot_survives_process_exit_and_detects_a_memory_only_write() {
    use std::process::Command;

    const PHASE: &str = "ASE_TODO_PERSISTENCE_TEST_PHASE";
    const ROOT: &str = "ASE_TODO_PERSISTENCE_TEST_DATA";
    const NAME: &str = "durable_snapshot_survives_process_exit_and_detects_a_memory_only_write";
    const MARKER: &str = "写入跨进程保留 🌱";
    let fixture = Fixture::new();
    match std::env::var(PHASE).as_deref() {
        Ok(phase @ ("write" | "read" | "memory-only")) => {
            let data = PathBuf::from(std::env::var_os(ROOT).unwrap());
            let binding = if phase == "memory-only" {
                fixture.binding()
            } else {
                fixture.durable(&data)
            };
            let client = lease(&binding, 1);
            if phase == "read" {
                assert_eq!(
                    list(&client).0.last(),
                    Some(&row(4, MARKER, false)),
                    "disk reopen did not restore the child writer's row"
                );
            } else {
                assert_eq!(
                    call(&client, "create_todo", vec![Argument::Text(MARKER.into())]).result,
                    Ok(Reply::Created(row(4, MARKER, false)))
                );
            }
            client.close();
            fixture.released(&binding);
        }
        Err(std::env::VarError::NotPresent) => {
            let exe = std::env::current_exe().unwrap();
            let run = |data: &Path, phase: &str| {
                Command::new(&exe)
                    .env(PHASE, phase)
                    .env(ROOT, data)
                    .args(["--exact", NAME, "--nocapture"])
                    .output()
                    .unwrap()
            };
            let verify = |output: std::process::Output| {
                assert!(
                    output.status.success(),
                    "{}\n{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            };
            let data = fixture.work.join("process-data");
            std::fs::create_dir(&data).unwrap();
            verify(run(&data, "write"));
            verify(run(&data, "read"));

            let missing = fixture.work.join("negative-control");
            std::fs::create_dir(&missing).unwrap();
            verify(run(&missing, "memory-only"));
            let failed = run(&missing, "read");
            assert!(!failed.status.success());
            assert!(String::from_utf8_lossy(&failed.stderr)
                .contains("disk reopen did not restore the child writer's row"));
        }
        _ => panic!("invalid child persistence test phase"),
    }
}

#[test]
fn actual_todo_signatures_and_crud() {
    let fixture = Fixture::new();
    let binding = fixture.binding();
    let lease = lease(&binding, 1);
    assert_eq!(binding.inventory().workers, 0);
    assert!(matches!(
        lease.submit("List", vec![], None, TIMEOUT),
        Err(ServiceError::UnknownOperation)
    ));
    assert!(matches!(
        lease.submit("create_todo", vec![Argument::Bool(true)], None, TIMEOUT),
        Err(ServiceError::WrongArguments)
    ));
    assert!(matches!(
        lease.submit(
            "create_todo",
            vec![Argument::Text("😀".repeat(MAX_TEXT_BYTES / 4 + 1))],
            None,
            TIMEOUT
        ),
        Err(ServiceError::Budget("todo.text-bytes"))
    ));
    assert_eq!(binding.inventory().executions, 0);
    assert_eq!(binding.inventory().workers, 0);
    let (items, first) = list(&lease);
    assert_eq!(
        items,
        [
            row(0, "Learn Auto Language", true),
            row(1, "Build a TodoMVC app", false),
            row(2, "Master the a2r transpiler", false),
            row(3, "Write integration tests", false),
        ]
    );
    let created = call(&lease, "create_todo", vec![Argument::Text("中😀".into())]);
    assert_eq!(created.result, Ok(Reply::Created(row(4, "中😀", false))));
    assert!(created.execution > first.execution);
    assert_eq!(created.id.generation, first.id.generation);
    let (items, _) = list(&lease);
    assert_eq!(items.len(), 5);
    assert_eq!(items.last(), Some(&row(4, "中😀", false)));
    let update = |id, text: &str, done| {
        call(
            &lease,
            "update_todo",
            vec![
                Argument::Int(id),
                Argument::Text(text.into()),
                Argument::Bool(done),
            ],
        )
        .result
    };
    let delete = |id| call(&lease, "delete_todo", vec![Argument::Int(id)]).result;
    assert_eq!(
        update(4, "更改😀", true),
        Ok(Reply::Updated(Some(row(4, "更改😀", true))))
    );
    assert_eq!(update(999, "unused", true), Ok(Reply::Updated(None)));
    assert_eq!(delete(999), Ok(Reply::Flag(false)));
    assert_eq!(delete(3), Ok(Reply::Flag(true)));
    assert_eq!(
        call(&lease, "remove_completed", vec![Argument::Bool(true)]).result,
        Ok(Reply::Flag(true))
    );
    assert_eq!(
        list(&lease).0.iter().map(|row| row.id).collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(delete(1), Ok(Reply::Flag(true)));
    assert_eq!(delete(2), Ok(Reply::Flag(true)));
    assert!(list(&lease).0.is_empty());
    let boundary = "😀".repeat(MAX_TEXT_BYTES / 4);
    assert_eq!(
        call(
            &lease,
            "create_todo",
            vec![Argument::Text(boundary.clone())]
        )
        .result,
        Ok(Reply::Created(row(5, &boundary, false))),
    );
    assert!(!binding
        .inventory()
        .last_package_hash
        .as_deref()
        .unwrap()
        .is_empty());
    lease.close();
    let result = fixture.released(&binding);
    assert_eq!(
        (result.links, result.initializations, result.executions),
        (1, 1, 13)
    );
}

#[test]
fn same_realm_shares_other_realm_isolates_and_stale_generation_is_refused() {
    let fixture = Fixture::new();
    let binding = fixture.binding();
    assert!(matches!(
        binding.open(RealmId(1), URI, TODO_SERVICE, "2.0.0"),
        Err(ServiceError::Unauthorized)
    ));
    assert!(matches!(
        binding.open(RealmId(1), "app://auto.demo.a/", TODO_SERVICE, TODO_VERSION),
        Err(ServiceError::Unauthorized)
    ));
    let a = lease(&binding, 1);
    let b = lease(&binding, 1);
    let c = lease(&binding, 2);
    let d = lease(&binding, 3);
    let start = std::sync::Barrier::new(2);
    let (one, two) = std::thread::scope(|threads| {
        let first = threads.spawn(|| {
            start.wait();
            call(&a, "create_todo", vec![Argument::Text("A".into())])
        });
        start.wait();
        let second = call(&b, "create_todo", vec![Argument::Text("B".into())]);
        (first.join().unwrap(), second)
    });
    let from_a = one.id;
    assert_eq!(from_a.generation, two.id.generation);
    assert_ne!(from_a.lease, two.id.lease);
    let (items, _) = list(&b);
    assert_eq!(items.len(), 6);
    assert!(items[4..].iter().any(|row| row.text == "A"));
    assert!(items[4..].iter().any(|row| row.text == "B"));
    assert_eq!(binding.inventory().initializations, 1);
    a.close();
    assert_eq!(list(&b).0.len(), 6);
    assert_eq!(list(&c).0.len(), 4);
    assert!(matches!(
        c.submit("list_todos", vec![], Some(from_a.generation), TIMEOUT),
        Err(ServiceError::StaleGeneration)
    ));
    assert!(matches!(
        d.submit("list_todos", vec![], None, TIMEOUT),
        Err(ServiceError::Budget("todo.instances"))
    ));
    b.close();
    c.close();
    d.close();
    let result = fixture.released(&binding);
    assert_eq!(
        (result.links, result.initializations, result.executions),
        (2, 2, 5)
    );
    let again = lease(&binding, 1);
    assert!(matches!(
        again.submit("list_todos", vec![], Some(from_a.generation), TIMEOUT),
        Err(ServiceError::StaleGeneration)
    ));
    let first = again.submit("list_todos", vec![], None, TIMEOUT).unwrap();
    let second = again.submit("list_todos", vec![], None, TIMEOUT).unwrap();
    assert_eq!(first.id().lease, second.id().lease);
    let again_read = first.wait();
    assert_eq!(again_read.id.generation, second.id().generation);
    let seeds = Ok(Reply::List(vec![
        row(0, "Learn Auto Language", true),
        row(1, "Build a TodoMVC app", false),
        row(2, "Master the a2r transpiler", false),
        row(3, "Write integration tests", false),
    ]));
    assert_eq!(again_read.result, seeds);
    assert_eq!(second.wait().result, seeds);
    assert_ne!(from_a.generation, again_read.id.generation);
    again.close();
    let reopened = fixture.released(&binding);
    assert_eq!(
        (
            reopened.links,
            reopened.initializations,
            reopened.executions
        ),
        (3, 3, 7)
    );
}

#[test]
fn changed_source_changes_results_and_bad_signature_releases_for_retry() {
    let fixture = Fixture::new();
    let binding = fixture.binding();
    let first = lease(&binding, 1);
    let (_, first_read) = list(&first);
    let before = binding.inventory().last_package_hash.unwrap();
    first.close();
    fixture.released(&binding);

    fixture.change("api.at", |s| {
        s.replace("pub fn delete_todo", "fn delete_todo")
    });
    let first = lease(&binding, 1);
    let rejected = call(&first, "list_todos", vec![]);
    assert!(
        matches!(&rejected.result, Err(ServiceError::Initialization(message)) if message.contains("missing public operation delete_todo"))
    );
    assert_eq!(rejected.execution, None);
    first.close();
    assert_eq!(fixture.released(&binding).executions, 1);
    fixture.change("api.at", |s| {
        s.replace("fn delete_todo", "pub fn delete_todo")
    });
    fixture.change("db.at", |s| {
        s.replace("use api: Todo", "use api: Todo, list_todos")
    });
    let cycle_lease = lease(&binding, 1);
    let mixed_cycle = call(&cycle_lease, "list_todos", vec![]);
    assert!(matches!(
        &mixed_cycle.result,
        Err(ServiceError::Initialization(message)) if message.contains("source.module-cycle"),
    ));
    assert_eq!(mixed_cycle.execution, None);
    cycle_lease.close();
    fixture.released(&binding);
    fixture.change("db.at", |s| {
        s.replace("use api: Todo, list_todos", "use api: Todo")
    });
    fixture.change("db.at", |s| {
        s.replace("Learn Auto Language", "Source changed 中😀")
    });
    let first = lease(&binding, 1);
    let (items, latest) = list(&first);
    assert_eq!(items[0], row(0, "Source changed 中😀", true));
    assert_ne!(latest.id.generation, first_read.id.generation);
    assert_ne!(binding.inventory().last_package_hash.unwrap(), before);
    first.close();
    fixture.released(&binding);
}

#[test]
fn twenty_lifetimes_release_actual_workers_roots_and_store() {
    let fixture = Fixture::new();
    let binding = fixture.binding();
    let mut previous = None;
    for round in 1..=20 {
        let lease = lease(&binding, 1);
        let (items, completed) = list(&lease);
        assert_eq!(items.len(), 4);
        assert_ne!(Some(completed.id.generation), previous);
        previous = Some(completed.id.generation);
        assert_eq!(binding.inventory().owners, 1);
        lease.close();
        let closed = fixture.released(&binding);
        assert_eq!(
            (closed.links, closed.initializations, closed.executions),
            (round, round, round)
        );
    }
}

#[test]
fn full_list_refuses_before_dispatch_and_allows_create_after_delete() {
    let fixture = Fixture::new();
    fixture.change("db.at", |mut s| {
        let start = s.find(".new([").unwrap() + ".new([".len();
        let end = s[start..].find("])").unwrap() + start;
        let seeds = (0..MAX_TODOS)
            .map(|id| format!("Todo {{ id: {id}, text: \"seed\", done: false }}"))
            .collect::<Vec<_>>()
            .join(",\n");
        s.replace_range(start..end, &seeds);
        s.replace(
            "var nextid int = 4",
            &format!("var nextid int = {MAX_TODOS}"),
        )
    });
    let binding = fixture.binding();
    let lease = lease(&binding, 1);
    assert_eq!(list(&lease).0.len(), MAX_TODOS);
    let create = || call(&lease, "create_todo", vec![Argument::Text("next".into())]);
    let rejected = create();
    assert_eq!(
        rejected.result,
        Err(ServiceError::Budget("todo.list-items"))
    );
    assert_eq!(rejected.execution, None);
    assert_eq!(
        call(&lease, "delete_todo", vec![Argument::Int(0)]).result,
        Ok(Reply::Flag(true))
    );
    assert_eq!(
        create().result,
        Ok(Reply::Created(row(MAX_TODOS as i32, "next", false)))
    );
    assert_eq!(list(&lease).0.len(), MAX_TODOS);
    lease.close();
    fixture.released(&binding);
}

#[test]
fn trap_after_global_writes_rolls_back_both_list_and_next_id() {
    let fixture = Fixture::new();
    fixture.change("db.at", |s| {
        s.replacen(
            "    return todo\n}",
            "    if text == \"trap\" {\n        let impossible = 1 / (nextid - nextid)\n        nextid = impossible\n    }\n    return todo\n}",
            1,
        )
    });
    let binding = fixture.binding();
    let lease = lease(&binding, 1);
    let (before, read) = list(&lease);
    let failed = call(&lease, "create_todo", vec![Argument::Text("trap".into())]);
    assert_eq!(failed.id.generation, read.id.generation);
    assert!(failed.execution.is_some());
    assert!(
        matches!(&failed.result, Err(ServiceError::Execution(message)) if message.contains("DivByZero"))
    );
    assert_eq!(list(&lease).0, before);
    assert_eq!(
        call(&lease, "create_todo", vec![Argument::Text("retry".into())]).result,
        Ok(Reply::Created(row(4, "retry", false))),
    );
    lease.close();
    fixture.released(&binding);
}
