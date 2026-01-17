use auto_val::{AutoStr, Shared};
use std::collections::HashMap;
use std::sync::Mutex;

pub mod io;
pub mod collections;
pub mod builder;

/// VM function signature: takes universe and single value argument
pub type VmFunction = fn(Shared<crate::Universe>, auto_val::Value) -> auto_val::Value;

/// VM method signature: takes universe, mutable instance reference, and arguments
pub type VmMethod =
    fn(Shared<crate::Universe>, &mut auto_val::Value, Vec<auto_val::Value>) -> auto_val::Value;

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

// Global VM registry instance
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

    file_type
        .methods
        .insert("close".into(), io::close_method as VmMethod);
    file_type
        .methods
        .insert("read_text".into(), io::read_text_method as VmMethod);

    io_module.types.insert("File".into(), file_type);

    // Register the module
    VM_REGISTRY.lock().unwrap().register_module(io_module);
}

/// Initialize and register the collections module with the VM registry
pub fn init_collections_module() {
    let mut collections_module = VmModule {
        name: "collections".into(),
        functions: HashMap::new(),
        types: HashMap::new(),
    };

    // Register HashMap type with methods
    let mut hashmap_type = VmTypeEntry {
        name: "HashMap".into(),
        methods: HashMap::new(),
    };

    hashmap_type
        .methods
        .insert("insert_str".into(), collections::hash_map_insert_str as VmMethod);
    hashmap_type
        .methods
        .insert("insert_int".into(), collections::hash_map_insert_int as VmMethod);
    hashmap_type
        .methods
        .insert("get_str".into(), collections::hash_map_get_str as VmMethod);
    hashmap_type
        .methods
        .insert("get_int".into(), collections::hash_map_get_int as VmMethod);
    hashmap_type
        .methods
        .insert("contains".into(), collections::hash_map_contains as VmMethod);
    hashmap_type
        .methods
        .insert("remove".into(), collections::hash_map_remove as VmMethod);
    hashmap_type
        .methods
        .insert("size".into(), collections::hash_map_size as VmMethod);
    hashmap_type
        .methods
        .insert("clear".into(), collections::hash_map_clear as VmMethod);
    hashmap_type
        .methods
        .insert("drop".into(), collections::hash_map_drop as VmMethod);

    // Register HashMap.new() as a static function
    collections_module.functions.insert(
        "HashMap.new".into(),
        VmFunctionEntry {
            name: "HashMap.new".into(),
            func: collections::hash_map_new_static,
            is_method: false,
        },
    );

    collections_module.types.insert("HashMap".into(), hashmap_type);

    // Register HashSet type with methods
    let mut hashset_type = VmTypeEntry {
        name: "HashSet".into(),
        methods: HashMap::new(),
    };

    hashset_type
        .methods
        .insert("insert".into(), collections::hash_set_insert as VmMethod);
    hashset_type
        .methods
        .insert("contains".into(), collections::hash_set_contains as VmMethod);
    hashset_type
        .methods
        .insert("remove".into(), collections::hash_set_remove as VmMethod);
    hashset_type
        .methods
        .insert("size".into(), collections::hash_set_size as VmMethod);
    hashset_type
        .methods
        .insert("clear".into(), collections::hash_set_clear as VmMethod);
    hashset_type
        .methods
        .insert("drop".into(), collections::hash_set_drop as VmMethod);

    // Register HashSet.new() as a static function
    collections_module.functions.insert(
        "HashSet.new".into(),
        VmFunctionEntry {
            name: "HashSet.new".into(),
            func: collections::hash_set_new,
            is_method: false,
        },
    );

    collections_module.types.insert("HashSet".into(), hashset_type);

    // Register the module
    VM_REGISTRY.lock().unwrap().register_module(collections_module);
}

/// Initialize and register the builder module with the VM registry
pub fn init_builder_module() {
    let mut builder_module = VmModule {
        name: "auto.builder".into(),
        functions: HashMap::new(),
        types: HashMap::new(),
    };

    // Register StringBuilder type with methods
    let mut stringbuilder_type = VmTypeEntry {
        name: "StringBuilder".into(),
        methods: HashMap::new(),
    };

    stringbuilder_type
        .methods
        .insert("append".into(), builder::string_builder_append as VmMethod);
    stringbuilder_type
        .methods
        .insert("append_char".into(), builder::string_builder_append_char as VmMethod);
    stringbuilder_type
        .methods
        .insert("append_int".into(), builder::string_builder_append_int as VmMethod);
    stringbuilder_type
        .methods
        .insert("build".into(), builder::string_builder_build as VmMethod);
    stringbuilder_type
        .methods
        .insert("clear".into(), builder::string_builder_clear as VmMethod);
    stringbuilder_type
        .methods
        .insert("len".into(), builder::string_builder_len as VmMethod);
    stringbuilder_type
        .methods
        .insert("drop".into(), builder::string_builder_drop as VmMethod);

    // Register StringBuilder.new() as a static function
    builder_module.functions.insert(
        "StringBuilder.new".into(),
        VmFunctionEntry {
            name: "StringBuilder.new".into(),
            func: builder::string_builder_new_static,
            is_method: false,
        },
    );

    // Register StringBuilder.new_with_default() as a static function
    builder_module.functions.insert(
        "StringBuilder.new_with_default".into(),
        VmFunctionEntry {
            name: "StringBuilder.new_with_default".into(),
            func: builder::string_builder_new_with_default_static,
            is_method: false,
        },
    );

    builder_module.types.insert("StringBuilder".into(), stringbuilder_type);

    // Register the module
    VM_REGISTRY.lock().unwrap().register_module(builder_module);
}
