//! PLAN-805 V01 — an EXTERNAL consumer using only `auto-toolchain-core`.
//!
//! Everything here must hold without auto-lang/auto-man/auto-gen/auto-ui/
//! auto-vm/auto-ast anywhere in the build (enforced structurally by this
//! crate's manifest and by `verify-deps.sh`; auto-val is the rules model's
//! legal data layer). It proves an external, language-free consumer can
//! construct the full toolchain model, map generic flags and render all four
//! command templates — pure in-memory rules only, no PATH/disk/process
//! observation (those stay in the auto-man adapter layer).

use auto_toolchain_core::{
    CommandTemplates, CompilerConfig, CompilerKind, CompilerLocation, ExecutableType, FlagFormat,
    FlagMapper, FlagMappings,
};
use std::collections::HashMap;

fn all_defaults() -> Vec<(&'static str, CompilerConfig)> {
    vec![
        ("msvc", CompilerConfig::msvc_default()),
        ("gcc", CompilerConfig::gcc_default()),
        ("clang", CompilerConfig::clang_default()),
        ("ghs", CompilerConfig::ghs_default()),
        ("iar", CompilerConfig::iar_default()),
        ("targeting", CompilerConfig::targeting_default()),
        ("hightec", CompilerConfig::hightec_default()),
        ("ticlang", CompilerConfig::ticlang_default()),
    ]
}

/// 外部 consumer 可构造八类完整配置并读取全部字段（不是闲置 DTO）。
#[test]
fn external_consumer_builds_all_eight_models() {
    let defaults = all_defaults();
    assert_eq!(defaults.len(), 8);

    for (name, cfg) in &defaults {
        assert_eq!(&cfg.name.as_str(), name, "default name matches kind key");
        // 四角色齐备
        for role in [
            ExecutableType::Compiler,
            ExecutableType::Archiver,
            ExecutableType::Linker,
            ExecutableType::Assembler,
        ] {
            assert!(
                cfg.get_executable(role).is_some(),
                "{} missing executable {:?}",
                name,
                role
            );
        }
        // 默认 cflags 非空 + 位置默认 Env
        assert!(!cfg.default_cflags.is_empty(), "{} cflags", name);
        assert_eq!(cfg.location, CompilerLocation::Env);
    }

    // 关键默认值抽查（与旧臂冻结值一致）
    let msvc = CompilerConfig::msvc_default();
    assert_eq!(msvc.get_object_extension(), ".obj");
    assert_eq!(msvc.get_executable_extension(), ".exe");
    assert_eq!(
        msvc.get_executable(ExecutableType::Assembler),
        Some(&"ml64.exe".into())
    );
    assert_eq!(
        msvc.default_cflags
            .iter()
            .map(|f| f.as_str())
            .collect::<Vec<_>>(),
        vec!["/O2", "/W3", "/MD"]
    );

    let ticlang = CompilerConfig::ticlang_default();
    assert_eq!(ticlang.get_object_extension(), ".o");
    assert_eq!(ticlang.get_executable_extension(), "");
    assert_eq!(
        ticlang
            .default_cflags
            .iter()
            .map(|f| f.as_str())
            .collect::<Vec<_>>(),
        vec!["-Oz", "-Wall"]
    );

    let iar = CompilerConfig::iar_default();
    assert_eq!(iar.default_cflags.len(), 12);
    assert_eq!(
        FlagMapper::map_compile_only(&iar),
        "--debug",
        "IAR 独有 compile_only 映射"
    );
}

/// 别名/大小写/未知 kind 与四角色解析（外部可复用同一判定规则）。
#[test]
fn external_consumer_parses_kinds_and_roles() {
    assert_eq!(CompilerKind::from_str("gnu"), Some(CompilerKind::GCC));
    assert_eq!(CompilerKind::from_str("LLVM"), Some(CompilerKind::Clang));
    assert_eq!(
        CompilerKind::from_str("TICLang"),
        Some(CompilerKind::TICLang)
    );
    assert_eq!(CompilerKind::from_str("nope"), None);
    assert_eq!(CompilerKind::GCC.to_string(), "gcc");

    assert_eq!(
        ExecutableType::from_str("compiler"),
        Some(ExecutableType::Compiler)
    );
    assert_eq!(
        ExecutableType::from_str("linker"),
        Some(ExecutableType::Linker)
    );
    assert_eq!(ExecutableType::from_str("CC"), None, "大小写敏感");
}

/// flag 映射 + 三种 FlagFormat + 批量/格式化 + 空集合。
#[test]
fn external_consumer_maps_flags() {
    let gcc = CompilerConfig::gcc_default();
    assert_eq!(FlagMapper::map_include(&gcc, &"inc 1".into()), "-Iinc 1");
    assert_eq!(FlagMapper::map_define(&gcc, &"DEBUG=1".into()), "-DDEBUG=1");
    assert_eq!(FlagMapper::map_library(&gcc, &"mylib".into()), "-lmylib");
    assert_eq!(
        FlagMapper::map_library_path(&gcc, &"D:/a b".into()),
        "-LD:/a b"
    );
    assert_eq!(
        FlagMapper::map_output(&gcc, &"main.o".into(), ".o"),
        "-omain",
        "Prefix 型剥对象尾缀"
    );
    assert_eq!(FlagMapper::map_compile_only(&gcc), "-c");
    assert_eq!(FlagMapper::format_includes(&gcc, &[]), "");
    assert_eq!(
        FlagMapper::format_defines(&gcc, &["A".into(), "B=2".into()]),
        "-DA -DB=2"
    );

    let msvc = CompilerConfig::msvc_default();
    assert_eq!(
        FlagMapper::map_output(&msvc, &"main.obj".into(), ".obj"),
        "/Fomain.obj",
        "Both 型前缀+剥尾缀+后缀"
    );

    // 三种 FlagFormat 直接应用（含 Unicode/空格值）
    assert_eq!(
        FlagFormat::Prefix("-I{}".to_string()).apply("路径/中文 目录"),
        "-I路径/中文 目录"
    );
    assert_eq!(
        FlagFormat::Postfix("{}.lib".to_string()).apply("my lib"),
        "my lib.lib"
    );
    assert_eq!(
        FlagFormat::Both("/Fo".to_string(), ".obj".to_string()).apply("out"),
        "/Foout.obj"
    );
}

/// 外部 consumer 可用 CompilerConfig::new 自定义完整配置并渲染四命令。
#[test]
fn external_consumer_renders_all_four_commands() {
    // 八类模板全部可构造
    for (_, cfg) in all_defaults() {
        let t = CommandTemplates::new(&cfg);
        assert!(t.compile.contains("{cc}"));
        assert!(t.assemble.contains("{as}"));
        assert!(t.link.contains("{link}"));
        assert!(t.archive.contains("{ar}"));
    }

    // TICLang 复用 clang 模板（旧臂冻结行为）
    let ticlang = CommandTemplates::new(&CompilerConfig::ticlang_default());
    let clang = CommandTemplates::new(&CompilerConfig::clang_default());
    assert_eq!(ticlang.compile, clang.compile);
    assert_eq!(ticlang.archive, clang.archive);

    // 自定义配置：CompilerConfig::new + 自定义 FlagMappings
    let custom = CompilerConfig::new(
        CompilerKind::GCC,
        "mygcc".into(),
        HashMap::from([
            (ExecutableType::Compiler, "cc.exe".into()),
            (ExecutableType::Archiver, "ar.exe".into()),
        ]),
        FlagMappings {
            include: FlagFormat::Prefix("--inc={}".to_string()),
            define: FlagFormat::Prefix("--def={}".to_string()),
            library: FlagFormat::Postfix("lib{}.a".to_string()),
            library_path: FlagFormat::Prefix("--libdir={}".to_string()),
            output: FlagFormat::Prefix("--out={}".to_string()),
            compile_only: FlagFormat::Prefix("--compile".to_string()),
        },
        CompilerLocation::Dir("C:/my tools".into()),
        vec!["-O2".into(), "-g".into()],
    );
    assert_eq!(FlagMapper::map_include(&custom, &"inc".into()), "--inc=inc");
    assert_eq!(
        FlagMapper::map_library(&custom, &"core".into()),
        "libcore.a"
    );
    assert_eq!(FlagMapper::map_compile_only(&custom), "--compile");
    assert_eq!(custom.get_executable(ExecutableType::Linker), None);

    // 四命令渲染 + 缺变量占位符保留 + render 低层入口。
    // 注意：命令模板按 kind 表生成（GCC → "-o{output}"），不消费自定义
    // FlagMappings.output——自定义 output 格式只经 FlagMapper 生效。
    assert_eq!(
        FlagMapper::map_output(&custom, &"main.o".into(), ".o"),
        "--out=main",
        "自定义 output FlagFormat 经 mapper 生效"
    );
    let t = CommandTemplates::new(&custom);
    let compile = t.render_compile(
        "C:/my tools/cc.exe",
        &FlagMapper::map_compile_only(&custom),
        &FlagMapper::format_includes(&custom, &["inc".into(), "inc 2".into()]),
        &FlagMapper::format_defines(&custom, &["DEBUG".into()]),
        &custom
            .default_cflags
            .iter()
            .map(|f| f.as_str())
            .collect::<Vec<_>>()
            .join(" "),
        "main.o",
        "main.c",
    );
    assert_eq!(
        compile,
        "C:/my tools/cc.exe --compile --inc=inc --inc=inc 2 --def=DEBUG -O2 -g -omain.o main.c"
    );

    let assemble = t.render_assemble("as", "-Iinc", "-g", "main.o", "main.S");
    assert!(assemble.starts_with("as -Iinc -g"));

    let link = t.render_link("cc.exe", "app", "main.o", "-Wl,-x", "libcore.a");
    assert_eq!(link, "cc.exe -oapp main.o -Wl,-x libcore.a");

    let archive = t.render_archive("ar.exe", "libmine.a", "main.o");
    assert!(archive.contains("libmine.a"));

    // render 直接入口：未知占位符原样保留（不擅自删除）
    let mut vars: HashMap<&str, String> = HashMap::new();
    vars.insert("cc", "gcc".to_string());
    assert_eq!(t.render("{cc} {unknown_var}", &vars), "gcc {unknown_var}");
}

/// 替换序规则（F-805-R1-02）：按 key 长度降序**一轮**顺序替换——后处理
/// 的更短 key 会展开先前插入值中的占位符；已处理 key 不回扫。等长 key
/// 的处理序继承 HashMap 迭代（原未定序角落）：本控制断言结果只落在
/// 两种合法排序结果之内，不存在第三态。
#[test]
fn external_consumer_pins_render_ordering_semantics() {
    let t = CommandTemplates::new(&CompilerConfig::gcc_default());

    // 长度不同 → 确定：长 key 先插入 {x}，短 key 后处理再展开
    let mut vars: HashMap<&str, String> = HashMap::new();
    vars.insert("longkey", "{x}".to_string());
    vars.insert("x", "done".to_string());
    assert_eq!(t.render("{longkey}", &vars), "done");

    // 长度不同 → 确定：x 后处理插入 {longkey}，longkey 已处理不回扫
    let mut vars: HashMap<&str, String> = HashMap::new();
    vars.insert("longkey", "done".to_string());
    vars.insert("x", "{longkey}".to_string());
    assert_eq!(t.render("{x}", &vars), "{longkey}");

    // 等长 key（aa / bb）→ 处理序不定，但结果只可能是两种排序之一：
    // aa 先 → "{bb}" 插入后 bb 再展开 → "done"；bb 先 → 模板无 "{bb}"
    // 不变，aa 后插入 "{bb}" 定型。
    let mut outcomes = std::collections::HashSet::new();
    for _ in 0..256 {
        let mut vars: HashMap<&str, String> = HashMap::new();
        vars.insert("aa", "{bb}".to_string());
        vars.insert("bb", "done".to_string());
        outcomes.insert(t.render("{aa}", &vars));
    }
    assert!(
        outcomes.iter().all(|o| o == "done" || o == "{bb}"),
        "等长 key 只允许两种合法排序结果，实际: {:?}",
        outcomes
    );
}
