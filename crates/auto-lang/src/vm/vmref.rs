// Plan 091: VmRefData extracted from universe.rs
// Enum-based storage for VM references, avoiding TypeId/downcasting issues

use std::fs::File;
use std::io::BufReader;

use crate::universe::ListData;
use crate::vm::builder::StringBuilderData;
use crate::vm::collections::{BTreeMapData, HashMapData, HashSetData, VecDequeData};
use crate::vm::object_data::ObjectData;

/// Enum-based storage for VM references, avoiding TypeId/downcasting issues
#[derive(Debug)]
pub enum VmRefData {
    HashMap(HashMapData),
    HashSet(HashSetData),
    BTreeMap(BTreeMapData), // Plan 085: Ordered map using Rust's BTreeMap
    VecDeque(VecDequeData), // Plan 085: Double-ended queue using Rust's VecDeque
    StringBuilder(StringBuilderData),
    File(BufReader<File>),
    List(ListData),
    Object(ObjectData), // Plan 073: Object literal support
}
