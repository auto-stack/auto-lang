use syn::{parse_macro_input, ItemFn, LitStr};

#[proc_macro_attribute]
pub fn rust_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute (e.g., "File.read_text")
    let name_attr = parse_macro_input!(attr as LitStr);
    let name = name_attr.value();

    // Parse the function
    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;
    let func_vis = &func.vis;
    let doc_attrs = &func.attrs;

    // Generate a unique shim name based on the FFI name
    // e.g., "File.read_text" -> "__shim_File_read_text"
    let shim_name_str = format!("__shim_{}", name.replace(".", "_"));
    let shim_name = syn::Ident::new(&shim_name_str, proc_macro2::Span::call_site());

    // Extract argument types and generate stack popping code
    let mut pop_args = Vec::new();
    let mut call_args = Vec::new();

    // We need to pop arguments in reverse order, so we process inputs in reverse
    for (i, arg) in func.sig.inputs.iter().rev().enumerate() {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                let arg_name = &pat_ident.ident;
                let arg_type = &pat_type.ty;

                // Code to pop this argument from the stack
                pop_args.push(quote! {
                    let #arg_name: #arg_type = crate::vm::ffi::VMConvertible::pop_from_stack(task, vm)
                        .map_err(|e| crate::vm::engine::VMError::RuntimeError(e.to_string()))?;
                });

                // The argument to pass to the original function
                // Wait, we reversed the iteration, so we must insert at the front of call_args
                call_args.insert(0, quote! { #arg_name });
            } else {
                panic!("rust_fn macro only supports simple identifier arguments");
            }
        } else {
            panic!("rust_fn macro does not support un-typed arguments or self");
        }
    }

    // Now call_args is in the correct order for the function call

    // Generate the expanded code
    let expanded = quote! {
        // Output the original function exactly as it was
        #(#doc_attrs)*
        #func_vis #func

        #[allow(non_snake_case)]
        pub fn #shim_name(task: &mut crate::vm::task::AutoTask, vm: &crate::vm::engine::AutoVM) -> Result<(), crate::vm::engine::VMError> {
            use crate::vm::ffi::VMConvertible;

            // 1. Pop arguments from the stack in reverse order
            #(#pop_args)*

            // 2. Call the actual function
            let result = #func_name(#(#call_args),*);

            // 3. Convert result back to VM value and push it
            result.push_to_stack(task, vm)
                .map_err(|e| crate::vm::engine::VMError::RuntimeError(e.to_string()))?;

            Ok(())
        }

        // Static registration
        crate::inventory::submit! {
            crate::vm::ffi::StaticFFIRegistration {
                name: #name,
                shim: #shim_name,
            }
        }
    };

    expanded.into()
}
