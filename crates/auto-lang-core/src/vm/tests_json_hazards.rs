//! PLAN-995 T-01（W01）：VM json/codegen hazard 目录永久 probe。
//!
//! T-00 处置表（见 docs/plans/995-vm-json-natives-perf-and-hazard-closure.md §9）
//! 的结论以可执行 oracle 钉住，防止回归：
//! - H1 动态集合二遍迭代：master+旧 pin 双绿，处置=豁免（此处钉住语义）。
//! - H2 大文本 `==` 比较位置（if/expr/交换/inline trim）：受控矩阵全绿；
//!   round23 的"inline trim 误判"归因修正为探针源缺陷（缺 `]` 闭合），
//!   此处钉住含闭合 oracle 的四位置对照 + 负控。
//! - H3 大串累积模块布局：布局无关（min/extrafn 逐字节同），钉住双布局等价。
//! - H4 模块 var 大串存取/比较：基线安全，钉住。
//! - H5（T-00 新发现）`json.stringify` 少参 ABI：1/2 参调用弹栈下溢
//!   （virt_memory pop_nv panic）或返回垃圾；T-02 修复的 red→green 永久例。
//!
//! 文本规模：h1 用 1000 册（round23 目录判别量级），比较/布局族用 300-400 册
//! （~65-87KB，语义 oracle 与规模解耦；规模阈值本身已被 T-00 h2b 矩阵证伪）。
//! 所有夹具在 .at 内确定性生成（repo-hermetic，无外部文件依赖）。

/// 生成 N 册 catalog 文档的 .at mk_doc 函数文本（probe-02 形状）。
fn mk_doc_fn() -> &'static str {
    r#"
fn mk_doc(n int) str {
    var doc str = "{\"version\":1,\"groups\":[{\"group_id\":\"group-a\",\"name\":\"甲\",\"created_at\":1},{\"group_id\":\"group-b\",\"name\":\"乙\",\"created_at\":2}],\"books\":["
    var i int = 0
    while i < n {
        if i > 0 {
            doc = doc + ","
        }
        var bid str = "b"
        if i < 10 {
            bid = bid + "000"
        } else {
            if i < 100 {
                bid = bid + "00"
            } else {
                if i < 1000 {
                    bid = bid + "0"
                }
            }
        }
        bid = bid + i.to_string()
        var it str = "{\"book_id\":\""
        it = it + bid
        it = it + "\",\"cover\":\"\",\"cover_source\":\"import\",\"title_override\":\"中文 书"
        it = it + i.to_string()
        it = it + "\",\"author_override\":\"作者\",\"tags\":[\"标签\"],\"group_ids\":[\"group-a\"],\"status\":\"unread\",\"last_opened_at\":1794038182,\"added_order\":"
        it = it + (i + 1).to_string()
        it = it + "}"
        doc = doc + it
        i = i + 1
    }
    doc = doc + "],\"prefs\":{\"view_mode\":\"grid\",\"sort_by\":\"title\",\"sort_asc\":true,\"filter_status\":\"all\",\"filter_format\":\"all\",\"filter_group\":\"\",\"query\":\"\"}}"
    return doc
}
"#
}

/// bool → "T"/"F" 的 .at 辅助（与主语句同段，先定义后用）。
const BOOL_FN: &str = r#"
fn b(x bool) str {
    if x {
        return "T"
    }
    return "F"
}
"#;

/// 重建 books 段内层（无外括号）的 helper 形（json_escape + entry_json 链，
/// proof_body 形状；与原文段的括号对齐由调用方处理）。
fn rebuild_helper_fn() -> &'static str {
    r#"
fn json_escape(s str) str {
    var out str = s
    if out.find("\\") >= 0 {
        out = out.replace("\\", "\\\\")
    }
    if out.find("\"") >= 0 {
        out = out.replace("\"", "\\\"")
    }
    if out.find("\n") >= 0 {
        out = out.replace("\n", "\\n")
    }
    return out
}

fn entry_json(bid str, tovr str, tags_json str, status str, order int) str {
    var out str = "{\"book_id\":\""
    out = out + json_escape(bid)
    out = out + "\",\"cover\":\"\",\"cover_source\":\"import\",\"title_override\":\""
    out = out + json_escape(tovr)
    out = out + "\",\"author_override\":\"作者\",\"tags\":"
    out = out + tags_json
    out = out + ",\"group_ids\":[\"group-a\"],\"status\":\""
    out = out + json_escape(status)
    out = out + "\",\"last_opened_at\":1794038182,\"added_order\":"
    out = out + order.to_string()
    out = out + "}"
    return out
}

fn rebuild_books_inner(raw str) str {
    let c = json.parse(raw)
    var bs str = ""
    var bn int = 0
    for bt in c.books {
        if bn > 0 {
            bs = bs + ","
        }
        bs = bs + entry_json("" + bt.book_id, "" + bt.title_override, "[\"标签\"]", "" + bt.status, bt.added_order)
        bn = bn + 1
    }
    return bs
}
"#
}

/// H1：千条文档 books 二遍迭代 + groups 后读 + prefs 字段在迭代后读取。
#[test]
fn h1_dynamic_collection_two_pass_1000() {
    let code = format!(
        r#"{mk_doc}
{bool_fn}
let doc = json.parse(mk_doc(1000))
var n1 int = 0
for bt in doc.books {{
    n1 = n1 + 1
}}
var n2 int = 0
for bt2 in doc.books {{
    n2 = n2 + 1
}}
var gn int = 0
for g in doc.groups {{
    gn = gn + 1
}}
let vm_after str = "" + doc.prefs.view_mode
n1.to_string() + "," + n2.to_string() + "," + gn.to_string() + "," + vm_after
"#,
        mk_doc = mk_doc_fn(),
        bool_fn = BOOL_FN
    );
    let result = crate::run(&code).expect("run failed");
    assert_eq!(
        result, "1000,1000,2,grid",
        "H1 two-pass iteration regressed"
    );
}

/// H1 边界：嵌套集合（tags）二遍 + 同对象字段二次读。
#[test]
fn h1_nested_collection_two_pass() {
    let code = format!(
        r#"{mk_doc}
let doc = json.parse(mk_doc(50))
let b0 = doc.books[0]
var t1 int = 0
for tv in b0.tags {{
    t1 = t1 + 1
}}
var t2 int = 0
for tv2 in b0.tags {{
    t2 = t2 + 1
}}
let bid_after str = "" + b0.book_id
let bid_idx str = "" + doc.books[0].book_id
t1.to_string() + "," + t2.to_string() + "," + bid_after + "," + bid_idx
"#,
        mk_doc = mk_doc_fn()
    );
    let result = crate::run(&code).expect("run failed");
    assert_eq!(
        result, "1,1,b0000,b0000",
        "H1 nested two-pass / second field read regressed"
    );
}

/// H2：~87KB 累积串（helper 形重建）在 if/expr/交换/inline-trim 四位置与
/// 原文段比较（两条构造路径、内容相等），加负控（真不等必须判 false）。
#[test]
fn h2_big_string_eq_positions_and_inline_trim() {
    let code = format!(
        r#"{mk_doc}
{bool_fn}
{helper}
let raw str = mk_doc(400)
// 原文 books 内层段（substr 直取——与重建不同的构造路径，内容相等）
let lead int = raw.find("\"books\":[") + 9
let tail int = raw.find("],\"prefs\"")
let books_raw str = raw.substr(lead, tail - lead)
let rebuilt str = rebuild_books_inner(raw)
var eq_if bool = false
if rebuilt == books_raw {{
    eq_if = true
}}
var eq_expr bool = rebuilt == books_raw
var eq_swap bool = false
if books_raw == rebuilt {{
    eq_swap = true
}}
var eq_inline_trim bool = false
if rebuilt == books_raw.trim() {{
    eq_inline_trim = true
}}
var neg bool = rebuilt == (books_raw + " ")
b(eq_if) + "," + b(eq_expr) + "," + b(eq_swap) + "," + b(eq_inline_trim) + "," + b(neg) + "," + rebuilt.len().to_string()
"#,
        mk_doc = mk_doc_fn(),
        bool_fn = BOOL_FN,
        helper = rebuild_helper_fn()
    );
    let result = crate::run(&code).expect("run failed");
    let fields: Vec<&str> = result.split(',').collect();
    assert_eq!(fields[0], "T", "H2 if-position compare regressed: {result}");
    assert_eq!(fields[1], "T", "H2 expr-position regressed: {result}");
    assert_eq!(fields[2], "T", "H2 swapped-position regressed: {result}");
    assert_eq!(fields[3], "T", "H2 inline-trim compare regressed: {result}");
    assert_eq!(
        fields[4], "F",
        "H2 negative control must stay false (oracle distinguishability): {result}"
    );
    let rebuilt_len: usize = fields[5].parse().expect("len field");
    assert!(
        rebuilt_len > 80_000,
        "rebuilt size sanity (expected ~87KB): {result}"
    );
}

/// H3：同一重建函数文本在"最小布局"与"额外函数插入布局"两程序中
/// 输出必须逐字节相等（布局无关性）。
#[test]
fn h3_accumulation_layout_independence() {
    let stmts = r#"let raw str = mk_doc(300)
rebuild_books_inner(raw)"#;
    let minimal = format!(
        "{mk_doc}\n{helper}\n{stmts}\n",
        mk_doc = mk_doc_fn(),
        helper = rebuild_helper_fn()
    );
    // extrafn 布局：主语句之前插入一个同形状的额外累积/比较函数
    // （round23 probe-13 形态）。
    let extra_fn = r#"
fn extra_layout_shape(raw str) bool {
    if raw.len() > 1048576 { return false }
    let c = json.parse(raw)
    var bs str = ""
    var bn int = 0
    for bt in c.books {
        if bn > 0 {
            bs = bs + ","
        }
        bs = bs + entry_json("" + bt.book_id, "" + bt.title_override, "[\"标签\"]", "" + bt.status, bt.added_order)
        bn = bn + 1
    }
    if bs == "]" {
        return true
    }
    return false
}
"#;
    let extra_layout = format!(
        "{mk_doc}\n{helper}\n{extra_fn}\n{stmts}\n",
        mk_doc = mk_doc_fn(),
        helper = rebuild_helper_fn(),
        extra_fn = extra_fn
    );
    let out_minimal = crate::run(&minimal).expect("minimal layout run failed");
    let out_extra = crate::run(&extra_layout).expect("extra-fn layout run failed");
    assert_eq!(
        out_minimal, out_extra,
        "H3 layout independence regressed: rebuilt output differs across module layouts"
    );
    assert!(
        out_minimal.len() > 60_000,
        "unexpected rebuilt size (sanity anchor): {}",
        out_minimal.len()
    );
}

/// H4：模块 var 大串赋值 + 存串在 if/expr 两位置比较（基线安全钉住）。
#[test]
fn h4_module_var_large_string_store_and_compare() {
    let code = format!(
        r#"{mk_doc}
var stored_big str = ""
fn build_and_store() int {{
    let raw str = mk_doc(300)
    var acc str = ""
    var i int = 0
    let total int = raw.len()
    while i < total {{
        let step int = 4096
        var end int = i + step
        if end > total {{
            end = total
        }}
        acc = acc + raw.substr(i, end - i)
        i = end
    }}
    stored_big = acc
    return acc.len()
}}
let built_len int = build_and_store()
let raw2 str = mk_doc(300)
var eq_inline str = "false"
if stored_big == raw2 {{
    eq_inline = "true"
}}
var eq_expr str = "false"
if stored_big == raw2 {{
    eq_expr = "true"
}}
built_len.to_string() + "," + stored_big.len().to_string() + "," + eq_inline + "," + eq_expr
"#,
        mk_doc = mk_doc_fn()
    );
    let result = crate::run(&code).expect("run failed");
    let fields: Vec<&str> = result.split(',').collect();
    assert_eq!(fields.len(), 4, "H4 result shape: {result}");
    assert_eq!(fields[0], fields[1], "H4 stored length drifted: {result}");
    assert_eq!(
        fields[2], "true",
        "H4 module-var if-position compare regressed: {result}"
    );
    assert_eq!(
        fields[3], "true",
        "H4 module-var expr-position compare regressed: {result}"
    );
}

/// H5（T-00 新发现，T-02 修复的 red→green 例）：`json.stringify` 少参
/// ABI——1 参调用必须给出与 3 参一致的 compact 结果（JS 语义：replacer/
/// space 缺省 undefined→紧凑输出）。修复前：1 参弹栈下溢（panic）或垃圾
/// ""，2 参垃圾 "0"（h6-mini service probe 实证）。
#[test]
fn h5_stringify_arity_abi() {
    let code = r#"
let d = json.parse("{\"a\":1,\"b\":\"x\",\"c\":[true,null],\"d\":{\"e\":1.5}}")
let s1 str = json.stringify(d)
let s3 str = json.stringify(d, "", 0)
if s1 == s3 { 1 } else { 0 }
"#;
    let result = crate::run(code).expect("run failed");
    assert_eq!(
        result, "1",
        "H5 json.stringify arity ABI broken: 1-arg != 3-arg compact output"
    );
}

/// W02 等价 oracle 预置（T-02 使用）：parse→stringify 往返逐字节等价。
/// 转义面取 serde 往返对称子集（字面 UTF-8、\"、\\、\n、\t、i64 域大整数、
/// 嵌套、空容器）；`\uXXXX` 转义非对称（serde 解码后按字面回发）不入 oracle。
#[test]
fn w02_json_roundtrip_byte_equality() {
    let code = r#"
let doc str = "{\"s\":\"中文 😀\",\"esc\":\"a\\\"b\\\\c\\nd\\te\",\"big\":9223372036854775807,\"neg\":-42,\"f\":1.5,\"arr\":[],\"obj\":{},\"nested\":{\"deep\":[1,[2,[3,null]]]},\"t\":true,\"n\":null}"
let d = json.parse(doc)
let s str = json.stringify(d, "", 0)
if s == doc { 1 } else { 0 }
"#;
    let result = crate::run(code).expect("run failed");
    assert_eq!(
        result, "1",
        "W02 roundtrip byte-equality failed (stringify(parse(x)) != x on canonical text)"
    );
}
