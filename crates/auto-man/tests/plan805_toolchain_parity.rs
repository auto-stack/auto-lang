//! PLAN-805 工具链规则抽取对拍（V02/V03/V04）。
//!
//! 本测试文件在两个臂上以**同一公开路径**运行：
//! - 旧臂（base 282e50053）：`auto_man::builder::ninja::{config,mapper,templates}`
//!   还是原实现；
//! - 新臂（抽取后）：同路径变为 `auto-toolchain-core` 的薄 re-export。
//!
//! oracle 冻结：设 `PLAN805_CAPTURE_DIR` 时把全部期望输出写为 golden 文件
//! （只允许在旧臂基线上执行一次）；不设时读取
//! `tests/fixtures/plan805/` 下的 golden 逐字节断言。golden 只描述行为，
//! 不复制实现细节。
//!
//! 确定性边界（T-01 登记）：NinjaBuilder 探针钉
//! `CompilerLocation::Dir` 到不存在的目录——不走 PATH/磁盘观测；每 target
//! 单 src——规避 `Target.srcs: HashSet` 迭代序；`all_incs()` 已排序、
//! `defines` 保序。`render()` 等长 key 的 HashMap 迭代序是原实现未定序
//! 角落，golden 只含与序无关的确定案例。

use auto_man::builder::ninja::config::{
    CompilerConfig, CompilerKind, CompilerLocation, ExecutableType, FlagFormat,
};
use auto_man::builder::ninja::mapper::FlagMapper;
use auto_man::builder::ninja::templates::CommandTemplates;
use auto_man::builder::ninja::NinjaBuilder;
use auto_man::builder::Builder;
use auto_man::pac::Pac;
use auto_man::port::Port;
use auto_man::target::{Target, TargetKind};
use auto_val::AutoPath;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const KINDS: [(&str, fn() -> CompilerConfig); 8] = [
    ("msvc", CompilerConfig::msvc_default),
    ("gcc", CompilerConfig::gcc_default),
    ("clang", CompilerConfig::clang_default),
    ("ghs", CompilerConfig::ghs_default),
    ("iar", CompilerConfig::iar_default),
    ("targeting", CompilerConfig::targeting_default),
    ("hightec", CompilerConfig::hightec_default),
    ("ticlang", CompilerConfig::ticlang_default),
];

/// 不存在的钉定目录：Dir 位置 join 不做存在性探测，MSVC 同目录优先的
/// candidate.exists() 恒 false，两条路径都确定。
const PROBE_TOOL_DIR: &str = "C:/__plan805_probe__/tc";

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/plan805")
}

fn capture_dir() -> Option<PathBuf> {
    std::env::var("PLAN805_CAPTURE_DIR").ok().map(PathBuf::from)
}

/// 每个用例经 emit(name, content) 落盘（捕获态）或对拍（断言态）。
fn emit(goldens: &mut Vec<(String, String)>, name: &str, content: String) {
    goldens.push((name.to_string(), content));
}

fn dump_config(cfg: &CompilerConfig) -> String {
    let mut out = String::new();
    out.push_str(&format!("kind={}\n", cfg.kind.as_str()));
    out.push_str(&format!("name={}\n", cfg.name.as_str()));
    // executables 是 HashMap：按角色名排序保证 dump 确定（值本身是行为）
    let mut roles: Vec<(&ExecutableType, &AutoPath)> = cfg.executables.iter().collect();
    roles.sort_by_key(|(t, _)| t.as_str());
    for (t, p) in roles {
        out.push_str(&format!("exec[{}]={}\n", t.as_str(), p.to_astr().as_str()));
    }
    out.push_str(&format!("flags.include={}\n", cfg.flags.include.apply("v")));
    out.push_str(&format!("flags.define={}\n", cfg.flags.define.apply("v")));
    out.push_str(&format!("flags.library={}\n", cfg.flags.library.apply("v")));
    out.push_str(&format!(
        "flags.library_path={}\n",
        cfg.flags.library_path.apply("v")
    ));
    out.push_str(&format!("flags.output={}\n", cfg.flags.output.apply("v")));
    out.push_str(&format!(
        "flags.compile_only={}\n",
        FlagMapper::map_compile_only(cfg)
    ));
    out.push_str(&format!(
        "default_cflags={}\n",
        cfg.default_cflags
            .iter()
            .map(|f| f.as_str().to_string())
            .collect::<Vec<_>>()
            .join(" ")
    ));
    out.push_str(&format!(
        "obj_ext={} exe_ext={}\n",
        cfg.get_object_extension(),
        cfg.get_executable_extension()
    ));
    match &cfg.location {
        CompilerLocation::Env => out.push_str("location=Env\n"),
        CompilerLocation::Dir(d) => {
            out.push_str(&format!("location=Dir({})\n", d.to_astr().as_str()))
        }
        CompilerLocation::Executable(t, p) => out.push_str(&format!(
            "location=Executable({},{})\n",
            t.as_str(),
            p.to_astr().as_str()
        )),
    }
    out
}

/// V02：八类默认 + 全部 alias/未知 kind + 四角色 + 三 FlagFormat +
/// 空/多值/含空格/Unicode/重复扩展尾缀。
fn config_and_mapper_goldens(goldens: &mut Vec<(String, String)>) {
    let mut dump = String::new();
    for (name, ctor) in KINDS {
        dump.push_str(&format!("== {} ==\n", name));
        dump.push_str(&dump_config(&ctor()));
    }
    emit(goldens, "config_defaults.txt", dump);

    let mut alias = String::new();
    for (input, expect) in [
        ("msvc", "msvc"),
        ("MSVC", "msvc"),
        ("gcc", "gcc"),
        ("GCC", "gcc"),
        ("gnu", "gcc"),
        ("clang", "clang"),
        ("LLVM", "clang"),
        ("ghs", "ghs"),
        ("iar", "iar"),
        ("targeting", "targeting"),
        ("hightec", "hightec"),
        ("ticlang", "ticlang"),
        ("unknown", "<none>"),
        ("", "<none>"),
    ] {
        let got = CompilerKind::from_str(input)
            .map(|k| k.as_str().to_string())
            .unwrap_or_else(|| "<none>".to_string());
        alias.push_str(&format!("kind({})={}\n", input, got));
        assert_eq!(got, expect, "CompilerKind::from_str({})", input);
    }
    for (input, expect) in [
        ("cc", "cc"),
        ("compiler", "cc"),
        ("ar", "ar"),
        ("archiver", "ar"),
        ("link", "link"),
        ("linker", "link"),
        ("as", "as"),
        ("assembler", "as"),
        ("CC", "<none>"), // 大小写敏感：旧臂行为
        ("unknown", "<none>"),
    ] {
        let got = ExecutableType::from_str(input)
            .map(|t| t.as_str().to_string())
            .unwrap_or_else(|| "<none>".to_string());
        alias.push_str(&format!("exe_type({})={}\n", input, got));
        assert_eq!(got, expect, "ExecutableType::from_str({})", input);
    }
    // CompilerKind Display
    alias.push_str(&format!("display={}\n", CompilerKind::TICLang));
    emit(goldens, "aliases.txt", alias);

    // FlagFormat 三型 apply（含值本身含 "{}" 的边界——replace 全量替换）
    let mut ff = String::new();
    ff.push_str(&format!(
        "prefix={}\n",
        FlagFormat::Prefix("-I{}".to_string()).apply("inc dir")
    ));
    ff.push_str(&format!(
        "postfix={}\n",
        FlagFormat::Postfix("{}.lib".to_string()).apply("my lib")
    ));
    ff.push_str(&format!(
        "both={}\n",
        FlagFormat::Both("/Fo".to_string(), ".obj".to_string()).apply("out put")
    ));
    ff.push_str(&format!(
        "unicode={}\n",
        FlagFormat::Prefix("-I{}".to_string()).apply("路径/中文 目录")
    ));
    ff.push_str(&format!(
        "value_with_hole={}\n",
        FlagFormat::Prefix("[{}]".to_string()).apply("a{}b")
    ));
    emit(goldens, "flag_format.txt", ff);

    // FlagMapper 边界：空/多值/空格/Unicode/重复尾缀
    let gcc = CompilerConfig::gcc_default();
    let msvc = CompilerConfig::msvc_default();
    let iar = CompilerConfig::iar_default();
    let mut mapper = String::new();
    mapper.push_str(&format!(
        "includes_empty={}\n",
        FlagMapper::format_includes(&gcc, &[])
    ));
    mapper.push_str(&format!(
        "defines_empty={}\n",
        FlagMapper::format_defines(&gcc, &[])
    ));
    mapper.push_str(&format!(
        "includes_multi={}\n",
        FlagMapper::format_includes(&gcc, &["inc1".into(), "inc 2".into(), "含空格/目录".into()])
    ));
    mapper.push_str(&format!(
        "defines_multi={}\n",
        FlagMapper::format_defines(&gcc, &["DEBUG=1".into(), "VERSION=2".into()])
    ));
    mapper.push_str(&format!(
        "libraries={}\n",
        FlagMapper::format_libraries(&gcc, &["mylib1".into(), "my lib2".into()])
    ));
    mapper.push_str(&format!(
        "library_paths={:?}\n",
        FlagMapper::build_library_paths(&msvc, &["C:/libs".into(), "D:/a b".into()])
    ));
    // map_output：Prefix/Both 剥尾缀（trim_end_matches 全部重复尾缀），Postfix 不剥
    for (tag, cfg, output, ext) in [
        ("gcc", &gcc, "main.o", ".o"),
        ("gcc_noext", &gcc, "output", ".o"),
        ("gcc_repeat", &gcc, "main.o.o", ".o"),
        ("msvc", &msvc, "main.obj", ".obj"),
        ("msvc_repeat", &msvc, "main.obj.obj", ".obj"),
        ("msvc_noext", &msvc, "output", ".obj"),
    ] {
        mapper.push_str(&format!(
            "map_output({})={}\n",
            tag,
            FlagMapper::map_output(cfg, &output.into(), ext)
        ));
    }
    // Postfix 型 output（旧臂：用原始 output 不剥尾缀）
    let mut post = CompilerConfig::gcc_default();
    post.flags.output = FlagFormat::Postfix("{}.elf".to_string());
    mapper.push_str(&format!(
        "map_output(postfix)={}\n",
        FlagMapper::map_output(&post, &"main.elf".into(), ".elf")
    ));
    mapper.push_str(&format!(
        "compile_only_gcc={}\n",
        FlagMapper::map_compile_only(&gcc)
    ));
    mapper.push_str(&format!(
        "compile_only_msvc={}\n",
        FlagMapper::map_compile_only(&msvc)
    ));
    mapper.push_str(&format!(
        "compile_only_iar={}\n",
        FlagMapper::map_compile_only(&iar)
    ));
    // Both 型 compile_only（concat）
    let mut both = CompilerConfig::gcc_default();
    both.flags.compile_only = FlagFormat::Both("-c".to_string(), " extra".to_string());
    mapper.push_str(&format!(
        "compile_only(both)={}\n",
        FlagMapper::map_compile_only(&both)
    ));
    emit(goldens, "mapper_edges.txt", mapper);
}

/// V03：八类命令模板 + render 语义（缺变量保留、长短 key、四命令渲染）。
fn template_goldens(goldens: &mut Vec<(String, String)>) {
    let mut dump = String::new();
    for (name, ctor) in KINDS {
        let t = CommandTemplates::new(&ctor());
        dump.push_str(&format!(
            "== {} ==\ncompile={}\nassemble={}\nlink={}\narchive={}\n",
            name, t.compile, t.assemble, t.link, t.archive
        ));
    }
    emit(goldens, "templates_all_kinds.txt", dump);

    let mut render = String::new();
    let t = CommandTemplates::new(&CompilerConfig::gcc_default());
    // 缺变量：模板里的未知占位符原样保留
    let mut vars: HashMap<&str, String> = HashMap::new();
    vars.insert("cc", "gcc".to_string());
    render.push_str(&format!(
        "missing_var={}\n",
        t.render("{cc} {nope} {input}", &vars)
    ));
    // 长短 key 并存（{in} 与 {input} 互不为子串，结果与替换序无关）
    let mut vars: HashMap<&str, String> = HashMap::new();
    vars.insert("in", "X".to_string());
    vars.insert("input", "in.c".to_string());
    render.push_str(&format!("long_short={}\n", t.render("{input} {in}", &vars)));

    // 替换序控制（F-805-R1-02，复审 R1 复放集，期望值锚定冻结核心=旧臂
    // blob）：按 key 长度降序做**一轮**顺序替换——后处理的（更短）key 会
    // 展开先前插入值中的占位符；但已处理的 key 不回扫（非不动点迭代）。
    // 两例 key 长度不同、确定性；不入 golden（golden 保持旧臂捕获面）。
    let mut vars: HashMap<&str, String> = HashMap::new();
    vars.insert("longkey", "{x}".to_string());
    vars.insert("x", "done".to_string());
    assert_eq!(
        t.render("{longkey}", &vars),
        "done",
        "长 key 先插入 {{x}}，短 key 后处理再展开"
    );

    let mut vars: HashMap<&str, String> = HashMap::new();
    vars.insert("longkey", "done".to_string());
    vars.insert("x", "{longkey}".to_string());
    assert_eq!(
        t.render("{x}", &vars),
        "{longkey}",
        "已处理的 key 不再回扫（非不动点）"
    );

    // 四命令渲染（空 ldflags 双空格保持——不去重/不清理）
    render.push_str(&format!(
        "render_compile={}\n",
        t.render_compile(
            "gcc",
            "-c",
            "-Iinclude -Iinc 2",
            "-DDEBUG -DVERSION=2",
            "-O2 -Wall",
            "main.o",
            "main.c"
        )
    ));
    render.push_str(&format!(
        "render_assemble={}\n",
        t.render_assemble("as", "-Iinclude", "-g", "main.o", "main.S")
    ));
    render.push_str(&format!(
        "render_link={}\n",
        t.render_link("gcc", "main", "main.o", "", "-lmylib -l\"my lib2\"")
    ));
    render.push_str(&format!(
        "render_archive={}\n",
        t.render_archive("ar", "libmylib.a", "obj1.o obj2.o")
    ));
    render.push_str(&format!(
        "render_unicode={}\n",
        t.render_compile("gcc", "-c", "-I路径/中文", "-DDEBUG", "-O2", "主.o", "主.c")
    ));
    // MSVC 渲染（/Fo{output} 形模板 + /c）
    let tm = CommandTemplates::new(&CompilerConfig::msvc_default());
    render.push_str(&format!(
        "render_msvc_compile={}\n",
        tm.render_compile(
            "cl.exe",
            "/c",
            "/Iinc",
            "/DDEBUG",
            "/O2 /W3 /MD",
            "main.obj",
            "main.c"
        )
    ));
    emit(goldens, "render_semantics.txt", render);
}

/// 构造探针 Pac：port 平台/编译器可钉，targets 单 src（HashSet 序确定性）。
fn probe_pac(platform: &str, compiler: CompilerConfig) -> Pac {
    let mut pac =
        Pac::new(auto_lang::config::AutoConfig::new("name: \"plan805-probe\"\n").unwrap());
    pac.port = Port {
        name: format!("{}_ninja", platform).into(),
        builder: "ninja".into(),
        platform: platform.into(),
        at: "build".into(),
        os: Some(platform.into()),
        arch: Some("x86_64".into()),
        toolchain: Some(compiler.kind.as_str().into()),
        exports: vec![],
        sdks: vec![],
        compiler: Some(compiler),
    };
    pac.defines = vec!["DEBUG=1".into(), "VERSION=2".into()];

    let mut app = Target::new("app", TargetKind::App);
    app.rename = "app".into();
    app.srcs.insert("src/main.c".into());
    app.incs.insert("include".into());
    app.incs.insert("路径 中文".into());
    app.port.platform = platform.into();
    pac.targets.push(app);

    let mut math = Target::new("math", TargetKind::Lib);
    math.rename = "math".into();
    math.srcs.insert("math/math.c".into());
    math.port.platform = platform.into();
    pac.targets.push(math);

    let mut t = Target::new("t", TargetKind::Test);
    t.rename = "t".into();
    t.srcs.insert("t/test.c".into());
    t.port.platform = platform.into();
    pac.targets.push(t);

    pac
}

fn ninja_memory_output(pac: &mut Pac) -> String {
    let mut builder = NinjaBuilder::new(AutoPath::new("build/build.ninja"));
    builder.enable_memory_output().unwrap();
    builder.build(pac).unwrap();
    let map: HashMap<String, Vec<u8>> = builder.get_memory_output();
    let bytes = map.get("build.ninja").expect("memory build.ninja");
    String::from_utf8(bytes.clone()).expect("utf8 build.ninja")
}

/// V04：八类 NinjaBuilder 真实生产输出（经 CompilerStore 外的同一生管线：
/// load_compiler_config→FlagMapper/format_defines→CommandTemplates→quote_if_spaced）。
fn ninja_goldens(goldens: &mut Vec<(String, String)>) {
    for (name, ctor) in KINDS {
        let mut cfg = ctor();
        cfg.location = CompilerLocation::Dir(AutoPath::new(PROBE_TOOL_DIR));
        let out = ninja_memory_output(&mut probe_pac("windows", cfg));
        emit(goldens, &format!("ninja_{}_windows.txt", name), out);
    }
    // linux 平台命名（exe 无后缀 / lib lib{}.a）+ 自定义 flag + 含空格工具路径
    let mut gcc = CompilerConfig::gcc_default();
    gcc.location = CompilerLocation::Dir(AutoPath::new(PROBE_TOOL_DIR));
    gcc.default_cflags = vec!["-O2".into(), "-g3".into(), "--custom-flag".into()];
    gcc.executables.insert(
        ExecutableType::Compiler,
        AutoPath::new("C:/tools with space/bin/cc.exe"),
    );
    let out = ninja_memory_output(&mut probe_pac("linux", gcc));
    emit(goldens, "ninja_gcc_linux_custom.txt", out);
}

fn compiler_store_goldens(goldens: &mut Vec<(String, String)>) {
    // CompilerStore 上层合同冻结（PLAN-805 不改加载算法）：真实 am.at 三态。
    // 注意：现行 load_from_am_config 用 get_kid(Str("compilers")) 找节点，
    // 而解析器对文档形 kids 产位置 Int 键——合法文档形态下恒空载。这是
    // 预存上游缺口（本计划登记入 KNOWN-DEBT，不在此修复）；golden 冻结
    // 的就是这个旧臂真实行为，抽取前后必须逐字节一致。
    use auto_man::builder::ninja::compiler_store::CompilerStore;
    let tmp = tempfile::tempdir().unwrap();

    // ① 合法 am.at（compilers 节点形态）：旧臂行为 = Ok + 空 store
    let am_at = tmp.path().join("am.at");
    fs::write(
        &am_at,
        "compilers {\n  compiler(\"mygcc\") {\n    kind: \"gcc\"\n  }\n}\n",
    )
    .unwrap();
    let store = CompilerStore::load_from_am_config(&am_at).unwrap();
    let mut dump = String::new();
    dump.push_str(&format!(
        "valid_doc_names_empty={}\n",
        store.list_names().is_empty()
    ));
    dump.push_str(&format!(
        "valid_doc_has_mygcc={}\n",
        store.has(&"mygcc".into())
    ));

    // ② 坏语法 am.at：AutoConfig::read Err → load Err 上抛
    let bad = tmp.path().join("bad.at");
    fs::write(&bad, "compilers {{{{ not valid .at {{{\n").unwrap();
    let bad_result = CompilerStore::load_from_am_config(&bad);
    dump.push_str(&format!("bad_syntax_is_err={}\n", bad_result.is_err()));

    // ③ 缺文件：Ok + 空 store（不报错）
    let missing = CompilerStore::load_from_am_config(&tmp.path().join("nope.at")).unwrap();
    dump.push_str(&format!(
        "missing_names_empty={}\n",
        missing.list_names().is_empty()
    ));
    emit(goldens, "compiler_store.txt", dump);
}

// （V02 类型同一性断言在抽取后由 tests/plan805_type_identity.rs 承担——
// 本文件必须在 base 提交可编译以捕获旧臂 golden。）

#[test]
fn plan805_toolchain_parity_goldens() {
    let mut goldens: Vec<(String, String)> = Vec::new();
    config_and_mapper_goldens(&mut goldens);
    template_goldens(&mut goldens);
    ninja_goldens(&mut goldens);
    compiler_store_goldens(&mut goldens);

    if let Some(dir) = capture_dir() {
        fs::create_dir_all(&dir).unwrap();
        for (name, content) in &goldens {
            fs::write(dir.join(name), content).unwrap();
        }
        eprintln!(
            "[plan805] captured {} goldens to {}",
            goldens.len(),
            dir.display()
        );
        return;
    }

    let dir = fixtures_dir();
    for (name, content) in &goldens {
        let golden = fs::read_to_string(dir.join(name)).unwrap_or_else(|e| {
            panic!(
                "golden {} 读取失败（先在旧臂基线 PLAN805_CAPTURE_DIR 捕获）: {}",
                name, e
            )
        });
        assert_eq!(&golden, content, "golden {} 与当前实现输出不一致", name);
    }
}
