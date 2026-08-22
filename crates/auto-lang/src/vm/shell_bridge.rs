// Plan 060(api.at 契约归一):merged 模式 shell 执行请求队列 —— 传输件。
//
// 链路:shell.at 的 run_command 经 native `auto.shell.exec_submit` 把请求压到
// 这里;iced renderer 的 merged_exec_loop 从这里取走执行(spawn + 流式,
// 零 shell 语义)。取代旧的 __pending_command renderer 桥 —— 那条链绕过
// api.at 契约、语义长在 renderer,已随 Plan 060 退役。
//
// 队列放 vm 模块(不 feature-gate):native shim(vm/native.rs,总是编译)与
// renderer(ui-iced)都能引用,依赖方向合法(ui → vm)。HTTP 模式不触达:
// AUTO_BACKEND 非空时 codegen 把 api 调用编译为 fetch,本队列无人生产。

/// 请求类型:进程执行(外部命令)或直发结果(.at 侧已算好语义的 builtin,
/// 如 ls/cd/pwd —— Plan 060 M2:语义在 .at 提交侧完成,传输层零语义)。
#[derive(Debug, Clone)]
pub enum ShellExecKind {
    Process,
    Result,
}

/// 一条待执行命令(merged 传输的请求单元)。
#[derive(Debug, Clone)]
pub struct ShellExecRequest {
    pub kind: ShellExecKind,
    pub block_id: i64,
    pub cmd: String,
    pub cwd: String,
    /// Result 变体:完整的 command_result payload JSON(执行线程直发)。
    pub result_json: String,
}

lazy_static::lazy_static! {
    static ref QUEUE: std::sync::Mutex<std::collections::VecDeque<ShellExecRequest>> =
        std::sync::Mutex::new(std::collections::VecDeque::new());
}

/// 压入一条执行请求(shell_exec_submit native 调用,来自 shell.at)。
pub fn submit(req: ShellExecRequest) {
    QUEUE.lock().unwrap().push_back(req);
}

/// 取出一条执行请求(merged_exec_loop 轮询);队列空返回 None。
pub fn pop() -> Option<ShellExecRequest> {
    QUEUE.lock().unwrap().pop_front()
}

// ── show 下沉(2026-08-22)────────────────────────────────────────────
// 根因:shell.at 在 VM 里用 `js = js + ...` 拼 show 的 Code payload
// (~100KB 巨串,types.at ~30KB、block_item.at ~100KB),大文件时触发 VM
// 字符串堆损坏 —— 实测会话 cwd 被写成单字符 "r"、payload 丢失、块永挂
// Running、进程满转后静默退出。语义(读文件+逐行高亮+payload JSON)整体
// 下沉 Rust,经 auto.shell.emit_show native 触发;renderer 的 font-mono
// Text 自动高亮也复用同一实现,避免两份色板漂移。

/// 单行语法高亮 → (文本, 颜色 rgb) 序列;None = 默认前景色。
/// 2026-08-22:自 ui/iced/renderer.rs 的 highlight_code 下沉为中立实现
/// (renderer 侧把结果转 iced Span,emit_show 侧转 payload span,单一来源)。
/// 分词/色板 = Plan 411 P2-A① 版(prism-tomorrow:keyword/builtin #cc99cd、
/// string #7ec699、comment #999、number/boolean/function #f08d49、
/// punctuation #ccc、operator #67cdcc;function = 标识符后随 `(`)。
pub fn highlight_rgb(code: &str) -> Vec<(String, Option<(u8, u8, u8)>)> {
    // 与 vue 生成器(cmd_vue.rs)自定义 'auto' 语言的 keyword 正则同源。
    // null/nil 在该正则中属 keyword;true/false 走 boolean(橙)。
    const KW: &[&str] = &["widget", "fn", "let", "mut", "const", "var", "if", "else", "for",
        "in", "loop", "while", "break", "continue", "return", "use", "import", "export",
        "type", "struct", "enum", "impl", "trait", "pub", "private", "static", "async",
        "await", "try", "catch", "throw", "new", "null", "nil", "self", "super"];
    const BOOL: &[&str] = &["true", "false"];
    // bash 代码块(npm install …)中这些命令词在 prism-bash 里着 function 橙;
    // 手写 lexer 不做语言区分,按词表近似同一色相。
    const CMD: &[&str] = &["npx", "npm", "yarn", "pnpm", "cd", "bun", "git", "cargo",
        "pip", "python", "node", "deno"];
    const C_KW: (u8, u8, u8) = (0xcc, 0x99, 0xcd); // keyword/builtin 紫
    const C_STR: (u8, u8, u8) = (0x7e, 0xc6, 0x99); // string 绿
    const C_COM: (u8, u8, u8) = (0x99, 0x99, 0x99); // comment 灰
    const C_NUM: (u8, u8, u8) = (0xf0, 0x8d, 0x49); // number/boolean/function 橙
    const C_PUN: (u8, u8, u8) = (0xcc, 0xcc, 0xcc); // punctuation 浅灰
    const C_OP: (u8, u8, u8) = (0x67, 0xcd, 0xcc);  // operator 青

    let bytes = code.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    let mut out: Vec<(String, Option<(u8, u8, u8)>)> = Vec::new();
    let push = |out: &mut Vec<_>, text: String, color: Option<(u8, u8, u8)>| {
        if !text.is_empty() {
            out.push((text, color));
        }
    };
    while i < n {
        let b = bytes[i];
        if (b == b'/' && i + 1 < n && bytes[i + 1] == b'/') || b == b'#' {
            let s = i;
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
            push(&mut out, code[s..i].to_string(), Some(C_COM));
        } else if b == b'"' || b == b'\'' || b == b'`' {
            let q = b;
            let s = i;
            i += 1;
            while i < n && bytes[i] != q {
                if bytes[i] == b'\\' && i + 1 < n {
                    i += 1;
                }
                i += 1;
            }
            if i < n {
                i += 1;
            }
            push(&mut out, code[s..i].to_string(), Some(C_STR));
        } else if b.is_ascii_digit() {
            let s = i;
            while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'.') {
                i += 1;
            }
            push(&mut out, code[s..i].to_string(), Some(C_NUM));
        } else if b.is_ascii_alphabetic() || b == b'_' {
            let s = i;
            while i < n && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-') {
                i += 1;
            }
            let t = &code[s..i];
            // Prism 'auto' 语言 function 正则:`ident(?=\s*\()` — 标识符后
            // (允许空白)紧跟左括号即 function(橙)。
            let mut j = i;
            while j < n && matches!(bytes[j], b' ' | b'\t') {
                j += 1;
            }
            let is_call = j < n && bytes[j] == b'(';
            let color = if KW.contains(&t) { Some(C_KW) }
                else if BOOL.contains(&t) || is_call || CMD.contains(&t) { Some(C_NUM) }
                else { None };
            push(&mut out, t.to_string(), color);
        } else if b == b' ' || b == b'\n' || b == b'\t' || b == b'\r' {
            let s = i;
            while i < n && matches!(bytes[i], b' ' | b'\n' | b'\t' | b'\r') {
                i += 1;
            }
            push(&mut out, code[s..i].to_string(), None);
        } else if matches!(b, b'+' | b'-' | b'*' | b'/' | b'%' | b'=' | b'<' | b'>'
            | b'!' | b'&' | b'|' | b'^' | b'~' | b'?' | b':') {
            let s = i;
            // 连续运算符字符并成一档(如 =>、==、&&)。
            while i < n && matches!(bytes[i], b'+' | b'-' | b'*' | b'/' | b'%' | b'='
                | b'<' | b'>' | b'!' | b'&' | b'|' | b'^' | b'~' | b'?' | b':') {
                i += 1;
            }
            push(&mut out, code[s..i].to_string(), Some(C_OP));
        } else {
            // 多字节字符按完整 UTF-8 长度推进(G3:按字节切片会 panic)。
            let ch_len = code[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            push(&mut out, code[i..i + ch_len].to_string(), Some(C_PUN));
            i += ch_len;
        }
    }
    out
}

/// show 命令的 command_result payload(读文件 → 逐行高亮 → Code 变体;
/// 文件缺失 → Failed)。JSON 在 Rust 侧拼装(见模块头注释)。
/// 行语义对齐旧 .at 实现:按 '\n' 切、剥尾 '\r'、末尾无内容的行不发;
/// span 显式带 r/g/b(默认前景 = 229,231,235,bold/italic 恒 false)。
pub fn show_result_json(block_id: i64, cwd: &str, path: &str) -> String {
    let display = std::path::Path::new(path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());
    let content = match std::fs::read(path) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
        Err(_) => {
            return serde_json::json!({
                "block_id": block_id,
                "cwd": cwd,
                "status": {"Failed": format!("show: {}: no such file", display)},
                "output": serde_json::Value::Null,
                "duration_ms": 0,
                "exit_code": 1,
            })
            .to_string();
        }
    };
    // 扩展名:最后一个 '.' 之后(无则空)。
    let lang = std::path::Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut parts: Vec<&str> = content.split('\n').collect();
    // 对齐 .at 侧行为:结尾的空段不发(show 的行扫描只发射有内容的尾段)。
    if parts.last() == Some(&"") {
        parts.pop();
    }
    let mut lines_json: Vec<serde_json::Value> = Vec::new();
    for line in parts {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let mut spans_json: Vec<serde_json::Value> = Vec::new();
        for (text, color) in highlight_rgb(line) {
            let (r, g, b) = color.unwrap_or((229, 231, 235));
            spans_json.push(serde_json::json!({
                "text": text, "r": r, "g": g, "b": b, "bold": false, "italic": false,
            }));
        }
        lines_json.push(serde_json::Value::Array(spans_json));
    }

    serde_json::json!({
        "block_id": block_id,
        "cwd": cwd,
        "status": "Success",
        "output": {"Code": {"lines": lines_json, "language": lang}},
        "duration_ms": 0,
        "exit_code": 0,
    })
    .to_string()
}
