//! PLAN-1117 T-02（V-01 发射侧钉子）：候选路由 = 普通语言通道。
//!
//! `transpile_scalar_source` 是兄弟 CLI（PLAN-982 的 `AUTO_ACE_RUST=ace`）注册
//! 的渲染器名；PLAN-1117 起它改走 PLAN-1115/1116 的普通语言通道。本文件钉住：
//! 1. 老项目形状（record/list/闭包/嵌套 place 写）真实发射，通道身份是
//!    `auto.a2x.rust.language.fileset/1`（而不是标量 `auto.a2x.rust.fileset/1`
//!    ——路由不可能来自旧 emitter 或标量通道）；
//! 2. 泛型实例化与 `#{...}` CT 准备钩子随路由生效（生成文本无 `<T>` 残留、
//!    CT 常量被冻结进产物）；
//! 3. 同输入逐字节确定。
//! 真编译/真运行与仓外消费者由 m2-build 的 A2R 语言车道用例与 PLAN-1117 §7
//! 的显式 V-01 证据承担（本文件不 spawn rustc）。

use auto_lang_ace_host::rust_transpile::transpile_scalar_source;

/// 老项目形状（1115 v01-e 语形的单文件版 + 文本长度）：record 构造、List 嵌套
/// place 写、闭包实参、owned 文本——候选路由必须原样吃下并给出语言通道产物。
/// （`tag` 是保留字（enum/union/tag 语法族），变量名用 `label`。）
const OLD_PROJECT: &str = "fn each_plus(f fn(int)int, xs List<int>) int {\n\
     var n = 0\n\
     for x in xs {\n\
         let y = f(x)\n\
         n = n + y\n\
     }\n\
     return n\n\
 }\n\
 type Row { xs List<int> }\n\
 fn main() int {\n\
     var r = Row([19, 20])\n\
     r.xs[1] += 3\n\
     let label = \"auto\".clone()\n\
     return each_plus((x int) => x * 1, r.xs) + label.len()\n\
 }";

#[test]
fn p1117_candidate_route_rides_the_language_channel() {
    let fileset = transpile_scalar_source(OLD_PROJECT, "p1117-old", "main").expect("rendered");
    assert_eq!(
        fileset
            .files
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        vec!["main.rs", "a2x_runtime.rs", "generated.rs"]
    );
    assert_eq!(fileset.entry, "fn_main");
    assert!(
        fileset
            .manifest
            .contains("auto.a2x.rust.language.fileset/1"),
        "the language channel's own fileset schema: {}",
        fileset.manifest
    );
    // 生成文本头携带语言通道身份标签（`PLAN-1115 IF-ACE-A2R-LANGUAGE/1`）——
    // 产物只可能来自语言通道，不可能来自旧 emitter 或标量通道。
    assert!(
        fileset.files[2]
            .text
            .contains("PLAN-1115 IF-ACE-A2R-LANGUAGE/1"),
        "generated.rs header carries the language channel identity"
    );
    assert!(
        !fileset.files[2].text.contains("<T>"),
        "no un-instantiated generics ride the emitted text"
    );
}

#[test]
fn p1117_candidate_route_is_deterministic() {
    let a = transpile_scalar_source(OLD_PROJECT, "p1117-old", "main").expect("first");
    let b = transpile_scalar_source(OLD_PROJECT, "p1117-old", "main").expect("second");
    assert_eq!(a, b, "two renders must be byte-identical");
}

/// 泛型实例化（共享 pass）与 `#{...}` CT（单门）随候选路由生效：
/// `#{ sum(20, 22) }` 在 bind 期冻结为常量 42，`fn id<T>` 被实例化为具体函数。
#[test]
fn p1117_generic_and_ct_hooks_ride_the_route() {
    let src = "fn id<T>(x T) T { return x }\n\
         fn sum(a int, b int) int { return a + b }\n\
         fn main() int {\n\
             let n = #{ sum(20, 22) }\n\
             if id(true) { return n }\n\
             return 0\n\
         }";
    let fileset = transpile_scalar_source(src, "p1117-ct", "main").expect("rendered");
    let generated = &fileset.files[2].text;
    assert!(
        !generated.contains("<T>"),
        "the shared instance pass concretized the generic"
    );
    assert!(
        !generated.contains("#{"),
        "no CT demand survives into the emitted program"
    );
    assert!(
        fileset.manifest.contains("auto.a2x.rust.language.fileset/1"),
        "language channel schema: {}",
        fileset.manifest
    );
}

/// 前端拒绝仍具名（`ace.rust/compile`），入口缺失仍 `ace.rust/lower`——
/// 语言通道没有放宽"绝不静默"的合同。
#[test]
fn p1117_refusals_stay_named() {
    let err = transpile_scalar_source("fn main() int { this is not auto }", "p1117", "main")
        .expect_err("frontend refuses");
    assert_eq!(err.code, "ace.rust/compile", "{err}");
    let err = transpile_scalar_source(OLD_PROJECT, "p1117-old", "no_such_fn")
        .expect_err("entry refuses");
    assert_eq!(err.code, "ace.rust/lower", "{err}");
}
