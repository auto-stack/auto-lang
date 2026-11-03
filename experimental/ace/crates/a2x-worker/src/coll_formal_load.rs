//! PLAN-926 T-10: the FROZEN representative collections load shared by
//! the 904 (Rust) and 926 (C) formal samplers — the 904 build_load
//! VERBATIM (40 real functions: 2 producers + 32 unchanged callers +
//! 16 data helpers + 5 disjoint scalar helpers + main; >= 16KiB
//! non-comment source; >= 4KiB live string data) plus its hand-computed
//! closed-form oracle. Extracted from the 904 sampler bin unchanged so
//! both channels sample the SAME load bytes (the plan's
//! "保904完整build_load（含data helper，不按注释删到40函数）").

/// The frozen representative load (PLAN-904 §5.7): one nominal record +
/// one enum family, `List<Str>` / owned-record / variant elements, 40+
/// real functions and a variable-length body-edit target.
pub fn build_load(left: &str) -> String {
    let mut src = String::new();
    src.push_str("fn left_pad() int { return ");
    src.push_str(left);
    src.push_str(" }\n");
    src.push_str("fn right_pad() int { return 22 }\n");
    src.push_str("type Item { name Str; code int }\n");
    src.push_str("enum Choice { Data(Item)\nEnd }\n");
    src.push_str("fn mk_str_a() Str { return \"aaaa-bbbb-cccc-dddd-eeee\" }\n");
    src.push_str("fn mk_str_b() Str { return \"中文字符串-数据-载荷-二\" }\n");
    src.push_str("fn mk_item(code int) Item { return Item(mk_str_a(), code) }\n");
    src.push_str("fn mk_choice(code int) Choice { return Choice.Data(mk_item(code)) }\n");
    // a >=4KiB block of DISTINCT live string data (referenced by the
    // callers; the load carries the data, not comments)
    for b in 0..16 {
        let unit = "数据-载荷-字符串-区块-abcdefgh-";
        let lit: String = std::iter::repeat(unit).take(10).collect();
        src.push_str(&format!(
            "fn data_block_{b}() Str {{ return \"{lit}-{b}\" }}\n"
        ));
    }
    // 32 unchanged callers over the collections surface (owned pushes go
    // through whole-local `.move` or fresh constructions — the C3 gate)
    for i in 0..32 {
        src.push_str(&format!(
            "fn caller_{i}(v int) int {{\n    var xs = List<Item>.new()\n    let it0 Item = mk_item(v + {i})\n    xs.push(it0.move)\n    xs.push(Item(mk_str_b(), {i}))\n    let it1 Item = mk_item(v + {i} + 1)\n    xs.push(it1.move)\n    xs.set(2, Item(mk_str_a(), {i}))\n    var zs = List<Str>.new()\n    let sa Str = mk_str_a()\n    zs.push(sa.move)\n    let sb Str = mk_str_b()\n    zs.push(sb.move)\n    let names = zs.get(0).len() + zs.get(1).len()\n    let dropped Str = zs.remove(1)\n    let total = xs.get(0).name.len() * 10 + xs.get(1).code + names + dropped.len()\n    var ys = List<Choice>.new()\n    let c0 Choice = mk_choice(total)\n    ys.push(c0.move)\n    let t = ys.remove(0)\n    is t {{\n        Choice.Data(item) -> {{ return item.code + left_pad() + right_pad() }}\n        Choice.End -> {{ return 0 }}\n    }}\n}}\n"
        ));
    }
    // 5 disjoint scalar helpers (unchanged across revisions)
    for i in 0..5 {
        src.push_str(&format!(
            "fn scalar_{i}(v int) int {{ return v * {} + {i} }}\n",
            i + 2
        ));
    }
    // main fans the callers out and folds with the disjoint helpers
    src.push_str("fn main() int {\n    var total = 0\n");
    for i in 0..32 {
        src.push_str(&format!("    total = total + caller_{i}({i})\n"));
    }
    for i in 0..5 {
        src.push_str(&format!("    total = total + scalar_{i}(total)\n"));
    }
    src.push_str("    return total\n}\n");
    src
}

/// The hand-computed closed-form oracle of `build_load(left_v)` (the
/// independent expectation both channels certify against).
pub fn oracle_of(left_v: i64) -> i64 {
    // left_pad returns the given value. Hand-computed per caller:
    // xs[0].name.len()=24 (mk_str_a) *10 = 240; xs[1].code=i; names =
    // 24 + 13 (mk_str_b = 13 Unicode scalars) = 37; dropped.len() = 13;
    // so item.code = 240+i+37+13 = 290+i, and the caller returns
    // item.code + left + 22 = 312 + left + i.
    let mut total: i64 = 0;
    for i in 0..32i64 {
        total += 312 + left_v + i;
    }
    // main folds total = total + scalar_i(total), i.e. the recurrence
    // t' = t + (t*(i+2)+i) = t*(i+3)+i, applied for i = 0..=4
    for i in 0..5i64 {
        total = total * (i + 3) + i;
    }
    total
}

/// Byte offsets of `name`'s single-line declaration in `src`.
pub fn fn_line_range(src: &str, name: &str) -> (usize, usize) {
    let needle = format!("fn {name}(");
    let start = src.find(&needle).expect("fn");
    let end = src[start..]
        .find('\n')
        .map(|i| start + i)
        .unwrap_or(src.len());
    (start, end)
}
