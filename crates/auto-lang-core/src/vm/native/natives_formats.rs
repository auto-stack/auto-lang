//! PLAN-843 结构拆分：native formats 域实现（自 native.rs 原样搬移）。
use super::*;

/// Version.parse(ver_str) → opaque handle
/// Stack: [ver_str] -> [handle_i32]
pub fn shim_semver_opaque_parse(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let ver_str = pop_vm_string(task, vm);
    match semver::Version::parse(&ver_str) {
        Ok(ver) => {
            let obj = RustStdlibObject::new("semver::Version", std::sync::Mutex::new(ver));
            let id = vm.insert_heap_object(obj);
            vm.rc_push(task, auto_val::encode_object(id as u32));
        }
        Err(e) => {
            return Err(VMError::RuntimeError(format!(
                "Version::parse failed: {}",
                e
            )));
        }
    }
    Ok(())
}

/// Stack: [handle_i32] -> [major_i32]
pub fn shim_semver_opaque_major(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let ver_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_ver_id = crate::vm::native::StakeGuard::new(vm, ver_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(ver_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(ver) = rso.downcast_ref::<std::sync::Mutex<semver::Version>>() {
                task.ram.push_i32(ver.lock().unwrap().major as i32);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// v.minor → u64
/// Stack: [handle_i32] -> [minor_i32]
pub fn shim_semver_opaque_minor(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let ver_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_ver_id = crate::vm::native::StakeGuard::new(vm, ver_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(ver_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(ver) = rso.downcast_ref::<std::sync::Mutex<semver::Version>>() {
                task.ram.push_i32(ver.lock().unwrap().minor as i32);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// v.patch → u64
/// Stack: [handle_i32] -> [patch_i32]
pub fn shim_semver_opaque_patch(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let ver_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_ver_id = crate::vm::native::StakeGuard::new(vm, ver_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(ver_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(ver) = rso.downcast_ref::<std::sync::Mutex<semver::Version>>() {
                task.ram.push_i32(ver.lock().unwrap().patch as i32);
                return Ok(());
            }
        }
    }
    task.ram.push_i32(0);
    Ok(())
}

/// v.pre → string
/// Stack: [handle_i32] -> [pre_str]
pub fn shim_semver_opaque_pre(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let ver_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_ver_id = crate::vm::native::StakeGuard::new(vm, ver_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(ver_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(ver) = rso.downcast_ref::<std::sync::Mutex<semver::Version>>() {
                push_vm_string(task, vm, &ver.lock().unwrap().pre.to_string());
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// v.to_string() → string
/// Stack: [handle_i32] -> [ver_str]
pub fn shim_semver_opaque_to_string(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let ver_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_ver_id = crate::vm::native::StakeGuard::new(vm, ver_id as i64 as u64);
    if let Some(obj) = vm.get_heap_object(ver_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(ver) = rso.downcast_ref::<std::sync::Mutex<semver::Version>>() {
                push_vm_string(task, vm, &ver.lock().unwrap().to_string());
                return Ok(());
            }
        }
    }
    push_vm_string(task, vm, "");
    Ok(())
}

/// v1 > v2 → bool
/// Stack: [v2_handle, v1_handle] -> [bool_i32]
pub fn shim_semver_opaque_cmp_gt(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let v2_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_v2_id = crate::vm::native::StakeGuard::new(vm, v2_id as i64 as u64);
    let v1_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_v1_id = crate::vm::native::StakeGuard::new(vm, v1_id as i64 as u64);

    let v1 = if let Some(obj) = vm.get_heap_object(v1_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(ver) = rso.downcast_ref::<std::sync::Mutex<semver::Version>>() {
                Some(ver.lock().unwrap().clone())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let v2 = if let Some(obj) = vm.get_heap_object(v2_id) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard.as_any().downcast_ref::<RustStdlibObject>() {
            if let Some(ver) = rso.downcast_ref::<std::sync::Mutex<semver::Version>>() {
                Some(ver.lock().unwrap().clone())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    match (v1, v2) {
        (Some(a), Some(b)) => task.ram.push_nv(auto_val::encode_bool(a > b)),
        _ => task.ram.push_nv(auto_val::encode_bool(false)),
    }
    Ok(())
}

/// version.drop() — no-op, GC handles cleanup
pub fn shim_semver_opaque_drop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Plan 419: 语义更新 —— rc 释放(RC 归零才真回收;单持有时即手动提前释放)。
    let id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_id = crate::vm::native::StakeGuard::new(vm, id as i64 as u64);
    vm.rc_release_id(id);
    Ok(())
}

/// VersionReq.parse(">=1.0") → opaque VersionReq handle
/// Stack: [spec_str] -> [handle_i32]
pub fn shim_semver_opaque_versionreq_parse(
    task: &mut AutoTask,
    vm: &AutoVM,
) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let spec_str = pop_vm_string(task, vm);
    match semver::VersionReq::parse(&spec_str) {
        Ok(req) => {
            let obj = RustStdlibObject::new("semver::VersionReq", std::sync::Mutex::new(req));
            let id = vm.insert_heap_object(obj);
            vm.rc_push(task, auto_val::encode_object(id as u32));
        }
        Err(e) => {
            return Err(VMError::RuntimeError(format!(
                "VersionReq::parse failed: {}",
                e
            )));
        }
    }
    Ok(())
}

/// req.matches(version_handle) → 1 if true, 0 if false
/// Stack: [req_handle, version_handle] -> [result_i32]
pub fn shim_semver_opaque_versionreq_matches(
    task: &mut AutoTask,
    vm: &AutoVM,
) -> Result<(), VMError> {
    use crate::vm::ffi::rust_stdlib::RustStdlibObject;

    let ver_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_ver_id = crate::vm::native::StakeGuard::new(vm, ver_id as i64 as u64);
    let req_id = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_req_id = crate::vm::native::StakeGuard::new(vm, req_id as i64 as u64);

    let version: Option<semver::Version> = (|| {
        let obj = vm.get_heap_object(ver_id)?;
        let guard = obj.read().ok()?;
        let rso = guard.as_any().downcast_ref::<RustStdlibObject>()?;
        let mutex = rso.downcast_ref::<std::sync::Mutex<semver::Version>>()?;
        let locked = mutex.lock().ok()?;
        Some(locked.clone())
    })();

    let req: Option<semver::VersionReq> = (|| {
        let obj = vm.get_heap_object(req_id)?;
        let guard = obj.read().ok()?;
        let rso = guard.as_any().downcast_ref::<RustStdlibObject>()?;
        let mutex = rso.downcast_ref::<std::sync::Mutex<semver::VersionReq>>()?;
        let locked = mutex.lock().ok()?;
        Some(locked.clone())
    })();

    match (version, req) {
        (Some(ver), Some(req)) => {
            task.ram.push_nv(auto_val::encode_bool(req.matches(&ver)));
        }
        _ => {
            task.ram.push_nv(auto_val::encode_bool(false));
        }
    }
    Ok(())
}

/// Parse CSV text into JSON array of arrays.
/// Stack: str_idx -> str_idx (JSON)
pub fn shim_csv_parse(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_idx = task.ram.pop_str_idx() as u32;
    let text = if let Some(bytes) = vm.get_string(str_idx) {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        String::new()
    };
    let rows: Vec<Vec<String>> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.split(',').map(|f| f.trim().to_string()).collect())
        .collect();
    let json = serde_json::to_string(&rows)
        .map_err(|e| VMError::RuntimeError(format!("csv_parse: {}", e)))?;
    let str_idx = vm.add_string(json.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// Parse CSV text with custom delimiter.
/// Stack: str_idx(delimiter), str_idx(text) -> str_idx (JSON)
pub fn shim_csv_parse_delim(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let delim_idx = task.ram.pop_str_idx() as u32;
    let text_idx = task.ram.pop_str_idx() as u32;
    let delim = if let Some(bytes) = vm.get_string(delim_idx) {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        ",".to_string()
    };
    let text = if let Some(bytes) = vm.get_string(text_idx) {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        String::new()
    };
    let delim_char = delim.chars().next().unwrap_or(',');
    let rows: Vec<Vec<String>> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.split(delim_char)
                .map(|f| f.trim().to_string())
                .collect()
        })
        .collect();
    let json = serde_json::to_string(&rows)
        .map_err(|e| VMError::RuntimeError(format!("csv_parse_delim: {}", e)))?;
    let str_idx = vm.add_string(json.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// Encode list of list of strings (JSON) to CSV.
/// Stack: str_idx (JSON) -> str_idx (CSV)
pub fn shim_csv_encode(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let json_idx = task.ram.pop_str_idx() as u32;
    let json = if let Some(bytes) = vm.get_string(json_idx) {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        String::new()
    };
    let rows: Vec<Vec<String>> = serde_json::from_str(&json)
        .map_err(|e| VMError::RuntimeError(format!("csv_encode: {}", e)))?;
    let mut result = String::new();
    for (i, row) in rows.iter().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        result.push_str(&row.join(","));
    }
    let str_idx = vm.add_string(result.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// Encode CSV with custom delimiter.
/// Stack: str_idx(delim), str_idx(JSON) -> str_idx (CSV)
pub fn shim_csv_encode_delim(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let delim_idx = task.ram.pop_str_idx() as u32;
    let json_idx = task.ram.pop_str_idx() as u32;
    let delim = if let Some(bytes) = vm.get_string(delim_idx) {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        ",".to_string()
    };
    let json = if let Some(bytes) = vm.get_string(json_idx) {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        String::new()
    };
    let rows: Vec<Vec<String>> = serde_json::from_str(&json)
        .map_err(|e| VMError::RuntimeError(format!("csv_encode_delim: {}", e)))?;
    let mut result = String::new();
    for (i, row) in rows.iter().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        result.push_str(&row.join(&delim));
    }
    let str_idx = vm.add_string(result.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// MD5 hash.
/// Stack: str_idx -> str_idx (hex digest)
pub fn shim_hash_md5(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_idx = task.ram.pop_str_idx() as u32;
    let data = if let Some(bytes) = vm.get_string(str_idx) {
        bytes.clone()
    } else {
        Vec::new()
    };
    // Simple MD5 implementation using digest
    let digest = md5_hash(&data);
    let str_idx = vm.add_string(digest.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// SHA1 hash.
/// Stack: str_idx -> str_idx (hex digest)
pub fn shim_hash_sha1(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_idx = task.ram.pop_str_idx() as u32;
    let data = if let Some(bytes) = vm.get_string(str_idx) {
        bytes.clone()
    } else {
        Vec::new()
    };
    let digest = sha1_hash(&data);
    let str_idx = vm.add_string(digest.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// SHA256 one-shot hash.
/// Stack: str_idx -> str_idx (hex digest)
pub fn shim_hash_sha256(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_idx = task.ram.pop_str_idx() as u32;
    let data = if let Some(bytes) = vm.get_string(str_idx) {
        bytes.clone()
    } else {
        Vec::new()
    };
    let digest = sha256_hash(&data);
    let str_idx = vm.add_string(digest.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// SHA512 hash.
/// Stack: str_idx -> str_idx (hex digest)
pub fn shim_hash_sha512(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let str_idx = task.ram.pop_str_idx() as u32;
    let data = if let Some(bytes) = vm.get_string(str_idx) {
        bytes.clone()
    } else {
        Vec::new()
    };
    let digest = sha512_hash(&data);
    let str_idx = vm.add_string(digest.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

// Simple hash implementations using pure Rust
fn md5_hash(data: &[u8]) -> String {
    use md5::{Digest, Md5};
    let mut hasher = Md5::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:02x}", result)
}

fn sha1_hash(data: &[u8]) -> String {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:02x}", result)
}

fn sha256_hash(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:02x}", result)
}

fn sha512_hash(data: &[u8]) -> String {
    use sha2::{Digest, Sha512};
    let mut hasher = Sha512::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:02x}", result)
}

/// HMAC-SHA256.
/// Stack: str_idx(key), str_idx(data) -> str_idx (hex digest)
pub fn shim_hash_hmac_sha256(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key_idx = task.ram.pop_str_idx() as u32;
    let data_idx = task.ram.pop_str_idx() as u32;
    let key = vm.get_string(key_idx).unwrap_or_default();
    let data = vm.get_string(data_idx).unwrap_or_default();
    let digest = hmac_sha256_hash(&key, &data);
    let str_idx = vm.add_string(digest.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// MD5 hash of file contents.
/// Stack: str_idx(path) -> str_idx (hex digest)
pub fn shim_hash_file_md5(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u32;
    let path = vm
        .get_string(path_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let data = std::fs::read(&path)
        .map_err(|e| VMError::RuntimeError(format!("file_md5: {}: {}", path, e)))?;
    let digest = md5_hash(&data);
    let str_idx = vm.add_string(digest.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// SHA256 hash of file contents.
/// Stack: str_idx(path) -> str_idx (hex digest)
pub fn shim_hash_file_sha256(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u32;
    let path = vm
        .get_string(path_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let data = std::fs::read(&path)
        .map_err(|e| VMError::RuntimeError(format!("file_sha256: {}: {}", path, e)))?;
    let digest = sha256_hash(&data);
    let str_idx = vm.add_string(digest.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

fn hmac_sha256_hash(key: &[u8], data: &[u8]) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data);
    let result = mac.finalize();
    format!("{:02x}", result.into_bytes())
}
