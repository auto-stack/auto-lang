//! PLAN-884 T-04：exit-audit / crash-trace 基础设施自 stdlib 壳迁入。
use super::*;

/// PLAN-575 D1 退出审计路径：env `AUTO_DESKTOP_EXIT_LOG` 优先（空串视为未
/// 设），缺省 `%LOCALAPPDATA%/auto-desktop/exit-audit.log`（无 LOCALAPPDATA
/// 的平台退回系统临时目录）。
pub fn exit_audit_path() -> StdPathBuf {
    if let Ok(p) = std::env::var("AUTO_DESKTOP_EXIT_LOG") {
        if !p.is_empty() {
            return StdPathBuf::from(p);
        }
    }
    let base = std::env::var("LOCALAPPDATA")
        .map(StdPathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    base.join("auto-desktop").join("exit-audit.log")
}

/// PLAN-575 D1 退出审计核心：向 `path` 追加一行 `{ts} pid={pid} code={code}
/// site={site}`。写失败静默吞掉——审计绝不引入新退出路径、绝不 panic
/// （G3 零行为变更）。父目录不存在则尝试创建（同样失败即放弃）。
pub(crate) fn exit_audit_to(path: &Path, code: i32, site: &str) {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let line = format!("{ts} pid={} code={code} site={site}\n", std::process::id());
    let _ = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| f.write_all(line.as_bytes()));
}

/// PLAN-575 D1 退出审计入口：落到审计路径（见 `exit_audit_path`）。
pub fn exit_audit(code: i32, site: &str) {
    exit_audit_to(&exit_audit_path(), code, site);
}

/// PLAN-575 D1 挂点②：全局 panic hook——先落一行审计（code=101 = Rust
/// panic 惯例退出码；site=panic，尽力携带消息+位置）再调既有 hook。被
/// catch_unwind 捕获的 panic 同样落笔（进程未退时审计行与存活事实并存，
/// 归因时以"审计行 + 进程死亡"组合判读）。
///
/// PLAN-747：打印腿改容错分派——RUST_BACKTRACE 未置位（自动化/常态）走
/// [`print_panic_tolerant`]，RUST_BACKTRACE 置位（开发态要栈）保留原生
/// `prev`。原生 hook 的打印走 stderr，管道关闭（os error 232，驱动方已
/// 断开）/资源枯竭（os error 1450，多实例并发）时 `_eprint` 自身 panic →
/// panic-in-panic → abort（fastfail 7），桌面实例以 0xCFFFFFFF 消失
/// （2026-10-09 退出审计 133 例打印失败 panic 实证 = WER c0000409 家族）。
pub fn install_exit_audit_panic_hook() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let one_line = format!(
            "panic msg={} loc={}",
            panic_payload_str(info).replace('\n', " "),
            info.location()
                .map(|l| format!("{}:{}", l.file(), l.line()))
                .unwrap_or_default()
        );
        exit_audit(101, &one_line);
        if backtrace_requested() {
            prev(info);
        } else {
            print_panic_tolerant(info);
        }
    }));
}

/// panic payload 的字符串视图（&str/String downcast，非 str 载荷回退占位）。
fn panic_payload_str(info: &std::panic::PanicHookInfo<'_>) -> String {
    info.payload()
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| info.payload().downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "non-str payload".to_string())
}

/// PLAN-765 T-03（纪要三 F-9）：Windows 原生崩溃审计钩——
/// `SetUnhandledExceptionFilter` 兜底落笔，与既有三挂点（vm_process_exit/
/// panic/main_return + main_err）同册互补。
///
/// 覆盖边界（文档化）：
/// - 覆盖：未经处理的 SEH 异常（access violation 0xC0000005 / 栈溢出
///   0xC00000FD 等）——此前绕过全部 Rust 挂点，零痕迹死亡无法与外部
///   终止区分；此后"审计册零行 + 退出码 1" = 外部击杀指纹（四分法完备）。
/// - 不覆盖：外部 TerminateProcess/taskkill（不可拦截）；`__fastfail`
///   （刻意绕过 SEH——该家族由 WER 判读）。
/// - 语义：落笔后返回 `EXCEPTION_CONTINUE_SEARCH`——保留 WER/转储既有
///   行为，只加观测不加处置；栈溢出场景运行于残栈，尽力而为。
/// - 限制：后续加载的模块（CRT/第三方）可能覆盖本 filter——best effort。
#[cfg(windows)]
pub fn install_native_crash_audit() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| unsafe {
        native_crash_ffi::set_filter(Some(native_crash_ffi::audit_filter));
    });
}

#[cfg(windows)]
mod native_crash_ffi {
    // 最小 FFI 面——不引 windows-sys 依赖（本仓 windows crate 为 optional
    // feature 门控，崩溃钩应无条件可用）。
    #[repr(C)]
    pub struct ExceptionRecord {
        pub exception_code: i32,
        pub exception_flags: u32,
        pub exception_record: *mut ExceptionRecord,
        pub exception_address: *mut core::ffi::c_void,
        pub number_parameters: u32,
        pub exception_information: [usize; 15],
    }
    #[repr(C)]
    pub struct ExceptionPointers {
        pub exception_record: *mut ExceptionRecord,
        pub context_record: *mut core::ffi::c_void,
    }

    const EXCEPTION_CONTINUE_SEARCH: i32 = 1;

    extern "system" {
        fn SetUnhandledExceptionFilter(
            filter: Option<extern "system" fn(*mut ExceptionPointers) -> i32>,
        ) -> *mut core::ffi::c_void;
    }

    pub unsafe fn set_filter(f: Option<extern "system" fn(*mut ExceptionPointers) -> i32>) {
        SetUnhandledExceptionFilter(f);
    }

    pub extern "system" fn audit_filter(info: *mut ExceptionPointers) -> i32 {
        let code = unsafe {
            if info.is_null() || (*info).exception_record.is_null() {
                -1
            } else {
                (*(*info).exception_record).exception_code
            }
        };
        // 崩溃上下文尽力而为：format!/fs 分配失败仅丢一行审计，不影响
        // 后续 WER 语义（EXCEPTION_CONTINUE_SEARCH 原样放行）。
        super::exit_audit(code, "native_crash");
        EXCEPTION_CONTINUE_SEARCH
    }
}

/// PLAN-765 r2 T-05 诊断：ExitProcess 的 IAT 追踪钩（env `P765_EXIT_TRACE=1`
/// 装配）。背景：musk bigpage 矩阵四轮 + r2 复跑两轮实测 VM 进程以码 1 静默
/// 消失——panic/main_err/native_crash/vm_process_exit 四审计面与 WER 全零痕
/// （r1 四分法该指纹=外部击杀，但 r2 复现确定性排除外部随机击杀），唯一
/// 未覆盖的进程内路径 = 依赖代码直接调用 `ExitProcess(1)`（不经本仓任何
/// 审计 site）。本钩在自身模块 IAT 上拦截 ExitProcess/TerminateProcess，
/// 落笔调用栈（模块基址相对偏移，可离线符号化）后确定性自终止
/// （ExitProcess 臂经 TerminateProcess(自身)，规避 detach 竞态、退出码
/// 精确保持；TerminateProcess 臂转调原函数）——诊断模式专用，语义面=
/// 退出码与死亡事实不变，与 F-7~F-9 同谱系。外部 TerminateProcess（taskkill 等）不经
/// 本进程 IAT，不在覆盖面（文档化边界）。
#[cfg(windows)]
pub fn install_exit_process_trace() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| unsafe {
        exit_trace_ffi::patch_iat();
    });
}

#[cfg(windows)]
mod exit_trace_ffi {
    // 最小 FFI 面（同 native_crash_ffi 纪律：不引 windows-sys 依赖）。
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct ImageDosHeader {
        e_magic: u16,
        _pad: [u8; 58],
        e_lfanew: i32,
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct ImageNtHeaders64 {
        signature: u32,
        _file_header: [u8; 20],
        optional_header: ImageOptionalHeader64,
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct ImageOptionalHeader64 {
        magic: u16,
        // SizeOfImage 位于偏移 56，DataDirectory（PE32+）起始于 112。
        _pad0: [u8; 56 - 2],
        size_of_image: u32,
        _pad1: [u8; 112 - 60],
        data_directory: [ImageDataDirectory; 16],
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct ImageDataDirectory {
        virtual_address: u32,
        size: u32,
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct ImageImportDescriptor {
        original_first_thunk: u32,
        time_date_stamp: u32,
        forwarder_chain: u32,
        name_rva: u32,
        first_thunk: u32,
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct ImageThunkData {
        address_of_data: usize,
    }

    const IMAGE_DIRECTORY_ENTRY_IMPORT: usize = 1;
    const PAGE_READWRITE: u32 = 0x04;

    extern "system" {
        fn GetModuleHandleW(name: *const u16) -> *mut core::ffi::c_void;
        fn GetModuleHandleA(name: *const u8) -> *mut core::ffi::c_void;
        fn GetProcAddress(
            module: *mut core::ffi::c_void,
            name: *const u8,
        ) -> *mut core::ffi::c_void;
        fn GetCurrentProcess() -> *mut core::ffi::c_void;
        fn VirtualProtect(
            addr: *const core::ffi::c_void,
            size: usize,
            new_protect: u32,
            old_protect: *mut u32,
        ) -> i32;
        fn RtlCaptureStackBackTrace(
            skip: u32,
            count: u32,
            back_trace: *mut *mut core::ffi::c_void,
            back_trace_hash: *mut u32,
        ) -> u16;
    }

    /// 原函数指针缓存（IAT 槽位被替换前的值；0=未替换）。
    static REAL_EXIT_PROCESS: std::sync::atomic::AtomicUsize =
        std::sync::atomic::AtomicUsize::new(0);
    static REAL_TERMINATE_PROCESS: std::sync::atomic::AtomicUsize =
        std::sync::atomic::AtomicUsize::new(0);

    fn rva_ptr<T>(base: *mut u8, rva: u32) -> *mut T {
        unsafe { base.add(rva as usize) as *mut T }
    }

    fn cstr_at(base: *mut u8, module_size: u32, rva: u32) -> Option<&'static str> {
        if rva == 0 || rva >= module_size {
            return None;
        }
        let p = rva_ptr::<u8>(base, rva);
        unsafe {
            let mut len = 0usize;
            while *p.add(len) != 0 && len < 64 {
                len += 1;
            }
            core::str::from_utf8(core::slice::from_raw_parts(p, len)).ok()
        }
    }

    /// 落笔一条带调用栈偏移的审计行（原函数由各 trampoline 直接转调）。
    unsafe fn trace_and_forward(site: &str, frames: &[*mut core::ffi::c_void], tag: &str) {
        let base = module_base();
        let offs: Vec<String> = frames
            .iter()
            .map(|f| format!("+{:x}", (*f as usize).saturating_sub(base as usize)))
            .collect();
        let line = format!(
            "{tag} frames=[{}] (symbolicate: addr = base + off)",
            offs.join(",")
        );
        // 复用审计册路径与格式（site 字段承载钩名与上下文）。
        super::exit_audit_to(&super::exit_audit_path(), 0, &format!("{site}|{line}"));
    }

    fn module_base() -> *mut u8 {
        unsafe { GetModuleHandleW(core::ptr::null()) as *mut u8 }
    }

    pub extern "system" fn traced_exit_process(code: u32) -> ! {
        let mut frames: [*mut core::ffi::c_void; 32] = [core::ptr::null_mut(); 32];
        let n =
            unsafe { RtlCaptureStackBackTrace(0, 32, frames.as_mut_ptr(), core::ptr::null_mut()) }
                as usize;
        unsafe {
            trace_and_forward(
                "exitproc_trace",
                &frames[..n],
                &format!("ExitProcess({code}) → self-terminate"),
            )
        };
        // 落笔后不走真 ExitProcess：其 LdrShutdownProcess 会跑系统 DLL 的
        // detach/onexit 表，与仍在运行的工作线程存在堆 free 竞态（实测子进程
        // 探针间歇 0xC0000005，基线路径 30/0 不可复现——竞态窗口由本钩的
        // 审计 I/O 扰动打开）。改经缓存指针直调 TerminateProcess(自身)：
        // 跳过 detach、退出码精确保持；Rust 侧清理（stdout flush）在进入
        // ExitProcess 前已由 std::process::exit 完成，不丢缓冲。诊断模式
        // （env 门控）专用路径，语义面=退出码与死亡事实不变。
        // 终止句柄解析：IAT 缓存优先；TerminateProcess 未按名入本模块 IAT 时
        // （按序号导入/未导入）经 GetProcAddress 动态解析——两条路都确定性
        // 死亡，不经 LdrShutdownProcess。
        let mut term = REAL_TERMINATE_PROCESS.load(std::sync::atomic::Ordering::Relaxed);
        if term == 0 {
            unsafe {
                let k32 = GetModuleHandleA(b"kernel32.dll\0".as_ptr());
                if !k32.is_null() {
                    term = GetProcAddress(k32, b"TerminateProcess\0".as_ptr()) as usize;
                }
            }
        }
        if term != 0 {
            let f: extern "system" fn(*mut core::ffi::c_void, u32) -> i32 =
                unsafe { core::mem::transmute(term) };
            let self_proc = unsafe { GetCurrentProcess() };
            while f(self_proc, code) == 0 {
                // 极罕见失败：让出一拍重试（对自身终将成功）。
                std::thread::yield_now();
            }
            // 终止已发起：自旋等待调度器收割（本函数不返回）。
            loop {
                std::thread::yield_now();
            }
        }
        // 理论不可达回退（kernel32 解析全失败）：真 ExitProcess。
        let real = REAL_EXIT_PROCESS.load(std::sync::atomic::Ordering::Relaxed);
        if real != 0 {
            let f: extern "system" fn(u32) -> ! = unsafe { core::mem::transmute(real) };
            f(code)
        }
        std::process::exit(code as i32)
    }

    pub extern "system" fn traced_terminate_process(
        handle: *mut core::ffi::c_void,
        code: u32,
    ) -> i32 {
        let mut frames: [*mut core::ffi::c_void; 32] = [core::ptr::null_mut(); 32];
        let n =
            unsafe { RtlCaptureStackBackTrace(0, 32, frames.as_mut_ptr(), core::ptr::null_mut()) }
                as usize;
        unsafe {
            trace_and_forward(
                "exitproc_trace",
                &frames[..n],
                &format!("TerminateProcess(handle={:p},{code})", handle),
            )
        }
        let real = REAL_TERMINATE_PROCESS.load(std::sync::atomic::Ordering::Relaxed);
        if real != 0 {
            let f: extern "system" fn(*mut core::ffi::c_void, u32) -> i32 =
                unsafe { core::mem::transmute(real) };
            return f(handle, code);
        }
        0
    }

    /// 遍历自身模块导入表，把 kernel32 的 ExitProcess/TerminateProcess
    /// IAT 槽替换为追踪 trampoline。失败静默（诊断件不引入启动风险）。
    pub unsafe fn patch_iat() {
        let base = module_base();
        if base.is_null() {
            return;
        }
        let dos = base as *const ImageDosHeader;
        if unsafe { *dos }.e_magic != 0x5A4D {
            return;
        }
        let nt = rva_ptr::<ImageNtHeaders64>(base, unsafe { *dos }.e_lfanew as u32);
        if unsafe { *nt }.signature != 0x00004550 {
            return;
        }
        let import_dir =
            unsafe { (*nt).optional_header.data_directory[IMAGE_DIRECTORY_ENTRY_IMPORT] };
        if import_dir.virtual_address == 0 {
            return;
        }
        let mut desc = rva_ptr::<ImageImportDescriptor>(base, import_dir.virtual_address);
        let mut patched = 0usize;
        while unsafe { (*desc).original_first_thunk != 0 || (*desc).first_thunk != 0 } {
            let module_size = unsafe { (*nt).optional_header.size_of_image };
            let dll = cstr_at(base, module_size, unsafe { (*desc).name_rva });
            let is_kernel32 = dll
                .map(|d| d.eq_ignore_ascii_case("kernel32.dll"))
                .unwrap_or(false);
            let has_int = unsafe { (*desc).original_first_thunk } != 0;
            if is_kernel32 && has_int {
                let mut int =
                    rva_ptr::<ImageThunkData>(base, unsafe { (*desc).original_first_thunk });
                let mut iat = rva_ptr::<ImageThunkData>(base, unsafe { (*desc).first_thunk });
                loop {
                    let ord = unsafe { (*int).address_of_data };
                    if ord == 0 {
                        break;
                    }
                    // 高位为 1 = 序号导入（无名字，跳过——kernel32 关键两函数
                    // 均按名导入）。
                    if ord & (1usize << 63) == 0 {
                        // IMAGE_IMPORT_BY_NAME: u16 hint + ASCIZ name
                        let name_rva = (ord & 0xFFFF_FFFF) as u32;
                        if let Some(fname) = cstr_at(base, module_size, name_rva.wrapping_add(2)) {
                            let replacement = match fname {
                                "ExitProcess" => traced_exit_process as usize,
                                "TerminateProcess" => traced_terminate_process as usize,
                                _ => 0,
                            };
                            if replacement != 0 {
                                let slot = &mut (*iat).address_of_data as *mut usize;
                                let mut old = 0u32;
                                if VirtualProtect(
                                    slot.cast(),
                                    core::mem::size_of::<usize>(),
                                    PAGE_READWRITE,
                                    &mut old,
                                ) == 1
                                {
                                    let prev = unsafe { *slot };
                                    unsafe { *slot = replacement };
                                    let _ = VirtualProtect(
                                        slot.cast(),
                                        core::mem::size_of::<usize>(),
                                        old,
                                        &mut old,
                                    );
                                    match fname {
                                        "ExitProcess" => REAL_EXIT_PROCESS
                                            .store(prev, std::sync::atomic::Ordering::Relaxed),
                                        "TerminateProcess" => REAL_TERMINATE_PROCESS
                                            .store(prev, std::sync::atomic::Ordering::Relaxed),
                                        _ => {}
                                    }
                                    patched += 1;
                                }
                            }
                        }
                    }
                    int = unsafe { int.add(1) };
                    iat = unsafe { iat.add(1) };
                }
            }
            desc = unsafe { desc.add(1) };
        }
        let _ = patched;
    }
}

/// 是否请求 backtrace：`RUST_BACKTRACE` 置位且非 "0"（与 std 默认 hook 同口径）。
pub(crate) fn backtrace_requested() -> bool {
    std::env::var_os("RUST_BACKTRACE")
        .map(|v| v != "0")
        .unwrap_or(false)
}

/// PLAN-747 容错 panic 打印：自组默认格式（thread 名 + 位置 + 消息 +
/// RUST_BACKTRACE 提示行），`write_all`/`flush` 的 io::Error 一律吞掉——
/// 打印失败绝不升级为 panic-in-panic/abort（与 std `_eprint` 的
/// panic-on-failure 语义唯一差异点，仅丢失 backtrace 能力）。
pub(crate) fn print_panic_tolerant(info: &std::panic::PanicHookInfo<'_>) {
    let name = std::thread::current()
        .name()
        .unwrap_or("<unnamed>")
        .to_string();
    let loc = info
        .location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        .unwrap_or_else(|| "unknown location".to_string());
    let text = format_panic_report(&name, &loc, &panic_payload_str(info));
    let mut err = std::io::stderr().lock();
    let _ = err.write_all(text.as_bytes());
    let _ = err.flush();
}

/// 容错报告文本（默认 hook 同形：thread 名/位置/消息/提示行）。
pub(crate) fn format_panic_report(thread_name: &str, location: &str, message: &str) -> String {
    format!(
        "thread '{thread_name}' panicked at {location}:\n{message}\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\n"
    )
}
