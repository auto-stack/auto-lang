//! PLAN-945 冻结公共标准库包 `std` 的源身份与具名未支持面。包名 `std`
//! 保留给本仓 `stdlib/ace/std/` 的冻结切片：非逐字节同源的 `std` 输入在
//! 绑定前拒绝（防用户包遮蔽冒充公共绑定）；`std.math` 上未导出的 i64
//! 公共族 abs/min/max 以 `ace.stdlib-unwired` 具名拒绝（先于调用，无窄化
//! 适配），区别于普通 `source.module-symbol` 拼写错误。
use super::binding::PackageModuleInput;

pub const STDLIB_PACKAGE: &str = "std";
pub const STDLIB_VERSION: &str = "1.0.0";
pub const STDLIB_UNWIRED: &str = "ace.stdlib-unwired";
pub const STDLIB_RESERVED: &str = "ace.stdlib-reserved";

const MATH_SOURCE: &str = include_str!("../../../../../../../stdlib/ace/std/math.at");
const TEXT_SOURCE: &str = include_str!("../../../../../../../stdlib/ace/std/text.at");
const LIST_SOURCE: &str = include_str!("../../../../../../../stdlib/ace/std/list.at");

/// 冻结 stdlib 输入（math/text 两模块，plain packages 面的原契约——
/// PLAN-946 加入 `std.list` 后本函数**不变**，math/text 消费者逐字节稳定）。
/// 身份 = 仓库文件内容本身。
pub fn stdlib_inputs() -> Vec<PackageModuleInput> {
    vec![
        PackageModuleInput {
            package: STDLIB_PACKAGE.into(),
            version: STDLIB_VERSION.into(),
            id: "math".into(),
            source: MATH_SOURCE.into(),
            provider: None,
        },
        PackageModuleInput {
            package: STDLIB_PACKAGE.into(),
            version: STDLIB_VERSION.into(),
            id: "text".into(),
            source: TEXT_SOURCE.into(),
            provider: None,
        },
    ]
}

/// PLAN-946：aggregates packages 线的冻结 stdlib 输入（仅 `std.list`——
/// List 参数公共函数的承载面；math/text 属 TextRecord 域，留在 plain 线，
/// 混域图在 aggregates 线具名拒）。身份同 = 仓库文件内容。
pub fn stdlib_aggregates_inputs() -> Vec<PackageModuleInput> {
    vec![PackageModuleInput {
        package: STDLIB_PACKAGE.into(),
        version: STDLIB_VERSION.into(),
        id: "list".into(),
        source: LIST_SOURCE.into(),
        provider: None,
    }]
}

/// `std` 包身份保留：模块 id、版本与源文本必须与冻结切片一致。
pub fn is_frozen_std_module(id: &str, version: &str, source: &str) -> bool {
    version == STDLIB_VERSION
        && (matches!(id, "math" if source == MATH_SOURCE)
            || matches!(id, "text" if source == TEXT_SOURCE)
            || matches!(id, "list" if source == LIST_SOURCE))
}

/// `std.math` 上具名未支持的 i64 公共族（abs/min/max）；其余未命中仍走
/// 普通 `source.module-symbol`。
pub fn unwired_reason(unit_key: &str, name: &str) -> Option<String> {
    if unit_key != "std::math" || !matches!(name, "abs" | "min" | "max") {
        return None;
    }
    Some(format!(
        "math.{name} is the frozen i64 public API; the ACE i32 profile has no narrowing adapter (PLAN-945)"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_inputs_match_repo_files_and_identity_is_stable() {
        let inputs = stdlib_inputs();
        assert_eq!(inputs.len(), 2);
        let agg = stdlib_aggregates_inputs();
        assert_eq!(agg.len(), 1);
        assert_eq!(agg[0].id, "list");
        assert!(agg.iter().all(|input| is_frozen_std_module(
            &input.id,
            &input.version,
            &input.source
        )));
        for input in &inputs {
            assert_eq!(input.package, "std");
            assert_eq!(input.version, "1.0.0");
            assert!(is_frozen_std_module(
                &input.id,
                &input.version,
                &input.source
            ));
        }
        // a tampered std module is never the frozen identity
        assert!(!is_frozen_std_module(
            "math",
            "1.0.0",
            "pub fn square(x int) int { return x * x + 1 }"
        ));
        assert!(!is_frozen_std_module(
            "list",
            "1.0.0",
            "pub fn sum(xs List<int>) int { return 1 }"
        ));
        assert!(!is_frozen_std_module("text", "9.9.9", TEXT_SOURCE));
    }

    #[test]
    fn unwired_reason_names_exactly_the_i64_family() {
        for name in ["abs", "min", "max"] {
            let reason = unwired_reason("std::math", name).unwrap();
            assert!(reason.contains("i64"), "{reason}");
            assert!(reason.contains(name), "{reason}");
        }
        // everything else — typos, other modules, delivered symbols — is not
        // the named-unwired family and keeps the generic rejection path.
        assert!(unwired_reason("std::math", "square").is_none());
        assert!(unwired_reason("std::text", "min").is_none());
        assert!(unwired_reason("app::math", "min").is_none());
    }
}
