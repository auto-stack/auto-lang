// Plan 432 S4 / M4 闸门:AAVM v2 codegen 与 Rust codegen 的字节码结构级一致。
//
// 语料:test/vm/aavm2/corpus_m4/*.at(fn-only 程序:wrapper/let/assign/if/
// while/for-range/fn 调用/字符串/逻辑/多局部)。
// 判据:两侧反汇编文本逐行相等 —— Rust 侧:execute 管线编译段(parse→
// codegen 脚本 wrapper→HALT)+ 单模块链接 + 反汇编;AAVM 侧:auto/lib/
// {token,lexer,parser,typeinfo,codegen}.at 的 codegen_dump(source)。
// 规范化(计划允许的元数据差异):①load.str 操作数显示池内容(Rust {:?}
// 转义;corpus 限 ASCII 简单串);②fn 末尾/作用域尾的槽释放组
// (push.nil+store)按槽位排序 —— Rust pop_scope 按 HashMap 迭代序发射,
// 跨进程不定。
// 格式规格:docs/specs/aavm/m4-bytecode-format.md(S4 前置考古落盘)。

use crate::error::AutoResult;
use crate::run_with_capture;
use crate::vm::codegen::Codegen;
use crate::vm::loader::{Linker, Module};
use crate::vm::opcode::OpCode;
use std::path::PathBuf;

fn escape_for_at_literal(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// 镜像 execute_autovm_with_path 的编译段 + 单模块链接。
fn compile_and_link(code: &str) -> AutoResult<(Vec<u8>, Vec<Vec<u8>>)> {
    let mut parser = crate::parser::Parser::new(code);
    let ast = parser.parse()?;
    let mut codegen = Codegen::new_with_type_store(parser.type_store.clone());
    let (type_decls, other_stmts): (Vec<_>, Vec<_>) = ast
        .stmts
        .iter()
        .partition(|stmt| {
            matches!(
                stmt,
                crate::ast::Stmt::TypeDecl(_) | crate::ast::Stmt::Ext(_) | crate::ast::Stmt::EnumDecl(_)
            )
        });
    for stmt in &type_decls {
        codegen.compile_stmt(stmt)?;
    }
    if !other_stmts.is_empty() {
        codegen.emit_op(OpCode::FN_PROLOG);
        codegen.emit_byte(0);
        codegen.emit_byte(16);
        codegen.emit_op(OpCode::RESERVE_STACK);
        codegen.emit_byte(16);
        for stmt in &other_stmts {
            codegen.compile_stmt(stmt)?;
        }
    }
    codegen.code.push(OpCode::HALT as u8);

    let mut linker = Linker::new();
    linker.add_module(Module {
        name: "__main__".to_string(),
        code: codegen.code.clone(),
        exports: codegen.exports.clone(),
        relocs: codegen.relocs.clone(),
        strings: codegen.strings.clone(),
        object_keys: codegen.object_keys.clone(),
        object_types: codegen.object_types.clone(),
        has_globals: false,
    });
    let (final_code, _symbols) = linker.link()
        .map_err(|e| crate::error::AutoError::Msg(e.message.clone()))?;
    Ok((final_code, codegen.strings.clone()))
}

/// 规范化反汇编:load.str 显内容;连续 (push.nil + store) 释放组按槽位排序。
fn normalized_dump(code: &[u8], strings: &[Vec<u8>]) -> String {
    let flash = crate::vm::virt_memory::VirtualFlash::new_with_code(code.to_vec());
    let dis = crate::vm::disasm::Disassembler::new(&flash);
    let lines: Vec<(usize, String, String)> = dis
        .disassemble_range(0, code.len())
        .into_iter()
        .map(|l| (l.offset, l.mnemonic.to_string(), l.operands.clone()))
        .collect();

    let mut out_lines: Vec<(usize, String, String)> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        // 检测释放组:连续的 (push.nil, store*) 对,按槽位排序组内顺序
        if lines[i].1 == "push.nil" && i + 1 < lines.len() && is_release_store(&lines[i + 1].1) {
            // 每对携带自身 offset;按槽位序输出(store 与其 push.nil 保持原 offset)
            let mut pairs: Vec<(usize, usize, usize, (usize, String, String))> = Vec::new();
            // (slot, push_off, store_off, store_line)
            let mut j = i;
            while j + 1 < lines.len() && lines[j].1 == "push.nil" && is_release_store(&lines[j + 1].1) {
                pairs.push((
                    slot_of(&lines[j + 1].1, &lines[j + 1].2),
                    lines[j].0,
                    lines[j + 1].0,
                    lines[j + 1].clone(),
                ));
                j += 2;
            }
            pairs.sort_by_key(|(s, _, _, _)| *s);
            // offset 按规范尺寸重算:push(1B)+store(loc.0/1=1B,local=2B),
            // 槽位升序 —— 消除 HashMap 乱序与 2B store 布局漂移的元数据差异
            let mut off = pairs.iter().map(|(_, p, _, _)| *p).min().unwrap_or(0);
            for (_slot, _poff, _soff, st) in pairs.iter() {
                out_lines.push((off, "push.nil".to_string(), String::new()));
                out_lines.push((off + 1, st.1.clone(), st.2.clone()));
                off += 1 + if st.1 == "store.local" { 2 } else { 1 };
            }
            i = j;
            continue;
        }
        let (off, mn, ops) = lines[i].clone();
        let ops = if mn == "load.str" {
            // operands 形如 "str[N]"
            let idx = ops
                .strip_prefix("str[")
                .and_then(|r| r.strip_suffix(']'))
                .and_then(|r| r.parse::<usize>().ok())
                .unwrap_or(0);
            format!("{:?}", String::from_utf8_lossy(strings.get(idx).map(|b| b.as_slice()).unwrap_or(b"")))
        } else {
            ops
        };
        out_lines.push((off, mn, ops));
        i += 1;
    }

    let mut out = String::new();
    for (off, mn, ops) in out_lines {
        out.push_str(&format!("{:04x}  {} {}\n", off, mn, ops));
    }
    out
}

fn is_release_store(mnemonic: &str) -> bool {
    mnemonic == "store.loc.0" || mnemonic == "store.loc.1" || mnemonic == "store.local"
}

fn slot_of(mnemonic: &str, operands: &str) -> usize {
    match mnemonic {
        "store.loc.0" => 0,
        "store.loc.1" => 1,
        _ => operands.trim().parse::<usize>().unwrap_or(usize::MAX),
    }
}

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/vm/aavm2/corpus_m4")
}

fn test_m4_corpus_file(path: &std::path::Path) -> AutoResult<()> {
    let code = std::fs::read_to_string(path)?;
    let (linked, strings) = compile_and_link(&code)?;
    let expected = normalized_dump(&linked, &strings);
    // 前置拼接 AAVM v2 lib(AUTO_LIB_FILES_V2,单一事实源)
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let mut lib_code = String::new();
    for f in crate::AUTO_LIB_FILES_V2 {
        lib_code.push_str(&std::fs::read_to_string(root.join(f))?);
        lib_code.push('\n');
    }
    let program = format!(
        "{}\nfn main() {{\n    print(codegen_dump(\"{}\"))\n}}\n",
        lib_code,
        escape_for_at_literal(&code)
    );
    let (_r, stdout) = run_with_capture(&program)?;
    if stdout.trim_end() != expected.trim_end() {
        // 失败现场:打印原始(未归一)反汇编,定位组形态
        let flash = crate::vm::virt_memory::VirtualFlash::new_with_code(linked.clone());
        let dis = crate::vm::disasm::Disassembler::new(&flash);
        eprintln!("=== RAW {} ===", path.display());
        for l in dis.disassemble_range(0x30, linked.len()) {
            eprintln!("RAW {:04x}  {} {}", l.offset, l.mnemonic, l.operands);
        }
    }
    assert_eq!(
        stdout.trim_end(),
        expected.trim_end(),
        "M4 bytecode mismatch for {}\n--- rust ---\n{}\n--- aavm ---\n{}",
        path.display(),
        expected,
        stdout
    );
    Ok(())
}

#[test]
fn test_aavm2_m4_codegen_corpus() {
    let dir = corpus_dir();
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("corpus dir {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "at").unwrap_or(false))
        .collect();
    entries.sort();
    assert!(!entries.is_empty(), "no corpus files under {}", dir.display());
    let mut checked = 0;
    for p in entries {
        test_m4_corpus_file(&p).unwrap();
        checked += 1;
    }
    eprintln!("M4 corpus: {checked} files, bytecode identical");
}


// ── Plan 511 W3:corpus_use 多文件腿(Rust 侧镜像 resolve_uses+Linker)──

fn corpus_use_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/vm/aavm2/corpus_use")
}

/// DisasmLine 流的规范化(镜像 normalized_dump:load.str 显池内容 +
/// 释放组按槽位排序重排 offset)。多文件腿的 Rust 参考口径。
fn normalized_dump_lines(lines: Vec<crate::vm::disasm::DisasmLine>, strings: &[String]) -> String {
    let lines: Vec<(usize, String, String)> = lines
        .into_iter()
        .map(|l| (l.offset, l.mnemonic.to_string(), l.operands.clone()))
        .collect();
    let mut out_lines: Vec<(usize, String, String)> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].1 == "push.nil" && i + 1 < lines.len() && is_release_store(&lines[i + 1].1) {
            let mut pairs: Vec<(usize, usize, usize, (usize, String, String))> = Vec::new();
            let mut j = i;
            while j + 1 < lines.len() && lines[j].1 == "push.nil" && is_release_store(&lines[j + 1].1) {
                pairs.push((
                    slot_of(&lines[j + 1].1, &lines[j + 1].2),
                    lines[j].0,
                    lines[j + 1].0,
                    lines[j + 1].clone(),
                ));
                j += 2;
            }
            pairs.sort_by_key(|(s, _, _, _)| *s);
            let mut off = pairs.iter().map(|(_, p, _, _)| *p).min().unwrap_or(0);
            for (_slot, _poff, _soff, st) in pairs.iter() {
                out_lines.push((off, "push.nil".to_string(), String::new()));
                out_lines.push((off + 1, st.1.clone(), st.2.clone()));
                off += 1 + if st.1 == "store.local" { 2 } else { 1 };
            }
            i = j;
            continue;
        }
        let (off, mn, ops) = lines[i].clone();
        let ops = if mn == "load.str" {
            let idx = ops
                .strip_prefix("str[")
                .and_then(|r| r.strip_suffix(']'))
                .and_then(|r| r.parse::<usize>().ok())
                .unwrap_or(0);
            format!("{:?}", strings.get(idx).map(|s| s.as_str()).unwrap_or(""))
        } else {
            ops
        };
        out_lines.push((off, mn, ops));
        i += 1;
    }
    let mut out = String::new();
    for (off, mn, ops) in out_lines {
        out.push_str(&format!("{:04x}  {} {}
", off, mn, ops));
    }
    out
}

fn aavm_lib_program(call: &str) -> AutoResult<String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let mut lib_code = String::new();
    for f in crate::AUTO_LIB_FILES_V2 {
        lib_code.push_str(&std::fs::read_to_string(root.join(f))?);
        lib_code.push('\u{a}');
    }
    Ok(format!("{}
fn main() {{
    print({})
}}
", lib_code, call))
}

/// Plan 511 W3:多模块 S4 形态编译链接(session.resolve_uses 取 dep 模块 +
/// 主模块 codegen S4 wrapper 约定 + lib.rs 1102-1181 池合并镜像 + Linker)。
/// 返回 (链接镜像字节, 合并字符串池)——多文件腿的 Rust 参考口径。
fn compile_and_link_multi(dir: &std::path::Path) -> AutoResult<(Vec<u8>, Vec<Vec<u8>>)> {
    let main_code = std::fs::read_to_string(dir.join("main.at"))?;
    let mut session = crate::compile::CompileSession::new();
    session.add_source_dir(dir.to_path_buf());
    session.resolve_uses(&main_code)?;
    let mut dep_modules = session.take_compiled_modules();

    let mut parser = crate::parser::Parser::new_with_type_store(&main_code, session.type_store());
    let ast = parser.parse()?;
    let mut codegen = Codegen::new_with_type_store(parser.type_store.clone());
    let (type_decls, other_stmts): (Vec<_>, Vec<_>) = ast
        .stmts
        .iter()
        .partition(|stmt| {
            matches!(
                stmt,
                crate::ast::Stmt::TypeDecl(_) | crate::ast::Stmt::Ext(_) | crate::ast::Stmt::EnumDecl(_)
            )
        });
    for stmt in &type_decls {
        codegen.compile_stmt(stmt)?;
    }
    if !other_stmts.is_empty() {
        codegen.emit_op(OpCode::FN_PROLOG);
        codegen.emit_byte(0);
        codegen.emit_byte(16);
        codegen.emit_op(OpCode::RESERVE_STACK);
        codegen.emit_byte(16);
        for stmt in &other_stmts {
            codegen.compile_stmt(stmt)?;
        }
    }
    codegen.code.push(OpCode::HALT as u8);

    // 池合并 + 索引重映射(lib.rs 镜像:主池去重并入 dep 池)
    let mut strings = codegen.strings.clone();
    let mut main_pool_idx: std::collections::HashMap<Vec<u8>, u32> = strings
        .iter()
        .enumerate()
        .map(|(i, s)| (s.clone(), i as u32))
        .collect();
    for module in dep_modules.iter_mut() {
        if !module.strings.is_empty() {
            let mut remap = vec![0u32; module.strings.len()];
            for (old_idx, s) in module.strings.iter().enumerate() {
                if let Some(&existing) = main_pool_idx.get(s) {
                    remap[old_idx] = existing;
                } else {
                    let new_idx = strings.len() as u32;
                    strings.push(s.clone());
                    main_pool_idx.insert(s.clone(), new_idx);
                    remap[old_idx] = new_idx;
                }
            }
            crate::remap_string_indices(&mut module.code, &remap);
        }
        // corpus_use 无 CREATE_OBJ 面(object 池恒空);object 池合并不镜像
    }
    let mut linker = Linker::new();
    for module in dep_modules {
        linker.add_module(module);
    }
    linker.add_module(Module {
        name: "__main__".to_string(),
        code: codegen.code.clone(),
        exports: codegen.exports.clone(),
        relocs: codegen.relocs.clone(),
        strings: codegen.strings.clone(),
        object_keys: codegen.object_keys.clone(),
        object_types: codegen.object_types.clone(),
        has_globals: !codegen.global_vars.is_empty(),
    });
    let (final_code, _symbols) = linker.link()
        .map_err(|e| crate::error::AutoError::Msg(e.message.clone()))?;
    Ok((final_code, strings))
}

/// Plan 511 W3 harness 自证:多模块管线对无 use 的单文件(临时目录隔离)
/// 与 S4 既有 compile_and_link 逐字符一致——证明多文件腿的 Rust 参考口径
/// 与已确立闸门同源(harness 架构正确性先于 aavm 实现)。
#[test]
fn test_aavm2_m4_use_harness_selfcheck() {
    let dir = corpus_dir();
    let tmp_root = std::env::temp_dir().join("aavm2_m4_selfcheck");
    let _ = std::fs::remove_dir_all(&tmp_root);
    std::fs::create_dir_all(&tmp_root).expect("temp dir");
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .expect("corpus dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "at").unwrap_or(false))
        .collect();
    entries.sort();
    let mut checked = 0;
    for (n, p) in entries.iter().enumerate() {
        let code = std::fs::read_to_string(p).unwrap();
        let case_dir = tmp_root.join(format!("{:03}", n));
        std::fs::create_dir_all(&case_dir).expect("case dir");
        std::fs::write(case_dir.join("main.at"), &code).expect("write main");
        let (multi_code, multi_strings) = compile_and_link_multi(&case_dir)
            .unwrap_or_else(|e| panic!("multi pipeline failed on {}: {e}", p.display()));
        let via_multi = normalized_dump(&multi_code, &multi_strings);
        let (linked, strings) = compile_and_link(&code).unwrap();
        let via_s4 = normalized_dump(&linked, &strings);
        assert_eq!(
            via_multi.trim_end(),
            via_s4.trim_end(),
            "harness selfcheck divergence for {}",
            p.display()
        );
        checked += 1;
    }
    eprintln!("M4 use-harness selfcheck: {checked} single-file corpus identical");
}

/// Plan 511 W3:corpus_use 多文件 M4 对拍(Rust 链接镜像 disasm vs aavm
/// codegen_dump_files)。aavm 侧 ev_run_files/codegen_dump_files 未实现时
/// 以运行期错误形态转红(W3 实现启动条件)。
#[test]
fn test_aavm2_m4_use_corpus() {
    let dir = corpus_use_dir();
    let mut cases: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("corpus_use dir {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    cases.sort();
    assert!(!cases.is_empty(), "no corpus_use cases");
    for case in cases {
        let main_path = case.join("main.at");
        let code = std::fs::read_to_string(&main_path)
            .unwrap_or_else(|e| panic!("read {}: {e}", main_path.display()));
        let (linked, strings) = compile_and_link_multi(&case)
            .unwrap_or_else(|e| panic!("rust reference failed on {}: {e}", case.display()));
        let expected = normalized_dump(&linked, &strings);
        let program = aavm_lib_program(&format!(
            "codegen_dump_files(\"{}\")",
            escape_for_at_literal(&main_path.display().to_string())
        ))
        .unwrap();
        let (_r2, stdout) = crate::run_with_capture(&program)
            .unwrap_or_else(|e| panic!("aavm run failed on {}: {e}", case.display()));
        assert_eq!(
            stdout.trim_end(),
            expected.trim_end(),
            "M4 use-corpus mismatch for {}
--- rust ---
{}
--- aavm ---
{}",
            case.display(),
            expected,
            stdout
        );
    }
    eprintln!("M4 use corpus: multi-file bytecode identical");
}

/// 诊断用:打印 Rust 参考侧对语料的规范化反汇编(--nocapture)。
#[test]
fn test_aavm2_m4_rust_disasm_print() {
    let dir = corpus_dir();
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .expect("corpus dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "at").unwrap_or(false))
        .collect();
    entries.sort();
    for p in entries {
        let code = std::fs::read_to_string(&p).unwrap();
        match compile_and_link(&code) {
            Ok((bc, strs)) => {
                eprintln!("=== {} ===
{}
", p.display(), normalized_dump(&bc, &strs))
            }
            Err(e) => eprintln!("=== {} === COMPILE ERROR: {}
", p.display(), e),
        }
    }
}
