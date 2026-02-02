use auto_lang::parser::Parser as AutoParser;
use auto_lang::vm::codegen::Codegen;
use auto_lang::vm::engine::BigVM;
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

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let source = fs::read_to_string(&args.input)
        .map_err(|e| anyhow::anyhow!("Failed to read input file: {}", e))?;

    // 1. Parse Source
    // Parser expects a string slice
    let mut parser = AutoParser::from(&source);
    let code = parser
        .parse()
        .map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;

    // 2. Compile to BigVM Bytecode
    let mut codegen = Codegen::new();
    for stmt in code.stmts {
        codegen
            .compile_stmt(&stmt)
            .map_err(|e| anyhow::anyhow!("Codegen error: {:?}", e))?;
    }

    // 3. Initialize and Run VM
    // We assume the generated code is complete (no external linking for now, or just simple one string)
    // Note: If we need linking, we should use Loader. For now, Codegen output is raw bytes + exports.
    // Ideally we should link if there are calls.
    // Codegen::new() produces raw bytes in `.code`.
    // If we have functions, they are in the same code buffer.
    // If we have calls, they need `relocs`.
    // Simple script might not have calls or just calls native.
    // If we support function calls in the script, we need to handle relocs.
    // But Codegen `compile_stmt` updates `code` linearly.
    // If we have forward calls, we rely on `relocs`.
    // BUT `BigVM` expects a `VirtualFlash` which is `Vec<u8>`.
    // It doesn't do linking at runtime (Loader does linking before creating Flash).

    // Check if we need to link.
    // If `codegen.relocs` is not empty, we might need a linker pass.
    // Currently, Codegen handles *internal* jumps (backpatching).
    // Relocs are for *function calls* (using absolute addresses).
    // `Codegen` for `Stmt::Fn` exports the function entry point.
    // `Codegen` for `Expr::Call` emits `CALL` with placeholder and adds `RelocEntry`.
    // So correct flow is:
    // Codegen -> Module (Code + Relocs + Exports) -> Linker -> Linked Code -> Flash.

    // Implementation of simple linking for single file:
    // We can just iterate relocs and prevent external calls (or resolve against known exports).
    // Since we are compiling one file, all functions should be in `exports`.

    // Let's implement a simple linker logic here or use `Loader` if available.
    // `Loader` is in `vm::loader`.

    if !codegen.relocs.is_empty() {
        // Resolve relocations against exports
        for reloc in codegen.relocs {
            match reloc.reloc_type {
                auto_lang::vm::loader::RelocType::FuncCall => {
                    let name = &reloc.symbol_name;
                    if let Some(&addr) = codegen.exports.get(name) {
                        // Patch the code at reloc.offset
                        let bytes = addr.to_le_bytes();
                        let offset = reloc.offset as usize;
                        for (i, b) in bytes.iter().enumerate() {
                            codegen.code[offset + i] = *b;
                        }
                    } else {
                        return Err(anyhow::anyhow!("Undefined symbol: {}", name));
                    }
                }
                _ => {}
            }
        }
    }

    let flash = VirtualFlash::new_with_code(codegen.code);
    let mut vm = BigVM::new(flash, args.memory);

    vm.run()
        .map_err(|e| anyhow::anyhow!("VM execution failed: {:?}", e))?;

    Ok(())
}
