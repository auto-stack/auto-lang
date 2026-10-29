//! PLAN-843 结构拆分：sys_env 域实现（原样搬移，算法/行为零变化）。
use super::*;

/// Get an environment variable
#[auto_macros::rust_fn("Env.get")]
pub fn shim_env_get(key: String) -> String {
    std::env::var(&key).unwrap_or_default()
}

/// Set an environment variable
#[auto_macros::rust_fn("Env.set")]
pub fn shim_env_set(key: String, value: String) {
    // SAFETY: Environment variable modification is safe in single-threaded context
    // In multi-threaded context, this could cause data races
    #[allow(deprecated)]
    unsafe {
        std::env::set_var(&key, &value);
    }
}

/// Remove an environment variable
#[auto_macros::rust_fn("Env.remove")]
pub fn shim_env_remove(key: String) {
    // SAFETY: Environment variable modification is safe in single-threaded context
    // In multi-threaded context, this could cause data races
    #[allow(deprecated)]
    unsafe {
        std::env::remove_var(&key);
    }
}

/// Get an environment variable with a default value
#[auto_macros::rust_fn("Env.get_or")]
pub fn shim_env_get_or(key: String, default: String) -> String {
    std::env::var(&key).unwrap_or(default)
}

/// Track identification (PLAN-680 Q1): the VM runtime IS the vm track.
/// The vue generator folds `Env.track()` to the "vue" literal at generation
/// time, so the same `.at` source yields its track's true value on both
/// sides — the degradation-guard signal for demo corpora (027 file-manager).
#[auto_macros::rust_fn("Env.track")]
pub fn shim_env_track() -> String {
    "vm".to_string()
}

/// storage.get(key) → stored value, or "" if absent (mirrors JS localStorage).
#[auto_macros::rust_fn("Storage.get")]
pub fn shim_storage_get(key: String) -> String {
    storage_load();
    super::STORAGE_MAP
        .lock()
        .unwrap()
        .get(&key)
        .cloned()
        .unwrap_or_default()
}

/// storage.set(key, value) → persists (file-backed; see Plan 438 M2 note).
#[auto_macros::rust_fn("Storage.set")]
pub fn shim_storage_set(key: String, value: String) {
    storage_load();
    storage_mark_dirty(&key);
    super::STORAGE_MAP.lock().unwrap().insert(key, value);
    storage_persist();
}

/// storage.remove(key) → drops the entry.
#[auto_macros::rust_fn("Storage.remove")]
pub fn shim_storage_remove(key: String) {
    storage_load();
    super::STORAGE_MAP.lock().unwrap().remove(&key);
    storage_mark_dirty(&key);
    storage_persist();
}

/// Get the local data directory (e.g. ~/.local/share on Unix, %APPDATA% on Windows)
#[auto_macros::rust_fn("Env.local_data_dir")]
pub fn shim_env_local_data_dir() -> String {
    if cfg!(target_os = "windows") {
        std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string())
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        home + "/.local/share"
    }
}

/// Get the user's home directory
#[auto_macros::rust_fn("Env.home_dir")]
pub fn shim_env_home_dir() -> String {
    if cfg!(target_os = "windows") {
        std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string())
    } else {
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    }
}

/// Get current time in milliseconds since Unix epoch.
/// PLAN-872 实测钉死（族四）：本纯 shim 经 inventory 绑定 id 1200，是
/// `Time.now_ms` 的**生效实现**——`.unwrap()` 意味着时钟不可用
/// （SystemTime Err）环境 panic；native.rs 的 nv 版（`unwrap_or(0)`，同槽
/// 1200）被后写覆盖，稳态不在派发路径上（RETIRE 候选，单独签核）。
/// a2r = a2r-std/src/time.rs 同为 unwrap（GOAL-003 一致口径；
/// attachments/872/binding-table.md 族四）。
#[auto_macros::rust_fn("Time.now_ms")]
pub fn shim_time_now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// Get current time in seconds since Unix epoch（生效实现=本纯 shim，id 1201，
/// unwrap/时钟不可用 panic；nv `unwrap_or(0)` 版被覆盖——PLAN-872 族四同上）。
#[auto_macros::rust_fn("Time.now_sec")]
pub fn shim_time_now_sec() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Sleep for specified milliseconds
#[auto_macros::rust_fn("Time.sleep_ms")]
pub fn shim_time_sleep_ms(ms: i32) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
}

/// Get current timestamp as a string (seconds since epoch)（生效实现=本纯
/// shim，id 1205，unwrap 语义同上——PLAN-872 族四）。
#[auto_macros::rust_fn("Time.now")]
pub fn shim_time_now() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string()
}

/// Plan 394 Phase A fixture: `test.delay_async(ms) -> Future` (External).
/// Resolves to Int(ms) after `ms` milliseconds. Pushes future_bits.
pub fn shim_test_delay_async(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let ms = task.ram.pop_i32();
    let ms = if ms < 0 { 0 } else { ms as u64 };
    let fid = vm.register_external_future(task.id);
    if let Some(future_arc) = vm.futures.get(&fid) {
        let future_arc = future_arc.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            // PLAN-705 T-02: External future 完成走统一入口（含完成通知）。
            crate::vm::ffi::async_http::complete_external_future(
                &future_arc,
                Ok(auto_val::Value::Int(ms as i32)),
            );
        });
    }
    task.ram.push_nv(auto_val::encode_future(fid));
    Ok(())
}

/// Plan 394 Phase A fixture: `test.fail_async(msg) -> Future` (External Failed).
pub fn shim_test_fail_async(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _msg: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let fid = vm.register_external_future(task.id);
    if let Some(future_arc) = vm.futures.get(&fid) {
        // PLAN-705 T-02: 完成走统一入口（本臂同步完成，通知仍发出——
        // enable→check 模式下先完成再等待同样立即可见）。
        crate::vm::ffi::async_http::complete_external_future(&future_arc, Err("failed".into()));
    }
    task.ram.push_nv(auto_val::encode_future(fid));
    Ok(())
}

/// `Array.isArray(x)`——JS 语义：仅数组（ListData 全变体）为 true，
/// 对象/字符串/标量/None 一律 false。推 encode_bool（`x == true` 字面量
/// 比较与 print 均按 tag 消费；早前 i32 1/0 形态使 case A/B/C 的
/// expected=true 跨 tag EQ 恒假）。
pub fn shim_web_is_array(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let nv = task.ram.pop_nv();
    let is_arr = web_builtin_heap_id(nv)
        .map(|id| heap_is_list(vm, id))
        .unwrap_or(false);
    task.ram.push_nv(auto_val::encode_bool(is_arr));
    Ok(())
}

/// `JSON.stringify(v[, replacer][, space])`——固 arity 3 弹栈（codegen 对
/// 缺参调用点编译期补位：replacer=null、space=0，沿 Plan 197 str.slice
/// 先例）。replacer 采纳 JS 语义子集：忽略（musk 传 null）。space>0 走
/// serde 两空格 pretty（与 JS JSON.stringify(v,null,2) 形态一致）。
/// 不可序列化（None 顶层/NaN 等）推 None（musk T0 守卫口径）。
pub fn shim_web_json_stringify(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let space_nv = task.ram.pop_nv();
    let _replacer_nv = task.ram.pop_nv();
    let value_nv = task.ram.pop_nv();

    if auto_val::is_null(value_nv) {
        task.ram.push_nv(auto_val::encode_null());
        return Ok(());
    }
    let space = if auto_val::is_i32(space_nv) {
        auto_val::decode_i32(space_nv)
    } else {
        0
    };
    let value = nv_to_vm_value(vm, value_nv);
    let jv = vm_value_to_json(vm, &value, 0)?;
    let text = if space > 0 {
        serde_json::to_string_pretty(&jv)
            .map_err(|e| VMError::RuntimeError(format!("JSON.stringify: {}", e)))?
    } else {
        serde_json::to_string(&jv)
            .map_err(|e| VMError::RuntimeError(format!("JSON.stringify: {}", e)))?
    };
    let idx = vm.add_string(text.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

/// `Math.trunc(x)`——int 表达式（含经宽通道的大整数商）恒等直通
/// （wl_probe2 实证：原静默桩使 nowSec 回绕成 -2147483647 垃圾）；
/// float 截断为 i32（.at 比较语义整型中心，避免 f64/i32 跨 tag EQ 陷阱）。
pub fn shim_web_math_trunc(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let nv = task.ram.pop_nv();
    if auto_val::is_f64(nv) {
        let t = auto_val::decode_f64(nv).trunc() as i32;
        task.ram.push_nv(auto_val::encode_i32(t));
    } else if auto_val::is_f32(nv) {
        let t = auto_val::decode_f32(nv).trunc() as i32;
        task.ram.push_nv(auto_val::encode_i32(t));
    } else {
        // int（任意宽度编码）/其它：恒等直通
        task.ram.push_nv(nv);
    }
    Ok(())
}

/// `Math.imul(a, b)`——JS 32 位回绕乘（hash 链语义依赖，musk agentColors
/// 的 hash*31 滚动乘）。
pub fn shim_web_math_imul(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let b_nv = task.ram.pop_nv();
    let a_nv = task.ram.pop_nv();
    let to_i32 = |nv: auto_val::NanoValue| -> i32 {
        if auto_val::is_f64(nv) {
            auto_val::decode_f64(nv) as i32
        } else if auto_val::is_f32(nv) {
            auto_val::decode_f32(nv) as i32
        } else {
            auto_val::decode_i32(nv)
        }
    };
    let product = to_i32(a_nv).wrapping_mul(to_i32(b_nv));
    task.ram.push_nv(auto_val::encode_i32(product));
    Ok(())
}

/// Log a debug message to stdout
#[auto_macros::rust_fn("Log.debug")]
pub fn shim_log_debug(msg: String) {
    println!("[DEBUG] {}", msg);
}

/// Log an info message to stdout
#[auto_macros::rust_fn("Log.info")]
pub fn shim_log_info(msg: String) {
    println!("[INFO] {}", msg);
}

/// Log a warning message to stdout
#[auto_macros::rust_fn("Log.warn")]
pub fn shim_log_warn(msg: String) {
    println!("[WARN] {}", msg);
}

/// Log an error message to stderr
#[auto_macros::rust_fn("Log.error")]
pub fn shim_log_error(msg: String) {
    eprintln!("[ERROR] {}", msg);
}

/// `session.create(data_json: String) -> String`
/// Create a new session with the given JSON data. Returns a random session ID.
pub fn shim_session_create(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let data: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    // Generate a simple session ID (timestamp + counter).
    let session_id = format!("sess_{}", NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst));

    if let Ok(mut sessions) = SESSIONS.lock() {
        sessions.insert(session_id.clone(), data);
    }

    let idx = vm.add_string(session_id.into_bytes());
    vm.rc_push_str_idx(task, idx);
    Ok(())
}

/// `session.get(id: String) -> String`
/// Get session data by ID. Returns the JSON data string, or "null" if not found.
pub fn shim_session_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let id: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let data = SESSIONS
        .lock()
        .ok()
        .and_then(|sessions| sessions.get(&id).cloned())
        .unwrap_or_else(|| "null".to_string());

    let idx = vm.add_string(data.into_bytes());
    vm.rc_push_str_idx(task, idx);
    Ok(())
}

/// `session.set(id: String, data_json: String) -> bool`
/// Update session data. Returns true if session existed.
pub fn shim_session_set(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let data: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let id: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let exists = SESSIONS
        .lock()
        .ok()
        .map(|mut sessions| {
            if sessions.contains_key(&id) {
                sessions.insert(id, data);
                true
            } else {
                false
            }
        })
        .unwrap_or(false);

    task.ram.push_nv(auto_val::encode_bool(exists));
    Ok(())
}

/// `session.destroy(id: String) -> bool`
/// Destroy a session. Returns true if session existed.
pub fn shim_session_destroy(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let id: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let existed = SESSIONS
        .lock()
        .ok()
        .map(|mut sessions| sessions.remove(&id).is_some())
        .unwrap_or(false);

    task.ram.push_nv(auto_val::encode_bool(existed));
    Ok(())
}

/// Spawn a new task instance
///
/// Creates a new TaskInstance with its own mailbox and returns a handle.
/// The handle can be used to send messages to the task.
///
/// # Arguments
/// * `task_type` - The name of the task type (e.g., "CounterTask")
/// * `capacity` - The mailbox capacity (default: 64)
///
/// # Returns
/// A handle ID (i32) that can be used to reference the task
#[auto_macros::rust_fn("Task.spawn")]
pub fn shim_task_spawn(task_type: String, capacity: i32) -> i32 {
    vm_debug!(
        "DEBUG shim_task_spawn: task_type='{}', capacity={}",
        task_type,
        capacity
    );
    let cap = if capacity <= 0 { 64 } else { capacity as usize };

    // Create a new task instance
    let mut instance = TaskInstance::new(task_type.clone(), cap);
    let handle = instance.handle.clone();

    // Generate a unique handle ID
    let handle_id = TASK_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    vm_debug!("DEBUG shim_task_spawn: generated handle_id={}", handle_id);

    // Plan 128: Get the global registry and store mailbox receiver
    // This allows spawn_initial_tasks to use the actual receiver
    let registry = get_global_task_registry();
    if let Some(receiver) = instance.take_receiver() {
        registry.store_mailbox_receiver(task_type.clone(), instance.instance_id, receiver);
        vm_debug!(
            "DEBUG shim_task_spawn: stored receiver for {}#{}",
            task_type,
            instance.instance_id
        );
    } else {
        eprintln!(
            "WARN shim_task_spawn: failed to take receiver for {}#{}",
            task_type, instance.instance_id
        );
    }

    // Register the handle with the global registry
    registry.register_instance(handle.clone());

    // Store the handle in thread-local storage (for legacy send operations)
    TASK_HANDLES.with(|handles| {
        handles.borrow_mut().insert(handle_id, handle);
    });

    // Note: We no longer store the full TaskInstance in TASK_INSTANCES
    // since the receiver has been extracted and stored in TaskRegistry

    vm_debug!("DEBUG shim_task_spawn: returning {}", handle_id as i32);
    // Return as i32 (fits in one stack slot)
    handle_id as i32
}

/// Plan 317 Phase 1: VM-aware Task.spawn.
///
/// Stack: [..., task_type_str_tag, capacity_i32] -> [..., handle_id_i32]
///
/// Spawns a real AutoVM task (registered in `vm.tasks`) whose ip points at the
/// task's `fn start()!{}` hook (exported as `"{task_type}#start"`). The handle
/// id returned is the AutoVM task id, so TaskHandle.send can look it up
/// directly. After start hook runs, its trailing TASK_LOOP opcode parks the
/// task in Waiting/message_loop; subsequent messages wake it via run_task_loop.
pub fn shim_task_spawn_vm(
    task: &mut crate::vm::task::AutoTask,
    vm: &crate::vm::engine::AutoVM,
) -> Result<(), crate::vm::engine::VMError> {
    // Plan 390 §5 Phase A (M1): stack layout (pushed bottom→top by codegen):
    //   [task_type_str, capacity, n_init, initN, ..., init0]
    // Pop order (top first): n_init count, then n_init values (rev → field 0 first),
    // then capacity, then task_type.
    let n_init = crate::vm::native::pop_arg_i32(task) as usize;

    let _stake_n_init = crate::vm::native::StakeGuard::new(vm, n_init as i64 as u64);
    let mut init_values: Vec<auto_val::NanoValue> = Vec::with_capacity(n_init);
    for _ in 0..n_init {
        init_values.push(crate::vm::native::pop_arg_nv(task));
    }
    // Popped values are in reverse declaration order (top was field 0); reverse
    // to get declaration order (field 0 first) for positional state_vars write.
    init_values.reverse();

    let capacity = crate::vm::native::pop_arg_i32(task);

    let _stake_capacity = crate::vm::native::StakeGuard::new(vm, capacity as i64 as u64);
    let ram_size = if capacity <= 0 {
        8192
    } else {
        (capacity as usize) * 64
    };

    // Pop task_type string (tagged string index)
    let task_type = {
        let nv = crate::vm::native::pop_arg_nv(task);

        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
        if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv);
            vm.get_string(idx)
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            String::new()
        }
    };

    // Look up the start hook offset from exports ("{task_type}#start").
    let start_key = format!("{}#start", task_type);
    let start_offset = vm
        .flash
        .exports_by_name
        .get(&start_key)
        .copied()
        .unwrap_or(0) as usize;

    // Spawn an AutoVM task starting at the start hook.
    let task_id = vm.spawn_task(start_offset, ram_size);

    // Plan 390 M1: write the init args into the spawned task's state_vars
    // BEFORE its #start hook runs the STORE_STATE_FIELD default initializers,
    // and lock those fields so the defaults are skipped (injected value wins).
    if !init_values.is_empty() {
        if let Some(task_arc) = vm.tasks.get(&task_id) {
            // try_lock (non-blocking): the task was just spawned Ready and has
            // not been stepped yet, so nothing else holds its lock. blocking_lock
            // would panic inside the async runtime.
            if let Ok(mut spawned) = task_arc.try_lock() {
                if spawned.state_vars.len() < n_init {
                    spawned.state_vars.resize(n_init, auto_val::encode_i32(0));
                }
                for (i, v) in init_values.into_iter().enumerate() {
                    if i < spawned.state_vars.len() {
                        spawned.state_vars[i] = v;
                        // VM bug fix: lock this field so #start's (now-working)
                        // default initializers don't clobber the spawn-injected
                        // value. Cleared at TASK_LOOP so handlers can write freely.
                        spawned.locked_state_fields.insert(i as u8);
                    }
                }
            }
        }
    }

    // Create the mailbox for this task (DashMap<task_id, Mutex<Vec<Value>>).
    vm.task_mailboxes
        .insert(task_id, std::sync::Mutex::new(Vec::new()));

    vm_debug!(
        "DEBUG shim_task_spawn_vm: spawned task_id={} for type {} at offset {} (init_args={})",
        task_id,
        task_type,
        start_offset,
        n_init
    );

    // Return the AutoVM task id as the handle (send uses it directly).
    task.ram.push_i32(task_id as i32);
    Ok(())
}

/// Plan 317 Phase 1: VM-aware TaskHandle.send.
///
/// Stack: [..., handle_id_i32, msg_i32] -> [..., ok_i32]
///
/// handle_id IS the AutoVM task id (returned by shim_task_spawn_vm). Pushes the
/// message onto the target task's pending_messages. The target is either
/// running (will see the message on its next TASK_LOOP park) or Waiting (woken
/// by run_task_loop's message-loop check).
pub fn shim_task_send_vm(
    task: &mut crate::vm::task::AutoTask,
    vm: &crate::vm::engine::AutoVM,
) -> Result<(), crate::vm::engine::VMError> {
    // Stack: handle_id, msg (msg on top)
    // Plan 390 §14 L3: msg can be either a scalar i32 (legacy/TypeBinding/
    // Literal patterns) or an object VmRef (WithBindings variant messages built
    // by codegen's task-variant constructor). Decode the tag to decide.
    let msg_nv = task.ram.pop_nv();
    let handle_id = task.ram.pop_i32() as u64;
    let msg_value = if auto_val::is_object(msg_nv) {
        // Structured message: preserve the object reference so the wake path can
        // push it back and the handler can GET_FIELD each binding.
        let id = auto_val::decode_object(msg_nv) as u64;
        // Plan 419: 邮箱持有消息 —— retain 一份(CALL_NAT 死区结算会释放被弹
        // 参数槽的原 stake,retain 与之配平,净效果为 stake 转移进邮箱)。
        vm.rc_retain_id(id);
        auto_val::Value::VmRef(auto_val::VmRef { id: id as usize })
    } else {
        // Scalar message (i32/bool/etc): encode as Value::Int for backward compat
        // with Literal/TypeBinding/Simple patterns.
        auto_val::Value::Int(auto_val::decode_i32(msg_nv))
    };

    // Plan 317 Phase 1: use the DashMap mailbox (std Mutex, safe in sync native).
    // Lazily create the mailbox entry if missing.
    if let Some(mailbox) = vm.task_mailboxes.get(&handle_id) {
        if let Ok(mut q) = mailbox.lock() {
            q.push(msg_value);
            vm_debug!(
                "DEBUG shim_task_send_vm: delivered msg to task {}",
                handle_id
            );
            task.ram.push_i32(1); // success
            return Ok(());
        }
    }
    // Plan 419: 邮箱缺失路径 —— 消息未投递,structured 引用按死亡释放。
    if let auto_val::Value::VmRef(r) = &msg_value {
        vm.rc_release_id(r.id as u64);
    }
    eprintln!("TaskHandle.send failed: no mailbox for task {}", handle_id);
    task.ram.push_i32(0);
    Ok(())
}

/// Send a message to a task
///
/// # Arguments
/// * `handle_id` - The handle ID returned by Task.spawn
/// * `msg` - The message value to send (enum variant as i32)
///
/// # Returns
/// 1 on success, 0 on failure
#[auto_macros::rust_fn("TaskHandle.send")]
pub fn shim_task_send(handle_id: i32, msg: i32) -> i32 {
    let id = handle_id as u64;

    TASK_HANDLES.with(|handles| {
        let handles = handles.borrow();
        if let Some(handle) = handles.get(&id) {
            match handle.try_send(auto_val::Value::Int(msg)) {
                Ok(()) => 1,
                Err(e) => {
                    eprintln!("TaskHandle.send failed: {}", e);
                    0
                }
            }
        } else {
            eprintln!("TaskHandle.send failed: invalid handle ID {}", id);
            0
        }
    })
}

/// Send a message to a singleton task
///
/// # Arguments
/// * `task_type` - The task type name (e.g., "MonitorTask")
/// * `msg` - The message value to send (enum variant as i32)
///
/// # Returns
/// 1 on success, 0 on failure
#[auto_macros::rust_fn("Task.singleton_send")]
pub fn shim_task_singleton_send(task_type: String, msg: i32) -> i32 {
    // First, check if the singleton task already exists
    let existing_handle_id = SINGLETON_TASKS.with(|singletons| {
        let singletons = singletons.borrow();
        singletons.get(&task_type).copied()
    });

    let handle_id = if let Some(id) = existing_handle_id {
        id
    } else {
        // Auto-spawn the singleton task on first access
        vm_debug!("DEBUG: Auto-spawning singleton task '{}'", task_type);
        let mut instance = TaskInstance::new(task_type.clone(), 64);
        let handle = instance.handle.clone();
        let new_id = TASK_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);

        // Plan 128: Extract and store the mailbox receiver in global TaskRegistry
        if let Some(receiver) = instance.take_receiver() {
            let registry = get_global_task_registry();
            registry.store_mailbox_receiver(task_type.clone(), instance.instance_id, receiver);
            registry.register_singleton(task_type.clone(), handle.clone());
            vm_debug!(
                "DEBUG: Stored receiver for singleton {}#{}",
                task_type,
                instance.instance_id
            );
        } else {
            eprintln!(
                "WARN: Failed to take receiver for singleton {}#{}",
                task_type, instance.instance_id
            );
            // Still register the singleton even if receiver extraction failed
            let registry = get_global_task_registry();
            registry.register_singleton(task_type.clone(), handle.clone());
        }

        // Store the handle in thread-local storage
        TASK_HANDLES.with(|handles| {
            handles.borrow_mut().insert(new_id, handle);
        });

        // Register as singleton in thread-local tracking
        SINGLETON_TASKS.with(|singletons| {
            singletons.borrow_mut().insert(task_type.clone(), new_id);
        });

        new_id
    };

    // Now send the message
    TASK_HANDLES.with(|handles| {
        let handles = handles.borrow();
        if let Some(handle) = handles.get(&handle_id) {
            match handle.try_send(auto_val::Value::Int(msg)) {
                Ok(()) => 1,
                Err(e) => {
                    eprintln!("Task.send failed: {}", e);
                    0
                }
            }
        } else {
            eprintln!("Task.send failed: handle not found for {}", task_type);
            0
        }
    })
}

// Plan 124 Phase 2.2: send().await with backpressure
//
// Sends a message to a task, blocking (suspending) if the mailbox is full.
// This is the async version that supports backpressure.
//
// In Phase 2.2, this is a simplified implementation that uses try_send
// with immediate return. Full implementation would:
// 1. Check if mailbox has capacity
// 2. If full, suspend the current task
// 3. Resume when space becomes available
//
// Usage in AutoLang:
//   TaskHandle.send_await(msg)  // Returns Future<void>
//   TaskHandle.send(msg).await  // Syntactic sugar for above
#[auto_macros::rust_fn("TaskHandle.send_await")]
pub fn shim_task_send_await(handle_id: i64, msg: String) -> Result<i64, String> {
    let id = handle_id as u64;

    // Phase 2.2: Simplified - just wrap send in a Future
    // Full implementation would actually suspend if mailbox is full
    TASK_HANDLES.with(|handles| {
        let handles = handles.borrow();
        if let Some(handle) = handles.get(&id) {
            let auto_str: auto_val::AutoStr = msg.into();
            match handle.try_send(auto_val::Value::Str(auto_str)) {
                Ok(()) => {
                    // Return a completed Future (represented as 0 for now)
                    // In full implementation, this would return a Future that
                    // resolves when the message is actually delivered
                    Ok(0)
                }
                Err(e) => Err(format!("TaskHandle.send_await failed: {}", e)),
            }
        } else {
            Err(format!(
                "TaskHandle.send_await failed: invalid handle ID {}",
                id
            ))
        }
    })
}

// Plan 124 Phase 2.3: ask/reply bidirectional RPC
//
// Sends a message to a task and returns a Future that will receive the reply.
// The receiver uses `reply expr` to send back a response.
//
// Usage in AutoLang:
//   let result = TaskHandle.ask(msg).await.?
//
// Implementation:
// 1. Creates a oneshot channel for the reply
// 2. Sends the message with the reply sender attached
// 3. Returns a Future that awaits the oneshot receiver
//
// Phase 2.3: Simplified implementation that just wraps send
#[auto_macros::rust_fn("TaskHandle.ask")]
pub fn shim_task_ask(handle_id: i64, msg: String) -> Result<i64, String> {
    let id = handle_id as u64;

    // Phase 2.3: Simplified - just wrap send in a Future
    // Full implementation would:
    // 1. Create a oneshot channel (reply_tx, reply_rx)
    // 2. Send message with reply_tx attached
    // 3. Return Future that awaits reply_rx
    TASK_HANDLES.with(|handles| {
        let handles = handles.borrow();
        if let Some(handle) = handles.get(&id) {
            let auto_str: auto_val::AutoStr = msg.into();
            match handle.try_send(auto_val::Value::Str(auto_str)) {
                Ok(()) => {
                    // Return a completed Future (represented as 0 for now)
                    // In full implementation, this would return a FutureReceiver
                    Ok(0)
                }
                Err(e) => Err(format!("TaskHandle.ask failed: {}", e)),
            }
        } else {
            Err(format!("TaskHandle.ask failed: invalid handle ID {}", id))
        }
    })
}

/// Check if a task handle is null/empty
///
/// # Arguments
/// * `handle_id` - The handle ID to check
///
/// # Returns
/// 1 if null, 0 otherwise
#[auto_macros::rust_fn("TaskHandle.is_null")]
pub fn shim_task_handle_is_null(handle_id: i64) -> Result<i32, String> {
    let id = handle_id as u64;

    if id == 0 {
        return Ok(1);
    }

    TASK_HANDLES.with(|handles| {
        let handles = handles.borrow();
        if let Some(handle) = handles.get(&id) {
            Ok(if handle.is_null() { 1 } else { 0 })
        } else {
            // Invalid handle ID is treated as null
            Ok(1)
        }
    })
}

/// Get the task type from a handle
///
/// # Arguments
/// * `handle_id` - The handle ID
///
/// # Returns
/// The task type name
#[auto_macros::rust_fn("TaskHandle.task_type")]
pub fn shim_task_handle_type(handle_id: i64) -> Result<String, String> {
    let id = handle_id as u64;

    TASK_HANDLES.with(|handles| {
        let handles = handles.borrow();
        if let Some(handle) = handles.get(&id) {
            Ok(handle.task_type.clone())
        } else {
            Err(format!(
                "TaskHandle.task_type failed: invalid handle ID {}",
                id
            ))
        }
    })
}

/// Get the instance ID from a handle
///
/// # Arguments
/// * `handle_id` - The handle ID
///
/// # Returns
/// The instance ID
#[auto_macros::rust_fn("TaskHandle.instance_id")]
pub fn shim_task_handle_id(handle_id: i64) -> Result<u64, String> {
    let id = handle_id as u64;

    TASK_HANDLES.with(|handles| {
        let handles = handles.borrow();
        if let Some(handle) = handles.get(&id) {
            Ok(handle.instance_id)
        } else {
            Err(format!(
                "TaskHandle.instance_id failed: invalid handle ID {}",
                id
            ))
        }
    })
}

/// Start the task system scheduler
///
/// This method blocks the main thread and waits for Ctrl+C signal.
/// When Ctrl+C is received, all registered stop hooks are executed in LIFO order.
///
/// # Note
/// This is a blocking call that will not return until Ctrl+C is received.
#[auto_macros::rust_fn("TaskSystem.start")]
pub fn shim_task_system_start() -> Result<(), String> {
    use crate::vm::native::NativeInterface;
    use crate::vm::scheduler::GlobalMeta;
    use crate::vm::virt_memory::VirtualFlash;
    use std::collections::HashMap;

    let registry = get_global_task_registry();

    // Build GlobalMeta if not already set
    // For MVP, we create a minimal GlobalMeta with empty bytecode
    // Full integration would populate this from the compiled VM state
    if registry.get_global_meta().is_none() {
        let meta = Arc::new(GlobalMeta::from_components(
            VirtualFlash::new(0),
            Vec::new(),
            NativeInterface::new(),
            HashMap::new(),
        ));
        registry.set_global_meta(meta);
    }

    registry.start_scheduler();
    Ok(())
}

/// Stop the task system scheduler
///
/// This method signals the scheduler to stop and executes all stop hooks in LIFO order.
/// Unlike start_scheduler, this does NOT block - it just signals shutdown.
///
/// # Note
/// This is useful for testing and graceful shutdown without Ctrl+C.
#[auto_macros::rust_fn("TaskSystem.stop")]
pub fn shim_task_system_stop() -> Result<(), String> {
    let registry = get_global_task_registry();

    // Signal the scheduler to stop (Plan 127)
    registry.signal_shutdown();

    Ok(())
}

// Plan 224: TaskSystem.run() - VM-aware async body execution
//
// Executes an async block by looking up the future in the VM registry
// and running its body bytecode via execute_single_frame.
//
// Usage in AutoLang:
//   TaskSystem.run(~{
//     // async code here
//   })
//
// The future_id is encoded on the VM stack as (future_id << 8) | 0xF0.
#[allow(non_snake_case)]
pub fn shim_task_system_run(
    task: &mut crate::vm::task::AutoTask,
    vm: &crate::vm::engine::AutoVM,
) -> Result<(), crate::vm::engine::VMError> {
    use crate::vm::engine::{FrameResult, FutureState};

    // Pop the future handle from stack（PLAN-776 R7-01：TAG_FUTURE 逐值载荷
    // 优先，legacy 位模式兼容臂）。
    let future_nv = task.ram.pop_nv();
    task.ram.raw_nv[task.ram.sp] = 0;
    let Some(future_id) = AutoVM::decode_future_handle(future_nv) else {
        // Not a valid future — push nil and return
        task.ram.push_i32(0);
        return Ok(());
    };

    // Look up the future
    let future_arc = match vm.futures.get(&future_id) {
        Some(f) => f,
        None => {
            task.ram.push_i32(0);
            return Ok(());
        }
    };

    let body_offset = {
        let fv = future_arc.read().unwrap();
        fv.body_offset
    };

    // Execute the async body using VM engine
    let saved_ip = task.ip;
    task.ip = body_offset as usize;

    let mut result_value = auto_val::Value::Int(0);
    let success;

    loop {
        match vm.execute_single_frame(task, 10_000) {
            FrameResult::Return => {
                if task.ram.sp > task.bp + 1 {
                    let raw = crate::vm::native::pop_arg_i32(task);

                    let _stake_raw = crate::vm::native::StakeGuard::new(vm, raw as i64 as u64);
                    result_value = auto_val::Value::Int(raw);
                }
                success = true;
                break;
            }
            FrameResult::AwaitFuture {
                future_id: inner_id,
                body_offset: inner_offset,
            } => {
                vm.handle_await_future(task, inner_id, inner_offset)?;
            }
            FrameResult::BudgetExhausted | FrameResult::Yielded => {
                // Continue executing
                continue;
            }
            FrameResult::Error(e) => {
                if let Some(fv) = vm.futures.get(&future_id) {
                    fv.write().unwrap().state = FutureState::Failed;
                }
                task.ip = saved_ip;
                return Err(e);
            }
            FrameResult::Continue => unreachable!(),
        }
    }

    task.ip = saved_ip;

    // Update future state
    if let Some(fv) = vm.futures.get(&future_id) {
        let mut future = fv.write().unwrap();
        future.state = if success {
            FutureState::Ready
        } else {
            FutureState::Failed
        };
        future.result = Some(result_value.clone());
    }

    // Push result onto stack
    match &result_value {
        auto_val::Value::Int(n) => task.ram.push_i32(*n),
        auto_val::Value::Nil => task.ram.push_i32(0),
        _ => task.ram.push_i32(0),
    }

    Ok(())
}

/// Run all VM file-based tests in a directory.
/// Pops path (String) from stack, pushes failure count (i64) to stack.
pub fn shim_test_run_vm_dir(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let dir = Path::new(&path);
    let cases = crate::test_runner::discover_vm_tests(dir);

    if cases.is_empty() {
        return Err(VMError::RuntimeError(format!(
            "Test.run_vm_dir: no tests found in: {}",
            path
        )));
    }

    let mut failures = 0i64;
    for case in &cases {
        let report = crate::run_vm_file_test(case);
        match &report.outcome {
            crate::test_runner::TestOutcome::Passed => {
                print!("test {} ... ok\n", case.name);
            }
            crate::test_runner::TestOutcome::Failed(msg) => {
                print!("test {} ... FAILED\n", case.name);
                print!("    {}\n", msg);
                failures += 1;
            }
        }
    }

    failures
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// Run all a2c transpiler tests in a directory.
/// Pops path (String) from stack, pushes failure count (i64) to stack.
pub fn shim_test_run_a2c_dir(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let dir = Path::new(&path);
    let cases = crate::test_runner::discover_a2c_tests(dir);

    if cases.is_empty() {
        return Err(VMError::RuntimeError(format!(
            "Test.run_a2c_dir: no tests found in: {}",
            path
        )));
    }

    let mut failures = 0i64;
    for case in &cases {
        let report = crate::run_a2c_file_test(case);
        match &report.outcome {
            crate::test_runner::TestOutcome::Passed => {
                print!("test {} ... ok\n", case.name);
            }
            crate::test_runner::TestOutcome::Failed(msg) => {
                print!("test {} ... FAILED\n", case.name);
                print!("    {}\n", msg);
                failures += 1;
            }
        }
    }

    failures
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// Run all a2ts transpiler tests in a directory.
/// Pops path (String) from stack, pushes failure count (i64) to stack.
pub fn shim_test_run_a2ts_dir(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let dir = Path::new(&path);
    let cases = crate::test_runner::discover_a2ts_tests(dir);

    if cases.is_empty() {
        return Err(VMError::RuntimeError(format!(
            "Test.run_a2ts_dir: no tests found in: {}",
            path
        )));
    }

    let mut failures = 0i64;
    for case in &cases {
        let report = crate::run_a2ts_file_test(case);
        match &report.outcome {
            crate::test_runner::TestOutcome::Passed => {
                print!("test {} ... ok\n", case.name);
            }
            crate::test_runner::TestOutcome::Failed(msg) => {
                print!("test {} ... FAILED\n", case.name);
                print!("    {}\n", msg);
                failures += 1;
            }
        }
    }

    failures
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// Run all a2r transpiler tests in a directory.
/// Pops path (String) from stack, pushes failure count (i64) to stack.
pub fn shim_test_run_a2r_dir(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let dir = Path::new(&path);
    let suite_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("a2r");
    let cases = crate::test_runner::discover_a2r_tests(dir, suite_name);

    if cases.is_empty() {
        return Err(VMError::RuntimeError(format!(
            "Test.run_a2r_dir: no tests found in: {}",
            path
        )));
    }

    let mut failures = 0i64;
    for case in &cases {
        let report = crate::run_a2r_file_test(case);
        match &report.outcome {
            crate::test_runner::TestOutcome::Passed => {
                print!("test {} ... ok\n", case.name);
            }
            crate::test_runner::TestOutcome::Failed(msg) => {
                print!("test {} ... FAILED\n", case.name);
                print!("    {}\n", msg);
                failures += 1;
            }
        }
    }

    failures
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

// ---- PLAN-884 T-04：自 stdlib 壳迁入（原样搬移） ----

// ============================================================================
// Plan 353 stage 6: Async file IO (non-blocking yield, same pattern as
// Plan 349 step7's async HTTP)
// ============================================================================

/// Plan 349 table consolidation: payload shape for the unified async result
/// table. Each variant corresponds to a former specialized table.
pub enum AsyncResult {
    /// Body string (former ASYNC_HTTP_RESULTS / ASYNC_IO_RESULTS).
    Body(String),
    /// Structured HTTP response (former ASYNC_HTTP_RESULTS_HANDLE).
    Structured {
        status: u16,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    },
    /// Auth-bearing HTTP response (former ASYNC_HTTP_RESULTS_AUTH).
    /// status is i32 to match LAST_HTTP_STATUS.
    Auth { status: i32, body: String },
}

/// Plan 394 Phase C: extract future ids from a stack list (array literal → ListData<Value>).
fn pop_future_ids_from_list(task: &mut AutoTask, vm: &AutoVM) -> Result<Vec<u32>, VMError> {
    let list_id = task.ram.pop_i32() as u64;
    let mut fids = Vec::new();
    if let Some(obj) = vm.get_heap_object(list_id) {
        let guard = obj.read().unwrap();
        if let Some(list) = guard
            .as_any()
            .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
        {
            for i in 0..list.len() {
                // PLAN-776 R8-01：句柄元素为 Value::Future（保真映射）；
                // legacy 位形态 Int 兼容保留（无生产者，防手工构造）。
                match list.get(i) {
                    Some(auto_val::Value::Future(f)) => fids.push(f.id as u32),
                    Some(auto_val::Value::Int(bits)) if (bits & 0xFF) == 0xF0 => {
                        fids.push((bits >> 8) as u32)
                    }
                    _ => {}
                }
            }
        }
    }
    if fids.is_empty() {
        return Err(VMError::RuntimeError(
            "future_all/race: empty or invalid future list".into(),
        ));
    }
    Ok(fids)
}

/// Plan 394 Phase C: `future_all([f1, f2, ...]) -> Future` (External).
/// Resolves when every component future is Ready/Failed; result is `"[v1, v2, ...]"`.
pub fn shim_future_all(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let fids = pop_future_ids_from_list(task, vm)?;
    let mut arcs = Vec::new();
    for fid in &fids {
        match vm.futures.get(fid) {
            Some(a) => arcs.push(a.clone()),
            None => {
                return Err(VMError::RuntimeError(format!(
                    "future_all: missing future {fid}"
                )))
            }
        }
    }
    let combo = vm.register_external_future(task.id);
    if let Some(combo_arc) = vm.futures.get(&combo) {
        let combo_arc = combo_arc.clone();
        std::thread::spawn(move || {
            loop {
                let all_done = arcs.iter().all(|a| {
                    a.read()
                        .map(|f| f.state != crate::vm::engine::FutureState::Pending)
                        .unwrap_or(true)
                });
                if all_done {
                    let parts: Vec<String> = arcs
                        .iter()
                        .map(|a| {
                            let f = a.read().unwrap();
                            match &f.result {
                                Some(auto_val::Value::Int(n)) => n.to_string(),
                                Some(other) => format!("{:?}", other),
                                None => "nil".to_string(),
                            }
                        })
                        .collect();
                    // PLAN-705 T-02: 完成走统一入口（含完成通知）。
                    crate::vm::ffi::async_http::complete_external_future(
                        &combo_arc,
                        Ok(auto_val::Value::Str(
                            format!("[{}]", parts.join(", ")).into(),
                        )),
                    );
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        });
    }
    task.ram.push_nv(auto_val::encode_future(combo));
    Ok(())
}

/// Plan 394 Phase C: `future_race([f1, f2, ...]) -> Future` (External).
/// Resolves with the first Ready component's result.
pub fn shim_future_race(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let fids = pop_future_ids_from_list(task, vm)?;
    let mut arcs = Vec::new();
    for fid in &fids {
        match vm.futures.get(fid) {
            Some(a) => arcs.push(a.clone()),
            None => {
                return Err(VMError::RuntimeError(format!(
                    "future_race: missing future {fid}"
                )))
            }
        }
    }
    let combo = vm.register_external_future(task.id);
    if let Some(combo_arc) = vm.futures.get(&combo) {
        let combo_arc = combo_arc.clone();
        std::thread::spawn(move || {
            loop {
                for a in &arcs {
                    let ready = a
                        .read()
                        .map(|f| f.state != crate::vm::engine::FutureState::Pending)
                        .unwrap_or(true);
                    if ready {
                        let result = a.read().unwrap().result.clone();
                        // PLAN-705 T-02: 完成走统一入口（含完成通知）。
                        crate::vm::ffi::async_http::complete_external_future(
                            &combo_arc,
                            Ok(result.unwrap_or(auto_val::Value::Nil)),
                        );
                        return;
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        });
    }
    task.ram.push_nv(auto_val::encode_future(combo));
    Ok(())
}

/// Execute a shell command string, capture stdout+stderr, truncate at 64KB.
/// Takes: cmd (String), timeout_ms (i32) — timeout currently unused.
/// Returns: JSON string {"exit_code": N, "stdout": "...", "stderr": "..."}
pub fn shim_sys_exec(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop timeout as i32 (Auto int = i32, not i64, to avoid slot mismatch)
    let timeout_ms: i32 = crate::vm::native::pop_arg_i32(task);
    let cmd: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let shell = if cfg!(windows) { "cmd" } else { "sh" };
    let flag = if cfg!(windows) { "/C" } else { "-c" };

    let output = std::process::Command::new(shell)
        .arg(flag)
        .arg(&cmd)
        .output()
        .map_err(|e| VMError::RuntimeError(format!("sys.exec failed: {} - {}", cmd, e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    let stdout_truncated = if stdout.len() > 65536 {
        format!(
            "{}...(truncated, {} bytes total)",
            &stdout[..65536],
            stdout.len()
        )
    } else {
        stdout
    };

    let result_json = serde_json::json!({
        "exit_code": exit_code,
        "stdout": stdout_truncated,
        "stderr": stderr,
    });

    let result_str = serde_json::to_string(&result_json)
        .map_err(|e| VMError::RuntimeError(format!("JSON serialization failed: {}", e)))?;

    result_str
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let _ = timeout_ms;
    Ok(())
}

/// 运行时值类型名（PLAN-781 T-25 R05 r6）：`type_of(x) -> str`——
/// int/float/str/bool/null/list/object/future。多态参数须裸 shim（rust_fn
/// 的类型化编组不支持 any）；.at 调用形如 `type_of(x)`（auto. 前缀省略，
/// 同 json.parse 先例）。动机：Reader meta 数字字段的消费域判别——
/// 字符串形态判别（concat 位模式 vs to_string 截断）对浮点 0.0 失真
/// 且受池/槽混叠（F-781-E02 家族）波及，类型谓词为可靠面。
pub fn shim_type_of(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // F-781-R07：消费参数须配平所有权——pop_arg_nv 取值并清槽、
    // StakeGuard 恰一次释放其堆/池引用份额（同 shim_json_from_value
    // 先例）。裸 pop_nv 只读 tag 不释放：复审实证 100 次调用残留
    // 100 份输入引用（对象 heap 0→100、动态串 pool +100）。
    let nv = crate::vm::native::pop_arg_nv(task);
    let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
    let t: &'static str = if auto_val::is_i32(nv) {
        "int"
    } else if auto_val::is_f32(nv) || auto_val::is_f64(nv) {
        "float"
    } else if auto_val::is_string(nv) {
        "str"
    } else if auto_val::is_bool(nv) {
        "bool"
    } else if auto_val::is_null(nv) {
        "null"
    } else if auto_val::is_list(nv) {
        "list"
    } else if auto_val::is_object(nv) {
        "object"
    } else if auto_val::is_future(nv) {
        "future"
    } else {
        "unknown"
    };
    push_string_result(task, vm, t.to_string())
}

// ============================================================================
// Option Functions (ID 1550-1559) — Plan 200 Task 2.4
// ============================================================================

/// Option.or(default) / Option.unwrap_or(default) — returns default if None, unwraps Some
pub fn shim_option_or(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop default value
    let default_val = {
        let nv = crate::vm::native::pop_arg_nv(task);
        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
        auto_val::decode_i32(nv)
    };

    // Pop option value — check for None (null or -1 sentinel)
    {
        let opt_nv = crate::vm::native::pop_arg_nv(task);

        let _stake_opt_nv = crate::vm::native::StakeGuard::nv(vm, opt_nv);
        // None is represented as encode_null() (TAG_NULL) or encode_i32(-1)
        if auto_val::is_null(opt_nv) {
            task.ram.push_i32(default_val);
            return Ok(());
        }
        let opt_val = auto_val::decode_i32(opt_nv);
        if opt_val == -1 {
            task.ram.push_i32(default_val);
            return Ok(());
        }
        // Check for heap-based Option
        if opt_val > 0 {
            let instance_id = opt_val as u64;
            if vm.is_option_none(instance_id) {
                task.ram.push_i32(default_val);
                return Ok(());
            }
            if vm.is_option_some(instance_id) {
                if let Some(inner) = vm.get_option_inner(instance_id) {
                    match inner {
                        auto_val::Value::Int(n) => task.ram.push_i32(n),
                        auto_val::Value::Bool(b) => task.ram.push_nv(auto_val::encode_bool(b)),
                        _other => task.ram.push_i32(opt_val),
                    }
                    return Ok(());
                }
            }
        }
        task.ram.push_nv(opt_nv);
        return Ok(());
    }
}

// ============================================================================
// PLAN-057 T6: web 内建 natives 补齐（等价性缺陷族③——未实现者原运行期
// 静默 None 桩）。手动 shim（非 rust_fn 宏：需按 nv 标签分支/固 arity 弹栈），
// 注册在 engine 的 AutoVM::new 覆盖块（build_from_inventory 之后，沿
// shim_json_parse_vm 惯例）。
// ============================================================================

/// 堆 id 提取：object-tag 引用 / i32 裸句柄（≥ HEAP_ID_BASE）/ list-tag。
/// 命中返回堆 id；标量与非法形态返回 None。
fn web_builtin_heap_id(nv: auto_val::NanoValue) -> Option<u64> {
    if auto_val::is_object(nv) {
        Some(auto_val::decode_object(nv) as u64)
    } else if auto_val::is_list(nv) {
        Some(auto_val::decode_list(nv) as u64)
    } else if auto_val::is_i32(nv) {
        let v = auto_val::decode_i32(nv);
        if v as i64 >= crate::vm::rc::HEAP_ID_BASE as i64 {
            Some(v as u64)
        } else {
            None
        }
    } else {
        None
    }
}

fn heap_is_list(vm: &AutoVM, id: u64) -> bool {
    use crate::vm::types::ListData;
    vm.get_heap_object(id)
        .map(|o| {
            let guard = o.read().unwrap();
            let any = guard.as_any();
            any.downcast_ref::<ListData<i32>>().is_some()
                || any.downcast_ref::<ListData<String>>().is_some()
                || any.downcast_ref::<ListData<bool>>().is_some()
                || any.downcast_ref::<ListData<auto_val::Value>>().is_some()
        })
        .unwrap_or(false)
}

/// 栈 nv → `auto_val::Value`（stringify 序列化入口；标量直编码、堆引用
/// 包 VmRef 交 vm_value_to_json 递归）。
/// PLAN-057 T11 实机补遗：堆引用判定必须在 i32 标量臂**之前**——嵌套
/// VmRef 字段读（GET_FIELD ObjectData 臂经 rc_push_id）以裸 int 句柄
/// （≥HEAP_ID_BASE 约定，数组/列表槽同款）出栈，先走 Int 臂会把句柄
/// 序列化成数字（实机工具卡 Arguments 显示 "4067504" 现场根因）。
fn nv_to_vm_value(vm: &AutoVM, nv: auto_val::NanoValue) -> auto_val::Value {
    use auto_val::Value;
    if auto_val::is_null(nv) {
        Value::Nil
    } else if auto_val::is_bool(nv) {
        Value::Bool(auto_val::decode_bool(nv))
    } else if let Some(id) = web_builtin_heap_id(nv) {
        Value::VmRef(auto_val::VmRef { id: id as usize })
    } else if auto_val::is_i32(nv) {
        Value::I64(auto_val::decode_i32(nv) as i64)
    } else if auto_val::is_i64(nv) {
        // PLAN-734 T-03（E2 修复）：48 位内联 i64——此前无臂落 Value::Nil，
        // json.encode(5000000001) 输出 4294967295（decode_i32 跨 tag 误读）。
        Value::I64(auto_val::decode_i64(nv))
    } else if auto_val::is_u64(nv) {
        // 算术 lane 的 U64 形态（Plan 377 单槽编码）——按数值交付。
        Value::I64(auto_val::decode_u64(nv) as i64)
    } else if auto_val::is_bigint(nv) {
        let id = auto_val::decode_bigint_handle(nv) as u64;
        let v = vm
            .get_heap_object(id)
            .and_then(|o| {
                let g = o.read().ok()?;
                let b = g
                    .as_any()
                    .downcast_ref::<crate::vm::heap_object::BigIntData>()?;
                Some(b.as_i64())
            })
            .unwrap_or(0);
        Value::I64(v)
    } else if auto_val::is_f64(nv) {
        Value::Double(auto_val::decode_f64(nv))
    } else if auto_val::is_f32(nv) {
        Value::Float(auto_val::decode_f32(nv).into())
    } else if auto_val::is_string(nv) {
        let idx = auto_val::decode_string(nv);
        let s = vm
            .get_string(idx)
            .map(|b| String::from_utf8_lossy(b.as_slice()).to_string())
            .unwrap_or_default();
        Value::Str(s.into())
    } else {
        Value::Nil
    }
}

// ============================================================================
// Log Functions (debug/info/warn/error = 1805-1808, noop = 1804;P499-7 移段)
// ============================================================================

/// PLAN-066 T-12（F-W1）：`i18n.t(key)` VM 轨查表——接 PLAN-050 C7 的
/// i18n_lookup 注册表（front 根目录 i18n/{lang}.json，AUTO_LOCALE 装载；
/// 查不中回落 key 本身）。此前 computed/handler 合成体的 `i18n.t(...)`
/// 无路由：`i18n` 标识符在合成作用域未绑定（组件级
/// `let i18n = useI18n()` 不进合成体）→ codegen "Undefined variable:
/// i18n" → computed 导出毒化 → App link failed 启动 exit 1（KD-066
/// F-W1）。codegen P240 map ("i18n","t") 路由至此，接收者不再编译。
/// 入参 CALL_NAT 约定（自顶向下）：[key]。
// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub fn shim_i18n_t(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let out = crate::i18n_lookup(&key).unwrap_or_else(|| key.clone());
    VMConvertible::push_to_stack(&out, task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

// ============================================================================
// Task/Msg Functions (Plan 121)
// ============================================================================

use crate::vm::task_system::{TaskHandle, TaskInstance, TaskRegistry};

// TaskHandle wrapper for passing through VM
// The handle is stored as a tuple: (task_type: String, instance_id: u64, tx_ptr: u64)
// We use a thread-local storage to keep actual handles alive
thread_local! {
    static TASK_HANDLES: std::cell::RefCell<std::collections::HashMap<u64, TaskHandle>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
    // Keep TaskInstance alive so the receiver (rx) doesn't get dropped
    static TASK_INSTANCES: std::cell::RefCell<std::collections::HashMap<u64, TaskInstance>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
    // Map task_type name -> handle_id for singleton tasks (#[single] annotation)
    static SINGLETON_TASKS: std::cell::RefCell<std::collections::HashMap<String, u64>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

static TASK_HANDLE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Global TaskRegistry for TaskSystem operations
static GLOBAL_TASK_REGISTRY: std::sync::OnceLock<TaskRegistry> = std::sync::OnceLock::new();

/// Get or initialize the global TaskRegistry
fn get_global_task_registry() -> &'static TaskRegistry {
    GLOBAL_TASK_REGISTRY.get_or_init(TaskRegistry::new)
}

/// Plan 224: Helper for executing async operations in FFI shims.
/// Creates an independent tokio runtime and runs the future with block_on.
/// Note: This blocks the current thread. Do not call from within an existing tokio runtime.
pub fn ffi_async_block_on<F, T>(f: F) -> Result<T, crate::vm::engine::VMError>
where
    F: std::future::Future<Output = Result<T, String>>,
{
    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        crate::vm::engine::VMError::RuntimeError(format!("Failed to create tokio runtime: {}", e))
    })?;
    rt.block_on(f)
        .map_err(|e| crate::vm::engine::VMError::RuntimeError(e))
}

// Plan 127 Phase 3: ctx.reply() - Send reply from message handler
//
// This function is called from task message handlers that have a context parameter:
//
//   on(ctx) {
//       "ping" => { ctx.reply("pong") }
//   }
//
// The reply is sent via the current task's MessageContext, which was set up
// when the handler was invoked by HANDLE_MSG.
//
// Note: This FFI shim is for fallback compatibility. The primary execution
// path uses the REPLY OpCode directly, which reads the MessageContext from
// task.current_msg_context set by HANDLE_MSG.
#[auto_macros::rust_fn("ctx.reply")]
pub fn shim_ctx_reply(value: i64) -> Result<(), String> {
    // The actual reply is handled by REPLY OpCode in engine.rs
    // This shim exists for FFI registration and potential future use
    // For now, we just return Ok - the real work is done by OpCode::REPLY
    if crate::is_vm_debug() {
        eprintln!("[ctx.reply] FFI called with value: {}", value);
    }
    Ok(())
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_sys_exec_option_shims(natives: &mut crate::vm::native::NativeInterface) {
    // Option functions (manual shims only)
    natives.register_shim_by_name("Option.or", shim_option_or);
    natives.register_shim_by_name("Option.unwrap_or", shim_option_or);

    // Math functions (abs/min/max now via #[rust_fn] multi-name registration)
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_task_run_shims(natives: &mut crate::vm::native::NativeInterface) {
    // Task system (manual shim — VM access for event loop)
    natives.register_shim_by_name("auto.task_system.run", shim_task_system_run);
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_test_runner_shims(natives: &mut crate::vm::native::NativeInterface) {
    // Plan 263 Phase 2-3: Test runners (manual shims — calls discover/run from test_runner)
    natives.register_shim_by_name("auto.test.run_a2r_dir", shim_test_run_a2r_dir);
    natives.register_shim_by_name("auto.test.run_vm_dir", shim_test_run_vm_dir);
    natives.register_shim_by_name("auto.test.run_a2c_dir", shim_test_run_a2c_dir);
    natives.register_shim_by_name("auto.test.run_a2ts_dir", shim_test_run_a2ts_dir);
}
