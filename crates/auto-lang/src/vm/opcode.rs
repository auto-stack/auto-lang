#![allow(non_camel_case_types)]

/// AutoByteCode (ABC) OpCode Definitions
/// Based on docs/design/abc.md

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    // === Stack Manipulation ===
    NOP = 0x00,
    POP = 0x01,
    POP_N = 0x02,
    DUP = 0x03,
    SWAP = 0x04,
    DROP = 0x05, // RAII cleanup: pops and frees owned value

    // === Constants ===
    CONST_I32 = 0x10,
    CONST_U8 = 0x11,
    CONST_0 = 0x12,
    CONST_1 = 0x13,
    CONST_F32 = 0x14,
    CONST_F64 = 0x15,     // Plan 073: Double precision
    CONST_I64 = 0x16,     // Plan 073: 64-bit integer
    CONST_U64 = 0x17,     // Plan 073: 64-bit unsigned
    LOAD_STR = 0x1F,
    // Plan 075: Object field manipulation
    SET_FIELD = 0x2A,     // Plan 075: Set field on object (value, field_str_idx) -> void
    SET_ELEM = 0x2B,      // Plan 073: Set element in array (array_id, index, value) -> void
    GET_ELEM = 0x2C,      // Plan 073: Get element from array (array_id, index) -> value
    GET_FIELD = 0x2D,     // Plan 073: Get field from object (obj_id, field_str_idx) -> value
    CREATE_OBJ = 0x2E,    // Plan 073: Create object from field_count -> object_id
    CREATE_ARRAY = 0x2F,   // Plan 073: Create array from elem_count -> array_id
    CREATE_RANGE = 0x75,  // Plan 073: Create exclusive range (0..10) from (start, end) -> range_value
    CREATE_RANGE_EQ = 0x76, // Plan 073: Create inclusive range (0..=10) from (start, end) -> range_value
    BUILD_FSTR = 0x77,    // Plan 073: Build f-string from part_count -> string
    NULL_COALESCE = 0x78, // Plan 073: May<T> null coalesce: ?? operator (left ?? right) -> value
    ERROR_PROPAGATE = 0x79, // Plan 073: May<T> error propagate: .? operator (expr.?) -> unwrapped_value
    CREATE_NODE = 0x74,   // Plan 073: Create node from name_str_idx, arg_count -> node_id (changed from 0x30 to avoid conflict with ADD)
    // Plan 075: Template string opcodes
    TO_STR = 0x7A,        // Convert any value to string
    IS_NIL = 0x7B,        // Check if value is nil (returns 1 if nil, 0 otherwise)
    STR_CAT = 0x7C,       // Concatenate two strings (optimized string joining)

    // === Local Variables ===
    LOAD_LOCAL = 0x20,
    STORE_LOCAL = 0x21,
    LOAD_LOC_0 = 0x22,
    LOAD_LOC_1 = 0x23,
    LOAD_LOC_2 = 0x24,
    STORE_LOC_0 = 0x25,
    STORE_LOC_1 = 0x26,

    // === Arithmetic & Logic ===
    ADD = 0x30,
    SUB = 0x31,
    MUL = 0x32,
    DIV = 0x33,
    MOD = 0x34,
    NEG = 0x35,

    // Plan 073: Floating-point arithmetic
    ADD_F = 0x36,     // f32 + f32 -> f32
    SUB_F = 0x37,     // f32 - f32 -> f32
    MUL_F = 0x38,     // f32 * f32 -> f32
    DIV_F = 0x39,     // f32 / f32 -> f32
    NEG_F = 0x3A,     // -f32 -> f32

    // Plan 073: Double precision arithmetic
    ADD_D = 0x3B,     // f64 + f64 -> f64
    SUB_D = 0x3C,     // f64 - f64 -> f64
    MUL_D = 0x3D,     // f64 * f64 -> f64
    DIV_D = 0x3E,     // f64 / f64 -> f64
    NEG_D = 0x3F,     // -f64 -> f64

    AND = 0x40,
    OR = 0x41,
    XOR = 0x42,
    NOT = 0x43,
    SHL = 0x44,
    SHR = 0x45,

    // === Comparison ===
    EQ = 0x50,
    NE = 0x51,
    LT = 0x52,
    GT = 0x53,
    LE = 0x54,
    GE = 0x55,

    // === Control Flow ===
    JMP = 0x60,
    JMP_IF_Z = 0x61,
    JMP_IF_NZ = 0x62,
    JMP_L = 0x63,

    // === Function Call ===
    CALL = 0x70,
    RET = 0x71,
    CALL_NAT = 0x72,

    // === Concurrency ===
    SPAWN = 0x80,    // func_id: u32, arg_count: u8 -> task_id: u32
    TASK_ID = 0x81,  // -> task_id: u32
    YIELD = 0x82,    // -> void
    SLEEP = 0x83,    // ms: u32 -> void
    JOIN = 0x84,     // task_id: u32 -> result
    CHAN_NEW = 0x85, // -> channel_id: u32
    SEND = 0x86,     // channel_id: u32, data: i32 -> void
    RECV = 0x87,     // channel_id: u32 -> data: i32
    TRY_RECV = 0x88, // channel_id: u32 -> data: i32 | 0 (non-blocking)

    // === Closures (Plan 071: Direct Capture) ===
    CLOSURE = 0x90,         // func_addr, capture_count × value -> closure_id: u32
    CAPTURE_VAR = 0x91,     // -> value (load variable by name)
    LOAD_CAPTURED = 0x92,   // closure_id -> value (load captured var by name)
    STORE_CAPTURED = 0x93,  // closure_id, value -> (store captured var by name)
    CALL_CLOSURE = 0x94,    // closure_id -> (call closure with captured env)

    // === Plan 076 Phase 3: Generic List Opcodes ===
    // Type-specific list operations for monomorphized generics
    CREATE_LIST_INT = 0xA0,     // -> list_id (create List<int> with Heap storage)
    CREATE_LIST_STR = 0xA1,     // -> list_id (create List<string> with Heap storage)
    CREATE_LIST_BOOL = 0xA2,    // -> list_id (create List<bool> with Heap storage)
    LIST_PUSH_INT = 0xA3,       // list_id, value: int -> void
    LIST_POP_INT = 0xA4,        // list_id -> int
    LIST_GET_INT = 0xA5,        // list_id, index: int -> int
    LIST_SET_INT = 0xA6,        // list_id, index: int, value: int -> void

    // === Plan 076 Phase 4: Storage Strategy Opcodes ===
    // InlineInt64 storage variants (fixed 64-element capacity, no heap)
    CREATE_LIST_INT_INLINE = 0xA7,  // -> list_id (create List<int> with InlineInt64 storage)
    CREATE_LIST_STR_INLINE = 0xA8,  // -> list_id (create List<string> with InlineInt64 storage)
    CREATE_LIST_BOOL_INLINE = 0xA9, // -> list_id (create List<bool> with InlineInt64 storage)

    // === Debug ===
    PRINT = 0xF0,
    HALT = 0xFF,
}

impl From<u8> for OpCode {
    fn from(v: u8) -> Self {
        unsafe { std::mem::transmute(v) }
    }
}
