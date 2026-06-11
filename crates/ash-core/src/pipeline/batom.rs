//! Batom (Binary Atom) — high-performance binary encoding for Atom data.
//!
//! Batom provides a compact binary representation for Atom/AtomPipeline data,
//! designed for fast serialization and deserialization in cross-command pipelines.
//!
//! ## Binary format
//!
//! ```text
//! ┌──────────┬───────────┬──────────────────────────────────┐
//! │ Magic    │ Header    │ Payload                          │
//! │ 4 bytes  │ 16 bytes  │ Variable                         │
//! │ "BATM"   │           │                                  │
//! └──────────┴───────────┴──────────────────────────────────┘
//!
//! Header (16 bytes):
//!   version:               u8    (currently 1)
//!   flags:                 u8    (bit 0: reserved)
//!   atom_type_tag:         u8    (AtomType as u8)
//!   _reserved:             u8
//!   string_table_offset:   u32   (byte offset from start)
//!   string_table_entries:  u32   (number of deduplicated strings)
//!   payload_size:          u32   (bytes of payload section)
//!
//! Payload:
//!   1. String table: [u32 len, u8[len] bytes] × N
//!   2. Value tree (recursive, using tags)
//! ```
//!
//! ## Design decisions
//!
//! - **No serde dependency** — keeps ash-core dependency-light
//! - **String deduplication** — repeated strings stored once, referenced by index
//! - **Little-endian** — matches modern CPUs
//! - **Single-pass encode** — collect strings first, then encode in one pass
//! - **Focused on pipeline types** — only encodes Value variants that appear in
//!   shell pipelines (primitives, strings, arrays, objects, etc.)

use auto_val::Value;
use std::collections::HashMap;

use super::atom::{Atom, AtomType};
use super::atom_pipeline::AtomPipeline;
use super::atom_stream::AtomStream;

// ── Constants ──────────────────────────────────────────────

/// Magic number identifying a Batom binary blob.
const BATM_MAGIC: [u8; 4] = *b"BATM";

/// Current format version.
const BATM_VERSION: u8 = 1;

/// Header size in bytes (magic excluded, since magic is checked separately).
/// version(1) + flags(1) + atom_type_tag(1) + reserved(1) +
/// string_table_offset(4) + string_table_entries(4) + payload_size(4)
const HEADER_SIZE: usize = 16;

// ── Value type tags (single byte) ─────────────────────────

const TAG_NIL: u8 = 0x00;
const TAG_VOID: u8 = 0x01;
const TAG_NULL: u8 = 0x02;
const TAG_BOOL: u8 = 0x03;
const TAG_INT: u8 = 0x04;
const TAG_UINT: u8 = 0x05;
const TAG_I64: u8 = 0x06;
const TAG_FLOAT: u8 = 0x07;
const TAG_ARRAY: u8 = 0x09;
const TAG_OBJ: u8 = 0x0A;
const TAG_RANGE: u8 = 0x0B;
const TAG_RANGE_EQ: u8 = 0x0C;
const TAG_SOME: u8 = 0x0D;
const TAG_NONE: u8 = 0x0E;
const TAG_OK: u8 = 0x0F;
const TAG_ERR: u8 = 0x10;
const TAG_PAIR: u8 = 0x11;
const TAG_GRID: u8 = 0x12;
const TAG_BYTE: u8 = 0x13;
const TAG_USIZE: u8 = 0x14;
const TAG_CHAR: u8 = 0x15;
const TAG_ERROR: u8 = 0x16;

// ── String table reference tag ────────────────────────────

/// Inline string: tag byte + u16 length + bytes
const TAG_STRING_INLINE: u8 = 0x20;
/// String table reference: tag byte + u16 index
const TAG_STRING_REF: u8 = 0x21;

// ── Atom type tags ────────────────────────────────────────

const ATOM_FILE_ENTRY: u8 = 0x01;
const ATOM_FILE_LIST: u8 = 0x02;
const ATOM_PROCESS_ENTRY: u8 = 0x03;
const ATOM_PROCESS_LIST: u8 = 0x04;
const ATOM_DISK_ENTRY: u8 = 0x05;
const ATOM_CPU_INFO: u8 = 0x06;
const ATOM_MEMORY_INFO: u8 = 0x07;
const ATOM_SYSTEM_INFO: u8 = 0x08;
const ATOM_MATCH_LIST: u8 = 0x09;
const ATOM_COUNT_RESULT: u8 = 0x0A;
const ATOM_TABLE: u8 = 0x0B;
const ATOM_RECORD: u8 = 0x0C;
const ATOM_TEXT: u8 = 0x0D;
const ATOM_PATH: u8 = 0x0E;
const ATOM_BUILD_RESULT: u8 = 0x0F;
const ATOM_RUN_RESULT: u8 = 0x10;
const ATOM_HELP_INFO: u8 = 0x11;
const ATOM_NOTHING: u8 = 0x00;

// ── Errors ────────────────────────────────────────────────

/// Errors that can occur during Batom encoding/decoding.
#[derive(Debug, thiserror::Error)]
pub enum BatomError {
    #[error("invalid magic: expected BATM, got {0:?}")]
    InvalidMagic([u8; 4]),
    #[error("unsupported version: {0}")]
    UnsupportedVersion(u8),
    #[error("unexpected end of data at offset {0}")]
    UnexpectedEof(usize),
    #[error("invalid tag byte: 0x{0:02X}")]
    InvalidTag(u8),
    #[error("invalid atom type tag: 0x{0:02X}")]
    InvalidAtomType(u8),
    #[error("string table index out of range: {index} (max {max})")]
    StringIndexOutOfRange { index: usize, max: usize },
    #[error("string table offset {offset} exceeds data length {len}")]
    BadStringTableOffset { offset: usize, len: usize },
    #[error("payload too large: {0} bytes")]
    PayloadTooLarge(usize),
}

// ── String Table ──────────────────────────────────────────

/// Collects unique strings during encoding for deduplication.
struct StringTable {
    /// Ordered list of unique strings.
    strings: Vec<String>,
    /// Map from string content → index in the table.
    index: HashMap<String, u16>,
}

impl StringTable {
    fn new() -> Self {
        Self {
            strings: Vec::new(),
            index: HashMap::new(),
        }
    }

    /// Insert a string, returning its index. Deduplicates automatically.
    fn insert(&mut self, s: &str) -> u16 {
        if let Some(&idx) = self.index.get(s) {
            idx
        } else {
            let idx = self.strings.len() as u16;
            self.strings.push(s.to_string());
            self.index.insert(s.to_string(), idx);
            idx
        }
    }

    /// Recursively collect all strings from a Value tree.
    fn collect_from_value(&mut self, value: &Value) {
        match value {
            Value::Str(_) | Value::String(_) | Value::StrSlice(_) | Value::CStr(_) => {
                self.insert(value.as_str());
            }
            Value::Array(arr) | Value::Block(arr) => {
                for item in &arr.values {
                    self.collect_from_value(item);
                }
            }
            Value::Obj(obj) => {
                for (k, v) in obj.iter() {
                    if let auto_val::ValueKey::Str(s) = k {
                        self.insert(s.as_str());
                    }
                    self.collect_from_value(v);
                }
            }
            Value::Pair(k, v) => {
                if let auto_val::ValueKey::Str(s) = k {
                    self.insert(s.as_str());
                }
                self.collect_from_value(v);
            }
            Value::Some(v) | Value::Ok(v) => {
                self.collect_from_value(v);
            }
            Value::Err(s) | Value::Error(s) => {
                self.insert(s.as_str());
            }
            Value::Grid(grid) => {
                for (k, v) in &grid.head {
                    if let auto_val::ValueKey::Str(s) = k {
                        self.insert(s.as_str());
                    }
                    self.collect_from_value(v);
                }
                for row in &grid.data {
                    for cell in row {
                        self.collect_from_value(cell);
                    }
                }
            }
            Value::Range(_, _) | Value::RangeEq(_, _) => {
                // Integer ranges — no strings
            }
            // Primitives and non-string types: nothing to collect
            _ => {}
        }
    }
}

// ── Encoder ───────────────────────────────────────────────

/// Batom binary encoder.
pub struct BatomEncoder {
    buf: Vec<u8>,
    string_table: StringTable,
}

impl BatomEncoder {
    /// Create a new encoder with pre-allocated buffer.
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(256),
            string_table: StringTable::new(),
        }
    }

    // ── Public API ─────────────────────────────────────

    /// Encode an Atom into Batom binary format.
    pub fn encode_atom(&mut self, atom: &Atom) -> Result<Vec<u8>, BatomError> {
        self.buf.clear();
        self.string_table = StringTable::new();

        // Phase 1: Collect all strings
        self.string_table.collect_from_value(&atom.value);

        // Phase 2: Reserve space for magic + header
        self.buf.resize(4 + HEADER_SIZE, 0);

        // Write magic
        self.buf[0..4].copy_from_slice(&BATM_MAGIC);

        // Phase 3: Write payload (value tree)
        let payload_start = self.buf.len();
        self.encode_value(&atom.value)?;

        // Phase 4: Write string table
        let string_table_offset = self.buf.len() as u32;
        let string_table_entries = self.string_table.strings.len() as u32;
        self.encode_string_table();

        let payload_size = (string_table_offset - payload_start as u32) as u32;

        // Phase 5: Fill header
        let header_start = 4;
        self.buf[header_start] = BATM_VERSION;          // version
        self.buf[header_start + 1] = 0;                  // flags
        self.buf[header_start + 2] = atom_type_to_u8(atom.atom_type); // atom type
        self.buf[header_start + 3] = 0;                  // reserved
        self.write_u32_le(header_start + 4, string_table_offset);
        self.write_u32_le(header_start + 8, string_table_entries);
        self.write_u32_le(header_start + 12, payload_size);

        Ok(self.buf.clone())
    }

    /// Encode an AtomPipeline into Batom binary format.
    pub fn encode_pipeline(&mut self, pipeline: &AtomPipeline) -> Result<Vec<u8>, BatomError> {
        match pipeline {
            AtomPipeline::Atom(atom) => self.encode_atom(atom),
            AtomPipeline::Stream(stream) => {
                // Encode as a stream: header with Stream marker, then N atoms
                self.encode_stream(stream)
            }
            AtomPipeline::Text(text) => {
                let atom = Atom::text(text);
                self.encode_atom(&atom)
            }
            AtomPipeline::Empty => {
                let atom = Atom::empty();
                self.encode_atom(&atom)
            }
        }
    }

    /// Encode a stream of atoms.
    pub fn encode_stream(&mut self, stream: &AtomStream) -> Result<Vec<u8>, BatomError> {
        // For streams, we encode each atom individually and prefix with a u32 count.
        // This allows incremental decoding.
        self.buf.clear();
        self.string_table = StringTable::new();

        // Collect strings from ALL atoms first for global dedup
        for atom in &stream.items {
            self.string_table.collect_from_value(&atom.value);
        }

        // Reserve: magic(4) + header(16) + count(4)
        self.buf.resize(4 + HEADER_SIZE + 4, 0);
        self.buf[0..4].copy_from_slice(&BATM_MAGIC);

        // Write atom count
        let count = stream.items.len() as u32;
        self.write_u32_le(4 + HEADER_SIZE, count);

        let payload_start = self.buf.len();

        // Encode each value
        for atom in &stream.items {
            self.buf.push(atom_type_to_u8(atom.atom_type));
            self.encode_value(&atom.value)?;
        }

        // String table
        let string_table_offset = self.buf.len() as u32;
        let string_table_entries = self.string_table.strings.len() as u32;
        self.encode_string_table();

        let payload_size = (string_table_offset - payload_start as u32) as u32;

        // Header — use special stream marker
        let hs = 4;
        self.buf[hs] = BATM_VERSION;
        self.buf[hs + 1] = 0x01; // flags bit 0 = stream
        self.buf[hs + 2] = ATOM_NOTHING; // stream has no single atom type
        self.buf[hs + 3] = 0;
        self.write_u32_le(hs + 4, string_table_offset);
        self.write_u32_le(hs + 8, string_table_entries);
        self.write_u32_le(hs + 12, payload_size);

        Ok(self.buf.clone())
    }

    // ── Internal: String table encoding ───────────────

    fn encode_string_table(&mut self) {
        for s in &self.string_table.strings {
            let bytes = s.as_bytes();
            let len = bytes.len() as u32;
            self.buf.extend_from_slice(&len.to_le_bytes());
            self.buf.extend_from_slice(bytes);
        }
    }

    // ── Internal: Value encoding ──────────────────────

    fn encode_value(&mut self, value: &Value) -> Result<(), BatomError> {
        match value {
            Value::Nil => self.buf.push(TAG_NIL),
            Value::Void => self.buf.push(TAG_VOID),
            Value::Null => self.buf.push(TAG_NULL),

            Value::Bool(b) => {
                self.buf.push(TAG_BOOL);
                self.buf.push(if *b { 1 } else { 0 });
            }

            Value::Int(i) => {
                self.buf.push(TAG_INT);
                self.buf.extend_from_slice(&i.to_le_bytes());
            }

            Value::Uint(u) => {
                self.buf.push(TAG_UINT);
                self.buf.extend_from_slice(&u.to_le_bytes());
            }

            Value::I64(i) => {
                self.buf.push(TAG_I64);
                self.buf.extend_from_slice(&i.to_le_bytes());
            }

            Value::USize(u) => {
                self.buf.push(TAG_USIZE);
                self.buf.extend_from_slice(&(*u as u64).to_le_bytes());
            }

            Value::Float(f) => {
                self.buf.push(TAG_FLOAT);
                self.buf.extend_from_slice(&f.to_le_bytes());
            }

            Value::Byte(b) | Value::U8(b) => {
                self.buf.push(TAG_BYTE);
                self.buf.push(*b);
            }

            Value::Char(c) => {
                self.buf.push(TAG_CHAR);
                let mut buf = [0u8; 4];
                let s = c.encode_utf8(&mut buf);
                let len = s.len() as u8;
                self.buf.push(len);
                self.buf.extend_from_slice(&buf[..len as usize]);
            }

            // String types — use string table reference
            Value::Str(_) | Value::String(_) | Value::StrSlice(_) | Value::CStr(_) => {
                self.encode_string(value.as_str());
            }

            Value::Array(arr) | Value::Block(arr) => {
                self.buf.push(TAG_ARRAY);
                let len = arr.values.len() as u32;
                self.buf.extend_from_slice(&len.to_le_bytes());
                for item in &arr.values {
                    self.encode_value(item)?;
                }
            }

            Value::Obj(obj) => {
                self.buf.push(TAG_OBJ);
                let len = obj.len() as u32;
                self.buf.extend_from_slice(&len.to_le_bytes());
                for (k, v) in obj.iter() {
                    // Key: encode based on ValueKey variant
                    match k {
                        auto_val::ValueKey::Str(s) => self.encode_string(s.as_str()),
                        auto_val::ValueKey::Int(i) => {
                            // Use TAG_INT + i32 bytes for int keys
                            self.buf.push(TAG_INT);
                            self.buf.extend_from_slice(&i.to_le_bytes());
                        }
                        auto_val::ValueKey::Bool(b) => {
                            self.buf.push(TAG_BOOL);
                            self.buf.push(if *b { 1 } else { 0 });
                        }
                    }
                    // Value
                    self.encode_value(v)?;
                }
            }

            Value::Pair(k, v) => {
                self.buf.push(TAG_PAIR);
                match k {
                    auto_val::ValueKey::Str(s) => {
                        self.buf.push(0); // string key
                        self.encode_string(s.as_str());
                    }
                    auto_val::ValueKey::Int(i) => {
                        self.buf.push(1); // int key
                        self.buf.extend_from_slice(&i.to_le_bytes());
                    }
                    auto_val::ValueKey::Bool(b) => {
                        self.buf.push(2); // bool key
                        self.buf.push(if *b { 1 } else { 0 });
                    }
                }
                self.encode_value(v)?;
            }

            Value::Grid(grid) => {
                self.buf.push(TAG_GRID);
                // Headers (head is Vec<(ValueKey, Value)>)
                let header_count = grid.head.len() as u32;
                self.buf.extend_from_slice(&header_count.to_le_bytes());
                for (k, v) in &grid.head {
                    // Encode key
                    match k {
                        auto_val::ValueKey::Str(s) => self.encode_string(s.as_str()),
                        auto_val::ValueKey::Int(i) => {
                            // Encode int key as special inline
                            self.buf.push(TAG_INT);
                            self.buf.extend_from_slice(&i.to_le_bytes());
                        }
                        _ => self.encode_string(&k.to_string()),
                    }
                    // Encode header value (usually contains column metadata)
                    self.encode_value(v)?;
                }
                // Rows (data is Vec<Vec<Value>>)
                let row_count = grid.data.len() as u32;
                self.buf.extend_from_slice(&row_count.to_le_bytes());
                for row in &grid.data {
                    for cell in row {
                        self.encode_value(cell)?;
                    }
                }
            }

            Value::Range(a, b) => {
                self.buf.push(TAG_RANGE);
                self.buf.extend_from_slice(&a.to_le_bytes());
                self.buf.extend_from_slice(&b.to_le_bytes());
            }

            Value::RangeEq(a, b) => {
                self.buf.push(TAG_RANGE_EQ);
                self.buf.extend_from_slice(&a.to_le_bytes());
                self.buf.extend_from_slice(&b.to_le_bytes());
            }

            Value::Some(v) => {
                self.buf.push(TAG_SOME);
                self.encode_value(v)?;
            }

            Value::None => {
                self.buf.push(TAG_NONE);
            }

            Value::Ok(v) => {
                self.buf.push(TAG_OK);
                self.encode_value(v)?;
            }

            Value::Err(s) => {
                self.buf.push(TAG_ERR);
                self.encode_string(s.as_str());
            }

            Value::Error(s) => {
                self.buf.push(TAG_ERROR);
                self.encode_string(s.as_str());
            }

            // Unsupported pipeline types — encode as Nil with a warning
            // These types (Fn, ExtFn, Type, Lambda, Node, Widget, Model, View,
            // Instance, Method, Args, Meta, Ref, VmRef, ValueRef, Closure, Future)
            // don't belong in shell pipelines.
            _ => {
                self.buf.push(TAG_NIL);
            }
        }
        Ok(())
    }

    fn encode_string(&mut self, s: &str) {
        if let Some(&idx) = self.string_table.index.get(s) {
            // Use string table reference (2 bytes)
            self.buf.push(TAG_STRING_REF);
            self.buf.extend_from_slice(&idx.to_le_bytes());
        } else {
            // Inline string (for strings added after table collection, shouldn't normally happen)
            self.buf.push(TAG_STRING_INLINE);
            let bytes = s.as_bytes();
            let len = bytes.len() as u16;
            self.buf.extend_from_slice(&len.to_le_bytes());
            self.buf.extend_from_slice(bytes);
        }
    }

    // ── Helpers ───────────────────────────────────────

    fn write_u32_le(&mut self, offset: usize, value: u32) {
        self.buf[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

impl Default for BatomEncoder {
    fn default() -> Self {
        Self::new()
    }
}

// ── Decoder ───────────────────────────────────────────────

/// Batom binary decoder.
pub struct BatomDecoder<'a> {
    data: &'a [u8],
    pos: usize,
    string_table: Vec<String>,
}

impl<'a> BatomDecoder<'a> {
    /// Create a new decoder wrapping the binary data.
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            string_table: Vec::new(),
        }
    }

    // ── Public API ─────────────────────────────────────

    /// Decode a Batom binary blob into an Atom.
    pub fn decode_atom(&mut self) -> Result<Atom, BatomError> {
        // Check magic
        if self.data.len() < 4 + HEADER_SIZE {
            return Err(BatomError::UnexpectedEof(self.pos));
        }
        let magic: [u8; 4] = self.data[0..4].try_into().unwrap();
        if magic != BATM_MAGIC {
            return Err(BatomError::InvalidMagic(magic));
        }

        // Read header
        let hs = 4; // header start
        let version = self.data[hs];
        if version != BATM_VERSION {
            return Err(BatomError::UnsupportedVersion(version));
        }
        let _flags = self.data[hs + 1];
        let atom_type_tag = self.data[hs + 2];
        let _reserved = self.data[hs + 3];
        let string_table_offset = u32_from_le(&self.data[hs + 4..hs + 8]) as usize;
        let string_table_entries = u32_from_le(&self.data[hs + 8..hs + 12]) as usize;
        let _payload_size = u32_from_le(&self.data[hs + 12..hs + 16]);

        // Parse atom type
        let atom_type = u8_to_atom_type(atom_type_tag)?;

        // Parse string table first (it's at the end)
        self.parse_string_table(string_table_offset, string_table_entries)?;

        // Parse payload (value tree starts after header)
        self.pos = 4 + HEADER_SIZE;
        let value = self.decode_value()?;

        Ok(Atom::new(value, atom_type))
    }

    /// Decode a Batom binary blob into an AtomPipeline.
    pub fn decode_pipeline(&mut self) -> Result<AtomPipeline, BatomError> {
        if self.data.len() < 4 + HEADER_SIZE {
            return Err(BatomError::UnexpectedEof(self.pos));
        }

        let flags = self.data[4 + 1];

        if flags & 0x01 != 0 {
            // Stream encoding
            let stream = self.decode_stream()?;
            Ok(AtomPipeline::from_stream(stream))
        } else {
            let atom = self.decode_atom()?;
            Ok(AtomPipeline::from_atom(atom))
        }
    }

    /// Decode a stream of atoms.
    pub fn decode_stream(&mut self) -> Result<AtomStream, BatomError> {
        // Check magic + header
        if self.data.len() < 4 + HEADER_SIZE + 4 {
            return Err(BatomError::UnexpectedEof(self.pos));
        }
        let magic: [u8; 4] = self.data[0..4].try_into().unwrap();
        if magic != BATM_MAGIC {
            return Err(BatomError::InvalidMagic(magic));
        }

        let hs = 4;
        let string_table_offset = u32_from_le(&self.data[hs + 4..hs + 8]) as usize;
        let string_table_entries = u32_from_le(&self.data[hs + 8..hs + 12]) as usize;

        // Parse string table
        self.parse_string_table(string_table_offset, string_table_entries)?;

        // Read atom count
        self.pos = 4 + HEADER_SIZE;
        let count = self.read_u32()? as usize;

        // Decode each atom
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            let atom_type_tag = self.read_u8()?;
            let atom_type = u8_to_atom_type(atom_type_tag)?;
            let value = self.decode_value()?;
            items.push(Atom::new(value, atom_type));
        }

        Ok(AtomStream::new(items))
    }

    // ── Internal: String table ────────────────────────

    fn parse_string_table(
        &mut self,
        offset: usize,
        count: usize,
    ) -> Result<(), BatomError> {
        if offset > self.data.len() {
            return Err(BatomError::BadStringTableOffset {
                offset,
                len: self.data.len(),
            });
        }

        self.string_table.clear();
        self.string_table.reserve(count);

        let mut pos = offset;
        for _ in 0..count {
            if pos + 4 > self.data.len() {
                return Err(BatomError::UnexpectedEof(pos));
            }
            let len = u32_from_le(&self.data[pos..pos + 4]) as usize;
            pos += 4;
            if pos + len > self.data.len() {
                return Err(BatomError::UnexpectedEof(pos));
            }
            let s = String::from_utf8_lossy(&self.data[pos..pos + len]).to_string();
            self.string_table.push(s);
            pos += len;
        }

        Ok(())
    }

    // ── Internal: Value decoding ──────────────────────

    fn decode_value(&mut self) -> Result<Value, BatomError> {
        let tag = self.read_u8()?;
        match tag {
            TAG_NIL => Ok(Value::Nil),
            TAG_VOID => Ok(Value::Void),
            TAG_NULL => Ok(Value::Null),

            TAG_BOOL => {
                let b = self.read_u8()?;
                Ok(Value::Bool(b != 0))
            }

            TAG_INT => {
                let bytes = self.read_bytes(4)?;
                Ok(Value::Int(i32_from_le(bytes)))
            }

            TAG_UINT => {
                let bytes = self.read_bytes(4)?;
                Ok(Value::Uint(u32_from_le(bytes)))
            }

            TAG_I64 => {
                let bytes = self.read_bytes(8)?;
                Ok(Value::I64(i64_from_le(bytes)))
            }

            TAG_USIZE => {
                let bytes = self.read_bytes(8)?;
                let v = u64_from_le(bytes);
                Ok(Value::USize(v as usize))
            }

            TAG_FLOAT => {
                let bytes = self.read_bytes(8)?;
                Ok(Value::Float(f64_from_le(bytes)))
            }

            TAG_BYTE => {
                let b = self.read_u8()?;
                Ok(Value::Byte(b))
            }

            TAG_CHAR => {
                let len = self.read_u8()? as usize;
                let bytes = self.read_bytes(len)?;
                let s = std::str::from_utf8(bytes)
                    .map_err(|_| BatomError::InvalidTag(TAG_CHAR))?;
                let c = s.chars().next().ok_or(BatomError::InvalidTag(TAG_CHAR))?;
                Ok(Value::Char(c))
            }

            TAG_STRING_INLINE => {
                let bytes = self.read_bytes(2)?;
                let len = u16_from_le(bytes) as usize;
                let str_bytes = self.read_bytes(len)?;
                let s = String::from_utf8_lossy(str_bytes).to_string();
                Ok(Value::str(&s))
            }

            TAG_STRING_REF => {
                let bytes = self.read_bytes(2)?;
                let idx = u16_from_le(bytes) as usize;
                if idx >= self.string_table.len() {
                    return Err(BatomError::StringIndexOutOfRange {
                        index: idx,
                        max: self.string_table.len(),
                    });
                }
                Ok(Value::str(&self.string_table[idx]))
            }

            TAG_ARRAY => {
                let len = self.read_u32()? as usize;
                let mut items = Vec::with_capacity(len.min(1024));
                for _ in 0..len {
                    items.push(self.decode_value()?);
                }
                Ok(Value::Array(items.into()))
            }

            TAG_OBJ => {
                let len = self.read_u32()? as usize;
                let mut obj = auto_val::Obj::new();
                for _ in 0..len {
                    // Key is encoded based on ValueKey variant (tag byte first)
                    let key_tag = self.read_u8()?;
                    let key_str = match key_tag {
                        TAG_STRING_REF | TAG_STRING_INLINE => {
                            // Back up one byte so decode_string can read the tag
                            self.pos -= 1;
                            self.decode_string()?
                        }
                        TAG_INT => {
                            let bytes = self.read_bytes(4)?;
                            i32_from_le(bytes).to_string()
                        }
                        TAG_BOOL => {
                            let b = self.read_u8()?;
                            (b != 0).to_string()
                        }
                        _ => return Err(BatomError::InvalidTag(key_tag)),
                    };
                    let val = self.decode_value()?;
                    obj.set(key_str, val);
                }
                Ok(Value::Obj(obj))
            }

            TAG_PAIR => {
                let key_type = self.read_u8()?;
                let key = match key_type {
                    0 => {
                        let s = self.decode_string()?;
                        auto_val::ValueKey::Str(s.into())
                    }
                    1 => {
                        let bytes = self.read_bytes(4)?;
                        auto_val::ValueKey::Int(i32_from_le(bytes))
                    }
                    2 => {
                        let b = self.read_u8()?;
                        auto_val::ValueKey::Bool(b != 0)
                    }
                    _ => return Err(BatomError::InvalidTag(key_type)),
                };
                let value = self.decode_value()?;
                Ok(Value::Pair(key, Box::new(value)))
            }

            TAG_GRID => {
                // Headers (each header = key + value pair)
                let header_count = self.read_u32()? as usize;
                let mut head = Vec::with_capacity(header_count);
                for _ in 0..header_count {
                    // Decode key (could be string or int)
                    let key_tag = self.read_u8()?;
                    let key = match key_tag {
                        TAG_STRING_REF | TAG_STRING_INLINE => {
                            self.pos -= 1;
                            let s = self.decode_string()?;
                            auto_val::ValueKey::Str(s.into())
                        }
                        TAG_INT => {
                            let bytes = self.read_bytes(4)?;
                            auto_val::ValueKey::Int(i32_from_le(bytes))
                        }
                        _ => return Err(BatomError::InvalidTag(key_tag)),
                    };
                    let val = self.decode_value()?;
                    head.push((key, val));
                }
                // Rows
                let row_count = self.read_u32()? as usize;
                let mut data = Vec::with_capacity(row_count.min(256));
                for _ in 0..row_count {
                    // Each row has header_count cells
                    let mut row = Vec::with_capacity(header_count);
                    for _ in 0..header_count {
                        row.push(self.decode_value()?);
                    }
                    data.push(row);
                }
                Ok(Value::Grid(auto_val::Grid { head, data }))
            }

            TAG_RANGE => {
                let a = i32_from_le(self.read_bytes(4)?);
                let b = i32_from_le(self.read_bytes(4)?);
                Ok(Value::Range(a, b))
            }

            TAG_RANGE_EQ => {
                let a = i32_from_le(self.read_bytes(4)?);
                let b = i32_from_le(self.read_bytes(4)?);
                Ok(Value::RangeEq(a, b))
            }

            TAG_SOME => {
                let v = self.decode_value()?;
                Ok(Value::Some(Box::new(v)))
            }

            TAG_NONE => Ok(Value::None),

            TAG_OK => {
                let v = self.decode_value()?;
                Ok(Value::Ok(Box::new(v)))
            }

            TAG_ERR => {
                let s = self.decode_string()?;
                Ok(Value::Err(s.into()))
            }

            TAG_ERROR => {
                let s = self.decode_string()?;
                Ok(Value::Error(s.into()))
            }

            _ => Err(BatomError::InvalidTag(tag)),
        }
    }

    // ── Primitive readers ──────────────────────────────

    /// Decode a string that was encoded via `encode_string`.
    /// Reads the tag byte first, then handles ref vs inline.
    fn decode_string(&mut self) -> Result<String, BatomError> {
        let tag = self.read_u8()?;
        match tag {
            TAG_STRING_REF => {
                let bytes = self.read_bytes(2)?;
                let idx = u16_from_le(bytes) as usize;
                if idx >= self.string_table.len() {
                    return Err(BatomError::StringIndexOutOfRange {
                        index: idx,
                        max: self.string_table.len(),
                    });
                }
                Ok(self.string_table[idx].clone())
            }
            TAG_STRING_INLINE => {
                let bytes = self.read_bytes(2)?;
                let len = u16_from_le(bytes) as usize;
                let str_bytes = self.read_bytes(len)?;
                Ok(String::from_utf8_lossy(str_bytes).to_string())
            }
            _ => Err(BatomError::InvalidTag(tag)),
        }
    }

    fn read_u8(&mut self) -> Result<u8, BatomError> {
        if self.pos + 1 > self.data.len() {
            return Err(BatomError::UnexpectedEof(self.pos));
        }
        let v = self.data[self.pos];
        self.pos += 1;
        Ok(v)
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], BatomError> {
        if self.pos + len > self.data.len() {
            return Err(BatomError::UnexpectedEof(self.pos));
        }
        let slice = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(slice)
    }

    fn read_u32(&mut self) -> Result<u32, BatomError> {
        let bytes = self.read_bytes(4)?;
        Ok(u32_from_le(bytes))
    }
}

// ── Convenience functions ──────────────────────────────────

/// Encode an Atom to Batom binary.
pub fn encode_atom(atom: &Atom) -> Result<Vec<u8>, BatomError> {
    BatomEncoder::new().encode_atom(atom)
}

/// Decode a Batom binary blob to an Atom.
pub fn decode_atom(data: &[u8]) -> Result<Atom, BatomError> {
    BatomDecoder::new(data).decode_atom()
}

/// Encode an AtomPipeline to Batom binary.
pub fn encode_pipeline(pipeline: &AtomPipeline) -> Result<Vec<u8>, BatomError> {
    BatomEncoder::new().encode_pipeline(pipeline)
}

/// Decode a Batom binary blob to an AtomPipeline.
pub fn decode_pipeline(data: &[u8]) -> Result<AtomPipeline, BatomError> {
    BatomDecoder::new(data).decode_pipeline()
}

/// Encode an AtomStream to Batom binary.
pub fn encode_stream(stream: &AtomStream) -> Result<Vec<u8>, BatomError> {
    BatomEncoder::new().encode_stream(stream)
}

/// Decode a Batom binary blob to an AtomStream.
pub fn decode_stream(data: &[u8]) -> Result<AtomStream, BatomError> {
    BatomDecoder::new(data).decode_stream()
}

// ── Atom type ↔ u8 conversion ─────────────────────────────

fn atom_type_to_u8(at: AtomType) -> u8 {
    match at {
        AtomType::Nothing => ATOM_NOTHING,
        AtomType::FileEntry => ATOM_FILE_ENTRY,
        AtomType::FileList => ATOM_FILE_LIST,
        AtomType::ProcessEntry => ATOM_PROCESS_ENTRY,
        AtomType::ProcessList => ATOM_PROCESS_LIST,
        AtomType::DiskEntry => ATOM_DISK_ENTRY,
        AtomType::CpuInfo => ATOM_CPU_INFO,
        AtomType::MemoryInfo => ATOM_MEMORY_INFO,
        AtomType::SystemInfo => ATOM_SYSTEM_INFO,
        AtomType::MatchList => ATOM_MATCH_LIST,
        AtomType::CountResult => ATOM_COUNT_RESULT,
        AtomType::Table => ATOM_TABLE,
        AtomType::Record => ATOM_RECORD,
        AtomType::Text => ATOM_TEXT,
        AtomType::Path => ATOM_PATH,
        AtomType::BuildResult => ATOM_BUILD_RESULT,
        AtomType::RunResult => ATOM_RUN_RESULT,
        AtomType::HelpInfo => ATOM_HELP_INFO,
    }
}

fn u8_to_atom_type(tag: u8) -> Result<AtomType, BatomError> {
    match tag {
        ATOM_NOTHING => Ok(AtomType::Nothing),
        ATOM_FILE_ENTRY => Ok(AtomType::FileEntry),
        ATOM_FILE_LIST => Ok(AtomType::FileList),
        ATOM_PROCESS_ENTRY => Ok(AtomType::ProcessEntry),
        ATOM_PROCESS_LIST => Ok(AtomType::ProcessList),
        ATOM_DISK_ENTRY => Ok(AtomType::DiskEntry),
        ATOM_CPU_INFO => Ok(AtomType::CpuInfo),
        ATOM_MEMORY_INFO => Ok(AtomType::MemoryInfo),
        ATOM_SYSTEM_INFO => Ok(AtomType::SystemInfo),
        ATOM_MATCH_LIST => Ok(AtomType::MatchList),
        ATOM_COUNT_RESULT => Ok(AtomType::CountResult),
        ATOM_TABLE => Ok(AtomType::Table),
        ATOM_RECORD => Ok(AtomType::Record),
        ATOM_TEXT => Ok(AtomType::Text),
        ATOM_PATH => Ok(AtomType::Path),
        ATOM_BUILD_RESULT => Ok(AtomType::BuildResult),
        ATOM_RUN_RESULT => Ok(AtomType::RunResult),
        ATOM_HELP_INFO => Ok(AtomType::HelpInfo),
        _ => Err(BatomError::InvalidAtomType(tag)),
    }
}

// ── Little-endian read helpers ─────────────────────────────

fn u32_from_le(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn i32_from_le(bytes: &[u8]) -> i32 {
    i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn u16_from_le(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([bytes[0], bytes[1]])
}

fn i64_from_le(bytes: &[u8]) -> i64 {
    let arr: [u8; 8] = bytes.try_into().unwrap_or([0; 8]);
    i64::from_le_bytes(arr)
}

fn u64_from_le(bytes: &[u8]) -> u64 {
    let arr: [u8; 8] = bytes.try_into().unwrap_or([0; 8]);
    u64::from_le_bytes(arr)
}

fn f64_from_le(bytes: &[u8]) -> f64 {
    let arr: [u8; 8] = bytes.try_into().unwrap_or([0; 8]);
    f64::from_le_bytes(arr)
}

// ── Tests ──────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Primitive roundtrips ───────────────────────────

    #[test]
    fn test_nil_roundtrip() {
        let atom = Atom::nothing(Value::Nil);
        let bytes = encode_atom(&atom).unwrap();
        let decoded = decode_atom(&bytes).unwrap();
        assert!(matches!(decoded.value, Value::Nil));
        assert_eq!(decoded.atom_type, AtomType::Nothing);
    }

    #[test]
    fn test_void_roundtrip() {
        let atom = Atom::empty();
        let bytes = encode_atom(&atom).unwrap();
        let decoded = decode_atom(&bytes).unwrap();
        assert!(matches!(decoded.value, Value::Void));
    }

    #[test]
    fn test_bool_roundtrip() {
        for b in [true, false] {
            let atom = Atom::new(Value::Bool(b), AtomType::Nothing);
            let bytes = encode_atom(&atom).unwrap();
            let decoded = decode_atom(&bytes).unwrap();
            assert_eq!(decoded.value.as_bool(), b);
        }
    }

    #[test]
    fn test_int_roundtrip() {
        for i in [0, 1, -1, i32::MIN, i32::MAX] {
            let atom = Atom::new(Value::Int(i), AtomType::CountResult);
            let bytes = encode_atom(&atom).unwrap();
            let decoded = decode_atom(&bytes).unwrap();
            assert_eq!(decoded.value.as_int(), i);
            assert_eq!(decoded.atom_type, AtomType::CountResult);
        }
    }

    #[test]
    fn test_i64_roundtrip() {
        for i in [0i64, -1, i64::MIN, i64::MAX] {
            let atom = Atom::new(Value::I64(i), AtomType::Nothing);
            let bytes = encode_atom(&atom).unwrap();
            let decoded = decode_atom(&bytes).unwrap();
            match decoded.value {
                Value::I64(v) => assert_eq!(v, i),
                _ => panic!("expected I64, got {:?}", decoded.value),
            }
        }
    }

    #[test]
    fn test_uint_roundtrip() {
        for u in [0u32, 1, u32::MAX] {
            let atom = Atom::new(Value::Uint(u), AtomType::Nothing);
            let bytes = encode_atom(&atom).unwrap();
            let decoded = decode_atom(&bytes).unwrap();
            assert_eq!(decoded.value.as_uint(), u);
        }
    }

    #[test]
    fn test_float_roundtrip() {
        for f in [0.0f64, 1.5, -3.14, f64::MAX, f64::MIN] {
            let atom = Atom::new(Value::Float(f), AtomType::Nothing);
            let bytes = encode_atom(&atom).unwrap();
            let decoded = decode_atom(&bytes).unwrap();
            // Verify by matching the variant
            match decoded.value {
                Value::Float(d) => assert_eq!(d, f),
                _ => panic!("expected Float, got {:?}", decoded.value),
            }
        }
    }

    #[test]
    fn test_string_roundtrip() {
        for s in ["", "hello", "你好世界", "emoji 🎉"] {
            let atom = Atom::text(s);
            let bytes = encode_atom(&atom).unwrap();
            let decoded = decode_atom(&bytes).unwrap();
            assert_eq!(decoded.value.as_str(), s);
            assert_eq!(decoded.atom_type, AtomType::Text);
        }
    }

    #[test]
    fn test_byte_roundtrip() {
        for b in [0u8, 127, 255] {
            let atom = Atom::new(Value::Byte(b), AtomType::Nothing);
            let bytes = encode_atom(&atom).unwrap();
            let decoded = decode_atom(&bytes).unwrap();
            assert_eq!(decoded.value.as_byte(), b);
        }
    }

    // ── Complex roundtrips ─────────────────────────────

    #[test]
    fn test_array_roundtrip() {
        let arr: Vec<Value> = vec![Value::Int(1), Value::Int(2), Value::Int(3)];
        let atom = Atom::file_list(Value::Array(arr.into()));
        let bytes = encode_atom(&atom).unwrap();
        let decoded = decode_atom(&bytes).unwrap();
        assert_eq!(decoded.atom_type, AtomType::FileList);
        let arr = decoded.value.as_array();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_obj_roundtrip() {
        let mut obj = auto_val::Obj::new();
        obj.set("name", Value::str("test.txt"));
        obj.set("size", Value::Int(1024));
        let atom = Atom::new(Value::Obj(obj), AtomType::FileEntry);
        let bytes = encode_atom(&atom).unwrap();
        let decoded = decode_atom(&bytes).unwrap();
        assert_eq!(decoded.atom_type, AtomType::FileEntry);
        let obj = decoded.value.as_obj();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get("name").unwrap().as_str(), "test.txt");
    }

    #[test]
    fn test_range_roundtrip() {
        let atom = Atom::new(Value::Range(0, 10), AtomType::Nothing);
        let bytes = encode_atom(&atom).unwrap();
        let decoded = decode_atom(&bytes).unwrap();
        assert!(matches!(decoded.value, Value::Range(0, 10)));
    }

    #[test]
    fn test_some_none_roundtrip() {
        let atom = Atom::new(Value::Some(Box::new(Value::Int(42))), AtomType::Nothing);
        let bytes = encode_atom(&atom).unwrap();
        let decoded = decode_atom(&bytes).unwrap();
        match decoded.value {
            Value::Some(v) => assert_eq!(v.as_int(), 42),
            _ => panic!("expected Some"),
        }

        let atom = Atom::new(Value::None, AtomType::Nothing);
        let bytes = encode_atom(&atom).unwrap();
        let decoded = decode_atom(&bytes).unwrap();
        assert!(matches!(decoded.value, Value::None));
    }

    #[test]
    fn test_ok_err_roundtrip() {
        let atom = Atom::new(Value::Ok(Box::new(Value::Int(1))), AtomType::BuildResult);
        let bytes = encode_atom(&atom).unwrap();
        let decoded = decode_atom(&bytes).unwrap();
        assert_eq!(decoded.atom_type, AtomType::BuildResult);
        match decoded.value {
            Value::Ok(v) => assert_eq!(v.as_int(), 1),
            _ => panic!("expected Ok"),
        }

        let atom = Atom::new(Value::Err("something went wrong".into()), AtomType::Nothing);
        let bytes = encode_atom(&atom).unwrap();
        let decoded = decode_atom(&bytes).unwrap();
        match decoded.value {
            Value::Err(s) => assert_eq!(s.as_str(), "something went wrong"),
            _ => panic!("expected Err"),
        }
    }

    // ── Nested structures ──────────────────────────────

    #[test]
    fn test_nested_array_of_objects() {
        // Simulates ls output: array of file entries
        let mut items = Vec::new();
        for i in 0..5 {
            let mut entry = auto_val::Obj::new();
            entry.set("name", Value::str(&format!("file{}.txt", i)));
            entry.set("size", Value::Int(i * 100));
            entry.set("type", Value::str("file"));
            items.push(Value::Obj(entry));
        }
        let atom = Atom::file_list(Value::Array(items.into()));
        let bytes = encode_atom(&atom).unwrap();
        let decoded = decode_atom(&bytes).unwrap();
        assert_eq!(decoded.atom_type, AtomType::FileList);
        let arr = decoded.value.as_array();
        assert_eq!(arr.len(), 5);
    }

    #[test]
    fn test_string_dedup() {
        // Many entries with the same string values
        let mut items = Vec::new();
        for _ in 0..100 {
            let mut entry = auto_val::Obj::new();
            entry.set("type", Value::str("file")); // repeated
            entry.set("status", Value::str("ok")); // repeated
            items.push(Value::Obj(entry));
        }
        let atom = Atom::file_list(Value::Array(items.into()));
        let bytes = encode_atom(&atom).unwrap();
        // Should be much smaller than without dedup
        // 100 entries × (2 strings × overhead) would be huge without dedup
        assert!(bytes.len() < 2000, "expected dedup to reduce size, got {} bytes", bytes.len());

        let decoded = decode_atom(&bytes).unwrap();
        let arr = decoded.value.as_array();
        assert_eq!(arr.len(), 100);
    }

    // ── All AtomType tags ──────────────────────────────

    #[test]
    fn test_all_atom_types_roundtrip() {
        let types = [
            AtomType::Nothing, AtomType::FileEntry, AtomType::FileList,
            AtomType::ProcessEntry, AtomType::ProcessList,
            AtomType::DiskEntry, AtomType::CpuInfo, AtomType::MemoryInfo,
            AtomType::SystemInfo, AtomType::MatchList, AtomType::CountResult,
            AtomType::Table, AtomType::Record, AtomType::Text, AtomType::Path,
            AtomType::BuildResult, AtomType::RunResult, AtomType::HelpInfo,
        ];
        for at in types {
            let atom = Atom::new(Value::Int(1), at);
            let bytes = encode_atom(&atom).unwrap();
            let decoded = decode_atom(&bytes).unwrap();
            assert_eq!(decoded.atom_type, at, "AtomType {:?} roundtrip failed", at);
        }
    }

    // ── Pipeline roundtrips ────────────────────────────

    #[test]
    fn test_pipeline_atom_roundtrip() {
        let pipeline = AtomPipeline::atom(Value::Int(42), AtomType::CountResult);
        let bytes = encode_pipeline(&pipeline).unwrap();
        let decoded = decode_pipeline(&bytes).unwrap();
        assert!(decoded.is_atom());
        assert_eq!(decoded.atom_type(), AtomType::CountResult);
    }

    #[test]
    fn test_pipeline_text_roundtrip() {
        let pipeline = AtomPipeline::text("hello world");
        let bytes = encode_pipeline(&pipeline).unwrap();
        let decoded = decode_pipeline(&bytes).unwrap();
        assert_eq!(decoded.as_text(), "hello world");
    }

    #[test]
    fn test_pipeline_empty_roundtrip() {
        let pipeline = AtomPipeline::empty();
        let bytes = encode_pipeline(&pipeline).unwrap();
        let decoded = decode_pipeline(&bytes).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_stream_roundtrip() {
        let atoms: Vec<Atom> = (0..5)
            .map(|i| Atom::new(Value::Int(i), AtomType::FileEntry))
            .collect();
        let stream = AtomStream::new(atoms);
        let bytes = encode_stream(&stream).unwrap();
        let decoded = decode_stream(&bytes).unwrap();
        assert_eq!(decoded.total_count(), 5);
    }

    // ── Error cases ────────────────────────────────────

    #[test]
    fn test_invalid_magic() {
        let data = vec![0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let result = decode_atom(&data);
        assert!(result.is_err());
        match result.unwrap_err() {
            BatomError::InvalidMagic(m) => assert_eq!(m, [0xFF, 0xFF, 0xFF, 0xFF]),
            e => panic!("expected InvalidMagic, got {:?}", e),
        }
    }

    #[test]
    fn test_truncated_data() {
        let atom = Atom::new(Value::Int(42), AtomType::Nothing);
        let bytes = encode_atom(&atom).unwrap();
        let truncated = &bytes[..10]; // Too short
        let result = decode_atom(truncated);
        assert!(result.is_err());
    }

    // ── Binary format verification ─────────────────────

    #[test]
    fn test_magic_at_start() {
        let atom = Atom::text("test");
        let bytes = encode_atom(&atom).unwrap();
        assert_eq!(&bytes[0..4], b"BATM");
    }

    #[test]
    fn test_version_is_one() {
        let atom = Atom::text("test");
        let bytes = encode_atom(&atom).unwrap();
        assert_eq!(bytes[4], 1); // version = 1
    }
}
