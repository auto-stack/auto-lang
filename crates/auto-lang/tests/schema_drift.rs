//! Plan 435 P0 —— 内置组件四表漂移围栏(schema drift fence)。
//!
//! 来源:2026-08-23 漂移审计(scratch/schema_drift_audit.py,Plan 435 §1.1)的
//! Rust 化,常驻测试套。比对内置组件的全部事实表:
//!   1. `src/aura/schema.rs`                 验证声明(elements.insert)
//!   2. `src/ui_gen/vue.rs`                  Web 路径(components.insert + match tag 表)
//!   3. `src/ui/aura_view_builder.rs`        桌面路径(两张 match tag 派发表)
//!   4. `src/ui/render_support.rs`           iced 支持级(单张 match tag 表)
//!   5. `src/parser.rs`                      tag 特判(get_primary_prop 归类表)
//!   6. `src/a2ui/export.rs`                 tag → A2UI 互操作映射
//!   7. `src/a2ui/import.rs`                 A2UI → tag 反向映射(tag: "..." 字面量)
//! 另带 `schema/aura.at`(Plan 098 冻结孤儿,P1 重建)的对照维度。
//!
//! 围栏语义:**只拦新增漂移**。审计当日已知漂移冻结在
//! `tests/fixtures/schema_drift_baseline.txt`;出现 baseline 之外的新孤立项即红;
//! 漂移消除只打印提示(请顺手裁剪 baseline)。master 因此始终绿,P1-P3 逐项收编。
//! 同表重复 insert(如审计发现的 popover 两处)无 baseline 豁免,直接红。
//!
//! 生成/更新 baseline(提交前人工复核 diff,逐维度写明理由):
//! ```text
//! SCHEMA_DRIFT_UPDATE_BASELINE=1 cargo test -p auto-lang --test schema_drift
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// view_builder 里的语句级 `match tag {` 派发表数量(tracked + untracked)。
/// `let x = match tag {` 的小表(默认样式等)刻意不收。
const EXPECTED_VB_TABLES: usize = 2;
/// render_support 的 `get_support` 派发表数量。
const EXPECTED_RENDER_TABLES: usize = 1;
/// vue.rs 的语句级 `match tag {` 表数量(map_tag 原生映射 + PascalCase 归一)。
/// `match tag_lower.as_str() {` 这类小表刻意不收。
const EXPECTED_VUE_TABLES: usize = 2;
/// parser.rs 的语句级 `match tag {` 表数量(get_primary_prop 的 tag 归类)。
const EXPECTED_PARSER_TABLES: usize = 1;
/// a2ui/export.rs 的语句级 `match tag {` 表数量(tag → A2UIComponentBody)。
const EXPECTED_A2UI_EXPORT_TABLES: usize = 1;

fn repo_file(rel: &str) -> PathBuf {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    assert!(
        p.exists(),
        "源文件不存在: {} —— 本测试必须在仓库布局内运行",
        p.display()
    );
    p
}

fn read(rel: &str) -> String {
    fs::read_to_string(repo_file(rel)).expect("read source file")
}

fn is_ident_char(b: u8) -> bool {
    b == b'_' || b.is_ascii_alphanumeric()
}

/// 提取 `map.insert("tag", ...)` 的 tag 序表(保序,供重复检测)。
/// map 名要求精确匹配(前一字符不是标识符字符),排除
/// `shadcn_components_used.insert` 这类前缀撞名。
fn scan_insert_tags(src: &str, map_name: &str) -> Vec<String> {
    let needle = format!("{}.insert(\"", map_name);
    let mut out = Vec::new();
    for line in src.lines() {
        if let Some(pos) = line.find(&needle) {
            if pos > 0 && is_ident_char(line.as_bytes()[pos - 1]) {
                continue;
            }
            let rest = &line[pos + needle.len()..];
            if let Some(end) = rest.find('"') {
                out.push(rest[..end].to_string());
            }
        }
    }
    out
}

/// 行骨架:去掉行注释与字符串字面量**内容**(保留引号对占位),只留结构字符。
/// 括号深度与 `=>` 判定都用骨架,注释/文案里的 `{}`、`=>` 不再干扰。
fn skeleton(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    let mut in_str = false;
    while let Some(c) = chars.next() {
        if in_str {
            match c {
                '\\' => {
                    chars.next();
                }
                '"' => {
                    in_str = false;
                    out.push('"');
                }
                _ => {}
            }
            continue;
        }
        match c {
            '"' => {
                in_str = true;
                out.push('"');
            }
            '/' if chars.peek() == Some(&'/') => break,
            _ => out.push(c),
        }
    }
    out
}

/// 提取文件中所有行 trim 后恰为 `match tag {` 的派发表,返回每张表的 tag 集
/// (臂头的全部字符串字面量,含别名)。逐行扫描,用花括号/圆括号深度区分
/// 臂头与臂体:臂体里的字符串(shadcn 模块路径、说明文案)不会被误收。
fn scan_match_tables(src: &str) -> Vec<BTreeSet<String>> {
    let lines: Vec<&str> = src.lines().collect();
    let mut tables = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() != "match tag {" {
            i += 1;
            continue;
        }
        let mut tags = BTreeSet::new();
        let mut depth: i32 = 1;
        let mut paren: i32 = 0;
        let mut head: Vec<&str> = Vec::new();
        let mut in_body = false;
        i += 1;
        while i < lines.len() && depth > 0 {
            let line = lines[i];
            let skel = skeleton(line);
            let opens = skel.matches('{').count() as i32;
            let closes = skel.matches('}').count() as i32;
            let parens =
                skel.matches('(').count() as i32 - skel.matches(')').count() as i32;
            let trimmed = skel.trim();
            if !in_body {
                if line.trim().starts_with("//") {
                    i += 1;
                    continue;
                }
                depth += opens - closes;
                paren += parens;
                head.push(line);
                if skel.contains("=>") {
                    for h in &head {
                        // 臂头只可能是 `"tag" | "tag"` 模式;取首个 `=>` 前的原始文本
                        let before = h.split("=>").next().unwrap_or("");
                        for tag in quoted_strings(before) {
                            tags.insert(tag);
                        }
                    }
                    head.clear();
                    in_body = true;
                    if depth == 1 && paren == 0 && trimmed.ends_with(',') {
                        in_body = false; // 单行表达式臂:`"row" => self.f(...),`
                    }
                }
            } else {
                depth += opens - closes;
                paren += parens;
                let arm_done = depth == 1
                    && paren == 0
                    && (trimmed.ends_with(',') || trimmed.ends_with('}'));
                if arm_done {
                    in_body = false;
                }
            }
            i += 1;
        }
        tables.push(tags);
    }
    tables
}

/// 提取一段文本里的双引号字符串字面量(tag 不会含 `//`,不考虑转义)。
fn quoted_strings(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut j = 0;
    while j < bytes.len() {
        if bytes[j] == b'"' {
            if let Some(end) = text[j + 1..].find('"') {
                out.push(text[j + 1..j + 1 + end].to_string());
                j = j + 1 + end + 1;
            } else {
                break;
            }
        } else {
            j += 1;
        }
    }
    out
}

/// 提取 `tag: "xxx"` 字面量(a2ui/import.rs 的 enum → tag 反向映射形态)。
fn scan_tag_literals(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in src.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("tag: \"") {
            if let Some(end) = rest.find('"') {
                out.insert(rest[..end].to_string());
            }
        }
    }
    out
}

/// 提取 `schema/aura.at` 的 `element NAME {` 声明名表。
fn scan_aura_at(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in src.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("element ") {
            if let Some(name) = rest.strip_suffix('{') {
                out.insert(name.trim().to_string());
            }
        }
    }
    out
}

type Drift = BTreeMap<&'static str, BTreeSet<String>>;

fn add(drift: &mut Drift, dim: &'static str, items: BTreeSet<String>) {
    if !items.is_empty() {
        drift.entry(dim).or_default().extend(items);
    }
}

fn only_in(a: &BTreeSet<String>, b: &BTreeSet<String>) -> BTreeSet<String> {
    a.difference(b).cloned().collect()
}

#[test]
fn schema_drift_fence() {
    let schema_rs = read("src/aura/schema.rs");
    let vue_rs = read("src/ui_gen/vue.rs");
    let vb_rs = read("src/ui/aura_view_builder.rs");
    let render_rs = read("src/ui/render_support.rs");
    let aura_at = read("../../schema/aura.at");

    // ---- 提取四表 + 结构断言 ----
    let rs_seq = scan_insert_tags(&schema_rs, "elements");
    let mut rs: BTreeSet<String> = BTreeSet::new();
    let mut rs_dup: BTreeSet<String> = BTreeSet::new();
    for tag in &rs_seq {
        if !rs.insert(tag.clone()) {
            rs_dup.insert(tag.clone());
        }
    }
    let at = scan_aura_at(&aura_at);
    // vue.rs 的重复 insert 与 schema.rs 同罪:HashMap 后写覆盖,前者是死代码。
    let vue_seq = scan_insert_tags(&vue_rs, "components");
    let mut vue_shadcn: BTreeSet<String> = BTreeSet::new();
    let mut vue_dup: BTreeSet<String> = BTreeSet::new();
    for tag in &vue_seq {
        if !vue_shadcn.insert(tag.clone()) {
            vue_dup.insert(tag.clone());
        }
    }

    let vb_tables = scan_match_tables(&vb_rs);
    assert_eq!(
        vb_tables.len(),
        EXPECTED_VB_TABLES,
        "view_builder 语句级 `match tag {{` 派发表数量变化({} -> {}):新表若属组件派发,\
         请把它的维度加进本测试并更新 baseline;小表请改名避开 `match tag {{`",
        EXPECTED_VB_TABLES,
        vb_tables.len()
    );
    let (vb0, vb1) = (&vb_tables[0], &vb_tables[1]);
    let vb_union: BTreeSet<String> = vb0.union(vb1).cloned().collect();

    let render_tables = scan_match_tables(&render_rs);
    assert_eq!(
        render_tables.len(),
        EXPECTED_RENDER_TABLES,
        "render_support `match tag {{` 表数量变化({} -> {})",
        EXPECTED_RENDER_TABLES,
        render_tables.len()
    );
    let render = &render_tables[0];

    let vue_tables = scan_match_tables(&vue_rs);
    assert_eq!(
        vue_tables.len(),
        EXPECTED_VUE_TABLES,
        "vue.rs 语句级 `match tag {{` 表数量变化({} -> {}):新表若属 tag 归属决策,\
         请加维度并更新 baseline",
        EXPECTED_VUE_TABLES,
        vue_tables.len()
    );
    let (vue_mt0, vue_mt1) = (&vue_tables[0], &vue_tables[1]);

    let parser_rs = read("src/parser.rs");
    let parser_tables = scan_match_tables(&parser_rs);
    assert_eq!(
        parser_tables.len(),
        EXPECTED_PARSER_TABLES,
        "parser.rs 语句级 `match tag {{` 表数量变化({} -> {}):新表若属 tag 知识,\
         请把它的维度加进本测试并更新 baseline;小表请改名避开 `match tag {{`",
        EXPECTED_PARSER_TABLES,
        parser_tables.len()
    );
    let parser_tags = &parser_tables[0];

    let a2ui_export_rs = read("src/a2ui/export.rs");
    let a2ui_export_tables = scan_match_tables(&a2ui_export_rs);
    assert_eq!(
        a2ui_export_tables.len(),
        EXPECTED_A2UI_EXPORT_TABLES,
        "a2ui/export.rs `match tag {{` 表数量变化({} -> {})",
        EXPECTED_A2UI_EXPORT_TABLES,
        a2ui_export_tables.len()
    );
    let a2ui_export = &a2ui_export_tables[0];
    let a2ui_import = scan_tag_literals(&read("src/a2ui/import.rs"));

    println!("四表规模: schema.rs={} aura.at={} vue.shadcn={} vb0={} vb1={} render={}",
        rs.len(), at.len(), vue_shadcn.len(), vb0.len(), vb1.len(), render.len());

    // ---- 维度语义(新增漂移时随报错一起打印) ----
    let mut drift: Drift = BTreeMap::new();
    add(&mut drift, "rs_duplicate_insert", rs_dup);
    add(&mut drift, "vue_duplicate_insert", vue_dup);
    add(&mut drift, "at_not_in_rs", only_in(&at, &rs));
    add(&mut drift, "rs_not_in_at", only_in(&rs, &at));
    add(&mut drift, "vue_not_in_rs", only_in(&vue_shadcn, &rs));
    add(&mut drift, "rs_not_in_vue", only_in(&rs, &vue_shadcn));
    add(&mut drift, "vb_not_in_rs", only_in(&vb_union, &rs));
    add(&mut drift, "rs_not_in_vb", only_in(&rs, &vb_union));
    add(&mut drift, "vb0_not_in_vb1", only_in(vb0, vb1));
    add(&mut drift, "vb1_not_in_vb0", only_in(vb1, vb0));
    add(&mut drift, "render_not_in_rs", only_in(render, &rs));
    add(&mut drift, "rs_not_in_render", only_in(&rs, render));
    add(&mut drift, "vb_not_in_render", only_in(&vb_union, render));
    add(&mut drift, "render_not_in_vb", only_in(render, &vb_union));
    add(&mut drift, "vue_mt0_not_in_rs", only_in(vue_mt0, &rs));
    add(&mut drift, "vue_mt1_not_in_rs", only_in(vue_mt1, &rs));
    add(&mut drift, "parser_not_in_rs", only_in(parser_tags, &rs));
    add(&mut drift, "a2ui_export_not_in_rs", only_in(a2ui_export, &rs));
    add(&mut drift, "a2ui_import_not_in_rs", only_in(&a2ui_import, &rs));

    // ---- baseline:只拦新增 ----
    let baseline_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/schema_drift_baseline.txt");
    if std::env::var("SCHEMA_DRIFT_UPDATE_BASELINE").is_ok() {
        let mut text = String::from(
            "# Plan 435 P0 —— schema 漂移围栏 baseline(已知漂移白名单)\n\
             # 格式: 维度<TAB>tag,按字母序。围栏只拦 baseline 之外的新增漂移;\n\
             # 漂移消除后请顺手裁剪本文件。更新方式:\n\
             # SCHEMA_DRIFT_UPDATE_BASELINE=1 cargo test -p auto-lang --test schema_drift\n\
             # 约定:新增 baseline 条目必须在提交信息里写明理由(重生成会覆盖手动注释,\n\
             # 所以理由落在 commit message,不在本文件)。\n\n",
        );
        for (dim, tags) in &drift {
            for tag in tags {
                text.push_str(&format!("{}\t{}\n", dim, tag));
            }
        }
        fs::write(&baseline_path, text).expect("write baseline");
        panic!(
            "baseline 已重写 {} —— 复核 diff 后重跑(不带环境变量)确认绿",
            baseline_path.display()
        );
    }
    let baseline_text = fs::read_to_string(&baseline_path).unwrap_or_else(|_| {
        panic!(
            "baseline 缺失: {} —— 先跑一次:\n\
             SCHEMA_DRIFT_UPDATE_BASELINE=1 cargo test -p auto-lang --test schema_drift",
            baseline_path.display()
        )
    });
    let mut baseline: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for line in baseline_text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((dim, tag)) = line.split_once('\t') {
            baseline.entry(dim.to_string()).or_default().insert(tag.to_string());
        }
    }

    // 已消除的漂移:提示裁剪,不拦。
    let known_dims: BTreeSet<&str> = drift.keys().copied().collect();
    for (dim, tags) in &baseline {
        if !known_dims.contains(dim.as_str()) {
            println!("[schema-drift] baseline 维度 `{dim}` 已不在围栏中(计划推进后移除?),请裁剪");
            continue;
        }
        let current = &drift[dim.as_str()];
        for resolved in tags.difference(current) {
            println!("[schema-drift] 漂移已消除,请裁剪 baseline: {dim}\t{resolved}");
        }
    }

    // 新增漂移:红。
    let mut fresh: Vec<(String, Vec<String>)> = Vec::new();
    for (dim, current) in &drift {
        let allowed = baseline.get(*dim);
        let new_tags: Vec<String> = current
            .iter()
            .filter(|t| allowed.map_or(true, |a| !a.contains(*t)))
            .cloned()
            .collect();
        if !new_tags.is_empty() {
            fresh.push((dim.to_string(), new_tags));
        }
    }
    if !fresh.is_empty() {
        let mut msg = String::from(
            "Plan 435 P0 漂移围栏:发现 baseline 之外的新增漂移(四表不同步)。\n\
             维度语义:\n\
             - rs/vue_duplicate_insert: schema.rs / vue.rs 同一 tag insert 两次(HashMap 后写覆盖,前者是死代码)\n\
             - *_not_in_rs / rs_not_in_*: 声明表(schema.rs)与各实现表的孤儿 tag\n\
             - parser_not_in_rs: parser.rs tag 特判表里的孤儿(含 PascalCase 变体)\n\
             - a2ui_export/import_not_in_rs: A2UI 互操作映射的孤儿 tag\n\
             - vb0/vb1_not_in_vb1/vb0: view_builder 两张派发表不镜像(D-GAP 纪律)\n\
             - vb/render_not_in_render/vb: 桌面实现与 iced 支持级表不同步\n\
             - vue_mt0/1_not_in_rs: vue.rs map_tag 原生映射 / Pascal 归一表的孤儿\n\
             修复二选一:同步四表(推荐);或确属有意漂移,更新 baseline 并写明理由:\n\
             SCHEMA_DRIFT_UPDATE_BASELINE=1 cargo test -p auto-lang --test schema_drift\n\n",
        );
        for (dim, tags) in &fresh {
            msg.push_str(&format!("  [{}] {}\n", dim, tags.join(", ")));
        }
        panic!("{}", msg);
    }
}
