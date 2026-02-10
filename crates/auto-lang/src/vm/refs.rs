// Plan 088 Phase 5: Reference types for parameter passing

/// Immutable reference to a local variable
#[derive(Debug, Clone)]
pub struct VmRef {
    pub var_index: u32,
}

/// Mutable reference to a local variable
#[derive(Debug, Clone)]
pub struct VmMutRef {
    pub var_index: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_ref_creation() {
        let vm_ref = VmRef { var_index: 0 };
        assert_eq!(vm_ref.var_index, 0);
    }

    #[test]
    fn test_vm_mut_ref_creation() {
        let vm_mut_ref = VmMutRef { var_index: 1 };
        assert_eq!(vm_mut_ref.var_index, 1);
    }

    #[test]
    fn test_vm_ref_clone() {
        let vm_ref = VmRef { var_index: 5 };
        let cloned = vm_ref.clone();
        assert_eq!(cloned.var_index, 5);
    }

    #[test]
    fn test_vm_mut_ref_clone() {
        let vm_mut_ref = VmMutRef { var_index: 3 };
        let cloned = vm_mut_ref.clone();
        assert_eq!(cloned.var_index, 3);
    }
}
