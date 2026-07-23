//! Plan 128 Phase 8: Linker & Bootstrapper
//!
//! This module provides the loading infrastructure that connects CodeGen output
//! to the VM execution engine. It implements the "cartridge" pattern where
//! CodeGen produces a `CompiledPackage` (pure data), and `VMLoader` freezes
//! it into `GlobalMeta` for execution.

use std::collections::HashMap;
use std::sync::Arc;

use crate::vm::native::NativeInterface;
use crate::vm::scheduler::GlobalMeta;
use crate::vm::task_handler::{TaskHandlerTable, SerializedPattern, TaskHandler};
use crate::vm::virt_memory::VirtualFlash;

// ============================================================================
// Plan 128: CompiledPackage - CodeGen's Final Output (The "ROM Cartridge")
// ============================================================================

/// The compiled output from CodeGen - a pure data structure that can be
/// serialized, cached, and loaded into the VM.
///
/// This is the "game cartridge" that contains all the code and metadata
/// needed to run an Auto program.
#[derive(Debug, Clone, Default)]
pub struct CompiledPackage {
    /// Linked bytecode (all modules combined)
    pub bytecode: Vec<u8>,

    /// String constant pool (all string literals)
    pub string_pool: Vec<Vec<u8>>,

    /// Object keys metadata for object literal creation
    pub object_keys: Vec<Vec<auto_val::ValueKey>>,

    /// Object field types for runtime value conversion
    pub object_types: Vec<Vec<crate::vm::codegen::ObjectType>>,

    /// Exported symbols: Name -> Bytecode offset
    pub exports: HashMap<String, u32>,

    /// All task definitions with their handler tables
    pub tasks: HashMap<String, TaskDefinition>,

    /// Plan 312: Collected #[api] routes (method, path, fn_name).
    /// Propagated to AutoVM for HTTP server routing.
    pub api_routes: Vec<(String, String, String)>,
}

/// Definition of a compiled task type
#[derive(Debug, Clone)]
pub struct TaskDefinition {
    /// Task type name (e.g., "LoggerTask")
    pub name: String,

    /// Whether this is a #[single] singleton task
    pub is_single: bool,

    /// Serialized patterns for message matching
    pub patterns: Vec<SerializedPattern>,

    /// Handler entries
    pub handlers: Vec<TaskHandler>,

    /// Start hook bytecode offset (if present)
    pub start_hook_offset: Option<u32>,

    /// Stop hook bytecode offset (if present)
    pub stop_hook_offset: Option<u32>,

    /// Else handler bytecode offset (if present)
    pub else_handler_offset: Option<u32>,

    /// String literals used by patterns (local to this task)
    pub strings: Vec<String>,
}

// ============================================================================
// Plan 128: VMLoader - The Bootstrapper
// ============================================================================

/// The VM Loader takes a `CompiledPackage` and produces a frozen `GlobalMeta`.
///
/// This is the bridge between compile-time and runtime:
/// 1. Receives pure data from CodeGen
/// 2. Optionally registers native FFI functions
/// 3. Freezes everything into Arc-wrapped, read-only structures
/// 4. Produces `GlobalMeta` for the scheduler
pub struct VMLoader {
    package: CompiledPackage,
    native_interface: NativeInterface,
}

impl VMLoader {
    /// Create a new loader with the compiled package
    pub fn new(package: CompiledPackage) -> Self {
        Self {
            package,
            native_interface: NativeInterface::new(),
        }
    }

    /// Create a loader with pre-configured native interface
    pub fn with_native_interface(package: CompiledPackage, native_interface: NativeInterface) -> Self {
        Self {
            package,
            native_interface,
        }
    }

    /// Get a reference to the native interface for registration
    pub fn native_interface(&mut self) -> &mut NativeInterface {
        &mut self.native_interface
    }

    /// The core bootstrap operation: freeze all data into GlobalMeta
    ///
    /// This consumes the loader and produces an Arc<GlobalMeta> that
    /// can be shared across all task contexts without any locks.
    pub fn bootstrap(self) -> Arc<GlobalMeta> {
        // 1. Convert TaskDefinitions to TaskHandlerTables
        let handler_tables = self.build_handler_tables();

        // 2. Create VirtualFlash from bytecode
        let bytecode = VirtualFlash::from_vec_with_metadata(
            self.package.bytecode,
            self.package.exports,
            self.package.object_keys,
            self.package.object_types,
        );

        // 3. Freeze everything into GlobalMeta
        Arc::new(GlobalMeta::from_components(
            bytecode,
            self.package.string_pool,
            self.native_interface,
            handler_tables,
        ))
    }

    /// Build handler tables from task definitions
    fn build_handler_tables(&self) -> HashMap<String, TaskHandlerTable> {
        let mut tables = HashMap::new();

        for (name, task_def) in &self.package.tasks {
            let table = TaskHandlerTable::from_components(
                task_def.name.clone(),
                task_def.handlers.clone(),
                task_def.patterns.clone(),
                task_def.strings.clone(),
                task_def.start_hook_offset,
                task_def.stop_hook_offset,
                task_def.else_handler_offset,
            );
            tables.insert(name.clone(), table);
        }

        tables
    }
}

// ============================================================================
// Legacy: Low-level Linker (for module linking)
// ============================================================================

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
    /// Source position of the call site that generated this relocation
    pub source_pos: Option<crate::token::Pos>,
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
    /// String constant pool: index -> string bytes
    pub strings: Vec<Vec<u8>>,
    /// Plan 073: Object keys metadata for object literal creation
    /// Each entry is a Vec of keys for one object literal (indexed by key_index)
    pub object_keys: Vec<Vec<auto_val::ValueKey>>,
    /// Plan 073: Object field types for runtime value conversion
    pub object_types: Vec<Vec<crate::vm::codegen::ObjectType>>,
    /// Plan 348 E1: true if this module has top-level global initializers
    /// (module-level `var`/`const`). Such modules must be retained even when
    /// they export no functions, so their STORE_GLOBAL init code runs and the
    /// globals become visible across module boundaries.
    pub has_globals: bool,
}

/// Structured linker error with source position info
#[derive(Debug, Clone)]
pub struct LinkError {
    pub message: String,
    pub symbol: String,
    pub module: String,
    pub source_pos: Option<crate::token::Pos>,
}

impl std::fmt::Display for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for LinkError {}

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

    pub fn link(&self) -> Result<(Vec<u8>, HashMap<String, u32>), LinkError> {
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
                    // Duplicate symbol — use module-qualified name instead
                    let qualified = format!("{}#{}", module.name, sym_name);
                    global_symbols.insert(qualified, current_offset + sym_offset);
                } else {
                    global_symbols.insert(sym_name.clone(), current_offset + sym_offset);
                }
            }

            current_offset += module.code.len() as u32;
        }

        // Pass 2: Concatenate code and Apply Relocations
        for (idx, module) in self.modules.iter().enumerate() {
            let _base_offset = module_offsets[idx];
            let mut mod_code = module.code.clone();

            for reloc in &module.relocs {
                // Find symbol. Plan 317 Phase B: after module flattening,
                // "db.all_notes" should resolve to the "all_notes" export.
                // Try exact match first, then strip module prefix and retry.
                let target_addr = global_symbols.get(&reloc.symbol_name)
                    .copied()
                    // Plan 322: try module#name qualified lookup for dotted symbols.
                    // E.g. db.create_note → db#create_note (resolves to db.at's
                    // version, not api.at's duplicate). This prevents infinite
                    // recursion when api.at's create_note calls db.create_note
                    // and the unqualified fallback resolves back to api.at's.
                    .or_else(|| {
                        if reloc.symbol_name.contains('.') {
                            let prefix = reloc.symbol_name.split('.').next().unwrap_or("");
                            let rest = reloc.symbol_name.split('.').last().unwrap_or("");
                            let qualified = format!("{}#{}", prefix, rest);
                            global_symbols.get(&qualified).copied()
                        } else {
                            None
                        }
                    })
                    // Fallback: strip prefix, try unqualified name.
                    .or_else(|| {
                        reloc.symbol_name.split('.').last().and_then(|stripped| {
                            if stripped != reloc.symbol_name {
                                global_symbols.get(stripped).copied()
                            } else {
                                None
                            }
                        })
                    })
                    .ok_or_else(|| {
                        LinkError {
                            message: format!(
                                "Undefined symbol: {} in module {}",
                                reloc.symbol_name, module.name
                            ),
                            symbol: reloc.symbol_name.clone(),
                            module: module.name.clone(),
                            source_pos: reloc.source_pos,
                        }
                    })?;

                // Patch code
                match reloc.reloc_type {
                    RelocType::FuncCall => {
                        // CALL expects Absolute Address (u32)
                        // Write 4 bytes at reloc.offset
                        let bytes = target_addr.to_le_bytes();
                        let off = reloc.offset as usize;
                        if off + 4 > mod_code.len() {
                            return Err(LinkError {
                                message: format!("Reloc offset out of bounds in {}", module.name),
                                symbol: String::new(),
                                module: module.name.clone(),
                                source_pos: None,
                            });
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
