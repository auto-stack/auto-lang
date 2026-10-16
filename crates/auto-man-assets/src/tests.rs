//! PLAN-803 冻结 oracle：三组资源的键/顺序/字节 hash 与纯路径计划投影。
//!
//! 全部期望值取自迁移基线（worktree base = master `1dd123361a`）旧臂
//! 实测：枚举顺序 = 旧 `Templates/GHSTemplate/IARTemplate::iter()` 输出，
//! 字节 hash = 迁移前 `crates/auto-man/assets` 下同名文件 SHA-256，
//! 计划投影 = 旧 `Templates::copy` 的枚举/选择/`replace("name", …)` 语义。
//! 原始探针输出见 docs/plans/attachments/803/t01-oldarm-probe.txt。

use super::{plan_template_family, GhsAssets, IarAssets, PlannedFile, TemplateAssets};
use sha2::{Digest, Sha256};

/// 模板 13 键（顺序 = rust-embed walkdir sort_by_file_name = 旧臂 iter）。
const TEMPLATE_KEYS: [&str; 13] = [
    "app/name.at",
    "app/pac.at",
    "capp/name.c",
    "capp/pac.at",
    "clib/name.c",
    "clib/name.h",
    "clib/pac.at",
    "lib/name.at",
    "lib/pac.at",
    "libdemo/apps/name_demo/name_demo.c",
    "libdemo/libs/name/name.c",
    "libdemo/libs/name/name.h",
    "libdemo/pac.at",
];

const GHS_KEYS: [&str; 3] = ["default.gpj", "program.gpj", "subproject.gpj"];
const IAR_KEYS: [&str; 4] = ["iar.ewd", "iar.ewp", "iar.ewt", "iar.eww"];

/// 迁移前基线（移动前实测）三组文件 (组, 键, 字节数, SHA-256)。
const FROZEN_BYTES: [(&str, &str, usize, &str); 20] = [
    (
        "templates",
        "app/name.at",
        25,
        "4f2b1b0cee13c170d068abf94812d0fd829989d2df18928f6fc23810949d1e94",
    ),
    (
        "templates",
        "app/pac.at",
        142,
        "2bbe97123bc886c5d4c7ea928555e926faafcfd1579e1069d7b70596542b22e9",
    ),
    (
        "templates",
        "capp/name.c",
        86,
        "7f67a1686b8750aefcf943cc7322b0d80dd0719b520c1f06b20360e790c986bc",
    ),
    (
        "templates",
        "capp/pac.at",
        53,
        "fd5ef9fe88b42c57e9cd1155b3ae530b1a4dd0472b0447c2ac83295695327bfb",
    ),
    (
        "templates",
        "clib/name.c",
        74,
        "c1ed2dde232165ab346f2e9d5d16010c3e2809e02b9e9f7dbb465e2e0a7552e3",
    ),
    (
        "templates",
        "clib/name.h",
        116,
        "adbdfc6dcb0364d2682c404f1bce0b7a631b5f9a19bebf862274f5e7cbc9d39d",
    ),
    (
        "templates",
        "clib/pac.at",
        49,
        "5a9726123f39d3d775fe31a4561f606c9c6a7981f016cef908747ef6ee0b46b6",
    ),
    (
        "templates",
        "lib/name.at",
        39,
        "662055e119e74a43d8276c2a92ca132b496d30fae16763f9c12920d2bcee54ed",
    ),
    (
        "templates",
        "lib/pac.at",
        52,
        "00835541a38a8caadbcc7213e40d3ee30d887e5e5eefaf9aa4215376bd5d4333",
    ),
    (
        "templates",
        "libdemo/apps/name_demo/name_demo.c",
        130,
        "174b97b1314c2b50d69219875bc78c6384f78b56100e30908292b4bef19b6537",
    ),
    (
        "templates",
        "libdemo/libs/name/name.c",
        75,
        "e83197ec86f4f72f958fbe92fb4c82cb5d30aea1855ecc4fbc2cb4c3dafb3159",
    ),
    (
        "templates",
        "libdemo/libs/name/name.h",
        92,
        "adcc1b36b51045db5bec99d81cccc31b2e59a8cbbea4a45ed00df92f53a381e7",
    ),
    (
        "templates",
        "libdemo/pac.at",
        142,
        "158229bf8bc96ac9d52e356d1195b0b4e49e5c063fa70661f9773d56c514b66e",
    ),
    (
        "ghs",
        "default.gpj",
        2393,
        "7ce59757f6cfd060fce1d627c414841cc05cf4de020db80c68c825aace416e51",
    ),
    (
        "ghs",
        "program.gpj",
        646,
        "d8cd9fc7ead39cb547ef5edc73aea97308e3b409690b442d6ac6e205c39bfda5",
    ),
    (
        "ghs",
        "subproject.gpj",
        169,
        "a81eb033ee1138ffba51f01b8a6ca2b1103f32e1bf8ba37b13b8d875fb085652",
    ),
    (
        "iar",
        "iar.ewd",
        115379,
        "61d64d55ac552f8acb9bf53acc83eea7820e1b5b9958af691679eaadc61f9f81",
    ),
    (
        "iar",
        "iar.ewp",
        75078,
        "fb896388cb6a2effcd2c3e14daf88814267874be656c46ecb013da68d05bedfc",
    ),
    (
        "iar",
        "iar.ewt",
        215469,
        "dd73a37fb3dc392218bafc024a6bd04493ec37d248ac115b19d5269a0f1595d7",
    ),
    (
        "iar",
        "iar.eww",
        188,
        "e2482e3cb53b6b872e751d7d97b05dd01b9431469fe8b966e1f27d44b54bfae9",
    ),
];

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn enumeration_keys_and_order_frozen() {
    let templates: Vec<String> = TemplateAssets::iter().map(|k| k.to_string()).collect();
    assert_eq!(templates, TEMPLATE_KEYS, "模板键/顺序漂移");

    let ghs: Vec<String> = GhsAssets::iter().map(|k| k.to_string()).collect();
    assert_eq!(ghs, GHS_KEYS);

    let iar: Vec<String> = IarAssets::iter().map(|k| k.to_string()).collect();
    assert_eq!(iar, IAR_KEYS);
}

#[test]
fn bytes_frozen_to_baseline_hashes() {
    for (group, key, len, hash) in FROZEN_BYTES {
        let bytes = match group {
            "templates" => TemplateAssets::get(key).unwrap().data,
            "ghs" => GhsAssets::get(key).unwrap().data,
            "iar" => IarAssets::get(key).unwrap().data,
            _ => unreachable!(),
        };
        assert_eq!(bytes.len(), len, "{group}/{key} 长度漂移");
        assert_eq!(sha(&bytes), hash, "{group}/{key} 字节漂移");
    }
}

#[test]
fn plan_projection_frozen() {
    // libdemo 全投影：嵌套目录、子串 name 多次出现、顺序（替换发生在
    // 去掉族前缀的余段上——与旧臂 split_first 后 replace 逐字一致）
    let plan = plan_template_family("libdemo", "mylib");
    let got: Vec<(&str, &str)> = plan
        .iter()
        .map(|p: &PlannedFile| (p.source_key.as_str(), p.output_rel.as_str()))
        .collect();
    assert_eq!(
        got,
        [
            (
                "libdemo/apps/name_demo/name_demo.c",
                "apps/mylib_demo/mylib_demo.c"
            ),
            ("libdemo/libs/name/name.c", "libs/mylib/mylib.c"),
            ("libdemo/libs/name/name.h", "libs/mylib/mylib.h"),
            ("libdemo/pac.at", "pac.at"),
        ]
    );

    // 顶层族：name.at→mylib.at，pac.at 无 name 子串不动
    let app: Vec<String> = plan_template_family("app", "mylib")
        .iter()
        .map(|p| p.output_rel.clone())
        .collect();
    assert_eq!(app, ["mylib.at", "pac.at"]);

    // Unicode 包名（旧臂 replace 语义逐字）
    let uni: Vec<String> = plan_template_family("clib", "组件")
        .iter()
        .map(|p| p.output_rel.clone())
        .collect();
    assert_eq!(uni, ["组件.c", "组件.h", "pac.at"]);

    // 计划字节 = 嵌入原始字节，${name} 未渲染
    let name_at = &plan_template_family("app", "mylib")[0];
    assert_eq!(name_at.source_key, "app/name.at");
    assert_eq!(
        String::from_utf8_lossy(&name_at.bytes),
        "print(\"Hello, ${name}!\")\n"
    );
}

#[test]
fn plan_unknown_family_is_empty() {
    assert!(plan_template_family("no-such-family", "x").is_empty());
    // 旧臂对空族零副作用；纯计划零输出 IO（下一测试钉住）
}

/// 零**输出** IO 契约：枚举/计划不创建/不写任何文件（F803-R1-03：
/// 调试态 rust-embed 动态枚举读源码资源目录属构建模式边界，见 lib.rs
/// 模块文档与 mode-boundary-probe.py，不在本断言范围）。
#[test]
fn plan_has_no_output_io() {
    let dir = std::env::temp_dir().join(format!("man_assets_iofree_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let _ = plan_template_family("libdemo", "iofree");
    let _ = TemplateAssets::get("app/pac.at");
    let _ = GhsAssets::get("default.gpj");
    let _ = IarAssets::get("iar.eww");
    assert!(
        std::fs::read_dir(&dir).unwrap().next().is_none(),
        "枚举/计划不得创建任何文件"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn split_first_segment_mirrors_legacy_util() {
    use super::split_first_segment;
    assert_eq!(split_first_segment("app/pac.at"), ("app", "pac.at"));
    assert_eq!(
        split_first_segment("libdemo/libs/name/name.c"),
        ("libdemo", "libs/name/name.c")
    );
    // 无分隔符输入：与旧臂 util::split_first 同样 panic（镜像保留原语义）
    let panicked = std::panic::catch_unwind(|| split_first_segment("plain")).is_err();
    assert!(panicked, "无分隔符输入应与旧臂同样 panic");
}
