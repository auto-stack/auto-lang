//! Plan 566 T1: Value 装箱瘦身的尺寸审计表 + 防回退断言。
//!
//! 审计表（`cargo test -p auto-val size -- --nocapture`）打印各胖变体
//! 载荷的 `size_of` 排名，佐证装箱阶段顺序（Node 唯一鲸鱼 → Instance +
//! 单点胖变体 → 可选 Fn/ExtFn/Type）；`VALUE_SIZE_LIMIT` 随各阶段收紧
//! （232 → 96 → 48 → [32]），编译期 const 断言 + 运行期测试双锁。

use std::mem::size_of;

use crate::array::Array;
use crate::cstr::CStr;
use crate::kids::Kids;
use crate::meta::{Args, ExtFn, Fn, MetaID};
use crate::node::{Instance, Node};
use crate::obj::Obj;
use crate::owned_str::Str;
use crate::pair::ValueKey;
use crate::str_slice::StrSlice;
use crate::types::Type;
use crate::value::{Closure, FutureData, Grid, Method, Model, Value, View, Widget};
use crate::AutoStr;

/// 防回退上限（B）。Phase A（Node 装箱）落 112；Phase B 落 72；
/// Phase C（Fn/ExtFn/Type 装箱）终态实测 40——G1 ≤48 锁死（Str 40B 为
/// 地板设定者，热路径字符串不装箱）。
const VALUE_SIZE_LIMIT: usize = 48;

const _: () = assert!(
    size_of::<Value>() <= VALUE_SIZE_LIMIT,
    "Value 装箱防回退: size_of 超过本阶段上限（见 Plan 566）"
);

/// 运行期镜像断言（报错信息带实测值，便于定位回退提交）。
#[test]
fn value_size_floor() {
    assert!(
        size_of::<Value>() <= VALUE_SIZE_LIMIT,
        "size_of::<Value>() = {} > {VALUE_SIZE_LIMIT}（Plan 566 装箱回退?）",
        size_of::<Value>()
    );
}

/// 各变体载荷 size_of 排名表——确认 Node 为鲸鱼、装箱阶段顺序正确。
#[test]
fn size_table() {
    let mut rows: Vec<(&str, usize)> = vec![
        ("Node", size_of::<Node>()),
        ("Instance", size_of::<Instance>()),
        ("Obj", size_of::<Obj>()),
        ("Kids", size_of::<Kids>()),
        ("Grid", size_of::<Grid>()),
        ("Widget", size_of::<Widget>()),
        ("Closure", size_of::<Closure>()),
        ("FutureData", size_of::<FutureData>()),
        ("Method", size_of::<Method>()),
        ("Args", size_of::<Args>()),
        ("View", size_of::<View>()),
        ("Model", size_of::<Model>()),
        ("Fn", size_of::<Fn>()),
        ("ExtFn", size_of::<ExtFn>()),
        ("Type", size_of::<Type>()),
        ("MetaID", size_of::<MetaID>()),
        ("Array", size_of::<Array>()),
        ("Str", size_of::<Str>()),
        ("StrSlice", size_of::<StrSlice>()),
        ("CStr", size_of::<CStr>()),
        ("ValueKey", size_of::<ValueKey>()),
        ("AutoStr", size_of::<AutoStr>()),
    ];
    rows.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    println!("size_of::<Value>() = {} B", size_of::<Value>());
    println!("---- 载荷 size_of 排名（降序）----");
    for (name, sz) in &rows {
        println!("{sz:>4} B  {name}");
    }
}
