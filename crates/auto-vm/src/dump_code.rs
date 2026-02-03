use auto_lang::parser::Parser as AutoParser;
use auto_lang::vm::codegen::Codegen;

fn main() {
    let source = std::fs::read_to_string("tmp/test_debug.at").unwrap();
    let mut parser = AutoParser::from(&source);
    let code = parser.parse().unwrap();

    let mut codegen = Codegen::new();
    for stmt in code.stmts {
        codegen.compile_stmt(&stmt).unwrap();
    }

    // Dump bytecode with annotations
    println!("=== Bytecode Dump ===");
    println!("Total bytes: {}", codegen.code.len());

    let mut i = 0;
    while i < codegen.code.len() {
        let op = codegen.code[i];
        print!("{:04x}: {:02x} ", i, op);

        match op {
            0x10 => { // CONST_I32
                if i + 4 < codegen.code.len() {
                    let val = i32::from_le_bytes(codegen.code[i+1..i+5].try_into().unwrap());
                    println!("CONST_I32 {}", val);
                    i += 5;
                } else {
                    println!("CONST_I32 <truncated>");
                    i += 1;
                }
            }
            0x20 => { // LOAD_LOCAL
                if i + 1 < codegen.code.len() {
                    println!("LOAD_LOCAL {}", codegen.code[i+1]);
                    i += 2;
                } else {
                    println!("LOAD_LOCAL <truncated>");
                    i += 1;
                }
            }
            0x21 => { // STORE_LOCAL
                if i + 1 < codegen.code.len() {
                    println!("STORE_LOCAL {}", codegen.code[i+1]);
                    i += 2;
                } else {
                    println!("STORE_LOCAL <truncated>");
                    i += 1;
                }
            }
            0x22 => { println!("LOAD_LOC_0"); i += 1; }
            0x23 => { println!("LOAD_LOC_1"); i += 1; }
            0x24 => { println!("LOAD_LOC_2"); i += 1; }
            0x25 => { println!("STORE_LOC_0"); i += 1; }
            0x26 => { println!("STORE_LOC_1"); i += 1; }
            0x70 => { // CALL
                if i + 4 < codegen.code.len() {
                    let addr = u32::from_le_bytes(codegen.code[i+1..i+5].try_into().unwrap());
                    println!("CALL addr={:08x}", addr);
                    i += 5;
                } else {
                    println!("CALL <truncated>");
                    i += 1;
                }
            }
            0x71 => { // RET
                if i + 1 < codegen.code.len() {
                    println!("RET n_args={}", codegen.code[i+1]);
                    i += 2;
                } else {
                    println!("RET <truncated>");
                    i += 1;
                }
            }
            0x60 => { // JMP
                if i + 2 < codegen.code.len() {
                    let offset = i16::from_le_bytes(codegen.code[i+1..i+3].try_into().unwrap());
                    println!("JMP offset={}", offset);
                    i += 3;
                } else {
                    println!("JMP <truncated>");
                    i += 1;
                }
            }
            0x72 => { // CALL_NAT
                if i + 2 < codegen.code.len() {
                    let id = u16::from_le_bytes(codegen.code[i+1..i+3].try_into().unwrap());
                    println!("CALL_NAT id={}", id);
                    i += 3;
                } else {
                    println!("CALL_NAT <truncated>");
                    i += 1;
                }
            }
            _ => {
                println!("UNKNOWN");
                i += 1;
            }
        }
    }

    // Show exports
    println!("\n=== Exports ===");
    for (name, addr) in &codegen.exports {
        println!("{} @ {:04x}", name, addr);
    }
}
