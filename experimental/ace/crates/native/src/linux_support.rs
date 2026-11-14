//! PLAN-1040: the Linux support object — raw machine code the build tool
//! generates and loads into the target link (`mn-linux-support`).
//!
//! Cranelift has no generic inline-asm surface, so the few pieces the
//! Linux lane needs below the Cranelift boundary are emitted here as
//! short hand-assembled x86-64 stubs inside one ELF64 relocatable object
//! (design §3: compiler-generated short machine-code stubs — the calling
//! semantics still come from the shared verify/MIR pipeline, there is no
//! second semantics path):
//!
//! - `_start` — the process entry: nulls the frame pointer, walks the
//!   kernel stack (argc/argv/envp) to auxv, stores `AT_PAGESZ` into
//!   `mn_pagesz`, aligns the stack and `call`s the Cranelift `ac_start`
//!   wrapper (which owns the explicit `exit_group`). If `ac_start` ever
//!   returns it still exits 70 (the frozen trap code) — never falls off.
//! - `mn_syscall1..6` — System-V functions marshalling into the x86-64
//!   kernel ABI (`rax`=nr, args `rdi/rsi/rdx/r10/r8/r9`; `rcx`/`r11`
//!   clobbered), returning the raw kernel value bit-exact.
//! - `mn_exit` — `exit_group(code)` via `mn_syscall1`'s encoding; never
//!   returns (`ud2` guard).
//! - `mn_pagesz` — the 8-byte page-size slot `_start` fills.
//!
//! Zero libc/CRT, zero imports besides the two link-time references
//! (`ac_start`, and the local `mn_pagesz` slot).

use crate::config::{
    LINUX_EXIT_SYMBOL, LINUX_PAGESZ_SYMBOL, LINUX_START_TRAMPOLINE, LINUX_SYSCALL_SYMBOLS,
    MAX_COFF_BYTES,
};
use crate::MnError;
use cranelift_object::object::write::{
    Object, Relocation, RelocationFlags, Symbol as WriteSymbol, SymbolSection,
};
use cranelift_object::object::{
    elf::R_X86_64_PC32, elf::R_X86_64_PLT32, Architecture, BinaryFormat, Endianness, SectionKind,
    SymbolFlags, SymbolKind, SymbolScope,
};

/// Minimal label-based assembler for the fixed stub set below. Only the
/// pieces the Linux lane needs — internal jumps (rel8) are backpatched;
/// cross-object references (ac_start, mn_pagesz) become relocations.
struct StubAsm {
    out: Vec<u8>,
    labels: Vec<(&'static str, usize)>,
    holes: Vec<(usize, u8, &'static str)>,
}

impl StubAsm {
    fn new() -> Self {
        Self {
            out: Vec::new(),
            labels: Vec::new(),
            holes: Vec::new(),
        }
    }

    fn label(&mut self, name: &'static str) {
        self.labels.push((name, self.out.len()));
    }

    fn emit(&mut self, bytes: &'static [u8]) {
        self.out.extend_from_slice(bytes);
    }

    /// rel8 jump placeholder (opcode already emitted by `prefix`).
    fn rel8(&mut self, prefix: &'static [u8], target: &'static str) {
        self.emit(prefix);
        let at = self.out.len();
        self.emit(&[0]);
        self.holes.push((at, 1, target));
    }

    fn finish(mut self) -> Vec<u8> {
        let base = self.out.len();
        for (at, width, target) in self.holes.drain(..) {
            let pos = self
                .labels
                .iter()
                .find(|(n, _)| *n == target)
                .map(|(_, p)| *p)
                .unwrap_or_else(|| panic!("stub label {target} missing"));
            let disp = pos as i64 - (at as i64 + width as i64);
            let slice = &mut self.out[at..at + width as usize];
            match width {
                1 => {
                    let d = i8::try_from(disp).expect("rel8 in range");
                    slice.copy_from_slice(&d.to_le_bytes());
                }
                4 => {
                    let d = i32::try_from(disp).expect("rel32 in range");
                    slice.copy_from_slice(&d.to_le_bytes());
                }
                _ => unreachable!(),
            }
        }
        debug_assert_eq!(base, self.out.len());
        self.out
    }
}

const AT_PAGESZ: i64 = 6;

fn assemble_start() -> (Vec<u8>, usize, usize) {
    // Returns (code, lea_pagesz_hole_offset, call_start_hole_offset).
    let mut a = StubAsm::new();
    a.emit(&[0x31, 0xED]); // xor ebp, ebp
    a.emit(&[0x48, 0x89, 0xE7]); // mov rdi, rsp
    a.emit(&[0x48, 0x8B, 0x07]); // mov rax, [rdi]        (argc)
    a.emit(&[0x48, 0x8D, 0x77, 0x08]); // lea rsi, [rdi+8] (argv)
    a.emit(&[0x48, 0x8D, 0x34, 0xC6]); // lea rsi, [rsi+rax*8]
    a.emit(&[0x48, 0x83, 0xC6, 0x08]); // add rsi, 8        (skip argv NULL) → envp
    a.emit(&[0x48, 0x89, 0xF7]); // mov rdi, rsi
    a.emit(&[0x48, 0xC7, 0xC1, 0xFF, 0xFF, 0xFF, 0xFF]); // mov rcx, -1
    a.emit(&[0x31, 0xC0]); // xor eax, eax
    a.emit(&[0xFC]); // cld
    a.emit(&[0xF2, 0x48, 0xAF]); // repnz scasq → rdi = auxv (envp NULL consumed)
    a.label("aux");
    a.emit(&[0x48, 0x8B, 0x07]); // mov rax, [rdi]       (auxv key)
    a.emit(&[0x48, 0x83, 0xF8, AT_PAGESZ as u8]); // cmp rax, 6
    a.rel8(&[0x74], "found"); // je found
    a.emit(&[0x48, 0x85, 0xC0]); // test rax, rax
    a.rel8(&[0x74], "null"); // je null (AT_NULL)
    a.emit(&[0x48, 0x83, 0xC7, 0x10]); // add rdi, 16
    a.rel8(&[0xEB], "aux"); // jmp aux
    a.label("found");
    a.emit(&[0x48, 0x8B, 0x4F, 0x08]); // mov rcx, [rdi+8]  (AT_PAGESZ value)
    a.rel8(&[0xEB], "store"); // jmp store
    a.label("null");
    a.emit(&[0x31, 0xC9]); // xor ecx, ecx      (fail closed: no auxv → 0)
    a.label("store");
    // lea rdx, [rip+rel32]  → mn_pagesz (R_X86_64_PC32, addend -4)
    a.emit(&[0x48, 0x8D, 0x15]);
    let lea_hole = a.out.len();
    a.emit(&[0, 0, 0, 0]);
    a.emit(&[0x48, 0x89, 0x0A]); // mov [rdx], rcx
    a.emit(&[0x48, 0x83, 0xE4, 0xF0]); // and rsp, -16
                                       // call rel32 → ac_start (R_X86_64_PLT32, addend -4)
    a.emit(&[0xE8]);
    let call_hole = a.out.len();
    a.emit(&[0, 0, 0, 0]);
    a.emit(&[0xB8, 0xE7, 0x00, 0x00, 0x00]); // mov eax, 231 (exit_group)
    a.emit(&[0xBF, 0x46, 0x00, 0x00, 0x00]); // mov edi, 70 (frozen trap exit)
    a.emit(&[0x0F, 0x05]); // syscall
    a.emit(&[0xF4]); // hlt (never reached)
    (a.finish(), lea_hole, call_hole)
}

/// `mn_syscallN(nr, a1.., ..) -> i64` — SysV in, kernel ABI out.
/// Marshalling order avoids clobbering before the last read; only
/// caller-saved registers are touched (rcx/r11 die inside `syscall`).
fn assemble_syscall_n(n: usize) -> Vec<u8> {
    let mut c = Vec::new();
    let mut e = |b: &[u8]| c.extend_from_slice(b);
    if n >= 6 {
        e(&[0x4C, 0x8B, 0x5C, 0x24, 0x08]); // mov r11, [rsp+8]  (a6, stack arg)
    }
    e(&[0x48, 0x89, 0xF8]); // mov rax, rdi  (nr)
    e(&[0x48, 0x89, 0xF7]); // mov rdi, rsi  (a1)
    if n >= 2 {
        e(&[0x48, 0x89, 0xD6]); // mov rsi, rdx  (a2)
    }
    if n >= 3 {
        e(&[0x48, 0x89, 0xCA]); // mov rdx, rcx  (a3)
    }
    if n >= 4 {
        e(&[0x4D, 0x89, 0xC2]); // mov r10, r8   (a4)
    }
    if n >= 5 {
        e(&[0x4D, 0x89, 0xC8]); // mov r8, r9    (a5)
    }
    if n >= 6 {
        e(&[0x4D, 0x89, 0xD9]); // mov r9, r11   (a6)
    }
    e(&[0x0F, 0x05]); // syscall
    e(&[0xC3]); // ret
    c
}

/// `mn_exit(code)` — `exit_group` via the raw ABI; `ud2` guards the
/// never-returning contract.
fn assemble_exit() -> Vec<u8> {
    let mut c = Vec::new();
    c.extend_from_slice(&[0xB8, 0xE7, 0x00, 0x00, 0x00]); // mov eax, 231
    c.extend_from_slice(&[0x0F, 0x05]); // syscall (rdi = code already)
    c.extend_from_slice(&[0x0F, 0x0B]); // ud2
    c.extend_from_slice(&[0xC3]); // ret (unreachable; alignment)
    c
}

/// Emit the standalone `mn-linux-support` ELF64 relocatable object.
/// Symbols: `_start`, `mn_syscall1..6`, `mn_exit` (global text) and the
/// writable zero-initialized `mn_pagesz` slot. Exactly two relocations:
/// the `ac_start` call (PLT32) and the local `mn_pagesz` reference (PC32).
pub fn emit_linux_support_object() -> Result<Vec<u8>, MnError> {
    let mut obj = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    let text = obj.add_section(Vec::new(), b".text".to_vec(), SectionKind::Text);
    let bss = obj.add_section(Vec::new(), b".bss".to_vec(), SectionKind::UninitializedData);

    let (start_code, lea_hole, call_hole) = assemble_start();
    let start_len = start_code.len();
    let mut text_all = start_code;
    let mut stub_offs: Vec<(&str, usize, usize)> = Vec::new(); // (name, off, len)
    for (i, name) in LINUX_SYSCALL_SYMBOLS.iter().enumerate() {
        let code = assemble_syscall_n(i + 1);
        let off = text_all.len();
        let len = code.len();
        text_all.extend_from_slice(&code);
        stub_offs.push((name, off, len));
    }
    let exit_code = assemble_exit();
    let exit_off = text_all.len();
    let exit_len = exit_code.len();
    text_all.extend_from_slice(&exit_code);

    let start_sym = obj.add_symbol(WriteSymbol {
        name: LINUX_START_TRAMPOLINE.as_bytes().to_vec(),
        value: 0,
        size: start_len as u64,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(text),
        flags: SymbolFlags::None,
    });
    obj.add_symbol_data(start_sym, text, &text_all[..start_len], 1);
    for (name, off, len) in &stub_offs {
        let sym = obj.add_symbol(WriteSymbol {
            name: name.as_bytes().to_vec(),
            value: 0,
            size: *len as u64,
            kind: SymbolKind::Text,
            scope: SymbolScope::Linkage,
            weak: false,
            section: SymbolSection::Section(text),
            flags: SymbolFlags::None,
        });
        obj.add_symbol_data(sym, text, &text_all[*off..*off + *len], 1);
    }
    let exit_sym = obj.add_symbol(WriteSymbol {
        name: LINUX_EXIT_SYMBOL.as_bytes().to_vec(),
        value: 0,
        size: exit_len as u64,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(text),
        flags: SymbolFlags::None,
    });
    obj.add_symbol_data(exit_sym, text, &text_all[exit_off..exit_off + exit_len], 1);

    let pagesz_sym = obj.add_symbol(WriteSymbol {
        name: LINUX_PAGESZ_SYMBOL.as_bytes().to_vec(),
        value: 0,
        size: 8,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(bss),
        flags: SymbolFlags::None,
    });
    obj.add_symbol_bss(pagesz_sym, bss, 8, 8);

    // The `ac_start` import the `_start` trampoline calls.
    let ac_start_sym = obj.add_symbol(WriteSymbol {
        name: crate::config::START_SYMBOL.as_bytes().to_vec(),
        value: 0,
        size: 0,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Undefined,
        flags: SymbolFlags::None,
    });

    // Two link-time references only: call ac_start (PLT32) and the local
    // mn_pagesz lea (PC32); both rel32 placeholders live in _start's body
    // at the recorded text offsets.
    obj.add_relocation(
        text,
        Relocation {
            offset: call_hole as u64,
            symbol: ac_start_sym,
            addend: -4,
            flags: RelocationFlags::Elf {
                r_type: R_X86_64_PLT32,
            },
        },
    )
    .map_err(|e| MnError::Backend(format!("linux support call reloc: {e}")))?;
    obj.add_relocation(
        text,
        Relocation {
            offset: lea_hole as u64,
            symbol: pagesz_sym,
            addend: -4,
            flags: RelocationFlags::Elf {
                r_type: R_X86_64_PC32,
            },
        },
    )
    .map_err(|e| MnError::Backend(format!("linux support pagesz reloc: {e}")))?;

    let bytes = obj
        .write()
        .map_err(|e| MnError::Backend(format!("write linux support object: {e}")))?;
    if bytes.len() > MAX_COFF_BYTES {
        return Err(MnError::Resource(
            "coff-size",
            format!(
                "linux support object {} bytes above the frozen cap",
                bytes.len()
            ),
        ));
    }
    Ok(bytes)
}
