//! PLAN-871 T-06: the RELOCATABLE-COFF structural admission checker, split
//! out of the retiring `native_admission` module. It reads the object
//! FORMAT only (machine/sections/symbol table/string table ranges) — it does
//! not lower, link, or know which backend produced the bytes — so the
//! shared-MIR A09' restore path keeps the same structural gate the old
//! native admission had (the lowering-backed positive control retires with
//! the old lowering).

/// Why a byte sequence is not an admissible relocatable COFF object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoffAdmissionError {
    MalformedObject(&'static str),
}

impl std::fmt::Display for CoffAdmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoffAdmissionError::MalformedObject(w) => {
                write!(f, "coff admission: malformed object ({w})")
            }
        }
    }
}

impl std::error::Error for CoffAdmissionError {}

/// Validate ytes as a RELOCATABLE COFF machine object (AMD64, sections
/// present, symbol table present, NOT a linked image) with every claimed
/// window (section headers, raw data, relocations, symbol/string tables)
/// actually fitting the offered bytes.
pub fn check_coff_relocatable(bytes: &[u8]) -> Result<(), CoffAdmissionError> {
    const SYM_SIZE: u64 = 18;
    const SECTION_HEADER_SIZE: u64 = 40;
    if (bytes.len() as u64) < 20 {
        return Err(CoffAdmissionError::MalformedObject(
            "shorter than a COFF header",
        ));
    }
    if bytes[0] == b'M' && bytes[1] == b'Z' {
        return Err(CoffAdmissionError::MalformedObject(
            "MZ/PE image bytes are not a relocatable object",
        ));
    }
    let machine = u16::from_le_bytes([bytes[0], bytes[1]]);
    if machine != 0x8664 {
        return Err(CoffAdmissionError::MalformedObject("machine is not AMD64"));
    }
    let n_sections = u16::from_le_bytes([bytes[2], bytes[3]]) as u64;
    if n_sections == 0 {
        return Err(CoffAdmissionError::MalformedObject("no sections"));
    }
    let n_symbols = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as u64;
    if n_symbols == 0 {
        return Err(CoffAdmissionError::MalformedObject(
            "no symbol table records — not a linkable object",
        ));
    }
    let size_of_optional = u16::from_le_bytes([bytes[16], bytes[17]]) as u64;
    if size_of_optional != 0 {
        return Err(CoffAdmissionError::MalformedObject(
            "optional header present — object form required for a relocatable reference",
        ));
    }
    let len = bytes.len() as u64;
    // Section headers must fit between the COFF header and the end.
    let sections_end = 20 + n_sections * SECTION_HEADER_SIZE;
    if sections_end > len {
        return Err(CoffAdmissionError::MalformedObject(
            "section table overruns the object",
        ));
    }
    let sym_ptr = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as u64;
    if sym_ptr == 0 {
        return Err(CoffAdmissionError::MalformedObject(
            "no symbol table — not a linkable object",
        ));
    }
    let symbols_end = sym_ptr
        .checked_add(n_symbols.checked_mul(SYM_SIZE).expect("bounded sizes"))
        .expect("bounded offsets");
    if symbols_end > len {
        return Err(CoffAdmissionError::MalformedObject(
            "symbol table overruns the object",
        ));
    }
    // F798-R7-04: every section header's claimed raw-data and relocation
    // windows must fit the offered bytes — a header declaring data at an
    // out-of-range pointer is not a valid relocatable object, whatever the
    // outer tables claim.
    for si in 0..n_sections {
        let sh = (20 + si * SECTION_HEADER_SIZE) as usize;
        let u16_at = |o: usize| u16::from_le_bytes([bytes[sh + o], bytes[sh + o + 1]]);
        let u32_at = |o: usize| {
            u32::from_le_bytes([
                bytes[sh + o],
                bytes[sh + o + 1],
                bytes[sh + o + 2],
                bytes[sh + o + 3],
            ]) as u64
        };
        let size_of_raw = u32_at(16);
        let ptr_raw = u32_at(20);
        if size_of_raw > 0 {
            // saturating: a u32::MAX pointer plus a size must reject, not
            // overflow the checked add.
            let end = ptr_raw.saturating_add(size_of_raw);
            if end > len || ptr_raw < sections_end {
                return Err(CoffAdmissionError::MalformedObject(
                    "section raw-data window overruns the object",
                ));
            }
        }
        let n_relocs = u16_at(32) as u64;
        if n_relocs > 0 {
            let ptr_rel = u32_at(24);
            let end = ptr_rel
                .checked_add(n_relocs.saturating_mul(10))
                .expect("bounded sizes");
            if end > len {
                return Err(CoffAdmissionError::MalformedObject(
                    "section relocation table overruns the object",
                ));
            }
        }
    }
    // The string table follows the symbol table: a u32 byte length that
    // must close exactly at the object end (≥4 for the length field).
    if symbols_end + 4 <= len {
        let str_len = u32::from_le_bytes([
            bytes[symbols_end as usize],
            bytes[symbols_end as usize + 1],
            bytes[symbols_end as usize + 2],
            bytes[symbols_end as usize + 3],
        ]) as u64;
        if str_len < 4 || symbols_end + str_len != len {
            return Err(CoffAdmissionError::MalformedObject(
                "string table length does not close the object",
            ));
        }
    } else if symbols_end != len {
        return Err(CoffAdmissionError::MalformedObject(
            "symbol table does not close the object",
        ));
    }
    Ok(())
}
