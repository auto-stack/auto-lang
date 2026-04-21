use auto_lang::parser::Parser as AutoParser;
use auto_lang::vm::codegen::Codegen;
use auto_lang::vm::engine::AutoVM;
use auto_lang::vm::native_registry::register_builtin_natives;
use auto_lang::vm::virt_memory::VirtualFlash;
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(required = true)]
    input: PathBuf,

    /// Memory size in bytes (default: 1MB)
    #[arg(long, default_value_t = 1024 * 1024)]
    memory: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Register built-in native functions
    register_builtin_natives();

    let args = Args::parse();
    let source = fs::read_to_string(&args.input)
        .map_err(|e| anyhow::anyhow!("Failed to read input file: {}", e))?;

    // 1. Parse Source
    let mut parser = AutoParser::from(&source);
    let code = parser
        .parse()
        .map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;

    // 2. Compile to AutoVM Bytecode
    let mut codegen = Codegen::new();
    for (i, stmt) in code.stmts.iter().enumerate() {
        eprintln!("DEBUG: Compiling statement {}", i);
        let code_len_before = codegen.code.len();
        eprintln!("DEBUG:   code.len() before = {}, code[7] = {}", code_len_before, if codegen.code.len() > 7 { codegen.code[7] } else { 0 });
        codegen
            .compile_stmt(&stmt)
            .map_err(|e| anyhow::anyhow!("Codegen error: {:?}", e))?;
        let code_len_after = codegen.code.len();
        eprintln!("DEBUG:   code.len() after = {}, code[7] = {}", code_len_after, if codegen.code.len() > 7 { codegen.code[7] } else { 0 });
    }

    // Debug: Dump bytecode
    eprintln!("=== Bytecode Debug ===");
    eprintln!("DEBUG: code.len() = {}", codegen.code.len());
    eprintln!("DEBUG: BEFORE DUMP: code[7] = {} (expected 34 for LOAD_LOC_0)", codegen.code[7]);
    eprintln!("DEBUG: BEFORE DUMP: code[6] = {}, code[8] = {}", codegen.code[6], codegen.code[8]);
    for (i, &byte) in codegen.code.iter().enumerate() {
        // Use safe conversion - only show known opcodes by name
        // Known opcodes: 0x00 (NOP) to 0x3A (CALL_NAT)
        // For safety, just print the byte value without trying to convert to OpCode
        if byte <= 0x3A {
            // This is a valid opcode range, use std::mem::variant_count to check
            // For now, just print as hex to avoid unsafe transmute panics
            eprintln!("[{:04x}] {:02x}", i, byte);
        } else {
            eprintln!("[{:04x}] {:02x}", i, byte);
        }
    }
    eprintln!("=== End Bytecode ===");

    // Explicit Halt to be safe?
    // codegen.code.push(auto_lang::vm::opcode::OpCode::HALT as u8);
    // Actually our scripts might just end. VM handles EOF as termination.

    // 3. Link (Simple manual linking for single file)
    eprintln!("DEBUG: {} relocations to process", codegen.relocs.len());
    if !codegen.relocs.is_empty() {
        for (i, reloc) in codegen.relocs.iter().enumerate() {
            eprintln!("DEBUG: Reloc {}: type={:?}, name={}, offset={}", i, reloc.reloc_type, reloc.symbol_name, reloc.offset);
            match reloc.reloc_type {
                auto_lang::vm::loader::RelocType::FuncCall => {
                    let name = &reloc.symbol_name;
                    if let Some(&addr) = codegen.exports.get(name) {
                        let bytes = addr.to_le_bytes();
                        let offset = reloc.offset as usize;
                        eprintln!("DEBUG: Patching {} at offset {} with addr {}", name, offset, addr);
                        for (j, b) in bytes.iter().enumerate() {
                            eprintln!("DEBUG:   code[{}] = {} (was {})", offset + j, b, codegen.code[offset + j]);
                            codegen.code[offset + j] = *b;
                        }
                    } else {
                        return Err(anyhow::anyhow!("Undefined symbol: {}", name));
                    }
                }
                _ => {}
            }
        }
    }

    // 4. Initialize VM
    let flash = VirtualFlash::from_vec_with_metadata(
        codegen.code,
        codegen.exports.clone(),
        Vec::new(),
        Vec::new(),
    );
    let mut vm = AutoVM::new(flash, args.memory);
    vm.load_strings(codegen.strings);

    // 5. Find entry point and run
    // Look for main() or test() function, otherwise use address 0 for top-level scripts
    let entry_point = codegen
        .exports
        .get("main")
        .or_else(|| codegen.exports.get("test"))
        .copied()
        .unwrap_or(0) as usize; // Default to address 0 for scripts without main/test

    vm.spawn_task(entry_point, args.memory);
    vm.run_task_loop().await;

    Ok(())
}
