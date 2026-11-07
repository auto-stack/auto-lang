//! PLAN-843 结构拆分：strings_json 域实现（原样搬移，算法/行为零变化）。
use super::*;

/// Get string length in **UTF-8 bytes** (PLAN-872 实测钉死：本纯 shim 经
/// inventory 绑定 id 1500，是 `Str.len(s)`/静态可解析 `s.len()` 的生效实现；
/// 字符数语义在 `.length` 属性（nat#170，PLAN-055）与 CALL_SPEC 内联 "len"
/// 臂——见 attachments/872/binding-table.md，勿混用)。
#[auto_macros::rust_fn("Str.len")]
pub fn shim_str_len(s: String) -> i32 {
    s.len() as i32
}

/// Check if string is empty
#[auto_macros::rust_fn("Str.is_empty")]
pub fn shim_str_is_empty(s: String) -> bool {
    s.is_empty()
}

/// Check if the string is exactly one ASCII digit character (0-9).
/// Plan 504: static-dispatch predicate for UI tracks (e.g. 011-calculator) —
/// Vue side maps to `/^[0-9]$/.test(s)` in ts_adapter.
#[auto_macros::rust_fn("Str.is_digit")]
pub fn shim_str_is_digit(s: String) -> bool {
    s.len() == 1 && s.as_bytes()[0].is_ascii_digit()
}

/// UTF-8 byte length. PLAN-781 阶段 E（T-24）：Reader 章 meta 索引的
/// 字节偏移累计专用——PLAN-055 的**字符数**语义在 `.length` 属性
/// （nat#170）与 CALL_SPEC 内联 "len" 臂，**不在** `str.len()`（其生效实现
/// 是上方 id 1500 纯 shim，同为字节数；PLAN-872 实测订正本注释旧表述）、
/// `str.bytes()` 返回 VM 列表在大文本（10MiB 书）不可用；Rust
/// `String::len()` 即 UTF-8 字节数，与 `File.read_text_range`（nat#1016，
/// PLAN-687）的字节偏移语义天然一致。
#[auto_macros::rust_fn("Str.byte_len", "auto.str.byte_len")]
pub fn shim_str_byte_len(s: String) -> i32 {
    s.len() as i32
}

/// Get character at byte index (returns single-char string)
///
/// Plan 434 (AA2R): boundary-safe — a byte index inside a multi-byte char
/// returns 0 instead of panicking (`s[i..]` panicked on non-boundary; the
/// v2 lexer byte-wise scans hit it on CJK comment text the first time
/// AA2R lexed the lib source itself). ASCII behavior unchanged.
#[auto_macros::rust_fn("Str.char_at")]
pub fn shim_str_char_at(s: String, index: i32) -> i32 {
    if index < 0 || index as usize >= s.len() {
        return 0;
    }
    match s.get(index as usize..).and_then(|t| t.chars().next()) {
        Some(c) => c as i32,
        None => 0,
    }
}

/// Get substring (byte indices)
///
/// str-parity (audit B1 finding): `(start, LEN)` semantics, matching
/// a2r_std::str_substr — the old `(start, END)` reading made the same
/// `.substr` call mean different things in VM vs transpiled Rust
/// (013/023 token helpers hit this: `substr(6, 5)` gave "world" in Rust but
/// "" in the VM). Use `.slice(start, end)` for END semantics — that one is
/// unified across both backends.
///
/// Plan 057: clamp indices to the nearest char boundaries and return the
/// (possibly empty) slice instead of panicking when a byte index lands
/// mid-char in multi-byte UTF-8 input.
///
/// PLAN-995 T-01（H6）：start 的 clamp 方向改为**回退**（lo -= 1）。
/// 原前跳（lo += 1）在步进读缝隙丢整字：跨块中文字符被上一块的 end 回退
/// 排除、又被下一块的 start 前跳跳过——分块 substr 重建比原文短
/// （h7-diag 实证 66354→66348，分歧点即 CJK）。双向回退后
/// s[boundary↓(start)..boundary↓(start+len)] 使步进覆盖无损。
#[auto_macros::rust_fn("Str.substr")]
pub fn shim_str_substr(s: String, start: i32, len: i32) -> String {
    if start < 0 || len < 0 || start as usize > s.len() {
        return String::new();
    }
    let mut lo = start as usize;
    let mut hi = (start as usize + len as usize).min(s.len());
    // Debug preview must itself be boundary-safe: truncating at a raw byte
    // count would panic the same way the slice below used to.
    let preview: String = s.chars().take(16).collect();
    while lo > 0 && !s.is_char_boundary(lo) {
        if std::env::var("ASH_DEBUG_SUBSTR").is_ok() {
            eprintln!(
                "[SUBSTR-CLAMP] start {} inside multi-byte char of {:?}",
                lo, preview
            );
        }
        lo -= 1;
    }
    while hi > 0 && !s.is_char_boundary(hi) {
        if std::env::var("ASH_DEBUG_SUBSTR").is_ok() {
            eprintln!(
                "[SUBSTR-CLAMP] end {} inside multi-byte char of {:?}",
                hi, preview
            );
        }
        hi -= 1;
    }
    if hi < lo {
        return String::new();
    }
    s[lo..hi].to_string()
}

/// END-semantics substring for the `.slice(start, end)` / `.sub(start, end)`
/// spellings — matches a2r's `s[start..end]` / `&s[start..end]` emission.
/// Same Plan 057 boundary clamps as the LEN sibling above；PLAN-995 T-01（H6）
/// start 同样回退式 clamp（与 substr 一致，步进无缝隙丢字）。
#[auto_macros::rust_fn("Str.slice")]
pub fn shim_str_slice(s: String, start: i32, end: i32) -> String {
    if start < 0 || end < start || start as usize > s.len() || end as usize > s.len() {
        return String::new();
    }
    let mut lo = start as usize;
    let mut hi = end as usize;
    while lo > 0 && !s.is_char_boundary(lo) {
        lo -= 1;
    }
    while hi > 0 && !s.is_char_boundary(hi) {
        hi -= 1;
    }
    if hi < lo {
        return String::new();
    }
    s[lo..hi].to_string()
}

/// Trim whitespace from string
#[auto_macros::rust_fn("Str.trim")]
pub fn shim_str_trim(s: String) -> String {
    s.trim().to_string()
}

/// Split string by delimiter
#[auto_macros::rust_fn("Str.split")]
pub fn shim_str_split(s: String, delimiter: String) -> Vec<String> {
    s.split(&delimiter).map(|p| p.to_string()).collect()
}

/// Repeat string n times
#[auto_macros::rust_fn("Str.repeat")]
pub fn shim_str_repeat(s: String, n: i32) -> String {
    if n < 0 {
        String::new()
    } else {
        s.repeat(n as usize)
    }
}

/// Replace all occurrences of a pattern in a string
#[auto_macros::rust_fn("Str.replace")]
pub fn shim_str_replace(s: String, from: String, to: String) -> String {
    s.replace(&from, &to)
}

/// Convert string to uppercase
#[auto_macros::rust_fn("Str.to_upper")]
pub fn shim_str_to_upper(s: String) -> String {
    s.to_uppercase()
}

/// Convert string to lowercase
#[auto_macros::rust_fn("Str.to_lower")]
pub fn shim_str_to_lower(s: String) -> String {
    s.to_lowercase()
}

/// Reverse a string (unicode-aware)
#[auto_macros::rust_fn("Str.reverse")]
pub fn shim_str_reverse(s: String) -> String {
    s.chars().rev().collect()
}

/// Find first occurrence of substring with optional start position.
/// Supports both 2-arg (find(s, needle)) and 3-arg (s.find(needle, start_pos)) calling conventions.
/// Detects whether top-of-stack is start_pos (i32) or needle (string) to handle both cases.
pub fn shim_str_find_manual(
    task: &mut crate::vm::task::AutoTask,
    vm: &crate::vm::engine::AutoVM,
) -> Result<(), crate::vm::engine::VMError> {
    use crate::vm::ffi::convert::VMConvertible;

    // Peek at top of stack to determine calling convention
    let start_pos: i64 = {
        let top = task.ram.read_nv(task.ram.sp - 1);
        if auto_val::is_i32(top) {
            crate::vm::native::pop_arg_nv(task); // consume start_pos
            auto_val::decode_i32(top) as i64
        } else {
            0i64 // no start_pos, default to 0
        }
    };

    // Pop needle (String)
    let needle: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| crate::vm::engine::VMError::RuntimeError(e.to_string()))?;

    // Pop receiver (String)
    let s: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| crate::vm::engine::VMError::RuntimeError(e.to_string()))?;

    let result = if start_pos > 0 && (start_pos as usize) < s.len() {
        s[start_pos as usize..]
            .find(&needle)
            .map(|i| start_pos as i32 + i as i32)
            .unwrap_or(-1)
    } else {
        s.find(&needle).map(|i| i as i32).unwrap_or(-1)
    };

    // Push result
    task.ram.push_nv(auto_val::encode_i32(result));

    Ok(())
}

/// Split string into lines
#[auto_macros::rust_fn("Str.lines")]
pub fn shim_str_lines(s: String) -> Vec<String> {
    s.lines().map(|l| l.to_string()).collect()
}

/// Parse string as integer
#[auto_macros::rust_fn("Str.parse_int")]
pub fn shim_str_parse_int(s: String) -> Result<i64, String> {
    s.trim()
        .parse::<i64>()
        .map_err(|e| format!("Str.parse_int failed: {}", e))
}

/// Parse string as float
#[auto_macros::rust_fn("Str.parse_float")]
pub fn shim_str_parse_float(s: String) -> Result<f64, String> {
    s.trim()
        .parse::<f64>()
        .map_err(|e| format!("Str.parse_float failed: {}", e))
}

/// Split string at first occurrence of delimiter, returns empty list if not found
#[auto_macros::rust_fn("Str.split_once")]
pub fn shim_str_split_once(s: String, delimiter: String) -> Vec<String> {
    match s.split_once(&delimiter) {
        Some((before, after)) => vec![before.to_string(), after.to_string()],
        None => vec![],
    }
}

/// Count non-overlapping occurrences of pattern in string
#[auto_macros::rust_fn("Str.match_count")]
pub fn shim_str_match_count(s: String, pattern: String) -> i32 {
    s.matches(&pattern).count() as i32
}

/// Replace first occurrence of from with to
#[auto_macros::rust_fn("Str.replace_first")]
pub fn shim_str_replace_first(s: String, from: String, to: String) -> String {
    s.replacen(&from, &to, 1)
}

/// Generate a UUID-like string from timestamp and random components
#[auto_macros::rust_fn("Str.uuid")]
pub fn shim_str_uuid() -> String {
    ::a2r_std::uuid()
}

/// Convert unsigned integer to string
#[auto_macros::rust_fn("Str.from_uint")]
pub fn shim_str_from_uint(n: i64) -> String {
    n.to_string()
}

/// Parse string as unsigned integer
#[auto_macros::rust_fn("Str.to_uint")]
pub fn shim_str_to_uint(s: String) -> i64 {
    s.trim().parse::<i64>().unwrap_or(0)
}

/// Check if character is alphabetic
#[auto_macros::rust_fn("Char.is_alpha")]
pub fn shim_char_is_alpha(codepoint: i32) -> bool {
    char::from_u32(codepoint as u32).map_or(false, |c| c.is_alphabetic())
}

/// Check if character is a digit
#[auto_macros::rust_fn("Char.is_digit")]
pub fn shim_char_is_digit(codepoint: i32) -> bool {
    char::from_u32(codepoint as u32).map_or(false, |c| c.is_ascii_digit())
}

/// Check if character is alphanumeric
#[auto_macros::rust_fn("Char.is_alphanum")]
pub fn shim_char_is_alphanum(codepoint: i32) -> bool {
    char::from_u32(codepoint as u32).map_or(false, |c| c.is_alphanumeric())
}

/// Check if character is whitespace
#[auto_macros::rust_fn("Char.is_whitespace")]
pub fn shim_char_is_whitespace(codepoint: i32) -> bool {
    char::from_u32(codepoint as u32).map_or(false, |c| c.is_whitespace())
}

/// Check if character is valid for an identifier start or continue
#[auto_macros::rust_fn("Char.is_ident")]
pub fn shim_char_is_ident(codepoint: i32) -> bool {
    char::from_u32(codepoint as u32).map_or(false, |c| c.is_alphanumeric() || c == '_')
}

/// Convert character to lowercase
#[auto_macros::rust_fn("Char.to_lower")]
pub fn shim_char_to_lower(codepoint: i32) -> i32 {
    if let Some(c) = char::from_u32(codepoint as u32) {
        if let Some(lower) = c.to_lowercase().next() {
            return lower as i32;
        }
    }
    codepoint
}

/// Convert character to uppercase
#[auto_macros::rust_fn("Char.to_upper")]
pub fn shim_char_to_upper(codepoint: i32) -> i32 {
    if let Some(c) = char::from_u32(codepoint as u32) {
        if let Some(upper) = c.to_uppercase().next() {
            return upper as i32;
        }
    }
    codepoint
}

/// Convert a Unicode codepoint to its single-character string.
/// Plan 368 W5: `s.char_at(i)` returns the codepoint as `int`; this gives the
/// ergonomic codepoint→1-char-str conversion (replaces the StringBuilder dance
/// in c_process_app). Invalid codepoints yield the empty string.
#[auto_macros::rust_fn("Char.to_str", "char.to_str", "auto.char.to_str")]
pub fn shim_char_to_str(codepoint: i32) -> String {
    char::from_u32(codepoint as u32)
        .map(|c| c.to_string())
        .unwrap_or_default()
}

/// Encode a value to JSON string.
/// PLAN-734 T-07（E2 语义修复）：VM 值感知——数值按数值、复合经共享
/// vm_value_to_json（i64/u64/bigint 全域已修）；字符串值仍按 JSON 字符串。
#[auto_macros::rust_fn("Json.encode")]
pub fn shim_json_encode(value: String) -> String {
    // A String is always a JSON string. The VM override below serializes
    // actual primitive values before the legacy String ABI can erase tags.
    serde_json::to_string(&value).unwrap_or_default()
}

/// Parse a JSON string into a value (placeholder)
#[auto_macros::rust_fn("Json.parse")]
pub fn shim_json_parse(s: String) -> String {
    // For now, return the string as-is (placeholder)
    // Full implementation would return a JsonValue handle
    s
}

/// Plan 446 批三 D1: vm 感知的 `json.parse`——接通 Plan 340 的 JSON↔VM 转换器。
/// 占位 shim（上方 rust_fn 版）原样返回字符串，下游点访问/方法链全是垃圾
/// （os-config 现场：body.provider.kind == 0，"看似可用"的最坏形态）。
/// 本版把 JSON 文本物化为 VM 堆值：对象→`__json_object` GenericInstanceData、
/// 数组→ListData<Value>（GET_FIELD/for-in 运行期直读，无需注册模板）。
/// 非法 JSON 显式报错（占位时代静默透传是现场定位成本的主要来源）。
/// 注册在 engine 的 inventory 覆盖块（build_from_inventory 之后），与
/// str.contains 等 nanbox 覆盖同款惯例。
pub fn shim_json_parse_vm(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let s: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(format!("json.parse: {}", e)))?;
    // PLAN-995 T-02：单趟直驱物化（无中间 serde_json::Value 树）。
    parse_json_direct(task, vm, &s).map_err(map_parse_error)
}

/// `auto.json.to_value(json_str) -> NanoValue`
/// Parse a JSON string and push the resulting VM value onto the stack.
pub fn shim_json_to_value(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let nv = task.ram.pop_nv();
    // PLAN-042 T-08（P041-D6 根修）+ PLAN-043 Phase 2 扩面：to_value
    // 幂等——VM 值直通不转串。PLAN-080 F-2③ 起 `Http.get_json(url)`
    // 编译期内联 json.to_value（web 轨 fetch().json() 语义），文档配方
    // `json.to_value(Http.get_json(url))`（PLAN-617 T-10 双端同源配方）
    // 自此成为**二次转换**：旧 shim 把 Obj 强转 String 解析失败 → 静默
    // null → entries 恒空（P041-D6 病灶：音乐曲库 entries=0、延续值呈
    // 20 位数字串=NV 位型）。042 修 object/list；043 扩 bool/null
    // （auto-os PLAN-043 Part 2 实证链：双重解析全族根因）。幂等直通
    // 使显式/隐式两种调用形态等价，存量 .at 消费方零改绿。
    if auto_val::is_object(nv)
        || auto_val::is_list(nv)
        || auto_val::is_bool(nv)
        || auto_val::is_null(nv)
    {
        task.ram.push_nv(nv);
        return Ok(());
    }
    let _stake = crate::vm::native::StakeGuard::nv(vm, nv);
    task.ram.push_nv(nv);
    let json_str: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let trimmed = json_str.trim();
    if trimmed.is_empty() || trimmed == "undefined" || trimmed == "None" || trimmed == "nil" {
        task.ram.push_nv(auto_val::encode_null());
        return Ok(());
    }
    let parsed_direct = parse_json_direct(task, vm, &json_str);
    if let Err(e) = parsed_direct {
        let msg = e.to_string();
        if msg.starts_with("json_to_vm_value") {
            // 深度类错误：旧路径经 json_to_vm_value 的 `?` 原样上抛
            // （语法错误才走 WARN+null 兜底）——保持错误分类不变。
            return Err(VMError::RuntimeError(msg));
        }
        eprintln!(
            "[WARN] json.to_value: invalid JSON {:?} on payload: {:?}",
            e, json_str
        );
        task.ram.push_nv(auto_val::encode_null());
        return Ok(());
    }
    Ok(())
}

/// `auto.json.from_value(NanoValue) -> String`
/// Serialize a VM value (incl. heap objects) into a JSON string.
pub fn shim_json_from_value(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop the raw stack value and decode it.
    let nv = crate::vm::native::pop_arg_nv(task);

    let _stake_nv = crate::vm::native::StakeGuard::nv(vm, nv);
    let value = if auto_val::is_object(nv) {
        auto_val::Value::VmRef(auto_val::VmRef {
            id: auto_val::decode_object(nv) as usize,
        })
    } else if auto_val::is_i32(nv) {
        auto_val::Value::Int(auto_val::decode_i32(nv))
    } else if auto_val::is_bool(nv) {
        auto_val::Value::Bool(auto_val::decode_bool(nv))
    } else if auto_val::is_null(nv) {
        auto_val::Value::Nil
    } else if auto_val::is_string(nv) {
        let idx = auto_val::decode_string(nv) as usize;
        let s = vm
            .strings
            .read()
            .unwrap()
            .get(idx)
            .map(|b| String::from_utf8_lossy(b).to_string())
            .unwrap_or_default();
        auto_val::Value::Str(auto_val::AutoStr::from(s.as_str()))
    } else if auto_val::is_f32(nv) {
        // Plan 340 audit: f32 (1-slot) → Float so it serializes as a number
        // instead of being dropped to Nil. (f64 is 2-slot [bits, null-padding];
        // it can't be handled here since we already pop_nv'd the single TOS.
        // f64 top-level from_value args are rare — struct-wrapped doubles go
        // through vm_value_to_json which handles Value::Double correctly.)
        auto_val::Value::Float(auto_val::decode_f32(nv) as f64)
    } else {
        auto_val::Value::Nil
    };
    // PLAN-995 T-02：直序化。语义错误（深度/非有限浮点，文本同
    // vm_value_to_json）沿旧 `?` 上抛；其余序列化失败沿旧
    // unwrap_or_default → 空串。
    let direct = VmJsonDirect {
        vm,
        value: &value,
        depth: 0,
    };
    let s = match serde_json::to_string(&direct) {
        Ok(text) => text,
        Err(e) => {
            let msg = e.to_string();
            if msg.starts_with("vm_value_to_json") || msg.starts_with("json.from_value") {
                return Err(VMError::RuntimeError(msg));
            }
            String::new()
        }
    };
    let bytes = s.into_bytes();
    let str_idx = vm.add_string(bytes);
    vm.rc_push_str_idx(task, str_idx);
    Ok(())
}

/// Prettify a JSON string
#[auto_macros::rust_fn("Json.prettify")]
pub fn shim_json_prettify(s: String) -> String {
    serde_json::from_str::<serde_json::Value>(&s)
        .ok()
        .and_then(|v| serde_json::to_string_pretty(&v).ok())
        .unwrap_or_default()
}

/// Check if a string is valid JSON
#[auto_macros::rust_fn("Json.is_valid")]
pub fn shim_json_is_valid(s: String) -> bool {
    serde_json::from_str::<serde_json::Value>(&s).is_ok()
}

/// Get a value from a JSON object by key
#[auto_macros::rust_fn("Json.get")]
pub fn shim_json_get(json_str: String, key: String) -> String {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    match val.get(&key) {
        Some(v) => v.to_string(),
        None => String::new(),
    }
}

/// Get a value from a JSON array by index
#[auto_macros::rust_fn("Json.get_at")]
pub fn shim_json_get_at(json_str: String, index: i32) -> String {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    match val.as_array() {
        Some(arr) => {
            if index < 0 || index as usize >= arr.len() {
                return String::new();
            }
            arr[index as usize].to_string()
        }
        None => String::new(),
    }
}

/// Get the length of a JSON array or object
#[auto_macros::rust_fn("Json.len")]
pub fn shim_json_len(json_str: String) -> i32 {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return -1,
    };
    match &val {
        serde_json::Value::Array(arr) => arr.len() as i32,
        serde_json::Value::Object(map) => map.len() as i32,
        _ => -1,
    }
}

/// Get the type of a JSON value as a string
#[auto_macros::rust_fn("Json.type_of")]
pub fn shim_json_type_of(json_str: String) -> String {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return "invalid".to_string(),
    };
    match val {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(_) => "bool".to_string(),
        serde_json::Value::Number(_) => "number".to_string(),
        serde_json::Value::String(_) => "string".to_string(),
        serde_json::Value::Array(_) => "array".to_string(),
        serde_json::Value::Object(_) => "object".to_string(),
    }
}

/// Get a JSON string value as a plain string
#[auto_macros::rust_fn("Json.as_string")]
pub fn shim_json_as_string(json_str: String) -> String {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    val.as_str().unwrap_or("").to_string()
}

/// Get a JSON number value as f64
#[auto_macros::rust_fn("Json.as_number")]
pub fn shim_json_as_number(json_str: String) -> f64 {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return 0.0,
    };
    val.as_f64().unwrap_or(0.0)
}

/// Get a JSON number value as i64
#[auto_macros::rust_fn("Json.as_int")]
pub fn shim_json_as_int(json_str: String) -> i64 {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return 0,
    };
    val.as_i64().unwrap_or(0)
}

/// Get a JSON boolean value
#[auto_macros::rust_fn("Json.as_bool")]
pub fn shim_json_as_bool(json_str: String) -> bool {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return false,
    };
    val.as_bool().unwrap_or(false)
}

/// Check if a JSON value is null
#[auto_macros::rust_fn("Json.is_null")]
pub fn shim_json_is_null(json_str: String) -> bool {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return false,
    };
    val.is_null()
}

/// Get the keys of a JSON object as a string list
#[auto_macros::rust_fn("Json.keys")]
pub fn shim_json_keys(json_str: String) -> Vec<String> {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    match val.as_object() {
        Some(map) => map.keys().cloned().collect(),
        None => Vec::new(),
    }
}

/// Check if a JSON object has a given key
#[auto_macros::rust_fn("Json.has_key")]
pub fn shim_json_has_key(json_str: String, key: String) -> bool {
    let val: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return false,
    };
    val.as_object().map_or(false, |map| map.contains_key(&key))
}

/// Check if a string matches a regex pattern.
///
/// Returns 1 if match found, 0 otherwise.
#[auto_macros::rust_fn("Regex.is_match")]
pub fn shim_regex_is_match(pattern: String, text: String) -> Result<i32, String> {
    let re = regex::Regex::new(&pattern).map_err(|e| {
        format!(
            "Regex.is_match failed: invalid pattern '{}': {}",
            pattern, e
        )
    })?;
    Ok(if re.is_match(&text) { 1 } else { 0 })
}

/// Find all matches of a regex pattern in text.
///
/// Returns a JSON array of match objects: `[{"match": "...", "start": 0, "end": 5}, ...]`
pub fn shim_regex_find_all(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let text: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let pattern: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let re = regex::Regex::new(&pattern).map_err(|e| {
        VMError::RuntimeError(format!(
            "Regex.find_all failed: invalid pattern '{}': {}",
            pattern, e
        ))
    })?;

    let mut matches: Vec<serde_json::Value> = Vec::new();
    for cap in re.find_iter(&text) {
        matches.push(serde_json::json!({
            "match": cap.as_str(),
            "start": cap.start(),
            "end": cap.end(),
        }));
        // Limit to 250 matches to avoid excessive memory
        if matches.len() >= 250 {
            break;
        }
    }

    let json = serde_json::to_string(&matches)
        .map_err(|e| VMError::RuntimeError(format!("JSON serialization failed: {}", e)))?;

    json.push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// PLAN-066 T-05（KD-057② 残余根修）：`Regex.match(text, pattern[, flags])`
/// web 形态——JS `str.match`：无 'g' 返回 `[全匹配, 组1, …]`（musk
/// colonMatch[1]/p0[1] 组提取），含 'g' 返回全部匹配子串列表（围栏提取）；
/// 无匹配返回空列表（JS null 的 VM 对齐，`length > 0` 守卫双轨同真值）。
/// 原实现为 is_match 1/0 语义且弹参错位，列表消费恒不可用——musk 侧曾以
/// indexOf 纯串绕开（11b6c20，随本修回撤）。两参调用由 codegen 编译期补
/// flags=""（沿 057 T6 JSON.stringify 补参先例）。
/// 入参 CALL_NAT 约定（自顶向下，末参在顶）：[flags, pattern, text]。
pub fn shim_regex_match(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let flags: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let pattern: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let text: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    push_regex_match_list(
        task,
        vm,
        &text,
        &pattern,
        flags.contains('g'),
        flags.contains('i'),
    )
}

/// PLAN-066 T-06（055-4⑥ 现代真身）：`str.includes(pat)` 字符串包含判定。
/// 实例方法 CALL_NAT 约定（自顶向下）：[pat, receiver]。此前合成 fn 内
/// `str.includes` 无 native 注册且不达引擎 CALL_SPEC str 臂 → 静默 no-op
/// 桩（恒 Nil → if 恒假），musk filteredMessages 的 includes 过滤投影恒
/// 0 条（聊天搜索/画布投影脸）；t3_filter 脚本层另一分发路故绿。
pub fn shim_str_includes(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let pat = String::pop_from_stack(task, vm).map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let receiver =
        String::pop_from_stack(task, vm).map_err(|e| VMError::RuntimeError(e.to_string()))?;
    VMConvertible::push_to_stack(&receiver.contains(&pat), task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// PLAN-053 P-053-6: `Regex.replace(text, pattern, replacement, flags)` —
/// web 生态静态形态（对齐 musk forge_helpers.ts 的 Regex 工具：flags 含 "g"
/// 全局替换，否则只替换首处）。入参按 CALL_NAT 约定（自顶向下）：
/// [flags, replacement, pattern, text]（CALL_SPEC 路径的接收者已由引擎
/// 分派层剥离）。
pub fn shim_regex_replace(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let flags: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let replacement: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let pattern: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let text: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let re = regex::Regex::new(&pattern).map_err(|e| {
        VMError::RuntimeError(format!(
            "Regex.replace failed: invalid pattern '{}': {}",
            pattern, e
        ))
    })?;

    let out: String = if flags.contains('g') {
        re.replace_all(&text, replacement.as_str()).into_owned()
    } else {
        re.replace(&text, replacement.as_str()).into_owned()
    };
    crate::vm::ffi::convert::VMConvertible::push_to_stack(&out, task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// PLAN-053 P-053-6: `Regex.test(text, pattern) -> bool` — web 生态静态形态
/// （stripQuestionnaire 的探测现场）。入参（自顶向下）：[pattern, text]。
pub fn shim_regex_test(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let pattern: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let text: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let re = regex::Regex::new(&pattern).map_err(|e| {
        VMError::RuntimeError(format!(
            "Regex.test failed: invalid pattern '{}': {}",
            pattern, e
        ))
    })?;

    task.ram.push_nv(auto_val::encode_bool(re.is_match(&text)));
    Ok(())
}

// ---- PLAN-884 T-04：自 stdlib 壳迁入（原样搬移） ----

// ============================================================================
// Plan 446 批三 D1(续): 文档类 json native 双态入参
// ============================================================================

/// 判定栈值是否为 JSON 文档堆对象（__json_object GenericInstanceData /
/// ListData<Value>——json.parse 的产物；ListData<i32>——json.keys 等
/// 文本工具链的返回形态，批四 D5 补齐）。命中则返回堆 id。
fn json_doc_heap_id(vm: &AutoVM, nv: auto_val::NanoValue) -> Option<u64> {
    let id = if auto_val::is_object(nv) {
        auto_val::decode_object(nv) as u64
    } else if auto_val::is_i32(nv) {
        let v = auto_val::decode_i32(nv);
        if v > 0 {
            v as u64
        } else {
            return None;
        }
    } else {
        return None;
    };
    let is_doc = vm
        .get_heap_object(id)
        .map(|o| {
            let guard = o.read().unwrap();
            let any = guard.as_any();
            any.downcast_ref::<crate::vm::generic_registry::GenericInstanceData>()
                .is_some()
                || any
                    .downcast_ref::<crate::vm::types::ListData<auto_val::Value>>()
                    .is_some()
                || any
                    .downcast_ref::<crate::vm::types::ListData<i32>>()
                    .is_some()
        })
        .unwrap_or(false);
    if is_doc {
        Some(id)
    } else {
        None
    }
}

/// ListData<i32> 的 JSON 序列化（D5）：全负元素 = 字符串池负索引编码
/// （`Vec<String>::push_to_stack` 的形态，json.keys 的返回）→ 解码为
/// key 字符串数组；含非负元素 = 真 int 数组。不进共享的 vm_value_to_json
/// （其 i32 臂直读数值，负索引会被当负数）。
fn json_serialize_i32_list(vm: &AutoVM, id: u64) -> Option<String> {
    use crate::vm::types::ListData;
    use serde_json::Value as J;
    let obj = vm.get_heap_object(id)?;
    let guard = obj.read().unwrap();
    let list = guard.as_any().downcast_ref::<ListData<i32>>()?;
    let elems: Vec<J> = if list.elems.iter().all(|i| *i < 0) {
        let strings = vm.strings.read().unwrap();
        list.elems
            .iter()
            .map(|i| {
                strings
                    .get((-*i - 1) as usize)
                    .map(|b| J::String(String::from_utf8_lossy(b).to_string()))
                    .unwrap_or(J::Null)
            })
            .collect()
    } else {
        list.elems.iter().map(|i| serde_json::json!(*i)).collect()
    };
    serde_json::to_string(&J::Array(elems)).ok()
}

/// 文档类 json native（get/get_at/has_key/len/keys/is_valid）的首参规整：
/// 若首参是 json.parse 产出的堆文档，序列化回 JSON 文本原位替换；文本/
/// 其它形态原样透传。下游 rust_fn shim 的字符串语义（片段切片/type_of）
/// 字节级不变——`json.get(json.parse(x), k)` 文本工具链与 `json.parse` +
/// 点访问两种idiom自此互通（批三"语义统一"目标）。
fn normalize_json_doc_head(
    task: &mut AutoTask,
    vm: &AutoVM,
    extra_args: usize,
) -> Result<(), VMError> {
    let mut extras: Vec<auto_val::NanoValue> = Vec::with_capacity(extra_args);
    for _ in 0..extra_args {
        extras.push(task.ram.pop_nv());
    }
    let doc_nv = task.ram.pop_nv();
    let serialized: Option<String> = json_doc_heap_id(vm, doc_nv).and_then(|id| {
        // ListData<i32>（负索引编码）走专用解码；其余经共享转换器。
        let direct = json_serialize_i32_list(vm, id);
        if direct.is_some() {
            return direct;
        }
        let jv = vm_value_to_json(
            vm,
            &auto_val::Value::VmRef(auto_val::VmRef { id: id as usize }),
            0,
        )
        .ok()?;
        serde_json::to_string(&jv).ok()
    });
    match serialized {
        Some(text) => {
            let idx = vm.add_string(text.into_bytes());
            vm.rc_push_str_idx(task, idx);
        }
        None => task.ram.push_nv(doc_nv),
    }
    for nv in extras {
        task.ram.push_nv(nv);
    }
    Ok(())
}

/// 原始（inventory rust_fn）shim 的暂存表：wrapper 规整首参后委托原实现。
lazy_static::lazy_static! {
    static ref JSON_DOC_ORIGINALS: std::sync::Mutex<std::collections::HashMap<u16, crate::vm::native::ShimFunc>> =
        std::sync::Mutex::new(std::collections::HashMap::new());
}

/// 覆盖文档类 json native 为双态版（engine 的 AutoVM::new 覆盖块调用，
/// 须在 build_from_inventory 之后）。命名面含 auto.json.* 与 Json.* 双轨
/// （inventory 注册名经 canonical 归一到同一固定 ID）。
pub fn override_json_doc_natives(native_interface: &mut crate::vm::native::NativeInterface) {
    use crate::vm::native_registry::BIGVM_NATIVES;
    const DOC_NATIVES: &[(&str, usize)] = &[
        ("auto.json.get", 1),
        ("auto.json.get_at", 1),
        ("auto.json.has_key", 1),
        ("auto.json.len", 0),
        ("auto.json.keys", 0),
        ("auto.json.is_valid", 0),
    ];
    // Preserve actual value tags, including String("true") versus Bool(true).
    // Legacy getter fragments are normalized at the codegen consumption site;
    // their contents must never be used to guess the type of an ordinary string.
    if let Some(id) = BIGVM_NATIVES
        .lock()
        .unwrap()
        .resolve_qualified("auto.json.encode")
    {
        if let Some(original) = native_interface.get(id) {
            JSON_DOC_ORIGINALS
                .lock()
                .unwrap()
                .insert(id, original.clone());
            let wrapper = move |task: &mut AutoTask, vm: &AutoVM| -> Result<(), VMError> {
                let slot =
                    task.ram.sp.checked_sub(1).ok_or_else(|| {
                        VMError::RuntimeError("json.encode: missing value".into())
                    })?;
                // Legacy native lists use a bare i32 handle plus an owned
                // stack-slot stake. An unowned i32 is a number, even when its
                // bits coincide with a live document/list id (PLAN-062 ABI).
                let owned_id = task.ram.take_stake_at(slot);
                let nv = crate::vm::native::pop_arg_nv(task);
                let consumed_stake = || {
                    if auto_val::is_i32(nv) {
                        crate::vm::native::StakeGuard::new(vm, owned_id)
                    } else {
                        crate::vm::native::StakeGuard::nv(vm, nv)
                    }
                };
                if auto_val::is_i32(nv) && owned_id == 0 {
                    let text = auto_val::decode_i32(nv).to_string();
                    let idx = vm.add_string(text.into_bytes());
                    vm.rc_push_str_idx(task, idx);
                    return Ok(());
                }
                if let Some(doc_id) = json_doc_heap_id(vm, nv) {
                    let _stake = consumed_stake();
                    let direct = json_serialize_i32_list(vm, doc_id);
                    let text = direct.or_else(|| {
                        let jv = vm_value_to_json(
                            vm,
                            &auto_val::Value::VmRef(auto_val::VmRef {
                                id: doc_id as usize,
                            }),
                            0,
                        )
                        .ok()?;
                        serde_json::to_string(&jv).ok()
                    });
                    if let Some(text) = text {
                        let idx = vm.add_string(text.into_bytes());
                        vm.rc_push_str_idx(task, idx);
                        return Ok(());
                    }
                    return Err(VMError::RuntimeError(
                        "json.encode: document serialization failed".into(),
                    ));
                }
                if let Some(text) = crate::vm::ffi::http_server::nv_to_json(vm, nv, 0) {
                    let _stake = consumed_stake();
                    let idx = vm.add_string(text.into_bytes());
                    vm.rc_push_str_idx(task, idx);
                    return Ok(());
                }
                task.ram.push_nv(nv);
                task.ram.mark_top_stake(owned_id);
                let original = JSON_DOC_ORIGINALS
                    .lock()
                    .unwrap()
                    .get(&id)
                    .cloned()
                    .ok_or_else(|| {
                        VMError::RuntimeError(
                            "json.encode doc native: original shim missing".into(),
                        )
                    })?;
                original(task, vm)
            };
            native_interface.register(id, wrapper);
        }
    }
    for (name, extra) in DOC_NATIVES {
        let id = match BIGVM_NATIVES.lock().unwrap().resolve_qualified(name) {
            Some(id) => id,
            None => continue,
        };
        let original = match native_interface.get(id) {
            Some(f) => f.clone(),
            None => continue,
        };
        JSON_DOC_ORIGINALS.lock().unwrap().insert(id, original);
        let wrapper = move |task: &mut AutoTask, vm: &AutoVM| -> Result<(), VMError> {
            normalize_json_doc_head(task, vm, *extra)?;
            let original = JSON_DOC_ORIGINALS
                .lock()
                .unwrap()
                .get(&id)
                .cloned()
                .ok_or_else(|| {
                    VMError::RuntimeError("json doc native: original shim missing".into())
                })?;
            original(task, vm)
        };
        native_interface.register(id, wrapper);
    }
}

/// Parse a TOML string (placeholder)
#[auto_macros::rust_fn("toml.from_str")]
pub fn shim_toml_from_str(s: String) -> String {
    s
}

// ============================================================================
// Plan 340: JSON ↔ VM value conversion natives (for VM+VM split mode HTTP API)
// ============================================================================

/// Recursively convert a `serde_json::Value` into a VM heap object and return
/// the encoded object id (pushed onto the stack as a NaN-boxed object ref).
///
/// - JSON object  → GenericInstanceData (field_names = JSON keys)
/// - JSON array   → ListData<Value> (elements recurse)
/// - scalars      → pushed directly (int/bool/nil/string)
///
/// Returns `Some(obj_id)` for compound values (the caller pushes the encoded
/// ref), or `None` for scalars (already pushed by this function).
pub fn json_to_vm_value(
    task: &mut AutoTask,
    vm: &AutoVM,
    value: &serde_json::Value,
    depth: u32,
) -> Result<(), VMError> {
    const MAX_DEPTH: u32 = 64;
    if depth >= MAX_DEPTH {
        return Err(VMError::RuntimeError(format!(
            "json_to_vm_value: nesting depth exceeded ({})",
            MAX_DEPTH
        )));
    }
    use auto_val::Value;
    match value {
        serde_json::Value::Null => {
            task.ram.push_nv(auto_val::encode_null());
        }
        serde_json::Value::Bool(b) => {
            task.ram.push_nv(auto_val::encode_bool(*b));
        }
        serde_json::Value::Number(n) => {
            // PLAN-734 T-03（E4 修复）：Auto int 语义 = i64 全域——
            // 此前 `as i32` 静默截断；48 位内联/堆装箱经共享 helper。
            if let Some(i) = n.as_i64() {
                task.ram
                    .push_nv(crate::vm::ffi::convert::encode_i64_with_heap(vm, i));
            } else if let Some(u) = n.as_u64() {
                task.ram
                    .push_nv(crate::vm::ffi::convert::encode_u64_with_heap(vm, u));
            } else if let Some(f) = n.as_f64() {
                task.ram.push_f64(f);
            } else {
                return Err(VMError::RuntimeError(
                    "json_to_vm_value: unrepresentable number".into(),
                ));
            }
        }
        serde_json::Value::String(s) => {
            // 2026-08-22(池去重):宿主桥/JSON 每次调用都重解析全量历史
            // (438+ 条),直推令池随每条命令膨胀;去重后重复内容零增长。
            let str_idx = vm.add_string(s.as_bytes().to_vec());
            vm.rc_push_str_idx(task, str_idx as usize);
        }
        serde_json::Value::Array(arr) => {
            use crate::vm::types::ListData;
            // Recurse each element, collecting as Value (VmRef for compound).
            let mut list: ListData<Value> = ListData::new();
            for elem in arr {
                let v = json_to_vm_value_inner(vm, elem, depth + 1)?;
                // Plan 419 §9(复活): 外层容器组装同样「插入即 retain」——
                // 与 inner 两臂对齐。此前此处漏 retain,顶层容器的直接子引用
                // 无持有即被 child_refs 声明,父死连坐释放时抵消他人真实
                // stake(ash-gui UAF:__json_object 子数组被提前释放)。
                if let Value::VmRef(r) = &v {
                    vm.rc_retain_id(r.id as u64);
                }
                list.push(v);
            }
            let id = vm.insert_heap_object(list);
            vm.rc_push(task, auto_val::encode_object(id as u32));
        }
        serde_json::Value::Object(obj) => {
            use crate::vm::generic_registry::GenericInstanceData;
            let mut field_names = Vec::with_capacity(obj.len());
            let mut fields = Vec::with_capacity(obj.len());
            for (k, v) in obj {
                field_names.push(k.clone());
                let fv = json_to_vm_value_inner(vm, v, depth + 1)?;
                // Plan 419 §9(复活): 同上,外层字段插入即 retain。
                if let Value::VmRef(r) = &fv {
                    vm.rc_retain_id(r.id as u64);
                }
                fields.push(fv);
            }
            let inst = GenericInstanceData::new_with_names(
                "__json_object".to_string(),
                fields,
                field_names,
            );
            let id = vm.insert_heap_object(inst);
            vm.rc_push(task, auto_val::encode_object(id as u32));
        }
    }
    Ok(())
}

/// Inner helper: convert a JSON value to a `Value` without touching the task
/// stack. Used when building compound values (array elements / object fields)
/// where the element must be a `Value::VmRef`, not pushed to the stack.
fn json_to_vm_value_inner(
    vm: &AutoVM,
    value: &serde_json::Value,
    depth: u32,
) -> Result<auto_val::Value, VMError> {
    const MAX_DEPTH: u32 = 64;
    if depth >= MAX_DEPTH {
        return Err(VMError::RuntimeError(format!(
            "json_to_vm_value_inner: nesting depth exceeded ({})",
            MAX_DEPTH
        )));
    }
    use auto_val::Value;
    match value {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                // Keep the existing compact representation when exact;
                // object/array fields must not truncate the i64 value.
                Ok(match i32::try_from(i) {
                    Ok(small) => Value::Int(small),
                    Err(_) => Value::I64(i),
                })
            } else if let Some(u) = n.as_u64() {
                let id = vm.insert_heap_object(crate::vm::heap_object::BigIntData::from_u64(u));
                Ok(Value::VmRef(auto_val::VmRef { id: id as usize }))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Double(f))
            } else {
                Ok(Value::Int(0))
            }
        }
        serde_json::Value::String(s) => Ok(Value::Str(auto_val::AutoStr::from(s.as_str()))),
        serde_json::Value::Array(arr) => {
            use crate::vm::types::ListData;
            let mut list: ListData<Value> = ListData::new();
            for elem in arr {
                let v = json_to_vm_value_inner(vm, elem, depth + 1)?;
                // Plan 419: 子 VmRef 的持有随父容器建立(插入即 retain)。
                if let Value::VmRef(r) = &v {
                    vm.rc_retain_id(r.id as u64);
                }
                list.push(v);
            }
            let id = vm.insert_heap_object(list);
            Ok(Value::VmRef(auto_val::VmRef { id: id as usize }))
        }
        serde_json::Value::Object(obj) => {
            use crate::vm::generic_registry::GenericInstanceData;
            let mut field_names = Vec::with_capacity(obj.len());
            let mut fields = Vec::with_capacity(obj.len());
            for (k, v) in obj {
                field_names.push(k.clone());
                let fv = json_to_vm_value_inner(vm, v, depth + 1)?;
                // Plan 419: 同上。
                if let Value::VmRef(r) = &fv {
                    vm.rc_retain_id(r.id as u64);
                }
                fields.push(fv);
            }
            let inst = GenericInstanceData::new_with_names(
                "__json_object".to_string(),
                fields,
                field_names,
            );
            let id = vm.insert_heap_object(inst);
            Ok(Value::VmRef(auto_val::VmRef { id: id as usize }))
        }
    }
}

// ============================================================================
// PLAN-995 T-02：json 单趟物化 / 直序化（消除中间 serde_json::Value 树）
// ============================================================================

const JSON_DIRECT_MAX_DEPTH: u32 = 64;

/// 单趟 JSON→VM inner `Value` 物化 seed（serde 驱动，无中间树）。
/// 语义与 `json_to_vm_value_inner` 逐臂对齐：i64→i32 装箱/I64、超 i64 的
/// u64→BigIntData 堆 VmRef、f64→Double、str→AutoStr、子 VmRef 插入即
/// retain、64 层深度上限（错误文本保持 `json_to_vm_value_inner:` 前缀）。
struct DirectMatSeed<'a> {
    vm: &'a AutoVM,
    depth: u32,
}

impl<'de, 'a> serde::de::DeserializeSeed<'de> for DirectMatSeed<'a> {
    type Value = auto_val::Value;
    fn deserialize<D>(self, d: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        d.deserialize_any(DirectMatVisitor {
            vm: self.vm,
            depth: self.depth,
        })
    }
}

/// inner 物化 visitor（容器元素/字段位）。
struct DirectMatVisitor<'a> {
    vm: &'a AutoVM,
    depth: u32,
}

impl<'de, 'a> serde::de::Visitor<'de> for DirectMatVisitor<'a> {
    type Value = auto_val::Value;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("any JSON value")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(auto_val::Value::Nil)
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
        Ok(auto_val::Value::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
        Ok(match i32::try_from(v) {
            Ok(small) => auto_val::Value::Int(small),
            Err(_) => auto_val::Value::I64(v),
        })
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
        // serde_json 直驱对无符号语法形态的非负整数调 visit_u64（与经
        // serde_json::Value 中转时 `as_i64()` 先行的归一不同）——必须先
        // 归一到 i64 语义（i32 装箱/I64），仅真超 i64::MAX 才 BigIntData，
        // 否则每个正整数字段都物化成 BigInt 堆引用（.to_string() 断链）。
        if let Ok(i) = i64::try_from(v) {
            return Ok(match i32::try_from(i) {
                Ok(small) => auto_val::Value::Int(small),
                Err(_) => auto_val::Value::I64(i),
            });
        }
        let id = self
            .vm
            .insert_heap_object(crate::vm::heap_object::BigIntData::from_u64(v));
        Ok(auto_val::Value::VmRef(auto_val::VmRef { id: id as usize }))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
        Ok(auto_val::Value::Double(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
        Ok(auto_val::Value::Str(auto_val::AutoStr::from(v)))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        if self.depth >= JSON_DIRECT_MAX_DEPTH {
            return Err(serde::de::Error::custom(format!(
                "json_to_vm_value_inner: nesting depth exceeded ({})",
                JSON_DIRECT_MAX_DEPTH
            )));
        }
        use crate::vm::types::ListData;
        let mut list: ListData<auto_val::Value> = ListData::new();
        while let Some(elem) = seq.next_element_seed(DirectMatSeed {
            vm: self.vm,
            depth: self.depth + 1,
        })? {
            if let auto_val::Value::VmRef(r) = &elem {
                self.vm.rc_retain_id(r.id as u64);
            }
            list.push(elem);
        }
        let id = self.vm.insert_heap_object(list);
        Ok(auto_val::Value::VmRef(auto_val::VmRef { id: id as usize }))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if self.depth >= JSON_DIRECT_MAX_DEPTH {
            return Err(serde::de::Error::custom(format!(
                "json_to_vm_value_inner: nesting depth exceeded ({})",
                JSON_DIRECT_MAX_DEPTH
            )));
        }
        use crate::vm::generic_registry::GenericInstanceData;
        let mut field_names: Vec<String> = Vec::new();
        let mut fields: Vec<auto_val::Value> = Vec::new();
        while let Some(name) = map.next_key::<String>()? {
            let fv = map.next_value_seed(DirectMatSeed {
                vm: self.vm,
                depth: self.depth + 1,
            })?;
            if let auto_val::Value::VmRef(r) = &fv {
                self.vm.rc_retain_id(r.id as u64);
            }
            field_names.push(name);
            fields.push(fv);
        }
        let inst =
            GenericInstanceData::new_with_names("__json_object".to_string(), fields, field_names);
        let id = self.vm.insert_heap_object(inst);
        Ok(auto_val::Value::VmRef(auto_val::VmRef { id: id as usize }))
    }
}

/// 顶层物化 visitor（栈推送语义与 `json_to_vm_value` 对齐：标量直推、
/// 容器 rc_push）。
struct DirectTopMatVisitor<'a> {
    vm: &'a AutoVM,
    task: &'a mut AutoTask,
}

impl<'de, 'a> serde::de::Visitor<'de> for DirectTopMatVisitor<'a> {
    type Value = ();

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("any JSON value")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        self.task.ram.push_nv(auto_val::encode_null());
        Ok(())
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
        self.task.ram.push_nv(auto_val::encode_bool(v));
        Ok(())
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
        let nv = crate::vm::ffi::convert::encode_i64_with_heap(self.vm, v);
        self.task.ram.push_nv(nv);
        Ok(())
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
        // 同 inner 臂注记：无符号语法形态先归一 i64 编码，仅超 i64::MAX
        // 才 u64 宽编码（与旧 `as_i64()` 先行的顶层语义一致）。
        let nv = if let Ok(i) = i64::try_from(v) {
            crate::vm::ffi::convert::encode_i64_with_heap(self.vm, i)
        } else {
            crate::vm::ffi::convert::encode_u64_with_heap(self.vm, v)
        };
        self.task.ram.push_nv(nv);
        Ok(())
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
        self.task.ram.push_f64(v);
        Ok(())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
        let str_idx = self.vm.add_string(v.as_bytes().to_vec());
        self.vm.rc_push_str_idx(self.task, str_idx as usize);
        Ok(())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        use crate::vm::types::ListData;
        let mut list: ListData<auto_val::Value> = ListData::new();
        while let Some(elem) = seq.next_element_seed(DirectMatSeed {
            vm: self.vm,
            depth: 1,
        })? {
            if let auto_val::Value::VmRef(r) = &elem {
                self.vm.rc_retain_id(r.id as u64);
            }
            list.push(elem);
        }
        let id = self.vm.insert_heap_object(list);
        self.vm
            .rc_push(self.task, auto_val::encode_object(id as u32));
        Ok(())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        use crate::vm::generic_registry::GenericInstanceData;
        let mut field_names: Vec<String> = Vec::new();
        let mut fields: Vec<auto_val::Value> = Vec::new();
        while let Some(name) = map.next_key::<String>()? {
            let fv = map.next_value_seed(DirectMatSeed {
                vm: self.vm,
                depth: 1,
            })?;
            if let auto_val::Value::VmRef(r) = &fv {
                self.vm.rc_retain_id(r.id as u64);
            }
            field_names.push(name);
            fields.push(fv);
        }
        let inst =
            GenericInstanceData::new_with_names("__json_object".to_string(), fields, field_names);
        let id = self.vm.insert_heap_object(inst);
        self.vm
            .rc_push(self.task, auto_val::encode_object(id as u32));
        Ok(())
    }
}

/// 单趟物化解析入口（PLAN-995 T-02）。
/// pass 1：`IgnoredAny` 全量语法校验（零分配；尾随垃圾同 `from_str`
/// 拒绝）——保证 pass 2 不因语法错误中途中止而留下部分物化（旧两段式
/// 解析期零推送，错误路径栈干净，此性质必须保持）。
/// pass 2：直驱物化；仅深度类错误可中途失败， unwind 部分推送
/// （含影子份额释放），错误文本保持 `json_to_vm_value*:` 前缀。
pub(crate) fn parse_json_direct(
    task: &mut AutoTask,
    vm: &AutoVM,
    s: &str,
) -> Result<(), serde_json::Error> {
    use serde::Deserializer as _;
    serde_json::from_str::<serde::de::IgnoredAny>(s)?;
    let sp_before = task.ram.sp;
    let mut de = serde_json::Deserializer::from_str(s);
    let r = de
        .deserialize_any(DirectTopMatVisitor { vm, task })
        .and_then(|()| de.end());
    if let Err(e) = r {
        let sp_now = task.ram.sp;
        if sp_now > sp_before {
            vm.rc_release_slot_range(&mut task.ram, sp_before, sp_now);
            task.ram.sp = sp_before;
        }
        return Err(e);
    }
    Ok(())
}

/// 语义错误（深度/NaN）透传、语法错误加前缀——供 shim 错误映射与
/// 旧路径文本保持一致。
pub(crate) fn map_parse_error(e: serde_json::Error) -> VMError {
    let msg = e.to_string();
    if msg.starts_with("json_to_vm_value") {
        VMError::RuntimeError(msg)
    } else {
        VMError::RuntimeError(format!("json.parse: invalid JSON: {}", msg))
    }
}

/// PLAN-995 T-02：VM 值直序化（serde Serialize 直写，无中间 Value 树）。
/// 序列化形状与 `vm_value_to_json` + `serde_json::to_string` 逐臂对齐
/// （含 ObjectData 字典序键、GenericInstanceData 插入序、BigIntData
/// i64/u64、非有限浮点错误文本）。
pub(crate) struct VmJsonDirect<'a> {
    pub vm: &'a AutoVM,
    pub value: &'a auto_val::Value,
    pub depth: u32,
}

impl serde::Serialize for VmJsonDirect<'_> {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use auto_val::Value;
        use serde::ser::{SerializeMap as _, SerializeSeq as _};
        if self.depth >= JSON_DIRECT_MAX_DEPTH {
            return Err(serde::ser::Error::custom(format!(
                "vm_value_to_json: nesting depth exceeded ({})",
                JSON_DIRECT_MAX_DEPTH
            )));
        }
        match self.value {
            Value::Nil => s.serialize_unit(),
            Value::Bool(b) => s.serialize_bool(*b),
            Value::Int(i) => s.serialize_i64(*i as i64),
            Value::Uint(u) => s.serialize_u64(*u as u64),
            Value::I64(i) => s.serialize_i64(*i),
            Value::Float(f) => {
                let d = *f as f64;
                if !d.is_finite() {
                    return Err(serde::ser::Error::custom("json.from_value: NaN/Inf float"));
                }
                s.serialize_f64(d)
            }
            Value::Double(d) => {
                if !d.is_finite() {
                    return Err(serde::ser::Error::custom("json.from_value: NaN/Inf double"));
                }
                s.serialize_f64(*d)
            }
            Value::Str(st) => s.serialize_str(st),
            Value::Char(c) => s.serialize_char(*c),
            Value::VmRef(r) => {
                let id = r.id as u64;
                let Some(heap_ref) = self.vm.heap_objects.get(&id) else {
                    return s.serialize_unit();
                };
                let obj = heap_ref.read().unwrap();
                use crate::vm::generic_registry::GenericInstanceData;
                use crate::vm::types::ListData;
                if let Some(big) = obj
                    .as_any()
                    .downcast_ref::<crate::vm::heap_object::BigIntData>()
                {
                    if big.is_unsigned {
                        s.serialize_u64(big.as_u64())
                    } else {
                        s.serialize_i64(big.as_i64())
                    }
                } else if let Some(inst) = obj.as_any().downcast_ref::<GenericInstanceData>() {
                    let mut map = s.serialize_map(Some(inst.field_names.len()))?;
                    for (i, name) in inst.field_names.iter().enumerate() {
                        if let Some(fv) = inst.fields.get(i) {
                            map.serialize_entry(
                                name,
                                &VmJsonDirect {
                                    vm: self.vm,
                                    value: fv,
                                    depth: self.depth + 1,
                                },
                            )?;
                        }
                    }
                    map.end()
                } else if let Some(odata) =
                    obj.as_any().downcast_ref::<crate::vm::types::ObjectData>()
                {
                    // PLAN-057 T6 同款：ObjectData 键字典序（确定性；
                    // 与 JS 插入序的观感差为已知登记项）。键形与
                    // vm_value_to_json 一致（Str/Int/Bool 键 stringify）。
                    let mut pairs: Vec<(String, &auto_val::Value)> = odata
                        .fields
                        .iter()
                        .map(|(k, v)| {
                            let key = match k {
                                auto_val::ValueKey::Str(st) => st.to_string(),
                                auto_val::ValueKey::Int(i) => i.to_string(),
                                auto_val::ValueKey::Bool(b) => b.to_string(),
                            };
                            (key, v)
                        })
                        .collect();
                    pairs.sort_by(|a, b| a.0.cmp(&b.0));
                    let mut map = s.serialize_map(Some(pairs.len()))?;
                    for (k, v) in pairs {
                        map.serialize_entry(
                            &k,
                            &VmJsonDirect {
                                vm: self.vm,
                                value: v,
                                depth: self.depth + 1,
                            },
                        )?;
                    }
                    map.end()
                } else if let Some(list) = obj.as_any().downcast_ref::<ListData<auto_val::Value>>()
                {
                    let mut seq = s.serialize_seq(Some(list.elems.len()))?;
                    for elem in &list.elems {
                        seq.serialize_element(&VmJsonDirect {
                            vm: self.vm,
                            value: elem,
                            depth: self.depth + 1,
                        })?;
                    }
                    seq.end()
                } else if let Some(list) = obj.as_any().downcast_ref::<ListData<i32>>() {
                    let mut seq = s.serialize_seq(Some(list.elems.len()))?;
                    for e in &list.elems {
                        seq.serialize_element(&(*e as i64))?;
                    }
                    seq.end()
                } else if let Some(list) = obj.as_any().downcast_ref::<ListData<String>>() {
                    let mut seq = s.serialize_seq(Some(list.elems.len()))?;
                    for e in &list.elems {
                        seq.serialize_element(e)?;
                    }
                    seq.end()
                } else if let Some(list) = obj.as_any().downcast_ref::<ListData<char>>() {
                    let mut seq = s.serialize_seq(Some(list.elems.len()))?;
                    for c in &list.elems {
                        seq.serialize_element(&c.to_string())?;
                    }
                    seq.end()
                } else if let Some(list) = obj.as_any().downcast_ref::<ListData<bool>>() {
                    let mut seq = s.serialize_seq(Some(list.elems.len()))?;
                    for b in &list.elems {
                        seq.serialize_element(b)?;
                    }
                    seq.end()
                } else {
                    s.serialize_unit()
                }
            }
            _ => s.serialize_unit(),
        }
    }
}

/// Recursively serialize a VM heap object into a `serde_json::Value`.
pub(crate) fn vm_value_to_json(
    vm: &AutoVM,
    value: &auto_val::Value,
    depth: u32,
) -> Result<serde_json::Value, VMError> {
    const MAX_DEPTH: u32 = 64;
    if depth >= MAX_DEPTH {
        return Err(VMError::RuntimeError(format!(
            "vm_value_to_json: nesting depth exceeded ({})",
            MAX_DEPTH
        )));
    }
    use auto_val::Value;
    match value {
        Value::Nil => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Int(i) => Ok(serde_json::json!(*i)),
        Value::Uint(u) => Ok(serde_json::json!(*u)),
        // PLAN-734 R2（T-08 提交时意外丢失的臂——nv_to_vm_value 自 T-07 起
        // 把 i32/i64 统一产 Value::I64；缺此臂时 42 走兜底路径 → stringify
        // 形态漂移 → 664 值域 e2e 红（Json.type_of 比较两个不同字符串）。
        Value::I64(i) => Ok(serde_json::json!(*i)),
        Value::Float(f) => serde_json::Number::from_f64(*f as f64)
            .map(serde_json::Value::Number)
            .ok_or_else(|| VMError::RuntimeError("json.from_value: NaN/Inf float".into())),
        Value::Double(d) => serde_json::Number::from_f64(*d)
            .map(serde_json::Value::Number)
            .ok_or_else(|| VMError::RuntimeError("json.from_value: NaN/Inf double".into())),
        Value::Str(s) => Ok(serde_json::Value::String(s.to_string())),
        Value::Char(c) => Ok(serde_json::Value::String(c.to_string())),
        Value::VmRef(vmref) => {
            let id = vmref.id as u64;
            if let Some(heap_ref) = vm.heap_objects.get(&id) {
                let obj = heap_ref.read().unwrap();
                use crate::vm::generic_registry::GenericInstanceData;
                use crate::vm::types::ListData;
                if let Some(big) = obj
                    .as_any()
                    .downcast_ref::<crate::vm::heap_object::BigIntData>()
                {
                    Ok(if big.is_unsigned {
                        serde_json::json!(big.as_u64())
                    } else {
                        serde_json::json!(big.as_i64())
                    })
                } else if let Some(inst) = obj.as_any().downcast_ref::<GenericInstanceData>() {
                    let mut map = serde_json::Map::new();
                    for (i, name) in inst.field_names.iter().enumerate() {
                        if let Some(fv) = inst.fields.get(i) {
                            map.insert(name.clone(), vm_value_to_json(vm, fv, depth + 1)?);
                        }
                    }
                    Ok(serde_json::Value::Object(map))
                } else if let Some(odata) =
                    obj.as_any().downcast_ref::<crate::vm::types::ObjectData>()
                {
                    // PLAN-057 T6：对象字面量（ObjectData）——此前落末尾 else
                    // 静默序列化成 null（JSON.stringify({a:1}) 现场根因）。
                    // 键序：ObjectData 是 HashMap 无序，按字典序出键保确定性
                    // （JS 为插入序——登记为已知观感差，非语义差）。
                    let mut pairs: Vec<(String, &auto_val::Value)> = odata
                        .fields
                        .iter()
                        .map(|(k, v)| {
                            let key = match k {
                                auto_val::ValueKey::Str(s) => s.to_string(),
                                auto_val::ValueKey::Int(i) => i.to_string(),
                                auto_val::ValueKey::Bool(b) => b.to_string(),
                            };
                            (key, v)
                        })
                        .collect();
                    pairs.sort_by(|a, b| a.0.cmp(&b.0));
                    let mut map = serde_json::Map::new();
                    for (key, v) in pairs {
                        map.insert(key, vm_value_to_json(vm, v, depth + 1)?);
                    }
                    Ok(serde_json::Value::Object(map))
                } else if let Some(list) = obj.as_any().downcast_ref::<ListData<auto_val::Value>>()
                {
                    let mut arr = Vec::with_capacity(list.elems.len());
                    for elem in &list.elems {
                        arr.push(vm_value_to_json(vm, elem, depth + 1)?);
                    }
                    Ok(serde_json::Value::Array(arr))
                } else if let Some(list) = obj.as_any().downcast_ref::<ListData<i32>>() {
                    let arr = list.elems.iter().map(|i| serde_json::json!(*i)).collect();
                    Ok(serde_json::Value::Array(arr))
                } else if let Some(list) = obj.as_any().downcast_ref::<ListData<String>>() {
                    let arr = list.elems.iter().map(|s| serde_json::json!(s)).collect();
                    Ok(serde_json::Value::Array(arr))
                } else if let Some(list) = obj.as_any().downcast_ref::<ListData<char>>() {
                    // Plan 340 audit: char list → JSON array of 1-char strings.
                    let arr = list
                        .elems
                        .iter()
                        .map(|c| serde_json::json!(c.to_string()))
                        .collect();
                    Ok(serde_json::Value::Array(arr))
                } else if let Some(list) = obj.as_any().downcast_ref::<ListData<bool>>() {
                    // Plan 340 audit: bool list → JSON array of booleans.
                    let arr = list.elems.iter().map(|b| serde_json::json!(*b)).collect();
                    Ok(serde_json::Value::Array(arr))
                } else {
                    Ok(serde_json::Value::Null)
                }
            } else {
                Ok(serde_json::Value::Null)
            }
        }
        _ => Ok(serde_json::Value::Null),
    }
}

// ============================================================================
// Regex Functions (Plan 159: 2400-2499)
// ============================================================================

/// 匹配结果列表推送（堆 List<Value::Str>；fresh 列表 rc_push(+1) 与
/// Plan 419 配平——元素为 Str 非堆引用，无子份额需求）。无匹配 → 空列表
/// （length 0）。global 250 条上限沿 find_all 口径。
fn push_regex_match_list(
    task: &mut AutoTask,
    vm: &AutoVM,
    text: &str,
    pattern: &str,
    global: bool,
    case_insensitive: bool,
) -> Result<(), VMError> {
    let mut builder = regex::RegexBuilder::new(pattern);
    builder.case_insensitive(case_insensitive);
    let re = builder.build().map_err(|e| {
        VMError::RuntimeError(format!(
            "Regex.match failed: invalid pattern '{}': {}",
            pattern, e
        ))
    })?;
    use crate::vm::types::ListData;
    let mut list: ListData<auto_val::Value> = ListData::new();
    let mut count = 0usize;
    if global {
        for m in re.find_iter(text) {
            list.push(auto_val::Value::Str(auto_val::AutoStr::from(m.as_str())));
            count += 1;
            if count >= 250 {
                break;
            }
        }
    } else if let Some(caps) = re.captures(text) {
        for g in caps.iter().flatten() {
            list.push(auto_val::Value::Str(auto_val::AutoStr::from(g.as_str())));
        }
    }
    let id = vm.insert_heap_object(list);
    vm.rc_push(task, auto_val::encode_object(id as u32));
    Ok(())
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_string_find_shims(natives: &mut crate::vm::native::NativeInterface) {
    // String method — manual shim
    natives.register_shim_by_name("auto.str.find", shim_str_find_manual);
    // str-parity: .slice/.sub are END-semantics (a2r s[a..b]) — distinct
    // from .substr's (start, LEN). All three previously aliased one shim;
    // shim_str_slice registers via its #[rust_fn("Str.slice\)] attribute +
    // the catalog's auto.str.slice/sub entries (id 1524).
    natives.register_shim_by_name("Str.find", shim_str_find_manual);
    natives.register_shim_by_name("str.find", shim_str_find_manual);
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_json_value_shims(natives: &mut crate::vm::native::NativeInterface) {
    // Plan 340: JSON ↔ VM value conversion (for VM+VM split mode HTTP API).
    // These replace the placeholder Json.parse (returns string as-is) with real
    // (de)serialization that produces/consumes VM heap objects.
    natives.register_shim_by_name("auto.json.to_value", shim_json_to_value);
    natives.register_shim_by_name("auto.json.from_value", shim_json_from_value);
    natives.register_shim_by_name("Json.to_value", shim_json_to_value);
    natives.register_shim_by_name("Json.from_value", shim_json_from_value);

    // Plan 340: Simple JSON HTTP helpers (return body string directly) for
    // VM+VM split mode codegen rewriting.
    natives.register_shim_by_name("auto.http.get_json", shim_http_get_json);
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_regex_i18n_shims(natives: &mut crate::vm::native::NativeInterface) {
    // Regex (manual shim — heap objects for compiled regex)
    natives.register_shim_by_name("auto.regex.find_all", shim_regex_find_all);
    natives.register_shim_by_name("auto.regex.match", shim_regex_match);
    // PLAN-053 P-053-6: web 生态静态形态（CALL_SPEC Regex.replace/test 路由）。
    natives.register_shim_by_name("auto.regex.replace", shim_regex_replace);
    natives.register_shim_by_name("auto.regex.test", shim_regex_test);
    // PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
    natives.register_shim_by_name("auto.i18n.t", shim_i18n_t);
    // PLAN-066 T-06：str.includes 字符串包含（实例方法 CALL_NAT：[pat,
    // receiver]；register_shim_by_name 动态 id，沿 http_stream 先例）。
    natives.register_shim_by_name("auto.str.includes", shim_str_includes);
}

#[cfg(test)]
mod p949_tests {
    fn output(source: &str) -> String {
        crate::execution_engine::execute_with_engine_capture(
            crate::execution_engine::ExecutionEngine::AutoVM,
            source,
        )
        .expect("real VM compile/execute")
        .1
    }

    #[test]
    fn p949_encode_preserves_primitive_types_and_string_contents() {
        let cases = [
            ("true", serde_json::json!(true)),
            ("false", serde_json::json!(false)),
            ("null", serde_json::Value::Null),
            ("\"true\"", serde_json::json!("true")),
            ("\"false\"", serde_json::json!("false")),
            ("\"null\"", serde_json::json!("null")),
            ("\"123\"", serde_json::json!("123")),
            ("0", serde_json::json!(0)),
            ("-3", serde_json::json!(-3)),
            ("54.25", serde_json::json!(54.25)),
            ("5000000001", serde_json::json!(5000000001_i64)),
        ];
        for (source, expected) in cases {
            let out = output(&format!("print(Json.encode({source}))"));
            let actual: serde_json::Value = serde_json::from_str(out.trim())
                .unwrap_or_else(|e| panic!("source={source}, output={out:?}: {e}"));
            assert_eq!(actual, expected, "source={source}, output={out:?}");
        }
    }

    #[test]
    fn p949_encode_nested_legacy_json_getters_as_values() {
        let values = serde_json::json!([
            true, false, null, "true", "false", "null", "123", "", 0, -3,
            54.25, 5000000001_i64, i64::MIN, i64::MAX, 18446744073709551615_u64,
            ["true", true, 8.125],
            {"nested": [false, "null", 0, 5000000001_i64, 18446744073709551615_u64]}
        ]);
        for (index, expected) in values.as_array().unwrap().iter().enumerate() {
            let object = serde_json::json!({"value": expected}).to_string();
            let object_literal = serde_json::to_string(&object).unwrap();
            let array_literal = serde_json::to_string(&values.to_string()).unwrap();
            for getter in [
                format!("Json.get(Json.parse({object_literal}), \"value\")"),
                format!("json.get_at(json.parse({array_literal}), {index})"),
            ] {
                for argument in [getter.clone(), format!("value: {getter}")] {
                    let out = output(&format!("print(Json.encode({argument}))"));
                    let actual: serde_json::Value = serde_json::from_str(out.trim())
                        .unwrap_or_else(|e| panic!("getter={argument}, output={out:?}: {e}"));
                    assert_eq!(&actual, expected, "getter={argument}, output={out:?}");
                }
            }
        }
    }

    #[test]
    fn p949_legacy_text_getters_and_plain_function_results_stay_distinct() {
        let out = output(
            r#"
fn plain_text() str { return "true" }
let plain = "true"
let doc = Json.parse("{\"name\":\"auto\",\"n\":1,\"ok\":true}")
print(Json.get(doc, "name"))
print(Json.type_of(Json.get(doc, "n")))
print(Json.encode(plain))
print(Json.encode(Json.get(doc, "ok")))
print(Json.encode(plain_text()))
print(Json.encode(plain))
"#,
        );
        assert_eq!(
            out.trim(),
            "\"auto\"\nnumber\n\"true\"\ntrue\n\"true\"\n\"true\""
        );
    }

    #[test]
    fn p949_getter_normalization_evaluates_input_once() {
        let out = output(
            r#"
fn source_json() str {
    print("once")
    return "{\"ok\":true}"
}
print(Json.encode(Json.get(Json.parse(source_json()), "ok")))
"#,
        );
        assert_eq!(out.trim(), "once\ntrue");
    }

    #[test]
    fn p949_shadowed_json_receiver_is_an_ordinary_user_method() {
        let out = output(
            r#"
type FakeJson { value str }
ext FakeJson {
    fn get(self, key str) str {
        print("user-get")
        return self.value
    }
}
let json = FakeJson { value: "true" }
print(Json.encode(json.get("key")))
fn local_case() {
    let json = FakeJson { value: "true" }
    print(Json.encode(json.get("key")))
}
local_case()
"#,
        );
        assert_eq!(out.trim(), "user-get\n\"true\"\nuser-get\n\"true\"");
    }

    #[test]
    fn p949_number_identity_survives_live_documents_and_legacy_string_lists() {
        let out = output(
            r#"
fn number() int { return 4000001 }
let ordinary = 4000000
print(Json.encode(ordinary))
let doc = Json.parse("{\"ok\":true}")
let array = Json.parse("[1,2]")
print(Json.encode(4000000))
print(Json.encode(ordinary))
print(Json.encode(number()))
print(Json.encode(4000001))
print(Json.encode(doc))
print(Json.encode(array))
let keys = Json.keys(doc)
print(Json.encode(keys))
print(Json.encode(4000002))
print(Json.encode(keys))
print(Json.encode(doc))
let holder = { value: 4000000 }
print(Json.encode(holder.value))
let numbers = [4000001]
print(Json.encode(numbers[0]))
print(Json.encode("4000000"))
print(Json.encode("a,b".split(",")))
"#,
        );
        let actual: Vec<serde_json::Value> = out
            .lines()
            .map(|line| serde_json::from_str(line).expect("encoded JSON line"))
            .collect();
        assert_eq!(
            actual,
            serde_json::json!([
                4000000, 4000000, 4000000, 4000001, 4000001,
                {"ok": true}, [1,2], ["ok"], 4000002, ["ok"], {"ok": true},
                4000000, 4000001, "4000000", ["a", "b"]
            ])
            .as_array()
            .unwrap()
            .to_vec()
        );
    }

    #[test]
    fn p949_named_encode_preserves_single_evaluation_and_user_strings() {
        let out = output(
            r#"
fn source() str {
    print("once")
    return "{\"ok\":true}"
}
fn text() str { return "true" }
print(Json.encode(value: Json.get(Json.parse(source()), "ok")))
print(Json.encode(value: "true"))
print(Json.encode(value: true))
print(Json.encode(value: text()))
type FakeJson { value str }
ext FakeJson {
    fn get(self, key str) str {
        print("user-get")
        return self.value
    }
}
let json = FakeJson { value: "true" }
print(Json.encode(value: json.get("key")))
"#,
        );
        assert_eq!(
            out.trim(),
            "once\ntrue\n\"true\"\ntrue\n\"true\"\nuser-get\n\"true\""
        );
    }
}
