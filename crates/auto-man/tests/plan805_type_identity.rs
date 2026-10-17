//! PLAN-805 V02 — 旧公开路径与新核心 crate 的类型**同一性**证明。
//!
//! `builder::ninja::config/mapper/templates` 自抽取后是
//! `auto-toolchain-core` 的薄 re-export：下面全部按旧路径声明的类型/值
//! 直接传给新 crate 的函数（以及反向），若存在任何重复 enum/struct/
//! mapper/render 副本，这里将编译失败（E0308 类型不匹配）而非测试红。

use auto_man::builder::ninja::config::{
    CompilerConfig, CompilerKind, CompilerLocation, ExecutableType, FlagFormat, FlagMappings,
};
use auto_man::builder::ninja::mapper::FlagMapper;
use auto_man::builder::ninja::templates::CommandTemplates;
use std::collections::HashMap;

#[test]
fn old_public_paths_are_auto_toolchain_core_types() {
    // 旧路径构造 → 新 crate 函数直接消费（无转换/无副本）
    let gcc: CompilerConfig = CompilerConfig::gcc_default();
    assert_eq!(
        auto_toolchain_core::FlagMapper::map_compile_only(&gcc),
        "-c"
    );
    assert_eq!(
        auto_toolchain_core::FlagMapper::map_include(&gcc, &auto_val::AutoStr::from("inc")),
        "-Iinc"
    );

    // 新 crate 构造 → 旧路径函数直接消费
    let msvc: auto_toolchain_core::CompilerConfig =
        auto_toolchain_core::CompilerConfig::msvc_default();
    assert_eq!(FlagMapper::map_compile_only(&msvc), "/c");
    let templates = CommandTemplates::new(&msvc);
    assert!(templates.compile.contains("/Fo{output}"));

    // enum 同一性：跨 crate match（不同类型无法进同一 match）
    let kinds: Vec<CompilerKind> = vec![
        CompilerKind::MSVC,
        CompilerKind::GCC,
        CompilerKind::Clang,
        CompilerKind::GHS,
        CompilerKind::IAR,
        CompilerKind::Targeting,
        CompilerKind::Hightec,
        CompilerKind::TICLang,
    ];
    let core_kinds: Vec<auto_toolchain_core::CompilerKind> = kinds.clone();
    assert_eq!(kinds, core_kinds, "枚举跨路径相等 = 同一类型");

    // struct 字段级同一性：旧路径类型的字段就是新 crate 类型
    let _loc: &CompilerLocation = &gcc.location;
    let _flags: &FlagMappings = &gcc.flags;
    let _ff: &FlagFormat = &gcc.flags.include;
    let _exec: Option<&auto_val::AutoPath> = gcc.get_executable(ExecutableType::Compiler);

    // HashMap 键值同一性（ExecutableType 跨 crate 作键）
    let mut execs: HashMap<auto_toolchain_core::ExecutableType, auto_val::AutoPath> =
        HashMap::new();
    execs.insert(ExecutableType::Compiler, "cc.exe".into());
    let rebuilt: CompilerConfig = CompilerConfig::new(
        CompilerKind::GCC,
        "x".into(),
        execs,
        auto_toolchain_core::FlagMappings::gcc_default(),
        CompilerLocation::Env,
        vec![],
    );
    assert_eq!(
        rebuilt.get_executable(auto_toolchain_core::ExecutableType::Compiler),
        Some(&"cc.exe".into())
    );
}
