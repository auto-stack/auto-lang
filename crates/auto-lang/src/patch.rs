// =============================================================================
// Patch Generation: Hot-reloadable code patches
// =============================================================================
//
// Phase 3.5: Generate patches instead of full binaries for AutoLive
//
// Patches are incremental code updates that can be applied to a running
// system without full restart. Each patch contains:
// - Fragment ID (which function to update)
// - Code size (for validation)
// - Machine code/bytecode
// - Relocation table (for symbol resolution)
//
// This enables 亚秒级热重载 (sub-second hot reload) for AutoLive.

use crate::database::FragId;
use crate::error::AutoResult;
use crate::scope::Sid;

/// A code patch for hot reloading
///
/// Patches are generated for individual fragments (functions) and can be
/// applied to a running system without restarting.
///
/// # Architecture
///
/// ```text
/// Fragment → Codegen → Patch → Runtime
///                      ↓
///                   Apply to memory
///                      ↓
///                   Update function pointer
/// ```
///
/// # Phase 3.5: Basic Patch Structure
///
/// Currently stores bytecode/bytecode-level patches.
/// Future enhancements: machine code patches, verification metadata.
#[derive(Debug, Clone)]
pub struct Patch {
    /// Fragment this patch updates
    pub frag_id: FragId,

    /// Size of code (for validation)
    pub code_size: u32,

    /// Machine code or bytecode
    pub code: Vec<u8>,

    /// Relocation entries (for symbol resolution)
    pub relocs: Vec<Reloc>,
}

impl Patch {
    /// Create a new patch
    pub fn new(frag_id: FragId, code: Vec<u8>, relocs: Vec<Reloc>) -> Self {
        let code_size = code.len() as u32;
        Self {
            frag_id,
            code_size,
            code,
            relocs,
        }
    }

    /// Validate the patch
    ///
    /// Checks that:
    /// - Code size matches actual code length
    /// - Relocations are within code bounds
    pub fn validate(&self) -> bool {
        // Check code size
        if self.code_size as usize != self.code.len() {
            return false;
        }

        // Check relocation offsets
        for reloc in &self.relocs {
            if reloc.offset as usize >= self.code.len() {
                return false;
            }
        }

        true
    }
}

/// Relocation entry
///
/// Relocations describe where symbols need to be resolved in the code.
/// When a patch is applied, the runtime resolves each symbol reference
/// and patches the address into the code.
///
/// # Example
///
/// ```text
/// Code:  CALL @extern_function
/// Reloc: offset=10, symbol="extern_function", kind=RelocKind::PLT
///
/// After applying patch:
/// Code:  CALL 0x12345678  (address resolved)
/// ```
#[derive(Debug, Clone)]
pub struct Reloc {
    /// Offset in code where relocation should be applied
    pub offset: u32,

    /// Symbol to resolve
    pub symbol: Sid,

    /// Type of relocation
    pub kind: RelocKind,
}

impl Reloc {
    /// Create a new relocation
    pub fn new(offset: u32, symbol: Sid, kind: RelocKind) -> Self {
        Self {
            offset,
            symbol,
            kind,
        }
    }
}

/// Relocation kind
///
/// Different relocation types for different addressing modes.
/// Maps to standard relocation types in object file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelocKind {
    /// Absolute relocation (4-byte address)
    ///
    /// Used for: Direct function calls, global data access
    ///
    /// Example: `CALL 0x12345678`
    Abs,

    /// Relative relocation (4-byte offset)
    ///
    /// Used for: PC-relative branches, nearby jumps
    ///
    /// Example: `JMP +0x100`
    Rel,

    /// Global Offset Table entry
    ///
    /// Used for: PIC (Position Independent Code) access
    ///
    /// Example: `CALL [GOT + symbol_offset]`
    GOT,

    /// Procedure Linkage Table entry
    ///
    /// Used for: Lazy dynamic linking, extern functions
    ///
    /// Example: `CALL PLT_symbol`
    PLT,
}

impl RelocKind {
    /// Get the size of this relocation in bytes
    pub fn size(&self) -> usize {
        match self {
            RelocKind::Abs => 4,   // 32-bit absolute address
            RelocKind::Rel => 4,   // 32-bit relative offset
            RelocKind::GOT => 4,   // 32-bit GOT entry
            RelocKind::PLT => 4,   // 32-bit PLT entry
        }
    }
}

// =============================================================================
// Patch Generation
// =============================================================================

/// Generate a code patch for a fragment
///
/// Phase 3.5: Basic implementation that creates a placeholder patch.
/// Future phases will integrate with actual codegen.
///
/// # Arguments
///
/// * `frag_id` - The fragment to generate a patch for
///
/// # Returns
///
/// A patch containing the fragment's code and relocations
///
/// # Example
///
/// ```ignore
/// let patch = generate_patch(frag_id, &db)?;
/// runtime.apply_patch(patch)?;
/// ```
pub fn generate_patch(
    frag_id: FragId,
    _db: &crate::database::Database,
) -> AutoResult<Patch> {
    // Phase 3.5: Create placeholder patch
    // Future: Integrate with codegen to generate actual bytecode

    Ok(Patch {
        frag_id: frag_id.clone(),
        code_size: 0,
        code: vec![],
        relocs: vec![],
    })
}

/// Apply a patch to a running system
///
/// Phase 3.5: Placeholder for runtime integration.
/// Will be implemented in Phase 3.6 (MCU Runtime Integration).
///
/// # Arguments
///
/// * `patch` - The patch to apply
///
/// # Returns
///
/// Success if patch was applied
pub fn apply_patch(_patch: &Patch) -> AutoResult<()> {
    // Phase 3.6: Implement runtime patch application
    Ok(())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_new() {
        let frag_id = crate::database::FragId::new(
            crate::database::FileId::new(1),
            100,
        );

        let code = vec![0x01, 0x02, 0x03, 0x04];
        let relocs = vec![];

        let patch = Patch::new(frag_id, code.clone(), relocs);

        assert_eq!(patch.code_size, 4);
        assert_eq!(patch.code, code);
        assert!(patch.relocs.is_empty());
    }

    #[test]
    fn test_patch_validate_valid() {
        let frag_id = crate::database::FragId::new(
            crate::database::FileId::new(1),
            100,
        );

        // Code is 8 bytes, relocs at offsets 0 and 4
        let code = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let relocs = vec![
            Reloc::new(0, Sid::from("test_func"), RelocKind::Abs),
            Reloc::new(4, Sid::from("test_func2"), RelocKind::Rel),
        ];

        let patch = Patch::new(frag_id, code, relocs);

        assert!(patch.validate());
    }

    #[test]
    fn test_patch_validate_invalid_size() {
        let frag_id = crate::database::FragId::new(
            crate::database::FileId::new(1),
            100,
        );

        let code = vec![0x01, 0x02, 0x03, 0x04];
        let relocs = vec![];

        let mut patch = Patch::new(frag_id, code, relocs);
        patch.code_size = 999; // Wrong size!

        assert!(!patch.validate());
    }

    #[test]
    fn test_patch_validate_invalid_reloc_offset() {
        let frag_id = crate::database::FragId::new(
            crate::database::FileId::new(1),
            100,
        );

        let code = vec![0x01, 0x02, 0x03, 0x04];
        let relocs = vec![
            Reloc::new(0, Sid::from("test_func"), RelocKind::Abs),
            Reloc::new(999, Sid::from("test_func2"), RelocKind::Rel), // Out of bounds!
        ];

        let patch = Patch::new(frag_id, code, relocs);

        assert!(!patch.validate());
    }

    #[test]
    fn test_reloc_new() {
        let reloc = Reloc::new(
            100,
            Sid::from("test_function"),
            RelocKind::Abs,
        );

        assert_eq!(reloc.offset, 100);
        assert_eq!(format!("{}", reloc.symbol), "test_function");
        assert_eq!(reloc.kind, RelocKind::Abs);
    }

    #[test]
    fn test_reloc_kind_size() {
        assert_eq!(RelocKind::Abs.size(), 4);
        assert_eq!(RelocKind::Rel.size(), 4);
        assert_eq!(RelocKind::GOT.size(), 4);
        assert_eq!(RelocKind::PLT.size(), 4);
    }

    #[test]
    fn test_generate_patch_placeholder() {
        let db = crate::database::Database::new();
        let frag_id = crate::database::FragId::new(
            crate::database::FileId::new(1),
            100,
        );

        let result = generate_patch(frag_id, &db);
        assert!(result.is_ok());

        let patch = result.unwrap();
        assert_eq!(patch.code_size, 0);
        assert!(patch.code.is_empty());
        assert!(patch.relocs.is_empty());
    }

    #[test]
    fn test_apply_patch_placeholder() {
        let frag_id = crate::database::FragId::new(
            crate::database::FileId::new(1),
            100,
        );

        let patch = Patch::new(frag_id, vec![0x01, 0x02], vec![]);
        let result = apply_patch(&patch);
        assert!(result.is_ok());
    }
}
