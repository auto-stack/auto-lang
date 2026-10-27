//! PLAN-843 结构拆分：native sys 域实现（自 native.rs 原样搬移）。
use super::*;

/// `system(cmd: String) -> String` — run a shell command, return its stdout.
pub fn shim_shell_system(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let cmd = pop_string_arg(task, vm);
    let out = if let Some(host) = &vm.host {
        host.system(&cmd)
    } else {
        String::new()
    };
    // Push the result string into the pool and tag-push its index.
    let idx = vm.add_string(out.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

/// PLAN-082: `shell_query(cmd: String) -> records` — structured command
/// results. Pops the command string, asks the host for the pipeline's
/// final-stage value (records as `Value::Array` of `Value::Obj` by ash
/// convention), materializes it into VM heap objects, and pushes the root
/// reference. Host-less VMs get the trait default (empty array).
pub fn shim_shell_query(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let cmd = pop_string_arg(task, vm);
    let val = if let Some(host) = &vm.host {
        host.query(&cmd)
    } else {
        auto_val::Value::Array(auto_val::Array::new())
    };
    push_value_tree(task, vm, &val);
    Ok(())
}

/// PLAN-082: `shell_run(cmd: String)` — print-executing command bridge.
/// The host renders output on the shell side; the shim leaves null so the
/// call is stack-neutral in expression position.
pub fn shim_shell_run(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let cmd = pop_string_arg(task, vm);
    if let Some(host) = &vm.host {
        host.run(&cmd);
    }
    task.ram.push_nv(auto_val::encode_null());
    Ok(())
}

/// `auto.frame.begin_ms() -> int` — 帧开始时间戳（单调毫秒；PLAN-716 组B
/// 供②观测通道，AUTO_FRAME_BENCH 门控；未捕获=0）。
/// PLAN-716 Phase 2 (供⑨-a)：出口 int lane（i32 单槽）——原 push_i64_vm
/// 推 TAG_I64 vs .at int (i32) 消费面错配，json obj 字面量等消费形全断
/// （下游 plan-022 三断实录）；frame.vm.at 判型同步 `-> int`。值域=进程
/// 单调 ms <2^31（i32 封顶 ≈24.8 天，saturating——SD-B 注记面）。
/// PLAN-779：core 不再直读 UI 的 frame_bench 通道——经本 VM
/// NativeInterface 装配的 [`crate::vm::host_capabilities::FrameProvider`]
/// 供值（UI 宿主注册 frame_bench adapter；缺席=未捕获 0，与原 no-ui/
/// 未捕获语义一致）。
pub fn shim_frame_begin_ms(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let observed = vm
        .native_interface
        .frame_provider()
        .map_or(0, |p| p.begin_ms());
    task.ram.push_i32(observed.min(i32::MAX as i64) as i32);
    Ok(())
}

/// `auto.frame.present_ms() -> int` — 呈现完成时间戳（711 帧泵消费时刻，
/// R-1 序障≥present；单调毫秒；未捕获=0）。出口 int lane 同上（供⑨-a）。
/// PLAN-779 同 begin_ms：provider 供值，core 零 UI 引用。
pub fn shim_frame_present_ms(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let observed = vm
        .native_interface
        .frame_provider()
        .map_or(0, |p| p.present_ms());
    task.ram.push_i32(observed.min(i32::MAX as i64) as i32);
    Ok(())
}

/// `shell_add_recent(path) -> Bool` shim.
pub fn shim_shell_add_recent(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path = pop_string_arg(task, vm);
    let ok = shell_add_recent(&path);
    task.ram.push_nv(auto_val::encode_bool(ok));
    Ok(())
}

pub fn shim_sys_cpu_usage(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let val = crate::libs::sys::cpu_usage();
    task.ram.push_f64(val);
    Ok(())
}

pub fn shim_sys_cpu_count(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let val = crate::libs::sys::cpu_count();
    task.ram.push_i32(val);
    Ok(())
}

pub fn shim_sys_cpu_core_usage(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let idx = pop_arg_i32(task) as usize;
    let val = crate::libs::sys::cpu_core_usage(idx);
    task.ram.push_f64(val);
    Ok(())
}

pub fn shim_sys_cpu_brand(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let brand = crate::libs::sys::cpu_brand();
    let idx = vm.add_string(brand.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

pub fn shim_sys_mem_total_mb(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let val = crate::libs::sys::mem_total_mb();
    task.ram.push_i32(val);
    Ok(())
}

pub fn shim_sys_mem_used_mb(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let val = crate::libs::sys::mem_used_mb();
    task.ram.push_i32(val);
    Ok(())
}

pub fn shim_sys_net_sent_kbs(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let val = crate::libs::sys::net_sent_kbs();
    task.ram.push_f64(val);
    Ok(())
}

pub fn shim_sys_net_recv_kbs(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let val = crate::libs::sys::net_recv_kbs();
    task.ram.push_f64(val);
    Ok(())
}

pub fn shim_sys_os_name(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let s = crate::libs::sys::os_name();
    let idx = vm.add_string(s.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

pub fn shim_sys_os_version(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let s = crate::libs::sys::os_version();
    let idx = vm.add_string(s.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

pub fn shim_sys_kernel_version(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let s = crate::libs::sys::kernel_version();
    let idx = vm.add_string(s.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

pub fn shim_sys_hostname(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let s = crate::libs::sys::hostname();
    let idx = vm.add_string(s.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

pub fn shim_sys_uptime_s(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let val = crate::libs::sys::uptime_s();
    task.ram.push_i32(val);
    Ok(())
}

pub fn shim_sys_kill(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let pid = pop_arg_i32(task);
    let ok = crate::libs::sys::kill(pid);
    task.ram.push_nv(auto_val::encode_bool(ok));
    Ok(())
}

pub fn shim_sys_processes(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::generic_registry::GenericInstanceData;
    use crate::vm::types::ListData;
    use auto_val::Value;

    let procs = crate::libs::sys::processes();
    let mut list: ListData<Value> = ListData::new();

    let field_names = vec![
        "pid".to_string(),
        "name".to_string(),
        "cpu".to_string(),
        "mem_mb".to_string(),
        "disk_kbs".to_string(),
        "net_kbs".to_string(),
        "status".to_string(),
        "user".to_string(),
    ];

    for p in procs {
        let fields = vec![
            Value::Int(p.pid),
            Value::Str(p.name.into()),
            Value::Double(p.cpu),
            Value::Int(p.mem_mb),
            Value::Double(p.disk_kbs),
            Value::Double(p.net_kbs),
            Value::Str(p.status.into()),
            Value::Str(p.user.into()),
        ];
        let inst = GenericInstanceData::new_with_names(
            "ProcInfo".to_string(),
            fields,
            field_names.clone(),
        );
        let inst_id = vm.insert_heap_object(inst);
        vm.rc_retain_id(inst_id as u64);
        list.push(Value::VmRef(auto_val::VmRef {
            id: inst_id as usize,
        }));
    }

    let list_id = vm.insert_heap_object(list);
    vm.rc_push_id(task, list_id as u64);
    Ok(())
}

pub fn shim_sys_disks(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::generic_registry::GenericInstanceData;
    use crate::vm::types::ListData;
    use auto_val::Value;

    let disks = crate::libs::sys::disks();
    let mut list: ListData<Value> = ListData::new();

    let field_names = vec![
        "name".to_string(),
        "mount".to_string(),
        "total_mb".to_string(),
        "avail_mb".to_string(),
    ];

    for d in disks {
        let fields = vec![
            Value::Str(d.name.into()),
            Value::Str(d.mount.into()),
            Value::Int(d.total_mb),
            Value::Int(d.avail_mb),
        ];
        let inst = GenericInstanceData::new_with_names(
            "DiskInfo".to_string(),
            fields,
            field_names.clone(),
        );
        let inst_id = vm.insert_heap_object(inst);
        vm.rc_retain_id(inst_id as u64);
        list.push(Value::VmRef(auto_val::VmRef {
            id: inst_id as usize,
        }));
    }

    let list_id = vm.insert_heap_object(list);
    vm.rc_push_id(task, list_id as u64);
    Ok(())
}

pub fn shim_sys_users(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::generic_registry::GenericInstanceData;
    use crate::vm::types::ListData;
    use auto_val::Value;

    let users = crate::libs::sys::users();
    let mut list: ListData<Value> = ListData::new();

    let field_names = vec!["name".to_string()];

    for u in users {
        let fields = vec![Value::Str(u.name.into())];
        let inst = GenericInstanceData::new_with_names(
            "UserInfo".to_string(),
            fields,
            field_names.clone(),
        );
        let inst_id = vm.insert_heap_object(inst);
        vm.rc_retain_id(inst_id as u64);
        list.push(Value::VmRef(auto_val::VmRef {
            id: inst_id as usize,
        }));
    }

    let list_id = vm.insert_heap_object(list);
    vm.rc_push_id(task, list_id as u64);
    Ok(())
}

/// `system_status() -> Int` — exit code of the last `system()` call.
pub fn shim_shell_system_status(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let code = if let Some(host) = &vm.host {
        host.system_status()
    } else {
        0
    };
    task.ram.push_i32(code);
    Ok(())
}

/// `export(key: String, val: String)` — set an environment variable.
pub fn shim_shell_export(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let val = pop_string_arg(task, vm);
    let key = pop_string_arg(task, vm);
    if let Some(host) = &vm.host {
        host.export(&key, &val);
    }
    Ok(())
}

/// `exit(code: Int)` — request the script to stop with `code`.
/// Raises `VMError::ExitRequested` so the current run unwinds immediately;
/// the host records the code for the process-exit propagation.
pub fn shim_shell_exit(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let code = auto_val::decode_i32(crate::vm::native::pop_arg_nv(task));
    if let Some(host) = &vm.host {
        host.exit(code);
    }
    Err(VMError::ExitRequested(code))
}

/// Runtime panic: pops a string (tagged index) from stack and returns it as an error.
/// Used by #[vm] function stubs when no matching native implementation is found.
pub fn shim_runtime_panic(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg = {
        let nv = crate::vm::native::pop_arg_nv(task);

        let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
        if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv);
            vm.get_string(idx)
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_else(|| format!("Runtime panic (invalid string index {})", idx))
        } else {
            format!("Runtime panic (unexpected stack value: {:?})", nv)
        }
    };
    Err(VMError::RuntimeError(msg))
}

/// `sched.set_timeout(callback, delay_ms) -> timer_id`
/// Stack: callback, delay_ms -> timer_id.
/// The callback is either a closure value (Int closure id — the CLOSURE
/// opcode pushes exactly that) or a widget event-name string; the timer
/// fires on the render loop's timer tick, so unlike `Time.sleep_ms` nothing
/// stalls the engine thread.
pub fn shim_sched_set_timeout(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let delay_ms = crate::vm::native::pop_arg_i32(task) as u64;
        let cb_nv = crate::vm::native::pop_arg_nv(task);
        let _stake_cb = crate::vm::native::StakeGuard::nv(vm, cb_nv);
        let callback = if auto_val::is_string(cb_nv) {
            let idx = auto_val::decode_string(cb_nv) as usize;
            let s = vm
                .strings
                .read()
                .unwrap()
                .get(idx)
                .cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .ok_or_else(|| VMError::RuntimeError("set_timeout: bad callback string".into()))?;
            crate::vm::engine::TimerCallback::Event(s)
        } else if auto_val::is_i32(cb_nv) {
            crate::vm::engine::TimerCallback::Closure(auto_val::decode_i32(cb_nv) as u32)
        } else {
            return Err(VMError::RuntimeError(
                "set_timeout: callback must be a closure or an event-name string".into(),
            ));
        };
        let timer_id = vm.set_timer(callback, delay_ms);
        task.ram.push_i32(timer_id as i32);
    }
    Ok(())
}

/// `sched.clear_timeout(timer_id) -> bool`
/// Stack: timer_id -> whether a pending timer was removed.
pub fn shim_sched_clear_timeout(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let id = crate::vm::native::pop_arg_i32(task) as u32;
        let removed = vm.clear_timer(id);
        task.ram.push_nv(auto_val::encode_bool(removed));
    }
    Ok(())
}

/// `env.var(key) -> value or None` — rust-form `env::var` bridge (Plan 442
/// C2; musk backend .at sources call std::env::var via use.rust). Rust's
/// env::var returns Result; the VM-native convention collapses
/// Result-returning stdlib shims to Option shape at the producer boundary,
/// so the caller's `.ok()` chain step is the engine's identity passthrough.
pub fn shim_env_var(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let key_nv = crate::vm::native::pop_arg_nv(task);
        let _stake = crate::vm::native::StakeGuard::nv(vm, key_nv);
        let key = if auto_val::is_string(key_nv) {
            let idx = auto_val::decode_string(key_nv) as usize;
            vm.strings
                .read()
                .unwrap()
                .get(idx)
                .cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            auto_val::decode_i32(key_nv).to_string()
        };
        match std::env::var(&key) {
            Ok(v) => {
                let idx = vm.add_string(v.into_bytes());
                vm.rc_push_str_idx(task, idx); // Plan 510 G1-2: 返回串入栈配平
            }
            Err(_) => {
                task.ram.push_nv(auto_val::encode_null());
            }
        }
    }
    Ok(())
}

/// Plan 060(api.at 契约归一):merged 模式 shell 执行提交。
/// shell.at 的 run_command 调 auto.shell.exec_submit(block_id, cmd, cwd),
/// 请求进 vm::shell_bridge 队列,由 renderer 的 merged_exec_loop 取走执行。
/// Stack: int(block_id), str_idx(cmd), str_idx(cwd) ->
pub fn shim_shell_exec_submit(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let cwd_idx = task.ram.pop_str_idx() as u32;
    let cmd_idx = task.ram.pop_str_idx() as u32;
    let block_id = crate::vm::native::pop_arg_i32(task) as i64;

    let _stake_block_id = crate::vm::native::StakeGuard::new(vm, block_id as i64 as u64);
    let cmd = vm
        .get_string(cmd_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let cwd = vm
        .get_string(cwd_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    crate::vm::shell_bridge::submit(crate::vm::shell_bridge::ShellExecRequest {
        kind: crate::vm::shell_bridge::ShellExecKind::Process,
        block_id,
        cmd,
        cwd,
        result_json: String::new(),
    });
    Ok(())
}

/// Plan 060(M2):merged 模式 builtin 直发 —— shell.at 对 ls/cd/pwd 等
/// 已在 .at 侧算好语义结果,经此把完整 command_result payload JSON 压入
/// 队列(执行线程直发,不 spawn)。Stack: int(block_id), str_idx(json) ->
pub fn shim_shell_emit_result(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let json_idx = task.ram.pop_str_idx() as u32;
    let block_id = crate::vm::native::pop_arg_i32(task) as i64;

    let _stake_block_id = crate::vm::native::StakeGuard::new(vm, block_id as i64 as u64);
    let json = vm
        .get_string(json_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    crate::vm::shell_bridge::submit(crate::vm::shell_bridge::ShellExecRequest {
        kind: crate::vm::shell_bridge::ShellExecKind::Result,
        block_id,
        cmd: String::new(),
        cwd: String::new(),
        result_json: json,
    });
    Ok(())
}

/// 2026-08-22(show 下沉):读文件 + 逐行高亮 + Code payload 全在 Rust 侧完成。
/// 动机:.at 侧 `js = js + ...` 拼 ~100KB payload 巨串会触发 VM 字符串堆损坏
/// (实测:会话 cwd 被写成单字符、payload 丢失、块永挂 Running、进程退出)。
/// shell.at 只做路径归一后调本 native。Stack: int(block_id), str_idx(path),
/// str_idx(cwd) ->
pub fn shim_shell_emit_show(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let cwd_idx = task.ram.pop_str_idx() as u32;
    let path_idx = task.ram.pop_str_idx() as u32;
    let block_id = crate::vm::native::pop_arg_i32(task) as i64;

    let _stake_block_id = crate::vm::native::StakeGuard::new(vm, block_id as i64 as u64);
    let path = vm
        .get_string(path_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let cwd = vm
        .get_string(cwd_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    crate::vm::shell_bridge::submit(crate::vm::shell_bridge::ShellExecRequest {
        kind: crate::vm::shell_bridge::ShellExecKind::Result,
        block_id,
        cmd: String::new(),
        cwd: String::new(),
        result_json: crate::vm::shell_bridge::show_result_json(block_id, &cwd, &path),
    });
    Ok(())
}

/// Plan 060 M3(2026-08-22):通用宿主桥 —— merged 模式"进程内直调"。
/// ash-runner(auto-shell 仓,链接 ash_server)把 api.at 契约端点注册进
/// host_bridge;本 native 查表直调,返回 JSON 串。参数/返回编码与 HTTP
/// 模式同构(json body / json 响应),前端桩可复用 json.to_value。
/// Stack: str_idx(name), str_idx(args_json) -> str_idx(response_json)
pub fn shim_host_call(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let args_idx = task.ram.pop_str_idx() as u32;
    let name_idx = task.ram.pop_str_idx() as u32;
    let name = vm
        .get_string(name_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let args_json = vm
        .get_string(args_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    match crate::vm::host_bridge::call_host(&name, &args_json) {
        Ok(resp) => {
            let idx = vm.add_string(resp.into_bytes());
            vm.rc_push_str_idx(task, idx as usize);
            Ok(())
        }
        Err(e) => Err(VMError::RuntimeError(format!("host.call {}: {}", name, e))),
    }
}

/// Plan 060 M3:同 shim_host_call,但把 JSON 响应直接转换为 VM 值入栈
/// (复用 json.to_value 的转换管线)—— 前端桩免一层 json.to_value 调用。
/// Stack: str_idx(name), str_idx(args_json) -> value
pub fn shim_host_call_value(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let args_idx = task.ram.pop_str_idx() as u32;
    let name_idx = task.ram.pop_str_idx() as u32;
    let name = vm
        .get_string(name_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let args_json = vm
        .get_string(args_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let resp = crate::vm::host_bridge::call_host(&name, &args_json)
        .map_err(|e| VMError::RuntimeError(format!("host.call_value {}: {}", name, e)))?;
    let parsed: serde_json::Value = serde_json::from_str(&resp).map_err(|e| {
        VMError::RuntimeError(format!(
            "host.call_value {}: response parse error: {}",
            name, e
        ))
    })?;
    crate::vm::ffi::stdlib::json_to_vm_value(task, vm, &parsed, 0)
}
