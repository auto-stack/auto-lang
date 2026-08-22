//! 2026-08-22(侧栏串写根因)字符串池回归测试。
//!
//! 事故:运行期字符串池的索引在 natives 消费侧以 u16 截断(`decode_string
//! as u16` / `get_string(u16)` / `strings.len() as u16`),而字段读
//! (`push_value` 的 Value::Str 臂)每次都直推新池条目(无去重)——
//! ash-gui 侧栏每次视图重建重读 81 个命令的 name/description,池随敲键
//! 膨胀,越过 65535 后索引回绕互串:alias/bash 条目消失、ash 的 name
//! 读成 "."(name 走 u16 截断路径、description 走 u32 路径,故分裂)。
//!
//! 修复:get_string 放宽为 u32 + 清除全部截断点;重新驻留热点全部改走
//! add_string(内容去重)。本文件锁死两条不变量,防回归:
//! 1. 池规模 > 65535 后索引读写往返仍精确;
//! 2. 重复内容驻留零增长(增长引擎不得复活)。

#![cfg(test)]

use crate::vm::engine::AutoVM;
use crate::vm::virt_memory::VirtualFlash;

fn make_vm() -> AutoVM {
    AutoVM::new(VirtualFlash::new(1024), 1024)
}

#[test]
fn string_pool_survives_u16_boundary() {
    let vm = make_vm();
    // 锚点:先驻留两条早期字符串,后续海量分配不得改写它们。
    let anchor_a = vm.add_string(b"alias-command-name".to_vec());
    let anchor_b = vm.add_string(b"bash-command-name".to_vec());

    // 越过 u16 边界:塞 7 万条不同内容(去重下每条都是新条目)。
    for i in 0..70_000u64 {
        let s = format!("pool-entry-{}", i);
        vm.add_string(s.into_bytes());
    }

    // 高位索引往返:nanbox 标签是 u32 负数编码,2^31 内必须无损。
    let hi = vm.add_string(b"high-index-tail".to_vec());
    assert!(hi > 65535, "pool should exceed u16 range, got len {}", hi + 1);
    let nv = auto_val::encode_string(hi as u32);
    assert_eq!(auto_val::decode_string(nv) as usize, hi);
    assert_eq!(
        vm.get_string(hi as u32).unwrap(),
        b"high-index-tail".to_vec(),
        "high-index string must round-trip exactly"
    );

    // 早期锚点不得被改写(事故形态:name 变单字符 ".")。
    assert_eq!(
        vm.get_string(anchor_a as u32).unwrap(),
        b"alias-command-name".to_vec(),
        "early string must survive later allocations"
    );
    assert_eq!(
        vm.get_string(anchor_b as u32).unwrap(),
        b"bash-command-name".to_vec()
    );

    // 回绕窗口:第 65536/65537 条恰好落在 u16 回绕点,读回必须仍是自身。
    let idx_wrap = vm.add_string(b"wrap-point-probe".to_vec());
    assert_eq!(
        vm.get_string(idx_wrap as u32).unwrap(),
        b"wrap-point-probe".to_vec()
    );
}

#[test]
fn add_string_dedups_identical_content() {
    let vm = make_vm();
    let a = vm.add_string(b"repeated-view-label".to_vec());
    let before = {
        let strings = vm.strings.read().unwrap();
        strings.len()
    };
    // 模拟视图重建反复重读同一批 Value::Str 字段:内容相同 → 索引复用。
    for _ in 0..10_000 {
        let b = vm.add_string(b"repeated-view-label".to_vec());
        assert_eq!(a, b, "identical content must reuse the pool index");
    }
    let after = {
        let strings = vm.strings.read().unwrap();
        strings.len()
    };
    assert_eq!(before, after, "pool must not grow for repeated content");
    // 空串同理(多个 native 会推空串)。
    let e1 = vm.add_string(Vec::new());
    let e2 = vm.add_string(Vec::new());
    assert_eq!(e1, e2);
}

// ── 编译侧宽度锁(2026-08-22 池 u32 化)──────────────────────────────────────
// LOAD_STR 等池索引操作数从 u16 扩到 u32。若任何一侧(emit/decode)被改回
// 2 字节,字节码流错位 —— 池 <65536 时因高位补零侥幸不炸、越界后静默串写
// (见文件头事故记录)。以下断言直接锁死编码宽度。

#[test]
fn codegen_load_str_operand_is_u32() {
    use crate::ast::Expr;
    use crate::vm::codegen::Codegen;
    use crate::vm::opcode::OpCode;

    let mut codegen = Codegen::new();
    codegen.compile_expr(&Expr::Str("pool-u32-probe".into())).unwrap();
    assert_eq!(codegen.code[0], OpCode::LOAD_STR as u8, "expected LOAD_STR");
    assert_eq!(codegen.code.len(), 5, "LOAD_STR must be 1 opcode byte + 4 operand bytes");
    let idx = u32::from_le_bytes(codegen.code[1..5].try_into().unwrap());
    assert!(idx < codegen.strings.len() as u32);
    assert_eq!(codegen.strings[idx as usize], b"pool-u32-probe".to_vec());
}

#[test]
fn engine_load_str_decode_width_matches_codegen() {
    // 编译 → 装载 → 反汇编,断言流对齐:push.accum(9B) 后紧跟下一条指令。
    use crate::vm::codegen::Codegen;
    use crate::vm::disasm::Disassembler;
    use crate::vm::virt_memory::VirtualFlash;

    let mut parser = crate::parser::Parser::from("shadcn: off\n");
    parser.compile_dest = crate::parser::CompileDest::Config;
    let ast = parser.parse().unwrap();
    let mut args = auto_val::Obj::new();
    args.set("on", auto_val::Value::Bool(true));
    args.set("off", auto_val::Value::Bool(false));
    let mut codegen = Codegen::new_for_config();
    codegen.compile_config_program(&ast, &args).unwrap();
    let flash = VirtualFlash::new_with_code_and_keys(
        codegen.code.clone(),
        codegen.object_keys.clone(),
        codegen.object_types.clone(),
    );
    let lines = Disassembler::new(&flash).disassemble_range(0, codegen.code.len());
    // 找 push.accum:它占 1+8 字节,下一条指令偏移必须恰为 +9。
    let accum = lines.iter().find(|l| l.mnemonic.contains("push.accum"))
        .expect("config prolog must emit push.accum");
    let next = lines.iter().find(|l| l.offset > accum.offset)
        .expect("instruction after push.accum");
    assert_eq!(next.offset - accum.offset, 9,
        "push.accum operand must be 8 bytes (two u32 pool indices); got {}",
        next.offset - accum.offset);
}
