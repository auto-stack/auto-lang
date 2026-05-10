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
    for stmt in code.stmts.iter() {
        codegen
            .compile_stmt(&stmt)
            .map_err(|e| anyhow::anyhow!("Codegen error: {:?}", e))?;
    }

    // 3. Link (Simple manual linking for single file)
    if !codegen.relocs.is_empty() {
        for reloc in codegen.relocs.iter() {
            match reloc.reloc_type {
                auto_lang::vm::loader::RelocType::FuncCall => {
                    let name = &reloc.symbol_name;
                    if let Some(&addr) = codegen.exports.get(name) {
                        let bytes = addr.to_le_bytes();
                        let offset = reloc.offset as usize;
                        for (j, b) in bytes.iter().enumerate() {
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
    let entry_point = codegen
        .exports
        .get("main")
        .or_else(|| codegen.exports.get("test"))
        .copied()
        .unwrap_or(0) as usize;

    vm.spawn_task(entry_point, args.memory);
    vm.run_task_loop().await;

    Ok(())
}
