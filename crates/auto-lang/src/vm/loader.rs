use std::collections::HashMap;

// Defined in docs/design/abc.md
// struct FragHeader {
//     u32 magic;        // "AUTO"
//     u32 version;      // 0x00010000
//     u32 code_size;    // Bytecode size
//     u32 const_size;   // Constant pool size (not used yet?)
//     u32 reloc_count;  // Relocation count
// };

// Use a simplified struct for now, assuming we parse/construct it manually
pub struct FragHeader {
    pub magic: u32,
    pub version: u32,
    pub code_size: u32,
    pub const_size: u32,
    pub reloc_count: u32,
}

#[derive(Debug, Clone)]
pub struct RelocEntry {
    pub offset: u32,         // Offset in code
    pub symbol_name: String, // Resolving by name for now (sid logic can be added later)
    pub reloc_type: RelocType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelocType {
    FuncCall = 0,
    GlobalVar = 1,
}

pub struct Module {
    pub name: String,
    pub code: Vec<u8>,
    // Exported symbols: Name -> Offset in *this* module's code
    pub exports: HashMap<String, u32>,
    pub relocs: Vec<RelocEntry>,
}

pub struct Linker {
    pub modules: Vec<Module>,
}

impl Linker {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    pub fn add_module(&mut self, module: Module) {
        self.modules.push(module);
    }

    pub fn link(&self) -> Result<(Vec<u8>, HashMap<String, u32>), String> {
        let mut final_code = Vec::new();
        let mut global_symbols = HashMap::new(); // Name -> Absolute Address in final_code

        // Pass 1: Layout code and build symbol table
        let mut current_offset = 0;
        // Map module index -> start offset
        let mut module_offsets = Vec::new();

        for module in &self.modules {
            module_offsets.push(current_offset);

            // Register exports
            for (sym_name, sym_offset) in &module.exports {
                if global_symbols.contains_key(sym_name) {
                    return Err(format!("Duplicate symbol: {}", sym_name));
                }
                global_symbols.insert(sym_name.clone(), current_offset + sym_offset);
            }

            current_offset += module.code.len() as u32;
        }

        // Pass 2: Concatenate code and Apply Relocations
        for (idx, module) in self.modules.iter().enumerate() {
            let _base_offset = module_offsets[idx];
            let mut mod_code = module.code.clone();

            for reloc in &module.relocs {
                // Find symbol
                let target_addr = global_symbols.get(&reloc.symbol_name).ok_or_else(|| {
                    format!(
                        "Undefined symbol: {} in module {}",
                        reloc.symbol_name, module.name
                    )
                })?;

                // Patch code
                match reloc.reloc_type {
                    RelocType::FuncCall => {
                        // CALL expects Absolute Address (u32)
                        // Write 4 bytes at reloc.offset
                        let bytes = target_addr.to_le_bytes();
                        let off = reloc.offset as usize;
                        if off + 4 > mod_code.len() {
                            return Err(format!("Reloc offset out of bounds in {}", module.name));
                        }
                        mod_code[off] = bytes[0];
                        mod_code[off + 1] = bytes[1];
                        mod_code[off + 2] = bytes[2];
                        mod_code[off + 3] = bytes[3];
                    }
                    RelocType::GlobalVar => {
                        // TODO: Implement Global Var resolution
                        // For now assuming just absolute address like func
                        let bytes = target_addr.to_le_bytes();
                        let off = reloc.offset as usize;
                        mod_code[off] = bytes[0];
                        mod_code[off + 1] = bytes[1];
                        mod_code[off + 2] = bytes[2];
                        mod_code[off + 3] = bytes[3];
                    }
                }
            }

            final_code.extend(mod_code);
        }

        Ok((final_code, global_symbols))
    }
}
