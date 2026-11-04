//! PLAN-929（T-10）：重复 parser 退役结构门（V-11）。
//!
//! 门1（编译期不可达）由 Cargo feature 承载：旧手写整源/片段 parser 与
//! 词法器仅在 `legacy-syntax-oracle`（测试自依赖）下编译——生产消费者
//! 构建零该 feature（ace 工作区 check 全绿即证）。
//! 门2（改名复制检测）：消费 crate 不得新增自有 tokenize/Parser/手写
//! 解析签名（白名单 = core 旧面 + 不同领域语法清单）。
//! 门3（负控自证）：本测试在 feature-on 的测试构建里证明 oracle 面仍
//! 可达（差分 harness 依赖它），而生产路径不经过它（T-01..T-09 迁移后
//! 调用清单零生产引用——由 ace-source/ace-av/ace-a2x/ace-native 组回归
//! 承载）。

use std::path::PathBuf;

/// 门2：消费 crate 源面不得出现第二套 Auto 源码 tokenizer/parser 定义。
#[test]
fn retirement_no_duplicate_parser_definitions_in_consumers() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let consumers = [
        "crates/av/src",
        "crates/native/src",
        "crates/a2x-store/src",
        "crates/a2x-worker/src",
        "crates/semantics/src",
        "crates/comptime/src",
        "crates/comptime-core/src",
        "crates/case-worker/src",
        "crates/syntax-core/src",
    ];
    // 允许的不同领域/共享面（非 Auto 源码语法）：atom 文本、语料工具、
    /// 共享前门 re-export 缝、CT 请求 DSL、argv 面。
    let allow_files = [
        "atom", // atom_text/atom 面（不同语言）
        "deck",     // CT deck DSL
        "request",  // comptime-contract 迷你 parser（本 crate 不在扫描面）
        "syntax",   // syntax-* 共享前门/运行时（语法基础设施本体）
    ];
    let mut offenders: Vec<String> = Vec::new();
    for c in consumers {
        let dir = root.join(c);
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        stack_walk(&dir, &mut |path| {
            let rel = path.strip_prefix(&root).unwrap_or(path).to_string_lossy().to_string();
            if allow_files.iter().any(|a| rel.contains(a)) {
                return;
            }
            let Ok(text) = std::fs::read_to_string(path) else { return };
            for (i, line) in text.lines().enumerate() {
                let l = line.trim();
                if l.starts_with("fn tokenize(")
                    || l.starts_with("pub fn tokenize(")
                    || l.starts_with("struct Parser {")
                    || l.starts_with("pub struct Parser {")
                    || l.starts_with("fn parse_program(")
                    || l.starts_with("pub fn parse_program(")
                {
                    offenders.push(format!("{rel}:{}: {}", i + 1, l));
                }
            }
        });
    }
    assert!(
        offenders.is_empty(),
        "消费面出现第二套 Auto 源码解析定义（改名复制门）:\n{}",
        offenders.join("\n")
    );
}

/// 门3：oracle 面 feature-on 可达（差分 harness 的依赖面在位）。
#[test]
fn retirement_oracle_surface_reachable_for_differential() {
    // 测试构建经 dev 自依赖开启 `legacy-syntax-oracle`——编译通过即证
    // oracle 面在位（同刻生产构建零该 feature：见本文件头部门1说明与
    /// ace 工作区 check 收据）。
    let _ = ace_core::source::parser_is_oracle_gated();
}

fn stack_walk(dir: &std::path::Path, f: &mut impl FnMut(&std::path::Path)) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            stack_walk(&p, f);
        } else if p.extension().map(|e| e == "rs").unwrap_or(false) {
            f(&p);
        }
    }
}
