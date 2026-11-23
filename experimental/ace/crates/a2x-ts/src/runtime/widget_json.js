class A2xDyn {
  constructor(type, value) {
    this.type = type;
    this.value = value;
  }
}

class A2xObject {
  constructor(entries) {
    this.entries = entries;
  }
}

function a2x_obj_new(entries) {
  return new A2xObject(entries);
}

function a2x_dyn_pack(value, type) {
  if (type === "dyn") return value;
  if (type === "str" || type === "Str") {
    return new A2xDyn("str", _asBytes(value).slice());
  }
  return new A2xDyn(type, value);
}

function a2x_dyn_is_nil(value) {
  return value === null || (value instanceof A2xDyn && value.type === "nil");
}

function _json_type_equal(left, right) {
  while (Array.isArray(left) && Array.isArray(right) && left[0] === right[0]) {
    left = left[1];
    right = right[1];
  }
  return left === right;
}

function a2x_dyn_take(source, type) {
  if (!(source instanceof A2xDyn)) _trap("DynTypeMismatch", "expected a dynamic value");
  if (type === "dyn") return source;
  if (source.type === "str") {
    if (type === "str") return source.value;
    if (type === "Str") return new A2xOwnedStr(source.value);
  }
  if (_json_type_equal(source.type, type)) return source.value;
  if (source.type[0] === "list" && type[0] === "list") {
    const list = new A2xList();
    for (const item of source.value.items) {
      a2x_bill(1);
      const boxed = a2x_dyn_pack(item, source.type[1]);
      list.items.push(a2x_dyn_take(boxed, type[1]));
    }
    return list;
  }
  _trap("DynTypeMismatch", "dynamic value does not match " + type);
}

function _json_key(key) {
  if (key instanceof A2xDyn) key = key.value;
  if (typeof key === "number") return _checkedI32(key, "object key");
  if (typeof key === "boolean") return key;
  return a2x_str_utf8(key);
}

function _json_entries(base) {
  const object = base instanceof A2xDyn ? base.value : base;
  if (!(object instanceof A2xObject)) _trap("DynTypeMismatch", "expected an object");
  a2x_bill(object.entries.length);
  return object.entries;
}

function a2x_obj_get(base, key) {
  if (base instanceof A2xDyn && base.type[0] === "list") {
    return a2x_dyn_pack(a2x_list_get(base.value, _json_key(key)), base.type[1]);
  }
  const wanted = _json_key(key);
  const entry = _json_entries(base).find(([candidate]) => candidate === wanted);
  if (!entry) _trap("ObjectMissing", "object key is absent: " + wanted);
  return entry[1];
}

function a2x_obj_set(base, key, value) {
  if (base instanceof A2xDyn && base.type[0] === "list") {
    a2x_list_set(base.value, _json_key(key), a2x_dyn_take(value, base.type[1]));
    return;
  }
  const wanted = _json_key(key);
  const entries = _json_entries(base);
  const entry = entries.find(([candidate]) => candidate === wanted);
  if (entry) entry[1] = value;
  else entries.push([wanted, value]);
}

function a2x_dyn_display(value) {
  if (!(value instanceof A2xDyn)) _trap("DynTypeMismatch", "expected a dynamic scalar");
  if (value.type === "str") return new A2xOwnedStr(value.value);
  if (value.type === "f32") return a2x_str_display_f32(value.value);
  if (["i32", "u32", "i64", "u64", "f64"].includes(value.type)) {
    return new A2xOwnedStr(_encoder.encode(String(value.value)));
  }
  _trap("DynTypeMismatch", "expected displayable text or number");
}

const _jsonMaxText = 1 << 20;
const _jsonMaxDepth = 256;
const _jsonMaxNodes = 65_536;

function a2x_json_parse(input) {
  const bytes = _asBytes(input);
  if (bytes.length > _jsonMaxText) _trap("Budget", "json.parse: text limit exceeded");
  a2x_bill(bytes.length);
  const text = a2x_str_utf8(bytes);
  const token = /[ \t\r\n]*("(?:[^"\\\u0000-\u001f]|\\(?:["\\/bfnrt]|u[\da-fA-F]{4}))*"|-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?|true|false|null|[{}[\]:,])/y;
  let cursor = 0;
  let next = null;
  let nodes = 0;

  function malformed() {
    _trap("Fail", "json.parse: invalid JSON near character " + cursor);
  }

  function advance() {
    token.lastIndex = cursor;
    const scanned = token.exec(text);
    if (scanned) {
      next = scanned[1];
      cursor = token.lastIndex;
    } else {
      if (!/^[ \t\r\n]*$/.test(text.slice(cursor))) malformed();
      next = null;
    }
  }

  function read(expected) {
    if (next !== expected) malformed();
    advance();
  }

  function value(depth) {
    if (depth > _jsonMaxDepth || ++nodes > _jsonMaxNodes) {
      _trap("Budget", "json.parse: value limit exceeded");
    }
    a2x_bill(1);
    const item = next;
    if (item === null) malformed();
    advance();
    switch (item) {
      case "{": {
        const entries = [];
        if (next !== "}") {
          while (true) {
            if (next === null || next[0] !== '"') malformed();
            const key = a2x_str_utf8(_encoder.encode(JSON.parse(next)));
            advance();
            read(":");
            entries.push([key, value(depth + 1)]);
            if (next !== ",") break;
            advance();
          }
        }
        read("}");
        return new A2xDyn(["object", "dyn"], new A2xObject(entries));
      }
      case "[": {
        const list = new A2xList();
        if (next !== "]") {
          while (true) {
            list.items.push(value(depth + 1));
            if (next !== ",") break;
            advance();
          }
        }
        read("]");
        return new A2xDyn(["list", "dyn"], list);
      }
      case "null": return new A2xDyn("nil", null);
      case "true": return new A2xDyn("bool", true);
      case "false": return new A2xDyn("bool", false);
    }
    if (item[0] === '"') return new A2xDyn("str", _encoder.encode(JSON.parse(item)));
    if (!/^[\d-]/.test(item)) malformed();
    const number = Number(item);
    if (!Number.isFinite(number)) malformed();
    const i32 = !/[.eE]/.test(item) && number >= -2147483648 && number <= 2147483647;
    return new A2xDyn(i32 ? "i32" : "f64", i32 ? number + 0 : number);
  }

  advance();
  const parsed = value(0);
  if (next !== null) malformed();
  return parsed;
}

function a2x_json_number_f32(input) {
  if (input instanceof A2xDyn && ['i32', 'f32', 'f64'].includes(input.type)) {
    const value = Math.fround(input.value);
    if (Number.isFinite(value)) return value;
  }
  _trap("Fail", "expected a finite f32 JSON number");
}

function a2x_json_stringify(input) {
  const output = [];
  let nodes = 0;
  function append(text) {
    a2x_bill(text.length);
    output.push(text);
  }
  function value(item, depth) {
    if (depth > _jsonMaxDepth || ++nodes > _jsonMaxNodes) {
      _trap("Budget", "json.stringify: value limit exceeded");
    }
    if (item instanceof A2xDyn) item = item.value;
    if (item instanceof A2xObject) {
      append("{");
      for (const [index, [key, entry]] of item.entries.entries()) {
        if (index !== 0) append(",");
        append(JSON.stringify(String(key)));
        append(":");
        value(entry, depth + 1);
      }
      append("}");
    } else if (item instanceof A2xList) {
      append("[");
      for (const [index, entry] of item.items.entries()) {
        if (index !== 0) append(",");
        value(entry, depth + 1);
      }
      append("]");
    } else if (item instanceof A2xOwnedStr || item instanceof Uint8Array) {
      append(JSON.stringify(a2x_str_utf8(item)));
    } else if (typeof item === "bigint") {
      append(item.toString());
    } else if (item === null || typeof item === "boolean" || Number.isFinite(item)) {
      append(JSON.stringify(item));
    } else {
      _trap("Fail", "json.stringify: value is outside the JSON surface");
    }
  }
  value(input, 0);
  return new A2xOwnedStr(_encoder.encode(output.join("")));
}
