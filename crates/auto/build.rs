fn main() {
    // Set a 4MB stack for the main thread on Windows.
    // The parser uses deep recursion that overflows the 1MB default for complex UI files.
    // The MSVC linker uses /STACK for the PE header; the GNU linker uses -Wl,--stack.
    let stack_size = option_env!("AUTO_STACK_SIZE")
        .or(option_env!("auto_stack_size"))
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(4 * 1024 * 1024);

    #[cfg(target_env = "msvc")]
    println!("cargo:rustc-link-arg=/STACK:{}", stack_size);

    #[cfg(target_env = "gnu")]
    println!("cargo:rustc-link-arg=-Wl,--stack,{}", stack_size);
}
