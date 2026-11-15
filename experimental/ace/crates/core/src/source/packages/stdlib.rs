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

// ---------------------------------------------------------------------------
// PLAN-972（T-02）：装配标准库集（assembled std set）——packages-aggregates-ct
// 装配线的 `std` 冻结身份。与上方 plain/aggregates 线的冻结切片并存：
// 同包名、同模块 id 可以有两条各自冻结的仓内身份（lane 分离，绑定面互不
// 混装）；仓外来源两条线都不认。身份 = 仓库文件内容本身（include_str!）。
// ---------------------------------------------------------------------------

// PLAN-973（T-02）：系统库装配模块（file/path/env/process/time/rand/storage
// + syscommon + 双 OS 平台模块）。同 972 集：纯 Auto、无 provider 候选；
// #in 在公共模块内按装配目标选择平台模块。
const SYS_SYSCOMMON: &str = include_str!("../../../../../../../stdlib/ace/sys/syscommon.at");
const SYS_SYSPLAT_WIN: &str = include_str!("../../../../../../../stdlib/ace/sys/sysplat_win.at");
const SYS_SYSPLAT_LNX: &str = include_str!("../../../../../../../stdlib/ace/sys/sysplat_lnx.at");
const SYS_PATH: &str = include_str!("../../../../../../../stdlib/ace/sys/path.at");
const SYS_FILE: &str = include_str!("../../../../../../../stdlib/ace/sys/file.at");
const SYS_ENV: &str = include_str!("../../../../../../../stdlib/ace/sys/env.at");
const SYS_PROCESS: &str = include_str!("../../../../../../../stdlib/ace/sys/process.at");
const SYS_TIME: &str = include_str!("../../../../../../../stdlib/ace/sys/time.at");
const SYS_RAND: &str = include_str!("../../../../../../../stdlib/ace/sys/rand.at");
const SYS_STORAGE: &str = include_str!("../../../../../../../stdlib/ace/sys/storage.at");

const ASM_LIST_PUBLIC: &str = include_str!("../../../../../../../stdlib/ace/asm/list.at");
const ASM_MATH_PUBLIC: &str = include_str!("../../../../../../../stdlib/ace/asm/math.at");
const ASM_TEXT_PUBLIC: &str = include_str!("../../../../../../../stdlib/ace/asm/text.at");
const ASM_CHAR_PUBLIC: &str = include_str!("../../../../../../../stdlib/ace/asm/char.at");
const ASM_CONV_PUBLIC: &str = include_str!("../../../../../../../stdlib/ace/asm/conv.at");
const ASM_HASH_PUBLIC: &str = include_str!("../../../../../../../stdlib/ace/asm/hash.at");
const ASM_BYTES_PUBLIC: &str = include_str!("../../../../../../../stdlib/ace/asm/bytes.at");
const ASM_MAP_PUBLIC: &str = include_str!("../../../../../../../stdlib/ace/asm/map.at");

/// 装配标准库集的平铺文件闭包（discover/assemble 消费；按名排序由调用方
/// 的 assemble 逻辑负责）。纯 Auto 模块无来源候选（provider 选择面允许
/// 无候选——`std` 模块不接受仓外实现层）。
pub fn stdlib_assembled_files() -> Vec<(String, String)> {
    vec![
        ("list.at".to_string(), ASM_LIST_PUBLIC.to_string()),
        ("math.at".to_string(), ASM_MATH_PUBLIC.to_string()),
        ("text.at".to_string(), ASM_TEXT_PUBLIC.to_string()),
        ("char.at".to_string(), ASM_CHAR_PUBLIC.to_string()),
        ("conv.at".to_string(), ASM_CONV_PUBLIC.to_string()),
        ("hash.at".to_string(), ASM_HASH_PUBLIC.to_string()),
        ("bytes.at".to_string(), ASM_BYTES_PUBLIC.to_string()),
        ("map.at".to_string(), ASM_MAP_PUBLIC.to_string()),
    ]
}

/// PLAN-973（T-02）：系统库装配集（file/path/env/process/time/rand/storage
/// + syscommon + 双 OS 平台模块）——与 972 纯算法集**分离**的第二个冻结
/// 装配集：系统库的消费者（std 系统入口）显式并入本集；普通 972 集消费
/// 者不受系统模块的平台声明/依赖影响。同 972 纪律：纯 Auto、无 provider
/// 候选、#in 在公共模块内按装配目标选择平台模块；身份 = 仓库文件内容。
pub fn stdlib_sys_files() -> Vec<(String, String)> {
    vec![
        ("syscommon.at".to_string(), SYS_SYSCOMMON.to_string()),
        ("sysplat_win.at".to_string(), SYS_SYSPLAT_WIN.to_string()),
        ("sysplat_lnx.at".to_string(), SYS_SYSPLAT_LNX.to_string()),
        ("path.at".to_string(), SYS_PATH.to_string()),
        ("file.at".to_string(), SYS_FILE.to_string()),
        ("env.at".to_string(), SYS_ENV.to_string()),
        ("process.at".to_string(), SYS_PROCESS.to_string()),
        ("time.at".to_string(), SYS_TIME.to_string()),
        ("rand.at".to_string(), SYS_RAND.to_string()),
        ("storage.at".to_string(), SYS_STORAGE.to_string()),
    ]
}

/// 系统库装配集的 `std` 身份保留（与 972 集同一防遮蔽纪律）。
pub fn is_frozen_std_sys_module(
    id: &str,
    version: &str,
    source: &str,
) -> bool {
    version == STDLIB_VERSION
        && (matches!(id, "syscommon" if source == SYS_SYSCOMMON)
            || matches!(id, "sysplat_win" if source == SYS_SYSPLAT_WIN)
            || matches!(id, "sysplat_lnx" if source == SYS_SYSPLAT_LNX)
            || matches!(id, "path" if source == SYS_PATH)
            || matches!(id, "file" if source == SYS_FILE)
            || matches!(id, "env" if source == SYS_ENV)
            || matches!(id, "process" if source == SYS_PROCESS)
            || matches!(id, "time" if source == SYS_TIME)
            || matches!(id, "rand" if source == SYS_RAND)
            || matches!(id, "storage" if source == SYS_STORAGE))
}

/// PLAN-972（T-02）：装配线 `std` 身份保留。模块 id/版本/公共源（以及实现
/// 层 kind+源，若有）必须与本仓装配标准库集逐字节一致；任何仓外/漂移输入
/// 在绑定前拒绝（与 plain 冻结切片同一防遮蔽纪律）。
pub fn is_frozen_std_assembled_module(
    id: &str,
    version: &str,
    source: &str,
    layer: Option<(&super::provider_sources::ProviderKind, &str)>,
) -> bool {
    if version != STDLIB_VERSION {
        return false;
    }
    // 现集全部为纯 Auto 模块：不接受任何实现层（候选文件存在于仓内集时
    // 在此按 kind+源核对；当前无候选）。
    if layer.is_some() {
        return false;
    }
    matches!(id, "list" if source == ASM_LIST_PUBLIC)
        || matches!(id, "math" if source == ASM_MATH_PUBLIC)
        || matches!(id, "text" if source == ASM_TEXT_PUBLIC)
        || matches!(id, "char" if source == ASM_CHAR_PUBLIC)
        || matches!(id, "conv" if source == ASM_CONV_PUBLIC)
        || matches!(id, "hash" if source == ASM_HASH_PUBLIC)
        || matches!(id, "bytes" if source == ASM_BYTES_PUBLIC)
        || matches!(id, "map" if source == ASM_MAP_PUBLIC)
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
