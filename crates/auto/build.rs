fn main() {
    // Set a large stack for the main thread on Windows.
    // The parser uses deep recursion that overflows Windows' 1MB default thread
    // stack. 4MB covered small/medium files; large real-world .at modules
    // (e.g. auto-musk specs.at, ~1100 lines) need ~16MB+. Default raised to
    // 64MB (virtual reservation only — physical memory is committed on demand).
    // Linux/macOS default to an 8MB main-thread stack, so no flag is needed there.
    // NOTE: gate on target_os = "windows" — target_env = "gnu" is also true on
    // Linux (x86_64-unknown-linux-gnu), and `-Wl,--stack` is a Windows/MinGW flag
    // that Linux's linker rejects.
    let stack_size = option_env!("AUTO_STACK_SIZE")
        .or(option_env!("auto_stack_size"))
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(64 * 1024 * 1024);

    #[cfg(all(target_os = "windows", target_env = "msvc"))]
    println!("cargo:rustc-link-arg=/STACK:{}", stack_size);

    #[cfg(all(target_os = "windows", target_env = "gnu"))]
    println!("cargo:rustc-link-arg=-Wl,--stack,{}", stack_size);
}
