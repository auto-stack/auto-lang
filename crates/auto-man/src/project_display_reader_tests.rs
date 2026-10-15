//! PLAN-786 T-04: project_display_reader 永久对拍族（V01..V05）。
//!
//! 冻结旧 oracle（vue.rs 原 parse_pac_scalar/parse_pac_name 的逐字副本）
//! 仅存于此：旧实现不保留生产副本，行为对拍只在测试发生。

use crate::project_display_reader::{read, CompatReason, DisplayRead, ModelState, ReadPath};

// ---- 冻结旧 oracle ----

fn oracle_scalar(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with(&format!("{key}:")) {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_end_matches(',');
                let value = value.trim_matches('"').trim_matches('\'');
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

fn oracle_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name:") {
            if let Some(colon_pos) = line.find(':') {
                let value = line[colon_pos + 1..].trim();
                let value = value.trim_end_matches(',');
                let value = value.trim_matches('"').trim_matches('\'');
                return Some(value.to_string());
            }
        }
    }
    None
}

/// 三字段与冻结旧 oracle 逐值相等（reader == 旧实现行为，三方一致）。
fn assert_matches_oracle(case: &str, content: &str, r: &DisplayRead) {
    assert_eq!(
        r.name,
        oracle_name(content),
        "{case}: name 与旧 oracle 不一致（行为漂移）"
    );
    assert_eq!(
        r.title,
        oracle_scalar(content, "title"),
        "{case}: title 与旧 oracle 不一致（行为漂移）"
    );
    assert_eq!(
        r.title_zh,
        oracle_scalar(content, "title_zh"),
        "{case}: title_zh 与旧 oracle 不一致（行为漂移）"
    );
}

fn model(state: ModelState) -> ReadPath {
    ReadPath::Model(state)
}

fn compat(reason: CompatReason) -> ReadPath {
    ReadPath::Compat(reason)
}

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root")
}

fn display_fixture(name: &str) -> std::path::PathBuf {
    repo_root()
        .join("test/fixtures/modularity/project-model/display-reader")
        .join(name)
}

/// V01（AC01/02）：受管 pac.at 全语料——三字段 reader 与冻结旧 oracle
/// 逐值相等；Model/Compat 路径计数留档（收据，非断言语义）。
#[test]
fn display_reader_v01_managed_corpus_matches_frozen_oracle() {
    let out = std::process::Command::new("git")
        .args(["ls-files", "*.at"])
        .current_dir(repo_root())
        .output()
        .expect("git ls-files");
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let files: Vec<std::path::PathBuf> = text
        .lines()
        .map(|l| l.replace('\\', "/"))
        .filter(|l| l.ends_with("pac.at"))
        .map(|l| repo_root().join(l))
        .collect();
    assert!(
        files.len() >= 140,
        "expected the managed corpus (149 at PLAN-780 T-01), got {}",
        files.len()
    );
    let (mut model_hits, mut compat_hits) = (0usize, 0usize);
    for f in &files {
        let content = std::fs::read_to_string(f)
            .unwrap_or_else(|e| panic!("受管输入必须可读 {}: {e}", f.display()));
        let r = read(&content);
        assert_matches_oracle(&f.display().to_string(), &content, &r);
        for p in [r.name_path, r.title_path, r.title_zh_path] {
            match p {
                ReadPath::Model(_) => model_hits += 1,
                ReadPath::Compat(_) => compat_hits += 1,
            }
        }
    }
    eprintln!(
        "P786 V01 corpus: files={} field_reads={} model={} compat={}",
        files.len(),
        files.len() * 3,
        model_hits,
        compat_hits
    );
    // 受管语料以静态干净形态为主——Model 路径必须真实被使用（AC01:
    // DTO 路径实际被使用，而非全量落兼容臂）。
    assert!(
        model_hits >= files.len(),
        "Model 路径占比异常偏低：model={model_hits} compat={compat_hits}"
    );
}

/// V02（AC02）：冻结边界矩阵（T01 冻结表 §2 全行 + 变异控制）——期望值 +
/// 路径分类 + oracle 三方一致；未支持的输入不得假记 DTO 等价。
#[test]
fn display_reader_v02_frozen_edge_matrix() {
    struct Case {
        tag: &'static str,
        content: &'static str,
        name: Option<&'static str>,
        title: Option<&'static str>,
        title_zh: Option<&'static str>,
        name_path: ReadPath,
        title_path: ReadPath,
        title_zh_path: ReadPath,
    }
    let mv = ModelState::Value;
    let ma = ModelState::Absent;
    let me = ModelState::DeclaredEmpty;
    let cp = CompatReason::ParseError;
    let ch = CompatReason::HiddenOrPhantomLine;
    let cd = CompatReason::DuplicateKey;
    let cu = CompatReason::UnsupportedValue;
    let cl = CompatReason::LineFormMismatch;
    let cases = [
        // 1/2: 单字段与三字段干净形态 → DTO 直读
        Case {
            tag: "single-clean",
            content: "name: \"n\"\ntitle: \"X\"\ntitle_zh: \"中文\"\n",
            name: Some("n"),
            title: Some("X"),
            title_zh: Some("中文"),
            name_path: model(mv),
            title_path: model(mv),
            title_zh_path: model(mv),
        },
        // 3: 标题缺失 → 真实缺席
        Case {
            tag: "title-absent",
            content: "name: \"x\"\nversion: \"1\"\n",
            name: Some("x"),
            title: None,
            title_zh: None,
            name_path: model(mv),
            title_path: model(ma),
            title_zh_path: model(ma),
        },
        // 4: 重复 title → 旧首匹配 vs 模型 last-wins
        Case {
            tag: "dup-title",
            content: "name: \"d\"\ntitle: \"a\"\ntitle: \"b\"\n",
            name: Some("d"),
            title: Some("a"),
            title_zh: None,
            name_path: model(mv),
            title_path: compat(cd),
            title_zh_path: model(ma),
        },
        // 5: name 重复且首个空 → 旧首匹配 Some("")
        Case {
            tag: "dup-name-empty-first",
            content: "name: \"\"\nname: \"x\"\n",
            name: Some(""),
            title: None,
            title_zh: None,
            name_path: compat(cd),
            title_path: model(ma),
            title_zh_path: model(ma),
        },
        // 6: 单独空串 title（双引号/单引号）→ 旧跳空续扫无果 → None
        Case {
            tag: "empty-title",
            content: "name: \"e\"\ntitle: \"\"\n",
            name: Some("e"),
            title: None,
            title_zh: None,
            name_path: model(mv),
            title_path: model(me),
            title_zh_path: model(ma),
        },
        Case {
            tag: "empty-title-single-quoted",
            content: "name: \"e\"\ntitle: ''\n",
            name: Some("e"),
            title: None,
            title_zh: None,
            name_path: model(mv),
            title_path: model(me),
            title_zh_path: model(ma),
        },
        // 7: 空串在前、实值在后 → 旧跳空续扫取后者（重复键兼容）
        Case {
            tag: "empty-then-value",
            content: "title: \"\"\ntitle: \"x\"\n",
            name: None,
            title: Some("x"),
            title_zh: None,
            name_path: model(ma),
            title_path: compat(cd),
            title_zh_path: model(ma),
        },
        // 8: 单独 name 空串 → 旧 Some("")
        Case {
            tag: "empty-name",
            content: "name: \"\"\ntitle: \"t\"\n",
            name: Some(""),
            title: Some("t"),
            title_zh: None,
            name_path: model(mv),
            title_path: model(mv),
            title_zh_path: model(ma),
        },
        // 9/10: 引号内边缘空白——title 旧不 trim / name 两边一致
        Case {
            tag: "title-inner-ws",
            content: "title: \" X \"\n",
            name: None,
            title: Some(" X "),
            title_zh: None,
            name_path: model(ma),
            title_path: compat(cl),
            title_zh_path: model(ma),
        },
        Case {
            tag: "name-inner-ws",
            content: "name: \" n \"\n",
            name: Some(" n "),
            title: None,
            title_zh: None,
            name_path: model(mv),
            title_path: model(ma),
            title_zh_path: model(ma),
        },
        // 11: 嵌套同名独立行（dep 体）→ 行扫描无视结构
        Case {
            tag: "nested-own-line",
            content: "name: \"o\"\ndep i {\n    title: \"inner\"\n}\n",
            name: Some("o"),
            title: Some("inner"),
            title_zh: None,
            name_path: model(mv),
            title_path: compat(ch),
            title_zh_path: model(ma),
        },
        // 12: 嵌套同一行 → 双方一致看不见
        Case {
            tag: "nested-same-line",
            content: "dep x { title: \"i\" }\n",
            name: None,
            title: None,
            title_zh: None,
            name_path: model(ma),
            title_path: model(ma),
            title_zh_path: model(ma),
        },
        // 13: 行内注释 → 旧行归一保留全文
        Case {
            tag: "inline-comment",
            content: "title: \"x\" // note\n",
            name: None,
            title: Some("x\" // note"),
            title_zh: None,
            name_path: model(ma),
            title_path: compat(cl),
            title_zh_path: model(ma),
        },
        // 14: 单引号 → 旧接受
        Case {
            tag: "single-quoted",
            content: "title: 'sq'\n",
            name: None,
            title: Some("sq"),
            title_zh: None,
            name_path: model(ma),
            title_path: compat(cu),
            title_zh_path: model(ma),
        },
        // 15: 转义 → 旧保留反斜杠字面
        Case {
            tag: "escaped-quote",
            content: "title: \"a\\\"b\"\n",
            name: None,
            title: Some("a\\\"b"),
            title_zh: None,
            name_path: model(ma),
            title_path: compat(cl),
            title_zh_path: model(ma),
        },
        // 16: 插值 → 旧取原文不求值
        Case {
            tag: "interpolation",
            content: "title: \"a${b}c\"\n",
            name: None,
            title: Some("a${b}c"),
            title_zh: None,
            name_path: model(ma),
            title_path: compat(cu),
            title_zh_path: model(ma),
        },
        // 17: bare 值（title/name）
        Case {
            tag: "bare-title",
            content: "title: bare\n",
            name: None,
            title: Some("bare"),
            title_zh: None,
            name_path: model(ma),
            title_path: compat(cu),
            title_zh_path: model(ma),
        },
        Case {
            tag: "bare-name",
            content: "name: bare\n",
            name: Some("bare"),
            title: None,
            title_zh: None,
            name_path: compat(cu),
            title_path: model(ma),
            title_zh_path: model(ma),
        },
        // 18: 729 最小对象格式 → provenance 全记 `{` 行，gate 行不齐 → 兼容
        Case {
            tag: "object-doc",
            content: "{\n    name: \"objn\"\n    title: \"objt\"\n}\n",
            name: Some("objn"),
            title: Some("objt"),
            title_zh: None,
            name_path: compat(ch),
            title_path: compat(ch),
            title_zh_path: model(ma),
        },
        // 19: root 包裹体 → 兼容取嵌套值
        Case {
            tag: "root-wrapper",
            content: "root {\n    title: \"wrapped\"\n}\n",
            name: None,
            title: Some("wrapped"),
            title_zh: None,
            name_path: model(ma),
            title_path: compat(ch),
            title_zh_path: model(ma),
        },
        // 20: 块注释藏行 → 行扫描命中注释内行
        Case {
            tag: "block-comment-hidden",
            content: "/*\ntitle: \"c\"\n*/\n",
            name: None,
            title: Some("c"),
            title_zh: None,
            name_path: model(ma),
            title_path: compat(ch),
            title_zh_path: model(ma),
        },
        // 21: 跨行字符串吞行 → 被吞行仍可被行扫描命中
        Case {
            tag: "multiline-string-swallow",
            content: "note: \"aaa\ntitle: t\"\nname: \"n\"\n",
            name: Some("n"),
            title: Some("t"),
            title_zh: None,
            name_path: model(mv),
            title_path: compat(ch),
            title_zh_path: model(ma),
        },
        // 22: 冒号前空格 → 行门槛不过，旧 None；模型可见 → 走兼容保 None
        Case {
            tag: "space-before-colon",
            content: "name: \"n\"\ntitle : \"x\"\n",
            name: Some("n"),
            title: None,
            title_zh: None,
            name_path: model(mv),
            title_path: compat(ch),
            title_zh_path: model(ma),
        },
        // 23: 行尾逗号 → 双方一致
        Case {
            tag: "trailing-comma",
            content: "title: \"x\",\n",
            name: None,
            title: Some("x"),
            title_zh: None,
            name_path: model(ma),
            title_path: model(mv),
            title_zh_path: model(ma),
        },
        // 24: 引号内尾逗号 → 旧归一先剥逗号后剥引号，引号内逗号保留——
        // 旧与模型一致（oracle 修正：此处无分歧，Model 直读）
        Case {
            tag: "comma-in-quotes",
            content: "title: \"x,\"\n",
            name: None,
            title: Some("x,"),
            title_zh: None,
            name_path: model(ma),
            title_path: model(mv),
            title_zh_path: model(ma),
        },
        // 25: 前缀相似 key → 双方一致不误命中
        Case {
            tag: "prefix-key",
            content: "titlex: \"y\"\n",
            name: None,
            title: None,
            title_zh: None,
            name_path: model(ma),
            title_path: model(ma),
            title_zh_path: model(ma),
        },
        // 26: nil → astr 差异（模型空串）
        Case {
            tag: "name-nil",
            content: "name: nil\n",
            name: Some("nil"),
            title: None,
            title_zh: None,
            name_path: compat(cl),
            title_path: model(ma),
            title_zh_path: model(ma),
        },
        // 27: 数值 name → astr 谓词验证后 DTO 直读
        Case {
            tag: "name-numeric",
            content: "name: 123\n",
            name: Some("123"),
            title: None,
            title_zh: None,
            name_path: model(mv),
            title_path: model(ma),
            title_zh_path: model(ma),
        },
        // 28: f-string/调用/on-off → 原文片段
        Case {
            tag: "fstring",
            content: "title: `x`\n",
            name: None,
            title: Some("`x`"),
            title_zh: None,
            name_path: model(ma),
            title_path: compat(cu),
            title_zh_path: model(ma),
        },
        Case {
            tag: "call",
            content: "title: env(\"T\")\n",
            name: None,
            title: Some("env(\"T\")"),
            title_zh: None,
            name_path: model(ma),
            title_path: compat(cu),
            title_zh_path: model(ma),
        },
        Case {
            tag: "on-off",
            content: "title: on\n",
            name: None,
            title: Some("on"),
            title_zh: None,
            name_path: model(ma),
            title_path: compat(cu),
            title_zh_path: model(ma),
        },
        // 29: 赋值 → 双方一致 None
        Case {
            tag: "assignment",
            content: "title = \"x\"\n",
            name: None,
            title: None,
            title_zh: None,
            name_path: model(ma),
            title_path: model(ma),
            title_zh_path: model(ma),
        },
        // 30: 坏静态输入（尾随内容/缺值）→ 解析 Err 是全局前置：三字段全部
        // 走兼容臂（坏文件可能隐藏任意字段的可视性），兼容臂照旧行扫描
        Case {
            tag: "trailing-garbage",
            content: "title: \"x\" trailing\n",
            name: None,
            title: Some("x\" trailing"),
            title_zh: None,
            name_path: compat(cp),
            title_path: compat(cp),
            title_zh_path: compat(cp),
        },
        Case {
            tag: "missing-value",
            content: "name: \"n\"\ntitle:\n",
            name: Some("n"),
            title: None,
            title_zh: None,
            name_path: compat(cp),
            title_path: compat(cp),
            title_zh_path: compat(cp),
        },
        // 31: 引号内仅空白 → 非空判定在原文上，旧返回 Some("  ")
        Case {
            tag: "whitespace-only",
            content: "title: \"  \"\n",
            name: None,
            title: Some("  "),
            title_zh: None,
            name_path: model(ma),
            title_path: compat(cl),
            title_zh_path: model(ma),
        },
        // 变异控制：CRLF / 行首缩进 / 冒号后制表 / 行注释插行 —— 归一后仍 Model
        Case {
            tag: "crlf",
            content: "name: \"r\"\r\ntitle: \"T\"\r\n",
            name: Some("r"),
            title: Some("T"),
            title_zh: None,
            name_path: model(mv),
            title_path: model(mv),
            title_zh_path: model(ma),
        },
        Case {
            tag: "indented",
            content: "    name: \"i\"\n\ttitle: \"T\"\n",
            name: Some("i"),
            title: Some("T"),
            title_zh: None,
            name_path: model(mv),
            title_path: model(mv),
            title_zh_path: model(ma),
        },
        Case {
            tag: "tab-after-colon",
            content: "name: \"t\"\ntitle:\t\"T\"\n",
            name: Some("t"),
            title: Some("T"),
            title_zh: None,
            name_path: model(mv),
            title_path: model(mv),
            title_zh_path: model(ma),
        },
        Case {
            tag: "line-comments-interleaved",
            content: "// lead\nname: \"c\" // inline\n# hash too\ntitle: \"T\"\n",
            name: Some("c\" // inline"),
            title: Some("T"),
            title_zh: None,
            name_path: compat(cl),
            title_path: model(mv),
            title_zh_path: model(ma),
        },
    ];
    for case in &cases {
        let r = read(case.content);
        assert_matches_oracle(case.tag, case.content, &r);
        assert_eq!(
            r.name,
            case.name.map(str::to_string),
            "{}: name 值",
            case.tag
        );
        assert_eq!(
            r.title,
            case.title.map(str::to_string),
            "{}: title 值",
            case.tag
        );
        assert_eq!(
            r.title_zh,
            case.title_zh.map(str::to_string),
            "{}: title_zh 值",
            case.tag
        );
        assert_eq!(r.name_path, case.name_path, "{}: name 路径分类", case.tag);
        assert_eq!(
            r.title_path, case.title_path,
            "{}: title 路径分类",
            case.tag
        );
        assert_eq!(
            r.title_zh_path, case.title_zh_path,
            "{}: title_zh 路径分类",
            case.tag
        );
    }
}

/// V02 附：display-reader fixtures（8 个 .pac.at）逐文件冻结期望。
#[test]
fn display_reader_v02_fixture_expectations() {
    struct Fx {
        file: &'static str,
        name: Option<&'static str>,
        title: Option<&'static str>,
        title_zh: Option<&'static str>,
        title_path: ReadPath,
    }
    let mv = ModelState::Value;
    let ma = ModelState::Absent;
    let me = ModelState::DeclaredEmpty;
    let ch = CompatReason::HiddenOrPhantomLine;
    let cd = CompatReason::DuplicateKey;
    let cu = CompatReason::UnsupportedValue;
    let fx = [
        Fx {
            file: "three-fields.at",
            name: Some("demo-app"),
            title: Some("Demo App"),
            title_zh: Some("演示应用"),
            title_path: model(mv),
        },
        Fx {
            file: "missing-title.at",
            name: Some("no-title"),
            title: None,
            title_zh: None,
            title_path: model(ma),
        },
        Fx {
            file: "dup-title.at",
            name: Some("dup"),
            title: Some("first wins"),
            title_zh: None,
            title_path: compat(cd),
        },
        Fx {
            file: "single-quoted-title.at",
            name: Some("quoted"),
            title: Some("single quoted"),
            title_zh: None,
            title_path: compat(cu),
        },
        Fx {
            file: "nested-same-name.at",
            name: Some("outer"),
            title: Some("inner value"),
            title_zh: None,
            title_path: compat(ch),
        },
        Fx {
            file: "block-comment-hidden.at",
            name: Some("commented"),
            title: Some("hidden in block comment"),
            title_zh: None,
            title_path: compat(ch),
        },
        Fx {
            file: "dynamic-interp-title.at",
            name: Some("interp"),
            title: Some("v${version}"),
            title_zh: None,
            title_path: compat(cu),
        },
        Fx {
            file: "empty-title.at",
            name: Some("empty"),
            title: None,
            title_zh: None,
            title_path: model(me),
        },
    ];
    for f in &fx {
        let path = display_fixture(f.file);
        let content =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let r = read(&content);
        assert_matches_oracle(f.file, &content, &r);
        assert_eq!(r.name, f.name.map(str::to_string), "{}: name", f.file);
        assert_eq!(r.title, f.title.map(str::to_string), "{}: title", f.file);
        assert_eq!(
            r.title_zh,
            f.title_zh.map(str::to_string),
            "{}: title_zh",
            f.file
        );
        assert_eq!(r.title_path, f.title_path, "{}: title 路径分类", f.file);
    }
}

/// V03（AC02/03）：locale 链输入面（显式 bool，无 env 竞争）。链本体由既有
/// `pac_display_title_locale_chain`（vue.rs，显式 prefers_zh）覆盖，name/目录
/// 兜底由 `pac_title_overrides_document_title` 的 generate_index_html 断言覆盖
/// ——本测试只冻结 reader 字段层，不重复镜像链实现。
#[test]
fn display_reader_v03_locale_chain_explicit() {
    let three = std::fs::read_to_string(display_fixture("three-fields.at")).unwrap();
    let r = read(&three);
    assert_matches_oracle("three-fields", &three, &r);
    // zh（含 zh-CN 前缀族）→ title_zh 优先；en → 恒 title
    let zh = r.title_zh.clone().or_else(|| r.title.clone());
    let en = r.title.clone();
    assert_eq!(zh.as_deref(), Some("演示应用"));
    assert_eq!(en.as_deref(), Some("Demo App"));

    let missing = std::fs::read_to_string(display_fixture("missing-title.at")).unwrap();
    let r = read(&missing);
    assert_matches_oracle("missing-title", &missing, &r);
    // 链终点：无任何标题 → None（name/目录兜底是调用者职责，不在此伪装）
    assert_eq!(r.title_zh.clone().or_else(|| r.title.clone()), None);

    // 有 title_zh 无 title：zh 取中文、en 落空（链在调用者，字段层可判）
    let only_zh = read("name: \"z\"\ntitle_zh: \"仅中文\"\n");
    assert_matches_oracle("only-zh", "name: \"z\"\ntitle_zh: \"仅中文\"\n", &only_zh);
    assert_eq!(only_zh.title, None);
    assert_eq!(only_zh.title_zh.as_deref(), Some("仅中文"));
}

/// V04（AC02）：动态/坏输入——不执行 VM（模块零 auto-lang 依赖为结构性证明）、
/// 不写文件（输入字节不变）、动态字段不伪装静态（路径分类显式冻结）、
/// 10k 行输入有界完成（线性扫描的截止证明）。
#[test]
fn display_reader_v04_dynamic_bad_inputs_no_vm_no_writes() {
    let pm_fixture = |name: &str| {
        repo_root()
            .join("test/fixtures/modularity/project-model/fixtures")
            .join(name)
    };
    for name in [
        "dynamic-template.pac.at",
        "dynamic-expressions.pac.at",
        "dynamic-structured-values.pac.at",
        "dynamic-top-object-values.pac.at",
        "dynamic-root-wrapper.pac.at",
        "bad-missing-brace.pac.at",
        "bad-missing-value.pac.at",
    ] {
        let path = pm_fixture(name);
        let before = std::fs::read(&path).unwrap();
        let content = String::from_utf8(before.clone()).unwrap();
        let r = read(&content);
        assert_matches_oracle(name, &content, &r);
        let after = std::fs::read(&path).unwrap();
        assert_eq!(before, after, "{name}: 读取不得改动输入文件");
    }
    // 受影响字段的路径分类显式冻结：插值 title / root 包裹 title
    let interp = std::fs::read_to_string(display_fixture("dynamic-interp-title.at")).unwrap();
    let r = read(&interp);
    assert_matches_oracle("dynamic-interp", &interp, &r);
    assert_eq!(r.title_path, compat(CompatReason::UnsupportedValue));
    assert_eq!(
        r.title.as_deref(),
        Some("v${version}"),
        "插值取原文，不求值"
    );
    let wrapper = read("root {\n    title: \"w\"\n}\n");
    assert_eq!(
        wrapper.title_path,
        compat(CompatReason::HiddenOrPhantomLine)
    );
    // 截止控制：10k 行混合输入（门槛行在中段）有界完成且与 oracle 一致
    let mut big = String::with_capacity(200_000);
    for i in 0..10_000 {
        match i % 10 {
            0 => big.push_str(&format!("filler_{i}: \"v\"\n")),
            5 => big.push_str(&format!("// comment line {i}\n")),
            _ => big.push_str(&format!("prop_{i}: 1\n")),
        }
    }
    big.push_str("name: \"big\"\ntitle: \"Big\"\n");
    let r = read(&big);
    assert_matches_oracle("big", &big, &r);
    assert_eq!(r.name.as_deref(), Some("big"));
    assert_eq!(r.title.as_deref(), Some("Big"));
}

/// V05（AC01/03）：统一入口接线与非选字段不扩界——vue.rs 两个重复行循环已删、
/// 生产调用点走新入口；toggle 族与其余文本副本（rust_ui/vscode/ark/jet）原样。
#[test]
fn display_reader_v05_unified_entry_and_unselected_fields_intact() {
    let read_src = |rel: &str| {
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel))
            .unwrap_or_else(|e| panic!("{rel}: {e}"))
    };
    let vue = read_src("src/vue.rs");
    assert!(
        !vue.contains("fn parse_pac_scalar"),
        "vue.rs 重复行循环 parse_pac_scalar 必须删除"
    );
    assert!(
        !vue.contains("fn parse_pac_name"),
        "vue.rs 重复行循环 parse_pac_name 必须删除"
    );
    assert!(
        vue.contains("project_display_reader::read(&pac_content)"),
        "生产调用点必须走统一入口"
    );
    // toggle 族/front 路径行扫描（本计划非选字段）保留在 vue.rs
    assert!(
        vue.contains("fn parse_shadcn"),
        "parse_shadcn 非本轮范围，不得移除"
    );
    assert!(
        vue.contains("fn parse_default_classes"),
        "parse_default_classes 非本轮范围，不得移除"
    );
    assert!(
        vue.contains("fn parse_workspace_path"),
        "parse_workspace_path 非本轮范围，不得移除"
    );
    // 其余仓内文本副本不偷换（各有独立调用面，未纳入本轮收口）
    for (rel, needle) in [
        ("src/rust_ui.rs", "fn parse_pac_name"),
        ("src/rust_ui.rs", "fn parse_pac_title"),
        ("src/rust_ui.rs", "fn parse_pac_exe_name"),
        ("src/vscode.rs", "fn parse_pac_name"),
        ("src/ark.rs", "fn parse_pac_name"),
        ("src/jet.rs", "fn parse_pac_name"),
    ] {
        let src = read_src(rel);
        assert!(
            src.contains(needle),
            "{rel} 的 {needle} 不在本轮范围，不得移除"
        );
    }
    // 兼容扫描唯一副本：vue.rs 不得残留第二份实现
    assert_eq!(
        vue.matches("legacy_scan").count(),
        0,
        "vue.rs 不得保留第二份兼容扫描实现"
    );
    let reader = read_src("src/project_display_reader.rs");
    assert!(
        reader.contains("fn legacy_scan"),
        "兼容臂唯一冻结副本在 reader"
    );
}

/// R1 F786-R1-01 复现输入构造：无关字段携带 N 层嵌套数组 + 干净三字段行。
fn deep_nesting_input(depth: usize) -> String {
    let mut s = String::from("name: \"n\"\ntitle: \"ok\"\nextra: ");
    for _ in 0..depth {
        s.push('[');
    }
    s.push('0');
    for _ in 0..depth {
        s.push(']');
    }
    s.push('\n');
    s
}

/// R2 F786-R2-01 复现输入构造：跨行字符串（真实 lexer 跨 LF 直到闭引号）
/// 后接 N 层嵌套数组——R1 前置筛因"遇 LF 断"欠计数绕过包络的精确形态。
fn multiline_deep_input(depth: usize) -> String {
    let mut s = String::from("name: \"n\"\ntitle: \"ok\"\nextra: [\"hello\nworld\", ");
    for _ in 0..depth {
        s.push('[');
    }
    s.push('0');
    for _ in 0..depth {
        s.push(']');
    }
    s.push_str("]\n");
    s
}

/// 子进程探针结果。`TimedOut` 携带被 kill+wait 回收后的状态——证明
/// 截止路径终结并回收了子进程（R2 F786-R2-02）。
#[derive(Debug)]
enum ProbeOutcome {
    Completed {
        success: bool,
        diagnostics: String,
    },
    TimedOut {
        reaped_success: bool,
        diagnostics: String,
    },
}

/// 子进程生命周期守护：任何退出路径（成功/失败/超时/轮询错误/中途
/// panic 的 Drop 兜底）都保证 kill + wait 回收；管道诊断被收集并保留。
struct ChildGuard {
    child: Option<std::process::Child>,
}

impl ChildGuard {
    /// 显式终结并回收，返回 (状态成功性, 诊断)。幂等：取走后为空。
    fn terminate_and_reap(&mut self) -> (bool, String) {
        let mut child = match self.child.take() {
            Some(c) => c,
            None => return (false, String::new()),
        };
        let _ = child.kill();
        let status = child.wait().expect("reap killed child");
        let mut out = child.stdout.take().map(|mut s| {
            use std::io::Read;
            let mut buf = String::new();
            let _ = s.read_to_string(&mut buf);
            buf
        });
        let mut err = child.stderr.take().map(|mut s| {
            use std::io::Read;
            let mut buf = String::new();
            let _ = s.read_to_string(&mut buf);
            buf
        });
        let diagnostics = format!(
            "status={status} stdout={} stderr={}",
            out.take().unwrap_or_default(),
            err.take().unwrap_or_default()
        );
        (status.success(), diagnostics)
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        // 兜底：panic/提前返回路径同样终结+回收（take 后为空则无事可做）
        if self.child.is_some() {
            self.terminate_and_reap();
        }
    }
}

/// 以外部截止运行自执行子探针：截止保护的是**子进程生命周期**——超时
/// 即 kill+wait 回收并报告回收后状态，而非只断言了事（R2 F786-R2-02）。
/// 轮询错误路径同样终结+回收后以 Completed(false) 报告。
fn run_child_probe(
    test_name: &str,
    env_key: &str,
    env_value: &str,
    deadline_secs: u64,
) -> ProbeOutcome {
    use std::io::Read;
    use std::process::{Command, Stdio};
    let exe = std::env::current_exe().expect("test exe");
    let child = Command::new(exe)
        .args(["--exact", test_name, "--nocapture"])
        .env(env_key, env_value)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn probe child");
    let mut guard = ChildGuard { child: Some(child) };
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(deadline_secs);
    loop {
        let poll = guard
            .child
            .as_mut()
            .expect("guard holds child")
            .try_wait()
            .map_err(|e| e.to_string());
        match poll {
            Ok(Some(status)) => {
                // 正常回收路径：取走子进程，读尽管道诊断（进程已退出，读不阻塞）
                let mut child = guard.child.take().expect("child present");
                let mut buf = String::new();
                if let Some(s) = child.stdout.as_mut() {
                    let _ = s.read_to_string(&mut buf);
                }
                let mut errbuf = String::new();
                if let Some(s) = child.stderr.as_mut() {
                    let _ = s.read_to_string(&mut errbuf);
                }
                return ProbeOutcome::Completed {
                    success: status.success(),
                    diagnostics: format!("status={status} stdout={buf} stderr={errbuf}"),
                };
            }
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    let (reaped_success, diagnostics) = guard.terminate_and_reap();
                    return ProbeOutcome::TimedOut {
                        reaped_success,
                        diagnostics,
                    };
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                let (reaped_success, diagnostics) = guard.terminate_and_reap();
                return ProbeOutcome::Completed {
                    success: false,
                    diagnostics: format!("poll error: {e}; {diagnostics}"),
                };
            }
        }
    }
}

/// V04 补充（R1 F786-R1-01 + R2 F786-R2-01）：超深嵌套的进程隔离回归。
/// 栈溢出 abort 不可被 catch_unwind 捕获，只能以子进程验证——child 模式
/// 跑真实 read()（平面 512/1024 + 跨行字符串 512 反例原样重放），parent
/// 以 10s 外部截止守护子进程生命周期（超时 kill+reap，非仅断言）。
#[test]
fn display_reader_v04_deep_nesting_process_isolated() {
    if let Ok(mode) = std::env::var("P786_PROBE_MODE") {
        // child 模式：三种反例形态原样重放，必须返回旧 oracle 的标题且不崩溃
        if mode == "deep" {
            for input in [
                deep_nesting_input(512),
                deep_nesting_input(1024),
                multiline_deep_input(512),
            ] {
                let r = read(&input);
                assert_eq!(r.name.as_deref(), Some("n"));
                assert_eq!(r.title.as_deref(), Some("ok"));
                assert_eq!(r.title_path, compat(CompatReason::DeepNesting));
            }
            std::process::exit(0);
        }
        if mode == "hang" {
            // 挂起控制：故意睡 30s 超过父侧 2s 截止——证明截止路径终结+回收
            std::thread::sleep(std::time::Duration::from_secs(30));
            std::process::exit(0);
        }
    }
    // parent：深度探针 10s 截止
    match run_child_probe(
        "project_display_reader_tests::display_reader_v04_deep_nesting_process_isolated",
        "P786_PROBE_MODE",
        "deep",
        10,
    ) {
        ProbeOutcome::Completed {
            success: true,
            diagnostics: _,
        } => {}
        other => panic!("深度探针子进程失败（栈溢出 abort/断言失败/超时被回收）：{other:?}"),
    }
    // 挂起控制：故意睡 30s 的子进程 + 2s 截止——必须 TimedOut，且回收
    // 状态为被 kill 的失败态（证明截止路径终结+回收，而非其自行退出）。
    match run_child_probe(
        "project_display_reader_tests::display_reader_v04_deep_nesting_process_isolated",
        "P786_PROBE_MODE",
        "hang",
        2,
    ) {
        ProbeOutcome::TimedOut {
            reaped_success: false,
            diagnostics: _,
        } => {}
        other => panic!("挂起控制必须超时并被 kill+reap（非成功回收态）：{other:?}"),
    }
}

/// V04 补充（R1 F786-R1-01 同族状态）：包络边界与感知规则——64 层内仍走
/// 模型路径；65 层整体兼容；字符串/注释内括号不计深度；节点嵌套同理。
#[test]
fn display_reader_v04_deep_nesting_boundary_and_awareness() {
    let mv = ModelState::Value;
    // 边界：64 层（含干净 title）→ 模型路径可用；65 层 → DeepNesting 兼容
    let within = deep_nesting_input(64);
    let r = read(&within);
    assert_matches_oracle("depth-64", &within, &r);
    assert_eq!(r.title.as_deref(), Some("ok"));
    assert_eq!(r.title_path, model(mv), "包络内不收缩模型路径");
    let beyond = deep_nesting_input(65);
    let r = read(&beyond);
    assert_matches_oracle("depth-65", &beyond, &r);
    assert_eq!(r.title.as_deref(), Some("ok"), "兼容臂保旧行为");
    assert_eq!(r.title_path, compat(CompatReason::DeepNesting));
    assert_eq!(r.name_path, compat(CompatReason::DeepNesting));
    // 节点嵌套（{} 族）同因：512 层 dep 包裹 + 干净 name
    let mut nested_nodes = String::from("name: \"nn\"\n");
    for i in 0..512 {
        nested_nodes.push_str(&format!("n{i} {{\n"));
    }
    for _ in 0..512 {
        nested_nodes.push_str("}\n");
    }
    let r = read(&nested_nodes);
    assert_matches_oracle("nodes-512", &nested_nodes, &r);
    assert_eq!(r.name.as_deref(), Some("nn"));
    assert_eq!(r.name_path, compat(CompatReason::DeepNesting));
    // 感知规则：字符串/行注释/块注释内的括号不计深度——正常输入不受包络影响
    let quoted = "name: \"q\"\ntitle: \"[[[[[[\"\n";
    let r = read(quoted);
    assert_matches_oracle("brackets-in-string", quoted, &r);
    assert_eq!(r.title_path, model(mv), "字符串内括号不计数");
    let commented = "name: \"cm\"\n// [[[[[\n/* ]]]] */\ntitle: \"T\"\n";
    let r = read(commented);
    assert_matches_oracle("brackets-in-comments", commented, &r);
    assert_eq!(r.title_path, model(mv), "注释内括号不计数");

    // R2 F786-R2-01：跨行字符串语义——真实 lexer 跨 LF 直到闭引号，前置筛
    // 必须过近似（遇 LF 断会把闭引号误判为开引号、藏掉同行真实括号）。
    // 32 层（包络内，解析成功）→ 准入控制：title 仍 Model(Value)。
    let ml32 = multiline_deep_input(32);
    let r = read(&ml32);
    assert_matches_oracle("multiline-32", &ml32, &r);
    assert_eq!(r.title.as_deref(), Some("ok"));
    assert_eq!(r.title_path, model(mv), "包络内跨行字符串不收缩模型路径");
    // 65 层（跨行字符串后）→ DeepNesting：R2 反例形态在包络判定下正确路由
    let ml65 = multiline_deep_input(65);
    let r = read(&ml65);
    assert_matches_oracle("multiline-65", &ml65, &r);
    assert_eq!(r.title.as_deref(), Some("ok"));
    assert_eq!(r.title_path, compat(CompatReason::DeepNesting));
    // 512 层（R2 反例原量级）进程内安全（前置筛拦截，不触达解析递归）
    let ml512 = multiline_deep_input(512);
    let r = read(&ml512);
    assert_matches_oracle("multiline-512", &ml512, &r);
    assert_eq!(r.title_path, compat(CompatReason::DeepNesting));
    // CRLF 跨行字符串 + 深数组：串内 \r\n 不闭串
    let crlf = format!("name: \"c\"\ntitle: \"ok\"\nextra: [\"a\r\nb\", {}]\n", {
        let mut inner = String::new();
        for _ in 0..80 {
            inner.push('[');
        }
        inner.push('0');
        for _ in 0..80 {
            inner.push(']');
        }
        inner
    });
    let r = read(&crlf);
    assert_matches_oracle("crlf-multiline-80", &crlf, &r);
    assert_eq!(r.title_path, compat(CompatReason::DeepNesting));
    // 转义/引号转换：串内 \" 不闭串、串内转义的 \\n 与真实 LF 混排
    let escaped = format!(
        "name: \"e\"\ntitle: \"ok\"\nextra: [\"x\\\"y\\nz\nw\", {}]\n",
        {
            let mut inner = String::new();
            for _ in 0..70 {
                inner.push('{');
            }
            inner.push('0');
            for _ in 0..70 {
                inner.push('}');
            }
            inner
        }
    );
    let r = read(&escaped);
    assert_matches_oracle("escape-transitions-70", &escaped, &r);
    assert_eq!(r.title_path, compat(CompatReason::DeepNesting));
    // 单引号跨行（过近似最宽解释）+ 深数组 → 同样计入
    let sq = format!("name: \"s\"\ntitle: \"ok\"\nextra: ['ab\ncd', {}]\n", {
        let mut inner = String::new();
        for _ in 0..70 {
            inner.push('[');
        }
        inner.push('0');
        for _ in 0..70 {
            inner.push(']');
        }
        inner
    });
    let r = read(&sq);
    assert_matches_oracle("single-quote-multiline-70", &sq, &r);
    assert_eq!(r.title_path, compat(CompatReason::DeepNesting));
    // 未终止字符串吞剩余文本：欠计数不发生（真实 lexer 先 Err，无递归）
    let unterminated = "name: \"u\"\ntitle: \"ok\"\nextra: [\"abc\n[[[[[[[[\n";
    let r = read(unterminated);
    assert_matches_oracle("unterminated-string", unterminated, &r);
    assert_eq!(r.title.as_deref(), Some("ok"));
    assert_eq!(
        r.title_path,
        compat(CompatReason::ParseError),
        "未终止组先 Err（安全路径），不得 abort"
    );
}
