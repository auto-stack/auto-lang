//! PLAN-803 V03/V04 — asset.rs 兼容门面 vs 旧臂冻结投影 + 真 exporter 最小工程。
//!
//! 全部期望取自迁移基线（master `1dd123361a`）旧臂实测探针
//! （docs/plans/attachments/803/t01-oldarm-probe.txt；新臂重放逐字节一致
//! 收据 t01-newarm-replay-identical.txt）——非新 repr 合成。门面语义：
//! 枚举/选择/路径替换委托 auto-man-assets 纯计划；逐文件 Auto 求值/
//! 文件物化/失败阶段（含 unwrap panic、先建文件再求值遗留顺序）保持
//! 旧臂原样（行为改善另立计划）。

use auto_man::asset::{GHSTemplate, IARTemplate, Templates};
use auto_man::exporter::{Exporter, GHSExporter, IARExporter};
use auto_man::Pac;
use auto_man_assets::{GhsAssets, IarAssets, TemplateAssets};
use auto_val::{AutoPath, Obj};
use rust_embed::{Filenames, RustEmbed};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// 基线旧臂 Templates::copy("app", …/mylib) 输出冻结（attachments/803 t01 探针实测，非新 repr 合成）。
const FROZEN_APP: [(&str, &str); 2] = [
    ("mylib/mylib.at",
     "print(\"Hello, mylib!\")\n"),
    ("mylib/pac.at",
     "name: \"mylib\"\nversion: \"0.1.0\"\n\n// desktop_render: \"auto\" | \"queue\" | \"independent\" (Plan 500)\n// desktop_render: \"auto\"\n\napp(\"\") {}\n"),
];

/// 基线旧臂 Templates::copy("capp", …/mylib) 输出冻结（attachments/803 t01 探针实测，非新 repr 合成）。
const FROZEN_CAPP: [(&str, &str); 2] = [
    ("mylib/mylib.c",
     "#include <stdio.h>\n\nint main(void) {\n    printf(\"Hello, mylib!\\n\");\n    return 0;\n}\n"),
    ("mylib/pac.at",
     "name: \"mylib\"\nversion: \"0.1.0\"\n\napp(\"name: \"mylib\"\nversion: \"0.1.0\"\n\n\") {}\n\n"),
];

/// 基线旧臂 Templates::copy("clib", …/mylib) 输出冻结（attachments/803 t01 探针实测，非新 repr 合成）。
const FROZEN_CLIB: [(&str, &str); 3] = [
    ("mylib/mylib.c",
     "#include \"mylib.h\"\n\nint _add(int a, int b) {\n    return a + b;\n}\n"),
    ("mylib/mylib.h",
     "#ifndef _MYLIB_H\n#define _MYLIB_H\n\nint mylib_add(int a, int b);\n\n#endif // _MYLIB_H\n"),
    ("mylib/pac.at",
     "name: \"mylib\"\nversion: \"0.1.0\"\n\nlib name: \"mylib\"\nversion: \"0.1.0\"\n\n {}\n"),
];

/// 基线旧臂 Templates::copy("lib", …/mylib) 输出冻结（attachments/803 t01 探针实测，非新 repr 合成）。
const FROZEN_LIB: [(&str, &str); 2] = [
    ("mylib/mylib.at",
     "fn add(a int, b int) int {\n    a + b\n}\n"),
    ("mylib/pac.at",
     "name: \"mylib\"\nversion: \"0.1.0\"\n\nlib(\"name: \"mylib\"\nversion: \"0.1.0\"\n\n\") {}\n"),
];

/// 基线旧臂 Templates::copy("libdemo", …/mylib) 输出冻结（attachments/803 t01 探针实测，非新 repr 合成）。
const FROZEN_LIBDEMO: [(&str, &str); 4] = [
    ("mylib/apps/mylib_demo/mylib_demo.c",
     "#include <stdio.h>\n#include \"mylib.h\"\n\nint main() {\n    printf(\"#include <stdio.h>\n#include \"mylib.h\"\n\nint main() {\n_add(1, 2) = %d\\n\", #include <stdio.h>\n#include \"mylib.h\"\n\nint main() {\n_add(1, 2));\n    return 0;\n}\n\n"),
    ("mylib/libs/mylib/mylib.c",
     "#include \"mylib.h\"\n\nint _add(int a, int b) {\n    return a + b;\n}\n\n"),
    ("mylib/libs/mylib/mylib.h",
     "#ifndef MYLIB_H\n#define MYLIB_H\n\nint mylib_add(int a, int b);\n\n#endif\n"),
    ("mylib/pac.at",
     "name: \"mylib\"\nversion: \"0.1.0\"\n\nlib(\"name: \"mylib\"\nversion: \"0.1.0\"\n\n\") {}\n\napp(\"name: \"mylib\"\nversion: \"0.1.0\"\n\nlib(\"name: \"mylib\"\nversion: \"0.1.0\"\n\n\") {}\n\n_demo\") {\n    srcs: [\"apps//_demo.c\"]\n    links: [\"name: \"mylib\"\nversion: \"0.1.0\"\n\nlib(\"name: \"mylib\"\nversion: \"0.1.0\"\n\n\") {}\n\napp(\"name: \"mylib\"\nversion: \"0.1.0\"\n\nlib(\"name: \"mylib\"\nversion: \"0.1.0\"\n\n\") {}\n\n_demo\") {\n    srcs: [\"apps//_demo.c\"]\n\"]\n}\n"),
];

/// 基线旧臂 Templates::copy("app", …/组件) 输出冻结（attachments/803 t01 探针实测，非新 repr 合成）。
const FROZEN_UNI_APP: [(&str, &str); 2] = [
    ("组件/pac.at",
     "name: \"组件\"\nversion: \"0.1.0\"\n\n// desktop_render: \"auto\" | \"queue\" | \"independent\" (Plan 500)\n// desktop_render: \"auto\"\n\napp(\"\") {}\n"),
    ("组件/组件.at",
     "print(\"Hello, 组件!\")\n"),
];

/// 基线旧臂 Templates::copy("clib", …/组件) 输出冻结（attachments/803 t01 探针实测，非新 repr 合成）。
const FROZEN_UNI_CLIB: [(&str, &str); 3] = [
    (
        "组件/pac.at",
        "name: \"组件\"\nversion: \"0.1.0\"\n\nlib name: \"组件\"\nversion: \"0.1.0\"\n\n {}\n",
    ),
    (
        "组件/组件.c",
        "#include \"组件.h\"\n\nint _add(int a, int b) {\n    return a + b;\n}\n",
    ),
    (
        "组件/组件.h",
        "#ifndef _组件_H\n#define _组件_H\n\nint 组件_add(int a, int b);\n\n#endif // _组件_H\n",
    ),
];

/// 基线旧臂 Templates::copy("libdemo", …/组件) 输出冻结（attachments/803 t01 探针实测，非新 repr 合成）。
const FROZEN_UNI_LIBDEMO: [(&str, &str); 4] = [
    ("组件/apps/组件_demo/组件_demo.c",
     "#include <stdio.h>\n#include \"组件.h\"\n\nint main() {\n    printf(\"#include <stdio.h>\n#include \"组件.h\"\n\nint main() {\n_add(1, 2) = %d\\n\", #include <stdio.h>\n#include \"组件.h\"\n\nint main() {\n_add(1, 2));\n    return 0;\n}\n\n"),
    ("组件/libs/组件/组件.c",
     "#include \"组件.h\"\n\nint _add(int a, int b) {\n    return a + b;\n}\n\n"),
    ("组件/libs/组件/组件.h",
     "#ifndef 组件_H\n#define 组件_H\n\nint 组件_add(int a, int b);\n\n#endif\n"),
    ("组件/pac.at",
     "name: \"组件\"\nversion: \"0.1.0\"\n\nlib(\"name: \"组件\"\nversion: \"0.1.0\"\n\n\") {}\n\napp(\"name: \"组件\"\nversion: \"0.1.0\"\n\nlib(\"name: \"组件\"\nversion: \"0.1.0\"\n\n\") {}\n\n_demo\") {\n    srcs: [\"apps//_demo.c\"]\n    links: [\"name: \"组件\"\nversion: \"0.1.0\"\n\nlib(\"name: \"组件\"\nversion: \"0.1.0\"\n\n\") {}\n\napp(\"name: \"组件\"\nversion: \"0.1.0\"\n\nlib(\"name: \"组件\"\nversion: \"0.1.0\"\n\n\") {}\n\n_demo\") {\n    srcs: [\"apps//_demo.c\"]\n\"]\n}\n"),
];

// ---------------------------------------------------------------------------
// V03：真 Templates::copy 冻结投影（成功文件集/字节 = 旧臂实测）
// ---------------------------------------------------------------------------

fn fresh_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("plan803_facade_{}_{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn walk(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn rec(dir: &Path, rel: &str, out: &mut BTreeMap<String, Vec<u8>>) {
        let entries: Vec<_> = std::fs::read_dir(dir)
            .expect("read_dir")
            .filter_map(|e| e.ok())
            .collect();
        for entry in entries {
            let name = entry.file_name().to_string_lossy().to_string();
            let child = if rel.is_empty() {
                name.clone()
            } else {
                format!("{rel}/{name}")
            };
            let path = entry.path();
            if path.is_dir() {
                rec(&path, &child, out);
            } else {
                out.insert(child, std::fs::read(&path).expect("read"));
            }
        }
    }
    let mut out = BTreeMap::new();
    if root.exists() {
        rec(root, "", &mut out);
    }
    out
}

fn check_frozen(tag: &str, family: &str, pac: &str, frozen: &[(&str, &str)]) {
    let dir = fresh_dir(tag);
    Templates::copy(family, dir.join(pac).to_str().unwrap()).expect("copy ok");
    let files = walk(&dir);
    let expect: BTreeMap<&str, &str> = frozen.iter().copied().collect();
    assert_eq!(
        files.len(),
        expect.len(),
        "{tag}: 文件集漂移 got {:?} want {:?}",
        files.keys(),
        expect.keys()
    );
    for (rel, want) in &expect {
        let got = files
            .get(*rel)
            .unwrap_or_else(|| panic!("{tag}: 缺文件 {rel}"));
        assert_eq!(&String::from_utf8_lossy(got), want, "{tag}/{rel}: 字节漂移");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn copy_matches_frozen_old_arm() {
    check_frozen("app", "app", "mylib", &FROZEN_APP);
    check_frozen("capp", "capp", "mylib", &FROZEN_CAPP);
    check_frozen("clib", "clib", "mylib", &FROZEN_CLIB);
    check_frozen("lib", "lib", "mylib", &FROZEN_LIB);
    check_frozen("libdemo", "libdemo", "mylib", &FROZEN_LIBDEMO);
}

#[test]
fn copy_unicode_matches_frozen_old_arm() {
    check_frozen("uni-app", "app", "组件", &FROZEN_UNI_APP);
    check_frozen("uni-clib", "clib", "组件", &FROZEN_UNI_CLIB);
    check_frozen("uni-libdemo", "libdemo", "组件", &FROZEN_UNI_LIBDEMO);
}

#[test]
fn copy_unknown_family_zero_side_effects() {
    // 旧臂：未知族 Ok(()) 且不创建任何目录/文件
    let dir = fresh_dir("unknown");
    Templates::copy("no-such-family", dir.join("x").to_str().unwrap()).expect("ok");
    assert!(!dir.exists(), "未知族不得创建任何路径");
}

#[test]
fn copy_panics_on_dotdot_path_like_old_arm() {
    // 旧臂冻结：file_name() None → unwrap panic（消息与基线一致）
    let outcome = std::panic::catch_unwind(|| Templates::copy("app", "somewhere/.."));
    let panic = outcome.expect_err("旧臂在 .. 结尾路径必 panic");
    let msg = panic
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default();
    assert_eq!(msg, "called `Option::unwrap()` on a `None` value");
}

// ---------------------------------------------------------------------------
// V04：门面转发（inherent + RustEmbed 双形态）与资源层字节单一来源
// ---------------------------------------------------------------------------

/// 旧公开 RustEmbed 泛型调用面（derive 时代的消费方式）必须继续可编译。
fn embed_keys<E: RustEmbed>() -> Vec<String> {
    E::iter().map(|k| k.to_string()).collect()
}

/// F803-R1-01 编译正控：旧 derive trait impl 的具体返回类型 `Filenames`
/// 必须在跨 crate 调用方继续可见（RPITIT 隐藏类型为具体类型时跨 crate
/// 泄露）。若门面 trait impl 误写 `-> impl Iterator`，本函数以 E0308
/// 失编——与复审反例（review-r1/new-api-counterexample.log）同形。
#[test]
fn trait_iter_returns_concrete_filenames_type() {
    let _: Filenames = <Templates as RustEmbed>::iter();
    let _: Filenames = <GHSTemplate as RustEmbed>::iter();
    let _: Filenames = <IARTemplate as RustEmbed>::iter();
}

#[test]
fn facade_forwards_all_bytes_to_single_source() {
    // inherent 形态（无需 trait import 的旧调用）与 trait 形态逐一对照
    for key in Templates::iter() {
        let key = key.as_ref();
        let via_facade = Templates::get(key).expect("facade get");
        let via_source = TemplateAssets::get(key).expect("source get");
        assert_eq!(via_facade.data.as_ref(), via_source.data.as_ref(), "{key}");
    }
    assert_eq!(
        embed_keys::<Templates>(),
        embed_keys::<TemplateAssets>(),
        "iter 顺序漂移"
    );

    for key in GHSTemplate::iter() {
        let key = key.as_ref();
        assert_eq!(
            GHSTemplate::get(key).unwrap().data.as_ref(),
            GhsAssets::get(key).unwrap().data.as_ref(),
            "{key}"
        );
    }
    for key in IARTemplate::iter() {
        let key = key.as_ref();
        assert_eq!(
            IARTemplate::get(key).unwrap().data.as_ref(),
            IarAssets::get(key).unwrap().data.as_ref(),
            "{key}"
        );
    }
}

#[test]
fn builder_accessors_match_single_source_bytes() {
    // exporter 消费的 3+4 项：门面 inherent 方法 = 资源层字节（UTF-8 同构）
    let pairs = [
        (
            GHSTemplate::default().unwrap(),
            GhsAssets::get("default.gpj").unwrap(),
        ),
        (
            GHSTemplate::program().unwrap(),
            GhsAssets::get("program.gpj").unwrap(),
        ),
        (
            GHSTemplate::subproject().unwrap(),
            GhsAssets::get("subproject.gpj").unwrap(),
        ),
        (
            IARTemplate::eww().unwrap(),
            IarAssets::get("iar.eww").unwrap(),
        ),
        (
            IARTemplate::ewp().unwrap(),
            IarAssets::get("iar.ewp").unwrap(),
        ),
        (
            IARTemplate::ewt().unwrap(),
            IarAssets::get("iar.ewt").unwrap(),
        ),
        (
            IARTemplate::ewd().unwrap(),
            IarAssets::get("iar.ewd").unwrap(),
        ),
    ];
    for (acc, embed) in pairs {
        assert_eq!(acc.as_str().as_bytes(), embed.data.as_ref());
    }
}

// ---------------------------------------------------------------------------
// V04：真 exporter 最小工程（经门面/资源层真实取模板字节；不冒称商业
// 编译器编译通过——仅验证工程文件生成与关键结构）
// ---------------------------------------------------------------------------

fn mini_pac_source(builder: &str) -> String {
    format!(
        r#"
name: "demo"
version: "0.1.0"

port("win32") {{
    builder: "{builder}"
}}

app("demo") {{
    srcs: ["main.c"]
}}

device("board") {{
    icf: "demo.icf"
    ddf: "demo.ddf"
    board: "demo.board"
}}
"#
    )
}

/// 生产导出入口序列的最小化：parse→set_port→export。完整 resolve 还含
/// 依赖下载/链接（对无依赖最小工程与逐 target 传播 port 等价——extout/
/// groups 等 IAR 专属 props 由 target.to_node 按 port.builder 生成，见
/// target.rs `for IAR builder` 分支）。
fn mini_pac(builder: &str) -> Pac {
    let args = Obj::new();
    let code = mini_pac_source(builder);
    let config = auto_lang::config::AutoConfig::from_code(code, &args).expect("parse mini pac");
    let mut pac = Pac::new(config);
    pac.set_port("win32".into()).expect("set_port");
    let port = pac.port.clone();
    for t in pac.targets.iter_mut() {
        t.set_port(port.clone());
    }
    pac
}

fn files_under(root: &Path) -> BTreeMap<String, Vec<u8>> {
    walk(root)
}

#[test]
fn iar_exporter_minimal_project_writes_template_derived_files() {
    let dir = fresh_dir("iar-export");
    std::fs::create_dir_all(&dir).unwrap();
    let mut pac = mini_pac("iar");
    let mut exporter = IARExporter::new(AutoPath::new(dir.to_string_lossy().to_string()));
    exporter.export(&mut pac).expect("iar export");

    let files = files_under(&dir);
    assert!(!files.is_empty(), "iar 导出为空");
    // 工作区文件：eww 模板（资源层唯一字节）真实驱动生成
    let eww = String::from_utf8_lossy(&files["demo.eww"]).to_string();
    assert!(
        eww.starts_with("<?xml"),
        "eww 非 XML：{}",
        &eww[..40.min(eww.len())]
    );
    assert!(eww.contains("<workspace>"), "eww 缺 workspace 结构");
    assert!(eww.contains("demo"), "eww 未含应用名");
    // 应用三件套（ewp/ewt/ewd 模板经 rename 落为 demo.*）
    for name in ["demo.ewp", "demo.ewt", "demo.ewd"] {
        let content = &files[name];
        assert!(content.starts_with(b"<?xml"), "{name} 非 XML");
    }
    let ewp = String::from_utf8_lossy(&files["demo.ewp"]).to_string();
    assert!(ewp.contains("<project>"), "ewp 缺 project 结构");
    let _ = std::fs::remove_dir_all(&dir);
}

/// GHS 旧臂遗留事实（冻结，不修复——PLAN-803 只迁资源不改行为）：
/// default.gpj（default 模板）经真实 AutoGen 生成成功；app 级
/// program.gpj mold 在 auto-gen 翻转求值处 panic（解析级 `Expected
/// term`——flipped 代码只含模板文本+note 字符、与 pac 数据无关，
/// 且模板字节经 hash 冻结与旧臂逐字节相同、auto-gen/exporter 代码
/// 未动，故该 panic 为**预存**旧臂行为）。本测试钉住该边界：若未来
/// 行为计划修复模板求值，此断言变化即须重新冻结 exporter 投影。
#[test]
fn ghs_exporter_minimal_project_freezes_legacy_boundary() {
    let dir = fresh_dir("ghs-export");
    std::fs::create_dir_all(&dir).unwrap();
    let mut pac = mini_pac("ghs");
    let mut exporter = GHSExporter::new(AutoPath::new(dir.to_string_lossy().to_string()));
    let outcome =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| exporter.export(&mut pac)));

    // default.gpj 在 app mold 失败前已真实生成（模板字节驱动）
    let files = files_under(&dir);
    let default_gpj = files
        .get("default.gpj")
        .unwrap_or_else(|| panic!("缺 default.gpj：{:?}", files.keys()));
    let content = String::from_utf8_lossy(default_gpj).to_string();
    assert!(content.starts_with("#!gbuild"), "default.gpj 缺 gbuild 头");
    assert!(content.contains("demo"), "default.gpj 未含工程名");

    // 预存 panic 边界：app program.gpj mold 求值失败（消息与阶段冻结）
    assert!(
        outcome.is_err(),
        "app mold 应保持旧臂 panic（若已修复，须重冻 exporter 投影并更新本断言）"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
