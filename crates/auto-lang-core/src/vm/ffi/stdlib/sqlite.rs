//! PLAN-843 结构拆分：sqlite 域实现（原样搬移，算法/行为零变化）。
use super::*;

pub(super) fn shim_sqlite_open_impl(vm: &AutoVM, task_id: u64, path: String) -> i64 {
    let db = a2r_std::sqlite::open(path);
    let id = SQLITE_HANDLE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    vm.store_sqlite_handle(task_id, id, db);
    id
}

pub(super) fn shim_sqlite_exec_impl(vm: &AutoVM, handle: i64, sql: String) -> i64 {
    match sqlite_from_handle(vm, handle) {
        Some(db) => db.exec(sql),
        None => -1,
    }
}

pub(super) fn shim_sqlite_query_impl(vm: &AutoVM, handle: i64, sql: String) -> Vec<Vec<String>> {
    match sqlite_from_handle(vm, handle) {
        Some(db) => db.query(sql),
        None => Vec::new(),
    }
}

pub(super) fn shim_sqlite_last_insert_rowid_impl(vm: &AutoVM, handle: i64) -> i64 {
    match sqlite_from_handle(vm, handle) {
        Some(db) => db.last_insert_rowid(),
        None => 0,
    }
}

/// Pop the query rows off the FFI return and build the nested heap list the
/// .at surface expects (`List<List<str>>`): outer ListData<Value> whose
/// elements are VmRefs into inner ListData<i32> negative-sentinel string rows
/// (same layout the array-literal materialize path produces).
pub fn shim_sqlite_query(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let sql: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle: i64 = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let rows = shim_sqlite_query_impl(vm, handle, sql);
    use crate::vm::types::ListData;
    let mut outer: ListData<auto_val::Value> = ListData::new();
    for row in rows {
        let mut inner: ListData<i32> = ListData::new();
        for cell in row {
            let len = vm.add_string(cell.as_bytes().to_vec());
            vm.pool_retain(len);
            inner.push(-(len as i32) - 1);
        }
        let inner_id = vm.insert_heap_object(inner);
        outer.push(auto_val::Value::VmRef(auto_val::VmRef {
            id: inner_id as usize,
        }));
    }
    let outer_id = vm.insert_heap_object(outer);
    vm.rc_push_id(task, outer_id as u64);
    Ok(())
}

pub fn shim_sqlite_open(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let id = shim_sqlite_open_impl(vm, task.id, path);
    task.ram.push_i64(id);
    Ok(())
}

pub fn shim_sqlite_exec(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let sql: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle: i64 = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let n = shim_sqlite_exec_impl(vm, handle, sql);
    task.ram.push_i64(n);
    Ok(())
}

pub fn shim_sqlite_last_insert_rowid(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle: i64 = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let n = shim_sqlite_last_insert_rowid_impl(vm, handle);
    task.ram.push_i64(n);
    Ok(())
}

pub fn shim_sqlite_last_error(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg: String = a2r_std::sqlite::last_error();
    VMConvertible::push_to_stack(&msg, task, vm).map_err(|e| VMError::RuntimeError(e.to_string()))
}

// ---- PLAN-884 T-04：自 stdlib 壳迁入（原样搬移） ----

use a2r_std::sqlite::SqliteDb;

static SQLITE_HANDLE_COUNTER: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(1);

pub const SQLITE_NATIVE_NAMES: &[&str] = &[
    "auto.sqlite.open",
    "auto.sqlite.exec",
    "auto.sqlite.query",
    "auto.sqlite.last_insert_rowid",
    "auto.sqlite.last_error",
    // Type.method form (List.push -> auto.list.push canonicalization rule)
    "auto.sqlitedb.exec",
    "auto.sqlitedb.query",
    "auto.sqlitedb.last_insert_rowid",
];

fn sqlite_from_handle(vm: &AutoVM, handle: i64) -> Option<SqliteDb> {
    vm.sqlite_handle(handle)
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_sqlite_shims(natives: &mut crate::vm::native::NativeInterface) {
    // Plan 744 / D-VM-SQLITE: sqlite module on the VM track. Names are seeded
    // explicitly (same cwd caveat as image.vm.at below) and bound under both
    // the module form (auto.sqlite.*) and the Type.method form
    // (auto.sqlitedb.*) — the bytecode emits either depending on the call site.
    {
        let mut registry = crate::vm::native_registry::BIGVM_NATIVES.lock().unwrap();
        for name in SQLITE_NATIVE_NAMES {
            registry.register(name);
        }
        // Plan 744 D9 runtime face: fs.dir_names (receipts enumeration).
        registry.register("auto.fs.dir_names");
    }
    natives.register_shim_by_name("auto.sqlite.open", shim_sqlite_open);
    natives.register_shim_by_name("auto.sqlite.exec", shim_sqlite_exec);
    natives.register_shim_by_name("auto.sqlite.query", shim_sqlite_query);
    natives.register_shim_by_name(
        "auto.sqlite.last_insert_rowid",
        shim_sqlite_last_insert_rowid,
    );
    natives.register_shim_by_name("auto.sqlite.last_error", shim_sqlite_last_error);
    natives.register_shim_by_name("auto.sqlitedb.exec", shim_sqlite_exec);
    natives.register_shim_by_name("auto.sqlitedb.query", shim_sqlite_query);
    natives.register_shim_by_name(
        "auto.sqlitedb.last_insert_rowid",
        shim_sqlite_last_insert_rowid,
    );
    natives.register_shim_by_name("auto.fs.dir_names", shim_fs_dir_names);
}
