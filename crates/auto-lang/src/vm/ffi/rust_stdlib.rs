//! Plan 192: Rust stdlib type wrappers for dynamic dispatch
//!
//! Stores Rust stdlib types (Instant, Duration, PathBuf, etc.) as opaque
//! heap objects so they can be passed around in the VM.

use std::any::Any;
use crate::vm::heap_object::{HeapObject, TypeTag};

/// Wrapper for any Rust stdlib type stored in the VM heap.
pub struct RustStdlibObject {
    pub type_name: String,
    pub value: Box<dyn Any + Send + Sync>,
}

impl RustStdlibObject {
    pub fn new<T: Any + Send + Sync + 'static>(type_name: &str, value: T) -> Self {
        Self {
            type_name: type_name.to_string(),
            value: Box::new(value),
        }
    }

    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.value.as_ref().downcast_ref::<T>()
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.value.as_mut().downcast_mut::<T>()
    }
}

impl HeapObject for RustStdlibObject {
    fn type_tag(&self) -> TypeTag {
        TypeTag::RustStdlib(self.type_name.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
