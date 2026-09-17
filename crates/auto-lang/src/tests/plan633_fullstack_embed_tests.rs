//! PLAN-633: 画廊内嵌全栈 demo 的数据面回归（T-04 blocker 白盒化）。
//!
//! 形态对齐 ui-gallery 实测发射产物：host（use.web 适配器）→ demo widget →
//! store（`use <ns>_api: fns`）→ api（#[api] 端点，`use <ns>_db`）→ db
//! （模块级 `var` 种子 + 循环 `use <ns>_api: Todo`）。模块名用发射后的
//! `<ns>_<mod>` 唯一 stem 形态。
//!
//! Blocker 症状：内嵌语境 store Init 的 `list_todos()` 数据不达根态
//! （todos 恒 []），standalone 同源码种子可见。本测试把该链路锁进
//! `cargo t`：绿 = 数据面修复的验收锚；红 = 白盒调试起点。

#![cfg(feature = "ui-iced")]

use std::path::PathBuf;

struct Fixture {
    _dir: tempfile::TempDir,
    host_path: PathBuf,
}

fn write(dir: &std::path::Path, rel: &str, src: &str) {
    let p = dir.join(rel);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(p, src).unwrap();
}

fn build_embedded(files: &[(&str, &str)]) -> (crate::ui::dynamic::DynamicComponent, PathBuf) {
    let dir = tempfile::TempDir::new().unwrap();
    for (rel, src) in files {
        write(dir.path(), rel, src);
    }
    let host_path = dir.path().join("host.at");
    let path_str = host_path.to_string_lossy().to_string();
    let host_src = std::fs::read_to_string(&host_path).unwrap();
    let comp = crate::build_dynamic_component(&host_src, Some(&path_str))
        .expect("内嵌全栈 demo 宿主编译");
    (comp, host_path)
}

const HOST: &str = r#"
use.web component Demo013 from "demo013.at"

widget App {
    view {
        Demo013 {}
    }
}
"#;

const DEMO: &str = r#"
use d013todo_todo_store: TodoStore
use d013todo_todo_list: TodoList

widget Demo013 {
    model {
        var input str = ""
    }
    view {
        col {
            text f"n=${.TodoStore.active_count}"
            TodoList()
        }
    }
    on {
        .Init -> {
            TodoStore.Init()
        }
        .AddTodo -> {
            TodoStore.AddTodo()
        }
    }
}
"#;

const TODO_LIST: &str = r#"
use d013todo_todo_store: TodoStore

widget TodoList {
    view {
        col {
            for item in .TodoStore.todos {
                text item.text
            }
        }
    }
}
"#;

const STORE: &str = r#"
use d013todo_api: list_todos, create_todo

store TodoStore {
    model {
        var todos []Todo = []
        var active_count int = 0
        var input str = ""
    }
    msg { Init, AddTodo }
    on {
        .Init -> {
            .todos = d013todo_api.list_todos()
            .active_count = 0
            var i int = 0
            while i < .todos.len() {
                if .todos[i].done == false {
                    .active_count = .active_count + 1
                }
                i = i + 1
            }
        }
        .AddTodo -> {
            create_todo("x")
            .todos = d013todo_api.list_todos()
        }
    }
}
"#;

const API: &str = r#"
use d013todo_db

pub type Todo = {
    id: int
    text: str
    done: bool
}

#[api(method = "GET", path = "/api/todos")]
pub fn list_todos() []Todo {
    return d013todo_db.all_todos()
}

#[api(method = "POST", path = "/api/todos")]
pub fn create_todo(text str) Todo {
    return d013todo_db.create_todo(text)
}
"#;

const DB: &str = r#"
use d013todo_api: Todo

var todos List<Todo> = List<Todo>.new([
    Todo { id: 0, text: "Learn Auto Language", done: true },
    Todo { id: 1, text: "Build a TodoMVC app", done: false },
    Todo { id: 2, text: "Master the a2r transpiler", done: false },
    Todo { id: 3, text: "Write integration tests", done: false },
])

var nextid int = 4

pub fn all_todos() []Todo {
    return todos
}

pub fn create_todo(text str) Todo {
    let todo = Todo { id: nextid, text: text, done: false }
    nextid = nextid + 1
    todos.push(todo)
    return todo
}
"#;

/// 数据面锚：内嵌 demo 的 store Init 经 #[api] 调用拿到 db 模块种子。
#[test]
fn f5_embedded_fullstack_api_data_reaches_state() {
    let (mut comp, _keep) = build_embedded(&[
        ("host.at", HOST),
        ("demo013.at", DEMO),
        ("d013todo_todo_list.at", TODO_LIST),
        ("d013todo_todo_store.at", STORE),
        ("d013todo_api.at", API),
        ("d013todo_db.at", DB),
    ]);
    let (view, _, _) = comp.view_with_debug();
    let _rendered = format!("{view:?}");
    let todos = comp
        .read_state_as_vec("todos")
        .expect("store 字段 todos 应为可展开列表(统一根态)");
    assert_eq!(
        todos.len(),
        4,
        "store Init 经 #[api] 应拿到 db 种子 4 条, 实际 {} 条: {todos:?}",
        todos.len()
    );
    let active = comp.read_state("active_count").unwrap();
    assert_eq!(
        active,
        auto_val::Value::Int(3),
        "active_count 应为种子中未完成数 3: {active:?}"
    );
}

/// 写路径锚：AddTodo → create_todo（#[api] POST 形）→ 列表 +1。
#[test]
fn f5_embedded_fullstack_api_write_path() {
    let (mut comp, _keep) = build_embedded(&[
        ("host.at", HOST),
        ("demo013.at", DEMO),
        ("d013todo_todo_list.at", TODO_LIST),
        ("d013todo_todo_store.at", STORE),
        ("d013todo_api.at", API),
        ("d013todo_db.at", DB),
    ]);
    let _ = comp.view_with_debug();
    comp.on_with_input_for("Demo013", "AddTodo", None);
    let todos = comp.read_state_as_vec("todos").unwrap();
    assert_eq!(todos.len(), 5, "create_todo 后列表应 4→5");
}

/// 孙辈嵌套形（真实画廊拓扑）：host → AppViewport（模型态子件）→
/// Demo013Todo。store 调用链经两层子件间接——数据面必须仍达根态。
#[test]
fn f5_embedded_fullstack_grandchild_viewport() {
    let (mut comp, _keep) = build_embedded(&[
        ("host.at", VIEWPORT_HOST),
        ("viewport.vue", VIEWPORT_VUE),
        ("viewport.vm.at", VIEWPORT),
        ("demo013.at", DEMO),
        ("d013todo_todo_list.at", TODO_LIST),
        ("d013todo_todo_store.at", STORE),
        ("d013todo_api.at", API),
        ("d013todo_db.at", DB),
    ]);
    let (view, _, _) = comp.view_with_debug();
    let rendered = format!("{view:?}");
    assert!(
        rendered.contains("n=3"),
        "demo 层视图应读到 store 字段(active_count=3): {rendered:.800}"
    );
    assert!(
        rendered.contains("Learn Auto Language"),
        "TodoList 层视图应渲染 store 列表条目: {rendered:.800}"
    );
    let todos = comp
        .read_state_as_vec("todos")
        .expect("孙辈内嵌的 todos 应在统一根态");
    assert_eq!(todos.len(), 4, "AppViewport 两层间接下种子仍应 4 条");
}

// 忠实画廊拓扑：host 导入 .vue 源 → ext_stubs .vue 分支探测 <stem>.vm.at
// → 其内嵌套 .at component（Demo013）随装随注册。
const VIEWPORT_HOST: &str = r#"
use.web component AppViewport from "viewport.vue"

widget App {
    view {
        AppViewport {
            app: "013-todo"
            reloadKey: 0
            viewportMode: "desktop"
        }
    }
}
"#;

const VIEWPORT_VUE: &str = "// vue 源占位——VM 臂只消费同名 .vm.at 适配器";

const VIEWPORT: &str = r#"
use.web component Demo013 from "demo013.at"

widget AppViewport(app: str, reloadKey: int, viewportMode: str) {
    view {
        col {
            if .app == "013-todo" {
                Demo013 {}
            } else {
                text "无内嵌"
            }
        }
    }
}
"#;

/// 多 store 语境（真实画廊 = TodoStore+CalendarStore+NotesStore 同台）：
/// 第二套全栈 demo 的加入不得破坏第一套的数据面。
#[test]
fn f5_embedded_fullstack_multi_store_isolation() {
    let (comp, _keep) = build_embedded(&[
        ("host.at", MULTI_HOST),
        ("demo013.at", DEMO),
        ("d013todo_todo_list.at", TODO_LIST),
        ("d013todo_todo_store.at", STORE),
        ("d013todo_api.at", API),
        ("d013todo_db.at", DB),
        ("demo015.at", DEMO15),
        ("d015notes_notes_store.at", STORE15),
        ("d015notes_api.at", API15),
        ("d015notes_db.at", DB15),
    ]);
    let _ = comp.view_with_debug();
    let todos = comp.read_state_as_vec("todos").expect("todos 可读");
    assert_eq!(todos.len(), 4, "013 种子在多 store 语境仍应 4 条");
    let notes = comp.read_state_as_vec("notes").expect("notes 可读");
    assert_eq!(notes.len(), 2, "015 种子应 2 条(隔离)");
}

const MULTI_HOST: &str = r#"
use.web component Demo013 from "demo013.at"
use.web component Demo015 from "demo015.at"

widget App {
    view {
        col {
            Demo013 {}
            Demo015 {}
        }
    }
}
"#;

const DEMO15: &str = r#"
use d015notes_notes_store: NotesStore

widget Demo015 {
    view {
        col {
            text f"n=${.NotesStore.notes.len()}"
        }
    }
    on {
        .Init -> {
            NotesStore.Init()
        }
    }
}
"#;

const STORE15: &str = r#"
use d015notes_api: list_notes

store NotesStore {
    model {
        var notes []Note = []
    }
    msg { Init }
    on {
        .Init -> {
            .notes = d015notes_api.list_notes()
        }
    }
}
"#;

const API15: &str = r#"
use d015notes_db

pub type Note = {
    id: int
    title: str
}

#[api(method = "GET", path = "/api/notes")]
pub fn list_notes() []Note {
    return d015notes_db.all_notes()
}
"#;

const DB15: &str = r#"
use d015notes_api: Note

var notes List<Note> = List<Note>.new([
    Note { id: 0, title: "Welcome" },
    Note { id: 1, title: "Ideas" },
])

pub fn all_notes() []Note {
    return notes
}
"#;
