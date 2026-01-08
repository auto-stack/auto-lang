use auto_val::{AutoStr, Shared};
use std::collections::HashMap;
use std::sync::Mutex;

pub mod io;

/// VM function signature: takes universe and single value argument
pub type VmFunction = fn(Shared<crate::Universe>, auto_val::Value) -> auto_val::Value;

/// VM method signature: takes universe, mutable instance reference, and arguments
pub type VmMethod = fn(Shared<crate::Universe>, &mut auto_val::Value, Vec<auto_val::Value>) -> auto_val::Value;

/// Represents a VM function in the registry
#[derive(Clone)]
pub struct VmFunctionEntry {
    pub name: AutoStr,
    pub func: VmFunction,
    pub is_method: bool,
}

/// Represents a VM type with its methods
pub struct VmTypeEntry {
    pub name: AutoStr,
    pub methods: HashMap<AutoStr, VmMethod>,
}

/// Represents a VM module containing functions and types
pub struct VmModule {
    pub name: AutoStr,
    pub functions: HashMap<AutoStr, VmFunctionEntry>,
    pub types: HashMap<AutoStr, VmTypeEntry>,
}

/// Global VM registry for all modules
pub struct VmRegistry {
    modules: HashMap<AutoStr, VmModule>,
}

impl VmRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn register_module(&mut self, module: VmModule) {
        self.modules.insert(module.name.clone(), module);
    }

    pub fn get_module(&self, name: &str) -> Option<&VmModule> {
        self.modules.get(name)
    }

    pub fn get_function(&self, module_name: &str, func_name: &str) -> Option<&VmFunctionEntry> {
        self.modules.get(module_name)?.functions.get(func_name)
    }

    pub fn get_method(&self, _type_name: &str, _method_name: &str) -> Option<&VmMethod> {
        // Search all modules for the type
        for module in self.modules.values() {
            if let Some(type_entry) = module.types.get(_type_name) {
                if let Some(method) = type_entry.methods.get(_method_name) {
                    return Some(method);
                }
            }
        }
        None
    }

    pub fn modules(&self) -> &HashMap<AutoStr, VmModule> {
        &self.modules
    }

}

/// Global VM registry instance
lazy_static::lazy_static! {
    pub static ref VM_REGISTRY: Mutex<VmRegistry> = Mutex::new(VmRegistry::new());
}

/// Initialize and register the IO module with the VM registry
pub fn init_io_module() {
    let mut io_module = VmModule {
        name: "auto.io".into(),
        functions: HashMap::new(),
        types: HashMap::new(),
    };

    // Register 'open' function
    io_module.functions.insert(
        "open".into(),
        VmFunctionEntry {
            name: "open".into(),
            func: io::open,
            is_method: false,
        },
    );

    // Register File type with methods
    let mut file_type = VmTypeEntry {
        name: "File".into(),
        methods: HashMap::new(),
    };

    file_type.methods.insert("close".into(), io::close_method as VmMethod);
    file_type.methods.insert("read_text".into(), io::read_text_method as VmMethod);

    io_module.types.insert("File".into(), file_type);

    // Register the module
    VM_REGISTRY.lock().unwrap().register_module(io_module);
}
