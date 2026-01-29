use auto_val::{AutoStr, Shared};
use std::collections::HashMap;
use std::sync::Mutex;

pub mod builder;
pub mod collections;
pub mod io;
pub mod list;
pub mod memory;
pub mod storage;

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
    use std::sync::Mutex;

    static INIT: Mutex<bool> = Mutex::new(false);
    let mut initialized = INIT.lock().unwrap();
    if *initialized {
        return;
    }
    *initialized = true;
    drop(initialized);

    let mut io_module = VmModule {
        name: "auto.io".into(),
        functions: HashMap::new(),
        types: HashMap::new(),
    };

    // Register 'File.open' as a static function
    io_module.functions.insert(
        "File.open".into(),
        VmFunctionEntry {
            name: "File.open".into(),
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
    file_type
        .methods
        .insert("read_line".into(), io::read_line_method as VmMethod);
    file_type
        .methods
        .insert("write_line".into(), io::write_line_method as VmMethod);
    file_type
        .methods
        .insert("flush".into(), io::flush_method as VmMethod);
    file_type
        .methods
        .insert("read_char".into(), io::read_char_method as VmMethod);
    file_type
        .methods
        .insert("read_buf".into(), io::read_buf_method as VmMethod);

    io_module.types.insert("File".into(), file_type);

    // Register the module
    VM_REGISTRY.lock().unwrap().register_module(io_module);
}

/// Initialize and register the collections module with the VM registry
pub fn init_collections_module() {
    use std::sync::Mutex;

    static INIT: Mutex<bool> = Mutex::new(false);
    let mut initialized = INIT.lock().unwrap();
    if *initialized {
        return;
    }
    *initialized = true;
    drop(initialized);

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

    hashmap_type.methods.insert(
        "insert_str".into(),
        collections::hash_map_insert_str as VmMethod,
    );
    hashmap_type.methods.insert(
        "insert_int".into(),
        collections::hash_map_insert_int as VmMethod,
    );
    hashmap_type
        .methods
        .insert("get_str".into(), collections::hash_map_get_str as VmMethod);
    hashmap_type
        .methods
        .insert("get_int".into(), collections::hash_map_get_int as VmMethod);
    hashmap_type.methods.insert(
        "contains".into(),
        collections::hash_map_contains as VmMethod,
    );
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

    collections_module
        .types
        .insert("HashMap".into(), hashmap_type);

    // Register HashSet type with methods
    let mut hashset_type = VmTypeEntry {
        name: "HashSet".into(),
        methods: HashMap::new(),
    };

    hashset_type
        .methods
        .insert("insert".into(), collections::hash_set_insert as VmMethod);
    hashset_type.methods.insert(
        "contains".into(),
        collections::hash_set_contains as VmMethod,
    );
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

    collections_module
        .types
        .insert("HashSet".into(), hashset_type);

    // Register List type with methods
    let mut list_type = VmTypeEntry {
        name: "List".into(),
        methods: HashMap::new(),
    };

    list_type
        .methods
        .insert("push".into(), list::list_push as VmMethod);
    list_type
        .methods
        .insert("pop".into(), list::list_pop as VmMethod);
    list_type
        .methods
        .insert("len".into(), list::list_len as VmMethod);
    list_type
        .methods
        .insert("is_empty".into(), list::list_is_empty as VmMethod);
    list_type
        .methods
        .insert("capacity".into(), list::list_capacity as VmMethod);
    list_type
        .methods
        .insert("clear".into(), list::list_clear as VmMethod);
    list_type
        .methods
        .insert("reserve".into(), list::list_reserve as VmMethod);
    list_type
        .methods
        .insert("get".into(), list::list_get as VmMethod);
    list_type
        .methods
        .insert("set".into(), list::list_set as VmMethod);
    list_type
        .methods
        .insert("insert".into(), list::list_insert as VmMethod);
    list_type
        .methods
        .insert("remove".into(), list::list_remove as VmMethod);
    list_type
        .methods
        .insert("drop".into(), list::list_drop as VmMethod);
    list_type
        .methods
        .insert("iter".into(), list::list_iter as VmMethod);

    // Register List.new() as a static function
    collections_module.functions.insert(
        "List.new".into(),
        VmFunctionEntry {
            name: "List.new".into(),
            func: list::list_new,
            is_method: false,
        },
    );

    collections_module.types.insert("List".into(), list_type);

    // Register ListIter type with methods (Plan 051 Phase 2)
    let mut listiter_type = VmTypeEntry {
        name: "ListIter".into(),
        methods: HashMap::new(),
    };

    listiter_type
        .methods
        .insert("next".into(), list::list_iter_next as VmMethod);
    listiter_type
        .methods
        .insert("map".into(), list::list_iter_map as VmMethod);
    listiter_type
        .methods
        .insert("filter".into(), list::list_iter_filter as VmMethod);
    listiter_type
        .methods
        .insert("reduce".into(), list::list_iter_reduce as VmMethod);
    listiter_type
        .methods
        .insert("count".into(), list::list_iter_count as VmMethod);
    listiter_type
        .methods
        .insert("for_each".into(), list::list_iter_for_each as VmMethod);
    listiter_type
        .methods
        .insert("collect".into(), list::list_iter_collect as VmMethod);

    collections_module
        .types
        .insert("ListIter".into(), listiter_type);

    // Register MapIter type with methods (Plan 051 Phase 3)
    let mut mapiter_type = VmTypeEntry {
        name: "MapIter".into(),
        methods: HashMap::new(),
    };

    mapiter_type
        .methods
        .insert("next".into(), list::map_iter_next as VmMethod);
    mapiter_type
        .methods
        .insert("reduce".into(), list::list_iter_reduce as VmMethod);
    mapiter_type
        .methods
        .insert("count".into(), list::list_iter_count as VmMethod);
    mapiter_type
        .methods
        .insert("for_each".into(), list::list_iter_for_each as VmMethod);
    mapiter_type
        .methods
        .insert("collect".into(), list::list_iter_collect as VmMethod);

    collections_module
        .types
        .insert("MapIter".into(), mapiter_type);

    // Register FilterIter type with methods (Plan 051 Phase 4)
    let mut filteriter_type = VmTypeEntry {
        name: "FilterIter".into(),
        methods: HashMap::new(),
    };

    filteriter_type
        .methods
        .insert("next".into(), list::filter_iter_next as VmMethod);
    filteriter_type
        .methods
        .insert("reduce".into(), list::list_iter_reduce as VmMethod);
    filteriter_type
        .methods
        .insert("count".into(), list::list_iter_count as VmMethod);
    filteriter_type
        .methods
        .insert("for_each".into(), list::list_iter_for_each as VmMethod);
    filteriter_type
        .methods
        .insert("collect".into(), list::list_iter_collect as VmMethod);

    collections_module
        .types
        .insert("FilterIter".into(), filteriter_type);

    // Register memory management functions (Plan 052 Phase 2)
    // These functions enable self-hosted List<T> with manual reallocation
    collections_module.functions.insert(
        "alloc_array".into(),
        VmFunctionEntry {
            name: "alloc_array".into(),
            func: memory::alloc_array,
            is_method: false,
        },
    );

    collections_module.functions.insert(
        "free_array".into(),
        VmFunctionEntry {
            name: "free_array".into(),
            func: memory::free_array,
            is_method: false,
        },
    );

    collections_module.functions.insert(
        "realloc_array".into(),
        VmFunctionEntry {
            name: "realloc_array".into(),
            func: memory::realloc_array_wrapped,  // Wrapper that accepts [array, new_size]
            is_method: false,
        },
    );

    // Register the module
    VM_REGISTRY
        .lock()
        .unwrap()
        .register_module(collections_module);
}

/// Initialize and register the builder module with the VM registry
pub fn init_builder_module() {
    use std::sync::Mutex;

    static INIT: Mutex<bool> = Mutex::new(false);
    let mut initialized = INIT.lock().unwrap();
    if *initialized {
        return;
    }
    *initialized = true;
    drop(initialized);

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
    stringbuilder_type.methods.insert(
        "append_char".into(),
        builder::string_builder_append_char as VmMethod,
    );
    stringbuilder_type.methods.insert(
        "append_int".into(),
        builder::string_builder_append_int as VmMethod,
    );
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

    builder_module
        .types
        .insert("StringBuilder".into(), stringbuilder_type);

    // Register the module
    VM_REGISTRY.lock().unwrap().register_module(builder_module);
}

/// Initialize and register the storage module with the VM registry
/// This module provides Heap<T> and other storage strategies for Plan 052
pub fn init_storage_module() {
    use std::sync::Mutex;

    static INIT: Mutex<bool> = Mutex::new(false);
    let mut initialized = INIT.lock().unwrap();
    if *initialized {
        return;
    }
    *initialized = true;
    drop(initialized);

    let mut storage_module = VmModule {
        name: "storage".into(),
        functions: HashMap::new(),
        types: HashMap::new(),
    };

    // Register Heap<T> type with methods
    let mut heap_type = VmTypeEntry {
        name: "Heap".into(),
        methods: HashMap::new(),
    };

    // Register Heap.new() as a static function
    storage_module.functions.insert(
        "Heap.new".into(),
        VmFunctionEntry {
            name: "Heap.new".into(),
            func: storage::heap_new,
            is_method: false,
        },
    );

    // Register Heap instance methods
    heap_type
        .methods
        .insert("data".into(), storage::heap_data as VmMethod);
    heap_type
        .methods
        .insert("capacity".into(), storage::heap_capacity as VmMethod);
    heap_type
        .methods
        .insert("try_grow".into(), storage::heap_try_grow as VmMethod);
    heap_type
        .methods
        .insert("drop".into(), storage::heap_drop as VmMethod);

    storage_module.types.insert("Heap".into(), heap_type);

    // Register InlineInt64 type with methods
    let mut inline_int64_type = VmTypeEntry {
        name: "InlineInt64".into(),
        methods: HashMap::new(),
    };

    // Register InlineInt64.new() as a static function
    storage_module.functions.insert(
        "InlineInt64.new".into(),
        VmFunctionEntry {
            name: "InlineInt64.new".into(),
            func: storage::inline_int64_new,
            is_method: false,
        },
    );

    // Register InlineInt64 instance methods
    inline_int64_type
        .methods
        .insert("data".into(), storage::inline_int64_data as VmMethod);
    inline_int64_type
        .methods
        .insert("capacity".into(), storage::inline_int64_capacity as VmMethod);
    inline_int64_type
        .methods
        .insert("try_grow".into(), storage::inline_int64_try_grow as VmMethod);
    inline_int64_type
        .methods
        .insert("drop".into(), storage::inline_int64_drop as VmMethod);

    storage_module.types.insert("InlineInt64".into(), inline_int64_type);

    // Register the module
    VM_REGISTRY.lock().unwrap().register_module(storage_module);
}
