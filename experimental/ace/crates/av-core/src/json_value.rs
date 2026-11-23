//! PLAN-117（B-05 S1b）：checked 域 **JSON 值语义**原语目录与执行（`json.value/1`）。
//!
//! 源级目录在 ace-core `source::json_ops`（resolve 拦截/签名发射的单一真源）；
//! 本文件是 av-core 侧的**准入 + 执行**：`auto.json.parse.v1` 经 PlatformCall
//! 通道（与 PLAN-982 config 原语同通道）分派到本文件的解析器，返回**真实
//! Value**——object→`Value::Object`（PLAN-964 `ObjectData`，键=`ObjectKey::
//! Text`）、数组→`Value::List`（`ListHandle::detached`）、标量→I32/F64/Bool/
//! Str（detached 句柄）、`null`→`Value::Nil`。
//!
//! 预算：解析深度 256、节点数 65_536、文本字节 1 MiB（超限=Resource，与
//! config 的 DynTypeNodes 预算族同语义）。成员读的 missing 语义（JSON null）
//! 由既有 `MirInstr::ObjectGet` 消费侧承载——PLAN-964 object 读取在键缺席时
//! 按 elem 的 Nil 形返回（S1a resolve 侧 ObjectGetOpt 语义的执行底座）。

use crate::contract::{
    ListHandle, ObjectData, ObjectKey, StrHandle, Value,
};
use crate::host::HostError;
use crate::mir::{MirHostImport, MirType};

pub const CATEGORY: &str = "json.value/1";
pub const CAPABILITY: &str = "json.value/1";
pub const MAX_CALL_BYTES: usize = 65_536;
/// JSON 文本预算（1 MiB）。
pub const MAX_TEXT_BYTES: usize = 1 << 20;
/// 解析深度预算。
const MAX_DEPTH: usize = 256;
/// 值节点预算。
const MAX_NODES: usize = 65_536;

/// 该符号是否是 json 值原语。
pub fn is_json_symbol(sym: &str) -> bool {
    sym.starts_with("auto.json.")
}

/// 符号 → 结构（未知/拼写错误 = None，绝不静默接受）。
fn spec_of(symbol: &str) -> Option<()> {
    let rest = symbol.strip_prefix("auto.json.")?;
    let rest = rest.strip_suffix(".v1")?;
    match rest {
        "parse" | "stringify" | "number-f32" => Some(()),
        _ => None,
    }
}

/// 降级侧声明：符号必须可解析，签名由目录给出（与源级重载同一行）。
pub fn declaration(symbol: &str, sig: u32) -> Result<MirHostImport, crate::host::AdmissionError> {
    spec_of(symbol).ok_or(crate::host::AdmissionError {
        code: "json.unknown-symbol",
        message: "unknown json value primitive".into(),
    })?;
    Ok(MirHostImport {
        category: CATEGORY.into(),
        symbol: symbol.into(),
        sig,
        capability: CAPABILITY.into(),
        max_call_bytes: MAX_CALL_BYTES as u32,
    })
}

/// 准入校验：类别/能力/预算/符号逐项核（不是只看符号前缀）。
pub fn validate_import(
    module: &crate::mir::MirModule,
    h: &MirHostImport,
) -> Result<(), crate::host::AdmissionError> {
    if spec_of(&h.symbol).is_none() {
        return Err(crate::host::AdmissionError {
            code: "json.unknown-symbol",
            message: "unknown json value primitive".into(),
        });
    }
    let signature = module.sig_of(h.sig);
    let ok = h.category == CATEGORY
        && h.capability == CAPABILITY
        && h.max_call_bytes == MAX_CALL_BYTES as u32
        && signature.is_some_and(|sig| {
            if h.symbol != "auto.json.number-f32.v1" {
                return true;
            }
            let [parameter] = sig.params.as_slice() else { return false };
            module.types.get(*parameter as usize) == Some(&MirType::Dyn)
                && module.types.get(sig.result as usize) == Some(&MirType::F32)
        });
    if !ok {
        return Err(crate::host::AdmissionError {
            code: "json.signature",
            message: "json category/capability/budget/signature tuple mismatch".into(),
        });
    }
    if !module.capabilities.iter().any(|c| c == CATEGORY) {
        return Err(crate::host::AdmissionError {
            code: "json.unauthorized",
            message: "json capability requirement is missing".into(),
        });
    }
    Ok(())
}

/// JSON 序列化（PLAN-117 round-23④）：Value → JSON 文本。Dyn 值解包其
/// 载荷；Object 键按 `ObjectKey`（Text 原样、Int/Bool 转字符串键）。
fn serialize_value(v: &Value, out: &mut Vec<u8>, depth: usize) -> Result<(), HostError> {
    if depth > MAX_DEPTH {
        return Err(HostError::Resource);
    }
    match v {
        Value::Nil => out.extend_from_slice(b"null"),
        Value::Bool(b) => out.extend_from_slice(if *b { b"true" } else { b"false" }),
        Value::I32(x) => out.extend_from_slice(x.to_string().as_bytes()),
        Value::I64(x) => out.extend_from_slice(x.to_string().as_bytes()),
        Value::F64(x) => out.extend_from_slice(format!("{x}").as_bytes()),
        Value::Str(h) => {
            out.push(b'"');
            for &b in h.bytes() {
                match b {
                    0x22 | 0x5c => {
                        out.push(0x5c);
                        out.push(b);
                    }
                    0x0a => out.extend_from_slice(b"\n"),
                    0x0d => out.extend_from_slice(b"\r"),
                    0x09 => out.extend_from_slice(b"\t"),
                    _ => out.push(b),
                }
            }
            out.push(b'"');
        }
        Value::List(h) => {
            out.push(b'[');
            let buf = h.rc.borrow();
            for (i, e) in buf.elems.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                serialize_value(e, out, depth + 1)?;
            }
            out.push(b']');
        }
        Value::Object(cell) => {
            out.push(b'{');
            let borrowed = cell.borrow();
            for (i, (k, val)) in borrowed.entries.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                let key_text = match k {
                    ObjectKey::Text(h) => String::from_utf8_lossy(h.bytes()).into_owned(),
                    ObjectKey::Int(x) => x.to_string(),
                    ObjectKey::Bool(b) => b.to_string(),
                };
                out.push(b'"');
                out.extend_from_slice(key_text.as_bytes());
                out.extend_from_slice(b"\":");
                serialize_value(val, out, depth + 1)?;
            }
            out.push(b'}');
        }
        Value::Dyn(d) => serialize_value(d.as_value(), out, depth + 1)?,
        other => {
            return Err(HostError::Io(format!(
                "json.stringify: cannot serialize {}",
                other.type_name()
            )));
        }
    }
    Ok(())
}

/// 执行分派：`auto.json.parse.v1(text) -> Value`。
pub fn dispatch(symbol: &str, args: &[Value]) -> Result<Value, HostError> {
    match symbol {
        "auto.json.number-f32.v1" => {
            let invalid = || HostError::Io("expected a finite f32 JSON number".into());
            let [Value::Dyn(number)] = args else { return Err(invalid()) };
            let value = match number.as_value() {
                Value::I32(value) => *value as f32,
                Value::F32(value) => *value,
                Value::F64(value) => *value as f32,
                _ => return Err(invalid()),
            };
            if !value.is_finite() {
                return Err(invalid());
            }
            Ok(Value::F32(value))
        }
        "auto.json.stringify.v1" => {
            let Some(v) = args.first() else {
                return Err(HostError::Io("json.stringify: missing argument".into()));
            };
            let mut out = Vec::new();
            serialize_value(v, &mut out, 0usize)?;
            Ok(Value::Str(StrHandle::detached(out.into_boxed_slice())))
        }
        "auto.json.parse.v1" => {
            let Some(text) = args.first() else {
                return Err(HostError::Io("json.parse: missing argument".into()));
            };
            let owned: Vec<u8>;
            let bytes: &[u8] = match text {
                Value::Str(h) => h.bytes(),
                Value::StrView(view) => match &view.src {
                    crate::contract::StrViewSrc::Static { buf } => {
                        let (s, e) = crate::owned_text::scalar_range_to_bytes(
                            buf,
                            view.start as i32,
                            view.end as i32,
                        )
                        .map_err(|_| HostError::Io("json.parse: invalid text range".into()))?;
                        &buf[s..e]
                    }
                    crate::contract::StrViewSrc::Share(_) => {
                        let resolved = crate::place::resolve_share_view(view).ok_or_else(|| {
                            HostError::Io("json.parse: stale text view".into())
                        })?;
                        owned = resolved
                            .handle
                            .bytes()
                            [project_range(resolved.handle.bytes(), &view)]
                            .to_vec();
                        &owned
                    }
                },
                other => {
                    return Err(HostError::Io(format!(
                        "json.parse: argument holds {} (expected text)",
                        other.type_name()
                    )));
                }
            };
            if bytes.len() > MAX_TEXT_BYTES {
                return Err(HostError::Resource);
            }
            let mut p = Parser {
                b: bytes,
                i: 0,
                nodes: 0,
            };
            p.skip_ws();
            let v = p.value(0)?;
            p.skip_ws();
            if p.i != p.b.len() {
                return Err(HostError::Io(format!(
                    "json.parse: trailing content at byte {}",
                    p.i
                )));
            }
            // PLAN-117（B-05 S1b）：Dyn 擦除——json 值域的宿主槽是 Dyn 型
            // （resolve 侧 json.parse → VT::Dyn）；PLAN-824 的 DynValue::pack
            // 是唯一铸造面，逐值显式擦除（容器先擦子值）。
            erase_value(v)
        }
        _ => Err(HostError::Io(format!(
            "json.unknown-symbol: {symbol}"
        ))),
    }
}

/// PLAN-824 显式擦除：标量/容器 → `Value::Dyn(DynValue)`（pack=唯一铸造
/// 面；描述符按值形同构构造）。
fn erase_value(v: Value) -> Result<Value, HostError> {
    use crate::dynamic::{DynType, DynValue};
    let (value, ty) = match v {
        Value::Object(cell) => {
            // 条目值先擦（Object cell 的条目槽在 DYN 读侧按 elem 消费）。
            if let Ok(mut borrowed) = cell.try_borrow_mut() {
                for (_, entry) in borrowed.entries.iter_mut() {
                    let erased = erase_value(std::mem::replace(entry, Value::Nil))?;
                    *entry = erased;
                }
            }
            (Value::Object(cell), DynType::Object)
        }
        Value::List(handle) => {
            if let Ok(mut borrowed) = handle.rc.try_borrow_mut() {
                for elem in borrowed.elems.iter_mut() {
                    let erased = erase_value(std::mem::replace(elem, Value::Nil))?;
                    *elem = erased;
                }
            }
            (Value::List(handle), DynType::List(Box::new(DynType::Dyn)))
        }
        Value::Str(h) => (Value::Str(h), DynType::Str),
        Value::I32(x) => (Value::I32(x), DynType::I32),
        Value::I64(x) => (Value::I64(x), DynType::I64),
        Value::F64(x) => (Value::F64(x), DynType::F64),
        Value::Bool(x) => (Value::Bool(x), DynType::Bool),
        Value::Nil => (Value::Nil, DynType::Nil),
        other => {
            return Err(HostError::Io(format!(
                "json.parse: cannot erase {} value",
                other.type_name()
            )));
        }
    };
    let packed = DynValue::pack(value, ty)
        .map_err(|e| HostError::Io(format!(
            "json.parse: value erasure rejected ({})",
            e.code()
        )))?;
    Ok(Value::Dyn(std::rc::Rc::new(packed)))
}

fn project_range(bytes: &[u8], view: &crate::contract::StrView) -> std::ops::Range<usize> {
    match crate::owned_text::scalar_range_to_bytes(bytes, view.start as i32, view.end as i32) {
        Ok((s, e)) => s..e,
        Err(_) => 0..bytes.len(),
    }
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
    nodes: usize,
}

impl<'a> Parser<'a> {
    fn skip_ws(&mut self) {
        while let Some(c) = self.b.get(self.i) {
            match c {
                b' ' | b'\t' | b'\n' | b'\r' => self.i += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }

    fn eat(&mut self, want: u8) -> Result<(), HostError> {
        if self.peek() == Some(want) {
            self.i += 1;
            Ok(())
        } else {
            Err(HostError::Io(format!(
                "json.parse: expected `{}` at byte {}",
                want as char, self.i
            )))
        }
    }

    fn charge(&mut self) -> Result<(), HostError> {
        self.nodes += 1;
        if self.nodes > MAX_NODES || self.i > MAX_TEXT_BYTES {
            return Err(HostError::Resource);
        }
        Ok(())
    }

    fn value(&mut self, depth: usize) -> Result<Value, HostError> {
        if depth > MAX_DEPTH {
            return Err(HostError::Resource);
        }
        self.charge()?;
        match self.peek() {
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => self.string().map(|s| Value::Str(StrHandle::detached(s))),
            Some(b't') => self.literal("true").map(|_| Value::Bool(true)),
            Some(b'f') => self.literal("false").map(|_| Value::Bool(false)),
            Some(b'n') => self.literal("null").map(|_| Value::Nil),
            Some(c) if c == b'-' || c.is_ascii_digit() => self.number(),
            other => Err(HostError::Io(format!(
                "json.parse: unexpected byte {other:?} at {}",
                self.i
            ))),
        }
    }

    fn literal(&mut self, word: &str) -> Result<(), HostError> {
        if self.b[self.i..].starts_with(word.as_bytes()) {
            self.i += word.len();
            Ok(())
        } else {
            Err(HostError::Io(format!(
                "json.parse: bad literal at byte {}",
                self.i
            )))
        }
    }

    fn object(&mut self, depth: usize) -> Result<Value, HostError> {
        self.eat(b'{')?;
        let mut entries: Vec<(ObjectKey, Value)> = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.i += 1;
            return Ok(Value::Object(std::rc::Rc::new(std::cell::RefCell::new(
                ObjectData { entries },
            ))));
        }
        loop {
            self.skip_ws();
            let key = self.string()?;
            self.skip_ws();
            self.eat(b':')?;
            self.skip_ws();
            let value = self.value(depth + 1)?;
            entries.push((ObjectKey::Text(StrHandle::detached(key)), value));
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.i += 1;
                }
                Some(b'}') => {
                    self.i += 1;
                    return Ok(Value::Object(std::rc::Rc::new(std::cell::RefCell::new(
                        ObjectData { entries },
                    ))));
                }
                other => {
                    return Err(HostError::Io(format!(
                        "json.parse: expected `,` or `}}` at byte {} (got {other:?})",
                        self.i
                    )));
                }
            }
        }
    }

    fn array(&mut self, depth: usize) -> Result<Value, HostError> {
        self.eat(b'[')?;
        let mut elems: Vec<Value> = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.i += 1;
            return Ok(Value::List(ListHandle::detached(elems)));
        }
        loop {
            self.skip_ws();
            elems.push(self.value(depth + 1)?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.i += 1;
                }
                Some(b']') => {
                    self.i += 1;
                    return Ok(Value::List(ListHandle::detached(elems)));
                }
                other => {
                    return Err(HostError::Io(format!(
                        "json.parse: expected `,` or `]` at byte {} (got {other:?})",
                        self.i
                    )));
                }
            }
        }
    }

    fn string(&mut self) -> Result<Box<[u8]>, HostError> {
        self.eat(b'"')?;
        let mut out: Vec<u8> = Vec::new();
        loop {
            let Some(c) = self.peek() else {
                return Err(HostError::Io("json.parse: unterminated string".into()));
            };
            self.i += 1;
            match c {
                b'"' => return Ok(out.into_boxed_slice()),
                b'\\' => {
                    let Some(esc) = self.peek() else {
                        return Err(HostError::Io("json.parse: bad escape".into()));
                    };
                    self.i += 1;
                    match esc {
                        b'"' => out.push(b'"'),
                        b'\\' => out.push(b'\\'),
                        b'/' => out.push(b'/'),
                        b'b' => out.push(0x08),
                        b'f' => out.push(0x0C),
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'u' => {
                            let cp = self.hex4()?;
                            // 代理对（S1b 边界：Basic Multilingual Plane 之外
                            // 的 astral 字符按 UTF-8 替换序列降级——029/031
                            // 语料不含；完整代理对面=后续切片）。
                            let ch = char::from_u32(cp as u32)
                                .unwrap_or(char::REPLACEMENT_CHARACTER);
                            let mut buf = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        other => {
                            return Err(HostError::Io(format!(
                                "json.parse: unknown escape \\{}",
                                other as char
                            )));
                        }
                    }
                }
                other => out.push(other),
            }
        }
    }

    fn hex4(&mut self) -> Result<u16, HostError> {
        if self.i + 4 > self.b.len() {
            return Err(HostError::Io("json.parse: bad \\u escape".into()));
        }
        let s = std::str::from_utf8(&self.b[self.i..self.i + 4])
            .map_err(|_| HostError::Io("json.parse: bad \\u escape".into()))?;
        let v = u16::from_str_radix(s, 16)
            .map_err(|_| HostError::Io("json.parse: bad \\u escape".into()))?;
        self.i += 4;
        Ok(v)
    }

    fn number(&mut self) -> Result<Value, HostError> {
        let start = self.i;
        if self.peek() == Some(b'-') {
            self.i += 1;
        }
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.i += 1;
        }
        let mut is_float = false;
        if self.peek() == Some(b'.') {
            is_float = true;
            self.i += 1;
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.i += 1;
            }
        }
        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            is_float = true;
            self.i += 1;
            if matches!(self.peek(), Some(b'+') | Some(b'-')) {
                self.i += 1;
            }
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.i += 1;
            }
        }
        let text = std::str::from_utf8(&self.b[start..self.i])
            .map_err(|_| HostError::Io("json.parse: bad number".into()))?;
        if !is_float {
            if let Ok(v) = text.parse::<i32>() {
                return Ok(Value::I32(v));
            }
        }
        text.parse::<f64>()
            .map(Value::F64)
            .map_err(|_| HostError::Io(format!("json.parse: bad number `{text}`")))
    }
}
