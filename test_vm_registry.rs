use auto_lang::vm;

fn main() {
    vm::init_io_module();
    vm::init_collections_module();
    vm::init_builder_module();
    
    let registry = vm::VM_REGISTRY.lock().unwrap();
    
    // Check if HashMap.new exists
    let hashmap_new = registry.get_function("collections", "HashMap.new");
    println!("HashMap.new exists: {:?}", hashmap_new.is_some());
    
    // Check if List.new exists
    let list_new = registry.get_function("collections", "List.new");
    println!("List.new exists: {:?}", list_new.is_some());
    
    // List all functions in collections module
    if let Some(collections) = registry.get_module("collections") {
        println!("\nFunctions in collections module:");
        for (name, _) in &collections.functions {
            println!("  - {}", name);
        }
        
        println!("\nTypes in collections module:");
        for (name, _) in &collections.types {
            println!("  - {}", name);
        }
    }
}
