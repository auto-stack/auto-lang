//! PLAN-1040: the RELOCATABLE-ELF structural admission checker — the ELF
//! counterpart of [`crate::coff_admission`] (PLAN-871 T-06). It reads the
//! object FORMAT only (magic/class/machine/type, section table, symbol
//! table presence, every claimed window fitting the offered bytes) — it
//! does not lower, link, or know which backend produced the bytes — so
//! the shared-MIR A09' ELF restore path gets the same structural gate
//! the COFF restore has. A COFF payload offered to an ELF slot (or the
//! reverse) fails here on magic.

/// Why a byte sequence is not an admissible relocatable ELF64 object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ElfAdmissionError {
    MalformedObject(&'static str),
}

impl std::fmt::Display for ElfAdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElfAdmissionError::MalformedObject(w) => {
                write!(f, "elf admission: malformed object ({w})")
            }
        }
    }
}

impl std::error::Error for ElfAdmissionError {}

/// Validate bytes as a RELOCATABLE ELF64 x86-64 object (ET_REL, section
/// table present, a non-empty symbol table, no program headers, NOT a
/// linked image) with every claimed window actually fitting the offered
/// bytes.
pub fn check_elf_relocatable(bytes: &[u8]) -> Result<(), ElfAdmissionError> {
    const SECTION_HEADER_SIZE: u64 = 64;
    const SHT_SYMTAB: u32 = 2;
    const SHT_NOBITS: u32 = 8;
    const ET_REL: u16 = 1;
    const EM_X86_64: u16 = 62;
    const err: fn(&'static str) -> ElfAdmissionError = ElfAdmissionError::MalformedObject;

    if (bytes.len() as u64) < 64 {
        return Err(err("shorter than an ELF64 header"));
    }
    if !(bytes[0] == 0x7F && bytes[1] == b'E' && bytes[2] == b'L' && bytes[3] == b'F') {
        return Err(err("ELF magic missing (not an ELF object)"));
    }
    if bytes[4] != 2 {
        return Err(err("ELFCLASS32 — object must be ELF64"));
    }
    if bytes[5] != 1 {
        return Err(err("not little-endian"));
    }
    let u16_at = |o: usize| u16::from_le_bytes([bytes[o], bytes[o + 1]]);
    let u32_at =
        |o: usize| u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
    let u64_at = |o: usize| {
        u64::from_le_bytes([
            bytes[o],
            bytes[o + 1],
            bytes[o + 2],
            bytes[o + 3],
            bytes[o + 4],
            bytes[o + 5],
            bytes[o + 6],
            bytes[o + 7],
        ])
    };
    let e_type = u16_at(16);
    if e_type != ET_REL {
        return Err(match e_type {
            2 => err("ET_EXEC image bytes are not a relocatable object"),
            3 => err("ET_DYN image bytes are not a relocatable object"),
            _ => err("e_type is not ET_REL"),
        });
    }
    if u16_at(18) != EM_X86_64 {
        return Err(err("e_machine is not EM_X86_64"));
    }
    if u16_at(56) != 0 {
        return Err(err("program headers present — object form required"));
    }
    let shoff = u64_at(40);
    let shnum = u16_at(60) as u64;
    if shoff == 0 || shnum == 0 {
        return Err(err("no section table — not a linkable object"));
    }
    let len = bytes.len() as u64;
    let sections_end = shoff.saturating_add(shnum.saturating_mul(SECTION_HEADER_SIZE));
    if sections_end > len {
        return Err(err("section table overruns the object"));
    }
    let mut has_symbols = false;
    for si in 0..shnum {
        let sh = (shoff + si * SECTION_HEADER_SIZE) as usize;
        let sh_type = u32_at(sh + 4);
        let sh_offset = u64_at(sh + 24);
        let sh_size = u64_at(sh + 32);
        if sh_type == SHT_SYMTAB {
            if sh_size == 0 {
                return Err(err("empty symbol table — not a linkable object"));
            }
            has_symbols = true;
        }
        if sh_type == SHT_NOBITS {
            continue; // no file window
        }
        if sh_size > 0 {
            let end = sh_offset.saturating_add(sh_size);
            if end > len {
                return Err(err("section raw-data window overruns the object"));
            }
        }
    }
    if !has_symbols {
        return Err(err("no symbol table — not a linkable object"));
    }
    Ok(())
}
