//! PLAN-1040: the mn target model. The Windows lane stays the FROZEN
//! `x86_64-pc-windows-msvc` object/ABI it always was; this module adds
//! the Linux x86-64 ELF target (`x86_64-unknown-linux-gnu`) as a second
//! first-class target — never the host triple silently, never a cache
//! axis collapse between the two.

use cranelift_codegen::isa::CallConv;

/// The admitted mn targets. Every emit/link/cache/manifest axis carries
/// the target's triple string; `WinMsvc` keeps the frozen PLAN-818
/// constants bit-for-bit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MnOsTarget {
    WinMsvc,
    LinuxGnu,
}

impl MnOsTarget {
    /// The frozen target triple (identity axis in A09'/A10'/A17').
    pub fn triple(self) -> &'static str {
        match self {
            MnOsTarget::WinMsvc => crate::config::MN_TARGET,
            MnOsTarget::LinuxGnu => crate::config::MN_TARGET_LINUX,
        }
    }

    /// The private ABI tag (identity axis; targets never share an ABI tag).
    pub fn abi_tag(self) -> &'static str {
        match self {
            MnOsTarget::WinMsvc => crate::config::MN_ABI,
            MnOsTarget::LinuxGnu => crate::config::MN_ABI_LINUX,
        }
    }

    /// The runtime policy tag (identity axis).
    pub fn runtime_tag(self) -> &'static str {
        match self {
            MnOsTarget::WinMsvc => crate::config::MN_RUNTIME_TAG,
            MnOsTarget::LinuxGnu => crate::config::MN_RUNTIME_TAG_LINUX,
        }
    }

    /// Cranelift call convention of this target's private ABI.
    pub fn call_conv(self) -> CallConv {
        match self {
            MnOsTarget::WinMsvc => CallConv::WindowsFastcall,
            MnOsTarget::LinuxGnu => CallConv::SystemV,
        }
    }

    /// The import the trap path calls instead of a CRT exit: Windows
    /// keeps `ExitProcess`; Linux targets the raw `exit_group` stub
    /// `mn_exit` defined by the 1040 linux support object.
    pub fn exit_import(self) -> &'static str {
        match self {
            MnOsTarget::WinMsvc => "ExitProcess",
            MnOsTarget::LinuxGnu => "mn_exit",
        }
    }

    /// The process entry the OS jumps to: Windows links `ac_start` as the
    /// PE entry directly; Linux links the raw `_start` trampoline of the
    /// support object (stack align + auxv page-size read + `call ac_start`).
    pub fn link_entry(self) -> &'static str {
        match self {
            MnOsTarget::WinMsvc => crate::config::START_SYMBOL,
            MnOsTarget::LinuxGnu => crate::config::LINUX_START_TRAMPOLINE,
        }
    }

    /// True when this target's products are ELF64 x86-64 objects.
    pub fn is_elf(self) -> bool {
        matches!(self, MnOsTarget::LinuxGnu)
    }
}
