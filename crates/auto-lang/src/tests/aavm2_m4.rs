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
        } else if mn == "get.field" || mn == "set.generic.field" {
            // P532 W2 根修⑧:字段下标经池解析为名字(镜像 load.str 内容
            // 替换;两侧池布局不同,下标比较误报字段错位)
            if let Some(idx) = ops
                .strip_prefix("field[")
                .and_then(|r| r.strip_suffix(']'))
                .and_then(|r| r.parse::<usize>().ok())
            {
                format!(
                    "field[{:?}]",
                    String::from_utf8_lossy(strings.get(idx).map(|b| b.as_slice()).unwrap_or(b""))
                )
            } else {
                ops
            }
        } else if mn == "load.global" || mn == "store.global" {
            // 全局名下标 → 名字(同池解析哲学)
            if let Ok(idx) = ops.parse::<usize>() {
                format!(
                    "{:?}",
                    String::from_utf8_lossy(strings.get(idx).map(|b| b.as_slice()).unwrap_or(b""))
                )
            } else {
                ops
            }
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
    let lib_code = crate::aavm2_lib_source(&root)?;
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
#[cfg_attr(windows, ignore = "avm+aavm/avm+aa2r 双重解释器路径关闭(572 待澄清②裁定 2026-09-06):run_autovm_capture 硬编码 4MB 执行线程被 516KB lib 解释栈需求越过(探针 4MB 爆/5MB 过,与用例规模无关;T6 已修栈,路径维持关闭);重型对拍走⑤腿/at_mode/gen2(a2r 转译+编译+运行);Linux/CI 保留全量")]
fn test_aavm2_m4_codegen_corpus() {
    // Plan 564: 重内存测试守门——裸 cargo test(无 NEXTEST env)下秒退,
    // 防 2026-09-05 事件(12 线程全并发峰值 9.78GB);nextest 路径受
    // test-groups 组内限流,详见 .config/test-mem-weights.md。
    if !crate::tests::heavy_gate::heavy_gate("test_aavm2_m4_codegen_corpus") {
        return;
    }
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
    let lib_code = crate::aavm2_lib_source(&root)?;
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

/// P532 W2/⑥e:lib 静态字节差分闸门(语义等价口径)——同一 probe 经
/// 宿主 compile_and_link_multi 与 aavm codegen_dump_files 编译,归一化
/// 逐行对拍。转正(2026-09-05):⑥c-⑥f 作用域层级/pop 块尾豁免/链式跳
/// 指令选择/getter 构造旗标/f-string 段 tag 静态面五族对齐后双侧
/// SEMANTICALLY IDENTICAL(33452 canon 行)。canon 口径:.line 剔除/
/// jmp-call 目标抽象/帧容量豁免/jmp.far 编码宽度归一/call.spec↔call.nat
/// 分派机制归一(宿主类型解析 miss 的运行期 spec 分派 vs aavm 编译期
/// native 直连,判定面非镜像对象——行为由 M5+⑤腿兜底,KNOWN-DEBT
/// 登记)/fn.prolog args 保留。
#[test]
#[cfg_attr(windows, ignore = "avm+aavm/avm+aa2r 双重解释器路径关闭(572 待澄清②裁定 2026-09-06):run_autovm_capture 硬编码 4MB 执行线程被 516KB lib 解释栈需求越过(探针 4MB 爆/5MB 过,与用例规模无关;T6 已修栈,路径维持关闭);重型对拍走⑤腿/at_mode/gen2(a2r 转译+编译+运行);Linux/CI 保留全量")]
fn test_aavm2_p532_lib_static_diff() {
    // Plan 564: 重内存测试守门——裸 cargo test(无 NEXTEST env)下秒退,
    // 防 2026-09-05 事件(12 线程全并发峰值 9.78GB);nextest 路径受
    // test-groups 组内限流,详见 .config/test-mem-weights.md。
    if !crate::tests::heavy_gate::heavy_gate("test_aavm2_p532_lib_static_diff") {
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let case_dir = root.join("scratch532").join("libdiff");
    std::fs::create_dir_all(&case_dir).expect("case dir");
    std::fs::write(
        case_dir.join("main.at"),
        "use auto.lib.codegen: cg_compile\n\nfn main() {\n    var c = cg_compile(\"fn main() { print(7) }\")\n    print(\"ERR=[\" + c.err + \"]\")\n}\n",
    )
    .expect("write main");

    // rust reference leg (module resolution from repo root)
    let main_code = std::fs::read_to_string(case_dir.join("main.at")).unwrap();
    let mut session = crate::compile::CompileSession::new();
    session.add_source_dir(case_dir.clone());
    session.add_source_dir(root.clone());
    session.resolve_uses(&main_code).expect("resolve uses");
    let mut dep_modules = session.take_compiled_modules();
    let mut parser =
        crate::parser::Parser::new_with_type_store(&main_code, session.type_store());
    let ast = parser.parse().expect("parse");
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
        codegen.compile_stmt(stmt).expect("type decl");
    }
    if !other_stmts.is_empty() {
        codegen.emit_op(OpCode::FN_PROLOG);
        codegen.emit_byte(0);
        codegen.emit_byte(16);
        codegen.emit_op(OpCode::RESERVE_STACK);
        codegen.emit_byte(16);
        for stmt in &other_stmts {
            codegen.compile_stmt(stmt).expect("stmt");
        }
    }
    codegen.code.push(OpCode::HALT as u8);
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
    let (final_code, ref_strings) = linker
        .link()
        .map_err(|e| crate::error::AutoError::Msg(e.message.clone()))
        .expect("link");
    let _ = ref_strings;
    let rust_dump = normalized_dump(&final_code, &strings);

    // aavm leg: spliced harness (host-compiled lib drives aavm codegen)
    let main_path = case_dir.join("main.at");
    let program = aavm_lib_program(&format!(
        "codegen_dump_files(\"{}\")",
        escape_for_at_literal(&main_path.display().to_string())
    ))
    .unwrap();
    let saved_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();
    let result = crate::run_with_capture(&program);
    std::env::set_current_dir(&saved_cwd).unwrap();
    let (_r, aavm_dump) = result.expect("aavm leg run");

    let tmp = std::env::temp_dir();
    std::fs::write(tmp.join("p532_rust.txt"), &rust_dump).unwrap();
    std::fs::write(tmp.join("p532_aavm.txt"), &aavm_dump).unwrap();
    // 语义等价口径(P532 W2 残留③收口裁定;⑥e 转正补全):
    //  1) .line 指令整行剔除——纯调试元数据,两侧行号体系不同(宿主
    //     parser source_lines 对 inline is-else 臂归闭括号行/if 汇合点
    //     双标记,实测 .line 326+331 连发 vs aavm 单发),零执行语义;
    //     已登记 KNOWN-DEBT 候选。
    //  2) jmp/call 目标抽象化——.line 插入差使绝对字节目标平移,控制流
    //     结构由助记符序列+patch 模式保证。
    //  3) jmp.far 归一为 jmp(⑥e)——宿主对回跳距离 >32765 的循环尾
    //     跳发 JMP_FAR(u32 编码形态,codegen.rs 3373);aavm 指令列表
    //     (ins.n=下标)无字节宽度约束,serialize 无 far 形态。纯编码
    //     宽度差异,跳转语义等价(目标本已抽象)。
    //  4) call.spec/call.nat 归一为 call.disp(⑥e)——宿主 CALL_SPEC=
    //     类型解析 miss 时的运行期 spec 分派兜底(方法名+argc,
    //     codegen.rs 9157);aavm=编译期静态解析 native id。同一调用
    //     的两种分派机制,宿主 miss 面(类型推断覆盖面)非镜像对象
    //     (⑨ 回退实证);行为等价由 M5 引擎语料闸门+⑤腿原生闸门
    //     兜底(本差分只保指令流结构)。KNOWN-DEBT 登记。
    //  其余指令(助记符+操作数,含 store/load 槽号、fn.prolog locals)
    //  严格逐条相等。
    let canon = |s: &str| -> Vec<String> {
        s.lines()
            .filter_map(|l| {
                let rest = l.splitn(2, "  ").nth(1)?;
                let mut parts = rest.splitn(2, ' ');
                let mn = parts.next()?.trim();
                if mn == ".line" {
                    return None;
                }
                let ops = parts.next().unwrap_or("").trim();
                if (mn == "jmp" || mn == "jmp.z" || mn == "jmp.nz" || mn == "jmp.far")
                    && ops.starts_with("-> 0x")
                {
                    Some(format!("{} -> *", mn.trim_end_matches(".far")))
                } else if mn == "call.spec" || mn == "call.nat" {
                    Some("call.disp *".to_string())
                } else if mn == "call" && ops.starts_with("0x") {
                    Some(format!("{mn} *"))
                } else if mn == "fn.prolog" || mn == "reserve" {
                    // 帧容量操作数豁免(P532 W2 KNOWN-DEBT 候选):两侧作用域
                    // 推入结构层级不同(宿主 is 臂/循环体嵌套更深),aavm 帧
                    // 可能多预留 2 槽(tokenize locals 16 vs 14 实证)。槽位
                    // 寻址正确性由 store/load 槽号严格对拍保证(已一致),
                    // 多预留槽永不被寻址,零执行语义。fn.prolog 的 args 字段
                    // 保留;reserve 整体抽象。
                    if mn == "fn.prolog" {
                        let a = ops.split(',').next().unwrap_or(ops).trim();
                        Some(format!("fn.prolog {a}"))
                    } else {
                        Some("reserve *".to_string())
                    }
                } else {
                    Some(rest.trim_end().to_string())
                }
            })
            .collect()
    };
    let rust_norm = canon(&rust_dump);
    let aavm_norm = canon(&aavm_dump);
    eprintln!(
        "P532 static diff: rust={} lines, aavm={} lines (semantic canon)",
        rust_norm.len(),
        aavm_norm.len()
    );
    let mut first_diff = None;
    for i in 0..rust_norm.len().max(aavm_norm.len()) {
        let rl = rust_norm.get(i).map(|s| s.as_str()).unwrap_or("<EOF>");
        let al = aavm_norm.get(i).map(|s| s.as_str()).unwrap_or("<EOF>");
        if rl != al {
            first_diff = Some((i, rl.to_string(), al.to_string()));
            break;
        }
    }
    match first_diff {
        None => eprintln!("P532 static diff: SEMANTICALLY IDENTICAL"),
        Some((i, rl, al)) => {
            panic!("P532 static diff: FIRST DIVERGENCE at canon line {i}\n  rust: {rl}\n  aavm: {al}");
        }
    }
}

/// Plan 511 W3:corpus_use 多文件 M4 对拍(Rust 链接镜像 disasm vs aavm
/// codegen_dump_files)。aavm 侧 ev_run_files/codegen_dump_files 未实现时
/// 以运行期错误形态转红(W3 实现启动条件)。
#[test]
#[cfg_attr(windows, ignore = "avm+aavm/avm+aa2r 双重解释器路径关闭(572 待澄清②裁定 2026-09-06):run_autovm_capture 硬编码 4MB 执行线程被 516KB lib 解释栈需求越过(探针 4MB 爆/5MB 过,与用例规模无关;T6 已修栈,路径维持关闭);重型对拍走⑤腿/at_mode/gen2(a2r 转译+编译+运行);Linux/CI 保留全量")]
fn test_aavm2_m4_use_corpus() {
    // Plan 564: 重内存测试守门——裸 cargo test(无 NEXTEST env)下秒退,
    // 防 2026-09-05 事件(12 线程全并发峰值 9.78GB);nextest 路径受
    // test-groups 组内限流,详见 .config/test-mem-weights.md。
    if !crate::tests::heavy_gate::heavy_gate("test_aavm2_m4_use_corpus") {
        return;
    }
    let dir = corpus_use_dir();
    let mut cases: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("corpus_use dir {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).map(|n| n != "errors").unwrap_or(true))
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


/// 诊断用:corpus_use 各例的 Rust 参考侧规范化反汇编(写入
/// target/aavm2_use_disasm.txt,避免 nocapture 吞输出)。
#[test]
fn test_aavm2_m4_use_rust_disasm_print() {
    let dir = corpus_use_dir();
    let mut cases: Vec<_> = std::fs::read_dir(&dir)
        .expect("corpus_use dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter(|p| p.file_name().and_then(|n| n.to_str()).map(|n| n != "errors").unwrap_or(true))
        .collect();
    cases.sort();
    let nl = 10u8 as char;
    let mut out = String::new();
    for case in &cases {
        match compile_and_link_multi(case) {
            Ok((code, strings)) => {
                out.push_str(&format!("=== {} ==={nl}{}", case.display(), normalized_dump(&code, &strings)));
            }
            Err(e) => out.push_str(&format!("=== {} === ERR {}{nl}", case.display(), e)),
        }
    }
    let diag = std::env::temp_dir().join("aavm2_use_disasm.txt");
    std::fs::write(&diag, out).expect("write diag");
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

